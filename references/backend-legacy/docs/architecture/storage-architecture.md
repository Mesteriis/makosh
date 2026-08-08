# Storage Architecture

## Storage Goals

- durable local ownership
- reproducible projections
- source preservation
- efficient search and graph traversal
- clear backup and restore
- replaceable indexes

## Storage Components

| Component | Role |
| --- | --- |
| PostgreSQL | canonical relational data, event tables, graph tables, metadata |
| Host vault | secrets-only encrypted payload storage under `~/.makosh/vault` |
| Object storage | documents, attachments, OCR artifacts, extracted text |
| Tantivy | full text search indexes |
| Vector index | semantic retrieval indexes |
| Backup storage | encrypted local or self-hosted snapshots |

## PostgreSQL Responsibilities

- event envelope and payload storage
- normalized entities
- relationship objects
- task lifecycle
- provider account metadata
- secret reference metadata and account-to-secret bindings
- ingestion checkpoints
- projection offsets
- permissions and capability grants
- audit trails

Backend readiness verifies that the embedded SQLx migration ledger has the expected successful migration count and latest version before reporting the service ready.

PostgreSQL must not receive new provider credential ciphertext payloads. `encrypted_secret_vault_entries` is legacy/migration state after ADR-0076.

## Host Vault Responsibilities

- encrypted provider credential payloads
- encrypted local account keys, signing credentials and external service credentials
- recovery export material
- minimal non-secret manifest data needed to reconcile account secret bindings after PostgreSQL recreation

The host vault uses a dedicated SQLite `vault.db` under `~/.makosh/vault`. Release runtime stores the master key in macOS Keychain. Docker development mounts the host vault into the container and uses debug-only dev key storage.

## Object Storage Responsibilities

- immutable raw attachments
- imported document versions
- generated OCR text
- document previews
- extracted structured artifacts

Object keys must not encode secrets. Metadata that affects business logic belongs in PostgreSQL.

## Index Responsibilities

Indexes are derived. Tantivy and vector indexes may be rebuilt from canonical events, objects and relational state. Index corruption must be recoverable without losing memory.

## Backup Model

Backups must include:

- PostgreSQL dump or physical snapshot
- object storage content
- index rebuild metadata
- application configuration excluding secrets where possible
- host vault backup or recovery file/phrase handling plan

Restore must verify schema versions, projection offsets, index consistency and host-vault manifest reconciliation. PostgreSQL restore alone is not sufficient to recover secrets.
