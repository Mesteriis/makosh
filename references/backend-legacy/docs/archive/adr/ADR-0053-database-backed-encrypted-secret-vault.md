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
