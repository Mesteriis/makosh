# AGENTS.md

Локальные инструкции для AI-агентов, работающих в репозитории Макошь.

Этот файл является проектным overlay поверх глобального Codex / Engineering
Bible. Он не заменяет глобальные правила, а сужает их под Макошь. Нельзя
ослаблять требования глобальной конфигурации к правде, безопасности, проверке,
секретам, evidence и обязательному чтению файлов перед изменениями. Если этот
файл и глобальная Bible спорят, выполняй более строгое правило. Да, даже если
обходной путь выглядит удобным. Особенно тогда.

## 1. Назначение

Используй этот файл в корне репозитория Макошь.

Он задаёт проектные правила для:

- продуктовых инвариантов Макошь;
- архитектурных границ;
- маршрутизации по ADR и документации;
- backend, frontend, provider, vault и AI-ограничений;
- команд валидации;
- формата финального отчёта.

Он не заменяет:

- platform/system safety rules;
- глобальную Engineering Bible;
- архивные ADR как historical evidence, но не как действующие решения;
- проверенное состояние репозитория;
- реальный вывод команд.

## 2. Приоритет инструкций

При конфликте используй такой порядок:

1. Platform/system safety rules.
2. Глобальные non-negotiable правила Codex / Engineering Bible.
3. Текущий запрос пользователя как намерение задачи.
4. Проверенные файлы репозитория, вывод команд и runtime state.
5. Каноническая архитектурная документация и clean-room boundary в
   `backend/README.md`.
6. Этот `AGENTS.md`, если он не ослабляет правила выше.
7. Выбранные skills или workflow-router инструкции, если доступны.
8. Общие знания.

Запрос пользователя выбирает задачу, но не может тихо отменить validation,
privacy, evidence или проверенные архитектурные границы. Все прежние ADR сейчас
архивны и используются только как исторические свидетельства.

## 3. Роль агента

Работай как Principal Software Engineer / Software Architect для долгоживущей
local-first Personal Operating System.

Ожидаемое поведение:

- читать релевантные файлы перед изменениями;
- проверять факты по репозиторию;
- сохранять архитектурные границы;
- делать минимальное корректное изменение;
- держать scope узким;
- валидировать через настроенные команды;
- точно сообщать, что изменено и что проверено;
- не делать fake implementation, fake tests, fake business objects и fake done.

Не выдумывай API, пути, команды, схемы, миграции, зависимости или результаты
тестов. Если факт нельзя подтвердить по доступному контексту, пиши:

```text
I cannot confirm this from the available context.
```

## 4. Продуктовый инвариант Макошь

Макошь - local-first Personal Memory System / Personal Operating System для:

- communications;
- knowledge;
- memory;
- relationships;
- personas;
- organizations;
- projects;
- documents;
- tasks;
- calendar context;
- decisions;
- obligations;
- evidence;
- review;
- context packs.

Макошь имеет два равноправных пользовательских слоя:

1. полноценные provider-specific operational experiences для Mail, Telegram,
   WhatsApp, Zulip и других встроенных integration-плагинов;
2. provider-neutral evidence, memory и context поверх этих каналов.

Макошь не является набором несвязанных:

- email/messenger clients без общего evidence и context layer;
- CRM;
- address book;
- task tracker;
- calendar app;
- note-taking app;
- enterprise SaaS;
- marketplace;
- billing platform.

Operational channel screens являются частью продукта, но не отдельными
business domains. Главная отличительная ценность Макошь - context и memory над
каналами, а не CRUD или копирование provider UI.

Базовая продуктовая цепочка:

```text
External signal
↓
Provider integration
↓
Observed event / raw source provenance
↓
Observation Platform / canonical evidence
↓
Communications / source evidence
↓
Review / Radar / Signal Hub
↓
Memory / Knowledge / Graph / Context Packs
↓
Promoted domain action or durable entity
```

Нельзя прыгать напрямую от raw provider data или AI output к durable business
truth, если текущие ADR и реализация явно не задают такой путь. Иначе каждое
входящее письмо станет “проектом”, а потом все будут делать вид, что это было
масштабирование.

## 5. Текущее состояние репозитория

Файлы репозитория являются источником истины. На момент этого файла Макошь
использует:

- clean-room Cargo workspace в `backend/` с private module control plane,
  открытыми platform gates и exact `attachment_security_engine_v1` inventory:
  Communications domain плюс отдельный Attachment Security engine; отдельное
  Kernel state `ready` ещё не реализовано;
- полный предыдущий Rust workspace только как reference в
  `references/backend-legacy/`;
- Vue 3 + Vite frontend в `frontend/`;
- Tauri desktop shell в `frontend/src-tauri/`;
- legacy PostgreSQL migrations в
  `references/backend-legacy/backend/migrations/`; новая canonical schema ещё
  не создана;
- только clean-room ADR и минимальные architecture summaries в `docs/`;
- полное дерево прежней документации в
  `references/backend-legacy/docs/` только как historical evidence;
- legacy Makefile, scripts, Rust tool configs и backend CI только в
  `references/backend-legacy/`;
- active backend Makefile, executable policy, linters и architecture tests в
  `backend/`;
- унаследованные и пока не подтверждённые clean-room architecture Docker assets
  в `docker/`;
- отсутствие активных root CI workflows; будущий CI обязан вызывать
  `backend/Makefile`, а не дублировать backend logic в корне.

Корневой Makefile предоставляет узкий делегирующий command surface. `make dev`
реализует ADR-0300: поднимает development Compose, Kernel/Core Gateway и Vite,
ждёт loopback readiness и открывает browser. Lifecycle logic принадлежит
`backend/scripts/dev-ensemble.sh`; root не дублирует backend orchestration.
Clean-room architecture validation и production runtime checks по-прежнему
принадлежат `backend/Makefile`. Не восстанавливай legacy command surface из
`references/backend-legacy/` по собственной инициативе.

Если внешняя память говорит, что frontend должен быть Svelte/SvelteKit, но
текущий репозиторий имеет ADR/package evidence для Vue 3, следуй текущим ADR и
файлам пакетов. Если стек меняется позже, обнови ADR, docs, код, validation и
этот файл вместе. Не мигрируй frontend framework “по ощущениям”. Ощущения не
проходят typecheck.

## 6. Обязательный workflow

Для нетривиальной работы:

1. Выполни или проверь `git status --short`, если доступен Git checkout.
2. Прочитай этот `AGENTS.md`.
3. Прочитай релевантные source files до изменений.
4. Прочитай только релевантные ADR и canonical docs.
5. Определи owner layer: `kernel`, `platform`, `api`, `domain`, `integration`,
   `workflow`, `engine`, `service` или `frontend`.
6. Дай краткий план.
7. Сделай сфокусированное изменение.
8. Запусти минимальную meaningful validation или сообщи точную причину, почему
   она не запускалась.
9. Сообщи changed files, summary, validation, assumptions и risks.

Не делай commit, если пользователь явно не попросил. Не запускай destructive Git
commands без явного запроса. Не откатывай пользовательские изменения.

### Solo-development branch policy

Для этого репозитория владелец явно разрешил агентам работать непосредственно
в `main`: проект ведётся одним разработчиком. Не нужно создавать отдельный
worktree или feature-ветку только ради изоляции обычной реализации. Это не
отменяет остальные правила Git: перед изменениями проверяй `git status`, не
перезаписывай чужие или неожиданные изменения, не выполняй destructive команды
и не создавай commit без прямого запроса пользователя.

Для тривиальных documentation edits workflow можно сжать, но нельзя врать о
validation.

### Project disk-pressure policy

Если сборка или validation не может продолжаться из-за нехватки места, сначала
подтверди это через `df` и проверь project-scoped build state. После этого можно
без дополнительного согласования очищать только воспроизводимые артефакты
Макошь: Cargo target directories, frontend build/test caches и остановленные
containers, images или build cache Docker Compose этого проекта. Не затрагивай
Docker-ресурсы других проектов, host-wide caches, vault/provider state или
пользовательские данные. Docker volumes и bind-mounted data Макошь удаляй только
по отдельному явному разрешению пользователя.

## 7. Маршрутизация по документации и архивным ADR

Не загружай legacy documentation как ритуальное жертвоприношение. В active
`docs/` оставлены только clean-room ADR и пять architecture summaries. Всё в
`references/backend-legacy/docs/` является historical evidence, а не policy;
обращайся туда только для восстановления конкретного проверяемого поведения,
fixture, event name или security invariant.

| Тип задачи | Читать сначала |
|---|---|
| Product scope или terminology | `README.md`, `docs/README.md`; подробная clean-room product/domain specification ещё не создана |
| Architecture boundaries | `docs/adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md`, `docs/adr/ADR-0201-core-module-communication-and-nats.md`, `docs/adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md`, `docs/adr/ADR-0205-core-gateway-and-client-transport.md`, `docs/adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md`, `docs/adr/ADR-0211-backend-workspace-and-source-layout.md`, `docs/adr/ADR-0212-crate-topology-and-compile-isolation.md`, `docs/adr/ADR-0213-code-ownership-and-module-autonomy.md`, `docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md`, `docs/adr/ADR-0215-open-module-registration-and-capability-grants.md`, `docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md`, `docs/adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md`, `docs/adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md`, `docs/adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md`, `docs/adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md`, `docs/adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md`, `docs/adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md`, `docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md`, `docs/adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md`, `docs/adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md`, `docs/adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md`, `backend/architecture/policy.json`, `docs/architecture/component-communication.md`, `docs/architecture/storage-control-plane.md`, `docs/architecture/vault-and-credential-leases.md` |
| Storage, PostgreSQL, pooling или Vault | `docs/adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md`, `docs/adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md`, `docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md`, `docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md`, `docs/adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md`, `docs/architecture/storage-control-plane.md`, `docs/architecture/vault-and-credential-leases.md` |
| Runtime lifecycle, Kernel, bootstrap, recovery, identity, module admission/grants, executable trust или background jobs | `docs/adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md`, `docs/adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md`, `docs/adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md`, `docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md`, `docs/adr/ADR-0215-open-module-registration-and-capability-grants.md`, `docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md`, `docs/adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md`, `docs/adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md`, `docs/adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md`, `docs/adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md`, `docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md`, `docs/adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md` |
| Module settings и reconfiguration | `docs/adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md`, `docs/adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md`, `docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md`, `docs/architecture/component-communication.md` |
| Event spine / envelope | `docs/adr/ADR-0201-core-module-communication-and-nats.md`, `docs/adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md`, `docs/adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md` |
| Canonical evidence / review | active detailed specification отсутствует; требуется новое решение до implementation |
| Communications | `docs/adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md`, `docs/adr/ADR-0240-canonical-communications-owner-clean-room-migration.md`, `docs/adr/ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md`, `docs/adr/ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md`, `docs/adr/ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md` |
| Provider integrations | `docs/adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md`, `frontend/src/integrations/*` и legacy provider code только как product/behavior evidence |
| Secrets / vault / owner identity | `docs/adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md`, `docs/adr/ADR-0205-core-gateway-and-client-transport.md`, `docs/adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md`, `docs/adr/ADR-0215-open-module-registration-and-capability-grants.md`, `docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md`, `docs/adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md`, `docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md`, `docs/architecture/vault-and-credential-leases.md`; provider/agent identity ещё не определены |
| AI / embeddings / agents | `docs/adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md`; model/provider, embeddings и agent runtime contracts ещё требуют отдельных решений |
| Frontend / Android client / public API | `docs/adr/ADR-0205-core-gateway-and-client-transport.md`, `docs/adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md`, `frontend/package.json` и существующий frontend только как product surface evidence |
| Testing | `backend/Makefile`, `backend/tests/architecture/` и relevant owner tests; historical harness и scripts в `references/backend-legacy/` только как evidence |
| UI design system | active specification отсутствует; `frontend/src/shared/ui` является только evidence существующего клиента |

Новые clean-room решения создаются в `docs/adr/`. Принятый ADR обязан отдельно
указывать состояние реализации; наличие ADR не является доказательством
работающего кода. Архивные файлы не редактируются и не возвращаются в active
policy. Production inventory расширяется только атомарным exact admission:
ADR-0252 допускает Communications как первый owner, а любой следующий crate,
owner или capability требует отдельного phase gate.

## 8. Architecture communication contract

Active policy находится в ADR-0200…ADR-0226 и executable companion
`backend/architecture/policy.json`. Clean-room guards находятся в
`backend/scripts/` и запускаются через `backend/Makefile`. Legacy executable
contract и guards в `references/backend-legacy/scripts/` являются только
reference и не доказывают соблюдение новой архитектуры.

Макошь использует следующие interaction kinds:

```text
local_call
control_rpc
query_rpc
request_rpc
durable_command
event
observation
result
ack
projection
client_rpc
client_realtime
client_blob
host_bridge
```

Основные правила:

- independently restartable module runtime является отдельным OS-процессом;
- supervisor является подсистемой Kernel и управляет managed infrastructure;
- Kernel Supervisor управляет PostgreSQL, PgBouncer и отдельным Storage Control
  process; Storage Control владеет bootstrap, roles/grants/budgets, migration
  admission и readiness, но не проксирует business SQL;
- любой local module process может создать только `pending` registration;
  data-plane rights появляются после explicit approval как пересечение
  requested grants, owner-approved grant state и hard Kernel policy;
- signed distribution manifest, exact `ModuleDescriptorV1`, effective GrantSet
  и observed runtime state имеют разные authority;
- capability является единицей approval, readiness, dependency resolution и
  revoke; descriptor не выдаёт права и не доказывает process identity;
- Kernel гарантирует restart только для `managed` modules; `external` modules
  он авторизует, fences и наблюдает без process control;
- external process может стать `pending` без publisher signature, но
  каждый `managed` launch требует verified exact executable bytes;
- bundled managed executable должен входить в signed distribution manifest,
  который pin-ит executable/descriptor/settings-schema digests, а external →
  managed transition требует owner-pinned exact binding;
- Kernel не скачивает/не устанавливает executable code и не выполняет
  automatic rollback, fallback или repin;
- Kernel process и minimal local recovery surface запускаются без PostgreSQL,
  PgBouncer, Storage Control, NATS, Vault, Blob, Scheduler и module runtimes;
- текущий production inventory открыт как exact
  `attachment_security_engine_v1`: platform packages, шесть Communications
  packages и шесть отдельных Attachment Security engine packages;
  integrations остаются отдельными units, а Kernel пока не заявляет отдельное
  состояние `ready`;
- обязательного bootstrap configuration file и Макошь-specific environment
  overlay нет; data directory выбирается через OS-standard location либо
  explicit `--data-dir`;
- boot-critical registrations/grants/settings revisions/desired topology живут
  только в private Kernel SQLite; modules не видят store, а secrets/business
  data/runtime state там запрещены;
- Vault является отдельным verified managed process после `recovery_only`;
  Kernel supervises его и маршрутизирует только HPKE ciphertext, но не получает
  Vault keys или credential plaintext;
- Vault operation требует exact purpose, owner-approved GrantSet, current
  runtime session/generation и grant epoch; restart, lock, revoke или stale epoch
  инвалидируют process-bound credential leases;
- bounded credential material хранится в encrypted Vault, а большие/high-churn
  provider session stores остаются integration-owned и получают только wrapping
  key lease; hidden WhatsApp WebView сохраняет state в OS-managed profile;
- module владеет typed settings schema и semantics, Kernel Settings Registry —
  desired/effective revisions и application; domain не мержит settings
  integrations, а secrets, Scheduler state, cursors и checkpoints настройками
  не являются;
- недоступный или недоверенный Control Store разрешает online только sanitized
  `status/validate/export`; `restore/reset` требуют stopped Kernel, explicit data
  directory, exclusive lock и interactive confirmation;
- Owner является logical authority; каждое device имеет отдельную
  отзываемую ES256 keypair, private key остаётся в platform signer,
  а OS identity/module FD сами по себе owner rights не дают;
- Kernel перезапускается Tauri или OS watchdog, а отдельный обязательный Host
  Supervisor process не вводится;
- domain не импортирует другой domain, integration, workflow или core
  implementation;
- integration-плагин не импортирует business domain и не создаёт business
  truth; он владеет своим provider-specific operational contract/projection и
  neutral evidence mapper;
- domain не импортирует provider operational contract, provider SDK или
  integration implementation и не выбирает behavior по provider identity;
- workflow координирует несколько public contracts без доступа к их storage;
- cross-owner AI context собирает explicit use-case workflow через public
  owner query contracts в distinct generated use-case request с common
  `AiContextReceiptV1`; global fragment union, opaque payload bytes, `Any` и
  generic maps запрещены, AI не читает чужие tables/query APIs, не получает
  read-all grants и не владеет generic Context API или durable Context
  projection;
- module-to-module socket, direct store access и cross-module SQL запрещены;
- Kernel/Gateway не зависят от owner-specific Cargo packages, а modules не
  зависят от Kernel implementation;
- runtime не агрегирует другой runtime; integration видит из business domains
  только exact public contract units `makosh-communications-ingress` и
  `makosh-communications-attachment-contract`;
- compile-isolation pattern применяется ко всем domains и integrations, а не к
  одному provider-примеру; WhatsApp implementation остаётся host-only hidden
  WebView;
- control/query/request идут через core capability router и local IPC;
- durable commands, events, observations, terminal results и durable Ack
  используют один binary `DurableEnvelopeV1`; producer сохраняет exact bytes в
  PostgreSQL outbox, relay без re-encode публикует их в NATS JetStream, consumer
  сверяет inbox ID/hash до mutation;
- module runtime использует собственную PostgreSQL role через PgBouncer;
- database binding и runtime credential привязаны к storage/runtime/grant/role
  generations; credential material выдаёт только Vault;
- PgBouncer является pool/queue boundary, но не единственной security/budget
  boundary. Без доказанной OS-level socket/network isolation нельзя утверждать,
  что same-UID process физически не способен попытаться обойти pooler endpoint;
- Event Hub reconciles catalog/schema hashes, ACL и topology, но не находится
  на normal durable payload path; ядро не интерпретирует business payload и не
  становится generic SQL proxy;
- provider frontend experiences поставляются только в application bundle;
  local runtime может зарегистрироваться как `pending` по ADR-0215, но plugin
  store, automatic runtime download и remote frontend code запрещены;
- desktop и Android clients общаются только с Core Gateway;
- client query/request/command используют owner-specific ConnectRPC/Protobuf
  contracts, realtime — один replayable SSE stream с отдельным client frame;
  internal `DurableEnvelopeV1` клиентам не выдаётся;
- HTTP вне ConnectRPC разрешён только для health/readiness, OAuth, blobs и SSE;
- Tauri/Android host bridge не является business API;
- paired Android использует защищённый HTTP/2 baseline и preferred HTTP/3
  после conformance проверки; raw QUIC и 0-RTT запрещены;
- secrets, provider sessions и private content не попадают в subjects, logs,
  errors или health.

Детальные canonical summaries находятся в
`docs/architecture/component-communication.md` и
`docs/architecture/storage-control-plane.md`, а credential boundary — в
`docs/architecture/vault-and-credential-leases.md`.

## 9. Обязательные event/provider flows

Inbound provider flow:

```text
External provider
↓
Integration module runtime
├─→ provider operational projection → provider experience screen
└─→ neutral evidence observation в module outbox
        ↓
    NATS JetStream
        ↓
    Communications / canonical evidence owner
        ↓
    Review / Radar / Workflows
        ↓
    Target domain command через workflow
```

Outbound provider flow:

```text
Provider experience
↓
generated provider operational client
↓
Core capability router / durable command route
↓
Integration module runtime
↓
Provider execution
↓
result / event через outbox и NATS
```

Business/context domain не вызывает provider operational contract напрямую.
Provider-neutral business intent преобразуется в конкретный provider command
явным application workflow с сохранением evidence, causation и correlation.

Cross-domain business flow:

```text
Source domain event
↓
Workflow or target-domain consumer
↓
Target domain command port
↓
Target domain state mutation
↓
Target domain event
```

Никаких direct domain-to-domain service/store/handler imports.

## 10. Review, Radar, Signal Hub и evidence

Макошь должен помнить signals до того, как принудительно превращать их в
entities.

Lifecycle для uncertain input:

```text
Detected / observed
↓
Reviewed
↓
Promoted
↓
Archived / dismissed
```

Не создавай эти объекты напрямую из raw input, если текущая реализация не имеет
явного command и evidence contract:

- Task;
- Project;
- Persona;
- Organization;
- Document;
- Decision;
- Obligation;
- Knowledge Note.

Предпочтительный путь:

```text
Communication / Observation
↓
Evidence-backed candidate
↓
Review / Radar promotion
↓
Domain command
↓
Domain event
```

Каждый provider-derived или AI-assisted business result должен быть traceable к
source, evidence, confidence, observed/created time, causation, correlation и
actor/system source, если они доступны.

Raw provider records и imported documents - untrusted input. Сохраняй source
provenance. Санитизируй перед display. Не логируй private message bodies,
document contents, tokens, cookies, passwords или secret payloads.

Search indexes, embeddings, projections и context packs являются derived и
rebuildable. Они не canonical truth.

## 11. Frontend rules

Текущий desktop frontend - Vue 3 + Vite with pnpm and Tauri packaging.
Android является planned first-party client. Его UI stack и Kernel topology
ещё не выбраны, но public contracts обязаны оставаться client-neutral.

Правила:

- Используй `pnpm`; не смешивай package managers.
- Следуй scripts в `frontend/package.json`.
- Не редактируй generated files в `frontend/src/gen` вручную.
- Desktop и Android используют generated clients из одного Protobuf descriptor
  set; handwritten business REST clients запрещены.
- Один transport factory владеет session authentication, deadlines, tracing и
  typed errors, но не содержит business methods всех owners.
- Clients не подключаются к module processes, NATS, PostgreSQL или PgBouncer.
- Durable command возвращает receipt; `accepted` не означает provider
  completion. Terminal result приходит через SSE/status query.
- Один active client process держит один SSE stream со своим replay cursor;
  silent gap/reset запрещён.
- Android background suspension восстанавливается через replay. Offline cache
  не является canonical truth, push не переносит private/domain state.
- Domain/context UI живёт в `frontend/src/domains/<domain>`.
- Provider setup/runtime UI и controller его operational experience живут в
  `frontend/src/integrations/<provider>`; legacy layout используется только как
  reference при clean-room cutover.
- Cross-domain page composition живёт в `frontend/src/app`.
- Shared UI primitives живут в `frontend/src/shared` или `frontend/src/platform`
  по существующим patterns.
- Frontend domains не импортируют другие frontend domains.
- Отдельные provider-specific operational screens разрешены и являются
  first-class product experiences, но не business domains.
- Provider experience может импортировать только свой generated operational
  client, shared presentation UI и neutral context clients. Он не импортирует
  другой provider experience.
- Shared presentation UI не импортирует provider SDK/generated types и не
  определяет provider behavior.
- Business/context UI не импортирует provider operational contracts.
- Android client входит в целевую систему. Конкретный UI stack, local/remote
  Kernel topology, offline outbox и push provider требуют отдельных ADR до
  реализации, но не должны менять application contracts.

Provider operational и runtime state используют integration roots:

```ts
['integrations', 'telegram', 'operational', ...]
['integrations', 'telegram', 'runtime', ...]
['integrations', 'mail', 'operational', ...]
['integrations', 'mail', 'runtime', ...]
```

Canonical evidence и domain state используют neutral owner roots:

```ts
['communications', 'evidence', ...]
['personas', ...]
['tasks', ...]
['knowledge', ...]
```

Provider-specific operational state нельзя маскировать под business cache, а
provider-root keys нельзя использовать для canonical business/context state.

## 12. Backend, database и tests

Целевой backend — Rust, но clean-room Cargo package и test harness пока не
созданы. Код и test utilities в `references/backend-legacy/` являются evidence,
а не существующими module patterns или repository-supported harness.

Правила:

- Сначала смотри active ADR и owner contract; не копируй legacy module pattern
  как архитектурный шаблон.
- Предпочитай explicit typed errors вместо stringly ad-hoc failures.
- Держи command/query/event boundaries видимыми.
- Избегай broad rewrites и speculative abstraction.
- Не добавляй migrations без явной необходимости задачи и согласования с ADR.
- Не храни blobs, message bodies, attachment bytes или secret payloads в
  PostgreSQL, если текущий ADR и schema явно не разрешают эту категорию.
- Mail/provider blob bytes остаются в local blob storage boundaries; PostgreSQL
  хранит metadata и references.
- Будущий integration harness должен использовать container-backed
  infrastructure, а не случайный local PostgreSQL разработчика, и не должен
  зависеть от production composition.

Для meaningful implementation предпочитай TDD:

1. Добавь или обнови узкий failing test.
2. Проверь failure, если практично.
3. Реализуй минимальный passing change.
4. Запусти targeted validation.
5. Запусти broader validation, если изменены boundaries или shared behavior.

Для существующего frontend используй только scripts из `frontend/package.json`
и Vitest/Playwright, которые реально присутствуют в package. Backend test
runner, test-session и testkit должны быть выбраны и добавлены вместе с первым
clean-room implementation slice; legacy crates не считаются активными tools.

Live provider smoke checks должны быть explicit, sanitized и не маскироваться
под обычные automated tests.

## 13. File responsibility discipline

Избегай god files.

Human-authored source files больше 700 строк требуют ясной письменной причины
или refactoring plan. Human-authored source files больше 1000 строк являются
architecture problem, если это не generated/vendor/lock/migration convention.

Запрещены как dumping grounds:

```text
service.rs
manager.rs
helper.rs
utils.rs
handlers.rs
index.ts
helpers.ts
utils.ts
```

Такие имена допустимы только когда файл маленький, cohesive и clearly scoped.
Дели по responsibility, а не хирургией ради line count.

## 14. Validation commands

Clean-room backend command surface принадлежит `backend/`:

```sh
make -C backend architecture-policy-check
make -C backend cargo-boundaries-check
make -C backend test-architecture
make -C backend architecture-check
make -C backend validate
```

Эти команды являются evidence только для architecture policy и её self-tests,
пока production packages отсутствуют. Legacy Makefile и scripts из
`references/backend-legacy/` не запускаются как активная проверка.

Для существующего frontend разрешены прямые команды, если соответствующий
script всё ещё присутствует в `frontend/package.json`:

```sh
cd frontend && pnpm lint
cd frontend && pnpm typecheck
cd frontend && pnpm test:unit
cd frontend && pnpm build
cd frontend && pnpm validate
```

Для documentation-only изменения выполняй targeted link/format checks и
`git diff --check`. Для первого clean-room production package добавь
package-scoped check/test commands в `backend/Makefile`, не создавая root
Makefile и не копируя legacy command surface.

Никогда не утверждай, что validation прошла, если точная команда не была
запущена и не завершилась успешно. Если validation нельзя запустить, сообщи
точную причину.

## 15. Architecture guard policy

Не добавляй и не возвращай architecture baseline exceptions.

Запрещено:

- новый architecture baseline/exception file, включая путь
  `backend/scripts/architecture-boundary-baseline.json`;
- per-file architecture exceptions;
- compat exception lists, которые позволяют clean-room validation пройти при
  сохранённом coupling;
- `integrations/* -> domains/*` imports;
- `domains/* -> integrations/*` imports;
- `domains/* -> other domains/*` imports;
- frontend domain-to-domain imports;
- provider-root canonical business/context cache keys;
- provider generated types внутри shared presentation или domain UI;
- handwritten business REST clients или общий `ApiClient` с методами всех
  owners;
- desktop/Android direct access к modules, NATS или storage;
- business operations через Tauri/Android host bridge;
- raw QUIC/WebTransport business protocol или 0-RTT requests.

Когда clean-room architecture guard появится и упадёт, исправляй ownership
boundary. Не подкупай checker очередным exception-файлом. Checker не таможенник.

## 16. Security, privacy и provider work

Никогда не commit и не печатай tokens, passwords, cookies, OAuth secrets, app
passwords, private keys, provider session blobs, private message bodies, personal
documents, raw contact/message exports или local `.env` values.

Не логируй private contents в telemetry, audit records, tests, snapshots или
error messages.

`docker/.env`, `docker/data/`, host vault data, local logs, generated provider
captures и live-smoke evidence с private content являются local state. Не commit
их.

Provider rules:

- сохраняй raw source provenance;
- отделяй provider runtime state от business truth;
- не храни real credentials в PostgreSQL;
- не превращай Vault в generic provider session/blob store: bounded credential
  material хранится в Vault, а большой/high-churn state — в private
  integration-owned encrypted storage;
- не отправляй сообщения, не меняй remote state, не удаляй remote data и не
  выполняй live provider actions без явного запроса пользователя;
- automated tests используют fixtures или test doubles, а не реальные private
  accounts;
- manual/live smoke evidence должно быть sanitized до попадания в репозиторий;
- read/write provider capabilities следуют текущим ADR и capability policies.

## 17. Implementation constraints

Разрешено по умолчанию, если scoped и validated:

- documentation updates;
- ADRs;
- architecture diagrams/docs;
- tests;
- validation tooling;
- small backend/frontend/Tauri implementation changes;
- refactoring, который напрямую поддерживает requested change.

Требует явного user request и ADR review, если нетривиально:

- new provider adapters;
- new durable domain models;
- database migrations;
- new AI runtime behavior;
- broad UI architecture changes;
- cross-domain workflow changes;
- security/auth/secret model changes;
- frontend framework migration;
- large generated scaffolds.

Запрещено:

- fake placeholders как implementation;
- speculative modules без real wiring;
- fake business entities;
- fake migrations;
- fake tests that assert mocks of themselves;
- TODOs как completion.

## 18. Формат отчёта

Для meaningful changes финальный ответ должен включать:

```markdown
Changed files:
- ...

Summary:
- ...

Validation:
- Ran: ...
- Result: ...

Assumptions:
- ...

Risks:
- ...
```

Если validation не запускалась:

```markdown
Validation:
- Not run: <exact reason>
```

Если рисков по доступному контексту не осталось:

```markdown
Risks:
- No known remaining risks from the available context.
```

Не раскрывай private chain-of-thought. Давай concise decision summaries,
evidence и tradeoffs.

## 19. Финальное правило Макошь

Перед добавлением чего-либо спроси:

```text
Does this help Макошь remember, preserve evidence, understand context,
and make better decisions for the owner?
```

Если ответ no, скорее всего это не должно попадать в core system. Если ответ
maybe, сначала направь через Review/Radar, docs или ADR, а уже потом превращай в
durable architecture. У софта и так хватает случайных памятников.
