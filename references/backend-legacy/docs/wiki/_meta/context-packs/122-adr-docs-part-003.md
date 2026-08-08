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

- Chunk ID / ID чанка: `122-adr-docs-part-003`
- Group / Группа: `docs`
- Role / Роль: `adr`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `decisions/adr-index.md`

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

### `docs/adr/ADR-0051-v5-whatsapp-web-companion-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0051-v5-whatsapp-web-companion-boundary.md`
- Size bytes / Размер в байтах: `4689`
- Included characters / Включено символов: `4689`
- Truncated / Обрезано: `no`

```markdown
# ADR-0051 V5 WhatsApp Web Companion Boundary

Status: Proposed

## Context

Version 5 adds WhatsApp as a long-term communication memory source. The reliable integration path is materially different from email and Telegram:

- WhatsApp Web and desktop are linked-device companion experiences, not documented personal-account APIs.
- The official WhatsApp Business Platform Cloud API is for business messaging and requires Meta business assets such as a business portfolio, WhatsApp Business Account and business phone number.
- WhatsApp's consumer terms restrict unauthorized automated access, impermissible collection and unauthorized software or APIs that function like the service.
- Макошь is local-first and personal. It must preserve user-controlled local memory without turning WhatsApp Web into a hidden scraping or automation channel.

References checked during this decision:

- WhatsApp Terms of Service, "Acceptable Use Of Our Services".
- Meta's WhatsApp Cloud API Postman collection, which identifies Cloud API as the official WhatsApp Business Platform API and lists business onboarding requirements.

## Decision

Implement personal WhatsApp support through an explicit `whatsapp_web` companion boundary.

Rules:

- `whatsapp_web` is a communication provider kind for a user-linked local companion session.
- The first implementation path is fixture/manual companion state only. Live WhatsApp Web sessions must report `blocked` until a visible desktop runtime, session lifecycle, permission prompts, local storage policy and smoke validation exist.
- The companion runtime must be user-visible through the desktop shell or an explicitly controlled browser/WebView. Hidden headless scraping is not an accepted provider adapter.
- PostgreSQL stores account metadata, session status, raw record metadata, checkpoints, canonical projections and audit records. It must not store WhatsApp Web session secrets, pairing material or local browser profile secrets.
- Local WhatsApp Web session state lives under ignored local data paths such as `docker/data/whatsapp/<account_id>/` for development, encrypted where the runtime supports it.
- WhatsApp Web credentials and local session protection material are resolved by `account_id + secret_purpose`, using `whatsapp_web_session_key` for session encryption/protection. Provider kind alone must never select credentials.
- Raw WhatsApp Web records are append-only with provider provenance. The initial raw message record kind is `whatsapp_web_message`.
- Checkpoints are account-scoped. Initial stream IDs should be delimiter-safe, for example `whatsapp_web:global` or `whatsapp_web:<provider_chat_id>`.
- Canonical message projections may use `channel_kind = 'whatsapp_web'`.
- Outbound live sends are not part of the first V5 foundation. Future sends require the same policy, template, actor, audit and capability discipline established for Telegram automation, plus WhatsApp-specific validation.
- WhatsApp Business Platform Cloud API is a separate future provider shape, not a substitute for personal WhatsApp Web. If added, it should use a distinct provider kind such as `whatsapp_business_cloud` and its own ADR/update.

## Consequences

Positive:

- Макошь can model WhatsApp Web without pretending that an unofficial stable personal API exists.
- The provider boundary keeps WhatsApp Web runtime fragility away from canonical messages, graph projections and AI workflows.
- Fixture/manual state allows backend and UI work to start without live WhatsApp credentials or hidden browser automation.
- Session state and secrets stay outside PostgreSQL, preserving backup and debugging safety.

Negative:

- V5 cannot claim live WhatsApp Web sync until the visible runtime and safety checks exist.
- Companion sessions may remain more fragile than email, Telegram TDLib or future official business APIs.
- Manual linking, revocation and degraded-session UX become first-class product work.

Risk handling:

- Capability reporting must distinguish fixture/manual readiness from blocked live runtime.
- Provider adapters must never log message bodies, session secrets, pairing codes or browser profile paths containing private state.
- Tests must cover idempotent raw records, account-scoped credential lookup and refusal of live sends before any live runtime is enabled.

## Non-Goals

- Hidden WhatsApp Web scraping.
- Reverse engineering WhatsApp protocols as a production dependency.
- Bulk messaging, auto-messaging or auto-dialing.
- Live outbound WhatsApp sends in the V5 foundation slice.
- Replacing personal WhatsApp Web with WhatsApp Business Platform Cloud API.
- Training or fine-tuning models on WhatsApp data.
```

### `docs/adr/ADR-0052-capability-runtime-and-action-confirmation-policy.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0052-capability-runtime-and-action-confirmation-policy.md`
- Size bytes / Размер в байтах: `4386`
- Included characters / Включено символов: `4386`
- Truncated / Обрезано: `no`

```markdown
# ADR-0052 Capability Runtime and Action Confirmation Policy

Status: Proposed

## Context

ADR-0027 selects a capability-based permission model, but intentionally leaves the runtime shape open. ADR-0038, ADR-0039 and ADR-0040 add temporary local API protection, access audit and actor identity for the current single-user desktop implementation.

V3, V4 and V5 now expose source-backed AI, Telegram automation dry-runs and WhatsApp Web companion capability reporting. Live provider writes, destructive actions, plugin execution and secret access still need a concrete policy boundary before they can move from blocked capability states to available runtime behavior.

## Decision

Implement the long-term capability runtime around a backend application-layer policy boundary.

Rules:

- Capability checks are centralized in the backend application boundary before privileged reads, local writes, provider writes, destructive actions, exports, secret resolution, automation execution or plugin tool calls.
- UI, agent and plugin clients may present intent, but they do not authorize their own actions.
- The temporary `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>` and `X-Макошь-Actor-Id` headers remain valid only as local-development and desktop bootstrap guards until the capability runtime replaces them with authenticated actor and capability identifiers.
- Capability decisions classify requested actions as `read`, `local_write`, `provider_write`, `destructive`, `export`, `secret_access` or `automation`.
- Capability grants are scoped. Scopes may include actor, provider account, channel/chat/thread, project, document, data class, command, template, automation policy, time window, rate limit and expiry.
- Message sends, provider mutations, deletes, destructive local changes, sensitive exports and direct secret access require explicit confirmation unless an enabled scoped automation policy authorizes the action.
- Automation policies are never open-ended. They must bind account, destination scope, template, trigger, rate limit, quiet hours and expiry.
- AI may fill declared template variables inside an authorized policy, but it cannot choose destination, account, template, policy authority or send scope from retrieved content.
- Allowed and rejected high-risk actions write audit metadata with actor, capability, action class, target scope, policy/template identifiers where relevant, decision, reason and correlation ID.
- Audit metadata must not store API tokens, provider credentials, private message bodies, document contents, pairing codes or local browser profile paths containing private state.
- For external side effects and destructive actions, audit insertion is fail-closed: if the audit record cannot be written, the action is not executed.
- Plugins are untrusted by default. Plugin activation requires a declarative capability manifest, user-visible permissions and scoped data views. Plugins cannot access raw secrets or canonical tables directly.

## Consequences

Positive:

- Live Telegram, WhatsApp, plugin and future provider actions get one policy model instead of separate ad hoc confirmation checks.
- The UI can expose clear capability and confirmation states without being the source of authority.
- Audit records can explain why high-risk actions were allowed or rejected without leaking private content.
- Temporary local token and actor headers remain usable during development while their replacement boundary is defined.

Negative:

- Capability evaluation becomes security-critical application infrastructure.
- Provider adapters and agent tools must route through the policy boundary before side effects.
- Existing V4/V5 blocked live capabilities remain blocked until the runtime, persistence, UI and validation are implemented.

Risk handling:

- Do not enable live outbound sends, deletes, provider mutations, sensitive exports or plugin side effects until capability checks and high-risk audit records are covered by regression tests.
- Treat confirmation text, retrieved content and plugin manifests as untrusted input.
- Keep secret values out of capability grants, audit records, event payloads and test fixtures.

## Non-Goals

- Multi-user remote access.
- Cloud identity provider integration.
- Third-party plugin code execution.
- Replacing ADR-0038, ADR-0039 or ADR-0040 in the current local bootstrap implementation.
```

### `docs/adr/ADR-0053-database-backed-encrypted-secret-vault.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0053-database-backed-encrypted-secret-vault.md`
- Size bytes / Размер в байтах: `4319`
- Included characters / Включено символов: `4319`
- Truncated / Обрезано: `no`

```markdown
# ADR-0053 Database-Backed Encrypted Secret Vault

Status: Superseded by ADR-0076

This decision was superseded by ADR-0076, which moves new encrypted secret payloads out of PostgreSQL into a dedicated host vault under `~/.makosh/vault` while leaving PostgreSQL with non-secret metadata and account bindings.

Supersedes: ADR-0016, ADR-0042, ADR-0044

## Context

The previous secret model kept provider credential values outside PostgreSQL. `secret_references` stored metadata and account bindings, while ADR-0044 placed encrypted secret values in a local JSON vault file selected by `MAKOSH_SECRET_VAULT_PATH`.

That split makes backup, restore and local operational state harder than the rest of the local-first system: provider account metadata lives in PostgreSQL, but the encrypted credential payloads live in a separate file that must be moved and restored independently.

The goal is to move provider credential payload storage into PostgreSQL without storing plaintext secrets in ordinary application tables, provider config, event payloads, audit records, tests or documentation.

## Decision

Use a dedicated PostgreSQL-backed encrypted secret vault for provider credentials.

Rules:

- `secret_references` remains the non-secret metadata table and keeps stable `secret_ref` identifiers, labels, secret kinds and store kinds.
- Communication provider accounts continue to bind credentials through `communication_provider_account_secret_refs` by account ID and secret purpose.
- New account setup writes provider credential values to `encrypted_secret_vault_entries` and marks the corresponding `secret_references.store_kind` as `database_encrypted_vault`.
- `encrypted_secret_vault_entries` stores only encrypted payload material: `secret_ref`, KDF identifier, salt, nonce, ciphertext and timestamps.
- Plaintext provider credentials, OAuth token bundles, app passwords, mailbox passwords, API tokens and private keys must never be stored in provider account config, secret reference metadata, event payloads, audit records, logs, tests or docs.
- `MAKOSH_SECRET_VAULT_KEY` remains outside PostgreSQL and is required to decrypt database vault entries. It must not be logged, committed or persisted in PostgreSQL.
- Hardware identifiers such as CPU, board or disk serial numbers are not valid vault keys. They are non-secret, may be unavailable or unstable, and may only be used as non-secret binding context if an OS-backed key resolver later needs it.
- `MAKOSH_SECRET_VAULT_PATH` is no longer required for account setup. File-backed encrypted vault code may exist only as a legacy compatibility or explicit local migration utility, not as the primary write path.
- Database vault entries use per-entry AES-256-GCM encryption with an Argon2id-derived key, random per-entry salt, random nonce and authenticated `secret_ref` associated data.
- Database backups now include encrypted credential payloads. Restores require the matching external `MAKOSH_SECRET_VAULT_KEY`.
- SQL migrations must not attempt to decrypt or import existing file-vault secrets, because migrations do not have a safe credential/key interaction boundary. Any file-vault import must be an explicit trusted local operation.

## Consequences

Positive:

- PostgreSQL backup and restore can carry encrypted credential payloads with the rest of local state.
- Provider credential lookup keeps the existing account-scoped `secret_ref` boundary.
- Database compromise does not expose plaintext credentials without the external vault key.
- Account setup no longer depends on a separate vault file path.

Negative:

- PostgreSQL backups now contain high-value ciphertext and require stricter handling.
- Losing `MAKOSH_SECRET_VAULT_KEY` makes encrypted database vault entries unrecoverable.
- The database vault becomes security-critical persistence code.
- Existing file-vault installations need an explicit migration/import workflow before old `encrypted_vault` references are fully moved.

Risk handling:

- Keep the vault key outside PostgreSQL and outside committed files.
- Treat encrypted vault entries as sensitive backup material even though they are ciphertext.
- Preserve `SecretResolver` as the only runtime plaintext access boundary.
- Add regression coverage for ciphertext storage, wrong-key rejection and account setup writes.
```

### `docs/adr/ADR-0054-application-settings-store.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0054-application-settings-store.md`
- Size bytes / Размер в байтах: `3551`
- Included characters / Включено символов: `3551`
- Truncated / Обрезано: `no`

```markdown
# ADR-0054 Application Settings Store

Status: Proposed

## Context

Макошь has a growing number of local runtime and UI preferences. Environment variables are acceptable for bootstrap values, but they are not a good product surface for settings that a desktop user should inspect or change from the app.

The application also has provider accounts for email, Telegram, WhatsApp and future communication channels. Those accounts are durable domain records, not generic key-value settings, because they have provider kinds, account-scoped secret bindings and adapter-specific metadata.

## Decision

Store user-editable non-secret runtime and UI settings in PostgreSQL `application_settings`.

Rules:

- `application_settings` stores declared settings only. The UI and API may update existing keys but must not create arbitrary new keys.
- Setting values are typed JSONB values with `value_kind` of `boolean`, `integer`, `string` or `json`.
- Setting keys must not be secret-like. Keys containing `secret`, `password`, `token`, `credential` or `private_key` are rejected.
- Secret material remains under ADR-0053 and must not be placed in `application_settings`.
- Bootstrap values that are required before PostgreSQL is reachable remain outside this table. This includes `DATABASE_URL`, the temporary local API token and `MAKOSH_SECRET_VAULT_KEY`.
- The Settings UI should expose all declared non-secret runtime and UI settings except database connectivity. Bootstrap or restart-only settings may be stored as declared settings, but the UI must make that operational status visible.
- Provider accounts remain in provider/account tables such as `communication_provider_accounts` and are surfaced in the Settings UI as account records, not duplicated into `application_settings`.
- Settings writes go through protected backend endpoints and write audit metadata without storing setting values in audit records.
- AI/Ollama runtime settings are read from `application_settings` when PostgreSQL is available, with environment defaults retained only as bootstrap/fallback values.
- Backend startup must verify and repair the declared settings table after migrations and before serving API traffic. Repair recreates the settings table when it is missing, inserts missing declared rows, restores declared metadata/type/labels and resets invalid values to declared defaults.
- The API and UI expose only declared settings. Extra rows inserted manually into `application_settings` are ignored rather than becoming a supported configuration surface.

## Consequences

Positive:

- The desktop UI can expose a real settings tab backed by durable local state.
- Runtime settings can be backed up with PostgreSQL and changed without editing `.env`.
- Provider accounts stay attached to their existing secret reference and adapter boundaries.
- `docker/.env` is reduced toward bootstrap and development infrastructure values instead of becoming the product settings surface.

Negative:

- Settings become part of schema evolution and need migration coverage.
- Invalid settings can break runtime features if validation is too permissive.
- Bootstrap settings still need a separate operational surface.

Risk handling:

- Keep the initial settings allowlist small and typed.
- Validate updates against stored type and metadata constraints.
- Treat startup repair as idempotent and non-secret: it must never write credential values, local API tokens or vault keys.
- Do not audit setting values.
- Do not store account credentials in account config or settings.
```

### `docs/adr/ADR-0055-full-email-provider-networking.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0055-full-email-provider-networking.md`
- Size bytes / Размер в байтах: `3243`
- Included characters / Включено символов: `3241`
- Truncated / Обрезано: `no`

```markdown
# ADR-0055 Full Email Provider Networking (Read + Write)

Status: Accepted

Supersedes: ADR-0043

## Context

ADR-0043 mandated read-only email provider networking as a temporary safety measure during the initial implementation phase. Макошь has now matured to a point where full email functionality is required: the system must send emails, reply to threads, forward messages, and mutate server-side state (flags, labels, mailbox moves, deletions).

The read-only restriction was always intended to be temporary for a personal local-first system. The owner controls their own data and provider credentials. Макошь is not a multi-tenant SaaS — there is no risk of one user mutating another user's mailbox.

## Decision

Email provider networking supports both read and write operations.

### Read operations (unchanged from ADR-0043)
- Gmail: `users.messages.list`, `users.messages.get` with `format=raw`
- IMAP: `EXAMINE` or `SELECT`, `UID SEARCH`, `UID FETCH BODY.PEEK[]`

### New write operations
- **SMTP sending**: send email through provider SMTP with credentials resolved at runtime
- **IMAP flag mutations**: `UID STORE` for +FLAGS/-FLAGS (Seen, Answered, Flagged, Deleted, Draft)
- **IMAP mailbox mutations**: `UID COPY` + `UID STORE +FLAGS (\Deleted)` + `EXPUNGE` for move/delete
- **Gmail mutations**: `users.messages.modify` for label changes, `users.messages.trash`, `users.messages.send`
- **Gmail drafts**: `users.drafts.create`, `users.drafts.update`, `users.drafts.send`

### Read-only restriction retained only for tests
- Automated integration tests (`backend/tests/`) must use read-only paths where possible to avoid mutating real provider state. Test-specific adapters may use fixture/mock networking.
- IMAP integration tests must continue using `EXAMINE`.
- Gmail API integration tests must use read-only scopes.

### Secret safety (unchanged)
- OAuth tokens, app passwords, and mailbox passwords remain behind the secret boundary from ADR-0016.
- Secret values must not be stored in raw payloads, provenance, checkpoint JSON, logs, or errors.
- Credential lookup must use `account_id` plus secret purpose, never provider kind alone.

### Provider adapter boundary
- Each provider adapter exposes both read and write capabilities.
- Write operations require explicit user action (no auto-send without confirmation).
- The capability runtime from ADR-0052 governs which actions are allowed without confirmation.
- SMTP credentials are stored as separate secret entries with purpose `smtp_password`.

## Consequences

- Макошь gains full email client functionality: compose, reply, forward, flag management, mailbox organization.
- IMAP provider adapters must handle both `EXAMINE` (read) and `SELECT` (read-write) modes.
- SMTP networking introduces a new transport layer alongside existing IMAP/Gmail API clients.
- Test infrastructure must clearly separate read-only fixture tests from optional write-path integration tests.
- Provider account setup must capture SMTP configuration alongside IMAP/Gmail API config.
- The email sync pipeline must distinguish between read-only sync and user-initiated write operations.
- Send operations must be audited through the existing `api_audit_log` infrastructure.
```

### `docs/adr/ADR-0056-local-api-simplified-auth.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0056-local-api-simplified-auth.md`
- Size bytes / Размер в байтах: `2267`
- Included characters / Включено символов: `2263`
- Truncated / Обрезано: `no`

````markdown
# ADR-0056 Local API — Simplified Auth

Status: Accepted

Supersedes: ADR-0037, ADR-0038, ADR-0040

## Context

The backend serves a **single local user** via a desktop app (Tauri shell).
There is no multi-tenancy, no external network exposure (binds `127.0.0.1`),
and no user-facing authentication.

Previous ADRs mandated per-request `MAKOSH_LOCAL_API_TOKEN` verification
and `x-makosh-actor-id` extraction in every handler. This added boilerplate
to 200+ handlers with zero security benefit for a single-user local app.

## Decision

### 1. Router-level secret check

A single `tower::layer` on the router verifies a shared secret header.
If the header is missing or wrong → 403. No per-handler auth code.

```rust
Router::new()
    .layer(require_secret_layer("X-Макошь-Secret", &secret))
    .route(...)
```

### 2. Actor identity is a constant

All audit records use `"makosh-frontend"` as the actor.
No `x-makosh-actor-id` header extraction.

```rust
NewApiAuditRecord::setting_set("makosh-frontend", "theme")
```

### 3. Handlers are plain

```rust
pub async fn list(State(state): State<AppState>) -> Result<Json<T>, ApiError> {
    let store = XStore::new(state.db.pool()?.clone());
    Ok(Json(store.list().await?))
}
```

No `verify_local_api_capability`, no `AuthActor`, no `require_auth`.

## Consequences

### Removed
- `MAKOSH_LOCAL_API_TOKEN` configuration
- `x-makosh-actor-id` header requirement
- `verify_local_api_capability()` function
- `local_api_actor()` function
- `LocalApiActor` struct
- `ApiError::ApiTokenNotConfigured`, `ApiError::InvalidApiToken`, `ApiError::InvalidActorId`

### Added
- `tower::layer` with shared secret check (one place)
- `audit_actor` constant or helper

### Migration
1. Remove token config from `AppConfig`, `docker/.env`
2. Remove token/actor verification from all handlers
3. Replace `actor.actor_id` with `"makosh-frontend"` in audit calls
4. Add router-level secret layer
5. Delete `verify_local_api_capability`, `local_api_actor`, `is_valid_actor_id_byte`, `LocalApiActor`

### Risk
- A malicious process on the same machine could call the API if it knows the secret.
  Mitigation: the secret is in an env var, Tauri IPC provides additional isolation.
  This is acceptable for a single-user desktop app.
````

### `docs/adr/ADR-0057-person-memory-and-provenance.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0057-person-memory-and-provenance.md`
- Size bytes / Размер в байтах: `1168`
- Included characters / Включено символов: `1168`
- Truncated / Обрезано: `no`

```markdown
# ADR-0057 Person Memory and Provenance System

Status: Proposed

## Context

The functional spec requires every AI-extracted or discovered fact about a person to carry provenance: source, confidence, and verification timestamp. Ad-hoc storage in JSON columns or free-text notes breaks auditability and prevents systematic conflict detection and memory decay.

## Decision

Store all facts, memory cards, preferences, and expertise in dedicated domain tables (`person_facts`, `person_memory_cards`, `person_preferences`, `person_expertise`) with mandatory `source`, `confidence`, and `last_verified_at` columns. The enrichment engine writes through these tables and never mutates the person profile directly. Memory decay is a scheduled projection that lowers confidence for unverified facts older than a threshold.

## Consequences

- Every fact is traceable to its source.
- Knowledge conflicts (contradictory facts from different sources) are detectable.
- Memory decay provides automatic staleness detection.
- Snapshot-based history diff is possible via `person_snapshots`.
- Relationship timeline events (`relationship_events`) form an event-sourced projection.
```

### `docs/adr/ADR-0058-person-enrichment-engine.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0058-person-enrichment-engine.md`
- Size bytes / Размер в байтах: `1023`
- Included characters / Включено символов: `1023`
- Truncated / Обрезано: `no`

```markdown
# ADR-0058 Person Enrichment Engine Boundary

Status: Proposed

## Context

The functional spec describes automated enrichment from GitHub, LinkedIn, public web, and internal communication analysis. Enrichment must be auditable, reversible, and never silently overwrite user-confirmed data.

## Decision

A dedicated `EnrichmentEngine` service orchestrates data acquisition with pluggable providers. All results go through `enrichment_results` with status tracking (`pending`, `applied`, `rejected`, `conflict`). The engine never auto-applies enrichment without user confirmation for medium/low confidence results. High-confidence facts from verified sources may auto-apply with audit trail. Profile verification uses cross-source correlation.

## Consequences

- Enrichment is auditable and reversible.
- Providers (GitHub, LinkedIn, web) are pluggable behind a common trait.
- User retains control over what data enters the person profile.
- Auto-discovery from email domain probes GitHub, LinkedIn, and company website.
```

### `docs/adr/ADR-0059-person-communication-dna.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0059-person-communication-dna.md`
- Size bytes / Размер в байтах: `1323`
- Included characters / Включено символов: `1323`
- Truncated / Обрезано: `no`

```markdown
# ADR-0059 Person Communication DNA and Personas

Status: Superseded by ADR-0084

Superseded because ADR-0084 makes Persona the root domain entity. Communication
patterns remain part of Persona Intelligence, but `person_personas` as nested
interaction contexts is no longer the target domain model.

## Context

The functional spec distinguishes between Roles (who the person is to the user) and Personas (how the user interacts in a specific context). Communication DNA captures the person's natural style independently of any persona: formality, verbosity, technical depth, call preference, and response patterns.

## Decision

Store Communication DNA as typed columns on the `persons` table (`communication_style`, `verbosity`, `technical_depth`, `question_frequency`, `call_preference`, `response_pattern`, `active_hours`, `active_days`). Personas live in `person_personas` as named interaction contexts with their own tone, language, and channel preferences. The `PersonIntelligenceService` computes DNA from message corpus with heuristic fallback and optional LLM refinement via Ollama.

## Consequences

- DNA is always available even when Ollama is offline (heuristic computation).
- Personas override DNA defaults during compose/reply flows.
- DNA columns are nullable; missing values indicate uncomputed profile.
```

### `docs/adr/ADR-0060-person-timeline-and-graph-integration.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0060-person-timeline-and-graph-integration.md`
- Size bytes / Размер в байтах: `1242`
- Included characters / Включено символов: `1242`
- Truncated / Обрезано: `no`

```markdown
# ADR-0060 Person Timeline and Graph Integration

Status: Proposed

## Context

Relationship events (first message, contract signed, invoice paid, etc.) form a timeline that must be queryable and rebuildable. The relationship map and mutual connections views need to surface graph relationships between persons, projects, documents, and other entities.

## Decision

Store timeline events in `relationship_events` with optional links to source entities (`related_entity_id`, `related_entity_kind`). The timeline is a rebuildable projection materialized from communication history and document metadata. Graph integration uses existing `graph_nodes`/`graph_edges` tables from ADR-0045 with new relationship types (`person_has_identity`, `person_works_at_organization`, `person_has_expertise`, `person_involved_in_project`). Relationship map and mutual connections are graph traversal queries, not separate storage.

## Consequences

- Timeline is queryable by event type, date range, and related entity.
- History diff works by comparing `person_snapshots` across dates.
- Graph traversal depth is intentionally limited; complex queries use application-layer joins.
- No new graph tables; persons participate in the existing graph projection.
```

### `docs/adr/ADR-0061-organization-as-first-class-entity.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0061-organization-as-first-class-entity.md`
- Size bytes / Размер в байтах: `1022`
- Included characters / Включено символов: `1022`
- Truncated / Обрезано: `no`

```markdown
# ADR-0061 Organization as First-Class Domain Entity

Status: Proposed

## Context

The persons module has `organization_reference` as a free-text field. The functional spec requires organizations as independent entities with their own identities, memory, timeline, contacts, and enrichment. A string field cannot support this.

## Decision

Organizations are first-class domain entities with `organization_id = org:v1:{nanos}`. The `organization_contact_links` table provides many-to-many linkage between persons and organizations with role, department, and primary-flag semantics. The `organization_reference` field on persons becomes a cached value derived from the primary active link.

## Consequences

- 27 tables under the organizations domain.
- `organization_contact_links` enables one person to belong to multiple organizations with different roles.
- Person merge/split may trigger organization contact link reconciliation.
- The free-text `organization_reference` field is retained for backward compatibility.
```

### `docs/adr/ADR-0062-organization-identity-and-resolution.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0062-organization-identity-and-resolution.md`
- Size bytes / Размер в байтах: `1071`
- Included characters / Включено символов: `1069`
- Truncated / Обрезано: `no`

```markdown
# ADR-0062 Organization Identity and Resolution

Status: Proposed

## Context

Organizations have multiple identifiers: domains, VAT/CIF/NIF numbers, GitHub orgs, LinkedIn pages, phone numbers, and portal URLs. These must be stored with provenance and used for deduplication. ADR-0019 established identity resolution for persons; organizations need the same capability.

## Decision

`organization_identities` stores all identifiers with type, value, source, confidence, and status. Identity resolution compares organizations by domain overlap, VAT match, legal name similarity, and shared contacts. Merge candidates are generated with confidence scoring and user-confirmed. Merge is reversible — confirming a merge materializes a split candidate.

## Consequences

- Domain intelligence enables automatic linking of emails and contacts to organizations.
- VAT/VIES validation provides high-confidence identity matching.
- Aliases (`organization_aliases`) capture brand/trading/former names.
- Merge/split workflow mirrors the person identity resolution from ADR-0019.
```

### `docs/adr/ADR-0063-organization-passive-osint-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0063-organization-passive-osint-boundary.md`
- Size bytes / Размер в байтах: `1135`
- Included characters / Включено символов: `1135`
- Truncated / Обрезано: `no`

```markdown
# ADR-0063 Organization Passive OSINT Boundary

Status: Proposed

## Context

The functional spec requires enrichment from public sources: website, GitHub, LinkedIn, VIES, public registries. This must be done without active scanning, brute force, or access control bypass.

## Decision

Enrichment uses only public APIs and passive observation. Providers include: website (about page, contact info), VIES (VAT validation), GitHub (public org profile), LinkedIn (public company page), public registries. All results go through `organization_enrichment_results` with pending/applied/rejected/conflict status. Auto-apply only for high-confidence results from verified sources. The spec explicitly forbids: active infrastructure scanning, brute force, access control bypass, closed data collection, pentest, mass scraping without control.

## Consequences

- Enrichment is auditable through the results table.
- User retains control over what data enters the organization profile.
- VAT/VIES validation provides high-confidence legal identity verification.
- Technology profile and open source footprint are derived from public data only.
```

### `docs/adr/ADR-0064-organization-memory-and-provenance.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0064-organization-memory-and-provenance.md`
- Size bytes / Размер в байтах: `1056`
- Included characters / Включено символов: `1056`
- Truncated / Обрезано: `no`

```markdown
# ADR-0064 Organization Memory and Provenance

Status: Proposed

## Context

Organizations accumulate facts, memory cards, preferences, and timeline events. Like persons (ADR-0057), every piece of information must carry provenance: source, confidence, and verification timestamp.

## Decision

Store facts in `organization_facts`, memory cards in `organization_memory_cards`, preferences in `organization_preferences`, and timeline events in `organization_timeline_events`. All carry mandatory `source`, `confidence`, and `last_verified_at` columns. Memory decay lowers confidence for unverified facts. Snapshots (`organization_snapshots`) enable history diff. Knowledge conflicts are detected and surfaced in `organization_knowledge_conflicts`.

## Consequences

- Every organizational fact is traceable to its source.
- Required documents (`organization_required_documents`) track what documents an organization typically needs.
- Timeline is rebuildable from communication history and document metadata.
- History diff works by comparing two snapshots.
```

### `docs/adr/ADR-0065-organization-portals-procedures-playbooks.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0065-organization-portals-procedures-playbooks.md`
- Size bytes / Размер в байтах: `1298`
- Included characters / Включено символов: `1292`
- Truncated / Обрезано: `no`

```markdown
# ADR-0065 Organization Portals, Procedures, and Playbooks

Status: Proposed

## Context

Organizations often require interaction through specific portals (tax, banking, support), follow defined procedures (tax filing, contract signing), and can benefit from automated playbooks (email received → check deadline → create task). These are organization-specific knowledge that should be stored and surfaced.

## Decision

Portals (`organization_portals`) store URLs, portal types, login hints, and secret references (not the secrets themselves). Procedures (`organization_procedures`) store named workflows as JSONB step arrays. Playbooks (`organization_playbooks`) store automated scenarios with triggers, steps, and approval modes. Templates (`organization_templates`) store email/document templates specific to an organization. In v1, playbooks are stored as data but not automatically executed — execution is a future concern.

## Consequences

- Portals provide quick access links to external systems with login context.
- Procedures turn ad-hoc processes into repeatable, documented workflows.
- Playbooks define what should happen but require explicit user action or future automation runtime.
- Quick actions (`organization_quick_actions`) provide one-click shortcuts for common tasks.
```

### `docs/adr/ADR-0066-organization-graph-integration.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0066-organization-graph-integration.md`
- Size bytes / Размер в байтах: `1076`
- Included characters / Включено символов: `1076`
- Truncated / Обрезано: `no`

```markdown
# ADR-0066 Organization Graph Integration

Status: Proposed

## Context

Organizations must participate in the knowledge graph to surface relationships between organizations, persons, documents, projects, and domains. ADR-0045 established the graph core projection with PostgreSQL tables.

## Decision

Organizations participate in the existing `graph_nodes`/`graph_edges` tables. New relationship types: `org_has_domain`, `org_has_contact`, `org_has_document`, `org_involved_in_project`, `org_parent_of` (parent/subsidiary). The organization graph, relationship map, and mutual connections are read-side graph traversal queries, not separate storage. The `related_organizations` table provides explicit parent/subsidiary/division/partner relationships with provenance.

## Consequences

- No new graph tables; organizations reuse the existing graph infrastructure.
- Related organizations are queryable both through the graph and through the direct `related_organizations` table.
- Graph traversal depth is intentionally limited; complex queries use application-layer joins.
```

### `docs/adr/ADR-0067-calendar-multi-provider-architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0067-calendar-multi-provider-architecture.md`
- Size bytes / Размер в байтах: `1500`
- Included characters / Включено символов: `1500`
- Truncated / Обрезано: `no`

```markdown
# ADR-0067 Calendar as First-Class Domain with Multi-Provider Architecture

Status: Proposed

## Context

Макошь needs a calendar module that treats events as first-class knowledge graph nodes, not as isolated time blocks. Events must connect to persons, organizations, projects, documents, tasks, and emails. The system must support multiple calendar providers (Google, Microsoft, Apple, CalDAV, ICS, local) with multiple accounts per provider and multiple calendars per account.

## Decision

Calendar is a first-class domain with `calendar_account_id` = `cal:v1:{uuid}`. Each account binds to a provider with a capabilities model (read/write/delete/recurring/attendees/conference/attachments/reminders/availability/colors/push_sync). Events carry full source identity (`provider` + `account_id` + `source_id` + `source_event_id`). The `calendar_sources` table models individual calendars within an account with per-source sync and read-only flags.

Provider sync is deferred to a future phase (requires OAuth integration similar to email providers). The schema, API, and intelligence layer are fully implemented and ready for provider adapters.

## Consequences

- Events are queryable by account, source, time range, status, and type.
- Each event knows exactly which provider/account/calendar it belongs to.
- Multiple accounts and calendars are supported from day one.
- Provider sync implementation is scoped to a future task; the domain is usable with locally-created events immediately.
```

### `docs/adr/ADR-0068-calendar-event-as-graph-node.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0068-calendar-event-as-graph-node.md`
- Size bytes / Размер в байтах: `1320`
- Included characters / Включено символов: `1320`
- Truncated / Обрезано: `no`

```markdown
# ADR-0068 Calendar Event as Knowledge Graph Node

Status: Proposed

## Context

Calendar events in Макошь must not be isolated time-block rectangles. Each event is a system node connected to persons, organizations, projects, documents, tasks, emails, and notes. ADR-0045 established the graph core projection. Events need explicit, queryable relationships.

## Decision

Events participate in the knowledge graph through `event_relations` with `entity_type`/`entity_id`/`relation_type`. Supported entity types: `person`, `organization`, `project`, `document`, `task`, `email`, `note`, `decision`, `obligation`, `recording`. Event-participant links are stored in `event_participants` with resolved person references, email, role, and response status.

Event context packs aggregate related data (documents, tasks, open questions, risks, suggested agenda, suggested actions) into a materialized JSONB snapshot for fast retrieval.

## Consequences

- Events are traversable from any related entity through the graph.
- Context packs provide instant context without cross-domain joins at read time.
- Participants are first-class with person resolution, enabling participant intelligence.
- Graph integration (`graph_nodes`/`graph_edges`) is read-through and deferred until the graph projection is updated for event nodes.
```

### `docs/adr/ADR-0069-calendar-intelligence-heuristic-fallbacks.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0069-calendar-intelligence-heuristic-fallbacks.md`
- Size bytes / Размер в байтах: `1696`
- Included characters / Включено символов: `1682`
- Truncated / Обрезано: `no`

```markdown
# ADR-0069 Calendar Intelligence Layer with Heuristic Fallbacks

Status: Proposed

## Context

The calendar module needs event classification, importance scoring, readiness assessment, risk detection, meeting briefs, agenda generation, and natural-language search. Ollama is available but must not be mandatory per ADR-0009 (local AI through Ollama) and ADR-0022 (no fine-tuning on private data).

## Decision

All intelligence features use deterministic heuristics as primary implementation with optional Ollama refinement. Heuristics cover:
- **Event classification** — keyword analysis of title, participant count, duration
- **Importance scoring** — urgency keywords, participant count, project/deadline presence
- **Readiness scoring** — checklist completion (agenda, docs, context, participants)
- **Risk detection** — missing agenda, missing docs, no participants, no project link, upcoming-soon gap
- **Meeting brief** — aggregation of event data, participants, context pack
- **Agenda generation** — template-based per event type (meeting, review, planning)
- **Brain search** — ILIKE over title and description; structured weekly overview

The `CalendarIntelligenceService` is a pure function service (no state). The `CalendarBrainService` accepts a `PgPool` reference for database queries. All functions have explicit `CalendarIntelligenceError` / `CalendarBrainError` types.

## Consequences

- Calendar intelligence works without Ollama running.
- Heuristics are transparent, debuggable, and fast.
- Ollama can be added later as an optional refinement layer without changing the API.
- Template-based agenda generation produces predictable, domain-appropriate results.
```

### `docs/adr/ADR-0070-tasks-first-class-domain.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0070-tasks-first-class-domain.md`
- Size bytes / Размер в байтах: `1164`
- Included characters / Включено символов: `1164`
- Truncated / Обрезано: `no`

```markdown
# ADR-0070 Tasks as First-Class Domain with Local Overlay

Status: Proposed

## Context

Макошь needs a task management module that unifies local tasks and external trackers (Jira, YouTrack, GitHub Issues, etc.) with a personal context layer. Tasks must be linked to contacts, organizations, projects, emails, meetings, and documents.

## Decision

Tasks is a first-class domain with `task_id = task:v1:{nanos_hex}`. Each task has a local overlay (AI summary, private notes, context, risks) that is never synced to external providers. Multi-provider architecture via `task_provider_accounts` with capabilities model. External task identities stored in `external_task_identities` with per-provider status mapping.

The existing `task_candidates` pipeline (AI extraction from messages/documents) remains intact; confirmed candidates become tasks with full context.

## Consequences

- Tasks are queryable by status, project, source type.
- Local context is permanently separate from provider-synced data.
- Provider sync (Jira/YouTrack/GitHub) is schema-ready but deferred to a future phase.
- Privacy boundaries between local and external context are enforced.
```

### `docs/adr/ADR-0071-task-context-evidence-provenance.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0071-task-context-evidence-provenance.md`
- Size bytes / Размер в байтах: `914`
- Included characters / Включено символов: `914`
- Truncated / Обрезано: `no`

```markdown
# ADR-0071 Task Context and Evidence Provenance

Status: Proposed

## Context

Макошь Tasks must track where each task came from, why it exists, and what context surrounds it. AI-extracted tasks from emails, meetings, and documents need evidence provenance. Tasks need a materialized context pack for instant retrieval.

## Decision

Task Context Pack is a materialized JSONB snapshot containing summary, open questions, blockers, risks, and suggested next action. Task Evidence stores `source_type`, `source_id`, `quote`, and `confidence` for AI-extracted tasks. Low-confidence tasks route to the suggested inbox for user review. All facts carry `source` and `confidence` fields.

## Consequences

- Every AI-generated task has verifiable evidence.
- Context packs enable instant task understanding without cross-domain joins.
- User review flow prevents low-confidence tasks from polluting the active task list.
```

### `docs/adr/ADR-0072-task-intelligence-heuristic-fallbacks.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0072-task-intelligence-heuristic-fallbacks.md`
- Size bytes / Размер в байтах: `1000`
- Included characters / Включено символов: `1000`
- Truncated / Обрезано: `no`

```markdown
# ADR-0072 Task Intelligence with Heuristic Fallbacks

Status: Proposed

## Context

The task module needs priority scoring, risk analysis, readiness assessment, missing context detection, and next-action suggestions. Ollama is available but must not be mandatory per ADR-0009.

## Decision

All intelligence features use deterministic heuristics:
- **Priority**: weighted by deadline proximity, legal/tax context, contact presence, blockers
- **Risk**: deadline closeness, missing docs, no owner, external dependencies, legal context
- **Readiness**: description, context pack, docs, deadline, no blockers, contacts resolved
- **Missing context**: checklist-based gap detection
- **Next action**: template-based per status

Ollama refinement is optional and can be added without API changes.

## Consequences

- Task intelligence works without Ollama running.
- Heuristics are transparent, fast, and debuggable.
- Priority/risk/readiness scores are stored on the task row for sorting and filtering.
```

### `docs/adr/ADR-0073-backend-module-organization.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0073-backend-module-organization.md`
- Size bytes / Размер в байтах: `5725`
- Included characters / Включено символов: `3877`
- Truncated / Обрезано: `no`

````markdown
# ADR-0073 Backend Module Organization

Status: Accepted

## Context

На момент принятия этого ADR backend состоит из 106 файлов `.rs`, лежащих плоско в `backend/src/` без иерархии модулей. Файл `lib.rs` содержит 10 571 строку и смешивает: декларации модулей, HTTP routing, обработчики запросов, AppState, CORS, tracing, парсинг query-параметров и валидационные хелперы.

При росте системы такая структура становится неуправляемой:
- невозможно определить границы bounded context без чтения имён всех файлов;
- интеграции лежат вперемешку с доменами;
- общая платформа и AI не отделены от бизнес-логики;
- `lib.rs` невозможно рецензировать или тестировать изолированно.

## Decision

Вводится семислойная организация backend-крейта:

```text
backend/src/
├── app/            — HTTP-слой приложения (router, handlers, state, error)
├── domains/        — Bounded contexts (mail, persons, calendar, tasks, ...)
├── engines/        — Общие движки (search, automation)
├── integrations/   — Внешние адаптеры (gmail, telegram, whatsapp, ollama)
├── ai/             — AI-слой (семантические эмбеддинги, retrieval, AI-сервис)
├── workflows/      — Бизнес-процессы (email sync pipeline, email intelligence)
└── platform/       — Техническая платформа (config, events, secrets, db, audit, ...)
```

### Правила размещения

1. **app/** — HTTP-роутинг, обработчики, AppState, error types верхнего уровня. Не содержит бизнес-логики.
2. **domains/** — Каждый подкаталог — самостоятельный bounded context. Домены не импортируют друг друга напрямую. Для cross-domain коммуникации используют контракт из `ADR-architecture-communication-contract`.
3. **engines/** — Общие движки, не привязанные к конкретному домену.
4. **integrations/** — Адаптеры внешних систем. Не содержат бизнес-логики, только транспорт/протокол.
5. **ai/** — AI-компоненты изолированы от доменов. Домены используют AI через events или явные сервисные границы.
6. **workflows/** — Бизнес-процессы, координирующие несколько доменов.
7. **platform/** — Техническая платформа: конфигурация, события, БД, хранилище, секреты, аудит, capabilities.

### Domain Isolation

Домены не должны напрямую импортировать друг друга:

```rust
// ❌ Запрещено
use crate::domains::tasks::api::TaskStore;

// ✅ Разрешено — через events
use crate::platform::events::EventStore;

// ✅ Разрешено — через command/query/event contract владельца
use crate::platform::events::EventStore;
```

Прежнее исключение для `domains::graph` упразднено. Graph является доменом и
не может использоваться как shared spine через прямые импорты из других
доменов. Детальный контракт слоёв и допустимых способов взаимодействия описан в
`ADR-architecture-communication-contract`.

### Порог размера файлов

Файлы крупнее 700 строк требуют header-комментария в начале файла с объяснением, почему файл не разделён. Пример:

```rust
// This file exceeds 700 lines because it groups a single-responsibility
// store with its associated types (model, errors, queries) that share
// tight coupling through SQL query construction. Splitting would
// require either duplicating SQL fragments or introducing an
// abstraction layer that adds indirection without reducing complexity.
```

### Именование

- Имена файлов отражают доменную ответственность, а не техническую реализацию.
- `mod.rs` используется для реэкспорта публичного API модуля.
- Внутренние детали остаются `pub(crate)`.

## Consequences

- Структура проекта самодокументируется: по расположению файла понятна его роль.
- Рефакторинг `lib.rs` устраняет god file: routing, handlers, state, error разделены.
- Domain isolation предотвращает циклические зависимости при росте системы.
- Порог в 700 строк с обязательным обоснованием предотвращает повторное появление god files.
- Импорты становятся длиннее (`crate::domains::communications::core` вместо `crate::communications`), но это плата за явные границы.
````

### `docs/adr/ADR-0074-person-multi-channel-identity-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0074-person-multi-channel-identity-model.md`
- Size bytes / Размер в байтах: `1871`
- Included characters / Включено символов: `1871`
- Truncated / Обрезано: `no`

```markdown
# ADR-0074 Person Multi-Channel Identity Model

Status: Accepted

## Context

ADR-0019 established identity resolution as confidence-scored merge/split candidates. The current contact projection was originally derived from a single email address and is already referenced by graph projections, project links and task context. The functional spec for Макошь Persons requires multi-channel identities (email, Telegram, WhatsApp, phone, GitHub, LinkedIn, and others) linked to a single person entity.

## Decision

Keep the current person primary key as a stable text identifier: `person:v1:email:{len}:{normalized_email}` for email-created persons. The `persons.email_address` unique constraint remains in place as the primary-email compatibility contract for existing projections.

Add a separate `person_identities` table for all channel-specific identifiers. Each identity carries `source`, `confidence`, `status`, verification metadata, and a unique active `(identity_type, identity_value)` identity constraint. Auto-creation from incoming email creates or matches the primary email person row and backfills the matching `person_identities` record.

Opaque UUID person IDs are not part of this implementation slice. Moving from text person IDs to opaque IDs would require a separate ADR and migration plan covering graph nodes, tasks, projects, communication projections, API payloads and frontend state.

## Consequences

- One person can have many identities across channels.
- Identity resolution (merge/split) operates on current text `person_id` values.
- Backfill migrates existing `contact:v1:email:*` IDs to `person:v1:*` format and creates email identity rows.
- The `person_identity_candidates` table is renamed from `contact_identity_candidates`.
- A future opaque-ID migration remains possible, but it must be explicit and cannot be inferred from this ADR.
```

### `docs/adr/ADR-0076-host-vault-on-macos.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0076-host-vault-on-macos.md`
- Size bytes / Размер в байтах: `3312`
- Included characters / Включено символов: `3312`
- Truncated / Обрезано: `no`

```markdown
# ADR-0076 Host Vault on macOS

Status: Accepted

Supersedes: ADR-0044, ADR-0053

## Context

Макошь is a trusted single-user desktop application. The vault protects local secrets at rest, not against a compromised operating system or hostile local administrator.

ADR-0053 moved encrypted credential payloads into PostgreSQL. That improved database backup completeness, but it made PostgreSQL backups carry high-value ciphertext and did not meet the newer requirement that database deletion or recreation must not destroy credentials, account keys or signing material.

The target runtime is now macOS-only. Docker remains a development environment, not a production deployment model.

## Decision

Use a dedicated host vault under `~/.makosh/vault` for secrets-only encrypted payload storage.

Rules:

- PostgreSQL stores non-secret account metadata, `secret_references` and account-to-secret bindings only.
- New secret payloads are written to `vault.db`, a dedicated SQLite database under the host vault directory.
- New `secret_references.store_kind` values for host-vault secrets use `host_vault`.
- `encrypted_secret_vault_entries` remains legacy/migration state only. New runtime writes must not add provider credential payloads to PostgreSQL.
- The master key is stored outside application databases. Release runtime uses macOS Keychain. Docker/debug development may use `MAKOSH_DEV_KEY_PATH` only when `MAKOSH_DEV_MODE=true` and the build has debug assertions.
- Vault cryptography uses OS randomness, mouse/timing entropy from onboarding, SHA-512 mixing, HKDF-SHA256 domain keys and XChaCha20-Poly1305 record encryption.
- Per-entry AAD includes vault version, entry kind, account id, purpose and secret kind.
- The in-memory master key remains loaded after explicit unlock for the application lifetime and is zeroized on process shutdown or explicit lock.
- Onboarding keeps mouse movement as a trust-building UX signal, while OS randomness remains the cryptographic foundation.
- Recovery material is mandatory. Biometrics or Keychain authorization are unlock gates, not recovery mechanisms.
- Account binding recovery is represented by a host-vault manifest containing minimal non-secret account/secret mapping metadata. It must not contain plaintext secret values.

## Consequences

Positive:

- PostgreSQL can be dropped or recreated without destroying local credential payloads.
- Database backups and agent access to PostgreSQL do not expose encrypted secret payload rows for new entries.
- macOS Keychain becomes the release source of truth for the master key.
- Docker development remains usable through an explicit mounted host vault path.

Negative:

- Full restore now requires both PostgreSQL/object data and the host vault/recovery material.
- Cross-platform secure storage is intentionally not implemented. Windows/Linux runtime requires a new ADR.
- Recovery and manifest reconciliation must be treated as first-class lifecycle flows, not ad-hoc migrations.

Risk handling:

- Keep `MAKOSH_SECRET_VAULT_KEY` only as a legacy migration compatibility variable.
- Enforce release guard against dev storage.
- Keep all secret reads behind the `SecretResolver` boundary.
- Add tests for wrong-key/AAD/nonce failure, host-vault CRUD, onboarding status and PostgreSQL payload regression.
```
