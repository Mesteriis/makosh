# ADR-0373: Bounded attachment preview workflow

Статус: Принято

Дата: 2026-08-01

Состояние реализации: managed multi-format Gateway/client Blob/SSE slice. Clean-room owner boundary,
client/private-content boundary, event-only custody, renderer topology и phase
gate определены. Отдельные `makosh-attachment-preview-api`, `-ingress`, `-core`
и `-renderer-contract` admitted и реализуют versioned Start/Get/IssueRead/
client_blob contracts, target-owned custody envelopes, pure order-independent
evidence join/lifecycle/output policy и byte-only magic detection. Независимые
`makosh-attachment-preview-text`, `-image` и `-media` реализуют bounded UTF-8
normalization, decode-and-fresh-PNG image rendering и fail-closed MP3/MP4
container validation. Отдельный `makosh-attachment-preview-pdf` реализует
pure-Rust first-page rasterization с bounded viewport, fresh PNG output и
fail-closed active-content policy без native library или shell. Отдельный
`makosh-attachment-preview-docx` проверяет bounded OPC/ZIP structure, запрещает
external relationships, macros, ActiveX/OLE и source-provided fonts, извлекает
только bounded `word/document.xml` и строит fresh first-card PNG bundled
DejaVu Sans с pinned digest и включённой лицензией. Pure core теперь валидирует
полный status/result lifecycle, exact safe transition и source custody facts.
Отдельный `makosh-attachment-preview-persistence` реализует owner-local
PostgreSQL request replay, exact inbox/outbox, order-independent evidence join,
fenced jobs, derived artifact metadata, replayable realtime и hashed one-use
actor-bound read tickets без private content или ticket plaintext. Отдельный
`makosh-attachment-preview-runtime` теперь является admitted managed workflow
binary: он аутентифицирует exact descriptor/settings, получает fenced NATS,
Storage, Vault и Blob grants, owner-locally обрабатывает request/query/
client_blob, custody inbox/outbox, magic-only renderer dispatch, derived Blob
commit и metadata-only replayable realtime. Runtime не импортирует
Communications или Attachment Security implementation, не содержит SQL и не
возвращает private bytes через query/SSE. Отдельный
`makosh-attachment-preview-assembly` fail-closed материализует canonical
descriptor, empty typed settings schema, owner-local Storage bundle и sorted
unsigned runtime/storage release fragment; он не запускает runtime, renderer и
не получает signing authority. Development release теперь собирает этот fragment
отдельно и передаёт его release compiler для подписи. Authenticated managed
conformance поднимает exact signed Preview binary как отдельный OS-процесс,
применяет owner-local Storage bundle, выдаёт Vault credential lease и NATS grants.
Attachment Security теперь отдельно потребляет target-owned Preview custody
command, проверяет собственное safe/current-custody evidence и публикует exact
delegated/rejected result без RPC между owners. Authenticated managed gate
доказывает полный UTF-8, PNG, PDF, DOCX, MP3 и MP4 flow через
Start/Get/IssueRead, target-bound Blob redelegation, fresh derived Blob для
изображений и документов, exact bounded media copy, one-use actor-bound
`client_blob`, metadata-only SSE и exact restart replay без повторного render.
Blob data listener явно переводит принятый Unix stream в blocking mode, поэтому
partial large frame не превращается в `Broken pipe`; fenced Preview jobs
восстанавливаются после истечения lease и fail closed как `Unavailable` после
исчерпания bounded attempts. Managed failure-boundary conformance дополнительно
доказывает NATS outage/replay без duplicate custody/render/artifact, exact
request replay и operation-id conflict, actor-fenced one-use Blob tickets,
malformed PDF/PNG, active PDF и unsupported binary fail-closed outcomes, а
также отсутствие source bytes в status/SSE carriers. Следующий managed gate
доказывает stale renderer/state revision/runtime generation, expired/replayed/
wrong-actor ticket fencing, Blob и Vault outages, strict PNG trailing-payload
polyglot rejection и bounded DOCX expansion rejection. Общий Blob data client
получил bounded 30-second local-frame timeout, а loopback ClamAV conformance
явно переводит inherited accepted stream в blocking mode. Следующий managed
authority gate доказывает stale runtime/grant route fencing, stale delegated
custody proof после замены source grant epoch, exact source-receipt mismatch
после fenced lease recovery и отсутствие derived artifact/private bytes во
всех этих сценариях. ADR-0375 отдельно доказывает static renderer admission и
исключает fake runtime outage. Privacy-negative gate дополнительно фиксирует
закрытый enum runtime diagnostic stages, bounded sanitized reason codes без
`Debug` исходной ошибки, metadata-only client errors/SSE, identifier-only
managed readiness, отсутствие owner health endpoint и отсутствие telemetry
capability/signal у Preview. Authenticated managed Gateway gate теперь также
доказывает exact same-origin SSE continuation через browser-defined
`Last-Event-ID`: после известного cursor возвращается новый terminal Preview
event без повтора старого cursor или private source bytes. Generated Vue browser
workflow adapter реализован: app-level composition передаёт только canonical
attachment anchor, generated Start/Get/IssueRead и exact `client_blob` используют
Core Gateway, а navigation и Preview разделяют один replayable SSE hub без
polling. ADR-0376/ADR-0377 реализуют explicit owner-authorized exact-byte
retained-evidence replay без раскрытия producer selection клиенту. Shared hub
сохраняет current stream state для late consumer handshake, поэтому replay
начинается только после доказанного SSE `OPEN` и terminal frames не теряются в
гонке подключения. Live browser подтвердил исторический safe attachment через
`awaiting-evidence -> rendering -> ready` и one-use client Blob read.
Rejected/non-safe attachments не запускают workflow и отображаются disabled
`Unavailable`. Inventory gate `attachment_preview_v1` имеет состояние
`implemented` после полного repository pre-push gate.

Зависит от:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0273](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0360](ADR-0360-current-custodian-target-bound-blob-redelegation.md);
- [ADR-0374](ADR-0374-authenticated-client-blob-response-ceiling.md);
- [ADR-0375](ADR-0375-static-preview-renderer-admission-and-failure-semantics.md).

## Контекст

Legacy attachment preview находился внутри Communications и одновременно:

- читал Communications PostgreSQL rows и domain-owned filesystem paths;
- доверял provider filename/MIME при выборе behavior;
- возвращал text и base64 `data:` URL через один business query;
- отдавал исходные image/audio/video bytes браузеру;
- запускал PDF/DOCX renderer и сохранял derived Blob рядом с domain rows;
- объединял safety state, source-hash validation, rendering и delivery.

Такой перенос сделал бы Communications владельцем Blob data-plane, content
renderers, private-content transport и derived artifacts. Attachment Security
тоже не может владеть preview: malware verdict и пользовательское отображение
имеют разные authority, failure semantics и release cadence.

Clean-room Communications владеет только canonical attachment anchor и
provider-neutral safety projection. Attachment Security владеет verdict и
current source custody. Blob владеет bytes. Preview является отдельным
use-case workflow и производит только derived presentation artifact; он не
создаёт canonical Communications truth и не меняет safety verdict.

## Решение

Вводится отдельный managed workflow:

```text
owner_id  = attachment_preview
module_id = makosh-attachment-preview-runtime
kind      = workflow
```

Клиент передаёт Core Gateway только stable operation ID и canonical attachment
anchor. Клиент не передаёт Blob reference, filename, MIME, provider/account
identity, filesystem path, source digest, preview kind, renderer или bytes.

Workflow owner-locally и независимо от порядка объединяет:

1. authenticated Start request;
2. exact provider-neutral Attachment Security scan candidate;
3. canonical Communications transition `blob_admitted -> safe_for_delivery`.

Join создаёт durable target-owned custody-delegation command. Attachment
Security повторно проверяет completed clean verdict и current custody, получает
current-custodian target-bound proof по ADR-0360 и публикует exact delegated или
rejected result. Только exact command-linked delegated result создаёт fenced
preview job.

```text
client -> Gateway -> attachment-preview request_rpc
scan candidate event --------------------------\
safe_for_delivery event ------------------------> owner-local durable join
                                                   -> custody command outbox

custody command -> Attachment Security inbox
  -> clean verdict/current custody verification
  -> Kernel current-custodian redelegation
  -> delegated/rejected result event
  -> Preview result inbox
  -> target-bound one-use Blob read
  -> magic-based exact renderer adapter
  -> owner-local derived preview Blob
  -> metadata-only status + replayable SSE
```

Workflow не вызывает Communications или Attachment Security RPC, не читает их
storage и не импортирует implementation. Kernel, Gateway, Event Hub и Blob не
выбирают renderer и не интерпретируют preview content.

## Event-only custody contract

Target-owned `makosh-attachment-preview-ingress` содержит три exact durable
contracts:

```text
RequestAttachmentPreviewCustodyDelegationV1
AttachmentPreviewCustodyDelegatedV1
AttachmentPreviewCustodyDelegationRejectedV1
```

Request содержит только request/run/anchor, exact candidate message/hash,
safety message/evidence и logical owner. Target triple является константой:

```text
owner      = attachment_preview
module     = makosh-attachment-preview-runtime
capability = attachment_preview.blob.v1
```

Delegated result несёт opaque source reference, declared size/digest и bounded
target-bound proof только во внутренних event/runtime/persistence surfaces.
Proof, reference, filename, MIME, bytes и provider identity отсутствуют в
client API, SSE, logs, errors, health и telemetry.

Attachment Security импортирует только target-owned ingress contract. Preview
может импортировать public Attachment Security и Communications attachment
contracts, но не engine/domain implementation. Это contract edge, не engine
внутри workflow и не Communications facade.

## Client и private-content boundary

`makosh-attachment-preview-api` разделяет четыре причины вызова:

- `Start` — idempotent command/request receipt;
- `Get` — metadata-only status query;
- `IssueRead` — отдельный authenticated one-use read-ticket request;
- exact descriptor-declared `client_blob` route — bounded private bytes.

`Get` и realtime содержат run ID, anchor, state/revision, exact preview kind,
canonical output media kind, byte count, truncation и bounded error enum. Они
не содержат preview bytes, text, Blob reference/proof/receipt или read ticket.

`IssueRead` работает только для current `ready` revision и создаёт random
32-byte one-use opaque ticket с TTL 30 seconds, привязанный к authenticated
owner/device actor, run ID, revision, current runtime/grant generation и exact
artifact receipt. Клиент передаёт ticket только в exact `client_blob` body.
Gateway авторизует route, runtime обменивает ticket на internal Blob read и
возвращает exact bytes с `Cache-Control: no-store`. Клиент никогда не получает
internal Blob authority.

`accepted` не означает completion. Terminal status приходит через exact Get и
один общий replayable SSE stream; polling не вводится.

## V1 preview policy

Renderer выбирается только после bounded source read по verified magic и
container structure. Provider MIME, filename, extension и клиентский hint не
являются authority.

V1 поддерживает:

- valid UTF-8 plain/JSON/XML/YAML/CSV как normalized `text/plain`, visible
  artifact hard-truncated до 64 KiB;
- PNG/JPEG/GIF/WebP как decoded, bounded и заново encoded static `image/png`,
  максимум 5 MiB output и 16 megapixels;
- PDF как rendered first-page `image/png`, максимум 5 MiB output;
- DOCX как fixed-font, no-external-resource first-page/card `image/png`,
  максимум 5 MiB output;
- bounded MP3 audio и MP4 video как magic/container-validated owner-local
  presentation copy, максимум 24 MiB и 32 MiB соответственно.

SVG, HTML, JavaScript, active PDF/Office content, remote fonts/resources,
macros, embedded objects и browser rendering исходного PDF/DOCX запрещены.
Image metadata, animation и embedded profiles удаляются. PDF/DOCX adapters не
имеют network, shell, shared writable source mount или arbitrary filesystem
access. Media adapter не транскодирует и допускает только exact MP3/MP4 V1
container policy; неизвестный codec/container завершается `unsupported`.

Empty, malformed, polyglot, decompression-bomb или oversized renderer input
завершается bounded typed failure без partial artifact. Renderer V1 статически
связан с signed runtime и его availability проверяется admission/release gate
по ADR-0375; искусственный process outage не вводится. Unsupported не считается
empty successful preview.

Legacy base64 `data:` URL не восстанавливается: он дублировал private bytes в
JSON/heap/logging surfaces и обходил `client_blob` authorization.

## Diagnostics, health и telemetry privacy

Preview не вводит owner-specific health API. Kernel видит только generic
managed readiness signal с `registration_id`, `runtime_generation` и
`grant_epoch`; source/derived Blob authority, attachment metadata и private
content в этом signal отсутствуют. Descriptor не запрашивает telemetry
capability, а runtime не формирует telemetry signals.

Обычный managed launch направляет stdout в null и stderr в null. В explicit
developer-verbose режиме runtime может записать только fixed-shape diagnostic:

```text
developer_attachment_preview_runtime_error stage=<closed-enum> reason=<bounded-code>
```

Stage не принимается из request/provider data, reason является закрытым кодом,
а raw/`Debug` error не форматируется. Source/preview bytes, Blob reference,
receipt/proof, ticket, provider/account/filename/content metadata не входят в
diagnostic, client error, health или telemetry surfaces. Architecture и Rust
regression tests проверяют эти отрицательные границы, а managed format/failure
contours проверяют отсутствие source bytes в status, terminal event и SSE.

## Derived artifact и fencing

Runtime читает source только one-use receipt-bound Blob operation, передаёт
bytes exact adapter и записывает result через собственную Blob write
capability. Только после successful Blob commit persistence атомарно переводит
run в `ready`.

Owner-local PostgreSQL хранит opaque source/derived receipts, source evidence
и digest binding, renderer identity, status/revision, ticket hashes,
inbox/outbox/job и realtime state. Source и preview bytes, text и ticket
plaintext в PostgreSQL не хранятся.

Source replacement, safety evidence mismatch, stale custody proof,
runtime/grant generation mismatch, renderer revision mismatch, expired/used
ticket или missing derived Blob делают старый preview unreadable. Старый
artifact не становится fallback. Retry создаёт новый fenced run или продолжает
exact non-terminal job; он не переиспользует caller-selected artifact.

Text Extraction, archive inspection, CDR, translation, search indexing и AI
egress остаются отдельными owners/gates и не запускаются как side effect
preview.

## Единицы сборки и SRP

```text
makosh-attachment-preview-api
  generated Start/Get/IssueRead/client_blob/realtime contracts

makosh-attachment-preview-ingress
  target-owned custody command/result event contracts

makosh-attachment-preview-core
  pure join, lifecycle, format/output bounds and terminal decisions

makosh-attachment-preview-renderer-contract
  byte-only render request/result/error and exact format detection

makosh-attachment-preview-text
  bounded UTF-8 normalization and visible truncation

makosh-attachment-preview-image
  bounded image decode, metadata removal and PNG re-encode

makosh-attachment-preview-pdf
  isolated first-page PNG renderer

makosh-attachment-preview-docx
  isolated fixed-font card/page PNG renderer

makosh-attachment-preview-media
  bounded MP3/MP4 container validation and presentation copy

makosh-attachment-preview-persistence
  owner-local PostgreSQL inbox/outbox/run/job/artifact/ticket/realtime state

makosh-attachment-preview-runtime
  managed request/query/client_blob/Event/Blob/renderer orchestration only

makosh-attachment-preview-assembly
  descriptor/settings/Storage/runtime-resource artifacts and unsigned release fragment only
```

API/ingress/core/renderer-contract/adapters не зависят от runtime или
persistence. Persistence является единственным SQL owner. Runtime не
материализует release artifacts. Assembly не запускает renderer/runtime и не
подписывает manifest. Communications, integrations и Attachment Security не
получают renderer dependency. Build units разделены по функциональной причине
изменения, а не по количеству строк.

## Phase gate `attachment_preview_v1`

Gate становится `implemented` атомарно только после:

1. exact twelve-unit package inventory и compile isolation;
2. versioned Start/Get/IssueRead/client_blob/realtime и custody contracts;
3. owner-local request replay, exact message/hash inbox, outbox, ticket store и
   fenced jobs;
4. order-independent request/candidate/safety join;
5. current-custodian target-bound delegation and one-use source Blob read;
6. real bounded UTF-8, image re-encode, PDF, DOCX, MP3 and MP4 evidence;
7. derived owner-local Blob write without PostgreSQL private content;
8. wrong-owner, collision, stale revision/generation/grant/proof/renderer,
   source-hash and ticket replay/expiry negative matrix;
9. unavailable Blob/Vault/Event, renderer admission/integrity и
   malformed/polyglot/oversized input fail closed;
10. restart/NATS outage replay without duplicate custody, render or artifact;
11. authenticated Gateway Start/Get/IssueRead/client_blob and exact SSE cursor
   replay;
12. privacy-negative event/SSE/log/error/health/telemetry evidence;
13. architecture, SRP, Cargo, clippy, managed and full pre-push gates.

Все пункты gate закрыты; inventory state — `implemented`.

## Отклонённые варианты

### Вернуть GetAttachmentPreview в Communications

Отклонено: domain снова получил бы Blob I/O, renderer lifecycle, derived
artifact storage и private-content delivery.

### Добавить preview в Attachment Security

Отклонено: safety verdict не является presentation policy или renderer
authority.

### Отдать Blob reference или source bytes клиенту

Отклонено: клиент не является custody authority; exact `client_blob` route
сохраняет private bytes за authenticated one-use ticket.

### Доверять provider MIME/filename

Отклонено: metadata является untrusted observation и не выбирает executable
behavior.

### Объединить preview с Text Extraction, CDR или Translation

Отклонено: это разные use cases, output contracts, owners и failure semantics.
