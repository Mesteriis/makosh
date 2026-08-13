# Макошь

Макошь — local-first Personal Memory System / Personal Operating System.
Он объединяет коммуникации, evidence, знания, память, отношения, проекты,
документы, задачи, календарный контекст, решения и обязательства владельца.

## Текущее состояние

| Область | Текущее состояние |
|---|---|
| Production admission | `speech_to_text_whisper_admission_v1`: 283 exact production packages и 253 business capabilities |
| Implemented workspace | 420 Cargo packages / 421 manifests; 403 production-role packages, включая 120 `implemented-not-admitted` packages Tasks 10–26 |
| Development release | 41 managed modules с compiler-consumed runtime/Storage fragments; наличие артефакта не равно production admission |
| Предыдущий backend | Перенесён в `references/backend-legacy/` и используется только как evidence/reference |
| Desktop frontend | Vue 3 + Vite + Tauri используют generated clean-room Core Gateway contracts; staged owner clients остаются capability-gated |
| Android | Запланирован; код клиента и окончательная Kernel topology отсутствуют |
| Local development | Root `make dev` поднимает Compose, Kernel/Core Gateway и Vite, проверяет readiness и предоставляет loopback browser contour на `http://127.0.0.1:5173` |
| Active architecture | Clean-room ADR находятся в `docs/adr/`; executable policy, scripts и tests находятся внутри `backend/` |
| Предыдущая документация | Перенесена в `references/backend-legacy/docs/` и не является действующей policy |

Наличие workspace package или UI route не означает release admission либо
live provider readiness. `make dev` не создаёт provider credentials и не
запускает managed domain/integration runtimes за пределами их отдельных
assembly/admission units; клиент показывает фактическую availability из
Gateway bootstrap.

Последовательный admission после Task 9 пока заблокирован Task 10: нужны
авторизованные Apple/Xcode/device и Telegram provider evidence. Поэтому
Telegram, WhatsApp, Zulip, Review, lifecycle owners, Calendar, Organizations,
Documents, Relationships, Projects, Obligations, Decisions, Identity,
Search/Timeline/Graph и Memory/Consistency/Risk остаются
`implemented-not-admitted`. Zoom, Yandex Telemost и OmniRoute дополнительно
ожидают собственные реальные OAuth/API credentials и provider-success contours.
Эти блокировки нельзя обходить перестановкой slice или mock-success.

## Запуск и validation

Поддерживаемый command surface намеренно мал:

```sh
make build
make test
make dev
make docker
make tauri
make clean
make pre-commit
make pre-push
```

`test` запускает только проверки, затронутые текущими изменениями (базовый
commit можно переопределить через `MAKOSH_TEST_BASE`). `dev` поднимает
development Compose contour, создаёт только отсутствующую pristine
development owner identity, запускает Kernel с loopback Core Gateway и Vite,
ждёт direct и same-origin readiness, затем открывает
`http://127.0.0.1:5173/`. Kernel и Vite имеют один foreground lifecycle;
Compose остаётся запущенным после остановки. По умолчанию используется
`.local/kernel-dev`; другой абсолютный data directory задаётся через
`MAKOSH_DEV_DATA_DIR`. `tauri` собирает desktop app.

`.pre-commit-config.yaml` устанавливает оба Git hook type одной командой:

```sh
pre-commit install
```

`pre-commit` является быстрым локальным gate: architecture policy/evidence,
SRP и Cargo boundaries, architecture tests, Rust format, frontend lint и
typecheck. Медленные Rust workspace/integration tests, Clippy, dependency и
supply-chain checks, frontend unit/visual tests и production build выполняются
на `pre-push` через канонические `backend ci` и `frontend validate`.

Не следует использовать старые legacy `make`-цели,
`/api/v1/**` routes или `X-Макошь-Secret` как описание новой системы. Legacy
Makefile, scripts и связанные tool/CI configs перенесены в
`references/backend-legacy/` и не являются поддерживаемым command surface.

Для scoped frontend-работы сначала проверяйте актуальные scripts в
`frontend/package.json`. Успешная frontend-команда не является доказательством
работающего backend или end-to-end приложения.

Legacy backend можно читать и исследовать, но запрещено:

- импортировать его как dependency clean-room backend;
- считать его routes, schema, migrations или architecture действующим
  контрактом;
- запускать live provider actions или использовать реальные credentials;
- переносить код без повторной проверки ownership, security и новых ADR.

## Продуктовая модель

Макошь имеет два связанных пользовательских слоя:

1. Полноценные provider-specific operational experiences для Mail, Telegram,
   WhatsApp, Zulip и других встроенных integrations.
2. Provider-neutral evidence, memory и context над всеми каналами.

Integration владеет внешним протоколом, auth/session runtime, cursor,
operational contract и преобразованием наблюдений в neutral evidence. Domain
не знает об integration implementation и не меняет поведение по provider
identity.

Базовый поток:

```text
External signal
        ↓
Integration module
        ├─→ provider operational projection → channel screen
        └─→ neutral evidence observation
                    ↓
              Review / workflows
                    ↓
        domain command and durable truth
```

Raw provider data и AI output не становятся durable business truth напрямую.
Они сохраняются как evidence/candidate с provenance и проходят через owner
domain или явный workflow.

## Архитектура clean-room backend

```text
Tauri / planned Android / headless client
                    ↓
               Core Gateway
                    ↓
            Kernel control plane
   ├─ identity/capability router → isolated module runtimes
   │       ├─ business SQL → PgBouncer → PostgreSQL
   │       └─ durable envelopes → NATS JetStream
   ├─ runtime/infrastructure supervisor
   │       └─ Storage Control ── control only ──> PostgreSQL / PgBouncer
   └─ HPKE credential routing ↔ managed Vault
                                  └─ SQLCipher / platform key adapter
```

Основные инварианты:

- Kernel — только технический control plane, а не business layer.
- Каждый independently restartable module является отдельным OS-процессом.
- Ошибка одного domain, workflow или integration не останавливает соседние
  runtime.
- Kernel достигает `recovery_only` без PostgreSQL, PgBouncer, NATS, vault и
  modules.
- Обязательного bootstrap configuration file нет: используется OS-standard
  private data directory либо explicit `--data-dir`.
- Registrations, grants и desired infrastructure state читаются из private
  Kernel SQLite; он не содержит business data, credentials или module state.
- Недоступная SQLite оставляет online только sanitized
  `status/validate/export`; `restore/reset` требуют остановленного Kernel,
  explicit data directory, exclusive lock и interactive confirmation.
- Owner является logical authority, а каждое desktop/Android/operator
  device имеет собственную отзываемую ES256 keypair. Private key
  не покидает platform signer.
- External module может открыто зарегистрироваться как `pending`,
  но каждый `managed` launch требует verified exact executable bytes:
  signed bundled manifest либо owner-pinned digest.
- `DistributionManifestV1`, `ModuleDescriptorV1`, `GrantSet` и observed runtime
  state имеют разные authority; capability является единицей approval,
  readiness и revoke.
- Каждый module объявляет typed settings schema, а Kernel хранит
  desired/effective revisions в private SQLite и применяет их через hot reload
  либо supervised restart. Domain не мержит settings integrations.
- Vault является отдельным verified managed process. Kernel вычисляет grants и
  маршрутизирует только HPKE ciphertext; credential plaintext и Vault keys в
  Kernel не попадают.
- Process-bound credential leases ограничены owner/configuration/purpose,
  runtime generation и grant epoch. Vault restart/revoke инвалидирует leases,
  но не удаляет encrypted credential records.
- Module-to-module implementation imports, sockets и cross-module SQL
  запрещены.
- Kernel Supervisor управляет lifecycle PostgreSQL, PgBouncer и отдельного
  Storage Control process. Storage Control выполняет bootstrap, roles/grants,
  budgets, migration admission и readiness, но не проксирует business SQL.
- Каждый durable owner использует собственную generation-scoped PostgreSQL
  role/grants и ходит напрямую через PgBouncer; runtime credential выдаётся
  только scoped Vault lease.
- PgBouncer является pool/queue boundary, а не единственным security boundary.
  Пока OS-level socket/network isolation не доказана conformance tests, нельзя
  утверждать, что same-UID process физически не способен попытаться обойти
  pooler endpoint.
- Durable commands/events/observations/results/acks используют один binary
  `DurableEnvelopeV1`; outbox relay публикует exact bytes в NATS JetStream с
  at-least-once semantics.
- Desktop и Android общаются только с Core Gateway.
- Client queries/commands используют ConnectRPC/Protobuf, realtime —
  replayable SSE, blobs — bounded HTTP по opaque references.
- Provider operational contracts не видны context domains.
- Plugin store, remote executable code и silent topology fallback не
  поддерживаются.

## Структура репозитория

- [`backend/`](backend/) — единственная граница clean-room backend: Cargo
  workspace, owner-isolated packages, platform/runtime contracts, policy,
  scripts и tests.
- [`references/backend-legacy/`](references/backend-legacy/) — предыдущий Rust
  backend и workspace только для исследования.
- [`frontend/`](frontend/) — Vue 3 / Vite / Tauri client с generated Core
  Gateway contracts для принятого clean-room surface.
- [`docs/`](docs/) — только действующие clean-room ADR и минимальные
  architecture summaries.
- [`references/backend-legacy/docs/`](references/backend-legacy/docs/) — вся
  документация предыдущей реализации, включая archive, product/domain specs,
  roadmaps, testing/status материалы и generated wiki.
- [`docker/`](docker/) — унаследованные local infrastructure assets; до
  clean-room замены каждое использование требует проверки фактических путей и
  dependencies.
- [`references/backend-legacy/scripts/`](references/backend-legacy/scripts/) и
  [`references/backend-legacy/Makefile`](references/backend-legacy/Makefile) —
  неисполняемый operational reference предыдущей системы.

## Порядок чтения для разработки

1. [`AGENTS.md`](AGENTS.md) — обязательные правила работы в репозитории.
2. [Backend clean-room boundary](backend/README.md).
3. [Active ADR index](docs/adr/README.md).
4. [Architecture overview](docs/architecture/architecture-overview.md).
5. [Component communication contract](docs/architecture/component-communication.md).
6. [Storage Control Plane](docs/architecture/storage-control-plane.md).
7. [Vault and credential leases](docs/architecture/vault-and-credential-leases.md).
8. [Executable architecture policy](backend/architecture/README.md).

ADR-0225 открыл первый recovery slice; последующие exact gates довели текущий
admitted inventory до `speech_to_text_whisper_admission_v1`. Источником истины
является `backend/architecture/policy.json`, а не историческое описание первого
среза. Любой следующий package/runtime capability всё ещё требует
последовательного phase gate, executable policy и свежих provider/conformance
evidence; 120 production-role packages Tasks 10–26 пока не admitted.

Cargo topology уже зафиксирована ADR-0212 и executable guard: owner runtimes не
агрегируют друг друга, Kernel/Gateway не компилируют module packages, а
implementation changes любого domain или integration остаются внутри его
reverse-dependency graph. Telegram в документации является примером, а не
особым архитектурным случаем.

Code constitution зафиксирована ADR-0213: SRP определяется ownership и причиной
изменения, а каждый module обязан отдельно собираться, тестироваться,
останавливаться и деградировать без скрытой зависимости от соседних owners.

Durable background work зафиксирован ADR-0214: Scheduler хранит и изменяет
расписания, Event Hub обеспечивает declared delivery topology, а исполняемый
handler, checkpoint и result остаются внутри module-владельца. Правило едино
для integrations, domains, AI, workflows и platform maintenance.

Module admission зафиксирован ADR-0215: любой локальный process может создать
`pending` registration, но получает права только после явного owner approval в
Module Registry control surface. Effective GrantSet ограничен hard Kernel policy; только `managed`
runtime имеет restart guarantee, а `external` runtime Kernel лишь авторизует и
наблюдает.

Boot-critical control state зафиксирован ADR-0216: private SQLite доступен до
PostgreSQL/Vault и принадлежит только Kernel. Даже при недоступном Vault Kernel
поднимает local recovery surface; SQLite adapter не хранит secrets и не виден
modules.

ADR-0217 фиксирует zero-config bootstrap: обязательного `bootstrap.toml` нет,
default path берётся из OS-standard application-data location, а
`--data-dir` является единственным pre-store override. Полная mutable settings
model находится в ADR-0222 и доступна только после trustworthy Control Store.

ADR-0218 фиксирует owner/device identity без shared owner secret:
каждое device подписывает operation-bound challenges собственным
platform-backed ES256 key, а Control Store хранит только public keys,
capabilities и revocation state. First desktop enrollment идёт через
one-time inherited FD только для pristine instance.

ADR-0219 разделяет registration, grants и executable integrity. Publisher
signature не нужна для external `pending`, но Kernel проверяет exact bytes
перед каждым managed launch. Download/install принадлежат Tauri/OS,
а rollback всегда explicit и тоже проходит verification.

ADR-0220 фиксирует внутренний durable wire contract: пять `oneof` message kinds,
независимые envelope/owner versions, exact descriptor SHA-256, byte-for-byte
outbox-to-NATS delivery и owner inbox hash. Durable Ack не равен JetStream ACK,
dead letter является sanitized technical record, а client SSE использует
отдельный gateway frame.

ADR-0221 фиксирует exact binary `ModuleDescriptorV1`: distribution manifest
доказывает установленные bytes и pin-ит descriptor digest, descriptor заявляет
capabilities, а GrantSet отдельно выдаёт права. Dependencies указывают
contracts/capabilities, но не module implementation или process address.

ADR-0222 фиксирует Kernel Settings Registry: каждый module владеет schema и
смыслом полей, Kernel — catalog, typed desired/effective revisions, optimistic
concurrency и supervised application. `operator_managed` values меняет
аутентифицированный владелец, `kernel_managed` — только известный Kernel
controller; secrets, jobs, cursors и business state исключены.

ADR-0223 фиксирует отдельный encrypted Vault boundary: verified managed process,
SQLCipher плюс record-level AEAD, platform/recovery key slots и scoped
`CredentialLeaseV1`. Kernel supervises и authorizes Vault, но видит только
sanitized state и HPKE ciphertext. Большие или high-churn provider session
stores остаются у integration owner; hidden WhatsApp WebView сохраняет cookies
в OS-managed per-account profile. Решение принято, но production Vault ещё не
реализован.

ADR-0224 фиксирует Storage Control Plane: Kernel supervises PostgreSQL,
PgBouncer и отдельный control-only storage runtime; module business SQL идёт
напрямую через PgBouncer, а Storage Control управляет bootstrap, grants,
connection budgets, migration admission и readiness. Runtime credentials
принадлежат Vault, migrations являются immutable owner bundles. Решение принято,
но production packages и PostgreSQL/PgBouncer integration suite ещё не
существуют.

ADR-0225 отделяет текущую реализацию от полной конституции. В первом slice
активны только `supervisor` и локальный recovery `core_gateway`; PostgreSQL,
PgBouncer, NATS, Vault, Blob, Scheduler, managed modules и business data plane
не запускаются. Следующие фазы закрыты явными evidence gates.

ADR-0226 фиксирует AI context boundary: AI не читает таблицы или query APIs
других owners. Отдельный use-case workflow запрашивает явные owner contracts и
строит distinct generated request с common `AiContextReceiptV1` и concrete
use-case context message. Global fragment union, Generic Context API, read-all
grants и durable Context projection запрещены; AI output остаётся candidate до
workflow и target-domain validation.

## Безопасность и данные

- Не commit и не печатайте credentials, tokens, cookies, private keys,
  provider sessions, private messages или documents.
- PostgreSQL является canonical relational store для module-owned business и
  operational state; NATS является delivery/replay transport, а не source of
  truth. Kernel control state, Vault credentials, blobs и telemetry имеют
  отдельные boundaries.
- Storage Control не является SQL proxy: owner runtimes работают с PostgreSQL
  через PgBouncer по `StorageBindingV1`, а privileged bootstrap/migration paths
  остаются отдельным audited control plane.
- Search indexes, embeddings, projections и context packs являются
  rebuildable state.
- Cross-owner AI context собирается только use-case workflow через public owner
  contracts; read-only SQL к чужой таблице остаётся запрещённым cross-owner SQL.
- Bounded provider credential material хранится только в Vault. Большой или
  часто изменяемый session state остаётся в private integration-owned encrypted
  store; Vault выдаёт ему только wrapping-key lease.
- Imported content считается untrusted input и не является инструкцией для AI
  или tool runtime.

Security issues следует сообщать согласно [`SECURITY.md`](SECURITY.md), а не в
публичном issue.

## Документация и лицензия

- [Documentation index](docs/README.md)
- [Active ADR index](docs/adr/README.md)
- [Storage Control Plane](docs/architecture/storage-control-plane.md)
- [Vault and credential leases](docs/architecture/vault-and-credential-leases.md)
- [Legacy documentation reference](references/backend-legacy/docs/README.md)
- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [MIT License](LICENSE)
