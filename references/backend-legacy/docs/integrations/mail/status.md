# Email Channel — Implementation Status

Этот файл описывает текущую email-channel реализацию. Канонический домен —
Communications; Email is a channel/source boundary, not the product identity.
Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

The percentages below describe email-channel coverage only. They are not product
completion scores for Communications, Memory, Knowledge, Obligations,
Decisions or Polygraph.

Спецификация: 36 разделов. Статус на 2026-06-15.

| § | Раздел | Статус | % |
|---|---|---|---|
| 1 | Назначение модуля | ✓ | 100 |
| 2 | Ключевые принципы | ✓ | 100 |
| 3 | Источники почты и аккаунты | ◐ | 80 |
| 4.1 | Получение и чтение | ✓ | 100 |
| 4.2 | Создание и отправка | ◐ | 85 |
| 4.3 | Ответы | ✓ | 100 |
| 4.4 | Пересылка | ✓ | 100 |
| 4.5 | Организация | ✓ | 100 |
| 5 | Состояния письма и workflow | ✓ | 100 |
| 6 | AI Inbox и понимание писем | ✓ | 100 |
| 7 | Spam, scam, phishing | ◐ | 80 |
| 8 | Безопасность вложений | ◐ | 20 |
| 9 | Вложения и документы | ◐ | 78 |
| 10 | Финансовые письма и счета | ✓ | 100 |
| 11 | Юридические письма и контракты | ✓ | 100 |
| 12 | Цифровые подписи и сертификаты | ◐ | 50 |
| 13 | Многоязычная почта | ✓ | 100 |
| 14 | Compose, AI Reply | ◐ | 88 |
| 15 | Шаблоны писем | ◐ | 90 |
| 16 | Доставка и Outbox | ◐ | 75 |
| 17 | Obligation/Follow-Up engine integration | ✗ | 0 |
| 18 | Persona identity traces | ✓ | 100 |
| 19 | Проекты и привязка писем | ✓ | 100 |
| 20 | Задачи из писем | ✓ | 100 |
| 21 | Заметки и knowledge | ✓ | 100 |
| 22 | AI Rules и автоматизация | ✓ | 100 |
| 23 | Поиск | ✓ | 100 |
| 24 | Треды | ✓ | 100 |
| 25 | Исходящие и черновики | ✓ | 100 |
| 26 | Email attention analytics | ✓ | 100 |
| 27 | Подписки и рассылки | ✓ | 100 |
| 28 | Интеграции из почты | ✗ | 0 |
| 29 | Массовые действия | ◐ | 35 |
| 30 | Import, export, архив | ◐ | 55 |
| 31 | UI идеи | ◐ | 70 |
| 32-36 | Каталоги, домены, итоги | ✓ | 100 |

### Легенда

- ✓ — полностью реализовано
- ◐ — частично (детали ниже)
- ✗ — не реализовано (см. [блокеры](blockers.md))

### Детали частичной реализации

**§3 (80%)**: Gmail, iCloud, IMAP работают. Exchange, Proton, Fastmail, Maildir — требуют отдельных провайдер-адаптеров.

**iCloud Contacts**: iCloud address books are imported read-only through CardDAV
using the existing iCloud account identity and app-specific password from the
host vault. The adapter performs standard CardDAV discovery and projects vCard
entries through the existing address-book workflow. Remote iCloud contact writes
remain unsupported.

**§5 (100%)**: После открытия письма UI ждёт 2 секунды и сразу переводит
локальную проекцию Макошь в `reviewed`. Синхронизация с провайдером выполняется
best-effort: Gmail снимает label `UNREAD` через Gmail API; iCloud и generic IMAP
устанавливают `\\Seen` по сохранённым `mailbox` и `UID`. Ошибка provider-write
не откатывает локальный результат и записывается как sanitised warning. Gmail-
аккаунты, выданные до добавления OAuth scope `gmail.modify`, должны быть
переподключены, чтобы пользователь подтвердил обновлённое разрешение.

**§4.2 (85%)**: Черновики, отправка, durable redirect enqueue,
scheduled-send foundation and undo-send foundation work. Compose now exposes
scheduled send time and undo-send window controls that pass through the existing
draft/send APIs, and send review shows the scheduled delivery time plus undo
window before delivery handoff. The message actions UI now exposes Reply All and
Forward compose handoffs plus explicit-recipient Redirect enqueue through the
existing outbox path. Production runtime delivery still depends on the
remaining outbox scheduler/provider wiring.

**§7 (80%)**: Эвристики и SPF/DKIM/DMARC-парсинг работают. Нет карантина (инфраструктурная задача).

**§6 / AI Summary**: `POST /messages/{id}/analyze` now returns and persists a
structured local `summary_contract` under message metadata with `key_points`,
`action_items`, `risks`, `deadlines` and review-only Mail knowledge candidates
for events, personas, organizations, documents and agreements. The contract is
deterministic and works without the local AI runtime. Full AI runtime
orchestration, source-evidence citation and durable result lifecycle remain
future slices.

**§8 (20%)**: Mail projection now uses a conservative heuristic attachment
safety scanner. It flags executable payload magic bytes, active-content
extensions, macro-enabled Office extensions and known MIME/filename mismatches
as `malicious` or `suspicious`, and stores scan engine, timestamp, summary and
structured reasons in attachment metadata. It deliberately leaves unmatched
attachments as `not_scanned` and never emits `clean` without a real scanner
backend. ClamAV, sandbox execution, OLE macro parsing, quarantine and
scanner-backed clean-state UX remain blockers.

**§9 (78%)**: 15 категорий, дедупликация, cursor-paginated attachment
metadata search, a Zod/Vee metadata search panel with TanStack
Query/Table/Virtual results and a TanStack Table-backed message attachment
metadata table work. Bounded ZIP metadata inspection now exists with traversal,
entry-count, depth and uncompressed-size limits, protected attachment API wiring
and a message-detail UI inspection action. The protected attachment API and
message-detail UI can now render bounded safe text previews plus bounded
PNG/JPEG/GIF/WebP image previews for known local blobs while preserving scan
status and avoiding HTML execution. OCR,
extracted attachment content search, rich/PDF preview, persisted preview
artifacts, persisted inspection metadata, nested archive policy and RAR/7z
support — future slices.

**§12 (55%)**: Метаданные и детекция работают. MessageBodyTab now exposes
on-demand Security Review for SPF/DKIM/DMARC plus signature detection through
TanStack Query mutations and stores the result in the existing Mail insight
state. Compose can now insert stored persona signatures through a TanStack
Query-backed signature picker, including plain-text and HTML draft insertion
with autosave. Certificate inventory, expiring certificate and add-certificate
metadata APIs are now wired into the mail ActionBar through TanStack Query and a
Zod/Vee form. The UI records metadata and storage references only; certificate
payloads/private keys remain behind the configured secret/vault boundary. Smart
CC recipient suggestions are also exposed in the message reader through a
dedicated Recipient Suggestions review panel. Remote images in HTML mail are
blocked by default in the reader and can be loaded only through the
message-scoped backend `/remote-image?url=` proxy after explicit user action.
Крипто-верификация and message signing remain blockers (OpenSSL/GPG).

**§14 (88%)**: AI-генератор работает. Compose now has explicit Text, Rich and
raw HTML modes; Rich mode uses a TipTap-backed editor runtime with a local
mail-safe paragraph/heading/bullet-list/ordered-list/blockquote/bold/italic/link/alignment
schema, normalizes compose links to safe `http`, `https` and `mailto` hrefs
with `noopener noreferrer`, sanitizes pasted/dropped HTML to the supported
mail-safe subset before insertion, emits dropped files to Compose attachment
staging, persists `body_html` in autosaved drafts, derives `body_text` fallback
and sends `body_html` through the send mutation. Backend RFC2822 assembly now
emits HTML sends as `multipart/alternative` with text/plain and text/html parts
instead of dropping the HTML body. Compose can stage dropped or selected
attachment files as removable local chips and blocks send while those files are
staged, because durable attachment upload/draft persistence/provider send
integration is not wired yet.
Single-message AI Reply is now exposed in MessageBodyTab through tone/language
controls, a TanStack Query mutation-backed review card and Apply-to-Compose
handoff with reply subject/body plus `in_reply_to`. AI reply variants are also
exposed through `/messages/{id}/ai-reply-variants`, a typed frontend API,
TanStack mutation and review cards for language x tone candidates with the same
Apply-to-Compose handoff.
Bilingual reply review now has a
protected API and Vue panel that prepare the Original → Translation → Russian
reply → Back Translation → Tone contract with explicit local-AI runtime
fallback. Send review now surfaces scheduled-send timing and undo-send delay
before handoff. Rich delivery feedback remains incomplete until SMTP
feedback/outbox runtime wiring is complete.

**§13**: Message-level language detection and translation exist. MessageBodyTab
now exposes mail-local importance explanation and language detection through a
TanStack mutation-backed `Importance & Language` panel. Thread-level translation
now returns ordered per-message translation entries and degrades per message when
the local AI runtime is unavailable. Attachment-level translation now accepts
caller-provided extracted text for a known attachment and degrades to a
runtime-unavailable fallback instead of failing the request. OCR, persisted
extracted attachment text and review/persistence UX remain incomplete.
Thread lists are cursor-paginated, rendered as server-backed navigator rows
with a load-more path and exposed through TanStack infinite query hooks.
Selecting a thread now loads thread messages through a dedicated query hook and
renders a conversation timeline in the detail pane with a per-message handoff
back to the full mail reader. The timeline now auto-expands the latest message
when a thread opens, supports collapsed/expanded message bodies, quoted-content
separation for expanded reads, inline thread attachment surfacing with
scan-state badges, per-attachment inline preview/archive inspection actions,
text-attachment translation from the thread timeline and inline per-message
reply handoff to Compose using `provider_record_id`; the handoff now pre-fills
HTML/plain quoted reply content. A rich inline reply editor is available inside
the thread timeline and preserves typed HTML when continuing in Compose. Inline
replies now save drafts on editor blur or explicit Save Draft through the
existing draft mutation, and use an inline review step before sending through
the existing provider-write send mutation. The conversation header now also
exposes global `Expand all` / `Collapse all` controls plus a thread-wide
quoted-content show/hide toggle, so long chains can be scanned without
expanding every message or visually carrying the full quoted history at once.
Full conversation-reader ergonomics still remain incomplete for deeper
Outlook/Gmail-style thread controls.

**§6/Realtime**: Mail AI lifecycle state now has a first-class durable
`mail_ai_states` table, `GET/PUT /messages/{id}/ai-state` API and sanitized
`mail.ai_state.changed` canonical events for SSE replay/invalidation. Automatic
AI runtime orchestration and review UI remain future slices. Realtime transport
now has a protected backend WebSocket event stream at
`GET /api/events/ws?after_position=&makosh_secret=`, uses WebSocket-first
browser transport selection with SSE fallback, and adds protected JSON long-poll fallback through
`GET /api/v1/events?after_position=&limit=&wait_seconds=` with `event.list`
audit records. The frontend now monotonically persists the last replay cursor for
offline resume, surfaces live/degraded/offline transport status in the shell and
uses typed query invalidation for AI state, outbox delivery, read receipts,
drafts, saved searches, sync progress, local bulk message actions and folder
events. Local `mail.message.*` events now also apply entity-level TanStack Query
cache patches for affected message list/detail rows before invalidation, reducing
visible reload churn for read/unread, archive/trash/restore, pin/important,
label and snooze actions. Outbox delivery-status and read-receipt events now
patch cached outbox row metadata before invalidation, so the compact delivery
strip can update without waiting for a full refetch. AI-state events now patch
the dedicated TanStack Query AI-state cache before invalidating message/list
views. Draft-delete events now remove cached draft rows before invalidation;
draft create/update events remain refetch-driven because sanitized realtime
payloads intentionally exclude private draft subject/body/recipient content.
Folder create/update/delete events now patch cached folder-list rows before
invalidation. Folder-message copy/move responses and events now include the
projected folder-message row, allowing cached folder-message lists to insert the
destination row and remove moved messages from other cached folder lists before
invalidation.
Saved-search and smart-folder create/update/delete events now patch cached
definition lists before invalidation, preserving smart-folder/account query
filters.
Sync progress events now patch cached account sync-status rows before
invalidation, so progress phase/percent/counts can move without waiting for a
full refetch.
Provider sync settings are now user-editable from the mail ActionBar through a
TanStack Query-backed helper and Zod/Vee form for sync enablement, batch size
and poll interval; the component keeps provider credentials out of UI state and
does not import API clients directly.
Mail sync runs now append sanitized `mail.sync.started`,
`mail.sync.progress` and terminal `mail.sync.*` canonical events for replay.
Local draft mutations append sanitized `mail.draft.*` canonical events, and
local bulk message actions append sanitized `mail.message.*` canonical events
for SSE/long-poll invalidation. ComposeDrawer draft autosave now uses a tested
debounced helper wired through TanStack Query draft mutations instead of direct
component API calls; rendered Browser validation verified new-compose edits
persist after the debounce and show `Draft saved`. The communications store now
selects the first synced account when no account is selected so new compose
drafts have a valid `account_id`. Explicit user-facing transport-policy controls,
richer sync progress UX and entity-level patch streaming remain future work.

**§15 (90%)**: Переменные, conditional, table, button работают. The
`/templates/rich` backend API now durably saves, lists and renders templates
from `email_templates` with variable substitution into subject/body. Compose now
has a TanStack Query-backed template picker with variable inputs that applies a
rendered template into subject/body, plus a Zod/Vee-backed inline save flow and
selected-template update action for storing the current compose subject/body as a
reusable rich template. Durable template delete API and selected-template delete
action now remove obsolete templates and invalidate the template query. Basic
merge validation blocks render/apply until required variable values are filled.
Backend rendering now accepts whitespace-tolerant placeholders such as
`{{ name }}` while preserving unresolved placeholders, and render responses now
report missing declared variables, unresolved placeholders and malformed
placeholders. Template save/list responses now expose derived
`placeholder_variables`, `undeclared_variables`, `unused_variables` and
`malformed_placeholders` metadata so stale or API-created template definitions
can be audited without a schema migration. Empty, invalid-name and unclosed
placeholders are treated as malformed. Compose refuses to apply a rendered
template when the backend still reports unresolved variables or malformed
placeholders, and blocks save/update when the current compose content contains
malformed placeholders. The picker now surfaces stored-template diagnostics for
malformed placeholders, undeclared variables and unused variables, and blocks
Apply for selected templates that cannot render cleanly. The backend now exposes
a bounded non-sending `/templates/rich/mail-merge-preview` endpoint that renders
up to 250 variable rows and returns per-row readiness plus aggregate ready/blocked
counts. The compose-side template library now supports in-panel search,
updated-first ordering, auto-selection of the first visible template and preview
metadata so reusable templates are manageable without leaving the compose flow.
Variable inputs for the currently selected template now survive same-template
query refreshes and template updates instead of being reset to defaults on every
refetch, which keeps repeated apply/update work usable during a compose session.
The same surface now exposes the existing bounded `/templates/rich/mail-merge-preview`
API as a JSON-row preview tool with ready/blocked counts and per-row rendered
subject/body output, so this template preview capability is no longer API-only.
The Template Library save flow now supports explicit save-copy management: the
picker can open a duplicate-aware save form with a suggested `"<template> copy"`
name derived from the selected template, while plain save continues to suggest
the current compose subject when available.
The Template Library live compose path was browser-verified with a saved
reusable template, the compose-side mail-merge preview panel, category filters
in the library, and a recipient-mapping panel that fills mapped template
variables from current compose recipients and can build preview rows from the
current `To` list.
Full send-orchestration mail merge, external recipient import sources and full
template library/editor UX remain blockers.

**§4.5 / Organization**: Local custom folders now use a shared hierarchy-aware
presentation path for slash-delimited names such as `Projects / Client A / Q1`.
The strip derives depth/leaf/prefix from the path and reuses the same ordered
folder rows for visible rendering and drag/drop reordering, so the hierarchy the
user sees is the hierarchy the local reorder workflow operates on. Folder
create/edit now split hierarchy entry into parent-path suggestions plus a leaf
folder name with full-path preview, so users do not need to hand-type the whole
slash path to keep nested local folders consistent. This Folder Hierarchy path
was browser-verified in the live `New folder` dialog with `Parent folder`,
`Folder name` and `Full path`. The strip now also supports quick child-folder
creation from an existing folder row, pre-filling the parent path from the
selected folder, and delete confirmation now warns when descendant path rows
will keep their existing full names after the selected parent path is removed.
Saved searches and smart folders now expose an
explicit `Match` mode in the Rules Builder (`all` / `any`), and durable
`message_count` values are computed from the same parsed search semantics as
runtime message search, including field rules such as `subject:`, `body:` and
`from:`. The Rules Builder now tolerates unsaved invalid draft state while
rendering active chips, so opening `New saved search` no longer crashes on an
empty required `name`. Browser validation also confirmed that the live `New
saved search` dialog now opens with `Name`, `Query`, `Rules`, `Effective
query` and `Cancel`. The frontend Rules Builder now has a recursive visual
group editor with nested `All conditions` / `Any condition` groups, while the
mail search parser and dynamic SQL path now understand explicit parenthesized
`AND` / `OR` expressions such as `(subject:quarterly OR body:invoice) AND
sender:alex`. The nested builder header now surfaces explicit depth and
structure cues such as `Root group`, `Group 2`, current match mode and
rule/nested-group counts, so visual scanning of complex boolean trees no longer
depends only on the field rows themselves. Frontend unit/boundary tests and backend parser unit tests cover
this path; the Postgres-backed saved-search count integration tests are present
but could not run in the current sandbox because testkit could not start its
Docker PostgreSQL container.

**§16 (75%)**: Durable `email_outbox_tracking` foundation exists with queued,
scheduled, sending, sent, failed and canceled states. `/send` can enqueue
scheduled/undoable messages without immediate SMTP delivery, and immediate
Gmail sends now use the Gmail API when the account has `gmail.send` OAuth scope
enabled. A backend runtime scheduler runs the domain delivery worker, skips
claims while the host vault is locked, resolves account-scoped `smtp_password`
credentials or Gmail OAuth tokens, and sends due IMAP/iCloud/Gmail outbox items
through provider-aware transport. The worker marks sent, failed or
retry-scheduled with exponential backoff and appends canonical delivery events.
The delivery-notification API now parses DSN delivery-status reports into sanitized
`mail.outbox.delivery_status_changed` events and parses MDN displayed
notifications into durable read receipts. Read receipt records persist provider
evidence, correlate to sent outbox records by provider message id and append
sanitized `mail.read_receipt.recorded` events. Outbox list responses are
cursor-paginated, enrich metadata with sanitized latest-read evidence, and the
Communications UI surfaces queued/scheduled/retry/failed/delivered/read status
through a compact infinite-load query-backed status strip with undo-send actions.
A protected structured
`/provider-delivery-events` path now lets internal provider runtimes report
delivered/delayed/failed/read events without synthesizing raw DSN/MDN payloads.
Missing: external provider webhook/subscription integration and richer delivery
UX.

**§29 (35%)**: Bounded local bulk actions exist for message workflow state,
local trash/restore and local metadata flags/labels through
`POST /api/v1/communications/messages/bulk-actions`. The Vue/TanStack Query
bulk-action mutation now applies optimistic list/detail cache updates and rolls
them back on mutation failure before final invalidation. The selected-message
toolbar exposes read/unread, archive, trash, pin/unpin, important/normal,
Follow up label/unlabel and snooze commands through that bulk mutation. The
single-message Related tab now exposes local label add/remove and quick snooze
actions through Mail action mutations and message-detail refresh. Missing:
provider-side batch mutations, queued long-running bulk jobs, progress events,
optimistic
coverage for remaining mail mutations and richer multi-select keyboard
workflows. The mail list supports shift-click range selection for visible
messages plus keyboard Space toggle, Shift+Arrow range extension,
Ctrl/Cmd+A visible select-all and Escape clear selection.

**§23/31**: Saved searches and smart-folder definitions now have durable
PostgreSQL persistence, canonical event-log records, local API routes and
Vue/TanStack Query hooks. The mail UI can apply saved definitions from the list
strip and create/edit/delete saved searches or smart folders through
Zod/Vee Validate-backed dialogs. Creating a saved search or smart folder from
the Mail page now pre-fills the current text query, workflow state, local state
and channel context. The list API and strip UI expose derived per-definition
message counts. The Rules Builder now includes explicit `Match` mode,
effective-query preview, nested `All conditions` / `Any condition` groups,
depth/summary cues such as `Root group` and backend search/count execution
aligned with the same parsed parenthesized boolean semantics. Remaining work is
around broader provider-backed search validation and deeper saved-search
management UX.

**§4.5/31**: Local custom folders now have durable PostgreSQL persistence,
cursor-paginated folder/message APIs, canonical event-log records and
Vue/TanStack Query hooks for folder list, folder messages, create/update/delete,
copy and move. Folder list responses expose derived per-folder message counts.
The frontend exposes a Zod/Vee Validate-backed local folder create/edit/delete
strip and folder selection can browse cursor-paginated folder messages through
the existing virtualized mail list. Selected messages can be dropped on folder
chips for local move, with Alt-drop preserving a copy. Local custom folder chips
can now be reordered by drag/drop through the existing folder update mutation
and `sort_order` persistence. The Folder Hierarchy flow now includes slash-path
presentation, parent-path suggestions with full-path preview, quick
create-child actions from existing folder rows and descendant-aware delete
warnings. Missing: provider-side Gmail/IMAP folder synchronization and richer
provider-mapped folder management.

**§30 (55%)**: Экспорт EML/MD/JSON работает через API и Related tab UI:
пользователь выбирает формат, page controller выполняет TanStack mutation,
сохраняет последний export response и ActionBar показывает download link.
Импорт UI — фронтенд.

**§31 (70%)**: Compose, Zod/Vee Validate send validation, Zod/Vee saved-search
create/edit validation, Zod/Vee custom folder create/edit validation, Zod/Vee
account setup validation with real Gmail OAuth start and IMAP/iCloud setup API
calls, аналитика, фильтры, AI-карточка, saved-search strip, local folder
management strip, virtualized folder-message browsing, selected-message
drop-to-folder move/copy, selected-message
drag/drop to bulk actions and message-detail prefetch on mail-list hover/focus
work. The ActionBar now includes a Mail Resource Overview strip backed by
TanStack infinite queries for cursor-paginated subscriptions/newsletter sources
and top senders, with virtualized compact lists and explicit load-more; current
mail architecture blockers remain a fixed reference query. Thread rows now prefetch thread messages, attachment search rows prefetch parent message detail and saved-search/smart-folder
chips prefetch the first matching mail-list page into the shared TanStack Query
cache on hover/focus before opening the conversation timeline or applying a
saved filter, and the compact outbox strip prefetches the next delivery page
before explicit load-more. Drafts now use backend cursor pagination, a TanStack
infinite query and a virtualized ActionBar strip with explicit load-more, so
typed `subject:`, `body:`, `from:` and `mode:any` tokens in the saved-search
Query field normalize back into the Rules Builder on blur/save instead of
drifting away from the visible rule rows and persisted search definition.
The dialog now also shows the effective normalized query string before save, so
the user can see the exact persisted Mail search expression assembled from plain
text, rules and match mode.
It now blocks save when the builder still contains empty rows or duplicate
rules, giving explicit validation copy instead of silently dropping invalid rule
rows during query composition.
large draft sets no longer require loading or rendering the whole list. Mail query hooks now use shared explicit background
refetch policies for realtime lists/statuses, detail records and reference data
instead of relying on implicit TanStack defaults. Selected-message side effects
now live in `useSelectedMessageActions.ts`, keeping the page controller focused
on orchestration and safely below the frontend architecture size guard. The
frontend line-count guard now includes test files as well as production source,
and oversized realtime/API tests were split so current frontend source remains
below the 700-line failure threshold. Backend/repository-wide size guards remain future work. Local folder drag/drop
reordering and the attachment
metadata search panel work. Account setup now uses the shared Reka/Radix-backed Dialog
primitive and compose now uses the shared Reka/Radix-backed Sheet primitive instead
of custom modal/drawer overlays. Треды UI, nested/grouped folder organization,
редакторы правил/шаблонов and remaining custom overlay conversion — фронтенд.
