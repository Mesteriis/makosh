# Telegram API Reference — Foundation

См. также: [API Index](../api.md).

Статус: verified route audit + целевой API scope на 2026-06-17.

Все текущие маршруты защищены локальным API guard из ADR-0056, если явно не
указано иначе. Browser WebSocket clients передают local secret через
`makosh_secret`, потому что native WebSocket requests не могут выставить
`X-Макошь-Secret`.

## Base

```text
/api/v1/integrations/telegram
```

## Capability Contract

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/integrations/telegram/capabilities` | Detailed per-operation capability matrix with `operation`, `category`, `status`, `action_class`, `reason`, `confirmation_required`, `closure_gate` |
| GET | `/api/v1/integrations/telegram/accounts/{account_id}/capabilities` | Account-scoped capability matrix with the same operation contract plus selected-account scope metadata and runtime/provider overrides |

### Целевые capability states

```text
available
degraded
blocked
unsupported
```

### Целевые operation groups

- account lifecycle;
- runtime;
- sync;
- read/search;
- send;
- edit;
- delete;
- react;
- pin;
- media download/upload;
- export;
- session/proxy;
- calls/recording;
- admin actions.

Account-scoped responses now also include:

```text
account_scope.account_id
account_scope.provider_kind
account_scope.runtime_kind
account_scope.lifecycle_state
```

Current account-aware overrides cover:

- bot accounts not using TDLib QR operations;
- user accounts not using Bot API runtime operations;
- bot accounts not exposing TDLib forum-topic projection/runtime operations;
- account-scoped degraded forum-topic reads when only local projection is available;
- forum-topic write operations available for QR-authorized TDLib user accounts via the durable provider-write outbox;
- `logged_out` / `removed` lifecycle blocking selected runtime/sync/write actions.

Telegram inspector `About` tab now also surfaces the account-scoped capability
route as a read-only matrix for the selected account, exposing operation status,
action class, reason, confirmation requirement and closure-gate metadata
alongside the existing capability-gated controls.

## Accounts

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/integrations/telegram/fixtures/accounts` | Создать fixture `telegram_user` или `telegram_bot` account metadata |
| GET | `/api/v1/integrations/telegram/accounts?include_removed=` | Список Telegram provider accounts |
| POST | `/api/v1/integrations/telegram/accounts` | Создать live/live-blocked/QR-authorized Telegram account metadata и secret bindings |
| DELETE | `/api/v1/integrations/telegram/accounts/{account_id}` | Mark account `removed`, stop runtime actor, preserve local evidence |
| POST | `/api/v1/integrations/telegram/accounts/{account_id}/logout` | Mark account `logged_out`, stop runtime actor |

Account config хранит non-secret metadata. Credential payloads resolved через
host-vault/secret references. Telegram workbench `About` inspector tab now
contains a local account manager UI for `setup`, `logout` and `remove`, and the
header `Add Account` action opens that account-management surface.

### Дополнительно реализовано

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/integrations/telegram/accounts/{account_id}/capabilities` | Account-scoped detailed capability matrix |

### Недостающие маршруты

| Method | Path | Назначение |
|---|---|---|
| GET | `/api/v1/integrations/telegram/accounts/{account_id}/export-session` | Sanitized session bundle export без secrets unless explicitly encrypted |
| POST | `/api/v1/integrations/telegram/accounts/import-session` | Import encrypted session bundle |
| GET/PUT | `/api/v1/integrations/telegram/accounts/{account_id}/proxy` | Proxy / MTProxy / SOCKS5 profile binding |

## Runtime

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/integrations/telegram/runtime/status?account_id=` | Account-scoped runtime status, runtime kind, TDLib readiness, split Telegram app-credential flags, TDLib probe error and derived runtime blockers |
| POST | `/api/v1/integrations/telegram/runtime/start` | Start fixture или TDLib QR-authorized runtime actor |
| POST | `/api/v1/integrations/telegram/runtime/stop` | Stop the account-scoped runtime actor idempotently and return the current runtime status with local audit evidence |
| POST | `/api/v1/integrations/telegram/runtime/restart` | Stop then start the account-scoped runtime actor and return the current runtime status with local audit evidence |

Runtime kinds observed:

```text
fixture
tdlib_qr_authorized
live_blocked
```

Current runtime status payload now also exposes:

```text
tdjson_path
tdjson_probe_error
telegram_api_id_configured
telegram_api_hash_configured
runtime_blockers[]
```

Runtime restart routes are implemented. Native dependency remediation stays in
runtime diagnostics, while Bot API runtime and portable session/proxy controls
are ADR-0094 planned initiatives.

## QR Login

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| POST | `/api/v1/integrations/telegram/login/qr/start` | Start TDLib QR login setup |
| GET | `/api/v1/integrations/telegram/login/qr/{setup_id}` | Poll QR login status |
| DELETE | `/api/v1/integrations/telegram/login/qr/{setup_id}` | Cancel pending QR login session |
| POST | `/api/v1/integrations/telegram/login/qr/{setup_id}/password` | Submit 2-step verification password |

QR statuses:

```text
waiting_qr_scan
waiting_password
ready
expired
failed
runtime_unavailable
```

Telegram account management UI now exposes a QR login panel inside the
inspector `About` tab. It can call `start`, `status`, `password` and `cancel`
routes, render the current QR/2FA state, and apply suggested account metadata
back into the local account setup form before the user saves the QR-authorized
account record.
