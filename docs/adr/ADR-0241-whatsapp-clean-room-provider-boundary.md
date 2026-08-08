# ADR-0241: WhatsApp clean-room provider boundary

Status: accepted; supersedes the WhatsApp restriction in ADR-0212

## Decision

WhatsApp is an independent integration owner. Its API, lifecycle policy,
provider projections, persistence and runtime packages do not import Telegram,
Mail, a business domain, Gateway implementation or WebView implementation.

The WhatsApp runtime communicates with the external provider only through an
owner-local `WhatsAppProviderTransport` port. The concrete hidden WebView and
its browser/session state remain host-owned. Rust runtime code never evaluates
provider page scripts, stores cookies, or invents a provider-neutral business
model.

This decision explicitly opens the versioned `host_bridge_v1` seam that ADR-
0212 previously required before adding backend WhatsApp packages. It does not
move the WebView implementation into `backend/` and does not admit the legacy
unversioned HTTP relay as the final contract.

Provider commands are validated in `makosh-whatsapp-api`, operation lifecycle
is owned by `makosh-whatsapp-core`, projections and operation state are owned by
`makosh-whatsapp-persistence`, and orchestration is owned by
`makosh-whatsapp-runtime`. Provider observations enter through typed events and
are not promoted directly to durable business entities.

## Implementation state

The versioned API and host bridge contracts, metadata-only core policy,
owner-local durable observation/outbox persistence and managed
identity/storage bootstrap now exist. ADR-0276 replaces cloned V1
inherited-control readers with one correlation-owned
`ManagedControlChannelV2` and separates the private
`whatsapp.host_bridge.v1` operation/response oneofs from the public
`whatsapp.command.v1` and `whatsapp.query.v1` generated contracts. The Tauri
host and runtime bind the private route to the same exact descriptor digest;
accepted commands and their owner-local terminal status no longer pass through
an umbrella `whatsapp.client` DTO. These prerequisites do not open the
production phase gate. Canonical descriptor/settings/storage artifacts and the
separate unsigned `makosh-whatsapp-assembly` now exist; one admitted runtime is
bound by hidden configuration-scoped settings to one account, and its Storage
bundle contains only `makosh_data.whatsapp_*` tables. Exact assembly artifacts
are included in the signed distribution, and live managed conformance proves
owner-approved grants, signed Kernel launch, Storage admission through the
issued PgBouncer pool alias, private host-route binding, stale generation
rejection and revoke fencing without stopping Communications. A second live
contour proves public command acceptance, exact native host lease, owner-local
terminal result, metadata-only host observation, exact-byte outbox delivery,
Communications inbox causation, duplicate suppression and NATS outage replay.
Operational command receipts never become Communications evidence, and private
provider body/host-route material is absent from durable event bytes.
`whatsapp_integration_v1` is therefore open for the exact backend profile;
frontend cutover remains separate. No database URL environment variable,
runtime DDL or provider secret handoff is admitted.

The backend API/core/persistence/runtime/assembly packages remain independent
WhatsApp integration build units. "Host-owned" applies only to browser/WebView
execution and session state; it does not prohibit the integration's typed
runtime, owner-local durable queue or event outbox. Backend WhatsApp packages
must not depend on Tauri, Wry, WebKit or WebView runtimes.
