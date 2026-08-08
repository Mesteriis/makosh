# ADR-0242: WhatsApp versioned host bridge v1

Status: accepted; refined by ADR-0276

## Decision

The hidden Tauri WebView remains host-owned. Its only provider/runtime surface
is the private `whatsapp.host_bridge.v1` contract in
`makosh-whatsapp-api::host_bridge`.

The bridge carries one exact typed operation at a time: sanitized provider
observation metadata toward the runtime, or a bounded command claim toward the
host. A command result returns as a typed observation bound to operation and
host-claim identities. Cookies, local storage, IndexedDB, session material,
message bodies, media bytes and arbitrary JSON are forbidden. Host code does
not decide business state or invoke domain commands; it executes only a command
leased by the admitted WhatsApp runtime.

The operation and response use separate generated oneofs. Observation envelopes
are versioned by exact protocol major/revision and include account, provider
event identity and observed time. The old loopback HTTP relay is not a v1
contract and is not admitted as a compatibility surface.

## Implementation state

The typed API operation/response contract, exact route-binding handshake and
runtime-side durable metadata ingress exist. The former
`WhatsAppClientResponseV1` umbrella and provider-query decode probing were
removed. Kernel publishes an owner-private route descriptor only for the
lifetime of its admitted runtime; Tauri verifies
`whatsapp.host_bridge.v1`, its exact descriptor digest and route binding before
submission. The host executor emits only native-derived
`host_route_attached` and `webview_loaded` lifecycle observations through this
route. The remote WebView has no relay payload and cannot select an account,
event ID, timestamp, state, command, or observation content.

The API/runtime can encode and decode bounded command claims, but the host
executor intentionally has no provider-DOM relay, command polling or provider
execution loop yet. JSON fallback and fake provider-command acceptance are
forbidden. Live WebView smoke evidence, a metadata-only provider DOM extractor
and actual provider command/result execution remain migration work; no public
availability is claimed until those gates are present.
