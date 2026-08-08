# Email Channel — Current Module Map

This file maps current email implementation modules. In the canonical domain
model, these modules belong to the Communications domain and shared engine
pipeline; they do not make Mail a product identity or standalone top-level
application.

Paths below refer to the current Rust implementation under
`backend/src/domains/communications/` unless another path is shown.

## Core Pipeline

| Модуль | Файл | Назначение |
|---|---|---|
| Email Ingestion | `ingestion.rs` | Точка входа авто-анализа. Каждое входящее письмо проходит через Макошь. |
| Email Sync | `sync.rs`, `background_sync.rs` | Планирование синхронизации (Gmail/IMAP/iCloud) |
| Email Sync Pipeline | `backend/src/workflows/email_sync_pipeline.rs` | Полный пайплайн: парсинг → проекция → анализ → вложения |
| Email RFC822 | `rfc822.rs` | Парсинг RFC 2822 / MIME |
| Email Import | `import.rs` | Импорт фикстур |
| Email Fixture Pipeline | `fixtures/pipeline.rs` | Пайплайн для тестовых данных |
| Email Fixture Export | `fixtures/export.rs` | Экспорт фикстур |
| Messages | `messages.rs` | Проекция сообщений. 8 workflow-состояний, AI-поля, фильтры |

## Intelligence & Analysis

| Модуль | Назначение |
|---|---|
| `backend/src/workflows/email_intelligence.rs` | AI-анализ: скоринг (0-100), 13 категорий, LLM-классификация |
| `spf_dkim.rs` | SPF/DKIM/DMARC парсинг заголовков, оценка риска спуфинга |
| `explain.rs` | "Почему это письмо важно?" + умные подсказки CC |
| `multilingual.rs` | Определение языка (ru/en/es/de/uk/zh), перевод через LLM |
| `extract.rs` | Извлечение задач и заметок из письма (LLM + эвристики) |

## Organization & Workflow

| Модуль | Назначение |
|---|---|
| `threads.rs` | Группировка писем в треды, attention metrics |
| `flags.rs` | Pin / Snooze / Label / Mute — флаги в JSONB метаданных |
| `subscriptions.rs` | Детектор рассылок, поиск unsubscribe |
| `analytics.rs` | Attention analytics for mailbox-like channel views and top senders |

## Outgoing

| Модуль | Назначение |
|---|---|
| `send.rs` | SMTP клиент (EHLO, AUTH LOGIN, MAIL FROM, RCPT TO, DATA) |
| `drafts.rs` | Черновики: 5 статусов, авто-устаревание |
| `actions.rs` | Reply/Forward/reply-all с цитированием, EML-forward |
| `ai_reply.rs` | AI генератор ответов: выбор тона и языка, варианты |
| `templates.rs` | Шаблоны с подстановкой переменных |
| `rich_template.rs` | Rich-шаблоны: conditional, table, button, divider |
| `personas.rs` | Channel sending personas for an account; not the canonical Persona domain |

## Documents & Finance

| Модуль | Назначение |
|---|---|
| `finance.rs` | Счета: 7 статусов, суммы, валюты, контрагенты |
| `legal.rs` | Юрдокументы: 11 типов (contract/NDA/MSA/DPA/...), 6 статусов |

## Security & Trust

| Модуль | Назначение |
|---|---|
| `signatures.rs` | Сертификаты: 7 типов, 9 провайдеров, детекция S/MIME+PGP |
| `backend/src/domains/documents/attachment_intelligence.rs` | Классификация вложений: 15 категорий, уровни риска |
| `storage/scanner.rs` | Conservative attachment safety prefilter: executable magic, active-content extensions, macro-enabled document extensions and MIME/filename mismatch; does not emit `clean` without a real scanner backend |
| `attachment_dedup.rs` | Поиск дубликатов по SHA-256 + похожие имена |

## Automation

| Модуль | Назначение |
|---|---|
| `rules.rs` | Channel rule execution: field/operator/value, 4 execution modes |

## Search & Export

| Модуль | Назначение |
|---|---|
| `search.rs` | Мост к Tantivy: индексация и поиск email-backed communications |
| `export.rs` | Экспорт source message в EML / Markdown / JSON |

## Provider Networking

| Модуль | Назначение |
|---|---|
| `backend/src/integrations/mail/gmail/client.rs`, `accounts.rs` | Gmail API + account metadata |
| `imap_write.rs` | IMAP write операции (STORE, EXPUNGE) |
| `accounts.rs` | Настройка аккаунтов: Gmail OAuth, IMAP, шифрование |

## Support

| Модуль | Назначение |
|---|---|
| `sources.rs` | Типы источников писем |
| `storage.rs` | Blob-хранилище вложений (LocalFS) |
| `blockers.rs` | Документирование архитектурных блокеров |
