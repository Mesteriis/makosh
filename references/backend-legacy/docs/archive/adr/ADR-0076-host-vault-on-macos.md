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
