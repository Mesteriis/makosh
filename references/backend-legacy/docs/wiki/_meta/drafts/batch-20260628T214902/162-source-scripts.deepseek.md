## Summary / Резюме

Обновить страницу `components/scripts.md` в русской Obsidian‑wiki репозитория `makosh`. Страница должна документировать все скрипты из директории `scripts/`, их назначение, поведение, зависимости и ключевые функции – строго на основе предоставленного исходного кода. Добавить описания для build‑, dev‑, clean‑, backup‑, проверочных и WhatsApp‑readiness‑скриптов, а также общих библиотек `scripts/lib/`.

## Proposed pages / Предлагаемые страницы

### `components/scripts.md`

```markdown
# Скрипты (scripts)

Директория `scripts/` содержит операционные скрипты для сборки, разработки, тестирования, обслуживания и аудита архитектуры проекта.

## Библиотеки (`scripts/lib/`)

Общие модули, используемые другими скриптами.

### `common.sh`

Определяет глобальные пути и вспомогательные функции:

- `REPO_ROOT` – корень репозитория.
- `LOG_ROOT` – `$REPO_ROOT/.local/dev-logs` (логи dev-сессий).
- `BACKUPS_ROOT` – `$REPO_ROOT/backups`.
- `TOOLS_ROOT` – `$REPO_ROOT/.local/tools`, `TOOLS_BIN` – `$TOOLS_ROOT/bin`.
- `CARGO_TARGET_ROOT`, `CARGO_DEV_TARGET_DIR`, `CARGO_VALIDATE_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` – целевые директории Cargo.
- Цветовые константы (`color_blue`, `color_green`, `color_yellow`, `color_red`, `color_cyan`, `color_dim`, `color_reset`).
- `now_utc`, `today_utc`, `timestamp_compact_utc` – временные метки.
- `info`, `success`, `warn`, `error`, `dim` – цветной вывод.
- `ensure_dir`, `ensure_command`, `ensure_one_of`, `require_port_free`, `confirm_or_exit` – проверки и подтверждения.
- `json_escape` – экранирование для JSON.
- `emit_json_log`, `emit_live_log` – запись структурированных логов.
- `stream_service_pipe` – чтение из именованного канала и запись в лог-файлы и stdout.
- `wait_for_http`, `wait_for_service_http` – опрос HTTP‑эндпоинтов до успеха, с проверкой PID.

### `env.sh`

Загрузка переменных окружения:

- `ensure_docker_env_file` – если `docker/.env` отсутствует, копирует `docker/.env.example`.
- `load_makosh_env`:
  - вызывает `prepend_tools_bin_to_path`, `ensure_docker_env_file`.
  - экспортирует переменные из `docker/.env`.
  - задаёт значения по умолчанию (приведены ниже) и экспортирует составные переменные:
    - `MAKOSH_POSTGRES_DB=makosh_hub`
    - `MAKOSH_POSTGRES_USER=makosh`
    - `MAKOSH_POSTGRES_PASSWORD=change-me-local-dev-only`
    - `MAKOSH_POSTGRES_BIND=127.0.0.1`
    - `MAKOSH_POSTGRES_PORT=30432`
    - `MAKOSH_BACKEND_BIND=127.0.0.1`
    - `MAKOSH_BACKEND_PORT=8080`
    - `MAKOSH_BACKEND_STARTUP_ATTEMPTS=300`, `MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS=1`
    - `MAKOSH_FRONTEND_BIND=127.0.0.1`
    - `MAKOSH_FRONTEND_PORT=5174`
    - `MAKOSH_FRONTEND_STARTUP_ATTEMPTS=120`, `MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS=1`
    - `MAKOSH_LOCAL_API_SECRET=change-me-local-api-secret`
    - `MAKOSH_DEV_MODE=true`
    - `MAKOSH_HOST_VAULT_HOME=$HOME/.makosh/vault`
    - `MAKOSH_SECRET_VAULT_KEY=change-me-local-secret-vault-key`
    - `MAKOSH_OLLAMA_BASE_URL=http://127.0.0.1:11434`
    - `MAKOSH_OLLAMA_CHAT_MODEL=qwen3:4b`
    - `MAKOSH_OLLAMA_EMBED_MODEL=qwen3-embedding:4b`
    - `MAKOSH_OLLAMA_TIMEOUT_SECONDS=120`
  - `MAKOSH_VAULT_HOME=$MAKOSH_HOST_VAULT_HOME`
  - `MAKOSH_DEV_KEY_PATH=$MAKOSH_HOST_VAULT_HOME/dev/master.key`
  - `DATABASE_URL=postgres://<USER>:<PASSWORD>@127.0.0.1:30432/makosh_hub`
  - `MAKOSH_NATS_SERVER_URL=nats://127.0.0.1:${MAKOSH_NATS_PORT:-34222}`
- `ensure_bacon_available` – если `bacon` отсутствует, устанавливает его через `cargo install --root $TOOLS_ROOT bacon` и добавляет в `PATH`.
- `ensure_frontend_dependencies` – если `frontend/node_modules` отсутствует или нет `frontend/node_modules/.bin/tauri`, выполняет `pnpm install --frozen-lockfile`.

### `postgres.sh`

Управление PostgreSQL через Docker Compose:

- `compose_cmd` – вызов `docker compose` с `--env-file $DOCKER_ENV_FILE`, `--project-directory $REPO_ROOT/docker`, `-f $REPO_ROOT/docker/docker-compose.yml`.
- `postgres_up` – запускает контейнер `postgres`, ждёт готовности.
- `wait_for_postgres` – опрашивает `pg_isready` внутри контейнера.
- `postgres_status` – вызывает `docker compose ps postgres`.
- `postgres_stop` – останавливает контейнер.
- `postgres_data_dir` – возвращает `$REPO_ROOT/docker/data/postgres`.

### `resources.sh`

Подготовка bundled‑ресурсов для Tauri‑сборки:

- `prepare_backend_sidecar_macos` – собирает backend под целевую macOS‑архитектуру (`aarch64-apple-darwin` / `x86_64-apple-darwin`), копирует бинарник `makosh-backend` в `frontend/src-tauri/binaries`. Выполняется только на macOS.
- `prepare_google_oauth_resource` – проверяет JSON‑файл Google OAuth (должен содержать `installed` с полями `client_id`, `auth_uri`, `token_uri`), копирует в `frontend/src-tauri/resources/google-oauth/client_secret.json`. Требует `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.
- `prepare_tdlib_macos` – ищет или собирает `libtdjson.dylib` для macOS, копирует в `frontend/src-tauri/resources/tdlib/<platform_dir>/`. Источник определяется функциями `find_tdjson_source_lib` (переменные `MAKOSH_TDJSON_SOURCE`, `MAKOSH_TDJSON_PATH`, `brew --prefix tdlib`, стандартные пути) или `build_tdlib_from_source` (при `MAKOSH_TDLIB_BUILD_FROM_SOURCE=1`), которая клонирует `https://github.com/tdlib/td.git`, собирает cmake и ищет `libtdjson.dylib`.

### `rust-tooling.sh`

- `require_cargo_subcommand <subcommand> <install_hint>` – проверяет наличие cargo‑подкоманды, при отсутствии печатает подсказку и завершается с ошибкой.
- `require_binary <binary> <install_hint>` – аналогично для обычных бинарников.

## Сборка и запуск

### `build.sh`

Выполняет release‑сборку всего проекта:

1. `load_makosh_env`, проверяет наличие `cargo`, `node`, `pnpm`, устанавливает frontend‑зависимости.
2. `cargo build --release` backend (бинарник `makosh-backend`).
3. `pnpm build` frontend.
4. `prepare_google_oauth_resource`, `prepare_tdlib_macos`, `prepare_backend_sidecar_macos`.
5. `pnpm tauri build` – сборка Tauri‑релиза.

### `dev.sh`

Запускает локальную dev‑среду:

- Загружает окружение, проверяет `cargo`, `curl`, `bacon`, frontend‑зависимости.
- Запускает PostgreSQL через `postgres_up`.
- Проверяет, что порты `MAKOSH_BACKEND_PORT` и `MAKOSH_FRONTEND_PORT` свободны.
- Создаёт сессию логирования: папка `$LOG_ROOT/dev-<timestamp>-$$`, символическая ссылка `current` на неё, live‑лог `live.log`.
- Определяет `cleanup` (по сигналам `EXIT INT TERM`): убивает все дочерние процессы, удаляет именованные каналы.
- `run_service` – запускает сервис, перенаправляя stdout/stderr в именованные каналы, которые потоково читаются в JSON‑логи и live‑лог.
- Экспортирует переменные окружения для backend и frontend:
  - `DATABASE_URL`, `MAKOSH_LOCAL_API_SECRET`, `MAKOSH_DEV_MODE`, `MAKOSH_VAULT_HOME`, `MAKOSH_DEV_KEY_PATH`, `MAKOSH_SECRET_VAULT_KEY`, `MAKOSH_HTTP_ADDR`, `MAKOSH_FLOW_ID`, `MAKOSH_LOG_FORMAT=json`, `RUST_LOG`, `CARGO_TARGET_DIR`, `VITE_MAKOSH_API_BASE_URL`, `VITE_MAKOSH_LOCAL_API_SECRET`.
- Запускает backend через `bacon --headless backend-dev`.
- Ждёт healthz и readyz backend (до `MAKOSH_BACKEND_STARTUP_ATTEMPTS` попыток).
- Запускает frontend через `pnpm dev` с параметрами `--host`, `--port`, `--strictPort`.
- Ждёт доступности frontend.
- Выводит сводку: Flow ID, пути к логам, URL сервисов.

### `migrate.sh`

Запускает SQLx‑миграции:

- `load_makosh_env`, проверяет `cargo`, запускает PostgreSQL.
- Выполняет `cargo run --bin makosh_migrate` с `CARGO_TARGET_DIR=$CARGO_DEV_TARGET_DIR` и `MAKOSH_LOG_FORMAT=plain`.

## Очистка

### `clean.sh`

Удаляет артефакты сборки и временные файлы, не трогая базу данных:

- Удаляет: `$CARGO_TARGET_ROOT`, `frontend/src-tauri/target`, `frontend/node_modules/.vite`, `.vite-temp`, `frontend/dist`, `frontend/build`, `frontend/src-tauri/binaries/makosh-backend-*`, `$LOG_ROOT`, `$REPO_ROOT/tmp/makosh`, `*.log` в корне репозитория.

### `clean-data.sh`

Удаляет локальные данные PostgreSQL:

- `confirm_or_exit "This will delete local PostgreSQL data …" "DELETE"`.
- Останавливает контейнеры, удаляет `$(postgres_data_dir)`, создаёт заново.

### `clean-vault.sh`

Удаляет локальное хранилище vault:

- `confirm_or_exit "This will delete local vault data …" "DELETE"`.
- `rm -rf "$MAKOSH_HOST_VAULT_HOME"`.

## Резервное копирование и восстановление

### `vault-backup.sh`

Создаёт бэкап базы данных и vault:

- `load_makosh_env`, проверяет `psql`, `pg_dump`, запускает PostgreSQL.
- Создаёт директорию `$BACKUPS_ROOT/<дата>/<timestamp>/`.
- Выполняет `pg_dump` с параметрами `--no-owner --no-privileges` в `postgres.sql`.
- Копирует `$MAKOSH_HOST_VAULT_HOME` (если существует) в `vault/`.
- Пишет `manifest.json` (created_at, backup_dir, git_revision, параметры БД, vault).
- Пишет `RESTORE.txt` с инструкцией.

### `vault-restore.sh`

Восстанавливает из бэкапа:

- `load_makosh_env`, проверяет зависимости, запускает PostgreSQL.
- Показывает список доступных бэкапов из `$BACKUPS_ROOT`, предлагает выбрать.
- Проверяет наличие `postgres.sql`, `vault/`, `manifest.json`.
- `confirm_or_exit "Restore will replace …" "RESTORE"`.
- Терминирует активные соединения, `dropdb`, `createdb`.
- Восстанавливает дамп через `psql -f`.
- Очищает `$MAKOSH_HOST_VAULT_HOME` и копирует в него `vault/`.

## Логи

### `logs.sh`

Просмотр логов текущей dev‑сессии:

- Проверяет символическую ссылку `$LOG_ROOT/current` и файл `live.log`.
- При `MAKOSH_LOGS_FOLLOW=1` (по умолчанию) выполняет `tail -n 50 -f`, иначе `tail -n 50`.

## Проверки архитектуры и кода

### `check-architecture-contract.test.mjs`

Тест контракта архитектуры (`scripts/architecture-contract.json`):

- Проверяет отсутствие файла `architecture-boundary-baseline.json`.
- `schema_version = 1`.
- `interaction_kinds` равен `["direct_call", "command_port", "query_port", "event", "projection", "runtime_integration_api"]`.
- Проверяет правила deny/allow для слоёв backend:
  - `domains.deny` включает `other_domains`, `integrations`, `vault`.
  - `domains.owned` содержит `signal_hub`.
  - `integrations.deny` включает `domains`.
  - `workflows.allow` включает `domain_command_ports`, `domain_query_ports`.
  - `app.deny` включает `stores`.
  - `ai.deny` включает `domain_stores`.
  - `platform.deny` включает `business_table_sql`.
- Проверяет frontend: `domains.deny` включает `other_frontend_domains` и `integrations`; `integrations.deny` включает `domains`.
- `provider_business_cache_roots.forbidden` содержит `telegram`, `whatsapp`; `business_query_key_root` = `communications`.
- `forbidden_provider_business_roots` = `["/api/v1/integrations/mail/*", "/api/v1/integrations/telegram/*", "/api/v1/integrations/whatsapp/*"]`.

### `check-architecture.mjs` (частично обрезан)

Комплексная проверка архитектурных границ в Rust‑коде.

Из доступного фрагмента видны:

- Множество констант – владельцев операций (например, `communicationRawRecordInsertOwner`, `communicationMessageInsertOwner`, `reviewPromotionWorkflow`, `aiPromptMutationOwners`, и т.д.).
- `businessBackendDomains` – набор из 19 бизнес‑доменов (agents, calendar, communications, decisions, documents, graph, knowledge, mail, notes, obligations, organizations, personas, persons, projects, radar, relationships, signal_hub, tasks, timeline).
- `platformTechnicalTablePrefixes` – префиксы платформенных таблиц (ai_runtime_, api_audit_, application_, audit_, event_, observation_, projection_, secret_, settings_).
- `forbiddenCanonicalEvidenceDirs` – запрещённые директории для evidence.
- Функции анализа импортов Rust:
  - `extractGroupedBackendDomainImports` / `extractBackendDomainImports` – извлекает имена доменов из `crate::domains::{...}`.
  - `extractBackendRootImports` – извлекает имена из `crate::integrations`, `crate::app`, `crate::workflows`, `crate::vault`.
- `backendBoundaryViolations` – определяет нарушения:
  - домен не должен импортировать другой домен (кроме разрешённых `backendDomainProjectionBridgeOwners`).
  - интеграция не должна импортировать бизнес‑домен.
  - (дальнейшая логика обрезана, но присутствуют проверки и для воркфлоу, платформы, ai, engines).

Точное поведение за пределами 12000 символов не подтверждено данным контекстом.

### `check-code-boundaries.mjs`

Проверка исходных файлов на нарушения правил кодирования:

- Сканирует файлы из `scanRoots` (`AGENTS.md`, `Makefile`, `.pre-commit-config.yaml`, `backend`, `docs`, `frontend`, `scripts`), исключая `ignoredSegments`.
- Проверяет отслеживаемые git‑файлы на принадлежность к `generatedPrefixes` (кроме `docker/data/.gitkeep`).
- Анализирует содержимое файлов:
  - Ищет возможные хардкодированные секреты по `secretPattern` (исключая документацию и тесты).
  - Запрещает `MAKOSH_TEST_DATABASE_URL`, `MAKOSH_LOCAL_API_SECRET`, `DATABASE_URL` в backend‑тестах (кроме `backend/tests/config.rs`).
  - Запрещает широкие подавления предупреждений и lint/runtime-ignore директивы в Rust и TypeScript/ESLint.
  - В frontend‑шаблонах (`.vue`, `.html`) запрещает инлайн‑стили (`style=`) и встроенные `<style>` (кроме Vue SFC, где `<style>` разрешён).

## WhatsApp‑readiness аудит

### `whatsapp-business-cloud-edge-readiness.mjs`

Проверяет готовность edge‑прокси WhatsApp Business Cloud:

- Может выполнять live‑проверки (при `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1`):
  - `GET /healthz` (ожидает `status: "ok"`, `service: "makosh-whatsapp-business-cloud-edge-proxy"`).
  - `GET /manifest` (проверяет поля `public_webhook_path`, `protected_makosh_webhook_path`, `protected_makosh_manifest_path`, `local_auth_header`, `signature_header`, `payload_policy`).
  - `POST /webhooks/whatsapp/business-cloud` без подписи (ожидает 400 с `missing_x_hub_signature_256`).
  - `GET /readyz` (при `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE=1`).
- Статические проверки контрактов в файлах:
  - `backend/src/bin/makosh_whatsapp_business_cloud_edge_proxy.rs` – строки с константами маршрутов, заголовков, политик, env‑переменных, тестов.
  - `docker/docker-compose.yml` – сервис `whatsapp-business-cloud-edge-proxy`, профиль, healthcheck.
  - `docker/Dockerfile` – таргет `whatsapp-business-cloud-edge-proxy`.
  - `docker/.env.example` – переменные с loopback‑адресами, отсутствие секретов.
  - `Makefile` – таргеты `whatsapp-business-cloud-edge-config`, `…-up`, `…-stop`, `…-logs`.
  - `docs/integrations/whatsapp/live-smoke-checklist.md` – упоминание edge proxy.
  - `docs/integrations/whatsapp/status.md` – состояние DOMAIN CLOSURE и public exposure.

### `whatsapp-domain-closure-audit.mjs`

Аудит полного закрытия домена WhatsApp:

- Проверяет статические цели в `Makefile` (`whatsapp-live-smoke-readiness`, `whatsapp-native-md-sdk-gap-readiness`, `whatsapp-live-smoke-evidence`, `whatsapp-business-cloud-edge-readiness`).
- Проверяет, что `docs/integrations/whatsapp/status.md` содержит `DOMAIN CLOSURE = not achieved` и упоминания manual smoke, WebView, Business Cloud.
- Проверяет тесты: `communications_architecture_target.rs`, `whatsapp_signal_hub.rs`.
- Проверяет контракты evidence: `whatsapp-live-smoke-evidence.mjs` (common/personal/businessCloud gate IDs, префиксы, валидация).
- Проверяет `whatsapp-native-md-sdk-gap-readiness.mjs` на наличие контекста апгрейда.
- Валидирует evidence-файлы из `$MAKOSH_WHATSAPP_DOMAIN_CLOSURE_EVIDENCE_DIR` (по умолчанию `.local/whatsapp`) для трёх provider shapes.
- Извлекает `NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS` из `backend/src/integrations/whatsapp/runtime/native_md.rs` и фиксирует оставшиеся неподдерживаемые команды.
- Сверяет состояние `ADR-0101-whatsapp-provider-runtime-selection.md` (должен быть `Accepted`).
- Определяет `closureAchieved` только при отсутствии failed‑проверок, пустом списке blockers, `status.md` с `achieved` и ADR `Accepted`.
- При `--require-closed` или `MAKOSH_WHATSAPP_REQUIRE_DOMAIN_CLOSED=1` требует полного закрытия.

### `whatsapp-live-smoke-collect-evidence.mjs`

Сборщик evidence live‑smoke:

- Ожидает файл наблюдений (по умолчанию `.local/whatsapp/live-smoke-observations.json`) или генерирует шаблон с флагом `--observations-template`.
- `providerShape` – один из `whatsapp_web_companion`, `whatsapp_native_md`, `whatsapp_business_cloud`.
- `assertNoSecretLikeContent` – проверяет JSON на наличие запрещённых паттернов (session_blob, cookie, access_token, qr_code, номера телефонов, и т.д.).
- `mergeEvidence` – для каждого gate из observations обновляет шаблон: `status` (passed/pending), `observed_at`, `evidence_refs`.
- После слияния повторно проверяет на секреты, записывает evidence-файл (по умолчанию `.local/whatsapp/live-smoke-evidence-<shape>.json`).
- Выполняет `whatsapp-live-smoke-evidence.mjs` для валидации итогового файла.

### `whatsapp-live-smoke-evidence.mjs` (обрезан)

Схема и валидация evidence live‑smoke:

- `commonGateIds` – 16 общих ворот (preflight, runtime, event_flow, commands, media, redaction).
- `personalGateIds` – 50 ворот для personal‑провайдеров (auth, inbound, outbound, search).
- `businessCloudGateIds` – 15 ворот для Business Cloud (vault, edge_proxy, inbound/outbound webhook, rate_limit).
- `allowedEvidenceRefPrefixes` – разрешённые префиксы ссылок (audit:, blob:, command:, doc:, edge_proxy:, event_log:, log_scan:, projection:, raw_record:, runtime_api:, search:, signal_hub:, storage:, ui:, vault_binding:).
- `requiredEvidenceRefPrefixGroups` – обязательные группы префиксов для каждого gate.
- Генерация шаблона с `status=pending` или `passed`, заполнение `operator_attestation`.
- Валидация: `schema_version=1`, `run_id`, `generated_at`, `provider_shape`, `account_fingerprint` (sha256:...), evidence refs (не placeholder, соответствие префиксам).
- (Дальнейшая логика обрезана.)

### `whatsapp-live-smoke-readiness.mjs` (обрезан)

Проверка готовности к live‑smoke:

- Может зондировать runtime API (при `MAKOSH_WHATSAPP_RUNTIME_API_PROBE=1`):
  - `GET /api/v1/integrations/whatsapp/capabilities` (ожидает объект с `version`, `runtime_mode`, `provider_shapes`, `capabilities`, `account_scope=null`).
  - `GET /api/v1/integrations/whatsapp/accounts/{id}/capabilities` (проверяет `account_scope.account_id`, `provider_shape`, `capabilities`).
  - `GET /api/v1/integrations/whatsapp/runtime/status?account_id=...` (поля `account_id`, `provider_shape`, `runtime_kind`, `status`, `session_restore_available`, `runtime_blockers`).
  - `GET /api/v1/integrations/whatsapp/runtime/health?account_id=...` (поля `healthy`, `checks`, `checked_at`).
  - `assertNoSecretLeaks` – проверяет ответы на утечку секретов.
- Статические проверки:
  - `frontend/src-tauri/src/whatsapp_companion.rs` – наличие `RUNTIME_EVENTS_BRIDGE_PATH`, `dispatch_runtime_bridge_runtime_event`, `X-Макошь-Secret`, `is_allowed_local_backend_url`.
  - `frontend/src-tauri/capabilities/whatsapp-companion-relay.json` – `"local": false`, только `https://web.whatsapp.com`, нет `core:default`.
  - `frontend/src-tauri/capabilities/default.json` – не содержит `allow-whatsapp-web-companion-relay-observation`.
  - `backend/src/integrations/whatsapp/runtime/web_companion.rs` – контракты инжекции, отсутствие блокирующих комментариев.
  - `backend/tests/whatsapp_signal_hub.rs` – покрытие dispatch.
- (Дальнейшая логика обрезана.)

### `whatsapp-native-md-sdk-gap-readiness.mjs` (обрезан)

Проверка API WhatsApp Native MD (wa-rs):

- Ожидаемая версия: `wa-rs 0.2.0`.
- `requiredApis` – 7 обязательных публичных функций (send_message, revoke_message, edit_message, mark_as_read, leave_group, upload, download_from_params) с указанием ожидаемых файлов.
- `unsupportedExpectations` – проверяет отсутствие API для команд: status publish, dialog state write (archive/unarchive/mute/unmute/pin/unpin), join by invite.
- Поиск исходников wa-rs в локальном Cargo‑регистре.
- Анализ `wa-rs-core` – проверка `groups.rs` на наличие join‑IQ.
- Анализ `wa-rs-appstate` – поиск публичного энкодера app-state патчей (encode.rs или публичных функций с соответствующими именами).
- Проверка `wa-rs-proto` – наличие полей `forwarding_score` и `is_forwarded`.
- Может выполнять `cargo info wa-rs` при `MAKOSH_WA_RS_CRATES_IO_PROBE=1`.
- (Дальнейшая логика обрезана.)
```

## Source coverage / Покрытие источников

| Файл | Факты, использованные в wiki |
|------|------------------------------|
| `scripts/build.sh` | Последовательность команд: cargo build, pnpm build, подготовка ресурсов, tauri build. |
| `scripts/check-architecture-contract.test.mjs` | Проверка schema_version=1, interaction_kinds, deny/allow правил слоёв, forbidden кэш-корней. |
| `scripts/check-architecture.mjs` | Константы-владельцы операций, список бизнес-доменов, префиксы платформенных таблиц, логика `backendBoundaryViolations` и извлечения импортов (по доступному фрагменту). Факты за пределами 12k символов не подтверждены. |
| `scripts/check-code-boundaries.mjs` | Паттерны секретов, blanket-подавлений, запрещённые переменные тестов, проверка стилей, проверка tracked generated файлов. |
| `scripts/clean-data.sh` | Удаление данных PostgreSQL, подтверждение DELETE. |
| `scripts/clean-vault.sh` | Удаление `$MAKOSH_HOST_VAULT_HOME`, подтверждение DELETE. |
| `scripts/clean.sh` | Удаление артефактов сборки, временных файлов, логов без БД. |
| `scripts/dev.sh` | Запуск PostgreSQL, backend (bacon), frontend (pnpm dev), сессия логов, health-проверки, cleanup. |
| `scripts/lib/common.sh` | Определение REPO_ROOT, LOG_ROOT, BACKUPS_ROOT, функций логирования, wait_for_http, и т.д. |
| `scripts/lib/env.sh` | Загрузка docker/.env, значения по умолчанию для MAKOSH_*, установка bacon, pnpm install. |
| `scripts/lib/postgres.sh` | docker compose обёртка, postgres_up, wait_for_postgres, data_dir. |
| `scripts/lib/resources.sh` | prepare_backend_sidecar_macos, prepare_google_oauth_resource, prepare_tdlib_macos, функции поиска/сборки tdlib. |
| `scripts/lib/rust-tooling.sh` | require_cargo_subcommand, require_binary. |
| `scripts/logs.sh` | Просмотр live.log через tail. |
| `scripts/migrate.sh` | Запуск cargo run --bin makosh_migrate. |
| `scripts/vault-backup.sh` | pg_dump, копирование vault, manifest.json, RESTORE.txt. |
| `scripts/vault-restore.sh` | Выбор бэкапа, dropdb/createdb, восстановление дампа и vault. |
| `scripts/whatsapp-business-cloud-edge-readiness.mjs` | Live-проверки (healthz, manifest, signed/unsigned POST, readyz), статические проверки контрактов в edge proxy, docker-compose, Dockerfile, .env.example, Makefile, документации. |
| `scripts/whatsapp-domain-closure-audit.mjs` | Проверка Makefile, status.md, тестов, валидация evidence, извлечение NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS, состояние ADR-0101, вычисление closureAchieved. |
| `scripts/whatsapp-live-smoke-collect-evidence.mjs` | Шаблоны, слияние observations, проверка на секреты, валидация evidence. |
| `scripts/whatsapp-live-smoke-evidence.mjs` (truncated) | gate ID, evidenceRefPrefixes, валидация полей, генерация шаблона. Факты за пределами 12k символов не подтверждены. |
| `scripts/whatsapp-live-smoke-readiness.mjs` (truncated) | Зондирование runtime API, статические проверки companion, capabilities. Факты за пределами 12k символов не подтверждены. |
| `scripts/whatsapp-native-md-sdk-gap-readiness.mjs` (truncated) | requiredApis, unsupportedExpectations, анализ wa-rs, wa-rs-appstate, wa-rs-proto. Факты за пределами 12k символов не подтверждены. |

## Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно. Все описанные скрипты ссылаются на внешние файлы (конфигурации, документацию, тесты), но поскольку эти файлы не встроены в данный context pack, невозможно подтвердить или опровергнуть их актуальность. Рекомендуется выполнить полную проверку при наличии полного репозитория.
