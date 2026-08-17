# ADR-0413: Chunked Blob storage and session-bound client range delivery

Статус: Принято
Дата: 2026-08-17
Состояние реализации: Реализовано в source и focused tests, но ещё не принято
live desktop phase gate. Blob runtime пишет receipt-bound объекты
последовательными чанками, Core Gateway выдаёт session-bound range lease, а
Telegram video/audio frontend использует HTTP Range. Нужны пересборка ensemble,
реальная Telegram-авторизация и проверка воспроизведения до заявления live done.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0257: event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0412: frontend account lanes](ADR-0412-frontend-account-lanes-over-shared-client-realtime.md).

## Контекст

Формат `HBLBENC2` шифровал один Blob единым AEAD ciphertext. Любое
`read_range` читало и расшифровывало файл целиком, а Telegram runtime сначала
загружал TDLib-файл целиком в память. Gateway `client_blob` возвращал только
цельный POST response до 32 MiB. Такое поведение не является range delivery и
создаёт общий memory/latency bottleneck для provider runtime и клиента.

Telegram допускает получение файлов до 4 GiB, поэтому operational client не
может считать 32/64 MiB полным provider surface. При этом повышение лимита
цельнофайлового frame увеличило бы риск исчерпания памяти и не исправило бы
архитектуру.

## Решение

### Storage format

Новые chunked writes используют `HBLBENC3`:

- plaintext chunk равен 1 MiB, кроме последнего;
- каждый chunk имеет независимый XChaCha20-Poly1305 nonce/tag;
- AAD связывает chunk с полным `BlobRefV1`, custody scope, key revision,
  plaintext offset и length;
- staging принимает только точные последовательные offsets;
- отмена незавершённой загрузки использует отдельный idempotent cleanup request
  под свежим one-use WRITE grant и освобождает ciphertext staging вместе с
  quota reservation до перехода очереди к следующему файлу;
- final chunk публикует `.blob` только после потоковой SHA-256 проверки полного
  plaintext receipt и atomic rename;
- `HBLBENC2` остаётся читаемым для обратной совместимости;
- range read читает и расшифровывает только пересекающиеся `HBLBENC3` chunks.

Каждый chunk использует новую одноразовую Kernel-signed WRITE session grant.
Capability operation не расширяется: chunking является bounded transport
реализацией существующего `BlobDataOperationWriteV1`.

### Provider ingestion

Telegram TDLib-файл хешируется bounded 1 MiB buffer в одном отдельном
ограниченном worker thread. Account/realtime loop не ждёт полного hash или
полной загрузки: он передаёт в Blob runtime не больше одного 1 MiB chunk за
итерацию, а следующие файлы остаются в последовательной deduplicated queue.
Повторяемая platform-ошибка получает bounded backoff и предел попыток. Provider
bytes, filename, account/chat identity и receipt не логируются. Platform object
ceiling равен 4 GiB; Telegram custody quota равна 64 GiB и остаётся aggregate
owner-local budget, а не обещанием бесконечного локального cache.

### Client range lease

Descriptor-declared `client_blob` route сохраняет POST authorization request.
Для `range-v1` Gateway:

1. повторно авторизует текущую browser owner/device/session и provider module;
2. читает один authenticated probe byte;
3. создаёт random opaque in-memory lease без provider identifiers в URL;
4. на каждом GET повторно проверяет ту же browser session;
5. принимает только один byte range и ограничивает response 4 MiB;
6. получает новый one-use Blob READ_RANGE grant и возвращает `206`,
   `Content-Range`, `Accept-Ranges` и sanitized media type.

Lease истекает после часа бездействия, продлевается только запросом той же
authenticated browser session и исчезает при рестарте Gateway. Один Gateway
держит не больше 4096 активных lease. Inline response остаётся ограничен 32
MiB; объект больше лимита обязан использовать range lease. URL не является
durable reference, не передаётся через SSE/events и не содержит BlobRef,
account ID или provider file ID.

## Phase gate `chunked_client_blob_range_v1`

Gate закрывается только при наличии:

1. storage test через границу двух encrypted chunks;
2. rejection неправильного offset, final size и receipt;
3. совместимого чтения существующего `HBLBENC2`;
4. one-use Kernel grant на каждый write/read chunk;
5. browser range test с session mismatch, expired/unknown lease и invalid range;
6. доказательства, что inline path не читает объект больше 32 MiB целиком;
7. live Telegram video/audio playback с seek и повторным переключением чатов;
8. payload-safe spans без URL token, identifiers и content.

Source tests покрывают пункты 1–4, unknown lease, bounded single/suffix range и
доказательство одно-байтового probe перед выбором inline/stream path. Отдельные
session-mismatch/expiry tests, live пункты 7–8 и полный phase-gate прогон
остаются обязательными до закрытия gate.

## Отклонённые варианты

### Просто поднять цельнофайловый лимит

Отклонено: сохраняет allocation/decrypt bottleneck и позволяет одному файлу
заблокировать provider/client lane.

### Provider URL или filesystem path во frontend

Отклонено: обходит Core Gateway, browser session, capability grants, Vault key
fence и Blob custody.

### Отдельный WebSocket для media

Отклонено: bytes остаются HTTP range data plane; WebSocket не даёт cache/seek
semantics и дублирует authentication/reconnect state.
