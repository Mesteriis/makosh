# Zoom Gap Analysis

Status date: 2026-06-28.

The current repository has the Zoom provider foundation implemented. This
document records the remaining gap between the current checkout and a live,
end-to-end Zoom provider runtime.

## Current vs target

| Capability | Current state | Target state |
|---|---|---|
| Account setup | Fixture setup, blocked live metadata, initial OAuth/S2S authorization, explicit token refresh/renewal, token maintenance scan, scheduled token maintenance daemon and token rotation policy are implemented. | Operational provider worker enablement. |
| Runtime lifecycle | Metadata-level start/stop/remove implemented; authorized live accounts start as running and can participate in the recording-sync worker. | Broader live runtime state transitions beyond recording sync after auth. |
| Meeting ingestion | Runtime bridge accepts meeting observations and signed meeting webhooks as provider call evidence; `makosh-zoom-edge-proxy` provides public/edge forwarding and authorized accounts can reconcile managed app event subscriptions. | Broader provider worker coverage beyond webhook/event delivery setup. |
| Recording ingestion | Recording observation event, signed recording webhook normalization, webhook/provider-sync media download/import, and explicit per-import local retention/removal are implemented, gated by explicit privacy opt-in and local blob persistence. | Broader downstream media/document workflows. |
| Transcript ingestion | Runtime bridge stores explicit transcript text, imports already obtained VTT/SRT/plain transcript file text and auto-downloads transcript-like text files from signed `recording.completed` webhooks and authorized provider sync only after explicit privacy opt-in. | Full live provider worker coverage beyond current recording-driven transcript files. |
| Calendar matching | `zoom.meeting.observed` is matched to Calendar events through a downstream workflow and relation projection. | Meeting preparation context packs and broader Calendar-side downstream consumers. |
| Participant identity | `zoom.meeting.observed` participants can create conservative `attach_email_address` identity candidates for exact display-name matches and mirror them into the existing Review inbox flow. | Broader identity-trace and persona-resolution workflows beyond exact display-name candidate generation. |
| Radar | Zoom meeting/recording/transcript evidence is projected into Signal Hub `signal.raw.zoom.*` events and policy-derived `signal.accepted|muted|paused.zoom.*` detection events. | Richer Radar ranking/grouping and follow-up ergonomics beyond the current detection feed. |
| Frontend | Integration API/types/query modules, provider setup/status UI, OAuth/S2S authorization controls, token maintenance controls, local runtime-bridge lab controls, read-only observed call evidence inspection, provider-neutral Communications `calls`/`meetings` evidence views with participant snapshots and recording references, recording import audit inspection and account-scoped event audit inspection exist. | Richer Calendar/Communications downstream workflows beyond the current evidence views. |
| Security | Secret purposes, HostVault token storage/refresh/maintenance, scheduled token maintenance, token rotation policy, credential lifecycle audit events, owner-visible retention-policy settings, event/call metadata sanitization, protected webhook HMAC verification, public edge forwarding, recording import audit/remove surfaces, explicit expired-evidence cleanup, scheduled retention cleanup and account-scoped event audit view exist. | Broader media/document workflow promotion beyond current secure ingestion and cleanup boundaries. |

## Architectural gaps

### Live provider execution

The current implementation can register metadata, complete initial OAuth or
Server-to-Server token exchange, explicitly refresh/renew stored token bundles,
run scheduled maintenance scans over authorized accounts, reconcile managed
Zoom app event subscriptions, and run the authorized recording-sync worker
over recent cloud recordings. The live adapter still needs broader provider
API workers and should feed the same bridge methods rather than adding a
separate domain pathway.

### Public webhook ingress

The protected runtime bridge verifies account-scoped Zoom webhook signatures
before meeting/recording bridge ingestion. `makosh-zoom-edge-proxy` provides
the public/edge ingress path and preserves the raw body,
`x-zm-request-timestamp` and `x-zm-signature` when forwarding to the protected
bridge. Managed provider subscription reconciliation through Zoom APIs is now
implemented for authorized accounts.

### Recording and transcript download worker

The foundation now supports authorized provider-sync recording media imports and
webhook/provider-sync transcript-like text imports. The remaining gap is not the
basic worker itself but broader policy and downstream operational coverage:

```text
verified provider event
  -> secret-safe download worker
  -> local blob/document store
  -> media/document scan
  -> evidence event
  -> review/workflow promotion
```

Importing already obtained VTT, SRT and plain text transcript file content is
implemented through the runtime bridge. Signed `recording.completed` webhooks
best-effort download transcript-like text files when Zoom provides a
`download_url` plus `download_token`, but only after
`privacy.zoom_remote_transcript_download_enabled` is explicitly enabled.
Authorized recording sync now also best-effort downloads non-transcript
recording media files after
`privacy.zoom_remote_recording_download_enabled` is explicitly enabled, stores
them through the local communication blob store and records attachment-import
metadata plus heuristic scan status. Макошь now exposes recording import audit
plus explicit per-import remove controls for those blobs, the account event log
now records authorization completion plus token refresh success/skip/failure
activity, and owner-visible retention settings now stamp expiry intent into
recording-import metadata and transcript provenance. Макошь now also exposes an
explicit owner-triggered cleanup surface for expired recording imports and
expired transcript evidence, plus a local scheduled cleanup daemon that prunes
the same expired evidence automatically. Remaining work is richer downstream
workflow promotion beyond the current stamped policy, cleanup and audit path.

### Calendar matching

Meeting evidence should be matched to Calendar through workflow/query ports.
Zoom must not create or mutate calendar events directly.

### Participant identity resolution

Participant emails and display names now feed conservative
`attach_email_address` identity candidates when a Zoom participant exactly
matches an existing Persona display name. The remaining gap is broader
identity-trace coverage and resolution heuristics beyond that exact-match
candidate path.

### Signal detection

Zoom meeting, recording and transcript evidence now feeds downstream Signal Hub
detection through policy-aware `signal.raw.zoom.*` and derived
`signal.accepted|muted|paused.zoom.*` events. Remaining work is richer
attention-ranking, grouping and owner-facing follow-up ergonomics rather than
basic detection availability.

### Transcript intelligence

Transcript text can feed AI summaries, obligations, decisions and task
candidates. AI output must include source, confidence and evidence and must pass
through Radar/Review before domain mutation.

## Product gaps

- Richer meeting evidence workflows in Calendar and Communications beyond the current provider-neutral Calls/Meetings evidence detail view.
## Testing gaps

- Scheduled refresh daemon and degraded/reauthorization failure-path tests.
- Provider worker tests with mocked provider API responses.
- Frontend interaction tests for the runtime-bridge lab and provider-neutral
  downstream views beyond the current boundary/query coverage.
