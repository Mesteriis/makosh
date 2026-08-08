# Clean-room development platform

This Compose contour is only for `development_full_platform_v1`. It supplies
loopback PostgreSQL, PgBouncer and NATS JetStream so clean-room platform packages can be
implemented and tested without release images, Cosign or Docker socket access
from Kernel.

It intentionally uses PostgreSQL `trust` authentication and unauthenticated
NATS. The named volumes, endpoints and data are development-only and must not
be migrated into a release deployment or populated with production credentials
or private user data.

PgBouncer uses the disposable PostgreSQL superuser only as `auth_user` to look
up dynamically created test runtime roles; it does not carry role passwords.
That exists so the live conformance test can exercise a generation-scoped role
through the transaction pool. It is not a production authentication design.

```sh
make -C backend docker
docker compose -f backend/development/compose.yaml ps
docker compose -f backend/development/compose.yaml down
```

Normally the loopback endpoints are
`postgres://makosh_development@127.0.0.1:35432/makosh_development`,
`postgres://makosh_development@127.0.0.1:36532/makosh_development` through
PgBouncer, and `nats://127.0.0.1:34222`. Some Docker Desktop/OrbStack configurations do not
publish fixed host ports even when Compose accepts them; use
`node backend/scripts/print-development-platform-endpoints.mjs` to print the live native-host
addresses in that case. These addresses are not a future production
configuration surface.

Run `node backend/scripts/smoke-development-platform.mjs` after startup. It executes
real direct and PgBouncer `SELECT 1` queries plus a NATS health request inside
the Compose network, so it also
works where host port forwarding is unavailable.

For the password-authenticated PgBouncer adapter there is a separate disposable
contour. Run it only through the target below; the runner generates a fresh
random secret in a private temporary file, mounts it as a Docker secret, checks
that a wrong admin password is rejected and that the file-backed credential can
run `SHOW VERSION`, then removes the project, volumes and temporary secret.
The password never appears in the Compose environment or repository.

```sh
node backend/scripts/test-authenticated-storage.mjs 1.97.0
```

This proves the real inherited Storage Runtime boot path through its
descriptor-bound Kernel channel: the test relay carries only encrypted Vault
frames, while the runtime resolves separate PostgreSQL and PgBouncer admin
credentials and reaches `reconciling` only after both live services accept
them. It also proves four concrete adapters: PgBouncer rejects a wrong
password; Storage runtime performs its bounded PgBouncer admin check; Storage
runtime uses a separate PostgreSQL admin secret to reconcile the fixed platform
schemas; and the runtime applies a validated binding through its private
database include, `RELOAD` and PgBouncer catalog verification. The two secrets
are generated independently and mounted only as Docker secrets. They are
deliberately not Vault-issued, so this is not end-to-end proof of a production
Vault deployment, owner-authorized binding issuance, or credential rotation.

The same disposable contour also drives a real revoke conformance: it publishes
a generation-scoped pool alias, then executes PgBouncer `PAUSE`/`DISABLE`/`KILL`
and PostgreSQL `NOLOGIN` fencing through the production adapters. The Vault
invalidation boundary is a typed successful test port in that one check because
the contour does not yet package a live Vault service.

`makosh-development-kernel-operator` is a separate development-only workspace
binary. It does not control Docker or run inside Kernel. For an environment
where the endpoints are reachable from the native host, check that boundary
explicitly with:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- probe \
  --postgres-address 127.0.0.1:35432 \
  --nats-address 127.0.0.1:34222
```

The runtime only performs bounded TCP probes; service protocol/readiness stays
covered by `development-platform-smoke` inside Compose until Storage Control
and Event Hub are implemented.

It also has a development-only remote-pairing simulation. It is not evidence
for `server_bootstrap_pairing_v1` or a production Kernel initial enrollment:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- pairing create \
  --state-dir /absolute/private/development-pairing --ttl-seconds 300

cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- pairing consume \
  --state-dir /absolute/private/development-pairing --token YOUR_256_BIT_HEX_TOKEN
```

After `create`, start a bounded TLS listener. It prints an endpoint and the
ephemeral certificate SHA-256 fingerprint; the client must compare that
fingerprint before it sends the bearer token or device proof:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- pairing listen \
  --state-dir /absolute/private/development-pairing \
  --listen-address 127.0.0.1:0 --idle-timeout-seconds 300
```

The listener exposes a one-request challenge and enrollment protocol over TLS:
`GET /v1/pairing-challenge` with `Authorization: Bearer TOKEN`, then
`POST /v1/initial-owner-enrollment` with the same bearer and owner/device,
public-key and raw-signature headers. The helper below creates or opens the
owner-private `0600` ES256 file and prints only the public key and signature:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- pairing proof \
  --key-dir /absolute/private/development-device-key \
  --challenge CHALLENGE_HEX --owner-id owner_1 --device-id device_1
```

The persisted state contains a SHA-256 token digest, expiry, terminal state and
the receipt digest; the owner-private receipt itself contains public proof
material, never the bearer token. A consumed pairing blocks any second initial
enrollment. The listener never accesses Kernel SQLite directly. To apply the
receipt in a simulation, the separate development operator checks its state
hash and ES256 proof before it writes the initial owner into Control Store.
Production Kernel has no development-profile switch:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- \
  --data-dir /absolute/private/kernel-data \
  initial-owner-import-pairing \
  --pairing-state-dir /absolute/private/development-pairing
```

This explicit development-only handoff is not the production listener contract
and cannot open `server_bootstrap_pairing_v1`. Never use this development state
directory, key or token with production data.

For an already approved development module, an explicit owner-pinned artifact
record can be created and rechecked without spawning a process:

```sh
cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- \
  --data-dir /absolute/private/kernel-data \
  module-owner-pin-artifact \
  --registration-id REGISTRATION_ID \
  --artifact /absolute/path/to/local-artifact

cargo +1.97.0 run --locked --manifest-path backend/Cargo.toml \
  --package makosh-development-kernel-operator -- \
  --data-dir /absolute/private/kernel-data \
  module-owner-pinned-preflight \
  --registration-id REGISTRATION_ID
```

The record is bound to the canonical path, SHA-256, size, observed file
identity and a file-signer ES256 proof. Changed bytes require a new explicit
revision. It is development evidence only: it does not spawn, supervise or
authorize a managed runtime and does not open `managed_launch_trust_v1`.
