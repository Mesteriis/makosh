## Summary / Резюме

Чанк содержит исходные файлы бэкенда, относящиеся к доменам `calendar` и `communications`. Предлагаемая страница `components/backend.md` документирует компоненты этих доменов, включая:

- Calendar: тонкий сервис-фасад и функции экспорта в ICS/Markdown.
- Communications core: провайдер-аккаунты, чекпоинты ингестии, сырые записи, типы провайдеров и секреты.
- Communications: командный сервис, массовые действия, аналитика, AI-состояния, AI-ответы, действия (reply/forward/scheduled send), инспекция архивов, дедупликация и поиск вложений, архитектурные блокировщики.

Страница собирается исключительно из предоставленного контекста.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты бэкенда

## Домен Calendar

### Calendar Service

Файл: `backend/src/domains/calendar/service.rs`

Сервис календаря является тонкой обёрткой: он публично ре-экспортирует всё из модуля `command_service`.

```rust
pub use super::command_service::*;
```

Логика самого модуля `command_service` в данном чанке не представлена.

### Экспорт событий календаря

Файл: `backend/src/domains/calendar/sync.rs`

Предоставляет две функции экспорта одного события:

#### ICS-экспорт

`export_event_ics` формирует валидный `VCALENDAR`/`VEVENT` в формате ICS. Параметры:

- `title` — заголовок
- `description` — описание (опционально)
- `location` — место (опционально)
- `start_at`, `end_at` — дата-время в формате, совместимом с `TZID`
- `timezone` — часовой пояс (по умолчанию `Europe/Madrid`)

PRODID фиксирован: `-//Макошь//Calendar//EN`.

#### Markdown-экспорт

`export_event_md` формирует текстовое представление события в Markdown. Вывод включает заголовок первого уровня, блок **When:**, опционально **Where:**, описание и список участников (если передан непустой `participants: &[String]`).

#### Ошибки синхронизации

Перечисление `CalendarSyncError` содержит варианты `SyncFailed` и `ImportFailed` (каждый с сообщением).

---

## Домен Communications

### Core — провайдеры и инжестия

#### Компоненты модуля `core`

Файл: `backend/src/domains/communications/core.rs`

Публично ре-экспортирует:

- `CommunicationIngestionError`, `ProviderCredentialError`
- модели: `CommunicationProviderKind`, `DeletedProviderAccount`, `EmailProviderKind`, `IngestionCheckpoint`, `NewIngestionCheckpoint`, `NewProviderAccount`, `NewProviderAccountSecretBinding`, `NewRawCommunicationRecord`, `ProviderAccount`, `ProviderAccountSecretBinding`, `ProviderAccountSecretPurpose`, `ProviderAccountUsage`, `ProviderCredential`, `StoredRawCommunicationRecord`
- порты: `CommunicationProviderAccountPort`, `CommunicationProviderAccountStore`, `CommunicationProviderSecretBindingStore`
- reader: `ProviderCredentialReader`
- стораджи: `CommunicationIngestionStore` (также как `CommunicationIngestionPort`)

#### Provider Accounts

Файлы: `backend/src/domains/communications/core/accounts.rs`, `backend/src/domains/communications/core/models/accounts.rs`, `backend/src/domains/communications/core/provider_store.rs` (частично)

Модель:

- `ProviderAccount` — поля: `account_id`, `provider_kind: CommunicationProviderKind`, `display_name`, `external_account_id`, `config: Value`, `created_at`, `updated_at`.
- `ProviderAccountUsage` — `raw_record_count`, `message_count`, `checkpoint_count`. Метод `has_retained_evidence()` возвращает `true`, если есть сырые записи или сообщения.
- `NewProviderAccount` — строитель с обязательными полями и методом `config()`; валидация проверяет непустоту `account_id`, `display_name`, `external_account_id`, и что `config` — JSON-объект.
- `DeletedProviderAccount` — содержит опциональный `account` и список освобождённых секретных ссылок `unbound_secret_refs`.

Методы `CommunicationIngestionStore` (из `accounts.rs`) делегируются в `CommunicationProviderAccountStore`:

- `upsert_provider_account`
- `provider_account`
- `list_provider_accounts`
- `update_provider_account_config`
- `provider_account_usage`
- `delete_provider_account_metadata`

`CommunicationProviderAccountStore` (из `provider_store.rs`, частично) поддерживает:

- `upsert` / `upsert_with_origin` — вставка или обновление аккаунта с наблюдением (observation) и привязкой vault-entity.
- `get` — поиск по `account_id`.
- `list` — все аккаунты, сортировка по `provider_kind`, `display_name`, `account_id`.
- `update_config` / `update_config_with_origin` — обновление конфига с проверкой, что он объект, с observation.
- `update_whatsapp_lifecycle_state` — обновление поля `lifecycle_state` в JSONB-конфиге только для WhatsApp-провайдеров.
- `mark_logged_out` — устанавливает `auth_state: "logged_out"` и `logged_out_at`.
- `upsert_runtime_account` — парсит `provider_kind` из строки и вызывает `upsert`.

Все мутирующие операции выполняются в транзакции, фиксируют observation и линкуют vault-owning entity.

#### Checkpoints

Файлы: `backend/src/domains/communications/core/checkpoints.rs`, `backend/src/domains/communications/core/models/checkpoints.rs`

Модель:

- `IngestionCheckpoint` — поля `account_id`, `stream_id`, `checkpoint: Value`, `updated_at`.
- `NewIngestionCheckpoint` — строитель, валидация требует непустые строковые поля и `checkpoint` как JSON-объект.

Методы `CommunicationIngestionStore`:

- `save_checkpoint` — upsert по (`account_id, stream_id`), возвращает запись.
- `checkpoint` — получение по ключу.
- `delete_checkpoint` — удаление, возвращает `true` при успехе.

#### Raw Records

Файлы: `backend/src/domains/communications/core/raw_records.rs`, `backend/src/domains/communications/core/models/raw_records.rs`

Модель:

- `StoredRawCommunicationRecord` — поля: `raw_record_id`, `observation_id`, `account_id`, `record_kind`, `provider_record_id`, `source_fingerprint`, `import_batch_id`, `occurated_at`, `captured_at`, `payload`, `provenance`.
- `NewRawCommunicationRecord` — строитель с методами `occurred_at()` и `provenance()`. Валидация: все строковые поля непусты, `payload` и `provenance` — JSON-объекты.

Методы `CommunicationIngestionStore`:

- `record_raw_source` — проверяет уникальность по (`account_id, record_kind, provider_record_id`). Если запись уже существует, возвращает её (идемпотентно). В противном случае создаёт observation, вставляет запись (с `ON CONFLICT DO NOTHING`), при конфликте фоллбечит чтение существующей.
- `raw_record` — получение по `raw_record_id`.

`CommunicationIngestionStore` также реализует типаж `CommunicationRawRecordCommandPort`.

Observation создаётся с origin `VaultSource`, confidence `1.0`, kind-кодом `COMMUNICATION_MESSAGE` или `COMMUNICATION_ATTACHMENT` (в зависимости от `record_kind`).

#### Provider Kinds

Файл: `backend/src/domains/communications/core/models/provider_kind.rs`

`CommunicationProviderKind` — перечисление:

- `Gmail`, `Icloud`, `Imap`
- `TelegramUser`, `TelegramBot`
- `WhatsappWeb`, `WhatsappBusinessCloud`
- `ZoomUser`, `ZoomServerToServer`
- `YandexTelemostUser`

Методы на варианте: `as_str()`, `is_email()`, `is_telegram()`, `is_whatsapp()`, `is_zoom()`, `is_yandex_telemost()`. `EmailProviderKind` — синоним этого же типа.

Преобразование `TryFrom<&str>` вызывает `CommunicationIngestionError::UnsupportedProviderKind` для неизвестных значений.

#### Secrets

Файлы: `backend/src/domains/communications/core/models/secrets.rs`

`ProviderAccountSecretPurpose` — перечисление целей секретов для провайдеров:

- `OauthToken`, `ImapPassword`, `SmtpPassword`
- `TelegramApiHash`, `TelegramSessionKey`, `TelegramBotToken`
- `WhatsappWebSessionKey`, `WhatsappBusinessCloudAccessToken`, `WhatsappBusinessCloudAppSecret`, `WhatsappBusinessCloudWebhookVerifyToken`
- `ZoomOauthToken`, `ZoomClientSecret`, `ZoomWebhookSecret`
- `YandexTelemostOauthToken`

Метод `accepts_secret_kind()` проверяет совместимость с `SecretKind` (например, `OauthToken` принимает только `SecretKind::OauthToken`, пароли — `AppPassword` или `Password`, API-токены — `ApiToken`, сессионные ключи — `PrivateKey` или `Other`).

`ProviderAccountSecretBinding` — связка `account_id`, `secret_purpose`, `secret_ref`.

`NewProviderAccountSecretBinding` — строитель, валидация требует непустых `account_id` и `secret_ref`.

`ProviderCredential` — агрегат: binding + `SecretReference` + `ResolvedSecret`.

#### Ошибки инжестии

Файл: `backend/src/domains/communications/core/errors.rs`

- `CommunicationIngestionError` — охватывает `Sqlx`, `ObservationStoreError`, `ContractError`, а также `UnsupportedProviderKind`, `UnsupportedSecretPurpose`, `EmptyField`, `NonObjectJson`.
- `ProviderCredentialError` — охватывает `CommunicationIngestionError`, `SecretReferenceError`, `SecretResolutionError`, а также `MissingBinding`, `MissingSecretReference`, `IncompatibleSecretKind`.

#### Row-маппинги

Файл: `backend/src/domains/communications/core/rows.rs`

Функции `row_to_provider_account`, `row_to_raw_record`, `row_to_checkpoint`, `row_to_secret_binding` извлекают значения из `PgRow` и конвертируют строки в доменные типы (с парсингом `provider_kind` и `secret_purpose`).

### Command Service

Файл: `backend/src/domains/communications/command_service.rs` (частично)

`CommunicationCommandService` (оборачивает `PgPool`) предоставляет:

- `upsert_draft` — upsert черновика с observation; различает создание и обновление. Статус парсится из строки через `DraftStatus::parse`, по умолчанию `Draft`.
- `delete_draft` — удаление черновика с предварительным чтением (для метаданных observation). Возвращает `false`, если не найден.
- `create_folder` — создание папки с observation.
- `update_folder` — обновление папки с observation.
- `delete_folder` — удаление папки; возвращает `false` при отсутствии.
- `copy_message_to_folder` — копирование сообщения в папку с observation.
- `move_message_to_folder` — перемещение сообщения в папку с observation (дальше файл обрезан).

Каждая мутация фиксирует observation с типом источника `"makosh-frontend"`, actor-id `"makosh-frontend"`, и связывает сущности через `CommunicationFolderStore` или `CommunicationDraftStore`.

### Bulk Actions

Файл: `backend/src/domains/communications/bulk_actions.rs` (частично)

`BulkMessageAction` перечисляет поддерживаемые операции:

- `MarkRead`, `MarkUnread`
- `Archive`, `Trash`, `Restore`
- `Pin`, `Unpin`
- `Important`, `NotImportant`
- `AddLabel(String)`, `RemoveLabel(String)`
- `Snooze(DateTime<Utc>)`

`BulkMessageActionStore` выполняет операции в транзакции:

- `apply(message_ids, action)` — нормализует список id, вызывает соответствующий метод, собирает исход (`BulkMessageActionOutcome`), сохраняет observation trail и публикует событие в `EventStore`. Транзакция коммитится только при успехе всех шагов.

Каждое действие имеет свой тип события (например, `mail.message.read`, `mail.message.archived`, `mail.message.pinned` и т.д.).

Операции реализуют:

- `update_workflow_state` — устанавливает `workflow_state` (`new`, `reviewed`, `archived`).
- `move_to_trash` — устанавливает `local_state = 'trash'` с причиной `bulk_action`.
- `restore_from_trash` — возвращает `local_state = 'active'`.
- `set_metadata_bool` — обновляет JSONB-поле `message_metadata`, устанавливая булев ключ (`pinned`, `important`).
- `add_label` — добавляет метку в массив `labels` в `message_metadata` с дедупликацией.
- `remove_label` — удаляет метку из массива.
- `snooze` — записывает `snooze_until` в `message_metadata` (дальше файл обрезан).

### Analytics

Файл: `backend/src/domains/communications/analytics.rs`

`EmailAnalyticsStore` обращается к `communication_messages` с фильтром `channel_kind = 'email'` и `local_state = 'active'`.

- `mailbox_health(account_id)` — возвращает `MailboxHealth` с агрегатами:
  - `total_messages`, `unread` (workflow_state='new'), `needs_action`, `waiting`, `done`, `archived`, `spam`
  - `important` (importance_score >= 75)
  - `with_attachments` (наличие записей в `communication_attachments`)
  - `average_importance`, `oldest_message_days`

- `top_senders(account_id, limit)` — возвращает `Vec<SenderStats>`; делегирует в `top_senders_page`.
- `top_senders_page(account_id, limit, cursor)` — пагинация через `SenderStatsCursor`, закодированный в base64 URL-safe JSON. Поля `SenderStats`: `sender`, `message_count`, `avg_importance`, `last_message_days`.

Лимит clamped `[1, 50]`. Курсор декодируется с проверкой валидности.

### AI State (состояния AI-обработки)

Файл: `backend/src/domains/communications/ai_state.rs`

#### Модель состояний

`CommunicationAiState` — перечисление (serde SCREAMING_SNAKE_CASE):

- `New` → `"NEW"`
- `Processing` → `"PROCESSING"`
- `Processed` → `"PROCESSED"`
- `ReviewRequired` → `"REVIEW_REQUIRED"`
- `Failed` → `"FAILED"`
- `Archived` → `"ARCHIVED"`

Реализован `TryFrom<&str>` с ошибкой `CommunicationAiStateError::Invalid("ai_state")`.

#### Store и переходы

`CommunicationAiStateStore` использует таблицы `communication_messages` и `communication_ai_states`.

- `current(message_id)` — возвращает текущую запись. Если в `communication_ai_states` нет строки, начальное состояние — `NEW`. Временные метки заимствуются из `projected_at` сообщения при отсутствии.
- `transition(message_id, request)` — переход с валидацией. `NormalizedCommunicationAiStateTransition` требует:
  - `ReviewRequired` — обязательно `review_reason`
  - `Failed` — обязательно `last_error`
  При успехе: upsert в `communication_ai_states`, фиксация события `mail.ai_state.changed`, опциональная линковка observation.
- `transition_with_observation(..., observation_id, relationship_kind, metadata)` — дополнительно связывает observation через `link_mail_entity_in_transaction`.

Событие `mail.ai_state.changed` включает:

- предыдущее и новое состояние
- флаги `review_required`, `failed`
- provenance `source_kind: local_api`
- correlation_id = `message_id`

### AI Reply (генерация ответа)

Файл: `backend/src/domains/communications/ai_reply.rs`

`AiReplyService` содержит опциональный `SharedAiRuntimePort`.

- `generate_reply(message, options)` — если runtime отсутствует, возвращает `Ok(None)`. Иначе строит промпт с тоном (`tone`, по умолчанию `"professional"`), языком (`language`, по умолчанию `"auto-detect"`) и опциональным контекстом. Тело письма обрезается до 2000 символов (`truncate`). Тема: если начинается с `"re:"` (регистронезависимо) — сохраняется, иначе добавляется префикс `"Re: "`. Результат — `AiReplyDraft` с полями `subject`, `body`, `tone`, `language`.
- `generate_reply_variants(message, languages, tones)` — перебирает комбинации языков и тонов, собирает варианты.

Ошибки: `AiReplyError::Runtime` (оборачивает `AiRuntimePortError`).

### Actions (Reply, Forward, Scheduled Send, Undo Send)

Файл: `backend/src/domains/communications/actions.rs`

#### Конфигурации

- `ReplyConfig` — поля: `quote_original` (по умолчанию `true`), `include_attachments` (`false`), `reply_all` (`false`).
- `ForwardConfig` — `as_eml`, `attachments_only`, `include_ai_summary`, `note`.
- `ScheduledSend` — `send_at: DateTime<Utc>`, `draft_id`.
- `UndoSendWindow` — `window_seconds`, `enabled`.

#### Функции построения тел писем

- `build_reply_body(original_sender, original_date, original_body, reply_text, quote)` — если `quote = true`, добавляет исходное тело с префиксом `"> "` и строку `"On {date}, {sender} wrote:"`.
- `build_forward_body(original_sender, original_date, original_subject, original_body, note)` — формирует блок с заголовком From/Date/Subject, опциональной заметкой и текстом после `"--- Forwarded message ---"`.
- `build_eml_forward(original_sender, original_date, original_subject, original_body, forward_to)` — генерирует EML-представление с заголовком `Content-Type: message/rfc822`.

### Инспекция архивов

Файл: `backend/src/domains/communications/archive_inspection.rs`

Функция `inspect_zip_bytes(bytes, limits)` анализирует ZIP-архив с ограничениями:

- `ArchiveInspectionLimits`:
  - `max_archive_bytes` (по умолчанию 100 MB)
  - `max_uncompressed_bytes` (1 GB)
  - `max_entries` (1 000)
  - `max_depth` (3)

Проверки на превышение лимита выбрасывают соответствующие ошибки `ArchiveInspectionError`.

Для каждой записи:

- Нормализуется путь (`normalize_archive_entry_path`): обратные слеши в прямые, запрет пустых, начинающихся с `/`, содержащих `..` или `:`. Пропускаются части `.` и пустые.
- Вычисляется глубина пути.
- Определяется, является ли вложение вложенным архивом (по расширениям `.zip`, `.rar`, `.7z`).

Результат — `ArchiveInspectionReport` с полями `archive_kind = "zip"`, `entry_count`, `total_uncompressed_bytes`, `has_nested_archive`, `entries: Vec<ArchiveEntryInspection>`.

### Дедупликация вложений

Файл: `backend/src/domains/communications/attachment_dedup.rs`

`AttachmentDedupStore` выполняет две операции:

- `find_duplicates(limit)` — группирует активные вложения по `sha256`, возвращая `DuplicateGroup` с полями `sha256`, `filenames`, `message_ids`, `count`. Только группы с `count > 1`.
- `find_similar_filenames(limit)` — нормализует имена файлов (lowercase, удаление `_final`, `_v\d+`, `_copy`, ` (N)`) и группирует по `base_name`. `sha256` группы заполняется как `"name_group:{base_name}"`.

Лимит clamped `[1, 50]`.

### Поиск вложений

Файл: `backend/src/domains/communications/attachment_search.rs`

`AttachmentSearchStore.search(query)` принимает `AttachmentSearchQuery` с полями:

- `account_id`, `query`, `content_type` (ILIKE), `scan_status`, `cursor`, `limit` (clamped 1..250).

Пагинация: курсор на основе (`created_at`, `attachment_id`), кодированный в base64 URL-safe JSON. Запрашивается `limit + 1` записей для определения `has_more`.

Результат `AttachmentSearchResult` включает поля вложения, сообщения и blob-хранилища (`storage_kind`, `storage_path`). `disposition` и `scan_status` маппятся в строковые DTO (`snake_case`).

Фильтр `query` разбивает строку на слова и ищет их через конкатенацию полей: filename, content_type, sha256, provider_attachment_id, subject, sender.

### Архитектурные блокировщики

Файл: `backend/src/domains/communications/blockers.rs`

Функция `list_blockers()` возвращает фиксированный список из 6 `ArchitectureBlocker`:

1. **§8 Безопасность вложений** — эвристическое сканирование реализовано, полные вердикты требуют внешних инструментов (ClamAV, sandbox, OLE-парсер).
2. **§12 Крипто-верификация** — S/MIME, PGP, CAdES, XAdES, ГОСТ требуют нативных библиотек (OpenSSL, GPG, КриптоПро).
3. **§16-17 Outbox tracking и Follow-up** — базовая отправка, планирование, retry реализованы; полноценный трекинг доставки/прочтения требует webhook-интеграции.
4. **§28-29 Интеграции и массовые действия** — каждая интеграция требует отдельного коннектора; массовые provider-side операции — фоновые задачи.
5. **§8.2 Распаковка архивов** — требуется потоковая распаковка с защитой от zip bomb/path traversal.
6. **§9.3 OCR** — зависит от тяжёлой зависимости (Tesseract), планируется под feature-флагом.

Каждый блокировщик содержит поля `section`, `feature`, `reason`, `resolution`. Тесты проверяют, что все причины и решения непустые, и что количество блокировщиков ровно 6.

---

## Замечания по полноте

Некоторые файлы в чанке были обрезаны до лимита символов, поэтому их описание неполное:

- `backend/src/domains/communications/bulk_actions.rs` — содержит логику snooze и, вероятно, дополнительные детали `capture_observation_trail`.
- `backend/src/domains/communications/command_service.rs` — продолжение `move_message_to_folder` и последующие методы (например, send, send_later).
- `backend/src/domains/communications/core/provider_store.rs` — дальнейшие методы `CommunicationProviderAccountStore` и `CommunicationProviderSecretBindingStore`.
```

## Source coverage / Покрытие источников

| Файл | Покрытые факты |
|---|---|
| `backend/src/domains/calendar/service.rs` | Ре-экспорт `command_service` |
| `backend/src/domains/calendar/sync.rs` | Экспорт в ICS и Markdown, `CalendarSyncError` |
| `backend/src/domains/communications/actions.rs` | `ReplyConfig`, `ForwardConfig`, `ScheduledSend`, `UndoSendWindow`, функции `build_reply_body`, `build_forward_body`, `build_eml_forward`, значения по умолчанию |
| `backend/src/domains/communications/ai_reply.rs` | `AiReplyService`, генерация ответа и вариантов, тон, язык, обрезка тела до 2000 символов, `AiReplyError` |
| `backend/src/domains/communications/ai_state.rs` | `CommunicationAiState` (6 состояний), `CommunicationAiStateStore`, `current`, `transition` с валидацией, события `mail.ai_state.changed`, связывание observation |
| `backend/src/domains/communications/analytics.rs` | `MailboxHealth` агрегаты, `SenderStats` пагинация с курсором, `EmailAnalyticsError` |
| `backend/src/domains/communications/archive_inspection.rs` | `inspect_zip_bytes`, лимиты, нормализация пути (защита от path traversal), `ArchiveInspectionReport` |
| `backend/src/domains/communications/attachment_dedup.rs` | Дубликаты по SHA256 и похожим именам, `DuplicateGroup`, лимит clamped 1..50 |
| `backend/src/domains/communications/attachment_search.rs` | Пагинированный поиск с фильтрами, курсор, `AttachmentSearchResult`, DTO для disposition и scan_status |
| `backend/src/domains/communications/blockers.rs` | 6 архитектурных блокировщиков с причинами и планами разрешения, стабильный тест на количество |
| `backend/src/domains/communications/bulk_actions.rs` (частично) | `BulkMessageAction` варианты, `BulkMessageActionStore.apply`, обновление состояний, меток, snooze (до обрезания), события |
| `backend/src/domains/communications/command_service.rs` (частично) | `CommunicationCommandService`, upsert/delete draft, create/update/delete folder, copy/move message, observation-связывание |
| `backend/src/domains/communications/core.rs` | Ре-экспорты модулей core |
| `backend/src/domains/communications/core/accounts.rs` | Методы `CommunicationIngestionStore` для аккаунтов (делегирование в `CommunicationProviderAccountStore`) |
| `backend/src/domains/communications/core/checkpoints.rs` | Методы `CommunicationIngestionStore` для чекпоинтов |
| `backend/src/domains/communications/core/errors.rs` | `CommunicationIngestionError`, `ProviderCredentialError` |
| `backend/src/domains/communications/core/models.rs` | Ре-экспорт моделей |
| `backend/src/domains/communications/core/models/accounts.rs` | `ProviderAccount`, `ProviderAccountUsage`, `NewProviderAccount`, `DeletedProviderAccount` |
| `backend/src/domains/communications/core/models/checkpoints.rs` | `IngestionCheckpoint`, `NewIngestionCheckpoint` |
| `backend/src/domains/communications/core/models/provider_kind.rs` | `CommunicationProviderKind` с методами классификации |
| `backend/src/domains/communications/core/models/raw_records.rs` | `StoredRawCommunicationRecord`, `NewRawCommunicationRecord` |
| `backend/src/domains/communications/core/models/secrets.rs` | `ProviderAccountSecretPurpose`, `accepts_secret_kind`, `ProviderAccountSecretBinding`, `NewProviderAccountSecretBinding`, `ProviderCredential` |
| `backend/src/domains/communications/core/provider_store.rs` (частично) | `CommunicationProviderAccountStore` — upsert, get, list, update_config, mark_logged_out, upsert_runtime_account, observation-привязка |
| `backend/src/domains/communications/core/raw_records.rs` | `record_raw_source` (идемпотентность), `raw_record`, реализация `CommunicationRawRecordCommandPort`, observation для сырых записей |
| `backend/src/domains/communications/core/rows.rs` | Функции маппинга `PgRow` в доменные типы |

## Drift candidates / Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом и существующей документацией или ADR. Поскольку текущее состояние wiki-страницы `components/backend.md` не было встроено в чанк, подтвердить или опровергнуть дрейф невозможно. Видимых внутренних противоречий в самом коде не обнаружено.
