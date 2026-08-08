# ADR-0326: Vault audience-sequenced replay fencing

- Статус: принято
- Дата: 2026-07-29
- Состояние реализации: реализовано в production packages и доказано live
  generation 83; общий `make pre-push` остаётся integration gate полного
  переноса
- Связанные решения: ADR-0215, ADR-0223, ADR-0257, ADR-0314, ADR-0325

## Контекст

Live iCloud backfill и последующая Blob custody выявили конечный operational
failure существующего Vault transport:

- private replay guard сохраняет каждый authenticated random request ID;
- после 1024 HPKE requests текущая Vault generation навсегда отвечает
  `SessionCapacityExceeded`;
- обычная обработка Mail body/attachment evidence быстро исчерпывает этот
  лимит;
- Blob content-key operations, Communications custody и независимые provider
  credential routes после этого становятся недоступны;
- очистить старые random IDs нельзя: старый authenticated `IssueLease` frame
  снова станет допустимым replay.

Увеличение лимита только отложит отказ. LRU/TTL для random IDs ослабит replay
fencing. Mail, Blob или Communications не должны управлять Vault restart либо
его private replay cache.

## Решение

### Public request ID contract

`VaultTransportBindingV1.request_id` сохраняет длину 16 bytes, но получает
exact V1 encoding:

```text
bytes 0..8   = cryptographically random process stream ID, stream ID > 0
bytes 8..16  = unsigned big-endian sequence, sequence > 0
```

Owner-neutral `makosh-runtime-protocol`, который уже владеет opaque
`VaultCiphertextRouteV1`, владеет transport request ID:

- один random stream ID на process lifetime;
- process-local monotonic sequence allocation внутри stream;
- encoding exact request ID;
- decoding and validation exact request ID.

Managed Vault clients и Kernel platform Vault routes не создают random IDs
самостоятельно. Один process использует один stream allocator для всех своих
Vault callers. Stream randomness или sequence exhaustion fail-closes; sequence
не wraps.

Один exact audience может законно посылать commands через несколько процессов:
module runtime и Kernel-owned platform route. Отдельный authenticated stream ID
не даёт их process-local sequences конфликтовать и не требует central counter
в Kernel.

Predictability request ID не даёт authority: HPKE authentication, exact
operation digest, response recipient key, audience, Vault generation и current
Kernel/runtime/grant fences остаются обязательными.

### Private replay guard

Vault runtime хранит не множество всех requests, а bounded map:

```text
(exact LeaseAudienceV1, process stream ID) -> highest accepted sequence
```

Для authenticated request:

1. проверить Vault generation и `ToVault` direction;
2. HPKE-open и проверить operation digest;
3. decode exact V1 request sequence;
4. найти exact audience + process stream;
5. принять sequence только если он строго больше сохранённого;
6. записать новый high-watermark до выполнения command.

Повторный, старый или out-of-order sequence получает `ReplayDetected`.
Нулевой stream ID или sequence получает `InvalidBinding`.

Map ограничен 1024 distinct audience-stream pairs на одну Vault runtime
generation. Это лимит одновременно наблюдавшихся fenced process streams, а не
число операций. Новый pair сверх лимита получает `SessionCapacityExceeded`.
Vault restart создаёт новую HPKE key/generation и пустой map; старые frames
после этого отклоняются generation/key binding.

### Ordering contract

Один process stream посылает Vault commands последовательно через свой
inherited managed-control route. Parallel callers одного stream обязаны
сериализоваться до allocation/route. Transport не обещает принимать
out-of-order commands одного stream: это сохраняет deterministic single-writer
semantics Vault. Другой process с тем же audience использует другой stream и
независимый high-watermark.

Module restart меняет runtime instance/generation; новый audience начинает
с любого положительного global process sequence. Grant epoch входит в audience,
поэтому successor/revoke fencing не переиспользует predecessor watermark.

### Ownership и единицы сборки

- `makosh-runtime-protocol` владеет opaque request ID
  encoding/allocation contract рядом с ciphertext route framing;
- `makosh-vault-protocol` сохраняет authority над HPKE binding, command и
  lease semantics, но не вводит обратную зависимость transport callers;
- `makosh-managed-vault-client` и `makosh-storage-vault` только используют
  canonical allocator;
- `makosh-vault-runtime` владеет private audience high-watermark map;
- Kernel остаётся blind ciphertext router и не интерпретирует sequence;
- Blob, Communications, Mail и Attachment Security не импортируют Vault
  implementation и не получают replay-cache APIs;
- assembly units только связывают уже admitted artifacts.

Integration не становится platform. Domain не управляет Vault. Runtime не
становится assembly.

## Admission evidence

Решение считается реализованным только с:

1. protocol tests exact marker, big-endian sequence, non-zero и exhaustion;
2. runtime tests repeated/lower/out-of-order rejection и increasing acceptance;
3. runtime test более 1024 requests одного audience-stream без capacity
   failure;
4. runtime test одного audience с независимыми process streams;
5. runtime test 1025 distinct audience-stream pairs с fail-closed capacity;
6. client tests, что managed и platform routes используют canonical IDs;
7. architecture guard против private random Vault request-ID generators;
8. live evidence без `SessionCapacityExceeded`/cross-process
   `ReplayDetected` при Mail materialization,
   Blob custody и Communications canonical processing;
9. `make pre-push`.

## Последствия

Vault replay memory теперь зависит от числа fenced audience/process streams, а
не от числа обычных secret/blob operations. Старые authenticated frames остаются
непригодными для повторного исполнения. Transport contract становится
строже: самостоятельно сгенерированный random request ID больше не admitted.

External modules должны использовать canonical public protocol allocator.
Compatibility facade для random IDs не вводится.
