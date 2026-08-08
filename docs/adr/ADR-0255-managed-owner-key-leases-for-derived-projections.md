# ADR-0255: Managed owner-key leases for derived projections

Статус: Принято
Дата: 2026-07-23
Состояние реализации: protocol, descriptor validation, Vault ensure/resolve,
Kernel authorization and `ManagedOwnerDerivedKeyClientV1` реализованы в
existing `makosh-managed-vault-client` platform package. Descriptor/policy
negative conformance и Communications capability admission ещё не завершены.
Existing provider credential and session-store-key routes are not valid
substitutes for an owner-local derived projection key.

Зависит от:

- [ADR-0215: capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221: descriptor and lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0223: Vault leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0254: Communications derived search](ADR-0254-communications-derived-search-index-and-private-content-boundary.md).

## Context

Managed provider credentials are declared against a provider configuration
instance. Session-store keys are integration-owned wrapping authority. Neither
represents a stable key for an owner-local rebuildable projection such as the
Communications keyed-token search index. Reusing either would make a domain
pretend to be an integration or would couple unrelated key lifecycle.

The key must not be process-local, stored plaintext in PostgreSQL, carried by
Kernel, emitted as an event or derived from a provider credential.

## Decision

Vault gains an explicit `owner_derived_key` secret class and an exact
`issue_owner_key` managed-control operation. It is a generic platform contract,
not a Communications package or Gateway feature.

Each key is identified by this immutable tuple:

```text
logical owner ID
capability ID
purpose ID
key schema revision
```

The module descriptor declares that tuple, its bounded lease TTL and the
`OwnerDerivedProjectionKey` target scope. Kernel admission verifies all fields
against the approved registration and effective GrantSet before it routes an
HPKE-encrypted lease request to Vault. Configuration instance ID is absent from
this operation.

Vault atomically creates a 32-byte random key for a previously unseen tuple and
returns it only through a one-use, runtime-bound encrypted lease. A subsequent
request for the same tuple and schema revision resolves the same key. A changed
schema revision creates a distinct key; revocation, runtime restart, grant epoch
change, owner suspension, Vault lock or explicit key retirement invalidates all
outstanding leases.

The operation follows the existing durable Vault model:

```text
owner runtime
  -> inherited managed control FD: issue owner-key lease request
  -> Kernel: exact descriptor/grant/fence authorization
  -> ciphertext-only Vault route
  -> one-use lease ID delivery
  -> ciphertext-only resolve route
  -> owner runtime receives zeroized key bytes
```

Kernel never receives the key or stores the tuple as business state. The Vault
record is encrypted and owner-scoped. Telemetry records only sanitized operation
class and result.

## Contract boundaries

- `ManagedOwnerDerivedKeyClientV1` lives in the existing
  `makosh-managed-vault-client` platform package with no provider or domain
  dependency. It reuses common HPKE frame/binding transport, not copied crypto.
- Provider credential client remains provider-configuration scoped and cannot
  invoke `issue_owner_key`.
- A projection owner may use a key only in its runtime after an approved
  capability grants the exact declared purpose.
- The key is usable only for deterministic keyed derivation. It is not a
  generic encryption key, client secret, signing key or Blob key.
- A caller sends no key material to Vault. Vault generates the initial value;
  it is never included in a control request, descriptor, event, log or error.

## Required implementation

1. Add exact protocol enums and target scope with fail-closed validation.
2. Add managed control request/response and Kernel handler separate from
   provider credentials.
3. Add Vault transport operation with atomic ensure/resolve semantics and
   encrypted owner-scoped persistence.
4. Extend the existing `makosh-managed-vault-client` package with the
   owner-derived-key client using shared transport binding.
5. Add descriptor/policy evidence and conformance tests for wrong owner,
   purpose, capability, revision, stale generation and revoked grant.
6. Only then admit the Communications `search.index.v1` capability and use the
   client in its runtime.

## Consequences

Derived indexes become safely rebuildable without moving private content or
secret authority into a domain database. The capability is intentionally
narrower than a generic key-management API and does not reopen direct Vault
access for modules.
