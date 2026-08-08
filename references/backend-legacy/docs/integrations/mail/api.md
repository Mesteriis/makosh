# Email Channel — API Reference

This file documents legacy email-channel compatibility routes and is reference
evidence, not the clean-room API contract. The target Mail plugin exposes a
provider-specific operational experience and separately maps email observations
into neutral evidence for Personas, Organizations, Projects, Documents, Tasks,
Decisions and Obligations.

Base: `/api/v1/communications/`

## Account Management

| Метод | Путь | Описание |
|---|---|---|
| GET | `/api/v1/integrations/mail/accounts` | Список email provider accounts с capability flags |
| GET | `/api/v1/integrations/mail/accounts/{account_id}` | Детали email account и capability flags |
| DELETE | `/api/v1/integrations/mail/accounts/{account_id}` | Удалить только unused account metadata; retained raw/messages block deletion |
| POST | `/api/v1/integrations/mail/accounts/{account_id}/logout` | Локально выйти: пометить account logged_out и выключить sync |
| GET | `/api/v1/integrations/mail/accounts/{account_id}/export` | Экспорт sanitized settings без credentials и secret refs |
| POST | `/api/v1/integrations/mail/accounts/import` | Импорт sanitized account metadata и sync settings; secret-bearing payload rejected |
| POST | `/api/v1/integrations/mail/accounts/gmail/oauth/start` | Начать Gmail OAuth setup |
| POST | `/api/v1/integrations/mail/accounts/gmail/oauth/complete` | Завершить Gmail OAuth setup |
| POST | `/api/v1/integrations/mail/accounts/imap` | Создать iCloud/generic IMAP+SMTP account |
| GET | `/api/v1/integrations/mail/accounts/sync-status` | Account-scoped sync status list |
| GET/PUT | `/api/v1/integrations/mail/accounts/{account_id}/sync-settings` | Read/update sync settings |
| POST | `/api/v1/integrations/mail/accounts/{account_id}/sync-now` | Manual sync |
| POST | `/api/v1/integrations/mail/accounts/{account_id}/sync-full-resync` | Manual full resync |

Account export/import never includes credential values. Import rejects payloads
that contain secret-like keys such as `password`, `secret_ref`, `token` or
`credential`; credentials must be reconnected through account setup.

`POST /api/v1/integrations/mail/accounts/imap` accepts optional SMTP settings
(`smtp_host`, `smtp_port`, `smtp_tls`, `smtp_starttls`, `smtp_username`) for
IMAP-backed sending. Credential values are still stored through the configured
secret resolver; account config stores only non-secret SMTP metadata.

## Realtime Events

| Метод | Путь | Описание |
|---|---|---|
| GET | `/api/events/ws?after_position=&makosh_secret=` | Protected WebSocket event stream with replay and heartbeat foundation; browser clients pass the local API secret as `makosh_secret` because native WebSocket requests cannot set `X-Макошь-Secret` |
| GET | `/api/events/stream?after_position=` | Protected SSE stream with replay and heartbeat |
| GET | `/api/v1/events?after_position=&limit=&wait_seconds=` | Protected JSON replay / long-poll fallback; records `event.list` audit entry |

Canonical mail sync event types emitted by sync runs:
`mail.sync.started`, `mail.sync.progress`, `mail.sync.completed`,
`mail.sync.failed`, `mail.sync.skipped`.

Canonical local message-action event types emitted by bounded bulk actions:
`mail.message.read`, `mail.message.unread`, `mail.message.archived`,
`mail.message.deleted`, `mail.message.restored`, `mail.message.pinned`,
`mail.message.unpinned`, `mail.message.important`,
`mail.message.not_important`, `mail.message.labeled`,
`mail.message.unlabeled`, `mail.message.snoozed`.

Canonical local draft event types emitted by draft mutations:
`mail.draft.created`, `mail.draft.updated`, `mail.draft.deleted`.

## Delivery / Receipts

| Метод | Путь | Описание |
|---|---|---|
| POST | `/read-receipts` | Record a provider read receipt, correlate to sent outbox by `account_id` + `provider_message_id` when possible, and append sanitized `mail.read_receipt.recorded` event |
| POST | `/delivery-notifications` | Parse a provider DSN/MDN notification payload; DSN records sanitized outbox delivery status and emits `mail.outbox.delivery_status_changed`, MDN records a sanitized read receipt |
| POST | `/provider-delivery-events` | Protected structured provider-runtime callback path for delivered/delayed/failed/read events; reuses sanitized outbox delivery status and read-receipt persistence |

## Communication Messages

| Метод | Путь | Описание |
|---|---|---|
| GET | `/messages` | Список email-backed Communication messages (?account_id, ?workflow_state, ?channel_kind, ?limit) |
| GET | `/messages/{id}` | Details for an email-backed Communication message with attachments |
| PUT | `/messages/{id}/workflow-state` | Изменить workflow-состояние |
| GET | `/messages/states` | Счётчики по состояниям |
| POST | `/messages/{id}/analyze` | Запустить AI-анализ (эвристики); returns and persists `summary_contract` with `key_points`, `action_items`, `risks`, `deadlines` and review-only Mail knowledge candidates for events, personas, organizations, documents and agreements under message metadata |
| GET | `/messages/{id}/explain` | Почему письмо важно |
| GET | `/messages/{id}/smart-cc` | Умные подсказки CC |
| POST | `/messages/{id}/pin` | Переключить pin |
| POST | `/messages/{id}/important` | Переключить локальный important flag в `message_metadata` |
| POST | `/messages/{id}/snooze` | Отложить до даты |
| POST | `/messages/{id}/mute` | Переключить mute |
| POST | `/messages/{id}/labels` | Добавить метку |
| DELETE | `/messages/{id}/labels` | Удалить метку |
| POST | `/messages/bulk-actions` | Bounded local bulk actions: mark read/unread, archive, trash, restore, pin/unpin, important/not important, snooze, add/remove label; successful matched updates append canonical `mail.message.*` events |
| GET | `/messages/{id}/export?format=md\|eml\|json` | Export source message |
| GET | `/messages/{id}/remote-image?url=` | Privacy-preserving remote image proxy; only fetches public HTTP(S) image URLs referenced by that message HTML |

## Отправка

| Метод | Путь | Описание |
|---|---|---|
| POST | `/send` | Отправить письмо immediately via SMTP/Gmail API or enqueue it into durable outbox for scheduled/undoable delivery |
| GET | `/outbox?account_id=&status=&cursor=&limit=` | Cursor-paginated durable outbox rows for delivery UX; response includes `items`, `next_cursor` and `has_more`, and `metadata.delivery_status` plus sanitized `metadata.latest_read_receipt` expose provider evidence without recipients or diagnostics |
| POST | `/outbox/{outbox_id}/undo` | Cancel a queued/scheduled outbox row while its undo deadline is still open |
| POST | `/messages/{id}/reply` | Ответить |
| POST | `/messages/{id}/reply-all` | Ответить всем |
| POST | `/messages/{id}/forward` | Переслать |
| POST | `/messages/{id}/redirect` | Redirect/resend original message body and subject through durable outbox with original-message provenance |
| POST | `/messages/{id}/forward-eml` | Переслать как EML |

## AI

| Метод | Путь | Описание |
|---|---|---|
| POST | `/messages/{id}/ai-reply` | Сгенерировать AI-ответ |
| POST | `/messages/{id}/ai-reply-variants` | Варианты ответа (языки × тоны) |
| POST | `/messages/{id}/bilingual-reply-flow` | Prepare bilingual reply review: original message, Russian translation, Russian reply text, back-translation, selected tone and send-readiness flag; degrades with explicit runtime fallback when local AI is unavailable |
| GET/PUT | `/messages/{id}/ai-state` | Read or transition first-class mail AI lifecycle state (`NEW`, `PROCESSING`, `PROCESSED`, `REVIEW_REQUIRED`, `FAILED`, `ARCHIVED`); transitions append `mail.ai_state.changed` events |
| POST | `/messages/{id}/extract-tasks` | Извлечь задачи |
| POST | `/messages/{id}/extract-notes` | Извлечь заметки |
| GET | `/messages/{id}/detect-language` | Определить язык |
| POST | `/messages/{id}/translate` | Перевести |

## Безопасность

| Метод | Путь | Описание |
|---|---|---|
| GET | `/messages/{id}/spf-dkim` | SPF/DKIM/DMARC анализ |
| GET | `/messages/{id}/signature` | Детекция подписей (S/MIME, PGP) |

## Треды

| Метод | Путь | Описание |
|---|---|---|
| GET | `/threads?account_id=&cursor=&limit=` | Cursor-paginated thread list ordered by most recent activity |
| GET | `/threads/messages?account_id=&subject=` | Сообщения в треде, включая `provider_record_id` для корректного inline reply handoff |
| POST | `/threads/translate?account_id=&subject=&limit=` | Translate every message body in a thread to `target_language`; returns per-message fallback entries when local AI runtime is unavailable |

## Черновики

| Метод | Путь | Описание |
|---|---|---|
| GET | `/drafts?account_id=&status=&cursor=&limit=` | Cursor-paginated draft list ordered by `updated_at DESC, draft_id ASC`; returns `items`, `next_cursor`, `has_more` |
| POST | `/drafts` | Создать/обновить; appends sanitized `mail.draft.created` or `mail.draft.updated` events without subject/body content |
| GET | `/drafts/{id}` | Детали черновика |
| DELETE | `/drafts/{id}` | Удалить; appends sanitized `mail.draft.deleted` when a draft existed |

## Финансы

| Метод | Путь | Описание |
|---|---|---|
| GET | `/finance/invoices` | Список счетов |
| POST | `/finance/invoices` | Создать/обновить счёт |

## Юрдокументы

| Метод | Путь | Описание |
|---|---|---|
| GET | `/legal` | Список юрдокументов |
| POST | `/legal` | Создать/обновить |

## Сертификаты

| Метод | Путь | Описание |
|---|---|---|
| GET | `/certificates` | Список сертификатов |
| POST | `/certificates` | Добавить сертификат |
| GET | `/certificates/expiring?days=90` | Истекающие сертификаты |

## Аналитика

| Метод | Путь | Описание |
|---|---|---|
| GET | `/analytics/health` | Compatibility route for mailbox attention analytics |
| GET | `/analytics/senders?account_id=&cursor=&limit=` | Cursor-paginated top sender analytics ordered by `message_count DESC, sender ASC`; returns `items`, `next_cursor`, `has_more` |

## Подписки

| Метод | Путь | Описание |
|---|---|---|
| GET | `/subscriptions?account_id=&cursor=&limit=` | Cursor-paginated newsletter/source detection ordered by `message_count DESC, sender ASC`; returns `items`, `next_cursor`, `has_more` |

## Поиск

| Метод | Путь | Описание |
|---|---|---|
| GET | `/search?q=...` | Полнотекстовый поиск |

## Saved Searches / Smart Folders

| Метод | Путь | Описание |
|---|---|---|
| GET | `/saved-searches?smart_folder=&account_id=&limit=` | List durable saved searches and smart-folder definitions with `message_count` derived from the same parsed Rules Builder semantics used by runtime message search, including `mode:any` and field rules such as `subject:`, `body:` and `from:` |
| POST | `/saved-searches` | Create a saved search or smart folder; appends a canonical event |
| PUT | `/saved-searches/{saved_search_id}` | Update a saved search definition; appends a canonical event |
| DELETE | `/saved-searches/{saved_search_id}` | Delete a saved search definition; appends a canonical event |

## Custom Folders

| Метод | Путь | Описание |
|---|---|---|
| GET | `/folders?account_id=&cursor=&limit=` | Cursor-paginated local custom folder list with per-folder `message_count` |
| POST | `/folders` | Create a local custom folder; appends a canonical event |
| PUT | `/folders/{folder_id}` | Update a local custom folder; appends a canonical event |
| DELETE | `/folders/{folder_id}` | Delete a local custom folder; appends a canonical event |
| GET | `/folders/{folder_id}/messages?cursor=&limit=` | Cursor-paginated messages assigned to a local custom folder |
| POST | `/folders/{folder_id}/messages/{message_id}/copy` | Copy a message into a local custom folder; returns the projected folder-message row and appends a canonical event |
| POST | `/folders/{folder_id}/messages/{message_id}/move` | Move a message into a local custom folder, removing it from other custom folders; returns the projected folder-message row and appends a canonical event |

Custom folders are local-first Макошь organization state. These routes do not
perform provider-side Gmail/IMAP folder mutations.

The current Mail UI presents slash-delimited folder names as a local hierarchy
and applies the same hierarchy-aware ordering to both visible rows and
drag/drop reorder operations. Folder create/edit flows now split the hierarchy
into parent-path suggestions plus a leaf-name field with full-path preview
before the existing `name` payload is persisted.

## Вложения

| Метод | Путь | Описание |
|---|---|---|
| GET | `/attachments/search?q=&account_id=&content_type=&scan_status=&cursor=&limit=` | Cursor-paginated attachment metadata search; does not read blob bytes |
| POST | `/attachments/{attachment_id}/translate` | Translate extracted text supplied by the caller for a known attachment; returns runtime fallback when local AI is unavailable |
| GET | `/attachments/{attachment_id}/preview` | Return a bounded safe text or raster image preview for a known local attachment blob; blocks suspicious/malicious/failed scan states, allows only text-like files plus PNG/JPEG/GIF/WebP images and does not render HTML |
| GET | `/attachments/{attachment_id}/archive-inspection` | Read the local ZIP blob for a known attachment and return bounded archive metadata without extracting files |
| GET | `/attachments/duplicates` | Поиск дубликатов |

## Прочее

| Метод | Путь | Описание |
|---|---|---|
| GET | `/personas` | Список персон |
| POST | `/personas` | Создать персону |
| GET | `/templates/rich` | List durable rich email templates from `email_templates`; each row includes derived `placeholder_variables`, `undeclared_variables`, `unused_variables` and `malformed_placeholders` diagnostics used by the compose-side template library |
| POST | `/templates/rich` | Upsert a durable rich email template with `template_id`, `name`, `subject_template`, `body_template`, `variables` and optional `language`; response includes the same derived diagnostics as list |
| DELETE | `/templates/rich/{template_id}` | Delete a durable rich email template by `template_id` |
| POST | `/templates/rich/render` | Render a stored template by `template_id` with variable substitution into `rendered.subject` and `rendered.body`; `rendered.missing_variables`, `rendered.unresolved_variables` and `rendered.malformed_placeholders` report incomplete or invalid merge data |
| POST | `/templates/rich/mail-merge-preview` | Bounded non-sending mail-merge preview for up to 250 variable rows; returns per-row rendered subject/body, readiness and aggregate ready/blocked counts, and is now exposed in the compose-side template library as a JSON-row preview surface |
| GET | `/blockers` | Список архитектурных блокеров |
| POST | `/messages/{id}/imap-mark-read` | Синхронизировать read-флаг с сервером |
| POST | `/messages/{id}/imap-delete` | Удалить на сервере |
