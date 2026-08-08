# Telegram API Reference — Media and Search

See also: [API Index](../api.md), [Conversations](conversations.md), and [Operations and Realtime](operations-realtime.md).

## Ownership

Telegram provider runtime and debug surfaces live under:

```text
/api/v1/integrations/telegram/*
```

Shared Communication attachment and business read-model APIs live under:

```text
/api/v1/communications/*
```

## Media Routes

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/integrations/telegram/provider-media/download` | Download TDLib media and persist local attachment/blob state |
| POST | `/api/v1/integrations/telegram/provider-media/upload` | Queue provider-side media send from a local attachment or blob |
| GET | `/api/v1/communications/attachments/{attachment_id}/preview` | Shared safe preview endpoint |
| POST | `/api/v1/communications/attachments/import` | Shared local attachment import |
| GET | `/api/v1/communications/attachments/search` | Shared attachment search |
| GET | `/api/v1/communications/search/media` | Макошь projected media search/filter |

## Search Routes

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/integrations/telegram/provider-search` | Provider raw search/refresh trigger; returns status metadata only |
| GET | `/api/v1/communications/conversations/search` | Макошь projected conversation search |
| GET | `/api/v1/communications/search/messages` | Макошь projected message search |
| GET | `/api/v1/communications/search/media` | Макошь projected media search |
| GET | `/api/v1/communications/search` | Макошь email/full-text business search over the projected read-model |
| GET | `/api/v1/communications/saved-searches` | Shared saved-search surface |

## Notes

- Provider media upload does not accept raw file bytes or a bare blob reference. It requires an already imported local `attachment_id` with a `clean` scan verdict; an optional `blob_id` is checked only for consistency with that attachment.
- A completed provider-media download stores the bytes through the shared local blob boundary, records its scan verdict, and returns the canonical `attachment_id` and `blob_id`. The projected attachment keeps the provider attachment id separately, so preview and future downloads do not confuse provider identifiers with Макошь storage identifiers.
- TDLib ingestion normalizes `photo`, `video`, `document`, `audio`, `voice`, `video_note`, `sticker`, and `animation` into the same projected `attachments` metadata. For a photo, Макошь selects the largest provider `photoSize` file; it does not download bytes until the user requests it.
- Repeating a Telegram history sync enriches an already projected message through a provider observation when attachment metadata or a reaction summary was previously absent. Raw provider evidence remains immutable, and a pre-existing downloaded/canonical attachment or realtime reaction summary is never replaced by a stale descriptor.
- Shared Communication attachment APIs remain the canonical safe access path for local preview and import.
- Normal user-facing Communication search uses `/api/v1/communications/search/*`; provider search is runtime/debug/sync-assist only and must not return projected message or media items.
