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

- Chunk ID / ID чанка: `117-doc-docs-part-008`
- Group / Группа: `docs`
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

### `docs/integrations/zoom/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/api.md`
- Size bytes / Размер в байтах: `7469`
- Included characters / Включено символов: `7469`
- Truncated / Обрезано: `no`

````markdown
# Zoom API Reference

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

Current repository state: the foundation routes are implemented in this
checkout. Live authorization can exchange, store and explicitly refresh token
bundles, and started authorized runtimes can participate in the scheduled
recording-sync worker.

Base path:

```text
/api/v1/integrations/zoom
```

All routes belong to integration/runtime setup and runtime bridge ingestion.
They are not product-domain routes.

## Route summary

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/capabilities` | Returns runtime capabilities, planned features and unsupported features. |
| `GET` | `/accounts` | Lists Zoom provider accounts. |
| `POST` | `/accounts` | Registers live account metadata in blocked mode. |
| `POST` | `/fixtures/accounts` | Registers a fixture account for local validation. |
| `POST` | `/oauth/start` | Creates an OAuth user pending grant and returns the Zoom authorization URL. |
| `POST` | `/oauth/complete` | Exchanges an OAuth authorization code and stores the token bundle in HostVault. |
| `POST` | `/oauth/server-to-server/authorize` | Exchanges Server-to-Server account credentials and stores the token bundle in HostVault. |
| `POST` | `/oauth/refresh` | Refreshes OAuth user tokens or renews Server-to-Server credentials in HostVault. |
| `POST` | `/oauth/maintenance` | Scans authorized accounts and refreshes expiring HostVault token bundles. |
| `POST` | `/provider-sync/recordings` | Manually lists Zoom cloud recordings for an authorized account, records meeting/recording evidence, and best-effort imports recording media plus transcript-like text files. |
| `GET` | `/webhook-subscriptions/status?account_id=...` | Reads managed Zoom app webhook subscription status for an authorized account. |
| `POST` | `/webhook-subscriptions/reconcile` | Creates or recreates the managed Zoom app event subscription for an authorized account. |
| `POST` | `/webhook-subscriptions/remove` | Removes the managed Zoom app event subscription for an authorized account. |
| `GET` | `/runtime/status?account_id=...` | Reads runtime status by query parameter. |
| `GET` | `/accounts/{account_id}/runtime/status` | Reads runtime status by path parameter. |
| `GET` | `/accounts/{account_id}/recording-imports?limit=...` | Lists imported Zoom recording media blobs for operator-visible audit on the selected account. |
| `POST` | `/accounts/{account_id}/recording-imports/{attachment_id}/remove` | Removes a previously imported Zoom recording blob from local audit/storage state for the selected account. |
| `POST` | `/accounts/{account_id}/retention/prune` | Prunes expired imported recording blobs and expired transcript evidence for the selected account using stamped retention expiry intent. |
| `GET` | `/accounts/{account_id}/audit-events?limit=...` | Lists recent Zoom authorization, token-refresh, runtime and bridge event-log entries for operator-visible audit on the selected account. |
| `POST` | `/runtime/start` | Starts fixture runtime metadata, returns blocked unauthorized live state, or running authorized-live state. |
| `POST` | `/runtime/stop` | Stops runtime metadata unless removed. |
| `POST` | `/runtime/remove` | Marks runtime/account lifecycle as removed. |
| `POST` | `/runtime-bridge/meetings` | Ingests meeting observation as provider call evidence. |
| `POST` | `/runtime-bridge/recordings` | Ingests recording observation as sanitized event evidence. |
| `POST` | `/runtime-bridge/transcripts` | Ingests transcript observation as call transcript evidence. |
| `POST` | `/runtime-bridge/transcript-files` | Imports VTT/SRT/plain transcript file text as call transcript evidence. |
| `POST` | `/runtime-bridge/webhooks?account_id=...` | Handles account-scoped endpoint URL validation and signed Zoom meeting/recording webhooks. |
| binary | `makosh-zoom-edge-proxy` | Public/edge webhook ingress that forwards raw Zoom webhook bodies and `x-zm-*` headers into the protected bridge. |

Detailed pages:

- [Accounts API](api/accounts.md)
- [Runtime API](api/runtime.md)
- [Runtime Bridge API](api/runtime-bridge.md)

## Capabilities

```http
GET /api/v1/integrations/zoom/capabilities
```

Target response shape:

```json
{
  "version": "1.0",
  "runtime_mode": "fixture_plus_authorized_live_workers",
  "capabilities": [
    {
      "capability": "bridge.meetings",
      "category": "ingest",
      "status": "available",
      "action_class": "local_write",
      "confirmation_required": false,
      "reason": "Meeting observations are stored as provider call evidence and emitted as Zoom events."
    }
  ],
  "planned_features": [],
  "unsupported_features": [
    "hidden_recording",
    "joining_meetings_as_a_bot_without_explicit_setup",
    "auto_dialing",
    "training_models_on_zoom_content_by_default"
  ]
}
```

## Common account model

```json
{
  "account_id": "zoom_fixture_primary",
  "provider_kind": "zoom_user",
  "display_name": "Zoom Fixture",
  "external_account_id": "fixture-zoom-account",
  "auth_shape": "fixture",
  "lifecycle_state": "fixture_ready",
  "runtime_kind": "zoom_fixture_runtime",
  "account_email": "owner@example.test",
  "config": {},
  "created_at": "2026-06-27T00:00:00Z",
  "updated_at": "2026-06-27T00:00:00Z"
}
```

## Common runtime status model

```json
{
  "account_id": "zoom_fixture_primary",
  "provider_kind": "zoom_user",
  "runtime_kind": "zoom_fixture_runtime",
  "status": "running",
  "healthy": true,
  "auth_shape": "fixture",
  "live_runtime_available": false,
  "recording_ingest_available": true,
  "transcript_ingest_available": true,
  "runtime_blockers": [],
  "last_error": null,
  "checked_at": "2026-06-27T00:00:00Z",
  "metadata": {}
}
```

## Error policy

The integration should validate required ids and metadata shape before
persistence. Invalid account ids, missing meeting ids, missing transcript text
and non-object metadata should return integration API errors through the shared
app error layer.

## Realtime events

Frontend realtime invalidation should listen for:

```text
zoom.authorization.completed
zoom.runtime.status_changed
zoom.token.refreshed
zoom.token.refresh.skipped
zoom.token.refresh.failed
zoom.meeting.observed
zoom.recording.observed
zoom.transcript.observed
zoom.transcript.removed
zoom.recording.import.removed
zoom.retention.cleanup.completed
```

Direct local meeting/recording/transcript/provider-sync mutations should also
invalidate the provider-neutral calls cache and transcript cache for the
selected account, not only Zoom runtime metadata.

Owner-visible retention policy is configured through application settings:

```text
privacy.zoom_recording_import_retention_days
privacy.zoom_transcript_retention_days
```

`0` means explicit manual removal only. Positive values stamp retention mode
and expiry intent into imported-recording audit items and transcript
provenance. Expired Zoom recording imports and transcript evidence can be
pruned through `POST /accounts/{account_id}/retention/prune`.

Provider runtime caches are allowed under integration keys only, for example:

```text
['integrations', 'zoom', 'runtime', accountId]
['integrations', 'zoom', 'accounts']
['integrations', 'zoom', 'capabilities']
```

Provider-neutral call evidence reads for Zoom can use the shared Calls routes
with `provider=zoom`, for example:

```text
/api/v1/calls?account_id=<zoom-account-id>&provider=zoom&limit=20
/api/v1/calls/<call_id>/transcript
```
````

### `docs/integrations/zoom/api/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/api/README.md`
- Size bytes / Размер в байтах: `354`
- Included characters / Включено символов: `354`
- Truncated / Обрезано: `no`

```markdown
# Zoom API Details

Status: documentation package aligned to the current repository structure.

This package breaks the Zoom integration API surface into focused references.
The parent package remains the provider runtime boundary.

## Navigation

- [Accounts API](./accounts.md)
- [Runtime API](./runtime.md)
- [Runtime Bridge API](./runtime-bridge.md)
```

### `docs/integrations/zoom/api/accounts.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/api/accounts.md`
- Size bytes / Размер в байтах: `7945`
- Included characters / Включено символов: `7945`
- Truncated / Обрезано: `no`

````markdown
# Zoom Accounts API

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-27.

The account routes below are implemented for the Zoom provider foundation.
OAuth user and Server-to-Server authorization can exchange tokens and store
credential payloads in HostVault. Started authorized runtimes can then
participate in the scheduled recording-sync worker.

## Fixture account setup

```http
POST /api/v1/integrations/zoom/fixtures/accounts
```

Fixture routes are intended for deterministic local validation.

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "display_name": "Zoom Fixture",
  "external_account_id": "fixture-zoom-account",
  "account_email": "owner@example.test",
  "metadata": {
    "fixture": true
  }
}
```

Target response:

```json
{
  "account": {
    "account_id": "zoom_fixture_primary",
    "provider_kind": "zoom_user",
    "display_name": "Zoom Fixture",
    "external_account_id": "fixture-zoom-account",
    "auth_shape": "fixture",
    "lifecycle_state": "fixture_ready",
    "runtime_kind": "zoom_fixture_runtime",
    "account_email": "owner@example.test",
    "config": {}
  }
}
```

## Live account metadata setup

```http
POST /api/v1/integrations/zoom/accounts
```

Registers live account metadata and secret references. This route does not
accept raw token or secret values.

Request:

```json
{
  "account_id": "zoom_live_owner",
  "display_name": "Owner Zoom",
  "external_account_id": "zoom-account-id",
  "account_email": "owner@example.com",
  "auth_shape": "oauth_user",
  "client_id": "client-id-reference-or-public-id",
  "token_secret_ref": "secret_ref_zoom_token",
  "client_secret_ref": "secret_ref_zoom_client_secret",
  "webhook_secret_ref": "secret_ref_zoom_webhook",
  "metadata": {
    "environment": "local"
  }
}
```

Supported `auth_shape` values:

```text
fixture
oauth_user
server_to_server
```

Provider kind mapping:

```text
oauth_user       -> zoom_user
server_to_server -> zoom_server_to_server
fixture          -> zoom_user
```

Live account response should contain:

```text
lifecycle_state: blocked
runtime_kind: zoom_live_blocked_runtime
runtime_blockers: [zoom_live_authorization_required]
```

## OAuth user authorization

```http
POST /api/v1/integrations/zoom/oauth/start
POST /api/v1/integrations/zoom/oauth/complete
```

`/oauth/start` registers or updates OAuth live account metadata, creates a
pending grant in memory and returns an authorization URL. It accepts either a
raw `client_secret` for immediate HostVault storage during completion or an
existing `client_secret_ref`; raw values are never written to PostgreSQL account
config.

Start request:

```json
{
  "account_id": "zoom_live_owner",
  "display_name": "Owner Zoom",
  "external_account_id": "zoom-account-id",
  "account_email": "owner@example.com",
  "client_id": "zoom-client-id",
  "client_secret": "submitted-to-host-vault-only",
  "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback",
  "scopes": ["meeting:read", "recording:read"],
  "metadata": {}
}
```

Start response:

```json
{
  "setup_id": "opaque-setup-id",
  "authorization_url": "https://zoom.us/oauth/authorize?...",
  "state": "opaque-state",
  "redirect_uri": "http://127.0.0.1:8080/zoom/oauth/callback"
}
```

`/oauth/complete` validates the pending `state`, exchanges the authorization
code at the token endpoint, stores the OAuth bundle in HostVault, binds
`zoom_oauth_token` and marks the account authorized.

Complete request:

```json
{
  "setup_id": "opaque-setup-id",
  "state": "opaque-state",
  "authorization_code": "provider-code"
}
```

## Server-to-Server authorization

```http
POST /api/v1/integrations/zoom/oauth/server-to-server/authorize
```

Exchanges Zoom Server-to-Server account credentials with `grant_type =
account_credentials`, stores the returned OAuth bundle in HostVault and marks
the account authorized.

Request:

```json
{
  "account_id": "zoom_s2s_owner",
  "client_id": "zoom-s2s-client-id",
  "client_secret": "submitted-to-host-vault-only",
  "zoom_account_id": "zoom-provider-account-id",
  "metadata": {}
}
```

Authorization response:

```json
{
  "account_id": "zoom_s2s_owner",
  "provider_kind": "zoom_server_to_server",
  "auth_shape": "server_to_server",
  "lifecycle_state": "authorized",
  "runtime_kind": "zoom_live_authorized_runtime",
  "token_secret_ref": "secret:provider-account:zoom_s2s_owner:zoom_oauth_token",
  "client_secret_ref": "secret:provider-account:zoom_s2s_owner:zoom_client_secret",
  "secret_kind": "oauth_token",
  "store_kind": "host_vault",
  "authorized_at": "2026-06-27T00:00:00Z"
}
```

## Token refresh and renewal

```http
POST /api/v1/integrations/zoom/oauth/refresh
```

Refreshes an authorized Zoom credential bundle without returning the raw access
token. OAuth user accounts use the stored `refresh_token`. Server-to-Server
accounts renew by repeating the `account_credentials` token exchange. The
updated token bundle is written back to HostVault under the existing
`zoom_oauth_token` binding.

Request:

```json
{
  "account_id": "zoom_live_owner",
  "force": true
}
```

Response:

```json
{
  "account_id": "zoom_live_owner",
  "provider_kind": "zoom_user",
  "auth_shape": "oauth_user",
  "token_secret_ref": "secret:provider-account:zoom_live_owner:zoom_oauth_token",
  "refreshed": true,
  "refresh_strategy": "oauth_refresh_token",
  "status": "refreshed",
  "expires_at": "2026-06-27T01:00:00Z",
  "checked_at": "2026-06-27T00:00:00Z",
  "secret_kind": "oauth_token",
  "store_kind": "host_vault"
}
```

If `force` is false and the token bundle is still valid past the refresh
threshold, the route returns `refreshed = false` and
`status = skipped_not_expired`. The explicit refresh default threshold is 60
seconds, with a maximum accepted threshold of 86400 seconds.

## Token maintenance

```http
POST /api/v1/integrations/zoom/oauth/maintenance
```

Scans authorized Zoom live accounts and refreshes only bundles that expire
within the requested threshold. This is the integration-safe worker boundary
used by explicit API calls and by the local scheduled token maintenance daemon.
The scheduler is gated by Signal Hub runtime state, HostVault unlock status and
`MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`.
The maintenance default threshold is 300 seconds. Runtime status exposes the
same policy under `metadata.token_rotation_policy`, including refresh due
state, expiry state and the `zoom_token_rotation_required` failure/expiry
blocker.

Request:

```json
{
  "account_id": null,
  "force": false,
  "refresh_expiring_within_seconds": 300
}
```

Response:

```json
{
  "checked_count": 1,
  "refreshed_count": 1,
  "skipped_count": 0,
  "failed_count": 0,
  "refresh_expiring_within_seconds": 300,
  "checked_at": "2026-06-27T00:00:00Z",
  "items": [
    {
      "account_id": "zoom_live_owner",
      "provider_kind": "zoom_user",
      "auth_shape": "oauth_user",
      "status": "refreshed",
      "refreshed": true,
      "expires_at": "2026-06-27T01:00:00Z",
      "error": null
    }
  ]
}
```

Imported recording/blob retention cleanup is a separate local automation path.
The owner-visible prune route is `POST /api/v1/integrations/zoom/accounts/{account_id}/retention/prune`,
and the background cleanup daemon is toggled by
`MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED`.

## Account list

```http
GET /api/v1/integrations/zoom/accounts
GET /api/v1/integrations/zoom/accounts?include_removed=true
```

Default listing excludes removed accounts. `include_removed=true` includes
records whose config has `lifecycle_state = removed`.

## Validation rules

Required fields:

```text
account_id
display_name
external_account_id
metadata as JSON object when present
```

Secret values must not be submitted in metadata or account setup routes. OAuth
authorization routes may accept a one-time raw `client_secret` only to store it
in HostVault; PostgreSQL stores secret references, bindings and non-secret
metadata only.
````

### `docs/integrations/zoom/api/runtime-bridge.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/api/runtime-bridge.md`
- Size bytes / Размер в байтах: `6925`
- Included characters / Включено символов: `6925`
- Truncated / Обрезано: `no`

````markdown
# Zoom Runtime Bridge API

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

The runtime bridge routes below are implemented for the Zoom provider
foundation. They accept sanitized local/runtime observations and account-scoped
verified webhook notifications without changing downstream event contracts.

## Meeting observation

```http
POST /api/v1/integrations/zoom/runtime-bridge/meetings
```

Request:

```json
{
  "observation_id": "obs_zoom_meeting_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "meeting_uuid": "meeting-uuid",
  "topic": "Макошь Zoom Review",
  "host_email": "owner@example.test",
  "join_url": "https://example.invalid/j/987654321",
  "started_at": "2026-06-27T10:00:00Z",
  "ended_at": "2026-06-27T10:30:00Z",
  "duration_seconds": 1800,
  "participants": [
    {
      "participant_id": "p1",
      "display_name": "Owner",
      "email": "owner@example.test",
      "joined_at": "2026-06-27T10:00:00Z",
      "left_at": "2026-06-27T10:30:00Z",
      "metadata": {}
    }
  ],
  "recording_refs": [],
  "transcript_ref": "tr_zoom_001",
  "metadata": {
    "fixture": true
  },
  "causation_id": null,
  "correlation_id": "corr_zoom_fixture_001"
}
```

Target response:

```json
{
  "call_id": "zoom_call_<stable_hash>",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "event_id": "evt_zoom_meeting_<stable_hash>",
  "status": "recorded"
}
```

Effects:

```text
upsert provider call evidence
append zoom.meeting.observed
broadcast zoom.meeting.observed
```

## Recording observation

```http
POST /api/v1/integrations/zoom/runtime-bridge/recordings
```

Request:

```json
{
  "observation_id": "obs_zoom_recording_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "recording": {
    "recording_id": "rec_001",
    "recording_type": "shared_screen_with_speaker_view",
    "download_ref": "blob_or_provider_ref_only",
    "file_extension": "mp4",
    "file_size_bytes": 123456,
    "recorded_at": "2026-06-27T10:31:00Z",
    "metadata": {}
  },
  "metadata": {},
  "correlation_id": "corr_zoom_fixture_001"
}
```

Effects:

```text
append zoom.recording.observed
broadcast zoom.recording.observed
```

The direct `/runtime-bridge/recordings` route records sanitized event evidence
only. Provider-side recording file downloads are handled separately through the
authorized provider-sync worker and signed `recording.completed` webhook path
after explicit privacy opt-in.

## Transcript observation

```http
POST /api/v1/integrations/zoom/runtime-bridge/transcripts
```

Request:

```json
{
  "observation_id": "obs_zoom_transcript_001",
  "transcript_id": "tr_zoom_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "meeting_uuid": "meeting-uuid",
  "source_recording_ref": "rec_001",
  "language_code": "en",
  "transcript_text": "Meeting transcript text.",
  "segments": [],
  "metadata": {
    "fixture": true
  },
  "correlation_id": "corr_zoom_fixture_001"
}
```

Target response:

```json
{
  "transcript_id": "tr_zoom_001",
  "call_id": "zoom_call_<stable_hash>",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "event_id": "evt_zoom_transcript_<stable_hash>",
  "status": "recorded"
}
```

Effects:

```text
ensure placeholder provider call if needed
upsert call transcript
append zoom.transcript.observed
broadcast zoom.transcript.observed
```

## Transcript file import

```http
POST /api/v1/integrations/zoom/runtime-bridge/transcript-files
```

This route imports an already obtained Zoom transcript file into the same
`zoom.transcript.observed` contract. It does not download provider files and it
does not bypass account validation.

Supported input formats:

```text
WEBVTT / .vtt
SRT / .srt
plain text
```

Request:

```json
{
  "observation_id": "obs_zoom_transcript_file_001",
  "transcript_id": "tr_zoom_file_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "meeting_uuid": "meeting-uuid",
  "source_recording_ref": "rec_001",
  "language_code": "en",
  "file_name": "meeting.vtt",
  "content_type": "text/vtt",
  "file_text": "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nHello",
  "metadata": {
    "fixture": true
  }
}
```

Response:

```json
{
  "transcript_id": "tr_zoom_file_001",
  "call_id": "zoom_call_<stable_hash>",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "event_id": "evt_zoom_transcript_<stable_hash>",
  "status": "recorded",
  "import_format": "webvtt",
  "parsed_segment_count": 1
}
```

Effects:

```text
parse VTT/SRT/plain transcript file
ensure placeholder provider call if needed
upsert call transcript
append zoom.transcript.observed
broadcast zoom.transcript.observed
```

## Verified webhook bridge

```http
POST /api/v1/integrations/zoom/runtime-bridge/webhooks?account_id=<zoom_account_id>
```

This protected runtime-bridge route handles account-scoped Zoom webhook
notifications. It is not a public internet receiver; the standalone
`makosh-zoom-edge-proxy` may forward raw Zoom webhook requests here after
preserving the raw body and Zoom headers.

The implemented public/edge ingress for that forwarding role is:

```text
makosh-zoom-edge-proxy
PUBLIC:  POST /webhooks/zoom
FORWARDS TO:  POST /api/v1/integrations/zoom/runtime-bridge/webhooks?account_id=...
```

The proxy reads only local env configuration, never stores provider secrets,
and does not parse or rewrite the request body.

Endpoint URL validation:

```json
{
  "event": "endpoint.url_validation",
  "payload": {
    "plainToken": "zoom_plain_token"
  }
}
```

Response:

```json
{
  "plainToken": "zoom_plain_token",
  "encryptedToken": "<hmac_sha256_plain_token>"
}
```

Normal webhook ingestion requires:

```text
x-zm-request-timestamp
x-zm-signature: v0=<hmac_sha256>
```

The HMAC message is:

```text
v0:<x-zm-request-timestamp>:<raw_body>
```

Implemented normalization:

```text
meeting.* webhook -> ZoomMeetingObservationRequest -> zoom.meeting.observed
recording.* webhook -> ZoomRecordingObservationRequest(s) -> zoom.recording.observed
```

Signed `recording.completed` webhook payloads may also trigger best-effort
download/import of non-transcript recording media files and transcript-like
textual recording files when Zoom includes a `download_url` plus
`download_token`. Transcript text is not extracted from webhook metadata alone.
Import of already obtained VTT/SRT/plain transcript text remains available
through `/runtime-bridge/transcript-files`.

## Sanitization

The bridge recursively strips token-like fields from event payloads before
append and broadcast. Provider call/transcript metadata is sanitized with the
same rule before persistence:

```text
access_token
refresh_token
token
client_secret
webhook_secret
download_token
password
```

Do not rely on sanitization as the only defense. Runtime callers should avoid
submitting secret values outside secret-reference fields.
````

### `docs/integrations/zoom/api/runtime.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/api/runtime.md`
- Size bytes / Размер в байтах: `6507`
- Included characters / Включено символов: `6507`
- Truncated / Обрезано: `no`

````markdown
# Zoom Runtime API

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

The runtime routes below are implemented for the Zoom provider foundation.
Fixture accounts can transition through metadata lifecycle states. Live
accounts are blocked until authorization. Authorized live runtimes can then be
started, expose `running` runtime metadata and participate in the scheduled
recording-sync worker.

## Webhook subscription status

```http
GET /api/v1/integrations/zoom/webhook-subscriptions/status?account_id=zoom_live_primary
```

Behavior:

```text
- requires HostVault to be unlocked;
- requires an authorized OAuth user or Server-to-Server account;
- requires a bound HostVault-backed zoom_webhook_secret reference;
- exchanges an app-owned access token for provider-side webhook subscription management;
- returns the currently managed subscription id plus the remote subscription list.
```

## Webhook subscription reconcile

```http
POST /api/v1/integrations/zoom/webhook-subscriptions/reconcile
```

Request:

```json
{
  "account_id": "zoom_live_primary",
  "endpoint_url": "https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks",
  "subscription_name": "Макошь Zoom Runtime",
  "event_types": [
    "meeting.started",
    "meeting.ended",
    "meeting.participant_joined",
    "meeting.participant_left",
    "recording.completed"
  ]
}
```

Behavior:

```text
- OAuth user accounts exchange client_credentials for app-owned management tokens;
- Server-to-Server accounts exchange account_credentials for app-owned management tokens;
- when the managed subscription already matches endpoint + event types, response status is unchanged;
- when the managed subscription is missing or stale, Макошь deletes the stale managed subscription and recreates it.
```

## Webhook subscription remove

```http
POST /api/v1/integrations/zoom/webhook-subscriptions/remove
```

Request:

```json
{
  "account_id": "zoom_live_primary"
}
```

## Manual provider sync for cloud recordings

```http
POST /api/v1/integrations/zoom/provider-sync/recordings
```

Request:

```json
{
  "account_id": "zoom_live_primary",
  "from": "2026-06-01",
  "to": "2026-06-30",
  "page_size": 30,
  "max_meetings": 100,
  "user_id": "me"
}
```

Behavior:

```text
- requires HostVault to be unlocked;
- requires an authorized OAuth user or Server-to-Server account;
- calls Zoom cloud recording APIs with the stored bearer token;
- upserts meeting call evidence through the existing provider-neutral call store;
- emits recording events through the existing Zoom event contract;
- best-effort imports non-transcript recording media files exposed as recording
  files only when application setting
  privacy.zoom_remote_recording_download_enabled is true;
- best-effort imports transcript-like text files exposed as recording files only
  when application setting privacy.zoom_remote_transcript_download_enabled is true.
```

Target response:

```json
{
  "account_id": "zoom_live_primary",
  "user_id": "me",
  "from": "2026-06-01",
  "to": "2026-06-30",
  "meetings_seen": 4,
  "meetings_recorded": 4,
  "recordings_recorded": 7,
  "media_downloads_recorded": 1,
  "transcripts_recorded": 2,
  "failures": []
}
```

## Status by query

```http
GET /api/v1/integrations/zoom/runtime/status?account_id=zoom_fixture_primary
```

## Status by account path

```http
GET /api/v1/integrations/zoom/accounts/zoom_fixture_primary/runtime/status
```

Both routes return the same `ZoomRuntimeStatus` shape.

## Start runtime

```http
POST /api/v1/integrations/zoom/runtime/start
```

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "force": false
}
```

Fixture behavior:

```text
lifecycle_state -> running
runtime_kind    -> zoom_fixture_runtime
healthy         -> true
```

Live blocked behavior:

```text
lifecycle_state -> blocked
runtime_kind    -> zoom_live_blocked_runtime
healthy         -> false
runtime_blockers includes zoom_live_authorization_required
```

Authorized live behavior:

```text
lifecycle_state -> running
runtime_kind    -> zoom_live_authorized_runtime
healthy         -> true
live_runtime_available -> true
runtime_blockers empty unless token rotation or another runtime error is active
```

Emits:

```text
zoom.runtime.status_changed
```

Provenance action:

```text
zoom.runtime.start_requested
```

## Stop runtime

```http
POST /api/v1/integrations/zoom/runtime/stop
```

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "reason": "manual_validation_complete"
}
```

If account lifecycle is not `removed`, stop updates lifecycle to `stopped` and
emits `zoom.runtime.status_changed`.

## Remove runtime

```http
POST /api/v1/integrations/zoom/runtime/remove
```

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "reason": "fixture_cleanup"
}
```

Target response:

```json
{
  "account_id": "zoom_fixture_primary",
  "provider_kind": "zoom_user",
  "removed": true,
  "removed_at": "2026-06-27T00:00:00Z"
}
```

Removal is a lifecycle mark in provider account config. It is not a destructive
delete.

## Runtime status fields

| Field | Meaning |
|---|---|
| `account_id` | Макошь provider account id. |
| `provider_kind` | `zoom_user` or `zoom_server_to_server`. |
| `runtime_kind` | `zoom_fixture_runtime`, `zoom_live_blocked_runtime` or `zoom_live_authorized_runtime`. |
| `status` | Derived lifecycle status. |
| `healthy` | True for non-removed fixture runtime and for started authorized live runtimes without blockers. |
| `auth_shape` | `fixture`, `oauth_user`, `server_to_server`. |
| `live_runtime_available` | True only after live authorization is completed. |
| `recording_ingest_available` | False only when removed. |
| `transcript_ingest_available` | False only when removed. |
| `runtime_blockers` | Blocking reasons for live execution such as authorization or token-rotation failures. |
| `last_error` | Last recorded runtime error, if any. |
| `metadata` | Provider account config payload plus runtime-derived policy metadata, including the last recording-sync outcome when provider sync has run. |

`metadata.token_rotation_policy` is present on runtime status responses. It
contains the explicit refresh threshold, maintenance refresh threshold, maximum
accepted threshold, provider expiry safety margin, refresh due state, expiry
state, last refresh status and the `zoom_token_rotation_required` blocker name.
The blocker is exposed when a live authorized account has an expired token,
missing token binding or failed last refresh.
````

### `docs/integrations/zoom/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/architecture.md`
- Size bytes / Размер в байтах: `7469`
- Included characters / Включено символов: `7469`
- Truncated / Обрезано: `no`

````markdown
# Zoom Provider Architecture

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

Zoom is a provider runtime integration. It observes provider facts and
translates them into Макошь evidence. It does not own product meaning.

Current repository state: the foundation architecture below is implemented in
this checkout; ADR-0102 is `Accepted` after target backend and frontend zoom
validation.

## Ownership model

| Area | Target owner | Notes |
|---|---|---|
| Provider account metadata | `integrations/zoom` through provider account port | Stores account kind, display name, external account id and config metadata. |
| Runtime lifecycle | `integrations/zoom` | Stores fixture/running/stopped/blocked/removed status in provider account config. |
| Call evidence | `platform/calls` | Zoom meeting observations are upserted as provider-neutral calls. |
| Transcript evidence | `platform/calls` | Zoom transcript observations are upserted as call transcripts. |
| Canonical events | `platform/events` | Zoom emits canonical envelopes and broadcasts realtime events. |
| Business interpretation | Workflows/domains/engines | Radar, Calendar, Communications, Documents, Tasks, Personas and Knowledge consume downstream evidence. |

There is no `domains/zoom`.

## Inbound flow

```text
Zoom observation source
  -> runtime bridge request
  -> integrations/zoom validation
  -> provider account verification
  -> call/transcript evidence persistence when applicable
  -> sanitized zoom.* event append
  -> realtime broadcast
  -> Signal Hub / workflows / projections
```

The first bridge is local/runtime-safe. A protected account-scoped webhook
bridge validates Zoom URL validation requests and signed meeting/recording
webhooks before normalizing them into the same bridge methods. The
`makosh-zoom-edge-proxy` binary provides public/edge forwarding while preserving
the raw body and Zoom headers for protected bridge verification.

## Runtime shapes

```text
fixture
  -> runtime_kind: zoom_fixture_runtime
  -> lifecycle_state: fixture_ready | running | stopped | removed
  -> live_runtime_available: false

live blocked
  -> runtime_kind: zoom_live_blocked_runtime
  -> auth_shape: oauth_user | server_to_server
  -> lifecycle_state: blocked | stopped | removed
  -> runtime_blockers: [zoom_live_authorization_required]

live authorized
  -> runtime_kind: zoom_live_authorized_runtime
  -> auth_shape: oauth_user | server_to_server
  -> lifecycle_state: authorized | running | stopped | removed
  -> live_runtime_available: true
  -> runtime_blockers: []
```

Live account metadata can exist before authorization. Authorization stores
token bundles and client secrets in HostVault and marks the account authorized.
Explicit refresh, maintenance scans and scheduled token maintenance can renew
those token bundles. Authorized accounts can also reconcile managed Zoom app
event subscriptions through provider APIs. Started authorized runtimes can then
participate in the recording-sync worker, including best-effort recording media
downloads after explicit owner-visible opt-in.

## Event contract

Zoom events use canonical event envelopes with:

- stable `event_id` generated from event kind, account id, subject id and
  optional observation id;
- stable `event_type`;
- `source` identifying Zoom provider and account;
- `subject` identifying meeting, recording, transcript or runtime account;
- sanitized `payload`;
- `provenance` describing bridge source and storage effect;
- optional `causation_id`;
- `correlation_id` defaulting to request correlation id, observation id or
  event id.

Supported target event types:

```text
zoom.runtime.status_changed
zoom.meeting.observed
zoom.recording.observed
zoom.transcript.observed
```

## Evidence persistence

### Meetings

`ZoomMeetingObservationRequest` should be converted to a provider-neutral
`NewProviderCall` using a stable call id derived from account id and meeting id.

```text
stable_zoom_call_id(account_id, meeting_id)
  -> zoom_call_<sha256>
```

The call metadata keeps Zoom-specific context such as meeting id, meeting UUID,
topic, host email, join URL, duration, participants, recording references and
transcript reference.

### Recordings

Recording observations are event-sourced, and both the authorized
recording-sync worker and signed `recording.completed` webhook path can
best-effort import non-transcript recording media files after
`privacy.zoom_remote_recording_download_enabled` is enabled. Imported media is
stored through the local communication blob store, recorded as attachment-import
metadata and passed through the heuristic attachment safety scanner.
Retention/audit surfaces remain later phases.

### Transcripts

Transcript observations create or reuse a placeholder provider call and then
persist a call transcript with:

```text
stt_provider: zoom-cloud-transcript
transcript_status: succeeded
source_audio_ref: source_recording_ref
segments: provider/runtime supplied segments
provenance: zoom meeting metadata
```

Already obtained VTT, SRT and plain text transcript file contents can be
imported through `/runtime-bridge/transcript-files`. Provider-side transcript
file download from Zoom recording references is implemented for signed
`recording.completed` webhooks and authorized provider-sync after explicit
owner-visible opt-in.

## Sanitization boundary

Before an event is appended or broadcast, token-like fields are removed
recursively:

```text
access_token
refresh_token
token
client_secret
webhook_secret
download_token
password
```

This is a runtime safety boundary, not a substitute for secret storage policy.
Credentials must be stored through secret references.

## Authorization boundary

OAuth user grants and Server-to-Server account credentials are exchanged inside
the integration boundary. Raw provider credential payloads are written to
HostVault, while PostgreSQL stores `secret_references`,
provider-account-secret bindings and non-secret account metadata only.

The authorization boundary can explicitly refresh OAuth user credentials using
the stored refresh token, renew Server-to-Server credentials using account
credentials, scan authorized accounts for expiring token bundles, reconcile
managed provider webhook subscriptions, and schedule maintenance through
backend bootstrap. It also powers the authorized recording-sync worker that
downloads recording metadata, selected media files and transcript-like text
files through explicit opt-in policy gates. Authorized accounts remain degraded
until broader provider workers and downstream operational surfaces are
implemented and validated.

## Downstream interpretation

Zoom does not create business entities directly. The intended downstream flow
is:

```text
zoom.meeting.observed
  -> Calls projection
  -> Calendar matching workflow
  -> participant identity candidate workflow
  -> Radar signal detection
  -> Review promotion
  -> target domain command
```

Example target outcomes:

- meeting preparation context pack;
- follow-up task candidate;
- Persona identity trace;
- organization relationship evidence;
- document note from transcript;
- knowledge graph edge after review.

## Safety invariants

- No hidden recording.
- No automatic meeting joining without explicit owner-visible setup.
- No raw Zoom credentials in domain models.
- No raw secrets in event payloads.
- No AI extraction as source of truth.
- No provider-specific business UI cache for canonical communication data.
````

### `docs/integrations/zoom/blockers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/blockers.md`
- Size bytes / Размер в байтах: `4794`
- Included characters / Включено символов: `4794`
- Truncated / Обрезано: `no`

````markdown
# Zoom Blockers

Status date: 2026-06-28.

This document lists blockers that must be resolved before Zoom can move from
foundation authorization and bridge ingestion to live provider runtime workers.

## Foundation blockers

| Blocker | Impact | Required action |
|---|---|---|
| No Zoom integration module | No backend account, runtime or bridge routes can exist. | Add `backend/src/integrations/zoom` behind ADR-0102 boundaries. |
| No provider kind migration | Provider accounts cannot represent Zoom safely. | Add migration for `zoom_user` and `zoom_server_to_server` only after inspecting current constraints. |
| No event constants | Bridge cannot append canonical Zoom events. | Add `zoom.runtime.status_changed`, `zoom.meeting.observed`, `zoom.recording.observed`, `zoom.transcript.observed`. |
| No frontend integration module | Settings/runtime UI cannot call provider setup routes. | Add integration-only API/query/types, not a product Zoom domain. |
| No tests | Boundary and sanitization claims are unverifiable. | Add fixture and negative tests with repository test harness. |

## Live runtime blockers

No current live-runtime blocker remains inside the implemented foundation scope.
The remaining open items are downstream workflow concerns, not missing Zoom
runtime/account plumbing.

## Architecture blockers

### No direct domain mutation

Zoom integration must not call:

```text
domains/calendar
domains/persons
domains/tasks
domains/documents
domains/organizations
domains/communications
```

Cross-domain effects must use events and workflows.

### No provider-specific product domain

Do not add:

```text
backend/src/domains/zoom
frontend/src/domains/zoom
```

Provider setup belongs under integrations. Product views belong under
provider-neutral Calls, Calendar, Communications, Documents, Radar and Knowledge
surfaces.

### No hidden recording

Live meeting capture, local recording, screen capture or transcript generation
requires explicit owner-visible setup and audit. Silent capture is unsupported.

## Security blockers

- Explicit token refresh/renew route. Implemented for OAuth user refresh-token
  and Server-to-Server account-credentials renewal through HostVault-backed
  `zoom_oauth_token` bundles.
- Token maintenance runner. Implemented for authorized account scanning and
  expiring-token renewal.
- Scheduled token maintenance daemon. Implemented through backend bootstrap
  with Signal Hub runtime gating, HostVault unlock gating and
  `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`.
- Scheduled recording sync daemon. Implemented through backend bootstrap for
  started authorized runtimes, with Signal Hub runtime gating, HostVault unlock
  gating, `MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED` and the existing
  HostVault-backed provider sync boundary.
- Scheduled retention cleanup daemon. Implemented through backend bootstrap for
  expired imported recording blobs and transcript evidence, with Signal Hub
  runtime gating and `MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED`.
- Token rotation policy. Implemented through explicit refresh thresholds,
  proactive maintenance threshold metadata, expiry handling and
  `zoom_token_rotation_required` failure/expiry blocker exposure in runtime
  status.
- OAuth and Server-to-Server initial token exchange. Implemented through
  HostVault-backed `zoom_oauth_token` secret references.
- Owner-visible privacy opt-in for provider-side transcript-like file
  downloads. Implemented through application setting
  `privacy.zoom_remote_transcript_download_enabled`, enforced for webhook and
  manual provider-sync downloads.
- Webhook secret storage through secret references. Implemented for the
  protected account-scoped runtime bridge; public/edge ingress is implemented
  as `makosh-zoom-edge-proxy`.
- Sanitization test coverage for nested payloads. Implemented for event payload
  and provider-call metadata in the Zoom foundation tests.
- Audit events for authorization completion, token refresh success/skip/failure,
  runtime lifecycle and bridge ingestion. Implemented on the account-scoped
  Zoom audit surface.
- Explicit per-import retention removal for imported recording blobs.
- Owner-visible retention policy settings for imported recordings and
  transcript evidence. Implemented through
  `privacy.zoom_recording_import_retention_days` and
  `privacy.zoom_transcript_retention_days`, with stamped expiry intent in
  audit/provenance metadata.
- Explicit owner-triggered cleanup of expired imported recordings and expired
  transcript evidence. Implemented through the account-scoped retention prune
  route and settings-panel control.

## UI blockers

- Richer Calendar/Communications downstream workflows beyond the current
  provider-neutral Calls/Meetings evidence views.
````

### `docs/integrations/zoom/fixture-test-matrix.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/fixture-test-matrix.md`
- Size bytes / Размер в байтах: `4832`
- Included characters / Включено символов: `4832`
- Truncated / Обрезано: `no`

````markdown
# Zoom Fixture Test Matrix

Status date: 2026-06-27.

Fixture tests should validate the integration boundary without depending on
live provider access. The tests below are target coverage for the future Zoom
implementation.

## Account and runtime tests

| Scenario | Route | Fixture input | Expected result |
|---|---|---|---|
| Create fixture account | `POST /fixtures/accounts` | valid account id/display/external id | `provider_kind=zoom_user`, `auth_shape=fixture`, `lifecycle_state=fixture_ready`. |
| List accounts | `GET /accounts` | one fixture account | account returned and sorted by display name. |
| Get runtime status | `GET /runtime/status?account_id=...` | fixture account | `status=stopped`, `healthy=true`, ingest capabilities available. |
| Start fixture runtime | `POST /runtime/start` | fixture account | lifecycle becomes `running`, runtime event emitted. |
| Stop fixture runtime | `POST /runtime/stop` | running fixture account | lifecycle becomes `stopped`, runtime event emitted. |
| Remove fixture runtime | `POST /runtime/remove` | fixture account | lifecycle becomes `removed`, removed response returned. |
| List without removed | `GET /accounts` | removed account | removed account excluded. |
| List with removed | `GET /accounts?include_removed=true` | removed account | removed account included. |
| Register live blocked user | `POST /accounts` | `auth_shape=oauth_user` | `provider_kind=zoom_user`, `lifecycle_state=blocked`. |
| Register live blocked S2S | `POST /accounts` | `auth_shape=server_to_server` | `provider_kind=zoom_server_to_server`, `lifecycle_state=blocked`. |

## Bridge tests

| Scenario | Route | Fixture input | Expected result |
|---|---|---|---|
| Meeting observation | `POST /runtime-bridge/meetings` | account id and meeting id | provider call upserted, `zoom.meeting.observed` appended/broadcast. |
| Meeting with participants | `POST /runtime-bridge/meetings` | participant snapshots | participants preserved as metadata/evidence. |
| Recording observation | `POST /runtime-bridge/recordings` | recording id and meeting id | `zoom.recording.observed` appended/broadcast. |
| Transcript observation | `POST /runtime-bridge/transcripts` | transcript id, meeting id, text | placeholder call ensured, transcript upserted, event appended/broadcast. |
| Transcript without meeting call | `POST /runtime-bridge/transcripts` | transcript arrives first | placeholder provider call created with stable id. |
| VTT transcript file import | `POST /runtime-bridge/transcript-files` | VTT file text | transcript text and timed segments parsed, placeholder call ensured, event appended/broadcast. |
| SRT transcript file import | `POST /runtime-bridge/transcript-files` | SRT file text | transcript text and timed segments parsed. |
| Plain transcript file import | `POST /runtime-bridge/transcript-files` | plain text | transcript text imported with empty segment list. |
| Missing account | any bridge route | unknown account id | invalid request error. |
| Missing meeting id | meeting/recording/transcript bridge | empty meeting id | validation error. |
| Missing transcript text | transcript bridge | empty text | validation error. |
| Empty transcript file text | transcript file bridge | empty file text | validation error. |
| Malformed timed transcript file | transcript file bridge | invalid cue timestamp | validation error. |
| Malformed metadata | bridge route | metadata array/string | validation error. |

## Sanitization tests

Payloads containing these fields at any depth must not be appended or broadcast:

```text
access_token
refresh_token
token
client_secret
webhook_secret
download_token
password
```

Test cases:

| Scenario | Input location | Expected result |
|---|---|---|
| Top-level token | `metadata.access_token` | field removed. |
| Nested token | `metadata.auth.refresh_token` | field removed. |
| Recording download token | `recording.metadata.download_token` | field removed. |
| Participant password field | `participants[].metadata.password` | field removed. |

## Idempotency tests

Stable identifiers should be derived from stable inputs:

```text
call_id: account_id + meeting_id
event_id: kind + account_id + subject_id + optional observation_id
```

Expected behavior:

- repeated same meeting observation upserts same call id;
- same observation id produces same event id;
- changed observation id produces distinct event id for the same subject;
- causation/correlation are preserved when supplied.

## Example fixture payload set

```text
fixtures/zoom/account.fixture.json
fixtures/zoom/meeting.observed.json
fixtures/zoom/recording.observed.json
fixtures/zoom/transcript.observed.json
fixtures/zoom/security.sanitization.json
```

The current codebase does not yet include these fixture files. This matrix
defines the target fixture set.
````

### `docs/integrations/zoom/gap-analysis.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/gap-analysis.md`
- Size bytes / Размер в байтах: `7957`
- Included characters / Включено символов: `7957`
- Truncated / Обрезано: `no`

````markdown
# Zoom Gap Analysis

Status date: 2026-06-28.

The current repository has the Zoom provider foundation implemented. This
document records the remaining gap between the current checkout and a live,
end-to-end Zoom provider runtime.

## Current vs target

| Capability | Current state | Target state |
|---|---|---|
| Account setup | Fixture setup, blocked live metadata, initial OAuth/S2S authorization, explicit token refresh/renewal, token maintenance scan, scheduled token maintenance daemon and token rotation policy are implemented. | Operational provider worker enablement. |
| Runtime lifecycle | Metadata-level start/stop/remove implemented; authorized live accounts start as running and can participate in the recording-sync worker. | Broader live runtime state transitions beyond recording sync after auth. |
| Meeting ingestion | Runtime bridge accepts meeting observations and signed meeting webhooks as provider call evidence; `makosh-zoom-edge-proxy` provides public/edge forwarding and authorized accounts can reconcile managed app event subscriptions. | Broader provider worker coverage beyond webhook/event delivery setup. |
| Recording ingestion | Recording observation event, signed recording webhook normalization, webhook/provider-sync media download/import, and explicit per-import local retention/removal are implemented, gated by explicit privacy opt-in and local blob persistence. | Broader downstream media/document workflows. |
| Transcript ingestion | Runtime bridge stores explicit transcript text, imports already obtained VTT/SRT/plain transcript file text and auto-downloads transcript-like text files from signed `recording.completed` webhooks and authorized provider sync only after explicit privacy opt-in. | Full live provider worker coverage beyond current recording-driven transcript files. |
| Calendar matching | `zoom.meeting.observed` is matched to Calendar events through a downstream workflow and relation projection. | Meeting preparation context packs and broader Calendar-side downstream consumers. |
| Participant identity | `zoom.meeting.observed` participants can create conservative `attach_email_address` identity candidates for exact display-name matches and mirror them into the existing Review inbox flow. | Broader identity-trace and persona-resolution workflows beyond exact display-name candidate generation. |
| Radar | Zoom meeting/recording/transcript evidence is projected into Signal Hub `signal.raw.zoom.*` events and policy-derived `signal.accepted|muted|paused.zoom.*` detection events. | Richer Radar ranking/grouping and follow-up ergonomics beyond the current detection feed. |
| Frontend | Integration API/types/query modules, provider setup/status UI, OAuth/S2S authorization controls, token maintenance controls, local runtime-bridge lab controls, read-only observed call evidence inspection, provider-neutral Communications `calls`/`meetings` evidence views with participant snapshots and recording references, recording import audit inspection and account-scoped event audit inspection exist. | Richer Calendar/Communications downstream workflows beyond the current evidence views. |
| Security | Secret purposes, HostVault token storage/refresh/maintenance, scheduled token maintenance, token rotation policy, credential lifecycle audit events, owner-visible retention-policy settings, event/call metadata sanitization, protected webhook HMAC verification, public edge forwarding, recording import audit/remove surfaces, explicit expired-evidence cleanup, scheduled retention cleanup and account-scoped event audit view exist. | Broader media/document workflow promotion beyond current secure ingestion and cleanup boundaries. |

## Architectural gaps

### Live provider execution

The current implementation can register metadata, complete initial OAuth or
Server-to-Server token exchange, explicitly refresh/renew stored token bundles,
run scheduled maintenance scans over authorized accounts, reconcile managed
Zoom app event subscriptions, and run the authorized recording-sync worker
over recent cloud recordings. The live adapter still needs broader provider
API workers and should feed the same bridge methods rather than adding a
separate domain pathway.

### Public webhook ingress

The protected runtime bridge verifies account-scoped Zoom webhook signatures
before meeting/recording bridge ingestion. `makosh-zoom-edge-proxy` provides
the public/edge ingress path and preserves the raw body,
`x-zm-request-timestamp` and `x-zm-signature` when forwarding to the protected
bridge. Managed provider subscription reconciliation through Zoom APIs is now
implemented for authorized accounts.

### Recording and transcript download worker

The foundation now supports authorized provider-sync recording media imports and
webhook/provider-sync transcript-like text imports. The remaining gap is not the
basic worker itself but broader policy and downstream operational coverage:

```text
verified provider event
  -> secret-safe download worker
  -> local blob/document store
  -> media/document scan
  -> evidence event
  -> review/workflow promotion
```

Importing already obtained VTT, SRT and plain text transcript file content is
implemented through the runtime bridge. Signed `recording.completed` webhooks
best-effort download transcript-like text files when Zoom provides a
`download_url` plus `download_token`, but only after
`privacy.zoom_remote_transcript_download_enabled` is explicitly enabled.
Authorized recording sync now also best-effort downloads non-transcript
recording media files after
`privacy.zoom_remote_recording_download_enabled` is explicitly enabled, stores
them through the local communication blob store and records attachment-import
metadata plus heuristic scan status. Макошь now exposes recording import audit
plus explicit per-import remove controls for those blobs, the account event log
now records authorization completion plus token refresh success/skip/failure
activity, and owner-visible retention settings now stamp expiry intent into
recording-import metadata and transcript provenance. Макошь now also exposes an
explicit owner-triggered cleanup surface for expired recording imports and
expired transcript evidence, plus a local scheduled cleanup daemon that prunes
the same expired evidence automatically. Remaining work is richer downstream
workflow promotion beyond the current stamped policy, cleanup and audit path.

### Calendar matching

Meeting evidence should be matched to Calendar through workflow/query ports.
Zoom must not create or mutate calendar events directly.

### Participant identity resolution

Participant emails and display names now feed conservative
`attach_email_address` identity candidates when a Zoom participant exactly
matches an existing Persona display name. The remaining gap is broader
identity-trace coverage and resolution heuristics beyond that exact-match
candidate path.

### Signal detection

Zoom meeting, recording and transcript evidence now feeds downstream Signal Hub
detection through policy-aware `signal.raw.zoom.*` and derived
`signal.accepted|muted|paused.zoom.*` events. Remaining work is richer
attention-ranking, grouping and owner-facing follow-up ergonomics rather than
basic detection availability.

### Transcript intelligence

Transcript text can feed AI summaries, obligations, decisions and task
candidates. AI output must include source, confidence and evidence and must pass
through Radar/Review before domain mutation.

## Product gaps

- Richer meeting evidence workflows in Calendar and Communications beyond the current provider-neutral Calls/Meetings evidence detail view.
## Testing gaps

- Scheduled refresh daemon and degraded/reauthorization failure-path tests.
- Provider worker tests with mocked provider API responses.
- Frontend interaction tests for the runtime-bridge lab and provider-neutral
  downstream views beyond the current boundary/query coverage.
````

### `docs/integrations/zoom/implementation-plan.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/implementation-plan.md`
- Size bytes / Размер в байтах: `6997`
- Included characters / Включено символов: `6997`
- Truncated / Обрезано: `no`

```markdown
# Zoom Implementation Plan

Status date: 2026-06-27.

This plan keeps Zoom as an integration provider and moves business meaning
through workflows and domain-owned command/query ports.

## Phase 1 - Documentation and boundary closure

Status: `complete`.

Deliverables:

- Zoom README, architecture, API, modules, status, gap analysis and blockers;
- ADR-0102 provider runtime boundary;
- fixture bridge documentation;
- live smoke checklist;
- no `domains/zoom` ownership.

Completion condition: documentation merged with status values that match the
current repository.

## Phase 2 - Foundation implementation

Status: `complete`.

Deliverables:

- backend `integrations/zoom` module;
- migration for provider kinds and secret purposes;
- account setup/list/status lifecycle routes;
- runtime status/start/stop/remove routes;
- meeting, recording and transcript runtime bridge routes;
- event constants and sanitized canonical event append path;
- Signal Hub fixture source registration;
- frontend integration API/types/query key modules.

## Phase 3 - Test coverage hardening

Status: `complete`.

Deliverables:

- backend fixture integration tests for account setup/list/status lifecycle;
- meeting bridge test with provider call evidence assertion;
- recording bridge event append/broadcast assertion;
- transcript bridge call transcript assertion;
- sanitization regression test for nested token-like fields;
- idempotency test for stable event ids and stable call ids;
- frontend query key and invalidation tests.

## Phase 4 - Provider setup UI

Status: `implemented_foundation_ui`.

Deliverables:

- integration settings card for Zoom; implemented;
- fixture account setup form for local/dev; implemented;
- live account metadata form with blocked state explanation; implemented;
- runtime status card; implemented;
- runtime blocker list; implemented;
- runtime bridge lab for local meeting, recording, transcript and transcript
  file ingestion through existing integration mutations; implemented;
- read-only observed call evidence and transcript inspection panel for the
  selected Zoom account through provider-neutral call APIs; implemented;
- OAuth start/complete, Server-to-Server authorize, token refresh and token
  maintenance controls; implemented;
- no product meeting inbox under `frontend/src/integrations/zoom`.

## Phase 5 - Live authorization boundary

Status: `authorization_refresh_implemented`.

Deliverables:

- OAuth user setup flow; implemented for start/complete token exchange;
- Server-to-Server account credential exchange; implemented;
- secret-reference lifecycle; implemented for initial token/client-secret
  HostVault storage and provider-account bindings;
- explicit token refresh/renew route; implemented for OAuth refresh-token and
  Server-to-Server account-credentials renewal;
- token maintenance runner; implemented for scanning authorized accounts and
  refreshing expiring HostVault token bundles;
- scheduled token maintenance daemon; implemented through backend bootstrap,
  Signal Hub runtime gating, HostVault unlock gating and
  `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`;
- token rotation policy; implemented through explicit refresh thresholds,
  proactive maintenance threshold metadata, expiry handling and
  `zoom_token_rotation_required` failure/expiry blocker exposure in runtime
  status;
- audit trail for credential lifecycle; implemented through account-scoped
  authorization-complete plus token-refresh success/skip/failure events on the
  existing Zoom audit surface;
- explicit capability transitions from `blocked` to `available/degraded`;
  implemented for authorization and degraded live runtime metadata.

## Phase 6 - Webhook receiver and verification

Status: `edge_proxy_implemented`.

Deliverables:

- webhook endpoint under integration runtime boundary; implemented for
  protected account-scoped runtime bridge;
- public/edge receiver; implemented as `makosh-zoom-edge-proxy`, which
  forwards raw public Zoom webhook bodies and `x-zm-*` headers to the
  protected runtime bridge;
- signature verification before bridge ingestion; implemented for normal
  webhook events;
- endpoint URL validation response; implemented;
- replay/idempotency guard; timestamp replay window and stable bridge ids are
  implemented, DLQ-backed event processing remains planned;
- normalized bridge calls for meetings and recordings; implemented;
- transcript webhook import; implemented for already obtained VTT/SRT/plain
  text via `/runtime-bridge/transcript-files` and for transcript-like textual
  recording files exposed by signed `recording.completed` payloads with
  `download_url` plus `download_token`;
- provider subscription management; implemented for authorized accounts through
  status/reconcile/remove routes plus live provider API calls.

## Phase 7 - Recording and transcript workers

Status: `transcript_download_webhook_implemented`.

Deliverables:

- manual provider-sync route for authorized cloud recording history; implemented
  for recording list ingestion plus best-effort transcript-like text file
  import through the existing HostVault-backed bearer-token boundary;
- cloud recording reference resolver;
- secret-safe download worker;
- local blob/media persistence;
- document/media scan metadata;
- transcript import pipeline; implemented for already obtained VTT, SRT and
  plain text transcript file contents plus signed webhook-driven download of
  transcript-like textual recording files;
- optional local STT fallback only through explicit policy.

## Phase 8 - Domain workflows

Status: `partially_implemented`.

Deliverables:

- calendar event matching workflow; implemented through
  `zoom.meeting.observed` relation projection;
- participant identity candidate workflow; implemented through conservative
  `attach_email_address` candidate generation into Review;
- meeting follow-up Radar signal workflow;
- transcript note/document candidate workflow;
- task/obligation/decision candidate extraction with source, confidence and
  evidence;
- Review promotion before target domain mutation.

## Phase 9 - Provider-neutral product surfaces

Status: `partially_implemented`.

Deliverables:

- Calls/Calendar meeting evidence view; implemented for provider-neutral
  Communications `calls`/`meetings` evidence reads, broader Calendar-side
  product workflows remain open;
- transcript provenance view; implemented on the Zoom evidence surface and
  provider-neutral transcript reads;
- recording reference/document view; implemented on the Zoom evidence surface
  through recording-reference inspection and import audit/removal controls;
- Radar review cards for meeting follow-ups;
- Knowledge graph evidence links;
- search projection updates.

## Non-goals

- Creating `domains/zoom`.
- Building a Zoom client clone.
- Hidden recording or hidden transcription.
- Automatic domain mutation from AI summaries.
- Provider-specific business cache keys for canonical communication/call data.
```

### `docs/integrations/zoom/integration.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/integration.md`
- Size bytes / Размер в байтах: `5068`
- Included characters / Включено символов: `5068`
- Truncated / Обрезано: `no`

````markdown
# Zoom Integration

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-27.

Zoom is modeled as an external communication provider adapter, not as a Макошь
domain. Business memory remains in canonical domains and engines:
Communications, Calls, Calendar, Radar, Timeline, Documents and AI extraction.

Current repository state: the foundation integration is implemented in this
checkout, including account metadata, runtime lifecycle state, runtime bridge
ingestion, live authorization token exchange, protected webhook verification
and event publication.
The integration settings UI also exposes a local runtime bridge lab for
meeting, recording, transcript and transcript-file ingestion against the
selected Zoom account.
The same integration surface now exposes read-only observed call evidence and
projected transcript inspection for the selected Zoom account through the
provider-neutral call APIs.

## Boundary

```text
External Zoom
  -> integrations/zoom
  -> zoom.*.observed events
  -> Calls / Communications / Calendar workflows
  -> Radar / Review / target domains
```

The integration owns provider account metadata, runtime lifecycle state,
sanitized runtime bridge ingestion and event publication. It must not own tasks,
Personas, organizations, notes, documents or calendar truth.

## Runtime shape

The implemented foundation is intentionally conservative:

- fixture Zoom accounts can be registered for local validation;
- live OAuth and server-to-server account metadata can be registered;
- OAuth user and Server-to-Server credentials can be exchanged and stored
  through HostVault-backed secret references;
- OAuth user and Server-to-Server token bundles can be explicitly refreshed or
  renewed in HostVault;
- authorized accounts can be scanned by token maintenance for expiring bundle
  renewal;
- started authorized live accounts can participate in the scheduled
  recording-sync worker;
- meeting observations are projected into provider call evidence;
- recording observations are event-sourced and sanitized;
- transcript observations are stored in call transcripts and linked to the
  meeting evidence;
- already obtained VTT, SRT and plain text transcript file contents can be
  imported into the same call transcript evidence path;
- signed `recording.completed` payloads can auto-download non-transcript
  recording media files through `download_url` plus `download_token` and store
  them through the local communication blob pipeline after explicit privacy
  opt-in;
- signed `recording.completed` payloads can auto-download transcript-like text
  files through `download_url` plus `download_token` and import them into the
  same transcript evidence path;
- account-scoped webhook URL validation and signed meeting/recording webhook
  normalization are supported on the protected runtime bridge;
- all cross-boundary output uses canonical event envelopes with
  causation/correlation support.

## Target backend routes

```text
GET  /api/v1/integrations/zoom/capabilities
GET  /api/v1/integrations/zoom/accounts
POST /api/v1/integrations/zoom/accounts
POST /api/v1/integrations/zoom/fixtures/accounts
POST /api/v1/integrations/zoom/oauth/start
POST /api/v1/integrations/zoom/oauth/complete
POST /api/v1/integrations/zoom/oauth/server-to-server/authorize
POST /api/v1/integrations/zoom/oauth/refresh
POST /api/v1/integrations/zoom/oauth/maintenance
POST /api/v1/integrations/zoom/provider-sync/recordings
GET  /api/v1/integrations/zoom/webhook-subscriptions/status?account_id=...
POST /api/v1/integrations/zoom/webhook-subscriptions/reconcile
POST /api/v1/integrations/zoom/webhook-subscriptions/remove
GET  /api/v1/integrations/zoom/runtime/status?account_id=...
GET  /api/v1/integrations/zoom/accounts/{account_id}/runtime/status
GET  /api/v1/integrations/zoom/accounts/{account_id}/recording-imports?limit=...
POST /api/v1/integrations/zoom/accounts/{account_id}/recording-imports/{attachment_id}/remove
GET  /api/v1/integrations/zoom/accounts/{account_id}/audit-events?limit=...
POST /api/v1/integrations/zoom/runtime/start
POST /api/v1/integrations/zoom/runtime/stop
POST /api/v1/integrations/zoom/runtime/remove
POST /api/v1/integrations/zoom/runtime-bridge/meetings
POST /api/v1/integrations/zoom/runtime-bridge/recordings
POST /api/v1/integrations/zoom/runtime-bridge/transcripts
POST /api/v1/integrations/zoom/runtime-bridge/transcript-files
POST /api/v1/integrations/zoom/runtime-bridge/webhooks?account_id=...
```

## Target events

```text
zoom.authorization.completed
zoom.runtime.status_changed
zoom.token.refreshed
zoom.token.refresh.skipped
zoom.token.refresh.failed
zoom.meeting.observed
zoom.recording.observed
zoom.transcript.observed
zoom.recording.import.removed
```

Secrets and token-like fields must be stripped from event payloads before
append/broadcast.

## Provider kinds

```text
zoom_user
zoom_server_to_server
```

## Secret purposes

```text
zoom_oauth_token
zoom_client_secret
zoom_webhook_secret
```

These are references only. Макошь domains store references and lifecycle state,
never raw Zoom credentials.
````

### `docs/integrations/zoom/live-smoke-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/live-smoke-checklist.md`
- Size bytes / Размер в байтах: `4992`
- Included characters / Включено символов: `4992`
- Truncated / Обрезано: `no`

````markdown
# Zoom Live Smoke Checklist

Status date: 2026-06-27.

The current repository implements Zoom authorization and integration bridge
foundation plus the authorized recording-sync worker. This checklist validates
blocked-live and authorized-live runtime behavior in the current phase.

## Future blocked-live smoke check

### Preconditions

- Backend is running locally.
- Database migrations are applied.
- Fixture routes are enabled only in local/dev mode.
- No real secret values are placed in JSON metadata.
- `/api/v1/integrations/zoom/*` routes exist.

### Check 1 - Capabilities

```http
GET /api/v1/integrations/zoom/capabilities
```

Expected:

```text
runtime_mode = fixture_plus_authorized_live_workers
accounts.fixture = available
accounts.live_blocked = degraded
auth.oauth_user = available
auth.server_to_server = available
auth.token_refresh = available
bridge.meetings = available
bridge.recordings = available
bridge.transcripts = available
bridge.transcript_files = available
```

### Check 2 - Register blocked live account metadata

```http
POST /api/v1/integrations/zoom/accounts
```

Use secret references only:

```json
{
  "account_id": "zoom_live_owner",
  "display_name": "Owner Zoom",
  "external_account_id": "external-account-id",
  "account_email": "owner@example.com",
  "auth_shape": "oauth_user",
  "client_id": "client-id-or-public-ref",
  "token_secret_ref": "secret_ref_zoom_token",
  "webhook_secret_ref": "secret_ref_zoom_webhook",
  "metadata": {}
}
```

Expected:

```text
lifecycle_state = blocked
runtime_kind = zoom_live_blocked_runtime
runtime_blockers contains zoom_live_authorization_required
```

### Check 3 - Start blocked live runtime

```http
POST /api/v1/integrations/zoom/runtime/start
```

Expected:

```text
status = blocked
healthy = false
live_runtime_available = false
zoom.runtime.status_changed emitted
```

### Check 4 - Complete live authorization

For OAuth user accounts:

```http
POST /api/v1/integrations/zoom/oauth/start
POST /api/v1/integrations/zoom/oauth/complete
```

For Server-to-Server accounts:

```http
POST /api/v1/integrations/zoom/oauth/server-to-server/authorize
```

Expected:

```text
lifecycle_state = authorized
runtime_kind = zoom_live_authorized_runtime
token_secret_ref points to a host_vault secret_reference
raw access_token, refresh_token and client_secret absent from account config
```

### Check 5 - Start authorized live runtime

```http
POST /api/v1/integrations/zoom/runtime/start
```

Expected:

```text
status = running
healthy = true
live_runtime_available = true
runtime_blockers empty unless token rotation or another runtime error is active
zoom.runtime.status_changed emitted
```

### Check 6 - Scheduled recording sync capability

Expected:

```text
provider_sync.recordings.scheduler = available
started authorized runtimes can be polled by the local recording-sync daemon
```

### Check 7 - Refresh authorized token bundle

```http
POST /api/v1/integrations/zoom/oauth/refresh
```

Expected:

```text
refreshed = true or skipped_not_expired
token_secret_ref points to the existing host_vault secret_reference
raw access_token, refresh_token and client_secret absent from response and account config
```

## Fixture bridge smoke check

### Check 8 - Register fixture account

```http
POST /api/v1/integrations/zoom/fixtures/accounts
```

Expected:

```text
auth_shape = fixture
lifecycle_state = fixture_ready
```

### Check 9 - Start fixture runtime

Expected:

```text
status = running
healthy = true
zoom.runtime.status_changed emitted
```

### Check 10 - Ingest meeting observation

Expected:

```text
provider call evidence upserted
zoom.meeting.observed emitted
correlation_id preserved or generated
```

### Check 11 - Ingest recording observation

Expected:

```text
zoom.recording.observed emitted
secret-like fields absent from event payload
```

### Check 12 - Ingest transcript observation

Expected:

```text
call transcript upserted
placeholder call created if transcript arrives first
zoom.transcript.observed emitted
```

## Future live smoke check before enabling runtime

Do not enable live runtime until these checks exist:

- OAuth and Server-to-Server authorization store tokens through HostVault
  secret references only.
- Token refresh never writes raw token to provider account config.
- Webhook signature verification rejects invalid signatures.
- Replay/idempotency guard handles repeated provider events.
- Recording worker writes local blobs through approved storage boundary.
- Transcript import records provenance and source reference.
- Runtime lifecycle actions are audited.
- Consent/no-hidden-recording policy is visible and enforced.

## Rollback check

Runtime removal should mark lifecycle as `removed`, not destructively delete
provider account history.

```http
POST /api/v1/integrations/zoom/runtime/remove
```

Expected:

```text
removed = true
removed_at set
GET /accounts excludes it by default
GET /accounts?include_removed=true includes it
```
````

### `docs/integrations/zoom/modules.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/modules.md`
- Size bytes / Размер в байтах: `3982`
- Included characters / Включено символов: `3982`
- Truncated / Обрезано: `no`

````markdown
# Zoom Module Map

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-27.

Current repository state: the modules below are present in this checkout.

## Target backend modules

```text
backend/src/integrations/zoom/
|-- client.rs
|-- runtime.rs
`-- client/
    |-- errors.rs
    |-- models.rs
    |-- store.rs
    `-- validation.rs
```

| Module | Responsibility |
|---|---|
| `client.rs` | Public integration client module surface and re-exports. |
| `runtime.rs` | Runtime module placeholder/surface. |
| `client/models.rs` | DTOs, provider constants, runtime models, OAuth/S2S authorization, token-refresh and token-maintenance DTOs, meeting/recording/transcript request models, transcript file parser and shared sanitization helper. |
| `client/store.rs` | Account setup, OAuth/S2S token exchange, refresh and maintenance, HostVault-backed secret-reference storage, runtime lifecycle, bridge ingestion, transcript file import and event append/broadcast. |
| `client/errors.rs` | Zoom-specific error type and conversions. |
| `client/validation.rs` | Request validation helpers for non-empty ids and JSON shape checks. |

## Target backend app surface

```text
backend/src/app/provider_runtime_handlers/zoom.rs
backend/src/app/handlers/zoom.rs
backend/src/app/error/response/integrations/zoom.rs
backend/src/app/router/routes/messaging.rs
backend/src/bin/makosh_zoom_edge_proxy.rs
```

| Module | Responsibility |
|---|---|
| `provider_runtime_handlers/zoom.rs` | Axum route handlers for capabilities, accounts, OAuth/S2S authorization, refresh and maintenance, runtime lifecycle, bridge ingestion and protected webhook verification. |
| `handlers/zoom.rs` | Route handler re-export surface. |
| `error/response/integrations/zoom.rs` | Integration error response mapping. |
| `router/routes/messaging.rs` | Route registration under `/api/v1/integrations/zoom`. |
| `bin/makosh_zoom_edge_proxy.rs` | Public/edge webhook proxy that preserves raw Zoom bodies and `x-zm-*` headers while adding local Макошь auth. |

## Shared platform dependencies

| Platform module | Usage |
|---|---|
| `platform/communications` | Provider account command port and provider kind metadata. |
| `platform/calls` | Provider call evidence and transcript persistence. |
| `platform/events` | Canonical event envelope append and realtime broadcast. |
| `platform/events/bus.rs` | Zoom event type constants. |
| `platform/secrets` and `vault` | Secret reference metadata and HostVault credential payload storage. |
| `fixtures/signal_hub/system_sources.toml` | Registers Zoom as provider source in Signal Hub fixtures. |

## Migration

```text
backend/migrations/0160_add_zoom_provider_kind.sql
```

The migration adds Zoom provider kind support and secret purpose support
required by provider account and secret-reference flows.

## Target frontend modules

```text
frontend/src/integrations/zoom/
|-- api/zoom.ts
|-- queries/zoomQueryKeys.ts
|-- queries/useZoomRuntimeQuery.ts
`-- types/zoom.ts
```

| Module | Responsibility |
|---|---|
| `api/zoom.ts` | API client functions for Zoom integration and authorization routes. |
| `queries/zoomQueryKeys.ts` | TanStack Query key factory for Zoom provider runtime state. |
| `queries/useZoomRuntimeQuery.ts` | Runtime status query hook and integration-only runtime/authorization mutations. |
| `types/zoom.ts` | TypeScript DTOs aligned with backend models. |

## Boundary rules

Allowed:

```text
app -> integrations/zoom runtime API
integrations/zoom -> platform/communications provider account port
integrations/zoom -> platform/calls
integrations/zoom -> platform/events
frontend integrations/zoom -> backend integration API
```

Forbidden:

```text
integrations/zoom -> domains/*
domains/* -> integrations/zoom
frontend domains/* -> frontend integrations/zoom for product communication state
```

Provider-specific caches may be used for runtime/setup state only. Canonical
communication/call/business views should use provider-neutral domains and
projections.
````

### `docs/integrations/zoom/provider-runtime-research.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/provider-runtime-research.md`
- Size bytes / Размер в байтах: `2937`
- Included characters / Включено символов: `2937`
- Truncated / Обрезано: `no`

````markdown
# Zoom Provider Runtime Research Notes

Status date: 2026-06-27.

This document records Макошь-side runtime shape decisions. It is not a vendor
API reference. Before implementing live provider calls, verify the current Zoom
documentation and update this file with source links and exact constraints.

## Runtime shapes represented in Макошь

Макошь should model three account/auth shapes:

```text
fixture
  local deterministic validation

oauth_user
  user-authorized account shape, live runtime initially blocked

server_to_server
  account/server authorization shape, live runtime initially blocked
```

Provider kind mapping:

```text
fixture          -> zoom_user
oauth_user       -> zoom_user
server_to_server -> zoom_server_to_server
```

## Target adapters

A future live implementation can add adapters behind the same store methods:

```text
Webhook Adapter
  -> verify signature
  -> normalize payload
  -> call ZoomStore.observe_meeting / observe_recording / observe_transcript

Recording Worker
  -> resolve recording reference through secret-safe provider client
  -> persist local blob/document evidence
  -> emit recording/document events

OAuth Runtime
  -> store/refresh credentials through secret references
  -> expose health and blockers through runtime status
```

## Non-negotiable Макошь rules

- The adapter must never import business domains.
- The adapter must never create tasks, Personas, organizations, documents or
  calendar events directly.
- The adapter must use events/workflows for downstream effects.
- The adapter must preserve causation and correlation ids when bridging provider
  events.
- The adapter must verify webhook provenance before persistence.
- The adapter must never place raw secrets in provider config, metadata or event
  payloads.

## Open research questions

These must be answered against current provider documentation before live
runtime work:

1. What exact authorization flows and scopes are required for meeting,
   recording and transcript metadata?
2. What webhook events are required for meeting lifecycle, recording lifecycle
   and transcript availability?
3. What signature verification mechanism and replay window should be enforced?
4. What are the retention and download constraints for recordings and
   transcripts?
5. What rate limits affect historical sync and recording downloads?
6. What user-visible consent and audit behavior is required for
   recording/transcript processing?
7. How should failed downloads and partial transcript imports be retried and
   DLQ-backed?

## Documentation update requirement

When live runtime is implemented, update:

```text
docs/integrations/zoom/architecture.md
docs/integrations/zoom/api.md
docs/integrations/zoom/status.md
docs/integrations/zoom/gap-analysis.md
docs/integrations/zoom/blockers.md
docs/integrations/zoom/live-smoke-checklist.md
docs/adr/ADR-0102-zoom-provider-runtime-boundary.md or a successor ADR
```
````

### `docs/integrations/zoom/status.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/status.md`
- Size bytes / Размер в байтах: `12889`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Zoom Implementation Status

Status date: 2026-06-28.

Documentation status: `TARGET_DOCUMENTED`.

Implementation status in this checkout:
`FOUNDATION_IMPLEMENTED`.

ADR status: `ADR-0102` is `Accepted` in this checkout.
The foundation implementation artifacts are present and backend targeted
validation has passed in this environment.

Invariant: Zoom is a provider integration. Zoom does not own business domains.

## Current repository evidence

As of this implementation pass, the current checkout contains:

- `backend/src/integrations/zoom`;
- `frontend/src/integrations/zoom`;
- `backend/src/bin/makosh_zoom_edge_proxy.rs`;
- migration `backend/migrations/0160_add_zoom_provider_kind.sql`;
- `/api/v1/integrations/zoom/*` routes;
- Zoom event constants under the platform event bus.
- account-scoped verified webhook bridge for URL validation, signed meeting
  events and signed recording events.
- public/edge webhook proxy that forwards raw Zoom webhook bodies and
  `x-zm-*` headers to the protected runtime bridge.
- live OAuth user and Server-to-Server authorization boundary that exchanges
  provider tokens and stores credential payloads in HostVault.
- token refresh/renew route that updates OAuth user and Server-to-Server token
  bundles in HostVault without exposing raw access tokens.
- token maintenance route that scans authorized accounts and renews expiring
  HostVault token bundles through the same refresh boundary.
- scheduled token maintenance daemon that invokes the same HostVault-backed
  maintenance boundary through the local background-service bootstrap.
- manual provider-sync route for authorized Zoom cloud recordings that records
  meeting/recording evidence and best-effort imports recording media plus
  transcript-like text files through the same bearer-token boundary.
- scheduled recording sync daemon that periodically syncs recent cloud
  recording metadata for started authorized runtimes through the same
  HostVault-backed provider-sync boundary.
- live webhook subscription status/reconcile/remove routes that use app-owned
  access tokens plus HostVault-backed webhook secret bindings without exposing
  raw provider secrets.
- declared privacy setting `privacy.zoom_remote_recording_download_enabled`
  with backend enforcement for provider-side recording media downloads.
- declared privacy setting `privacy.zoom_remote_transcript_download_enabled`
  with backend enforcement for provider-side transcript-like file downloads.
- declared retention settings `privacy.zoom_recording_import_retention_days`
  and `privacy.zoom_transcript_retention_days`, with stamped retention metadata
  on imported recording blobs and transcript evidence provenance.
- transcript file import route for already obtained VTT, SRT and plain text
  transcript files.
- signed recording webhook downloader that imports non-transcript recording
  media files and transcript-like textual recording files from Zoom
  `download_url` plus `download_token`.
- read-only recording import audit route plus settings-panel inspection for
  imported Zoom recording blobs.
- explicit recording import retention/remove route plus settings-panel control
  for deleting imported Zoom recording blobs from local audit/storage state.
- explicit retention cleanup route plus settings-panel control for pruning
  expired imported Zoom recording blobs and expired transcript evidence using
  owner-visible retention settings.
- scheduled retention cleanup daemon that periodically prunes expired imported
  Zoom recording blobs and expired transcript evidence through the same local
  retention boundary.
- read-only Zoom event audit route plus settings-panel inspection for recent
  authorization, refresh/maintenance, runtime and bridge events on the
  selected account.
- downstream Zoom signal-detection workflow that projects
  `zoom.meeting.observed`, `zoom.recording.observed` and
  `zoom.transcript.observed` into Signal Hub `signal.raw.zoom.*` events and
  policy-derived `signal.accepted|muted|paused.zoom.*` detection events.
- provider-neutral Communications `calls` and `meetings` sections that read the
  shared call evidence store and surface projected Zoom meeting/transcript
  evidence, participant snapshots and recording references outside the
  integration settings surface.

Targeted Zoom backend validation passes in this environment:
`CARGO_TARGET_DIR=target/zoom-verify ./scripts/test/run-nextest.sh integration --test zoom_provider_foundation --test zoom_signal_detection --test zoom_calendar_matching --test zoom_participant_identity`
(`27` tests passed, `0` failed).
Frontend targeted checks do pass:
`pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/queries/zoomQueryKeys.test.ts src/platform/bootstrap/realtimeZoomInvalidation.test.ts src/domains/communications/api/callApi.test.ts`.
Backend/frontend static checks pass:
`make backend-fmt-check`, `make backend-clippy`, `node scripts/check-architecture.mjs`,
`node scripts/check-code-boundaries.mjs`, `pnpm lint`, `pnpm typecheck` and
`git diff --check`.

## Target foundation summary

| Area | Target state | Current repo state |
|---|---|---|
| Integration boundary | Implement under `backend/src/integrations/zoom`, not `domains/zoom`. | Present. |
| Fixture account setup | Deterministic local validation route. | Present and targeted-tested. |
| Live account metadata | Metadata and secret references registered in blocked mode. | Present and targeted-tested. |
| Live authorization | OAuth user and Server-to-Server token exchange with HostVault storage. | Present and targeted-tested. |
| Token refresh | Explicit OAuth refresh-token and Server-to-Server renew route. | Present and targeted-tested. |
| Token maintenance | Scan authorized accounts and refresh expiring token bundles. | Present and targeted-tested. |
| Scheduled token maintenance | Background scheduler invokes token maintenance with Signal Hub and HostVault gates. | Present and targeted-tested. |
| Manual recording provider sync | Authorized accounts can manually fetch Zoom cloud recording metadata and best-effort recording media plus transcript-like text files through the integration boundary. | Present and targeted-tested. |
| Webhook subscription management | Authorized accounts can query, reconcile and remove managed Zoom app event subscriptions through live provider APIs using app-owned access tokens. | Present and targeted-tested. |
| Remote transcript download consent policy | Provider-side transcript-like file downloads require explicit owner-visible opt-in through application settings. | Present and targeted-tested. |
| Runtime status | Query and account-path status routes. | Present and targeted-tested. |
| Runtime start/stop/remove | Lifecycle state updates and runtime status events. | Present and targeted-tested. |
| Meeting bridge | Upsert provider-neutral call evidence and emit event. | Present and targeted-tested. |
| Recording bridge | Emit sanitized recording evidence event. | Present and targeted-tested. |
| Transcript bridge | Upsert call transcript and emit event. | Present and targeted-tested. |
| Transcript file import | Parse VTT/SRT/plain transcript file text into call transcript evidence and auto-import transcript-like text files from signed recording webhooks. | Present and targeted-tested. |
| Recording webhook media import | Best-effort import non-transcript recording media files from signed recording webhooks after explicit owner-visible opt-in, local blob persistence and heuristic safety scan. | Present and targeted-tested. |
| Recording import audit view | Read-only route and settings-panel inspection for imported Zoom recording blobs, including source, scan status and storage metadata. | Present and targeted-tested. |
| Recording import retention control | Explicit per-import remove route and settings-panel control that deletes imported Zoom recording blobs from local audit/storage state and emits a follow-up audit event. | Present and targeted-tested. |
| Owner-visible retention policy | Application settings define retention days for imported recording blobs and transcript evidence; imports stamp retention mode and expiry intent into audit/provenance metadata. | Present and targeted-tested. |
| Retention cleanup control | Explicit account-scoped prune route and settings-panel control remove expired imported recording blobs and expired transcript evidence using stamped retention expiry intent. | Present and targeted-tested. |
| Scheduled retention cleanup | Background scheduler periodically prunes expired imported recording blobs and expired transcript evidence through the same local retention boundary. | Present and targeted-tested. |
| Credential lifecycle audit trail | Account-scoped Zoom event log records authorization completion plus token refresh success, skip and failure events, including maintenance-triggered refresh decisions. | Present and targeted-tested. |
| Runtime and bridge event audit view | Read-only route and settings-panel inspection for recent Zoom runtime and bridge events from the account-scoped event log. | Present and targeted-tested. |
| Event contract | Zoom event types registered and canonical envelopes used. | Present. |
| Payload sanitization | Token-like fields stripped recursively before event append/broadcast. | Present and targeted-tested. |
| Signal Hub source | Zoom provider source fixture registered. | Present and targeted-tested. |
| Signal detection workflow | Downstream Signal Hub detection path, not integration ownership. | Implemented and targeted-tested via `zoom.*` -> `signal.raw.zoom.*` -> policy-derived Signal Hub events. |
| Frontend API client/types | Integration API client, types, runtime query and settings UI controls for fixture/live setup, OAuth/S2S authorization, token maintenance, local runtime-bridge ingestion and read-only observed call evidence inspection, including recording references and transcript provenance. | Present and targeted-tested. |
| Provider-neutral Calls/Meetings view | Communications `calls` and `meetings` sections surface shared call evidence, transcript reads, participant snapshots and recording references, including projected Zoom meetings. | Present and targeted-tested. |
| Live provider runtime | Authorized accounts can be started into a running live runtime, and the background recording-sync worker polls recent Zoom cloud recordings through the existing provider-sync boundary. | Present and targeted-tested. |
| Webhook signature verification | Protected account-scoped runtime bridge. | Present and targeted-tested for URL validation, meeting events, recording events and invalid signatures. |
| Public/edge webhook receiver | Standalone `makosh-zoom-edge-proxy`. | Present and targeted-tested for readiness, raw body forwarding, Zoom header forwarding and account scoping. |
| Cloud recording download worker | Authorized accounts can best-effort import provider-side recording media files through the existing recording-sync boundary and signed recording webhooks after explicit owner-visible opt-in, local blob persistence and heuristic safety scan. | Present and targeted-tested. |
| Calendar matching workflow | Downstream workflow, not integration ownership. | Implemented and targeted-tested via `zoom.meeting.observed` -> Calendar event relation projection. |
| Participant identity resolution | Downstream candidate/review workflow. | Implemented and targeted-tested for conservative `attach_email_address` identity-candidate generation from `zoom.meeting.observed` participants into the existing Review inbox pipeline. |

## Target capability state

```text
available in the foundation implementation:
  accounts.fixture
  auth.oauth_user
  auth.server_to_server
  auth.token_refresh
  auth.token_maintenance
  auth.token_rotation_policy
  token_maintenance.scheduler
  provider_sync.recordings
  provider_sync.recordings.scheduler
  provider_sync.recording_media_downloads
  recording_imports
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/zoom/status/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/status/README.md`
- Size bytes / Размер в байтах: `240`
- Included characters / Включено символов: `240`
- Truncated / Обрезано: `no`

```markdown
# Zoom Status Details

Status: documentation package aligned to the current repository structure.

This package stores detailed Zoom implementation evidence split from the parent
status document.

## Navigation

- [Pass Log](./pass-log.md)
```

### `docs/integrations/zoom/status/pass-log.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/status/pass-log.md`
- Size bytes / Размер в байтах: `9386`
- Included characters / Включено символов: `9386`
- Truncated / Обрезано: `no`

````markdown
# Zoom Pass Log

Status date: 2026-06-28.

This pass log tracks documentation and implementation closure for the Zoom
provider stage.

## Documentation pass

| Check | Result | Evidence |
|---|---|---|
| Provider boundary documented | PASS | `docs/integrations/zoom/architecture.md`, ADR-0102. |
| No `domains/zoom` ownership introduced | PASS | Documentation specifies `integrations/zoom` target only. |
| Fixture account setup documented | PASS | `docs/integrations/zoom/api/accounts.md`. |
| Live account blocked mode documented | PASS | `docs/integrations/zoom/api/accounts.md`, `docs/integrations/zoom/status.md`. |
| Runtime lifecycle documented | PASS | `docs/integrations/zoom/api/runtime.md`. |
| Meeting bridge documented | PASS | `docs/integrations/zoom/api/runtime-bridge.md`. |
| Recording bridge documented | PASS | `docs/integrations/zoom/api/runtime-bridge.md`. |
| Transcript bridge documented | PASS | `docs/integrations/zoom/api/runtime-bridge.md`. |
| Transcript file import documented | PASS | `docs/integrations/zoom/api/runtime-bridge.md`. |
| Event contract documented | PASS | `docs/integrations/zoom/architecture.md`. |
| Sanitization boundary documented | PASS | `docs/integrations/zoom/architecture.md`, `docs/integrations/zoom/api/runtime-bridge.md`. |
| Gaps and blockers documented | PASS | `docs/integrations/zoom/gap-analysis.md`, `docs/integrations/zoom/blockers.md`. |
| Implementation status matches current checkout | PASS | `docs/integrations/zoom/status.md` marks Zoom as `FOUNDATION_IMPLEMENTED`. |

## Implementation pass

| Check | Result | Evidence |
|---|---|---|
| Backend integration module exists | PASS | `backend/src/integrations/zoom`. |
| Zoom edge proxy binary exists | PASS | `backend/src/bin/makosh_zoom_edge_proxy.rs`. |
| Frontend integration module exists | PASS | `frontend/src/integrations/zoom`. |
| Zoom migration exists | PASS | `backend/migrations/0160_add_zoom_provider_kind.sql`. |
| Zoom targeted backend tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom OAuth user authorization tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom Server-to-Server authorization tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom token refresh tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom token maintenance tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom credential lifecycle audit tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom capabilities contract reflects implemented downstream workflows | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom owner-visible retention policy tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation`; `cargo test --manifest-path backend/Cargo.toml --lib zoom_recording_import_retention_setting_is_declared_as_editable_integer`; `cargo test --manifest-path backend/Cargo.toml --lib zoom_transcript_retention_setting_is_declared_as_editable_integer`. |
| Zoom retention cleanup control tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation`; `cd frontend && pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/components/ZoomSettingsPanel.boundary.test.ts`; `cd frontend && pnpm typecheck`. |
| Zoom scheduled retention cleanup wiring tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation`; `cargo test --manifest-path backend/Cargo.toml --test config`; `cargo test --manifest-path backend/Cargo.toml --lib zoom_retention_cleanup_scheduler_registration_is_once_per_database_url`. |
| Zoom webhook subscription management tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom scheduled token maintenance tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom protected webhook bridge tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom webhook transcript download/import tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom webhook recording media download/import tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom recording import audit route tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom recording import retention/remove tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation --test mail_storage` |
| Zoom runtime/bridge audit route tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom manual recording provider-sync tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom recording media download/import tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom calendar matching workflow tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_calendar_matching`; `cargo test --manifest-path backend/Cargo.toml --lib zoom_calendar_matching_consumer_registration_is_once_per_database_url` |
| Zoom signal detection workflow tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_signal_detection`; `cargo test --manifest-path backend/Cargo.toml --lib zoom_signal_detection_consumer_registration_is_once_per_database_url` |
| Zoom participant identity workflow tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_participant_identity`; `cargo test --manifest-path backend/Cargo.toml --lib zoom_participant_identity_consumer_registration_is_once_per_database_url` |
| Zoom remote transcript download privacy policy tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom remote recording media download privacy policy tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom transcript file import tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation` |
| Zoom edge proxy tests pass | PASS | `cargo test --manifest-path backend/Cargo.toml --bin makosh-zoom-edge-proxy`. |
| Zoom targeted frontend tests pass | PASS | `pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/queries/zoomQueryKeys.test.ts src/platform/bootstrap/realtimeZoomInvalidation.test.ts`. |
| Zoom recording import retention/remove frontend API wiring tests pass | PASS | `cd frontend && pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/components/ZoomSettingsPanel.boundary.test.ts`; `cd frontend && pnpm typecheck`. |
| Zoom recording import audit frontend wiring tests pass | PASS | `cd frontend && pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/queries/zoomQueryKeys.test.ts src/integrations/zoom/components/ZoomSettingsPanel.boundary.test.ts`; `cd frontend && pnpm typecheck`. |
| Provider-neutral Communications Calls/Meetings evidence view tests pass | PASS | `cd frontend && pnpm exec vitest run src/domains/communications/api/callApi.test.ts src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts src/domains/communications/views/CommunicationsPage.boundary.test.ts`; `cd frontend && pnpm typecheck`. |
| Provider-neutral Communications meeting evidence detail tests pass | PASS | `cd frontend && pnpm exec vitest run src/domains/communications/components/CommunicationsCallsPanel.boundary.test.ts src/domains/communications/views/CommunicationsPage.boundary.test.ts`; `cd frontend && pnpm typecheck`; `cd frontend && pnpm lint`. |
| Shared Calls route filters Zoom evidence by `provider=zoom` | PASS | `cargo test --manifest-path backend/Cargo.toml --test zoom_provider_foundation`. |
| Zoom realtime invalidation covers recording import and audit keys | PASS | `cd frontend && pnpm exec vitest run src/platform/bootstrap/realtimeZoomInvalidation.test.ts`. |
| Zoom evidence panel recording/provenance frontend tests pass | PASS | `cd frontend && pnpm exec vitest run src/integrations/zoom/components/zoomEvidence.test.ts src/integrations/zoom/components/ZoomObservedCallsPanel.boundary.test.ts`; `cd frontend && pnpm typecheck`; `cd frontend && pnpm lint`. |
| Full backend validation gate | BLOCKED | Not rerun in this environment while container backends (`MAKOSH_TEST_POSTGRES_HOST_PORT`/`MAKOSH_TEST_NATS_HOST_PORT` or Docker sockets) are unavailable. |
| Frontend lint/typecheck gate | PASS | `cd frontend && pnpm lint`; `cd frontend && pnpm typecheck`. |
| Diff whitespace gate | PASS | `git diff --check`. |

## Local validation pass for future implementation

Use the repository-configured validation commands. For backend-only Zoom work,
prefer:

```bash
make backend-validate
```

For broad backend/frontend work, prefer:

```bash
make validate
```
````

### `docs/platform/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/README.md`
- Size bytes / Размер в байтах: `1350`
- Included characters / Включено символов: `1350`
- Truncated / Обрезано: `no`

```markdown
# Макошь Platform Layer

Status: documentation package aligned to the current repository structure.

The platform layer mirrors `backend/src/platform`.

Platform modules provide technical primitives used by domains, integrations,
workflows and the app layer. Platform code owns infrastructure contracts, not
business source-of-truth entities.

## Current Documentation Packages

- [Event Tracing](event-tracing/README.md)
- [Application Settings](settings/README.md)
- [Realtime Conversation](realtime-conversation/README.md)

## Current Code Areas

- `platform/events` - event store, event bus, trace context and dispatch.
- `platform/audit` - local audit records.
- `platform/config` - application/runtime configuration parsing.
- `platform/settings` - allowlisted application settings.
- `platform/secrets` - secret reference metadata and resolver contracts.
- `platform/storage` - local storage primitives.
- `platform/calls` - provider-neutral call/transcript evidence primitives.
- `platform/realtime_conversation` - provider-neutral live-conversation
  bundle and provider capability contracts.

## Documentation Rule

Use this folder for reusable technical contracts. If a document starts owning
Tasks, Personas, Communications, provider sessions or product decisions, move
that content to the owning domain, integration or workflow package.
```

### `docs/platform/event-tracing/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/README.md`
- Size bytes / Размер в байтах: `1346`
- Included characters / Включено символов: `1346`
- Truncated / Обрезано: `no`

````markdown
# Event Tracing

Status: documentation package aligned to the current repository structure.

## Purpose

Event Tracing is the causal observability layer of Макошь.

It does not replace the event backbone. It formalizes how persistent events are
connected into traces.

Макошь does not need a separate telemetry server to answer:

- why does this object exist?
- what caused this event?
- what happened before this projection?
- which provider signal created this communication message?
- which review promoted this task?

## Position

The canonical trace store is PostgreSQL append-only `event_log`.

```text
EventEnvelope = Span
event_id = span_id
correlation_id = trace_id
causation_id = parent_span_id
event_log = trace store
```

OpenTelemetry, logs and metrics may export or diagnose traces, but they do not
own the canonical causal graph.

## Package

- [ADR-0100](../../adr/ADR-0100-trace-first-event-observability.md)
- [Architecture](architecture.md)
- [Data model](data-model.md)
- [API](api.md)
- [Testing](testing.md)
- [Operations](operations.md)
- [Gap analysis](gap-analysis.md)
- [Status](status.md)

## Navigation

- [Architecture](./architecture.md)
- [API Reference](./api.md)
- [Data Model](./data-model.md)
- [Status](./status.md)
- [Gap Analysis](./gap-analysis.md)
- [Operations](./operations.md)
- [Testing](./testing.md)
````

### `docs/platform/event-tracing/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/api.md`
- Size bytes / Размер в байтах: `2709`
- Included characters / Включено символов: `2709`
- Truncated / Обрезано: `no`

````markdown
# Event Tracing API

## Purpose

The Event Tracing API exposes causal traces reconstructed from `event_log`.

## Position

Trace APIs are platform event APIs. Provider integrations may link to them but
must not own provider-specific trace endpoints.

## Endpoints

```text
GET /api/v1/events/{event_id}/trace
GET /api/v1/event-traces/{correlation_id}
GET /api/v1/events/{event_id}/children
```

`GET /api/v1/events/{event_id}/trace` resolves the anchor event and then loads
its trace by `correlation_id`. If the anchor is a legacy event with null
`correlation_id`, the event id is used as the legacy trace id.

`GET /api/v1/event-traces/{correlation_id}` loads a trace directly by trace id.

`GET /api/v1/events/{event_id}/children` returns events where
`causation_id = event_id`.

## Response Shape

```json
{
  "correlation_id": "obs_123",
  "root_event_ids": [
    "event:v1:observation-captured:obs_123"
  ],
  "events": [
    {
      "position": 1,
      "event": {
        "event_id": "event:v1:observation-captured:obs_123",
        "event_type": "observation.captured.v1",
        "schema_version": 1,
        "occurred_at": "2026-06-24T10:00:00Z",
        "recorded_at": "2026-06-24T10:00:01Z",
        "source": {},
        "actor": null,
        "subject": {},
        "payload": {},
        "provenance": {},
        "causation_id": null,
        "correlation_id": "obs_123"
      }
    }
  ],
  "edges": [
    {
      "parent_event_id": "event:v1:observation-captured:obs_123",
      "child_event_id": "signal_raw_telegram_..."
    }
  ],
  "orphan_event_ids": [],
  "missing_parent_ids": [],
  "consumer_annotations": [],
  "dead_letters": []
}
```

## Realtime Payload Requirements

Realtime event payloads must include:

- `event_id`;
- `event_type`;
- `schema_version`;
- `occurred_at`;
- `recorded_at` when available from stored events;
- `source`;
- `actor`;
- `subject`;
- sanitized `payload`;
- `provenance`;
- `causation_id`;
- `correlation_id`.

Private content must remain sanitized. Trace-specific structures must not store
secrets, tokens, raw private blobs or provider session material.

## Errors

Missing anchor event returns `404` from the event-id trace endpoint.

Invalid empty event id or trace id returns the existing platform validation
error mapping.

## Frontend Surface

The frontend event tracing surface lives in
`frontend/src/platform/event-tracing/`. It calls the platform event endpoints
with provider-neutral query keys:

```text
['events', eventId, 'trace']
['event-traces', correlationId]
['events', eventId, 'children']
```

Provider-specific runtime pages may link to event traces, but they must not own
trace state or provider-specific trace query namespaces.
````

### `docs/platform/event-tracing/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/architecture.md`
- Size bytes / Размер в байтах: `4319`
- Included characters / Включено символов: `4319`
- Truncated / Обрезано: `no`

````markdown
# Event Tracing Architecture

Status: target architecture with partial backend implementation.

## Purpose

Event Tracing reconstructs causal chains from canonical events. It explains how
observations, provider/source signals, domain events, workflow events and
projection events relate to one another.

## Position

Event Tracing belongs to `platform/events`.

Timeline Engine is a chronological projection engine. It may display trace
links, but it must not reconstruct or own traces.

## High-Level Flow

```text
Root Event
  -> Derived Event
  -> Domain Event
  -> Workflow Event
  -> Projection Event / Domain Object Event
```

Communication example:

```text
observation.captured.v1
  -> signal.raw.telegram.message.observed
  -> signal.accepted.telegram.message
  -> communication.message.recorded
  -> radar.signal.detected
  -> review.item.promoted
  -> task.created
```

WhatsApp example:

```text
observation.captured.v1
  -> signal.raw.whatsapp.message.observed
  -> signal.accepted.whatsapp.message
  -> communication.message.recorded
  -> radar.signal.detected
```

Mail example:

```text
observation.captured.v1
  -> signal.raw.mail.message.observed
  -> signal.accepted.mail.message
  -> communication.message.recorded
  -> document.import.requested
```

Calendar example:

```text
signal.calendar.event.observed
  -> calendar.event.recorded
  -> timeline.projection.updated
  -> meeting.preparation.requested
```

## Layer Ownership

| Layer | Owns | Must not own |
|---|---|---|
| `platform/events` | envelope, event store, trace context, trace reconstruction, event API support | business meaning |
| `platform/observations` | root evidence observations | provider protocol logic |
| `domains/signal_hub` | source control state, replay, mute/pause/health policy | communication messages |
| `integrations/*` | provider protocol, runtime, session and command execution | domain state |
| `domains/communications` | canonical messages, conversations, attachments and outbox | provider runtime sessions |
| `workflows/*` | cross-domain orchestration | direct store mutation outside owner |
| `engines/timeline` | chronological projections | causal trace reconstruction |
| `app/*` | HTTP, ConnectRPC, SSE and WebSocket surfaces | business causation decisions |

Trace reconstruction belongs to `platform/events`.

Timeline projection belongs to `engines/timeline`.

Timeline may display trace links, but must not be the trace source of truth.

## Backend Layers

```text
NewEventEnvelopeBuilder
  -> TraceContext
  -> EventStore append
  -> EventStore trace queries
  -> Trace API / realtime surfaces
```

Consumer state, retry metadata and DLQ records annotate trace execution. They
do not create business facts unless a domain explicitly emits an event.

## Frontend Layers

Trace UI belongs to shared platform/event observability surfaces, not provider
product pages.

Allowed locations:

```text
frontend/src/platform/events/*
frontend/src/platform/event-tracing/*
```

Provider runtime UI may link to a trace. Telegram and WhatsApp do not own trace
state, trace query keys or trace transport.

Provider-neutral query keys:

```text
['events', eventId, 'trace']
['event-traces', correlationId]
['events', eventId, 'children']
```

## Event Contracts

Every persistent event has:

```text
event_id
event_type
schema_version
occurred_at
recorded_at
source
actor
subject
payload
provenance
causation_id
correlation_id
```

Root events have no parent but still have a trace id. Derived events inherit
trace id from their parent and set parent span id through `causation_id`.

## API

Trace APIs are platform-level event APIs:

```text
GET /api/v1/events/{event_id}/trace
GET /api/v1/event-traces/{correlation_id}
GET /api/v1/events/{event_id}/children
```

## Testing

Trace tests should cover builder normalization, `TraceContext`, trace graph
reconstruction, missing parents, observation roots, provider fixture chains,
realtime payload fields and DLQ annotations.

## Operations

Operators debug from the domain object back to provenance:

```text
domain object
  -> provenance event id
  -> trace API
  -> root observation
  -> derived signal and workflow chain
  -> consumer and DLQ annotations
```

## Blockers/Gaps

See [gap-analysis.md](gap-analysis.md).

## Status

See [status.md](status.md).
````

### `docs/platform/event-tracing/data-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/data-model.md`
- Size bytes / Размер в байтах: `1835`
- Included characters / Включено символов: `1835`
- Truncated / Обрезано: `no`

````markdown
# Event Tracing Data Model

## Purpose

This document defines the canonical trace graph shape built from `event_log`.

## Position

Trace state is not a separate store. It is a read model reconstructed from
canonical event rows and consumer metadata.

## Canonical Mapping

| Trace concept | Макошь field |
|---|---|
| Trace | `correlation_id` |
| Span | `event_id` |
| Parent span | `causation_id` |
| Trace store | `event_log` |

Root event:

```text
causation_id = null
correlation_id = non-empty
```

Derived event:

```text
causation_id = parent.event_id
correlation_id = parent.correlation_id
```

## Trace Graph Model

```json
{
  "correlation_id": "obs_...",
  "root_event_ids": [],
  "events": [],
  "edges": [],
  "orphan_event_ids": [],
  "missing_parent_ids": [],
  "consumer_annotations": [],
  "dead_letters": []
}
```

`edges` are deterministic parent-child links:

```json
{
  "parent_event_id": "event:v1:observation-captured:obs_123",
  "child_event_id": "signal_raw_telegram_..."
}
```

## Event And Consumer Tables

| Table | Role |
|---|---|
| `event_log` | canonical append-only event and trace store |
| `event_outbox` | pending/delivered event transport dispatch state |
| `event_consumers` | durable consumer cursor and runtime status |
| `event_consumer_processed_events` | processed-event annotations |
| `event_consumer_failures` | retry/failure annotations |
| `event_dead_letters` | DLQ annotations and review state |

Consumer processing metadata is an annotation of trace execution. It must not
be confused with business or domain events.

## Legacy Events

Events created before trace normalization may have null `correlation_id`. When
read by event id, they are shown under a legacy orphan trace whose trace id is
the event id. They should not be silently mutated unless a safe migration plan
exists.
````

### `docs/platform/event-tracing/gap-analysis.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/gap-analysis.md`
- Size bytes / Размер в байтах: `1554`
- Included characters / Включено символов: `1554`
- Truncated / Обрезано: `no`

```markdown
# Event Tracing Gap Analysis

Status: living implementation gap list.

## Known Gaps

- legacy rows can have nullable `correlation_id`;
- realtime stored-event replay and in-memory bus payloads have different
  `recorded_at` availability;
- observation events are now root trace events in the canonical store path, but
  older observations remain legacy rows;
- raw provider/source signals are trace-linked in the current raw record paths,
  but older events remain disconnected;
- accepted Mail, Telegram and WhatsApp message signals now emit canonical
  `communication.message.recorded` or `communication.message.updated` events
  with inherited trace context after projection;
- Timeline Engine expects subject shapes and must not be used as Trace Engine;
- provider observation events can use idempotency key as root correlation id,
  but derived provider observations must prefer parent trace context;
- frontend trace UI exists as a shared platform surface; embedding links from
  domain detail pages remains follow-up work.

## Watchpoints

- Do not add Telegram-specific or WhatsApp-specific trace ownership.
- Do not make OpenTelemetry the trace source of truth.
- Do not infer missing trace links with AI.
- Do not store private content in trace-specific structures.
- The canonical builder guarantees a non-empty `correlation_id`, but it cannot
  infer whether a new event is semantically root or derived. New derived event
  paths must set `causation_id` explicitly or use `TraceContext` and add
  regression coverage for their causal chain.
```

### `docs/platform/event-tracing/operations.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/operations.md`
- Size bytes / Размер в байтах: `1514`
- Included characters / Включено символов: `1514`
- Truncated / Обрезано: `no`

````markdown
# Event Tracing Operations

## Purpose

Event Tracing gives local operators and developers a deterministic way to
explain domain state.

## Questions Trace Must Answer

- Why does this task exist?
- Which provider event created this message?
- Which observation is the root evidence?
- Which consumer failed?
- Can this trace be replayed?
- Which events are missing parent links?

## Debug Flow

```text
Open task
  -> get provenance event id
  -> fetch trace
  -> inspect root observation
  -> inspect workflow chain
  -> inspect consumer annotations
```

Communication debug flow:

```text
Open communication message
  -> read message provenance event id
  -> GET /api/v1/events/{event_id}/trace
  -> inspect provider/source signal chain
  -> inspect Signal Hub accepted/rejected state
  -> inspect consumer/DLQ annotations
```

## Legacy Events

Events created before trace normalization may have null `correlation_id`. They
should be displayed as `legacy_orphan_trace` unless migrated.

Do not rewrite append-only event rows casually. Any backfill must have a safe
migration plan, clear validation and rollback posture.

## Replay

Replay decisions use `event_log` and consumer state. Replaying a trace should
not invent missing parent links. Missing parent ids are data quality findings
that need explicit repair or acceptance.

## Privacy

Trace responses must sanitize private payloads. Trace-specific records must not
store raw message bodies, secrets, provider cookies, access tokens or session
material.
````

### `docs/platform/event-tracing/status.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/status.md`
- Size bytes / Размер в байтах: `2391`
- Included characters / Включено символов: `2391`
- Truncated / Обрезано: `no`

```markdown
# Event Tracing Status

Status date: 2026-06-24.

## Implemented

- ADR-0100 records Trace-First Event Observability.
- Canonical event builder normalizes missing or empty `correlation_id` to
  `event_id`.
- `TraceContext` supports root and child contexts.
- EventStore exposes trace by event id, trace by correlation id and children by
  causation id.
- Trace reconstruction returns roots, edges, orphan events, missing parents,
  consumer annotations and DLQ annotations.
- Observation capture writes `observation.captured.v1` as a root trace event in
  the current store path.
- Raw Mail, Telegram and WhatsApp source signal builders set causation to the
  deterministic observation captured event id in current raw-record paths.
- Signal Hub derived events already set `causation_id = raw_event.event_id` and
  inherit correlation when present.
- Communications emits canonical `communication.message.recorded` or
  `communication.message.updated` events after accepted Mail, Telegram and
  WhatsApp message signals mutate the communication projection.
- Event trace API endpoints exist under platform event routes.
- Shared frontend trace UI exists under `frontend/src/platform/event-tracing/`
  and uses provider-neutral `events` / `event-traces` query keys.
- Telegram, WhatsApp and Mail fixture tests cover complete chains from
  observation to communication event.

## Partially Implemented

- Realtime payloads expose trace fields, but in-memory bus events do not always
  have a stored `recorded_at`.
- Provider observation events support explicit correlation context; derived
  provider observations retain the parent correlation when a caller supplies
  it, while root provider observations may still use their idempotency key as
  trace id.
- Legacy events are readable as orphan traces but are not backfilled.
- Domain detail pages can link to the shared trace surface in future slices;
  trace state itself is already provider-neutral and platform-owned.

## Planned

- DLQ annotation API regression tests.
- Optional OpenTelemetry trace exporter sourced from `event_log`.

## Blocked

- Full live-provider trace validation is blocked on deterministic provider
  fixture coverage and should not require live accounts.

## Deprecated / Superseded

- Treating OpenTelemetry as canonical trace storage is superseded by ADR-0100.
- Provider-specific trace ownership is rejected.
```
