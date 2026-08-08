# Telegram API Reference — Conversations

See also: [API Index](../api.md), [Foundation](foundation.md), and [Media and Search](media-search.md).

## Ownership

Telegram runtime, sync, provider search, provider commands, and setup APIs live under:

```text
/api/v1/integrations/telegram/*
```

Communications business state remains provider-neutral and lives under:

```text
/api/v1/communications/*
```

## Current Telegram Runtime Routes

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/communications/conversations?account_id=&limit=` | Projected Telegram chat list |
| GET | `/api/v1/communications/conversation-folders?account_id=` | Projected Telegram folders for user-facing chat actions |
| GET | `/api/v1/communications/conversations/{conversation_id}` | Projected conversation detail |
| GET | `/api/v1/communications/conversations/{conversation_id}/members` | Projected member list |
| POST | `/api/v1/integrations/telegram/provider-sync/conversations/{telegram_chat_id}/members` | Provider-backed member sync |
| GET | `/api/v1/communications/conversations/{conversation_id}/pinned-messages` | Projected pinned messages |
| GET | `/api/v1/communications/conversations/search` | Projected conversation search |
| GET | `/api/v1/integrations/telegram/conversation-folders` | Provider folder projection |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/join` | Join command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave` | Leave command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/pin` | Pin command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unpin` | Unpin command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/archive` | Archive command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unarchive` | Unarchive command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/mute` | Mute command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unmute` | Unmute command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/read` | Provider read reconciliation command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unread` | Provider unread reconciliation command |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}` | Add folder assignment |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove` | Remove folder assignment |
| POST | `/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/reassign` | Replace folder assignments |
| POST | `/api/v1/integrations/telegram/provider-sync/chats` | Provider chat sync |
| POST | `/api/v1/integrations/telegram/provider-sync/history` | Provider history sync |
| POST | `/api/v1/communications/conversations/{telegram_chat_id}/avatar` | Download the current TDLib chat photo to local blob storage |
| GET | `/api/v1/communications/conversations/{telegram_chat_id}/avatar` | Read a locally stored raster chat photo; requires the normal Макошь API secret header |

## Topics

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/communications/conversations/{conversation_id}/topics` | Topic list |
| POST | `/api/v1/communications/conversations/{conversation_id}/topics` | Create topic command |
| GET | `/api/v1/communications/topics/{topic_id}` | Topic detail |
| POST | `/api/v1/integrations/telegram/provider-commands/topics/{topic_id}/close` | Close or reopen topic |
| GET | `/api/v1/communications/topics/{topic_id}/messages` | Topic-scoped message list |
| GET | `/api/v1/communications/topics/search` | Topic search |

## Message Runtime Routes

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/communications/messages?channel_kind=telegram&account_id=&conversation_id=&limit=` | Projected Telegram message list; `conversation_id` is the provider chat identifier for Telegram projections |
| POST | `/api/v1/integrations/telegram/fixtures/messages` | Fixture ingest |
| POST | `/api/v1/communications/conversations/{conversation_id}/messages` | User-facing send |
| POST | `/api/v1/communications/messages/{message_id}/reply` | User-facing reply |
| POST | `/api/v1/communications/messages/{message_id}/forward` | User-facing forward |
| PATCH | `/api/v1/communications/messages/{message_id}` | User-facing edit |
| DELETE | `/api/v1/communications/messages/{message_id}` | User-facing delete |
| POST | `/api/v1/communications/messages/{message_id}/restore-visibility` | Restore visibility command |
| POST | `/api/v1/communications/messages/{message_id}/pin` | Pin command |
| POST | `/api/v1/communications/messages/{message_id}/mark-read` | Message-level read command |
| GET | `/api/v1/communications/messages/{message_id}/versions` | Version list |
| GET | `/api/v1/communications/messages/{message_id}/tombstones` | Tombstone list |
| GET | `/api/v1/communications/messages/{message_id}/raw-evidence` | Sanitized raw evidence |
| GET | `/api/v1/communications/messages/{message_id}/reply-chain` | Reply chain |
| GET | `/api/v1/communications/messages/{message_id}/forward-chain` | Forward chain |
| GET/POST/DELETE | `/api/v1/communications/messages/{message_id}/reactions` | Reaction projection and commands |

## Notes

- Provider search and provider sync remain integration-scoped runtime surfaces.
- Chat avatar bytes remain local and content-addressed. Макошь only serves JPEG, PNG, or WebP
  after matching the stored blob to the current TDLib file reference; an avatar is never projected
  as a message attachment or exposed as a TDLib filesystem path.
- Provider-command message routes under `/api/v1/integrations/telegram/provider-commands/messages/*` are debug/control/recovery surfaces, not normal Communication UI APIs.
- Communication business UI should consume `/api/v1/communications/*` routes and not call these provider runtime routes directly.
- Provider observations are projected on the application side; integrations do not own communication business mutations.
- When a user opens a Telegram dialog without projected messages, the UI requests one
  successful `mode=latest` history read for that dialog during the current session.
  A transient failed read can be retried by reopening the dialog. TDLib data
  first enters the provider-observation flow; the reader updates when its
  Communications projection is available through realtime events or the bounded
  polling fallback. No provider send or state-changing command is issued.
- Selecting a composer attachment only stages it locally. The `send_media`
  provider command is queued after the user explicitly selects **Send**; the
  editor text becomes the plain-text media caption.
- Reaction commands derive `sender_id` from the authenticated Telegram account
  when the client omits it. Product UI must not ask a user to enter their own
  Telegram identifier; an explicit sender remains accepted only for compatible
  diagnostic clients.
