# Макошь Vault

Status: documentation package aligned to the current repository structure.

Vault documentation mirrors `backend/src/vault`.

The vault layer handles local host-vault lifecycle, secret payload storage
boundaries, key material handling and recovery support. PostgreSQL stores
secret metadata and references only; new provider credential payloads must not
be stored in database tables.

## Current Code Areas

- `backend/src/vault/secrets.rs` - vault-backed secret payload operations.
- `backend/src/vault/lifecycle.rs` - vault initialization and lifecycle.
- `backend/src/vault/storage.rs` - local storage implementation.
- `backend/src/vault/manifest.rs` - vault manifest metadata.
- `backend/src/vault/recovery.rs` - recovery support.

## Documentation Rule

Vault docs must not include secrets, tokens, passwords, private keys or local
`.env` values. Provider account metadata belongs to domains/integrations;
secret payload handling belongs here.
