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
