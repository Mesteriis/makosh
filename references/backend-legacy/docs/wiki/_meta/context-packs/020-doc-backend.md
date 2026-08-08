# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `020-doc-backend`
- Group / Группа: `backend`
- Role / Роль: `doc`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/documentation-map.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/README.md`
- Size bytes / Размер в байтах: `15975`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Backend

Rust backend for Макошь.

Current scope includes an executable backend foundation with configuration parsing, health/readiness endpoints, V1 status API, canonical event append/read API, event log storage, API access audit logging, host vault onboarding/unlock, legacy database-vault compatibility, Gmail/iCloud/IMAP account setup, secret reference metadata, communication ingestion storage, email sync preflight planning, email provider networking with explicit read/write capability boundaries, fixture email import/export, local mail blob/attachment metadata storage, message/Persona-compatible identity/document projection boundaries, Tantivy search boundary, projection cursors, projection runner batch semantics, graph core projection/read APIs, protected workflow APIs for projects, task candidates, Persona identity review and document processing, and local AI workflow APIs backed by Ollama plus pgvector semantic retrieval. Full MIME parsing, attachment extraction, graph editing, richer graph inference, first-class Polygraph observations and autonomous agent action runtime are not implemented yet.

## Commands

From the repository root:

```sh
make backend-run
make backend-run-dev
make backend-watch-dev
make backend-smoke-dev
make backend-storage-smoke-dev
make backend-secrets-smoke-dev
make backend-event-log-smoke-dev
make backend-communication-smoke-dev
make backend-email-sync-smoke-dev
make backend-email-provider-network-smoke-dev
make backend-email-sync-cache-dev
make backend-email-fixture-export-icloud-dev
make backend-email-fixture-import-dev
make backend-email-fixture-project-dev
make backend-account-setup-smoke-dev
make backend-email-import-smoke-dev
make backend-messages-smoke-dev
make backend-contacts-smoke-dev
make backend-documents-smoke-dev
make backend-graph-smoke-dev
make backend-workflow-smoke-dev
make backend-ai-smoke-dev
make backend-graph-project-dev
make backend-search-smoke-dev
make backend-projection-smoke-dev
make backend-projection-runner-smoke-dev
make backend-events-api-smoke-dev
make backend-v1-api-smoke-dev
make backend-validate
```

`backend-contacts-smoke-dev` is a legacy Makefile target name. It currently
runs the `persons` integration test suite and is kept only for compatibility
until the development command surface is renamed deliberately.

Graph core smoke:

```bash
make backend-graph-smoke-dev
```

This starts the local PostgreSQL container, runs graph store, projection and read API tests with `MAKOSH_TEST_DATABASE_URL`, then stops the Compose PostgreSQL service on exit. Do not run this while relying on the same Compose PostgreSQL service for an active development database session.

workflow smoke:

```bash
make backend-workflow-smoke-dev
```

This starts the local PostgreSQL container, creates isolated temporary databases on the dev PostgreSQL server, and runs the project, project API, project link review, task candidate, task candidate API, Persona identity, Persona identity API, document processing and document processing API integration suites serially. The target is included in `make validate`.

AI smoke:

```bash
make backend-ai-smoke-dev
```

This starts the local PostgreSQL container for pgvector/API integration tests and runs live Ollama validation against `http://192.168.1.2:11434` by default. Override the smoke endpoint with `MAKOSH_AI_SMOKE_OLLAMA_BASE_URL`.

Project current V1 data into the graph tables:

```bash
make backend-graph-project-dev
```

This starts the local PostgreSQL container if needed, applies migrations through the backend storage layer, runs `GraphProjectionService::project_from_v1()` against the current dev database and prints a JSON projection summary. It leaves PostgreSQL running for the active development session and does not connect to Gmail, iCloud or IMAP provider mailboxes.

Export a redacted iCloud IMAP fixture sample from the latest read-only messages:

```bash
MAKOSH_IMAP_FIXTURE_USERNAME=<icloud-email> \
MAKOSH_IMAP_FIXTURE_PASSWORD=<app-password> \
MAKOSH_IMAP_FIXTURE_MAX_MESSAGES=10 \
MAKOSH_IMAP_FIXTURE_OUTPUT=tmp/email-fixtures/icloud-inbox-redacted.json \
make backend-email-fixture-export-icloud-dev
```

The exporter uses `EXAMINE`, `UID SEARCH` and `BODY.PEEK[]` through the same IMAP network client as provider sync. It writes redacted fixture JSON by default, prints only a non-secret summary, and does not import into PostgreSQL. The default output path is under `tmp/`, which is ignored by git.

Import a redacted fixture JSON sample into the local development database:

```bash
make backend-email-fixture-import-dev
```

Project that fixture through canonical messages, Persona-compatible identity
records and graph projection:

```bash
make backend-email-fixture-project-dev
```

Both commands default to `tmp/email-fixtures/icloud-inbox-redacted.json`, create or update the local `dev-icloud-fixture` provider account, print JSON summaries and leave PostgreSQL running for the active development session. Override path and account metadata with `MAKOSH_EMAIL_FIXTURE_PATH`, `MAKOSH_EMAIL_FIXTURE_ACCOUNT_ID`, `MAKOSH_EMAIL_FIXTURE_DISPLAY_NAME`, `MAKOSH_EMAIL_FIXTURE_EXTERNAL_ACCOUNT_ID`, `MAKOSH_EMAIL_FIXTURE_IMPORT_BATCH_ID` and `MAKOSH_EMAIL_FIXTURE_PROVIDER`.

Fetch iCloud/raw IMAP mail into the persistent local development cache:

```bash
MAKOSH_EMAIL_SYNC_USERNAME=<imap-login> \
MAKOSH_EMAIL_SYNC_PASSWORD=<app-password> \
MAKOSH_EMAIL_SYNC_PROVIDER=icloud \
MAKOSH_EMAIL_SYNC_MAX_MESSAGES=25 \
make backend-email-sync-cache-dev
```

The command uses read-only IMAP, writes raw `.eml` blobs under
`docker/data/mail/`, stores only metadata and blob references in PostgreSQL, and
projects canonical messages plus Persona-compatible identity records for the UI.
It does not support Gmail OAuth yet; Gmail cache sync should use the same
pipeline after account setup exposes refreshed access tokens to the dev command.

Direct Cargo commands:

```sh
cargo run --manifest-path backend/Cargo.toml
cargo run --manifest-path backend/Cargo.toml --bin makosh-graph-project
cargo run --manifest-path backend/Cargo.toml --bin makosh-email-fixture-export
cargo run --manifest-path backend/Cargo.toml --bin makosh-email-fixture-dev
cargo run --manifest-path backend/Cargo.toml --bin makosh-email-sync-dev
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

For the normal full-stack development loop, use `make dev` from the repository root. It starts PostgreSQL, runs this backend with auto-restart on Rust/TOML/SQL changes, and starts the SvelteKit frontend with Vite HMR. Backend auto-restart requires either `watchexec` or `cargo-watch`.

## Environment

Supported environment variables:

- `MAKOSH_HTTP_ADDR` - backend bind address, defaults to `127.0.0.1:8080`.
- `MAKOSH_BACKEND_STARTUP_ATTEMPTS` / `MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS`
  - `make dev` backend readiness polling controls, defaulting to 300 attempts
  with 1 second between attempts to allow cold Rust builds after cache cleanup.
- `MAKOSH_FRONTEND_STARTUP_ATTEMPTS` / `MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS`
  - `make dev` frontend readiness polling controls, defaulting to 120 attempts
  with 1 second between attempts.
- `DATABASE_URL` - optional PostgreSQL URL. The current health endpoint does not require a database connection.
- `MAKOSH_LOCAL_API_SECRET` - local shared secret required by the router-level
  guard for protected local API endpoints.
- `MAKOSH_VAULT_HOME` - optional host vault directory; defaults to the local
  Макошь vault home.
- `MAKOSH_DEV_MODE` - enables debug-only host vault development key behavior
  when set to `true`.
- `MAKOSH_DEV_KEY_PATH` - debug-only host vault development key path.
- `MAKOSH_SECRET_VAULT_KEY` - legacy database encrypted vault master key kept
  for migration compatibility only; do not commit, log or persist this value in
  PostgreSQL.
- `MAKOSH_OLLAMA_BASE_URL` - Ollama runtime URL, defaults to `http://127.0.0.1:11434`.
- `MAKOSH_OLLAMA_CHAT_MODEL` - Ollama chat model, defaults to `qwen3:4b`.
- `MAKOSH_OLLAMA_EMBED_MODEL` - Ollama embedding model, defaults to `qwen3-embedding:4b`.
- `MAKOSH_OLLAMA_TIMEOUT_SECONDS` - Ollama request timeout, defaults to `120`.

## Endpoints

- `GET /healthz` - returns backend health status and service name.
- `GET /readyz` - returns readiness status; it is `503` when PostgreSQL is not configured, unavailable or missing required SQLx migrations.
- `GET /api/v1/status` - returns enabled V1 surfaces. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `GET /api/v1/vault/status` - returns host vault initialization/unlock status. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `POST /api/v1/vault/collect-entropy` - records onboarding entropy samples for host vault creation. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `POST /api/v1/vault/create` - creates and unlocks the host vault. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `POST /api/v1/vault/unlock` - unlocks an existing host vault. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `POST /api/v1/vault/recovery/export` - exports recovery material for an unlocked host vault. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `POST /api/v1/vault/recovery/import` - imports recovery material for an existing host vault. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `GET /api/v1/graph/summary` - returns graph node, edge and evidence summary counts. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `GET /api/v1/graph/search` - searches graph nodes by `q` with optional `limit`. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `GET /api/v1/graph/neighborhood` - returns the depth-1 graph neighborhood for `node_id`, including neighboring nodes, edges and evidence. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `POST /api/v1/integrations/mail/accounts/gmail/oauth/start` - starts Gmail OAuth account setup and returns a PKCE authorization URL. Requires local API headers, PostgreSQL and an initialized/unlocked host vault.
- `GET /api/v1/integrations/mail/accounts/gmail/oauth/callback` - displays OAuth callback code/state for the desktop setup flow.
- `POST /api/v1/integrations/mail/accounts/gmail/oauth/complete` - exchanges a Gmail authorization code, stores credential payloads in the host vault and creates provider account bindings. Requires local API headers, PostgreSQL and an initialized/unlocked host vault.
- `POST /api/v1/integrations/mail/accounts/imap` - creates iCloud/raw IMAP account metadata and stores password/app-password payloads in the host vault. Requires local API headers, PostgreSQL and an initialized/unlocked host vault.
- `POST /api/v1/events` - appends a canonical event through the application/API boundary. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `GET /api/v1/events/{event_id}` - loads a canonical event by ID. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.
- `GET /api/v1/audit/events` - returns event API audit records. Supports `target_id`, `actor_id`, `after_audit_id` and `limit` query parameters. Requires `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.

Authorized event API calls are recorded in `api_audit_log` with the constant `makosh-frontend` actor. The API secret value is never stored.

## Workflow APIs

Available endpoints below require `X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`.

- `GET /api/v1/projects` - lists local project records with derived stats.
- `GET /api/v1/projects/{project_id}` - returns project detail, timeline, messages, documents and people.
- `GET /api/v1/projects/{project_id}/link-candidates` - returns safe project message/document link candidates.
- `PUT /api/v1/projects/{project_id}/link-reviews` - records project link review state as a canonical event.
- `GET /api/v1/task-candidates` - lists source-backed task candidates.
- `PUT /api/v1/task-candidates/{task_candidate_id}/review` - records task ca
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
