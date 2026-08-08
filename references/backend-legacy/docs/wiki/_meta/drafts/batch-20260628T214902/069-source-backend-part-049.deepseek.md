### Summary / Резюме

Создать/обновить страницу `components/backend.md` на русском языке с обзором платформенных модулей бэкенда (`calls`, `capabilities`, `communications`, `config`). Страница собирает ключевые сущности, хранилища, traits и процессы, видимые в предоставленных исходных файлах. Описания подтверждаются только встроенным кодом; домысливаемые детали опущены.

### Proposed pages / Предлагаемые страницы

`components/backend.md`:

```markdown
# Бэкенд

Обзор платформенных модулей бэкенда Макошь.

## Модуль звонков (`calls`)

### Сущности

- `TelegramCall` — звонок Telegram (поля: `call_id`, `account_id`, `provider_call_id`, `provider_chat_id`, `direction`, `call_state`, `started_at`, `ended_at`, `transcription_policy_id`, `metadata`, `created_at`, `updated_at`).
- `CallTranscript` — транскрипт звонка (`transcript_id`, `call_id`, `account_id`, `provider_chat_id`, `transcript_status`, `stt_provider`, `source_audio_ref`, `language_code`, `transcript_text`, `segments`, `provenance`, `created_at`, `updated_at`).
- `NewTelegramCall`, `NewCallTranscript` — структуры для вставки (используют валидацию `validate()`).

### Хранилище (`CallIntelligenceStore`)

Инициализируется пулом `PgPool`:

- `upsert_call` — вставляет или обновляет запись `telegram_calls` по конфликту `(account_id, provider_call_id)`. Входные строковые поля обрезаются `.trim()`. Возвращает вставленную/обновлённую запись.
- `upsert_transcript` — вставляет или обновляет `call_transcripts` по конфликту `transcript_id`. Аналогичная обработка полей.
- `list_calls` — выборка звонков с необязательными фильтрами `account_id`, `provider_chat_id`, `provider` (из JSON-поля `metadata ->> 'provider'`). Сортировка по `COALESCE(started_at, created_at) DESC, call_id ASC`. Лимит от 1 до 100.
- `transcript_for_call` — последний (по `created_at DESC`) транскрипт для заданного `call_id`.
- `list_expired_transcripts` — транскрипты с `transcript.provenance -> 'retention_policy' ->> 'expires_at'` ≤ `now()`, с джойном `telegram_calls` для фильтра по провайдеру. Лимит зажимается в `1..500`.
- `remove_transcript` — удаляет транскрипт по `transcript_id`, возвращает удалённую запись.

### Речевые технологии (STT)

`trait SpeechToTextProvider`:

- `provider_name() -> &'static str`
- `transcribe_fixture(audio_ref) -> Result<FixtureTranscript, CallError>`

Реализация `FixtureSpeechToTextProvider`:

- `provider_name` возвращает `"fixture-stt"`.
- `transcribe_fixture` требует непустой `audio_ref` и возвращает фиктивный текст и сегменты (один сегмент, speaker `"local"`, интервал 0–2400 мс, текст `"follow up on the Telegram call"`).

### Валидация

- `validate_limit(limit)` — предел должен быть в диапазоне `1..=100`.
- `validate_non_empty(field, value)` — возвращает обрезанное значение, если не пустое.
- `validate_object(field, value)` — значение должно быть JSON-объектом.
- `validate_array(field, value)` — значение должно быть JSON-массивом.

## Модуль возможностей (`capabilities`)

### Классы действий (`CapabilityActionClass`)

`Read`, `LocalWrite`, `ProviderWrite`, `Destructive`, `Export`, `SecretAccess`, `Automation`.

Метод `as_str()` возвращает `"read"`, `"local_write"`, `"provider_write"`, `"destructive"`, `"export"`, `"secret_access"`, `"automation"`.

### Состояния решений (`CapabilityDecisionStatus`)

- `Allowed` → `"allowed"`
- `Rejected` → `"rejected"`

### Решение (`CapabilityDecision`)

Структура с закрытыми полями, доступ через конструкторы:

- `explicit_user_allowed(class, capability, reason)` — разрешение от пользователя без подтверждения.
- `scoped_automation_allowed(capability, automation_policy_id)` — автоматизированное разрешение по политике (`scoped_automation_policy = true`, `action_class = Automation`).
- `rejected_high_risk(class, capability, reason, automation_policy_id)` — отказ с требованием подтверждения (`confirmation_required = true`).
- `audit_metadata()` — возвращает `serde_json::Value` с ключами `action_class`, `capability`, `decision`, `reason`, `confirmation_required`, `scoped_automation_policy`, `automation_policy_id`.

## Модуль коммуникаций (`communications`)

> Файл `communications.rs` был обрезан в предоставленном контексте. Описанное ниже основано на видимой части.

### Провайдеры (`CommunicationProviderKind`)

Перечисление:

- `Gmail`, `Icloud`, `Imap` (почта, `is_email()`)
- `TelegramUser`, `TelegramBot` (`is_telegram()`)
- `WhatsappWeb`, `WhatsappBusinessCloud` (`is_whatsapp()`)
- `ZoomUser`, `ZoomServerToServer` (`is_zoom()`)
- `YandexTelemostUser` (`is_yandex_telemost()`)

Конвертация из `&str` через `TryFrom`; неизвестный вариант возвращает `CommunicationContractError::UnsupportedProviderKind`.

### Учётные записи

- `ProviderAccount` — учётная запись провайдера: `account_id`, `provider_kind`, `display_name`, `external_account_id`, `config` (JSON), `created_at`, `updated_at`.
- `ProviderAccountUsage` — `raw_record_count`, `message_count`, `checkpoint_count`; метод `has_retained_evidence()`.
- `NewProviderAccount` — структура для создания (детали обрезаны).
- `DeletedProviderAccount` — содержит опциональный `ProviderAccount` и `unbound_secret_refs`.

### Сообщения

- `ProviderChannelMessage` — сообщение: `message_id`, `raw_record_id`, `account_id`, `provider_record_id`, `subject`, `sender`, `body_text`, `occurred_at`, `projected_at`, `channel_kind`, `conversation_id`, `sender_display_name`, `delivery_state`, `message_metadata` (Value).
- `ProviderChannelMessagePortFuture<T>` — асинхронный тип для работы с сообщениями.
- `ProviderMessageReferenceSummary` — сводка сообщения для ссылок.
- `ProviderHeuristicMember` — эвристический участник (sender_id, message_count и т.д.).

### Поток событий наблюдений

`trait ProviderMessageObservationEventPort`:

- `append_provider_message_observation(observation: ProviderMessageObservationEvent) -> Future<Output = Result<Option<i64>, …>>`

`EventStoreProviderMessageObservationEventPort`:

- Реализация, строящая событие `NewEventEnvelope` с idempotency-ключом, хэшем payload (SHA-256), типом `evt_provider_observation_{стабильный_фрагмент}`.
- Cобытие отправляется через `EventStore::append_for_dispatch_idempotent`.
- Валидирует структуру наблюдения перед отправкой.

### Синхронизация почты (`email_sync`)

Функция `plan_email_sync(account) -> Result<EmailSyncPlan, EmailSyncPlanError>`:

- Проверяет, что `provider_kind` — одно из `Gmail`, `Icloud`, `Imap`; иначе ошибка.
- Для Gmail: требует `history_stream_id` (или `"gmail:history"` по умолчанию), `credential_purpose = OauthToken`.
- Для IMAP/Icloud: требует `host`, `port` (порт > 0, помещается в `u16`), `tls` (bool). `mailbox` по умолчанию `"INBOX"`. `credential_purpose = ImapPassword`. Идентификатор потока вычисляется как `imap:` + url-подобное кодирование `%` и `:` в имени ящика.
- Запрещает ключи в `config`, содержащие `password`, `secret`, `token`, `credential` (без учёта регистра).
- Запрещает управляющие символы в строковых полях.

### Сырые сигналы (`raw_signals`)

Перечисление `CommunicationRawSignalSource`:

- `Mail` → `"signal.raw.mail.message.observed"`
- `Telegram` → `"signal.raw.telegram.message.observed"`
- `Whatsapp` → `"signal.raw.whatsapp.message.observed"`

Функция `build_communication_raw_signal_event(source, raw_record, raw_blob_root)`:

- Создаёт `NewEventEnvelope` с типом события согласно источнику.
- Полезная нагрузка (`payload`) — исходный `payload` из `StoredRawCommunicationRecord`.
- `provenance` включает `blob_root`, если задан.
- `causation_id` ссылается на событие захвата наблюдения (`observation_captured_event_id`).
- `correlation_id` — `observation_id` записи.

### Парсинг RFC822 (`rfc822`)

Модуль предоставляет:

- `ParsedCommunicationSourceMessage` — результат парсинга: `subject`, `from`, `to` (список), `headers`, `body_text`, `body_html`, `attachments` (`Vec<ParsedEmailAttachment>`).
- `ParsedEmailAttachment` — вложение: `provider_attachment_id`, `filename`, `content_type`, `disposition` (вложение/встроенное/неизвестно), `body_bytes`.
- `parse_rfc822_message(raw: &[u8])` — основная функция:
  - Разделяет заголовки и тело (поддержка `\r\n\r\n` и `\n\n`).
  - Парсит заголовки, извлекает `subject`, `from`, `to`.
  - Обрабатывает тело: если `Content-Type` — multipart, рекурсивно обходит части; определяет вложения по `Content-Disposition` и наличию имени файла, иначе собирает `text/plain` и `text/html`.
  - Декодирует MIME-части с учётом `Content-Transfer-Encoding` (base64, quoted-printable) и charset (с fallback-цепочкой русских кодировок: `windows-1251`, `koi8-r`, `iso-8859-5`, также `windows-1252`, `iso-8859-1`, и скорингом на основе наличия кириллицы).
  - Декодирует RFC2047-закодированные слова в заголовках.
- Ошибка: `EmailRfc822ParseError::MalformedRfc822` при отсутствии разделителя заголовок/тело.

## Модуль конфигурации (`config`)

### Провайдер AI (`AiRuntimeProvider`)

- `Ollama`, `OmniRoute`.
- `TryFrom<&str>` распознаёт `"ollama"`, `"omniroute"`, `"omni_route"`, `"omni-route"`.

### Конфигурация приложения (`AppConfig`)

Структура с приватными полями (доступ через геттеры `pub fn`):

- `service_name`, `http_addr`, `database_url`, `local_api_secret`, `nats_server_url`.
- `secret_vault_path`, `secret_vault_key`, `vault_home`.
- `dev_mode`, `dev_key_path`.
- `tdjson_path`, `telegram_api_id`, `telegram_api_hash` (опциональны).
- `google_oauth_client`, `google_oauth_client_id`, `google_oauth_client_secret` (опциональны).
- `zoom_token_maintenance_scheduler_enabled`, `zoom_recording_sync_scheduler_enabled`, `zoom_retention_cleanup_scheduler_enabled` — булевы флаги.
- Параметры Ollama: `base_url`, `chat_model`, `embed_model`, `timeout_seconds`.
- Параметры OmniRoute: `base_url`, `chat_model`, `embed_model`, `timeout_seconds`, `api_key` (опционально).
- По умолчанию: `service_name = "makosh"`, HTTP на `127.0.0.1:3000`, AI провайдер — Ollama с предопределёнными моделями и таймаутами (значения см. в `constants`). ZooM-шедулеры по умолчанию `true`.

#### Загрузка из переменных окружения

- `apply_core_env` обрабатывает: `MAKOSH_HTTP_ADDR`, `DATABASE_URL`, `MAKOSH_LOCAL_API_SECRET`, `MAKOSH_NATS_SERVER_URL`, `MAKOSH_SECRET_VAULT_PATH`, `MAKOSH_SECRET_VAULT_KEY`, `MAKOSH_VAULT_HOME`, `MAKOSH_DEV_MODE`, `MAKOSH_DEV_KEY_PATH`.
- `apply_ai_env` обрабатывает: `MAKOSH_AI_PROVIDER`, `MAKOSH_OLLAMA_*`, `MAKOSH_OMNIROUTE_*`. Таймауты должны быть положительными числами.
- Пустые значения обязательных параметров приводят к соответствующей ошибке `ConfigError`.
- Секреты оборачиваются в `ResolvedSecret`.

Также есть загрузка для провайдеров (`provider_env`), но её код не представлен.
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `backend/src/platform/calls/rows.rs` | Функции `row_to_call`, `row_to_transcript`, соответствующие колонки БД. |
| `backend/src/platform/calls/store.rs` | `CallIntelligenceStore::new`, `upsert_call` (конфликт, SQL), `upsert_transcript`, `list_calls` (фильтры, сортировка), `transcript_for_call`, `list_expired_transcripts` (JOIN, фильтр по expires_at), `remove_transcript`. Обработка полей `.trim()`, `.as_deref()`. |
| `backend/src/platform/calls/stt.rs` | `SpeechToTextProvider` trait, `FixtureSpeechToTextProvider`, `transcribe_fixture` поведение, `FixtureTranscript`. |
| `backend/src/platform/calls/validation.rs` | `validate_limit` (1..=100), `validate_non_empty`, `validate_object`, `validate_array`. |
| `backend/src/platform/capabilities.rs` | `CapabilityActionClass`, `CapabilityDecisionStatus`, `CapabilityDecision` конструкторы и `audit_metadata()`. |
| `backend/src/platform/communications.rs` (truncated) | `CommunicationProviderKind` варианты и методы, `ProviderAccount`, `ProviderAccountUsage`, `ProviderChannelMessage`, `ProviderMessageObservationEventPort` trait, `EventStoreProviderMessageObservationEventPort` реализация (события, идемпотентность, SHA-256 хэш). `CommunicationContractError` варианты. `NewProviderAccount` (частично). |
| `backend/src/platform/communications/email_sync.rs` | `plan_email_sync` логика для Gmail/IMAP, `imap_mailbox_stream_id`, валидация конфига, запрет секретоподобных ключей. |
| `backend/src/platform/communications/raw_signals.rs` | `CommunicationRawSignalSource` варианты и их event-типы, `build_communication_raw_signal_event` создание событий. |
| `backend/src/platform/communications/rfc822.rs` | Реэкспорт `parse_rfc822_message`, `ParsedCommunicationSourceMessage`, `ParsedEmailAttachment`, `ParsedEmailAttachmentDisposition`, `EmailRfc822ParseError`. |
| `backend/src/platform/communications/rfc822/body.rs` | Обработка multipart-частей, определение вложений, извлечение `text/plain`/`text/html`, `strip_html_tags`. |
| `backend/src/platform/communications/rfc822/decoding.rs` | Декодирование base64, quoted-printable, выбор чарсета с fallback на русские кодировки, скоринг, `decode_rfc2047_words`. |
| `backend/src/platform/communications/rfc822/errors.rs` | `EmailRfc822ParseError::MalformedRfc822`. |
| `backend/src/platform/communications/rfc822/headers.rs` | Парсинг заголовков, `header_value`, `header_media_type`, `header_parameter` (поддержка RFC2231). |
| `backend/src/platform/communications/rfc822/models.rs` | Структуры `ParsedCommunicationSourceMessage`, `ParsedEmailAttachment`, `ParsedEmailAttachmentDisposition`. |
| `backend/src/platform/communications/rfc822/multipart.rs` | `multipart_parts` — разбор multipart-тела по boundary. |
| `backend/src/platform/communications/rfc822/parser.rs` | `parse_rfc822_message` логика сборки результата, значения по умолчанию для отсутствующих заголовков. |
| `backend/src/platform/communications/rfc822/util.rs` | `split_address_list`, `non_empty_or_default`, `non_empty_recipients`. |
| `backend/src/platform/communications/rfc822/wire.rs` | `split_headers_and_body`, `find_subslice`, `next_line_start`, `strip_trailing_cr`, `trim_ascii_whitespace`. |
| `backend/src/platform/config.rs` | Реэкспорт `AiRuntimeProvider`, `AppConfig`, `ConfigError`, `GoogleOAuthClientConfig`, `GoogleOAuthClientType`. |
| `backend/src/platform/config/ai.rs` | `AiRuntimeProvider` варианты, `as_str()`, `TryFrom<&str>`. |
| `backend/src/platform/config/app_config.rs` | Объявление структуры `AppConfig` (все поля). |
| `backend/src/platform/config/app_config/accessors.rs` | Геттеры `AppConfig` (все публичные методы доступа). |
| `backend/src/platform/config/app_config/ai_env.rs` | Применение AI-переменных окружения, `parse_positive_timeout`, ошибки непустоты. |
| `backend/src/platform/config/app_config/core_env.rs` | Применение ядерных переменных окружения, `non_empty`. |
| `backend/src/platform/config/app_config/defaults.rs` | Значения по умолчанию `AppConfig` (включая `DEFAULT_*` константы, нераскрытые в контексте, но использованные в коде). |

### Drift candidates / Кандидаты на drift

- **Не видно расхождений** между предоставленными исходными файлами. Все документированные утверждения подкреплены кодом.
- Файл `backend/src/platform/communications.rs` был обрезан после 12000 символов; некоторые сущности (часть `NewProviderAccount`, `CommunicationRawRecordCommandPort`, `CommunicationContractError` в полном объёме, `DeletedProviderAccount` методы) могли остаться недокументированными. Это ограничение контекста, а не дрейф.
- Модуль `config/app_config/provider_env.rs` не включён, его поведение не описано. Другие незагруженные файлы (`constants.rs`, `errors.rs`, `google.rs`, `parsing.rs`) не покрыты.
