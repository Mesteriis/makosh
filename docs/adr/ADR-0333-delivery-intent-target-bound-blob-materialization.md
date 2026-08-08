# ADR-0333: Delivery-intent target-bound Blob materialization

Статус: Принято

Дата: 2026-07-29

Состояние реализации: workflow-owned Blob write adapter, exact provider target
selection, Blob capability admission и receipt-only PostgreSQL persistence
реализованы. Четыре provider runtime custody consumers и provider-owned
command/result relay loops реализованы по ADR-0335. Workflow Event Hub loops,
его live managed admission и полное four-provider runtime evidence остаются
следующими production gates.

## Контекст

ADR-0330 первоначально оставлял тело delivery intent как workflow-local
`SealedDeliveryBodyV1`. Этот тип имел ciphertext, nonce и key epoch, но в
production inventory нет ни разрешённого generic payload-encryption key
contract, ни runtime sealer/unsealer. Такой объект нельзя передать provider
runtime без выдуманного Vault API или скрытого общего ключа.

ADR-0257 и ADR-0275 уже определяют другой исполнимый boundary: source runtime
может записать plaintext в managed Blob, запросить proof, заранее привязанный к
стабильной аудитории `(owner_id, module_id, capability_id)`, и передать только
opaque receipt/proof через durable event. Target runtime проверяет current
registration, runtime generation и grant epoch при custody transfer.

## Решение

`communication_delivery_intent` materializes validated body в Blob до создания
workflow job. Write использует отдельную required capability:

```text
communication_delivery_intent.blob.v1
```

и scope:

```text
communication_delivery_intent.body.v1
```

Source runtime выбирает одну из четырёх exact public integration audiences по
уже разрешённому canonical route:

| Provider | Target owner | Target module | Target Blob capability |
|---|---|---|---|
| Mail | `mail` | `makosh-mail-runtime` | `mail.blob.v1` |
| Telegram | `telegram` | `makosh-telegram-runtime` | `telegram.blob.v1` |
| WhatsApp | `whatsapp` | `makosh-whatsapp-runtime` | `whatsapp.blob.v1` |
| Zulip | `zulip` | `makosh-zulip-runtime` | `zulip.blob.v1` |

Workflow runtime импортирует только четыре integration-owned public contract
units. Он не импортирует provider runtime, storage, SDK или implementation.
Общего provider facade и discriminator payload не вводится.

## Persistence contract

PostgreSQL workflow job хранит:

- deterministic body reference ID;
- declared byte count;
- SHA-256 receipt;
- bounded target-bound custody source proof;
- deterministic request fingerprint;
- canonical route and workflow state.

Plaintext, ciphertext, nonce, key epoch и generic Vault payload key в workflow
database запрещены. Claim возвращает тот же receipt, который без повторного
Blob read передаётся существующему exact provider event adapter.

`communication_delivery_intent_v1` ещё не входил в admitted owner inventory и
его initial storage bundle не применялся production Storage Control. Поэтому
неисполняемая pre-admission initial schema заменяется до phase gate, а
destructive successor migration для никогда не admitted state не создаётся.

Request fingerprint связывает logical owner, intent/canonical identity, exact
route и Blob receipt. Повтор команды с тем же intent ID, но другим body или
route завершается conflict.

## Failure и retry

- Blob write обязан завершиться до PostgreSQL create.
- Reference ID детерминирован по owner, intent ID и body digest, поэтому retry
  запрашивает те же bytes под тем же reference.
- Ошибка Blob/control не создаёт workflow job.
- Ошибка PostgreSQL после успешного Blob write оставляет только owner-local
  deterministic orphan, пригодный для bounded cleanup; она не создаёт
  provider-visible business state.
- Provider offline не блокирует source write: proof привязан к стабильной
  public audience, а current runtime/grant fences проверяются при transfer.

## Supersession

Этот ADR заменяет только неисполняемый `SealedDeliveryBodyV1` persistence choice
из ADR-0330/ADR-0332. Provider-neutral planning, transactional event adapters,
outbox/inbox и четыре provider-owned wire contracts остаются действующими.

## Следующий gate

Для полного executable flow ещё обязательны:

1. четыре provider-owned durable inbox consumers;
2. custody transfer и body read под current provider fences;
3. provider operational command adapters и result outbox;
4. exact JetStream publish/consume permits и relay loops;
5. outage/replay/stale-generation/live managed evidence;
6. Gateway command/status/realtime closure.

До закрытия этих пунктов `communication_delivery_intent_v1` остаётся
`planned`.
