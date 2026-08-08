---
chunk_id: 028-source-backend-part-008
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 028-source-backend-part-008 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Чанк добавляет в русскую Obsidian‑wiki документацию по обработчикам (handlers) коммуникационного домена (`communications`) серверной части `makosh‑hub`. На основе встроенных исходных файлов описаны эндпоинты, модели запросов/ответов, ключевые механики безопасности (SSRF‑защита для удалённых изображений) и интеграция AI‑ответов. Страница `components/backend.md` обновляется и дополняется этими сведениями.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend (Серверная часть)

## Коммуникационные обработчики (Communications Handlers)

Модуль `backend/src/app/handlers/communications/` группирует все HTTP‑обработчики,
относящиеся к работе с коммуникациями (почта, сообщения).
Согласно комментарию в `mod.rs` (ссылка на ADR‑0073), обработчики разнесены
по limited‑контекстам: **communications**, **accounts**, **workflow**.

### Маршруты (Routes)

| Метод | Путь | Обработчик | Назначение |
|-------|------|------------|------------|
| POST | `/v1/attachment/import` | `post_v1_attachment_import` | Импорт вложения из base64 |
| GET | `/v1/outbox` | `get_v1_outbox` | Список исходящих с фильтрами |
| POST | `/v1/outbox/{outbox_id}/undo` | `post_v1_outbox_undo` | Отмена отправки |
| GET | `/v1/personas` | `get_v1_personas` | Список персон (personas) |
| POST | `/v1/persona` | `post_v1_persona` | Создание / обновление персоны |
| POST | `/v1/read_receipt` | `post_v1_read_receipt` | Запись read‑receipt сообщения |
| POST | `/v1/delivery_notification` | `post_v1_delivery_notification` | Уведомление о доставке письма |
| POST | `/v1/provider_delivery_event` | `post_v1_provider_delivery_event` | Событие доставки от провайдера |
| GET | `/v1/saved_searches` | `get_v1_saved_searches` | Список сохранённых поисков |
| POST | `/v1/saved_search` | `post_v1_saved_search` | Создание сохранённого поиска |
| PUT | `/v1/saved_search/{id}` | `put_v1_saved_search` | Обновление сохранённого поиска |
| DELETE | `/v1/saved_search/{id}` | `delete_v1_saved_search` | Удаление сохранённого поиска |
| GET | `/v1/email/search` | `get_v1_email_search` | Поиск по письмам |
| GET | `/v1/threads` | `get_v1_threads` | Список цепочек писем (threads) |
| GET | `/v1/thread_messages` | `get_v1_thread_messages` | Письма в конкретной цепочке |
| GET | `/v1/analytics/health` | `get_v1_analytics_health` | Здоровье почтового ящика |
| GET | `/v1/analytics/senders` | `get_v1_analytics_senders` | Статистика по отправителям |
| GET | `/v1/message/{id}/explain` | `get_v1_message_explain` | Объяснение важности письма |
| GET | `/v1/message/{id}/smart_cc` | `get_v1_message_smart_cc` | Умные предложения CC |
| GET | `/v1/invoices` | `get_v1_invoices` | Список инвойсов |
| POST | `/v1/invoice` | `post_v1_invoice` | Создание / обновление инвойса |
| GET | `/v1/legal_docs` | `get_v1_legal_docs` | Список юридических документов |
| POST | `/v1/legal_doc` | `post_v1_legal_doc` | Создание / обновление юр. документа |
| GET | `/v1/message/{id}/export` | `get_v1_message_export` | Экспорт письма (eml/json/md) |
| POST | `/v1/send` | `post_v1_send` (из `sending`) | Отправка письма |
| POST | `/v1/messages/bulk_action` | `post_v1_messages_bulk_action` | Групповое действие над письмами |
| POST | `/v1/message/{id}/pin` | `post_v1_message_pin` | Закрепить / открепить письмо |
| POST | `/v1/message/{id}/important` | `post_v1_message_important` | Пометить важным / снять пометку |
| POST | `/v1/message/{id}/snooze` | `post_v1_message_snooze` | Отложить письмо до указанного времени |
| POST | `/v1/message/{id}/mute` | `post_v1_message_mute` | Заглушить / снять глушение |
| POST | `/v1/message/{id}/label` | `post_v1_message_label` | Добавить ярлык |
| DELETE | `/v1/message/{id}/label` | `delete_v1_message_label` | Удалить ярлык |
| GET | `/v1/subscriptions` | `get_v1_subscriptions` | Обнаруженные подписки |
| GET | `/v1/attachment_duplicates` | `get_v1_attachment_duplicates` | Дубликаты вложений |
| GET | `/v1/message/{id}/ai_state` | `get_v1_message_ai_state` | Текущее AI-состояние письма |
| PUT | `/v1/message/{id}/ai_state` | `put_v1_message_ai_state` | Переход AI-состояния |
| GET | `/v1/communication/message/{id}/remote_image` | `get_v1_communication_message_remote_image` | Прокси‑загрузка удалённого изображения из письма |
| POST | `/v1/ai_reply` (предположительно под `sending`) | `post_v1_ai_reply` | Генерация AI‑ответа на письмо |
| POST | `/v1/ai_reply_variants` | `post_v1_ai_reply_variants` | Варианты AI‑ответа (языки/тоны) |

Также модуль `sending` реэкспортирует дополнительные обработчики, не раскрытые в данном контексте:
- `post_v1_forward`, `post_v1_forward_eml`, `post_v1_redirect`, `post_v1_reply`, `post_v1_reply_all` — пересылка и ответы.
- `post_v1_translate`, `post_v1_translate_attachment`, `post_v1_translate_thread`, `get_v1_detect_language` — переводы и определение языка.
- `get_v1_certs`, `get_v1_certs_expiring`, `get_v1_signature_check`, `get_v1_spf_dkim`, `post_v1_cert` — работа с сертификатами и проверками подлинности.
- `post_v1_extract_notes`, `post_v1_extract_tasks` — извлечение заметок и задач.
- `post_v1_imap_delete`, `post_v1_imap_mark_read`, `post_v1_message_restore`, `post_v1_message_trash` — действия над письмами на IMAP‑уровне.
- `post_v1_bilingual_reply_flow` — двуязычный поток ответа.

### Импорт вложений (`imports.rs`)

**Запрос:** `CommunicationAttachmentImportRequest`
- `account_id`, `channel_kind`, `filename`, `content_type`, `source_kind`, `metadata` — опциональны.
- `content_base64` — обязательное поле.

**Ответ:** `CommunicationAttachmentImportResponse`
- Содержит идентификатор вложения (`attachment_id`), `blob_id`, `filename`, `content_type`, `size_bytes`, `sha256`, `scan_status`, `storage_kind`, `storage_path`.

### Очередь исходящих (`outbox.rs`)

**Запрос списка (Query):** `OutboxListQuery`
- `account_id`, `status` (строка, парсится в `CommunicationOutboxStatus`), `cursor`, `limit` (по умолчанию 100).

**Отмена отправки:** `POST /v1/outbox/{outbox_id}/undo`
- Возвращает обновлённый объект `CommunicationOutboxItem`.

### Персоны (`personas.rs`)

**Список:** `GET /v1/personas` возвращает все персоны.

**Создание/обновление:** `POST /v1/persona` принимает `NewPersonaRequest`:
- Обязательные: `persona_id`, `name`, `account_id`, `display_name`.
- Опционально: `signature`, `default_language`, `default_tone`, `is_default`, `metadata`.
- `signature` по умолчанию — пустая строка, `metadata` — `{}`.

### Read‑receipts и уведомления о доставке (`read_receipts.rs`)

- `POST /v1/read_receipt` — запись флага прочтения.
- `POST /v1/delivery_notification` — принимает `NewCommunicationDeliveryNotification`, преобразуется в `NewProviderDeliveryEvent` через `provider_event_from_delivery_notification`, далее диспатчится через Signal Hub.
- `POST /v1/provider_delivery_event` — прямой приём события доставки от провайдера.
- Обработка событий: если `event_kind == Read`, то `event_kind` сигнала — `"read_receipt"`; иначе — `"delivery_status"`. Далее вызывается `dispatch_mail_delivery_event_signal`, при успехе — `project_accepted_mail_delivery_signal_if_runtime_allows`.

### Сохранённые поиски (`saved_searches.rs`)

- `GET /v1/saved_searches` — пагинированный список (по умолчанию `limit = 500`). Поддерживает фильтр `smart_folder`.
- `POST /v1/saved_search` — создание.
- `PUT /v1/saved_search/{id}` — обновление; возвращает `404`, если объект не найден.
- `DELETE /v1/saved_search/{id}` — удаление.

### Поиск по письмам (`search.rs`)

- `GET /v1/email/search` — параметры: `q` (обязательный, иначе ошибка), `limit` (по умолчанию 20).
- Если установлена переменная окружения `MAKOSH_SEARCH_INDEX_PATH`, используется файловый поисковый индекс (`SearchIndex`). В противном случае возвращается пустой результат.
- Перед поиском вызывается `index_messages` с лимитом 100 для пополнения индекса.

### Цепочки писем (`threads.rs`)

- `GET /v1/threads` — пагинированный список цепочек (`limit` по умолчанию 50).
- `GET /v1/thread_messages` — сообщения в цепочке, обязательны query-параметры `account_id` и `subject`.

### Финансовая аналитика (`finance_analytics/`)

Включает модули:
- `analytics.rs`: `GET /v1/analytics/health` (здоровье ящика, `MailboxHealth`), `GET /v1/analytics/senders` (статистика по отправителям, пагинация, по умолчанию `limit = 20`).
- `explain.rs`: `GET /v1/message/{id}/explain` — причины важности письма; `GET /v1/message/{id}/smart_cc` — список предлагаемых получателей копии.
- `invoices.rs`: `GET /v1/invoices` (фильтр по `status`), `POST /v1/invoice` (создание/обновление инвойса). Статус по умолчанию — `Received`.
- `models.rs`: общие типы ответов (`PinToggleResponse`, `ImportantToggleResponse`), видимые только внутри `handlers::communications`.

### Юридические документы и экспорт (`legal_export.rs`)

- `GET /v1/legal_docs` — список документов с фильтрами `document_type` и `status` (парсятся в `LegalDocType` / `LegalDocStatus`).
- `POST /v1/legal_doc` — создание. Тип по умолчанию `Other`, статус — `Draft`.
- `GET /v1/message/{id}/export?format=` — экспорт письма: `eml`, `json`, `markdown` (по умолчанию `markdown`).
- Также в этом файле определены (но обработчик не показан в этом чанке) модели `SendRequest` и `SendResponse`, используемые при отправке письма.

### Действия над письмами (`message_actions.rs`)

- **Групповые действия:** `POST /v1/messages/bulk_action` принимает `BulkMessageActionRequest`:
  - Поддерживаемые действия: `mark_read`, `mark_unread`, `archive`, `trash`, `restore`, `pin`, `unpin`, `important`, `not_important`.
  - `add_label` / `remove_label` требуют поле `label`.
  - `snooze` требует `snooze_until` — парсится в `DateTime<Utc>`.
- **Индивидуальные операции:**
  - `POST /v1/message/{id}/pin` — переключение закрепления.
  - `POST /v1/message/{id}/important` — переключение важности.
  - `POST /v1/message/{id}/snooze` — отложить до указанного времени (поле `until`).
  - `POST /v1/message/{id}/mute` — переключение глушения (возвращает `PinToggleResponse`, где поле `pinned` отражает состояние `muted`).
  - `POST /v1/message/{id}/label` — добавить ярлык.
  - `DELETE /v1/message/{id}/label` — удалить ярлык.
- **Подписки:** `GET /v1/subscriptions` — обнаруженные источники подписок (пагинация, `limit` по умолчанию 50).
- **Дубликаты вложений:** `GET /v1/attachment_duplicates` — группы дубликатов (по умолчанию `limit = 20`).
- Так же в файле объявлены query‑модели `LegalDocQuery` и `LegalDocListResponse`, используемые эндпоинтом юридических документов (обработчик в `legal_export.rs`).

### AI‑состояние сообщения (`message_ai_state.rs`)

- `GET /v1/message/{id}/ai_state` — текущее состояние; возвращает `404`, если запись отсутствует.
- `PUT /v1/message/{id}/ai_state` — переход состояния через `CommunicationAiStateTransitionRequest`.

### Удалённые изображения (`remote_images/`)

Механизм проксированной загрузки изображений из HTML‑тела письма с защитой от SSRF.

**Основные компоненты:**
- `url_policy.rs`: валидация URL (длина ≤ 4096 байт, только http/https, запрет localhost, `.local`, private‑IP, loopback и т.д.). Функция `is_public_ip` классифицирует IPv4/IPv6 адреса.
- `dns.rs`: разрешение публичных IP‑адресов через Google DNS API (тип A). Если адресов нет — ошибка `NoPublicAddress`.
- `fetcher.rs`: клиент с таймаутом 15 сек, лимит на тело ответа 12 МБ. Сначала делается запрос напрямую; при HTTP‑ошибке — повтор с DNS‑резолвингом и resolve‑to‑addrs.
- `reference.rs`: проверка, что URL действительно присутствует в HTML‑теле письма (учитываются экранированные амперсанды).
- `handler.rs`: конечная точка `GET /v1/communication/message/{id}/remote_image`. Проверяет наличие сообщения и HTML‑тела, затем вызывает `fetch_remote_image` и возвращает изображение с заголовками `Content-Type`, `Cache-Control: private, max-age=600` и `Referrer-Policy: no-referrer`.
- `errors.rs`: ошибки преобразуются в `ApiError::InvalidCommunicationQuery` с конкретным сообщением.

### Отправка и ответы (`sending/`)

Модуль реэкспортирует обработчики для отправки, пересылки, ответов, перевода, сертификатов и действий на IMAP.

**AI‑reply (`sending/ai_reply.rs`)**
- `POST /v1/ai_reply` — принимает `AiReplyRequest` с опциональными `tone`, `language`, `context`. Генерирует черновик ответа через `EmailAiReplyService`. При успешной генерации отправляет сигнал `ai_helper_signal` с operation = `reply_drafting`. Если LLM не настроен — возвращает `{"generated": false, "reason": "no LLM configured"}`.
- `POST /v1/ai_reply_variants` — принимает `AiReplyVariantsRequest` с опциональными списками `languages` (по умолчанию `["en", "es", "ru"]`) и `tones` (по умолчанию `["professional", "friendly"]`). При наличии вариантов отправляет сигнал `ai_helper_signal` с operation = `reply_variant_generation`.
```

## Покрытие источников

- `backend/src/app/handlers/communications/communication_queries/imports.rs`
  - Структура запроса `CommunicationAttachmentImportRequest` и ответа `CommunicationAttachmentImportResponse`.
  - Обработчик `post_v1_attachment_import`, вызывающий `CommunicationCommandService::import_attachment`.

- `backend/src/app/handlers/communications/communication_queries/outbox.rs`
  - Query-параметры `OutboxListQuery`, парсинг статуса в `CommunicationOutboxStatus`.
  - Обработчик `get_v1_outbox` с `limit` по умолчанию 100.
  - Обработчик `post_v1_outbox_undo` — отмена отправки.

- `backend/src/app/handlers/communications/communication_queries/personas.rs`
  - `PersonaListResponse` и `NewPersonaRequest`.
  - Обработчики `get_v1_personas` (список всех персон) и `post_v1_persona` (upsert).

- `backend/src/app/handlers/communications/communication_queries/read_receipts.rs`
  - Обработчики `post_v1_read_receipt`, `post_v1_delivery_notification`, `post_v1_provider_delivery_event`.
  - Логика преобразования delivery notification в provider event, диспатч через Signal Hub, классификация `event_kind` в `"read_receipt"` / `"delivery_status"`.

- `backend/src/app/handlers/communications/communication_queries/saved_searches.rs`
  - Query-модель `SavedSearchesQuery`, ответ `SavedSearchListResponse`, `SavedSearchDeleteResponse`.
  - CRUD-обработчики, `limit` по умолчанию 500.

- `backend/src/app/handlers/communications/communication_queries/search.rs`
  - `EmailSearchQuery` (обязательный `q`).
  - Переменная окружения `MAKOSH_SEARCH_INDEX_PATH` управляет включением поиска.
  - Вызов `search_emails` с лимитом по умолчанию 20.

- `backend/src/app/handlers/communications/communication_queries/threads.rs`
  - `ThreadListQuery` и `ThreadMessagesQuery` (обязательные `account_id`, `subject`).
  - `limit` по умолчанию 50.

- `backend/src/app/handlers/communications/finance_analytics.rs`
  - Модульная декларация: `analytics`, `explain`, `invoices`, `models`.

- `backend/src/app/handlers/communications/finance_analytics/analytics.rs`
  - Обработчики `get_v1_analytics_health` и `get_v1_analytics_senders` (`limit` по умолчанию 20).

- `backend/src/app/handlers/communications/finance_analytics/explain.rs`
  - `get_v1_message_explain` (причины важности) и `get_v1_message_smart_cc` (предложения CC).

- `backend/src/app/handlers/communications/finance_analytics/invoices.rs`
  - CRUD-запросы инвойсов: `get_v1_invoices` (фильтр по статусу), `post_v1_invoice` (статус по умолчанию `Received`).

- `backend/src/app/handlers/communications/finance_analytics/models.rs`
  - Типы ответов `PinToggleResponse` и `ImportantToggleResponse`.

- `backend/src/app/handlers/communications/legal_export.rs`
  - Обработчики `get_v1_legal_docs` и `post_v1_legal_doc`, значения по умолчанию для типа (`Other`) и статуса (`Draft`).
  - Экспорт сообщения (`get_v1_message_export`): форматы `eml`, `json`, `markdown`.
  - Модели `SendRequest` и `SendResponse`.

- `backend/src/app/handlers/communications/message_actions.rs`
  - `BulkMessageActionRequest` и все поддерживаемые действия, включая `add_label`, `remove_label`, `snooze` с обязательными полями.
  - Индивидуальные обработчики: `pin`, `important`, `snooze`, `mute`, `label`, `delete_label`.
  - Обработчики `get_v1_subscriptions` (limit 50) и `get_v1_attachment_duplicates` (limit 20).

- `backend/src/app/handlers/communications/message_ai_state.rs`
  - `get_v1_message_ai_state` (возвращает 404 при отсутствии записи), `put_v1_message_ai_state`.

- `backend/src/app/handlers/communications/mod.rs`
  - Полный список модулей, импорты, используемые типы и зависимости.
  - Декларация маршрутизации через `axum::Router`.

- `backend/src/app/handlers/communications/remote_images.rs`
  - Декларация подмодулей: `dns`, `errors`, `fetcher`, `handler`, `reference`, `url_policy`.

- `backend/src/app/handlers/communications/remote_images/dns.rs`
  - DNS‑резолвинг через Google DNS API, фильтрация публичных адресов.

- `backend/src/app/handlers/communications/remote_images/errors.rs`
  - Варианты ошибок и их маппинг в `ApiError`.

- `backend/src/app/handlers/communications/remote_images/fetcher.rs`
  - Клиент с таймаутом 15 сек, лимитом тела 12 МБ, повторная попытка с DNS‑оверрайдом.

- `backend/src/app/handlers/communications/remote_images/handler.rs`
  - Обработчик `get_v1_communication_message_remote_image`, проверка ссылки через `message_html_references_url`, заголовки кеша и referrer.

- `backend/src/app/handlers/communications/remote_images/reference.rs`
  - Логика поиска URL в HTML с учётом экранированных амперсандов. Тесты.

- `backend/src/app/handlers/communications/remote_images/url_policy.rs`
  - Валидация URL: схема, localhost, private IP, `is_public_ip`. Тесты.

- `backend/src/app/handlers/communications/sending.rs`
  - Декларация подмодулей и реэкспорт обработчиков: `ai_reply`, `bilingual_reply_flow`, `certificates`, `extraction`, `forwarding`, `local_state`, `multilingual`, `provider_send`.

- `backend/src/app/handlers/communications/sending/ai_reply.rs`
  - Обработчики `post_v1_ai_reply` и `post_v1_ai_reply_variants`, сигналы `ai_helper_signal`, поведение при отсутствии LLM.

## Исходные файлы

- [`backend/src/app/handlers/communications/communication_queries/imports.rs`](../../../../backend/src/app/handlers/communications/communication_queries/imports.rs)
- [`backend/src/app/handlers/communications/communication_queries/outbox.rs`](../../../../backend/src/app/handlers/communications/communication_queries/outbox.rs)
- [`backend/src/app/handlers/communications/communication_queries/personas.rs`](../../../../backend/src/app/handlers/communications/communication_queries/personas.rs)
- [`backend/src/app/handlers/communications/communication_queries/read_receipts.rs`](../../../../backend/src/app/handlers/communications/communication_queries/read_receipts.rs)
- [`backend/src/app/handlers/communications/communication_queries/saved_searches.rs`](../../../../backend/src/app/handlers/communications/communication_queries/saved_searches.rs)
- [`backend/src/app/handlers/communications/communication_queries/search.rs`](../../../../backend/src/app/handlers/communications/communication_queries/search.rs)
- [`backend/src/app/handlers/communications/communication_queries/threads.rs`](../../../../backend/src/app/handlers/communications/communication_queries/threads.rs)
- [`backend/src/app/handlers/communications/finance_analytics.rs`](../../../../backend/src/app/handlers/communications/finance_analytics.rs)
- [`backend/src/app/handlers/communications/finance_analytics/analytics.rs`](../../../../backend/src/app/handlers/communications/finance_analytics/analytics.rs)
- [`backend/src/app/handlers/communications/finance_analytics/explain.rs`](../../../../backend/src/app/handlers/communications/finance_analytics/explain.rs)
- [`backend/src/app/handlers/communications/finance_analytics/invoices.rs`](../../../../backend/src/app/handlers/communications/finance_analytics/invoices.rs)
- [`backend/src/app/handlers/communications/finance_analytics/models.rs`](../../../../backend/src/app/handlers/communications/finance_analytics/models.rs)
- [`backend/src/app/handlers/communications/legal_export.rs`](../../../../backend/src/app/handlers/communications/legal_export.rs)
- [`backend/src/app/handlers/communications/message_actions.rs`](../../../../backend/src/app/handlers/communications/message_actions.rs)
- [`backend/src/app/handlers/communications/message_ai_state.rs`](../../../../backend/src/app/handlers/communications/message_ai_state.rs)
- [`backend/src/app/handlers/communications/mod.rs`](../../../../backend/src/app/handlers/communications/mod.rs)
- [`backend/src/app/handlers/communications/remote_images.rs`](../../../../backend/src/app/handlers/communications/remote_images.rs)
- [`backend/src/app/handlers/communications/remote_images/dns.rs`](../../../../backend/src/app/handlers/communications/remote_images/dns.rs)
- [`backend/src/app/handlers/communications/remote_images/errors.rs`](../../../../backend/src/app/handlers/communications/remote_images/errors.rs)
- [`backend/src/app/handlers/communications/remote_images/fetcher.rs`](../../../../backend/src/app/handlers/communications/remote_images/fetcher.rs)
- [`backend/src/app/handlers/communications/remote_images/handler.rs`](../../../../backend/src/app/handlers/communications/remote_images/handler.rs)
- [`backend/src/app/handlers/communications/remote_images/reference.rs`](../../../../backend/src/app/handlers/communications/remote_images/reference.rs)
- [`backend/src/app/handlers/communications/remote_images/url_policy.rs`](../../../../backend/src/app/handlers/communications/remote_images/url_policy.rs)
- [`backend/src/app/handlers/communications/sending.rs`](../../../../backend/src/app/handlers/communications/sending.rs)
- [`backend/src/app/handlers/communications/sending/ai_reply.rs`](../../../../backend/src/app/handlers/communications/sending/ai_reply.rs)

## Кандидаты на drift

- **Модели отправки в `legal_export.rs`:** Файл `legal_export.rs` содержит определения `SendRequest` и `SendResponse`, которые не экспортируются в `mod.rs` из этого же модуля (реэкспорт происходит через `sending::provider_send`). Если эти структуры дублируются или расходятся с реальными сигнатурами `post_v1_send`, возможен дрейф. Подтвердить без кода `sending/provider_send.rs` невозможно.
- **Отсутствующие подмодули `sending/`:** Контекст предоставляет только `ai_reply.rs` из `sending/`. Остальные подмодули (`bilingual_reply_flow`, `certificates`, `forwarding`, `local_state`, `multilingual`, `provider_send`) не включены, поэтому их точное поведение, маршруты и модели не задокументированы и могут отличаться от ожидаемых.
- **Несовпадение лимитов пагинации:** В разных обработчиках используются разные значения `limit` по умолчанию (20, 50, 100, 500). Это может быть намеренным, но при несинхронизированных изменениях в domain‑слое возможно появление неожиданных лимитов.
- **Зависимость `MAKOSH_SEARCH_INDEX_PATH`:** Поиск включается только при установленной переменной окружения. Если документация предполагает, что поиск работает всегда, возникает функциональный дрейф.
- **Путь AI‑reply эндпоинтов:** В коде обработчиков `ai_reply.rs` путь явно не указан; точный маршрут (например, `/v1/message/{id}/ai_reply`) восстанавливается косвенно. Если маршрутизация определена иначе, документация будет неточной. ```
