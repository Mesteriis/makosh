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
