# ADR-0104: Yandex Telemost Provider Runtime Boundary

Status: Proposed
Date: 2026-06-28

## Context

Макошь needs Yandex Telemost support for conference creation, conference links,
visible WebView joining, local audio capture and later transcription. Telemost is
an external provider, not a Макошь domain.

The integration also needs local desktop behavior that the backend provider API
cannot own: visible WebView opening, system/loopback audio capture and speaker
hint extraction from the WebView.

## Decision

Add Yandex Telemost as a provider runtime integration:

```text
backend/src/integrations/yandex_telemost
frontend/src/integrations/yandexTelemost
frontend/src-tauri/src/yandex_telemost_companion.rs
```

Use provider kind:

```text
yandex_telemost_user
```

Use secret purpose:

```text
yandex_telemost_oauth_token
```

The backend stores raw OAuth tokens only in HostVault. The provider account
stores a secret reference and lifecycle/capability metadata.

The desktop companion opens Telemost only in a visible owner-controlled WebView.
Hidden recording is forbidden. Local MP3 recording requires
`consent_attested=true`.

Speaker timeline files derived from WebView DOM are hints only:

```text
truth_status = hint_not_truth
```

## Consequences

Positive:

- Telemost does not become a product domain.
- API credentials stay out of business state and events.
- Local recording is explicit and owner-visible.
- Whisper/diarization workflows get useful hint files without trusting the DOM
  as evidence.

Negative:

- macOS and Windows require explicit loopback audio setup.
- WebView speaker extraction is heuristic and must be improved against real DOM
  observations.
- Review and owner-domain promotion quality still depends on conservative
  provider-neutral workflow mapping rather than Telemost-specific direct domain
  mutation.

## Rejected options

### Create `domains/yandex_telemost`

Rejected because providers are external systems, not bounded contexts.

### Hidden background browser join and capture

Rejected because it breaks the Макошь owner-visible runtime model and creates a
hidden capture path that the provider runtime must not own.

### Treat WebView active-speaker DOM as truth

Rejected. It is only a weak hint for diarization.
