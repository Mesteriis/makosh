# Security Model

## Security Goals

- protect personal communication and documents
- prevent accidental data exfiltration
- keep local-first trust boundaries clear
- make AI/tool actions auditable
- avoid hidden outbound behavior

## Trust Boundaries

| Boundary | Risk | Control |
| --- | --- | --- |
| Provider adapters | credential leakage, malformed data | secret store, strict parsing, scoped permissions |
| AI runtime | prompt injection, untrusted content | tool capability checks, source labeling, constrained actions |
| Plugin host | arbitrary file/network access | explicit capability manifest and runtime enforcement |
| Tauri bridge | UI-to-system escalation | narrow commands, input validation |
| Android device | stolen/compromised paired client | hardware-backed device key, scoped grants, revoke |
| Remote Gateway listener | LAN/Internet exposure, downgrade | disabled by default, authenticated TLS, HTTP/2 baseline, HTTP/3 conformance |
| Search and export | bulk data disclosure | confirmation, audit, export scopes |

## Authentication and Client Access

Active client policy находится в
[ADR-0205](../adr/ADR-0205-core-gateway-and-client-transport.md).

Архитектура различает:

- ephemeral desktop/Android client session;
- revocable Android device identity;
- provider credentials;
- module/runtime capabilities;
- agent tool permissions;
- export and backup permissions.

Desktop получает ephemeral session через Tauri bootstrap и подключается только
к loopback Core Gateway. Paired Android генерирует non-exportable device key,
проходит явное owner-approved pairing и получает scoped revocable identity.
Remote listener выключен по умолчанию; plaintext remote access запрещён.

HTTP/3 over QUIC допускается для paired Android после conformance проверки и
использует ту же server/device identity, что защищённый HTTP/2. 0-RTT запрещён.
Изменение IP/network path не заменяет authentication.

Legacy `MAKOSH_LOCAL_API_SECRET`, `X-Макошь-Secret`, actor-id headers и старые
auth ADR являются reference предыдущего backend и не определяют clean-room
boundary.

## Secrets

Secrets must never be hardcoded or committed. Provider tokens, passwords, app passwords, private keys and recovery material belong behind the secret resolver boundary.

ADR-0076 defines the current vault model. New secret payloads live in the dedicated host vault under `~/.makosh/vault`, backed by a local `vault.db` SQLite database. PostgreSQL stores only non-secret `secret_references`, provider account metadata and account-to-secret bindings.

Release runtime is macOS-only and uses Keychain for the master key. Docker/debug development may use `MAKOSH_DEV_MODE=true` with `MAKOSH_DEV_KEY_PATH`; release builds must reject dev storage. `MAKOSH_SECRET_VAULT_KEY` is legacy migration compatibility only and is not the normal runtime vault key.

Secret values must remain out of ordinary application tables, provider account config, event payloads, audit records, logs, tests and docs. Recovery phrases and recovery files are sensitive and must not be logged.

## AI Tool Safety

Agents may propose actions. Execution requires:

- declared tool capability
- user-visible intent
- permission check
- audit event
- rollback path where feasible

High-risk actions, such as sending messages, deleting data or changing external state, require explicit confirmation.

## Prompt Injection Defense

Imported messages and documents are untrusted input. The agent runtime must treat them as evidence, not instructions. Tools exposed to agents must be scoped, typed and permissioned.
