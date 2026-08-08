# Telegram API Reference

Статус: verified route audit + целевой API scope на 2026-06-17.

Все текущие маршруты защищены локальным API guard из ADR-0056, если явно не
указано иначе. Browser WebSocket clients передают local secret через
`makosh_secret`, потому что native WebSocket requests не могут выставить
`X-Макошь-Secret`.

## Base

```text
/api/v1/integrations/telegram
```

## Navigation

- [Foundation: capability, accounts, runtime, QR login](api/foundation.md)
- [Conversations: chats, topics, messages, reactions](api/conversations.md)
- [Media and Search: media, attachments, search](api/media-search.md)
- [Operations and Realtime: automation, audit, calls, events, frontend client](api/operations-realtime.md)

## Scope Notes

- Telegram в Макошь остаётся Communication Channel, а не отдельным memory или
  intelligence доменом.
- Все provider writes должны проходить через capability gates, audit boundary и
  durable outbox/provider command model.
- Realtime контракты описаны отдельно, но используют общий Макошь event bus и
  общие transport routes.
