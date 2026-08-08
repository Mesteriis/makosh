# Email Channel — Architectural Blockers

Явно задокументированные блокеры с причинами и планом решения.
API: `GET /api/v1/communications/blockers`

These blockers apply to the current email-channel implementation. Cross-channel
Communications, Obligations, Decisions and Polygraph work is tracked in
`../../refactoring/implementation-alignment-plan.md`.

## 1. §8 — Безопасность вложений (sandbox, антивирус)

**Текущий статус**: Mail projection runs a conservative heuristic attachment
safety scanner and a timeout-bounded local ClamAV `INSTREAM` client. It can mark
obvious executable payload magic bytes, active-content extensions, macro-enabled
Office extensions and known MIME/filename mismatches as `malicious` or
`suspicious`; unmatched or unavailable-scanner cases remain `not_scanned`, so
Макошь does not mark an attachment `clean` without a real scanner verdict. The
local Compose environment also runs an isolated PDF/DOCX/OCR worker: it has a
read-only blob mount, no external route, dropped Linux capabilities, a
read-only root filesystem and bounded CPU/memory/process/temp-storage.

**Остаётся**: CDR, full OLE stream analysis, sandboxed nested archive parsing,
broader OCR language packs and safe bitmap previews for non-PDF rich documents.

**План**: Keep heuristic scanning as a prefilter/fallback and only replace
`not_scanned` with `clean` when ClamAV produced the verdict. Bounded CFB/OLE
directory inspection detects macro storage names without opening a content stream. Add CDR and
full document/archive analysis behind the same worker boundary without relaxing its
mount, network or resource limits.

## 2. §12 — Криптографическая верификация подписей

**Причина**: Требует OpenSSL, GPG, КриптоПро SDK. Это внешние нативные библиотеки (C/C++), не Rust-крейты. Нужна отдельная интеграционная работа с FFI или вызовом CLI.

**План**: Создать `email_crypto` модуль с привязкой к OpenSSL/GPG. Сертификаты из macOS Keychain читать через Security framework. ГОСТ-подписи — через КриптоПро CLI или отдельный микросервис.

## 3. §16-17 — Outbox tracking и Follow-up engine

**Причина**: Durable outbox tracking, the domain delivery worker, retry/backoff
handling, backend runtime scheduling, account-scoped SMTP sender wiring, Gmail
OAuth send scopes, immediate and scheduled Gmail API send, sanitized DSN
delivery-status ingestion, MDN read-receipt ingestion, latest-read outbox
metadata enrichment and a compact query-backed delivery/read status strip now
exist. A protected structured provider-runtime callback path now records
delivered/delayed/failed/read events through the same stores. Production delivery
tracking still requires external provider webhook/subscription wiring and richer
provider-specific delivery UX. This remains an asynchronous event-driven flow.

**План**: Connect external provider webhook/subscription sources to the
structured provider-delivery event path and expand delivery/read status UX beyond
the compact outbox strip.

## 4. §28-29 — Интеграции и provider-side массовые действия

**Причина**: Каждая интеграция (Jira, YouTrack, Google Calendar, Apple Notes, Obsidian) — отдельный коннектор со своим API и аутентификацией. Local bounded bulk actions exist, but provider-side batch mutations, long-running jobs and progress events still require queues.

**План**: Реализовать интеграции как plugin-коннекторы по образцу существующих Telegram/WhatsApp модулей. Provider-side массовые действия — через фоновые задачи projection runner with progress events.

## 5. §8.2 — Безопасная распаковка архивов

**Текущий статус**: Bounded ZIP metadata inspection exists in the mail domain
with limits for archive size, uncompressed size, entry count, path depth and
path traversal. The protected attachment API can inspect a known local ZIP blob,
and the message-detail attachment table exposes an inspection action. It does
not extract files to disk.

**Остается**: Persisted inspection results, nested archive policy, RAR/7z
support and any future extraction workflow.

**План**: Persist sanitized inspection metadata, define nested archive policy,
then add RAR/7z support behind the same limits if product scope still requires
it.

## 6. §9.3 — OCR (распознавание текста)

**Текущий статус**: The isolated local worker uses Tesseract with the bundled
English language pack for clean supported image attachments. OCR remains local;
no attachment bytes are sent to an external OCR service.

**Остаётся**: Additional language packs, image pre-processing and durable
preview artifacts.

**План**: Add approved local language packs and bounded image pre-processing to
the existing container image; keep external OCR disabled unless an account
content-egress policy explicitly permits it.

## Не-блокеры (не входят в scope email-модуля)

Следующие разделы спецификации не являются частью email-модуля и реализуются отдельно:

- **Exchange/Fastmail/Proton/Maildir адаптеры** (§3) — отдельные provider adapter'ы
- **Rich-редактор шаблонов в UI** (§31) — задача фронтенда, API готово
- **Импорт EML/MBOX через UI** (§30) — задача фронтенда, бекенд готов
- **Undo-send runtime UX** (§4.2) — depends on remaining §16 delivery-status
  work and user-facing timing/notification UX
