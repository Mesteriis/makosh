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
