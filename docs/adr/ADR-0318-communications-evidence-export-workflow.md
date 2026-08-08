# ADR-0318: Communications evidence export workflow

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented. Production units, independently managed
  backend contour, authenticated client delivery, live browser flow и полный
  negative/race evidence ниже реализованы.
- Связанные решения: ADR-0200, ADR-0201, ADR-0204, ADR-0205, ADR-0212,
  ADR-0213, ADR-0215, ADR-0220, ADR-0230, ADR-0231, ADR-0240, ADR-0253,
  ADR-0257, ADR-0275, ADR-0279, ADR-0282, ADR-0313, ADR-0314, ADR-0315

## Контекст

Clean-room Communications уже владеет canonical metadata и отдельно
авторизует чтение current message body. Историческая поверхность экспорта
удалена, потому что она не имела принятого owner contract и могла смешивать
provider records, domain state, filesystem download и UI orchestration.

Экспорт evidence не является новой mutation Communications и не принадлежит
Mail, Telegram, WhatsApp или Zulip. Он координирует canonical read, private
content и создание отдельного downloadable artifact, поэтому его owner role —
`workflow`.

Нельзя:

- собирать экспорт в Communications domain implementation;
- читать provider storage или provider operational contracts;
- передавать body bytes через NATS, Kernel control channel, SQL другой unit,
  logs, SSE или errors;
- выдавать browser внутренний `BlobRefV1`, digest, path, grant или source
  receipt;
- превращать Kernel/Gateway в export service или private-content proxy;
- строить artifact только во frontend и считать это backend gate.

## Решение

### Owner и единицы сборки

Создаётся отдельный owner `communications_export` с role `workflow`.
Минимальная package topology:

```text
makosh-communications-evidence-export-source-api
  Communications-owned durable source command/result contract

makosh-communications-export-api
  workflow client command/status/ticket and artifact-read contracts

makosh-communications-export-core
  one export job state machine and deterministic artifact encoder

makosh-communications-export-persistence
  workflow-owned jobs, inbox, outbox and artifact receipt

makosh-communications-export-runtime
  client/event/Blob orchestration and runtime fencing

makosh-communications-export-assembly
  unsigned release inputs for the independently supervised workflow runtime
```

Source API является public contract unit Communications, а не workflow
implementation. Communications runtime реализует этот port, но не импортирует
export core/runtime/persistence. Workflow импортирует только public source API,
Communications query/content contracts и platform Blob contracts.

Как и domain, integration и engine runtimes, independently supervised workflow
runtime может зависеть только от общей platform implementation
`makosh-events-jetstream` для transport adapter Event Hub. Это узкое исключение
не разрешает workflow импортировать Event Hub authority, storage или
owner-specific implementation; business payload остаётся в typed public
contracts.

Package, runtime, workflow, domain и integration остаются разными единицами
сборки. Количество строк не используется как критерий SRP.

### V1 scope

Первая revision экспортирует explicit bounded set из `1..=64` canonical
message IDs одного logical owner. Порядок IDs является частью request и
сохраняется в artifact. Duplicate IDs, zero IDs, mixed/unknown/deleted
messages и запрос с суммарным declared content больше `16 MiB` отклоняются.

Artifact — deterministic UTF-8 JSON Lines:

1. versioned manifest с export ID, logical owner, created time и item count;
2. одна canonical evidence record на message в request order;
3. только provider-neutral canonical metadata из ADR-0313;
4. current admitted UTF-8 body либо explicit bounded content-unavailable
   state;
5. terminal checksum record для exact preceding bytes.

Provider locator, provider account ID, raw MIME, rendered HTML, attachment
bytes, internal Blob metadata, search tokens, Review truth и AI output в
artifact не входят. Невалидный UTF-8 не преобразуется lossy decoder.

### Durable source preparation

Client command создаёт owner-local export job и exact outbox record в одной
транзакции. Workflow публикует typed durable command:

```text
communications / evidence_export_prepare / v1
```

Communications consumer:

1. сверяет inbox ID/hash до mutation;
2. авторизует exact canonical IDs и current evidence revisions;
3. читает body только своим read-only content capability;
4. создаёт отдельную bounded source copy через exact
   `communications.export-source.blob.v1` write capability;
5. target-bind-ит source proof только к
   `communications_export` / `makosh-communications-export-runtime` /
   `communications_export.source.blob.v1`;
6. атомарно сохраняет preparation result и exact outbox bytes;
7. публикует только typed metadata и opaque target-bound receipt/proof.

Новый source Blob capability разделяет Communications custody scope только
ради exact read/write preparation и не получает client route, generic transfer
target или произвольный recipient. Existing
`communications.content.v1` остаётся read-only client capability.

При delete/edit после request Communications либо подготавливает snapshot
exact current revision, либо возвращает typed stale/rejected result. Silent
partial export запрещён.

### Workflow artifact materialization

Workflow consumer атомарно принимает prepared result в inbox/job state.
Для каждого prepared item runtime:

1. получает Kernel-issued evidence-bound custody transfer в
   `communications_export.artifact.v1`;
2. читает target-owned bytes через exact `read_range`;
3. сверяет declared size и SHA-256;
4. передаёт bytes deterministic encoder;
5. записывает один encrypted artifact Blob с receipt-bound full write;
6. атомарно сохраняет terminal job state и artifact receipt.

Blob operations принадлежат exact capability
`communications_export.blob.v1` с собственным custody scope и bounded
`write`, `read_range`, `custody_transfer`. Runtime не видит provider identity,
не читает Communications SQL и не вызывает module socket напрямую.

Retry использует durable job ID, inbox/outbox hash и per-item idempotency.
Restart или generation/grant change инвалидирует active sessions, но не
создаёт второй terminal artifact. Policy rejection terminal; Blob/Vault/NATS/
Storage unavailability retryable без потери exact request.

### Client contract и download

Workflow предоставляет три отдельные generated surfaces под exact capability
`communications.export.v1`:

```text
StartEvidenceExport(message_ids) -> accepted export_id
GetEvidenceExportStatus(export_id) -> pending | ready | rejected
IssueEvidenceExportRead(export_id) -> one-use opaque capability
```

Терминальные и промежуточные status transitions доставляются через один общий
owner-scoped replayable Gateway SSE по ADR-0393. `GetEvidenceExportStatus`
остаётся разовым initial/manual recovery snapshot; timer-based polling не
является transport contract.

Artifact bytes возвращаются только через descriptor-declared authenticated
`client_blob` route. Read capability:

- 32 random bytes, one-use, TTL не больше 30 seconds;
- привязана к owner session, export ID, exact artifact receipt, runtime
  generation и grant epoch;
- не сохраняется в PostgreSQL, URL, localStorage, logs или telemetry;
- атомарно consume-ится до Blob read;
- после restart/revoke/expiry/replay fail closed.

Gateway проверяет exact route/capability and hard response bound, но не
декодирует JSONL и не получает Blob locator/digest. Frontend использует только
generated workflow client, показывает progress/terminal state и сохраняет
полученные bytes через first-party host/browser download adapter. Frontend
domain Communications передаёт только selected canonical IDs в app-owned
workflow controller и не импортирует workflow implementation.

### Kernel agreement

Kernel согласует только typed technical authority:

- platform descriptor ceiling для одного `client_blob` ответа равен `24 MiB`;
  каждый route обязан объявить собственный меньший либо равный hard bound;
- exact workflow descriptor, signed executable and settings schema digests;
- independent registration, grants, runtime generation and revoke;
- exact event publish/consume routes and schemas;
- workflow Storage namespace;
- exact Blob custody scope/operations/quota;
- exact ClientRpc and `client_blob` routes;
- hard limits, deadlines and output bounds.

Kernel не импортирует Communications/export packages, не знает JSONL semantics,
не читает body/artifact и не выбирает provider. Gateway остаётся opaque
transport. Module-to-module socket, cross-owner SQL и generic query/read-all
grant не вводятся.

Development assembly обновляет exact pre-export state из шести модулей по
stable runtime artifact identity: существующие registration IDs и fences
сохраняются как predecessors, новый workflow получает отдельные registration,
Storage binding и runtime fences. Произвольный partial state, перестановка
модулей или тот же distribution generation fail closed.

## Gate `communications_export_v1`

Gate становится `implemented` только атомарно при наличии:

1. exact source-contract и пяти workflow build units;
2. signed independently restartable workflow runtime and exact descriptor;
3. durable command/prepared/result subjects with inbox/outbox replay;
4. Communications current-revision authorization без provider fallback;
5. target-bound source copy and cross-owner Blob custody transfer;
6. deterministic bounded JSONL artifact and exact digest;
7. owner-local job persistence, restart/idempotency and terminal status;
8. one-use authenticated output `client_blob` route;
9. generated frontend controller/presentation and exact capability guard;
10. wrong-owner/stale/edit/delete/replay/oversize/invalid-UTF8/revoke/outage
    negative matrix;
11. private content/Blob/provider metadata non-disclosure checks;
12. architecture, SRP, Cargo, Clippy, managed NATS/Storage/Vault/Blob/Gateway
    and live browser evidence.

Наличие ADR, encoder unit test или frontend download button отдельно gate не
открывает.

### Текущее evidence

На 2026-07-29 реализованы и проверены:

- все шесть build units, exact descriptor/release assembly и independently
  restartable workflow runtime;
- durable source command/result, workflow job inbox/outbox, replay и terminal
  state;
- target-bound source copies, cross-owner Blob custody, deterministic bounded
  JSONL, artifact receipt и one-use authenticated `client_blob`;
- generated frontend workflow controller/presentation без domain-to-workflow
  implementation import;
- live root `make dev` browser flow:
  start → ready → one-use `/api/blobs/communications-export/v1/artifact`
  download через first-party adapter; Vite проксирует только exact
  `/api/blobs/` prefix в Core Gateway с development proxy proof;
- managed wrong-owner status/ticket, edit snapshot, delete, unknown ID,
  replay, aggregate size, invalid UTF-8, restart, revoke, NATS outage и Blob
  outage checks;
- atomic current-revision fence непосредственно перед Communications
  inbox/outbox commit: изменённая или удалённая canonical revision даёт typed
  `STALE_REVISION`, а не prepared result;
- deterministic managed race перехватывает exact Communications source-Blob
  write после snapshot, изменяет canonical revision в disposable owner
  PostgreSQL до result commit и подтверждает terminal rejected workflow state
  с сохранённым source code `STALE_REVISION`; production runtime и contracts не
  содержат test hooks или process-global flags;
- architecture/SRP/Cargo/Clippy, workspace/integration tests, dependency policy,
  SBOM и managed Storage/Vault/NATS/Blob/Gateway contour.

## Rollback

- revoke `communications.export.v1` и обе export Blob capabilities;
- stop только export workflow runtime;
- прекратить новые source preparations;
- сохранить terminal job/audit records и уже принятые canonical evidence;
- инвалидировать runtime-local read tickets;
- не удалять Communications source evidence и не откатываться к provider или
  legacy REST;
- owner-local retention export artifacts оформляется отдельным lifecycle
  решением и не выводится из факта download.
