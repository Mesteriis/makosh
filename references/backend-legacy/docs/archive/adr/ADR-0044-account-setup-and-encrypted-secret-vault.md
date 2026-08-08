# ADR-0044 Account Setup and Encrypted Secret Vault

Status: Superseded by ADR-0076

This decision was superseded by ADR-0053 and then ADR-0076. ADR-0076 keeps encrypted local account setup but moves new secret payloads to a dedicated macOS host vault under `~/.makosh/vault`.

## Context

ADR-0042 keeps provider credential values outside PostgreSQL, and ADR-0043 adds read-only Gmail API and IMAP networking. The remaining gap was account setup: users needed a way to obtain Gmail OAuth tokens, refresh them, and store iCloud/raw IMAP passwords without writing secrets into provider account config or secret reference metadata.

## Decision

Add a local account setup boundary backed by an encrypted secret vault.

Rules:

- `MAKOSH_SECRET_VAULT_PATH` points to the local encrypted vault file.
- `MAKOSH_SECRET_VAULT_KEY` is the local vault master key and must not be logged, persisted in PostgreSQL or committed.
- The encrypted vault uses per-entry AES-256-GCM encryption with an Argon2id-derived key and authenticated `secret_ref` associated data.
- Gmail account setup uses OAuth authorization code with PKCE and `gmail.readonly` scope.
- Gmail token bundles are stored only in the encrypted vault and include access token, refresh token, token endpoint and OAuth client material required for refresh.
- Gmail access token refresh reads the encrypted token bundle, exchanges the refresh token and updates the encrypted vault.
- iCloud and raw IMAP setup store app-password/password values only in the encrypted vault.
- PostgreSQL stores only provider account metadata, secret reference metadata and account-to-secret bindings.
- The desktop account wizard calls local API endpoints protected by the local API token and actor header.

## Consequences

- Local development and desktop account setup can create usable Gmail, iCloud and raw IMAP provider accounts without plaintext secrets in PostgreSQL.
- Provider networking can obtain runtime access tokens through refresh instead of requiring manual token injection.
- The encrypted vault becomes part of local operational setup and must be backed up with its master key handled separately.
- A native OS keychain resolver can still be added later as another `SecretStoreKind`, but account setup is no longer blocked on it.
