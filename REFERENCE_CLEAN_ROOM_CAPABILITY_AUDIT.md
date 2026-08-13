# Reference -> clean-room capability audit

Статус: read-only snapshot-аудит, не architecture policy и не phase-gate
решение.

Дата среза: 2026-08-09

Ветка и ревизия: `main`, `fa4630857`

Область: active clean-room backend, active frontend surfaces,
`references/backend-legacy/` и `references/frontend-legacy/`.

## Краткий вывод

Перенос нельзя честно описать одним процентом. В репозитории одновременно
существуют:

1. capabilities, уже включённые в exact clean-room owner/package inventory;
2. полностью реконструированные slices, которые ещё не admitted;
3. узкие clean-room slices на месте гораздо более широких legacy-доменов;
4. legacy-only возможности и намеренно отброшенные façade/demo surfaces.

Проверенный масштаб текущего backend:

| Метрика | Текущий срез |
|---|---:|
| Cargo workspace packages | 316 |
| Exact `implementation.productionPackages` | 264 |
| Workspace packages вне production package inventory | 52 |
| Из них test/development packages | 17 |
| Из них production-role code packages вне inventory | 35 |
| Admitted owner inventory | 5 domains, 2 integrations, 18 workflows, 4 engines |
| Exact business capabilities | 210 |
| Communications/Settings reconstruction inventory | 91/91 `implemented` |

35 production-role packages вне inventory образуют шесть чётких групп:
AI runtime/assembly (2), Speech-to-Text runtime/assembly/artifact (3), Telegram
(14), WhatsApp (5), Whisper STT (5) и Zulip (6). Ещё 17 пакетов являются
testkit/development units и поэтому сами по себе admission gap не создают.

Главный вывод по продукту:

- clean-room platform и Communications stack уже существенно шире старого
  `recovery_only` состояния, всё ещё описанного в части документации;
- Communications/Settings capability register полностью реконструирован на
  уровне declared evidence (`91/91 implemented`), но не все его owners и
  runtimes входят в текущий production owner/package inventory;
- Mail является единственной полноразмерной provider integration в admitted
  owner inventory;
- Telegram, WhatsApp и Zulip имеют clean-room contracts, runtime code,
  persistence, generated clients и compiled frontend adapters, но их полные
  packages/owners ещё не admitted;
- Contacts, Knowledge, Review и Tasks admitted только как узкие clean-room
  owner slices. Это не перенос всех прежних Personas/Knowledge/Review/Tasks
  функций;
- большая часть Personal OS вне Communications всё ещё находится в состоянии
  partial, facade или legacy-only.

## Как читать статусы

| Статус | Критерий |
|---|---|
| `production-admitted` | Owner/capability присутствует в exact active inventory, необходимые production packages разрешены policy; для клиентского use case есть capability-gated compiled adapter либо явно указано отсутствие UI. |
| `implemented-not-admitted` | Reconstruction inventory и tests фиксируют `implemented`, код/контракты существуют, но owner/capability либо runtime packages ещё не входят в exact active inventory. |
| `partial` | Перенесён только узкий slice, contract или scaffold; legacy capability шире, либо backend и frontend находятся на разных стадиях. |
| `legacy-only` | В active clean room нет соответствующего owner/runtime/client contract. Legacy служит только behavioral evidence. |
| `intentionally-dropped` | Historical façade/demo/compatibility surface явно не должен становиться clean-room capability. |

`implemented` из `communications-settings-reconstruction.json` не равен
production admission. Сам executable test требует, чтобы reconstruction gate
не появлялся как generic active business capability до отдельного exact
admission.

## Источники истины и ограничения

Текущий статус определялся в следующем порядке:

1. `backend/architecture/policy.json`: current slice, exact packages, owners и
   business capabilities;
2. Cargo metadata активного `backend/Cargo.toml`;
3. `backend/architecture/communications-settings-reconstruction.json` и его
   executable architecture test;
4. active source packages, generated frontend contracts, compiled client
   adapters и planned/facade surfaces;
5. legacy source/status files только как historical behavioral evidence.

Наличие legacy route, таблицы, Vue-компонента, scorecard или прошедшего в
прошлом теста не считается доказательством clean-room реализации. Этот аудит
также не подтверждает live deployment/provider readiness: полный `make
pre-push` и управляемые live provider contours не запускались.

## Platform и Core

| Capability family | Legacy/reference | Active clean room | Статус | Остаток |
|---|---|---|---|---|
| Kernel, recovery, module control, owner/device admission | Legacy monolith/kernel crates и локальные control routes | Изолированный Kernel, private SQLite Control Store, managed launch, Gateway session/control packages | `production-admitted` | Hardware-backed signer и Android device adapter не подтверждены этим срезом; live release не проверялся. |
| Vault и secrets | HostVault/database vault/secrets modules | Vault protocol, managed client, key provider, SQLCipher store и runtime; owner provisioning/retirement slices | `production-admitted` | Multi-client Android provisioning остаётся отдельным client gate. |
| Storage, PostgreSQL, PgBouncer, migrations | Общая legacy schema из 225 migrations | Owner-scoped Storage protocol/control/runtime/PostgreSQL/PgBouncer/migration packages | `production-admitted` | Legacy schema намеренно не переносится целиком; новые owners требуют собственных admitted DDL bundles. |
| Events/NATS | Legacy event bus, audit/event tracing | Durable envelope, JetStream, authority/runtime packages, owner outbox/inbox contracts | `production-admitted` | Global event-tracing product UI остаётся facade. |
| Blob и attachment custody | Local blob helpers и communication attachments | Blob protocol/client/runtime/service, delegated custody и client Blob routes | `production-admitted` | Каждый новый owner всё ещё требует exact custody capability. |
| Clock и Scheduler | Background daemons и local scheduling | Separate Clock/Scheduler protocols, persistence, JetStream и runtime packages | `production-admitted` | Не означает перенос всех legacy owner jobs; переносится только scheduler platform и admitted job kinds. |
| Gateway, SSE и client transport | `/api/v1/**`, handwritten clients, provider realtime patchers | Generated ConnectRPC contracts, shared replayable SSE, bounded Blob HTTP и capability routing | `production-admitted` | Planned owner screens пока не имеют compiled adapters. |
| Telemetry/diagnostics | Audit log и event-tracing modules | Sanitized telemetry protocol/collector и Settings diagnostics surface | `production-admitted` | Event trace search/envelope/causation UI пока facade. |
| Backup/restore/maintenance | Legacy scripts и direct operational commands | Component-owned recovery evidence и maintenance composition | `production-admitted` по source policy | Live disposable restore/release evidence не перепроверялось в этом аудите. |

## Business domains

| Legacy capability family | Clean-room owner/slice | Backend | Frontend | Итог |
|---|---|---|---|---|
| Communications | `communications` | 22 production packages: canonical evidence, content, saved search, sender insights, call evidence, retained replay and source contracts | Compiled `communications-owner` adapter и generated client | `production-admitted` |
| Contacts/address book | `contacts` + `mail_contacts_sync` | Contacts command/source runtime и Mail/CardDAV/Google People sync workflow admitted | Settings workflow panel существует; отдельный full Contacts/Personas client surface отсутствует | `partial`: admitted Mail identity/sync slice, не полный legacy Personas domain |
| Knowledge/notes | `knowledge` + reviewed note promotion | Verified note command/persistence/runtime и promotion path admitted | `knowledge` и `notes` surfaces planned/facade; часть старой query/store logic сохранена | `partial` |
| Review | `review` | Attention, task candidate и note candidate owners/persistence/runtimes admitted | Client catalog использует `planned-owner`; старый review orchestration остаётся facade | `partial` |
| Tasks | `tasks` + reviewed task promotion | Reviewed-candidate command/persistence/runtime admitted | `tasks` route использует `planned-owner`; legacy `/api/v1/tasks`, decisions и obligations logic остаётся неактивным scaffold | `partial` |
| Personas/identity/relationships around people | Нет полного clean-room owner; `contacts` покрывает только часть identity intake | Persona packages отсутствуют; Relationships domain policy-blocked | Personas surface маркирует себя active внутри старого layer, но client adapter остаётся `planned-owner` | `partial / legacy-only` |
| Calendar | Registered, development-allowed, но production packages отсутствуют | Есть только Settings composition slice; legacy Calendar имел широкий scheduled-event backend | Calendar surface planned, client adapter `planned-owner` | `legacy-only` для domain, `implemented-not-admitted` для Settings composition |
| Organizations | Registered и development-allowed, production packages отсутствуют | Нет clean-room contract/runtime/storage | Facade с сохранённой legacy query/selection orchestration | `legacy-only` |
| Documents | Registered и development-allowed, production packages отсутствуют | Нет Documents owner; attachment workflows не заменяют domain | Planned screen с legacy processing-query scaffold | `legacy-only` для domain; attachment processing вынесен отдельно |
| Projects | Registered, policy-blocked | Нет packages/capabilities | Facade с сохранённой legacy query/selection logic | `legacy-only` |
| Relationships | Registered, policy-blocked | Нет packages/capabilities | Остатки legacy review/persona logic | `legacy-only` |
| Obligations | Registered, policy-blocked | Нет packages/capabilities | Legacy REST logic остаётся внутри Tasks scaffold | `legacy-only` |
| Decisions | Registered, policy-blocked | Нет packages/capabilities | Legacy REST logic остаётся внутри Review/Tasks scaffolds | `legacy-only` |
| Graph | Не является admitted business owner | Нет global Graph projection; отдельные owner-local indexes разрешены | Knowledge scaffold всё ещё содержит graph query orchestration | `legacy-only / target projection undecided` |
| Signal Hub | App composition, не новый owner | Reconstruction slice `signal_hub_composition_v1` помечен implemented, отдельного production owner нет | Settings/app composition surface, без самостоятельного domain adapter | `implemented-not-admitted` |
| Home и Timeline | Future app projections | Owner/runtime отсутствуют | Явно `planned`; compatibility Communications path удалён | `legacy-only`, корректно fail-closed |
| AI Agents workspace | Narrow `ai` engine не равен Agents domain | AI contracts/core/persistence admitted; AI runtime/assembly вне package inventory | Agents surface facade и старые `/api/v1/ai/**` clients не являются compiled adapter | `partial` |

### Что именно не перенесено из широких legacy domains

- Calendar: event CRUD/read models, participants, outcomes, scheduling,
  reminders, analytics, provider sync и calendar intelligence.
- Personas: canonical persona directory, identity resolution, dossier, memory,
  relationship/timeline/trust views.
- Tasks: общий task CRUD/lifecycle, providers, context, dependencies,
  checklists, analytics, external sync и full product UI.
- Knowledge/Graph: global graph, contradictions, semantic/global search,
  context packs и memory projections.
- Organizations/Projects/Relationships/Obligations/Decisions/Documents: owner
  contracts, persistence, managed runtimes, generated clients и admitted UI.

Некоторые из этих функций присутствуют как legacy TypeScript orchestration в
active `frontend/src/domains/**`, но compiled navigation либо отсутствует, либо
использует `planned-owner`. Они являются migration scaffolds, а не работающими
clean-room capabilities.

## Provider integrations

| Integration | Реализованный clean-room scope | Active admission | Frontend | Итог |
|---|---|---|---|---|
| Mail | IMAP/Gmail/SMTP, account lifecycle, OAuth, operational read/commands, composition, contacts, attachment/evidence paths | 15 packages, owner `mail`, exact capabilities admitted | Compiled `mail-integration` adapter | `production-admitted` |
| Telegram | Core operational client, TDLib QR identity, reconfiguration, folders, automation, calls/signaling/media | Только delivery-intent contract в production packages; остальные 14 production-role packages вне inventory | Compiled Telegram adapter и generated clients существуют | `implemented-not-admitted` |
| WhatsApp | Host bridge, operational read/realtime, account/setup surfaces | Только delivery-intent contract admitted; 5 packages вне inventory | Compiled WhatsApp adapter существует | `implemented-not-admitted` |
| Zulip | Account lifecycle, history, operational read/realtime and HTTP adapter | Только delivery-intent contract admitted; 6 packages вне inventory | Compiled Zulip adapter существует | `implemented-not-admitted` |
| Ollama | Provider contracts/core/HTTP/persistence/runtime and AI workflow use | Packages находятся в production list, но `ollama` отсутствует в admitted integration owners | AI Settings slice существует, но domain surface не admitted | `implemented-not-admitted` как provider owner |
| Desktop call recording | Explicit-consent capture, Blob custody and call evidence handoff | Owner и 5 packages admitted | Call/transcription workflow panel существует | `production-admitted` |
| Whisper STT | Local process/core/persistence/runtime/assembly | Slice `implemented`; 5 packages вне production inventory, owner отсутствует | Используется как dependency reconstruction, отдельного active client surface нет | `implemented-not-admitted` |
| Zoom | Legacy foundation включал OAuth/webhooks/recordings/transcripts/call evidence | Clean-room owner/packages отсутствуют | Active Zoom integration surface отсутствует | `legacy-only`; требует нового provider admission |
| Yandex Telemost | Legacy foundation patch и companion/runtime boundary | Clean-room owner/packages отсутствуют | Active integration surface отсутствует | `legacy-only`; historical call façade intentionally dropped |
| OmniRoute | Legacy provider/router integration | Active clean-room owner/contract отсутствует | Нет compiled adapter | `legacy-only` |

## Workflows и engines

| Capability family | Clean-room состояние | Статус | Legacy gap |
|---|---|---|---|
| Attachment security | Contract, core, ClamAV adapter, persistence, runtime, assembly | `production-admitted` | Не заменяет весь legacy content intelligence. |
| Archive inspection | Separate engine with ZIP adapter and custody events | `production-admitted` | Другие archive formats требуют отдельных bounded adapters. |
| Text extraction | Plain/PDF/DOCX/OCR adapters, persistence/runtime | `production-admitted` | Legacy document domain и document lifecycle не восстановлены. |
| Preview/retained replay/translation | Separate workflows, renderers, custody and inference boundaries | `production-admitted` | UI доступен только в admitted source contexts. |
| Communications export | Source contract, Blob artifact, workflow runtime/status | `production-admitted` | Не является общим Documents export owner. |
| Delivery intent | Provider-neutral workflow plus provider event contracts | `production-admitted` | Provider delivery остаётся у integration owners. |
| Delayed delivery | Full package set и slice `implemented` | `implemented-not-admitted`: owner отсутствует в workflow inventory | Нужен exact owner/capability admission. |
| Bulk action | Full package set и slice `implemented` | `implemented-not-admitted`: owner отсутствует в workflow inventory | Нужен exact owner/capability admission. |
| Cross-channel forward | Explicit source preparation, Blob and delivery workflow | `production-admitted` | Требует admitted target provider для выполнения. |
| AI reply/summary/translation/explanation | Distinct workflows and typed context contracts | `production-admitted` для workflow packages/capabilities | Concrete provider runtime admission остаётся отдельным. |
| Recipient suggestion | Deterministic source/workflow | `production-admitted` | Не заменяет full Persona/identity engine. |
| Task/note candidate extraction and promotion | Communications -> Review -> Tasks/Knowledge workflows | `production-admitted` | Target domains покрывают только reviewed candidate slices. |
| Call transcription | Workflow owner и call evidence path admitted | `partial`: STT runtime/Whisper provider packages не admitted | End-user runnable contour зависит от concrete STT admission. |
| Mail contacts sync | Mail source -> Contacts commands, Scheduler and durable events | `production-admitted` | Full Personas intelligence не входит. |

### Legacy engines без полного clean-room replacement

| Legacy engine | Ближайший clean-room replacement | Статус |
|---|---|---|
| Attention | Review attention owner | `partial` |
| Automation | Telegram-specific automation code | `implemented-not-admitted`; generic engine не переносится автоматически |
| Call intelligence | Call evidence + recording + transcription | `partial`; analytics/speaker intelligence отсутствуют |
| Search | Communications derived search only | `partial`; global search отсутствует |
| Memory | Knowledge verified-note slice | `partial` |
| Consistency/contradictions | Нет admitted engine | `legacy-only` |
| Context packs | Explicit use-case context contracts, но нет generic Context API | `intentionally-redesigned`; broad legacy engine не переносится |
| Enrichment | Нет admitted engine | `legacy-only` |
| Identity resolution | Contacts intake/recipient suggestion only | `partial` |
| Obligation intelligence | Нет admitted owner/engine | `legacy-only` |
| Relationship intelligence | Relationships policy-blocked | `legacy-only` |
| Risk | Нет admitted engine | `legacy-only` |
| Speaker identity | Нет admitted engine | `legacy-only` |
| Timeline | Planned rebuildable projection | `legacy-only` |
| Trust | Нет admitted engine | `legacy-only` |

## Frontend surfaces

Текущий client catalog содержит 13 основных route surfaces. Compiled adapters
есть только для:

- Communications;
- Mail;
- Telegram;
- WhatsApp;
- Zulip;
- Settings/System Control.

Dashboard, Review, Personas, Knowledge, Tasks, Calendar и Documents используют
`planned-owner`; у них нет compiled adapter. Organizations, Projects, Agents,
Notes, Home и Timeline существуют как extracted domain/facade/planned
scaffolds, но не входят в основной compiled client route catalog.

Это создаёт два разных типа frontend gap:

1. Telegram/WhatsApp/Zulip: UI adapter и generated contracts готовы раньше
   production runtime admission.
2. Review/Tasks/Knowledge: backend owner slices admitted, но product routes
   всё ещё planned и сохранённая legacy query/store logic не переведена на
   точный clean-room client contract.

В active frontend всё ещё присутствуют handwritten `/api/v1/**` clients для
Calendar, Documents, Personas, Projects, Review, Tasks, Obligations, Decisions,
Agents и других legacy surfaces. Пока compiled navigation их не активирует,
они являются historical/scaffold logic. Их нельзя считать clean-room переносом
и нельзя снова подключать как compatibility path.

## Намеренно не переносится

Executable reconstruction inventory перечисляет historical presentation
facades, которые не должны становиться admitted capability:

- `discord_channels`;
- `google_meet_calls`;
- `mattermost_channels`;
- `microsoft_teams_calls`;
- `phone_calls_without_admitted_provider`;
- `slack_channels`;
- `telemost_calls`;
- `zoom_calls`.

Также намеренно не переносятся как совместимый surface:

- legacy `/api/v1/**` routes и handwritten DTO aliases;
- прежняя PostgreSQL schema/migration history целиком;
- direct cross-owner SQL и shared business facades;
- generic provider switch внутри Communications;
- generic Context/AI read-all API;
- dual-read, dual-write и fallback на legacy runtime;
- fake/demo cards без admitted provider implementation.

Zoom или Telemost могут быть реализованы позднее как отдельные integrations.
Отброшен именно historical façade, а не возможность когда-либо добавить
реального provider owner.

## Gap ledger и рекомендуемый порядок

### P0 — согласовать implemented code с production admission

1. Открывать отдельные exact gates для Telegram, WhatsApp и Zulip; добавить их
   full package inventories, owners, capabilities и signed release fragments
   только после повторной managed/provider conformance.
2. Admit concrete AI/STT execution path: AI runtime/assembly, Speech-to-Text
   runtime/assembly и Whisper provider либо явно понизить зависимые product
   slices до недоступных.
3. Отдельно admit `communication_delayed_delivery` и
   `communication_bulk_action`; наличие packages не заменяет owner capability.
4. Согласовать Ollama provider packages/capabilities с integration owner
   inventory и AI Settings surface.

### P1 — завершить UI уже admitted узких owners

1. Дать Review, Tasks, Knowledge и Contacts/Personas точные generated client
   contracts только для реально admitted slices.
2. Заменить `planned-owner` на compiled adapter лишь после backend route,
   capability guard, negative privacy tests и SSE/query evidence.
3. После cutover архивировать или удалить соответствующие handwritten
   `/api/v1/**` clients; не подключать их как временный fallback.

### P2 — открыть development-allowed domains

1. Calendar: canonical event owner, provider-neutral contracts, storage/runtime
   и только затем rebuilt UI.
2. Organizations и Documents: separate owner ADR, exact packages, evidence
   intake/promotion и generated clients.
3. Определить, является ли Persona самостоятельным owner или продуктовым
   представлением Contacts + Identity/Relationships; не переносить старую CRM
   schema по имени.

### P3 — снять policy-blocked domain decisions

Для Relationships, Projects, Obligations и Decisions сначала нужны ownership
и authority ADR, затем contract/core/persistence/runtime/assembly, durable
events, review/promotion path и client adapter. Их legacy REST/store scaffolds
не являются стартовым API.

### P4 — derived projections, engines и дополнительные providers

1. Отдельно решить Graph, Timeline, global Search, Memory/Consistency, Risk,
   Identity Resolution и Signal Hub composition.
2. После core domains рассматривать Zoom, Telemost и OmniRoute как новые
   independently admitted integrations.
3. Любая новая provider UI доступна только вместе с working runtime,
   credential boundary, release inventory и live conformance.

## Documentation drift register

Следующие файлы описывают более ранние slices и не совпадают с текущим
executable policy:

| Файл | Устаревшее утверждение | Текущая проверка |
|---|---|---|
| `README.md` | Текущий runtime якобы состоит только из первых шести packages; Vault/Storage/data plane ещё не существуют | Policy содержит 264 production packages и 210 business capabilities; Cargo guard проверяет 316 workspace packages. |
| `AGENTS.md` | Current production inventory остановлен на `attachment_security_engine_v1`; Kernel readiness и многие packages описаны как отсутствующие | `implementation.currentSlice` равен `call_transcription_managed_conformance_v1`; owner inventory значительно шире. |
| `CORE_CLOSURE_PLAN.md` | Основной status narrative остаётся вокруг `first_owner_v1` и раннего Mail/Communications contour | Current reconstruction inventory закрыт на 91/91 slices; policy уже содержит широкий owner/workflow stack. |
| `docs/architecture/container-diagram.md` | Current slice не содержит data-plane services или owner modules | Текущий production package inventory содержит platform data plane и owner modules. |
| `docs/architecture/architecture-overview.md` | Current exact inventory ограничен Communications + Attachment Security; integrations/workflows отсутствуют | Current policy содержит domains, integrations, workflows и engines. |
| `backend/README.md`, `AGENTS.md` | Рекомендуется `make -C backend architecture-check` | В `backend/Makefile` такого target нет; действующие цели: `architecture-policy-check` и `cargo-boundaries-check`. |

Этот аудит только регистрирует drift. Исправление canonical docs должно быть
отдельным scoped изменением с обновлением architecture evidence hashes и
проверкой всех связанных assertions.

## Validation evidence этого аудита

Выполнено на указанной ревизии:

```text
make -C backend architecture-policy-check
architecture-policy-check: ok (3979 production paths checked)

make -C backend cargo-boundaries-check
cargo-boundaries-check: ok (316 workspace packages checked)

node --test backend/tests/architecture/communications-settings-reconstruction.test.mjs
7 passed, 0 failed

cd frontend && pnpm exec vitest run \
  src/domains/domainSurfaces.boundary.test.ts \
  src/app/queries/useClientNavigationSurface.test.ts
2 test files passed, 7 tests passed
```

Не выполнялось и поэтому не заявляется:

- полный `make pre-push`;
- managed live provider conformance для Mail/Telegram/WhatsApp/Zulip/Ollama;
- реальный STT/Whisper media contour;
- live deployment/release smoke;
- migration/restore реальных legacy данных.

Существующий untracked файл
`frontend/public/assets/.!82699!makosh-icon-128.png` не изменялся.
