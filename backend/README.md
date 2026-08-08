# Hermes backend clean room

The current exact production policy slice is
`attachment_security_engine_v1`. It contains the admitted platform/Core
packages, the six-package Communications domain owner, and the six-package
Attachment Security engine owner. Attachment Security is neither a domain nor
an integration: it receives typed durable observations, owns its PostgreSQL
join/jobs/outbox, uses Kernel-issued Storage/Blob/Event capabilities, and emits
only the typed verdict fact consumed by Communications. Provider integrations
and workflows remain outside production inventory. With a trustworthy Control
Store Kernel reaches `module_control_plane`; without one it fails closed to
`recovery_only`, and no separate Kernel `ready` state is claimed.

The previous implementation is available at
`references/backend-legacy/`, but it is not a dependency and is not an
architectural template.

## Entry conditions for new code

The following questions remain mandatory for later product slices, while
ADR-0225 closes them only for the recovery-only Kernel slice:

1. the product capabilities that are actually supported;
2. canonical evidence and event invariants that must remain stable;
3. bounded-context ownership and dependency direction;
4. the frontend-to-backend contract and cutover rule;
5. runtime lifecycle, shutdown and provider isolation;
6. schema ownership and the fresh-database bootstrap;
7. secret ownership, Vault recovery and scoped credential lease boundaries;
8. test boundaries that do not compile production composition.

The active foundation decisions are:

- [module ownership and runtime isolation](../docs/adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [core/module IPC and NATS communication](../docs/adr/ADR-0201-core-module-communication-and-nats.md);
- [PostgreSQL ownership, roles/grants and PgBouncer](../docs/adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md).
- [managed infrastructure supervision and recovery](../docs/adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md).
- [bundled integration plugins and the provider-neutral context boundary](../docs/adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md).
- [Core Gateway and desktop/Android client transport](../docs/adr/ADR-0205-core-gateway-and-client-transport.md).
- [Kernel constitution and boot/recovery state machine](../docs/adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md).
- [canonical business domain registry](../docs/adr/ADR-0207-canonical-business-domain-registry.md).
- [domain development allowlist and projection freeze](../docs/adr/ADR-0208-domain-development-allowlist-and-projection-freeze.md).
- [Kernel Event Hub](../docs/adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md).
- [Telemetry Hub and local diagnostics](../docs/adr/ADR-0210-telemetry-hub-and-local-diagnostics.md).
- [backend workspace and source layout](../docs/adr/ADR-0211-backend-workspace-and-source-layout.md).
- [Cargo package topology and compile isolation](../docs/adr/ADR-0212-crate-topology-and-compile-isolation.md).
- [code ownership and module autonomy](../docs/adr/ADR-0213-code-ownership-and-module-autonomy.md).
- [durable Job Platform, Scheduler and runtime reconfiguration](../docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md).
- [open module registration and capability grants](../docs/adr/ADR-0215-open-module-registration-and-capability-grants.md).
- [private SQLite Kernel Control Store](../docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md).
- [zero external dependency Kernel bootstrap](../docs/adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md).
- [owner/device identity, enrollment and offline recovery](../docs/adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md).
- [managed module integrity, distribution manifest and explicit updates](../docs/adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md).
- [canonical durable envelope and contract evolution](../docs/adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md).
- [ModuleDescriptorV1 and capability-level lifecycle](../docs/adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md).
- [Kernel Settings Registry and supervised reconfiguration](../docs/adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md).
- [encrypted SQLite Vault and scoped credential leases](../docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md).
- [Storage Control Plane, owner-scoped PostgreSQL and migration lifecycle](../docs/adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
- [first production recovery-only Kernel slice and phase gates](../docs/adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).
- [AI context acquisition through use-case workflows](../docs/adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md).
- [deployment profiles and server bootstrap pairing](../docs/adr/ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).
- [full-platform development simulation profile](../docs/adr/ADR-0228-development-simulation-profile.md).
- [platform Clock contract and deterministic conformance](../docs/adr/ADR-0229-platform-clock-contract-and-deterministic-conformance.md).

They settle the runtime, communication, storage, infrastructure lifecycle and
integration-plugin/client boundary, plus the closed responsibility set and
boot/recovery states of Kernel. Provider-specific operational screens remain
first-class desktop/Android experiences, while context domains consume only
neutral evidence contracts. Canonical domains, the development allowlist and
the shared durable envelope are fixed. The current implementation inventory
contains six Kernel packages and five Vault packages
are authorized, and no owner module is part of the current slice. Owner-specific
product, evidence and provider contracts still require explicit decisions
before their phase gate is opened.

The compile-isolation policy is already executable: Kernel and Gateway cannot
depend on owner modules, modules cannot depend on Kernel, integrations can use
only the exact Communications ingress and attachment contract units, and
aggregate backend/common/provider packages are forbidden. The
current-inventory guard is intentionally separate from the
reusable dependency validator: the former accepts only the exact ADR-0225
package set in the real workspace, while the latter can still prove future
owner graphs without authorizing those packages now.

ADR-0213 additionally requires every production module to prove independent
build, test, lifecycle and failure behavior. Its Cargo/storage/test-layout
rules are already partly executable; code-shape and process-lifecycle evidence
must be added with the first production slice where they apply.

ADR-0214 applies one Job Platform contract to every integration, domain, AI,
workflow and platform owner. Scheduler owns timing and reconciliation;
executable handlers and durable execution state stay owner-local. The current
Scheduler packages are platform foundation only: no owner handler, NATS
dispatch or managed Scheduler runtime exists yet.

ADR-0215 defines open local registration with zero rights before explicit
approval. Runtime rights are a typed intersection of requested, approved and
hard-policy grants. Managed and external module lifecycles are distinct.

ADR-0216 selects a private kernel-owned SQLite Control Store for registrations,
grant epochs and desired infrastructure state. A single `ControlStoreHandle`
actor owns the only online SQLite connection behind a 64-request queue, with
2-second normal and 30-second maintenance deadlines. It exposes narrow
health/recovery, owner identity, module registry, settings registry and runtime
trust ports. Registration and its effective grant set are read as one atomic
`ModuleGrantSnapshot`, so authorization never combines different actor
snapshots. The store contains no business data or secrets and is available
before PostgreSQL, PgBouncer, NATS and Vault.

ADR-0217 removes a mandatory bootstrap configuration file. Kernel derives its
private data directory from the OS standard or one explicit `--data-dir`.
Untrusted SQLite blocks managed infrastructure and the business data plane but
keeps online recovery read-only. ADR-0218 defines per-device ES256 identities,
platform signing and inherited-FD first enrollment. Destructive restore/reset
is offline-only under an exclusive instance lock.

ADR-0219 keeps external registration open and unsigned, while every Kernel
managed launch must verify exact executable bytes. Bundled entries come from a
signed distribution manifest; a promoted external executable requires an
owner-pinned digest. Kernel never downloads code and never rolls back
automatically.

ADR-0220 defines the exact `hermes-events-protocol` package for binary
`DurableEnvelopeV1`. It keeps owner payload opaque, binds it to exact contract
revision/schema SHA-256, preserves outbox bytes through NATS publication and
separates durable Ack, broker ACK, terminal result, technical DLQ and client
SSE semantics. The protocol now includes a generic owner-local outbox relay
port: owner persistence marks an entry published only after a transport ACK.
Test-only delivery scaffolds exercised that port against PostgreSQL in a separate
schema per designable owner and relayed exact bytes through an authenticated
JetStream test runtime before the first owner admission, but contain no owner
package, domain table, handler, migration or public contract. Their one narrow SQL allowance is
`hermes-events-jetstream-testkit:dev:sqlx`; the Cargo guard rejects this client
for production packages and all other test packages. The NATS platform gate is
open. The later `first_owner_v1` gate admitted the Communications owner, and the
current `attachment_security_engine_v1` slice also admits the separate
Attachment Security Engine.

ADR-0221 defines the exact `hermes-runtime-protocol` package for
`ModuleDescriptorV1`. Distribution manifest, descriptor, GrantSet and
RuntimeState have separate authority; capabilities are approved, resolved and
degraded independently. The package contains wire types only; the current
private module control plane performs registration, approval and capability
routing, while owner modules and public data-plane activation remain closed.

ADR-0222 makes Settings Registry an exclusive Kernel component. Module owners
declare typed schemas, while Kernel persists desired/effective revisions in the
private Control Store and supervises hot apply or restart. Settings never carry
secrets, business/runtime state, cursors or Scheduler records. No production
settings schema, API or runtime implementation exists.

ADR-0223 makes Vault a separate verified managed process. Kernel supervises it,
computes grants and routes only HPKE ciphertext; it never receives credential
plaintext or Vault keys. SQLCipher plus record-level AEAD protects bounded
credential material, while process-bound leases are fenced by runtime
generation and grant epoch. Large or high-churn provider session stores remain
integration-owned. The canonical operational summary is
[Vault and credential leases](../docs/architecture/vault-and-credential-leases.md).
The exact five-package `vault_v1` inventory implements the
file-backed key provider, SQLCipher storage through a bounded single-writer
actor, an explicit encrypted recovery key slot and the isolated runtime
boundary, fenced online lease/runtime delivery and classified component
backup/restore. This is not whole-instance backup or a business data plane.

ADR-0224 makes Storage Control a separate managed control-plane process beside
PostgreSQL and PgBouncer. Modules send business SQL directly through PgBouncer;
Storage Control owns bootstrap, roles/grants/budgets, immutable migration bundle
admission and readiness, but never proxies business queries. Database
credentials are scoped Vault leases. The target PgBouncer-only runtime path is
not a proven same-UID process boundary until OS socket/network isolation passes
conformance tests. The canonical summary is
[Storage Control Plane](../docs/architecture/storage-control-plane.md). The
six-package foundation, `StorageBundleV1` contract, fail-closed additive DDL
admission, generation-derived pool aliases, credential-free PgBouncer command
adapter and PostgreSQL schema/role adapters are implemented. The SQL-free
revoker retains `Revoking` until Vault, PgBouncer and PostgreSQL report success.
The Storage Vault adapter creates a fenced encrypted `RevokeAudience` request
and, for every exact Storage binding, resolves or creates one opaque
`PlatformCredential` under that binding's runtime-principal scope and
credential-lease revision. Storage resolves the generated token only inside
its process and uses server-side PostgreSQL quoting to set the matching role
password; neither Control Store, environment nor Kernel receives it.
The trusted managed
Storage-to-Kernel inherited-FD client is typed and descriptor-bound; Kernel
fences its registration, caller runtime generation, grant epoch and active
Vault generation before signing and relaying it. Before a managed Vault start
returns, Kernel obtains its typed inherited-channel `ready` status and requires
the persisted generation plus a 32-byte ephemeral HPKE public key. Storage launch composition is
limited to private owner-authorized bind/start of an exact signed artifact.
Owner control also persists a non-secret, monotonic Storage topology (profile,
storage generation, identities, exact PostgreSQL/PgBouncer digests and validated
host:port endpoints). At launch Kernel combines it with the current non-secret
Vault instance, runtime generation and ephemeral HPKE public key in a private,
short-lived child configuration and accepts only an exact `reconciling` runtime
status whose Vault runtime generation still equals the current managed Vault
status. Desired topology and Control Store never persist credentials, Vault
private keys or runtime attestation.
After Storage finishes Vault credential bootstrap and platform checks, it emits
an exact `ManagedRuntimeReadyRequestV1`; Kernel validates the registration,
runtime generation and grant epoch before allowing normal status/control relay.
This prevents a regular request from being interleaved with a bootstrap Vault
route on the inherited FD.
Before returning every status, the child performs bounded credential-free TCP
preflight of both staged endpoints: failure is the sanitized
`failed/storage_endpoint_unavailable` state, while success remains only
`reconciling`. This proves reachability, not endpoint identity or credential
delivery.
Storage release admission pins one exact signed platform artifact,
descriptor digest and binding revision. The
PostgreSQL adapter rejects a mismatched binding. The PostgreSQL integration
suite verifies live endpoint preflight, persisted owner/role uniqueness, binding-fence persistence,
owner-bound DDL, isolated `search_path`, owner-only DML grants and the
PostgreSQL-only `NOLOGIN`/backend termination phase. PgBouncer now has a
credential-accepting simple-query admin adapter. During a managed Storage child
startup, its descriptor-bound inherited FD performs an encrypted Vault route
that resolves a pre-existing platform credential. The Vault initializer can
import the two initial service-scoped credentials from an explicit private file
directory (`pgbouncer-admin-password` and `postgres-admin-password`); the
files are validated as regular owner-private files and their contents enter
only encrypted Vault records. An absent credential can still create a new
opaque Vault token, but it cannot authenticate an already provisioned
PostgreSQL or PgBouncer deployment and therefore does not establish readiness.
Kernel sees only ciphertext, and Storage passes resolved credentials only
inside its own process. Owner control now durably reserves an exact
binding as `revoking` before it asks the live Storage child to fence it; the
child performs Vault invalidation followed by PgBouncer `PAUSE`/`DISABLE`/`KILL`
and PostgreSQL role/session fencing over the same authenticated inherited
channel. A failed child response leaves the durable reservation in place and
stops the child, so a restart cannot restage the old binding. The authenticated
The in-process composition exercises Storage credential bootstrap, the exact
Kernel route fence/signature and a live Vault service over inherited Unix
channels. An opt-in disposable Docker runner creates a temporary signed macOS
release bundle for the built Vault and Storage binaries, admits both with the
production release-binding path, and starts both through the production Kernel
launch path. It imports two service-scoped file credentials into Vault,
confirms fenced `reconciling` status, then restarts Storage and verifies the
next generation against the signed artifact again. This is signed-release
execution conformance in a disposable test bundle; it is not Developer ID
signing, notarization or a production release attestation.
Owner-private control can now issue a durable non-secret binding for an exact
managed module launch only after matching its current registration grant,
runtime generation and Storage topology. Each replacement is revisioned and
must advance role/credential lease fences; launch revalidates every staged
binding before the child receives its private configuration. The same owner
route admits exact canonical `StorageBundleV1` bytes, records their SHA-256 and
requires the durable binding to name that immutable revision/digest. Runtime
reconciles the bound roles, applies only that exact bundle, and publishes the
PgBouncer alias only after both phases succeed. The authenticated Docker
contour proves this role-plus-migration ordering with temporary runner secrets,
and separately proves the Storage child’s encrypted `RevokeAudience` route,
PgBouncer fence and PostgreSQL role/session fence for one exact staged binding.
External-runtime issuance now also requires an owner-authorized request plus
the exact current attestation, runtime generation and grant epoch before it can
write a non-secret binding. An attested external session can retrieve only its
current canonical `StorageBindingV1`, PgBouncer endpoint and current Vault
public context; the database credential still requires a separately fenced,
HPKE-encrypted Vault route. `make -C backend test-storage-external-process`
now proves live credential delivery and rotation through owner-control IPC, a
temporary signed Kernel bundle, a real managed Vault child and a distinct
proof-backed external runtime process. The former binding receives
`runtime_session_stale`; its successor receives a different credential, while
the process records only SHA-256 assertions, never credential plaintext. The
authenticated Docker contour verifies owner-local migrations, roles and
Vault-delivered credentials against real PostgreSQL/PgBouncer. Together these
open `storage_control_v1`; the documented same-UID direct-endpoint limitation
is not misrepresented as physical sandbox isolation.

ADR-0225 establishes the six-package `kernel_recovery_only_v1` baseline:
`hermes-events-protocol`,
`hermes-runtime-protocol`, `hermes-gateway-protocol`, the Control Store port and
SQLite adapter, and `hermes-kernel`. Later accepted foundation work opens
private module-control/managed-launch trust and adds the five Vault packages,
but it still has no NATS or business data plane and cannot reach `ready`.
`clock_v1` adds only the standalone Clock contract. `blob_v1` is open as a platform
gate: it provides an opaque-reference contract, encrypted atomic owner-fenced filesystem
storage, and descriptor-declared quota requests retained in the Control Store and exposed
only through approved grants. Its key lease accepts only a scoped Vault response and the
Blob runtime has a ciphertext-only Vault route adapter. The distinct `hermes-blob-service`
is a verified managed child with one-shot private configuration and generation-bound status
attestation. It owns a private 0600 direct data socket rather than sending content through
Kernel; the socket accepts only a short-lived, one-use Kernel-signed session grant and
resolves content keys over the inherited ciphertext-only Vault route. At that Blob
foundation gate no first owner existed, so it intentionally introduced no generic owner/module issuer or
domain content API. The current Communications owner and Attachment Security
Engine consume only their exact typed Blob contracts and custody grants; Blob
still exposes no generic content API.
The encrypted store rejects references at their own expiry before read/write and has a
current-fence/key-authorized atomic delete. Its private technical ledger now durably
reserves aggregate bytes per approved owner/capability quota before writing, records
`pending_write` / `active` / fenced `delete_reserved` states, removes an uncommitted
ciphertext together with its pending reservation after a crash, cleans only a private
staged ledger record, and finalizes deletion only after the owner-marked grace period.
It deliberately stores no owner metadata and exposes no content route.
The runtime now performs a technical scheduled deletion pass only for due
owner-marked reservations and only with a freshly resolved current key lease;
revoked or unavailable leases defer deletion without touching ciphertext. A
targeted conformance builds the real Blob and file-backed Vault binaries, admits
both through signed Kernel bindings, verifies their generation-bound status, and
executes bounded encrypted write/read over the live ciphertext-only Vault route
while rejecting replayed and stale-runtime grants. The fixture-only Vault-status
test remains as a narrow protocol negative-path check. Blob technical retention,
revoke fencing and backup classification are implemented; owner metadata and
whole-instance restore remain separate future gates. The NATS foundation
adds the separate `hermes-events-jetstream` adapter: an Event Hub-only topology
connection, bounded exact subjects and streams, and a runtime-only exact-byte
publisher with `Nats-Msg-Id` deduplication. Its Docker conformance uses separate
admin/runtime ACL identities and proves that an undeclared subject is rejected.
The adapter now implements the owner-neutral relay port, so an owner adapter can
atomically mark its own outbox only after exact-byte broker acknowledgement.
The test-only delivery scaffolds reserve separate PostgreSQL namespaces for
`communications`, `contacts`, `organizations`, `tasks`, `calendar`,
`documents` and `ai`; their disposable Docker contour proves exact outbox bytes
and inbox duplicate/hash-conflict handling. They contain no domain behavior or
production migrations, so they are conformance infrastructure rather than a
first owner implementation.
`EventRouteRequestV1` is validated and atomically retained with its pending
registration; Kernel resolves canonical catalog contracts only after capability approval at
the current grant epoch and rejects revision/schema conflicts before broker
reconciliation. It derives a deterministic broker-neutral topology plan with only
declared exact subjects, streams, publish permits and consumer identities. Consumer
routes now explicitly declare required/optional status, bounded `max_deliver` and
`ack_wait_millis`; a legacy route without that policy cannot become a consumer after
the Control Store migration. Retention remains a later contract field. The adapter checks a local exact-subject permit
against runtime generation and grant epoch before publish. `nats_data_plane_v1`
is open as a platform gate. Its current Vault lease adapter does create/resolve for a
per-runtime broker credential only through HPKE with exact runtime/grant fences;
it has no local-secret fallback, and can revoke the active Vault audience lease.
A kernel-owned Event Hub credential scope resolves only a pre-seeded
`nats-event-hub-password` file credential through the same ciphertext route.
Its lease audience is the current verified Events authority registration/runtime,
so the Kernel rejects a route from any other managed child; its identity is
redacted. The adapter also has a
short-lived non-bearer runtime JWT/NKey issuer with only exact catalog subjects
plus its reply inbox. `make -C backend test-events-jwt-integration` creates a
throwaway Docker Operator/Account resolver contour, verifies the broker-side
proof and allowlist, and proves rejection of an unknown signing key. It does
not place an account signer in Kernel. The same throwaway contour revokes an
active runtime NKey, then has the managed Events authority resolve its fenced
System Account credential through Vault and publish the resulting Account JWT
through the full resolver; it proves that the broker disconnects the active
runtime. The isolated
`hermes-events-authority` foundation validates an account signer before its
first enrollment, writes it only through an authority-fenced encrypted Vault
route, and resolves it only for one runtime-JWT issuance. It has no Kernel
dependency or local-secret fallback. Kernel now persists only the public NATS
account key and monotonic signer credential revision in an owner-authorized
Control Store record, then starts the authority as a verified managed child
only after a current Vault status. Configuration changes stop that child; the
signer seed never appears in Control Store or inherited arguments. The
resolver-backed Docker contour now proves the whole managed
Vault → Authority → Kernel → HPKE runtime delivery path and broker acceptance
of the resulting JWT, including broker-side rejection of an undeclared subject.
`hermes-events-jetstream` can also publish an already-signed Account JWT through
the full resolver with scoped System Account `.creds`: it validates the exact
Account-NKey binding and uses only the matching claims-update subject. The
broker resolver remains the cryptographic verifier; this adapter neither holds
nor creates an operator signature. The managed authority now exposes that
mutation only over its inherited Kernel channel: an owner-private command
relays an already-signed bounded Account JWT, the authority validates its
Account-NKey binding, resolves a fenced System Account credential through
Vault, and sends the exact resolver update. Docker conformance proves this
full path, including an Account-claim revocation that disconnects an active
runtime. `nats_data_plane_v1` is therefore open as a platform gate; a
production owner-local PostgreSQL outbox/inbox transaction remains part of the
separate `first_owner_v1` decision.
`make -C backend test-events-authority-integration` proves
authority-to-broker reconciliation in an ephemeral authenticated NATS JetStream
contour: the authority receives only two encrypted test Vault-route responses
for a fenced Event Hub lease and a password, then creates the declared stream
and consumer. It is not evidence of a real Vault process or a full Kernel
composition. `make -C backend test-events-managed-authority-integration`
proves that missing contour with a real file-initialized Vault runtime, signed
managed Vault/authority bindings, an authority-fenced Event Hub credential
lease, Kernel topology relay and independent broker verification. It does not
create owner data or replace durable owner outbox/inbox conformance.
`hermes-events-authority-runtime` is now the supervised-child foundation: it
authenticates its inherited descriptor, verifies the current signer through a
fenced Vault lease before it sends `ready`, and exposes a sanitized status plus
a private credential-issuance operation afterward. The operation accepts only
public runtime fences, exact sorted catalog subjects and a one-time X25519
recipient key. It resolves the signer through Vault, signs one short-lived JWT,
then HPKE-seals JWT and user NKey to that recipient with the owner,
registration, runtime generation, grant epoch, credential revision and request
ID as associated data. The inherited Kernel channel therefore carries only the
public request and opaque ciphertext. A managed runtime can now make its
typed private request only after descriptor admission; Kernel dispatches it to
one handler that resolves current approved Control Store state for every call,
validates its runtime instance/generation/grant fence, derives exact sorted
subjects from the Event Hub topology, and relays only the opaque authority
delivery back. Default Kernel startup configures this handler; owner-private
control can start the separately release-bound authority child. The resolver
update has live full-resolver conformance through that child, including
revocation-driven runtime disconnect. This opened `nats_data_plane_v1` as a
platform gate; the later `first_owner_v1` slice implemented the durable
Communications owner delivery path.

`hermes-events-protocol` now fixes the owner-local exact-byte `OutboxRecordV1`
and SHA-256-based `InboxRecordV1` duplicate/hash-conflict contract; it does not
introduce a shared event database. PostgreSQL outage/replay conformance remains
available through owner-neutral test scaffolds. Communications now owns its
business mutation and durable state under the admitted owner inventory; the
platform scaffolds still own no business tables or mutations.

ADR-0226 keeps AI from becoming a database superuser or cross-domain
orchestrator. Cross-owner context is assembled by a typed use-case workflow
through explicit public owner contracts and passed in a distinct generated
use-case request with common `AiContextReceiptV1` metadata. A global fragment
union, opaque payload bytes, `Any`, generic maps, direct owner reads and durable
Context projections remain forbidden.

Legacy evidence reported Mail, Telegram and Zulip as previously working, but no
provider is considered implemented in the clean-room backend until new
executable evidence proves it.

## Current recovery runtime

`hermes-kernel serve` bootstraps an owner-private data directory, anchors its
SQLite Control Store to an installation ID and holds one single-instance lock.
With a trustworthy store, one Kernel process owns four `0600` private Unix
sockets: recovery, owner control, module registration and external runtime
sessions. With an unavailable or untrusted store it exposes only recovery.
The recovery socket accepts typed `status`, `validate`, `export` and `shutdown`;
restore and reset remain explicit offline CLI operations. SIGTERM/SIGINT and
recovery shutdown close every active private socket before process exit. After
an unclean exit, Kernel removes a stale control socket only when it is a Unix
socket owned by the current user; a symlink or regular file fails closed. Run
the lifecycle coverage with:

```sh
make -C backend test-kernel-recovery
```

Browser Gateway остаётся выключенным по умолчанию. Для local HTTPS foundation
`serve` принимает только полный набор non-secret operator inputs:
`--browser-gateway-listen-address` (строго loopback),
`--browser-gateway-origin` (exact HTTPS), `--browser-gateway-rp-id`,
`--browser-gateway-certificate-der` (X.509 DER) и
`--browser-gateway-private-key-der` (DER private key). Неполный набор
отвергается до создания Control Store; недоверенный Control Store не запускает
Gateway. Этот listener пока даёт только technical/browser-session foundation:
не является remote/public admission и не поставляет browser bundle, ConnectRPC
или client-safe realtime owner. Уже доступен private owner-control start для
одноразового browser pairing: он требует действующую owner-device session и
выдаёт только opaque pairing ID с коротким TTL. Этот ID открывает на exact
HTTPS origin только WebAuthn registration options и finish route; неверный
`Origin` не расходует pairing, а credential/session identifier не возвращается
browser caller. Это ещё не browser application delivery и не `client_gateway_v1`.

Для UI-разработки доступен отдельный durable developer setting. Он выключен по
умолчанию и меняется только локально при остановленном Kernel:

```sh
cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data developer-mode status
cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data developer-mode enable
cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data developer-mode disable
```

В enabled state browser-Gateway input обязан указывать один literal
private-LAN IP:port одновременно как listen address и HTTP origin; RP ID равен
IP, а TLS inputs и `--browser-gateway-paired-remote` запрещены. В этом режиме
WebAuthn/cookie authentication bypassed, pairing/auth routes отсутствуют,
requests с public hostname, wildcard bind или proxy/forwarding headers
отвергаются, а console получает verbose sanitized route/status log без body,
cookie и dynamic IDs. Public `paired_remote` profile это не ослабляет. Полный
security contract зафиксирован в ADR-0235.

Offline restore/reset reserves monotonic generation, identity and grant fences
in `.hermes-recovery-fence-v1` before atomically replacing SQLite. Restored
registrations are suspended and runtime attestations, sessions and managed
launch records are removed. A fence/store mismatch therefore boots
recovery-only instead of accepting rollback state.

## Linux target compilation

`make -C backend ci TARGET=x86_64-unknown-linux-gnu` runs its shared policy and
runtime gates on the invoking host, then compiles the Linux target in the
`linux/amd64` `rust:1.97.0-bookworm` container. The source bind mount is
read-only and Cargo writes build output only to the container's temporary
directory, so macOS does not need `x86_64-linux-gnu-gcc`. This is compile
conformance, not evidence for a real Linux release host, TPM/FIDO hardware or
release preflight.

## File-backed initial owner bootstrap

The first signer implementation is a local `FileDeviceSigner`: an owner-private
`0600` key file behind the replaceable `DeviceSigner` boundary. It is exportable
by design: protecting this file is equivalent to protecting the owner-device
signing authority. Generate it explicitly, then enroll the first owner/device:

```sh
cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data device-key-generate

cargo +1.97.0 run --manifest-path backend/Cargo.toml --package hermes-kernel -- \
  --data-dir /absolute/hermes/data \
  initial-owner-enroll --owner-id owner-1 --device-id desktop-1
```

The generator never prints private material and never overwrites an existing
key. Enrollment verifies its ES256 proof in Kernel and rejects a second
first-owner claim. This is the current file-adapter baseline, not an inherited
FD ceremony or a hardware-protected signer.

The corresponding local PostgreSQL and NATS JetStream contour is documented in
[development platform](development/README.md). It is separate from both legacy
`docker/` assets and the signed/digest-pinned Linux release topology.

For the supported browser development contour, run from the repository root:

```sh
make dev
```

ADR-0300 keeps this as a development assembly unit: it starts the loopback-only
Compose dependencies, Kernel/Core Gateway and Vite, and waits for both direct
and same-origin readiness at `http://127.0.0.1:5173/`. The developer opens
that URL in an existing browser tab; automatic browser launching is opt-in via
`HERMES_DEV_NO_BROWSER=0`. A per-run
owner-private proxy proof stays outside browser JavaScript. The command does
not become a signed release assembly, create provider credentials, or silently
admit managed domain/integration runtimes.

For the two deployment profiles, the repository currently includes a macOS
Apple-Silicon sidecar packaging target and a Linux release-manifest preflight.
Linux Compose/systemd descriptors are generated only after each digest-pinned
OCI image passes Cosign verification; neither descriptor gives Kernel Docker
API authority. See [Linux release notes](release/linux/README.md). Hardware
identity, managed launch, Vault, Storage, NATS, Blob, Clock,
Scheduler, Gateway and whole-instance backup remain closed gates.

## Validation

Run the backend-owned architecture surface from the project root:

```sh
make -C backend architecture-check
make -C backend test-architecture
make -C backend validate
make -C backend test-telemetry
make -C backend test-storage-conformance
```

`test-storage-integration` starts the disposable development Compose contour,
discovers its live PostgreSQL endpoint, and checks bootstrap, readiness,
owner-bound DDL, isolated runtime role settings and owner-only DML grants. It
does not test PgBouncer or release credentials and is deliberately outside CI.

Production packages may live only below `backend/src/`; test-support packages
may live only below `backend/tests/support/`. Tests, fixtures and inline Rust
test modules are forbidden in the production tree by executable policy.
