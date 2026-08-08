## Summary / Резюме

В рамках чанка `059-source-backend-part-039` требуется создать (или обновить) русскую wiki-страницу `components/backend.md`, описывающую ключевые компоненты бэкенда: структуру модуля `integrations`, почтовую интеграцию (отправка, разбор RFC822, синхронизация), клиент Ollama и клиент OmniRoute. Страница должна основываться исключительно на предоставленных исходниках, без домысливания деталей других интеграций (telegram, whatsapp и т.д.), для которых контекст отсутствует.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты бэкенда

## Интеграции (`integrations`)

Модуль `backend/src/integrations/mod.rs` объявляет следующие публичные подмодули:

- `ai_runtime`
- `mail`
- `ollama`
- `omniroute`
- `telegram`
- `whatsapp`
- `yandex_telemost`
- `zoom`

Подробности реализации доступны в рамках предоставленного контекста только для `mail`, `ollama` и `omniroute`.

---

## Почта (`mail`)

### Разбор RFC822 (`rfc822.rs`)

Реэкспортирует типы и функцию разбора из `crate::platform::communications::rfc822`:

- `parse_rfc822_message`
- `EmailRfc822ParseError`
- `ParsedCommunicationSourceMessage`
- `ParsedEmailAttachment`
- `ParsedEmailAttachmentDisposition`

### Отправка (`send.rs`)

#### `LiveSmtpTransport`

Структура, реализующая trait `SmtpTransport` (определён в `crate::platform::communications`).

Метод `send` принимает `SmtpConfig`, `ResolvedSecret` (пароль) и `OutgoingEmail`, создаёт `SmtpClient` и вызывает его.

Фабрика `smtp_outbox_transport()` возвращает `impl SmtpTransport` (экземпляр `LiveSmtpTransport`).

#### `SmtpClient`

Реализует низкоуровневую отправку через SMTP:

- `new()` – конструктор.
- `send(&self, config, password, email)` – устанавливает TCP-соединение с хостом и портом из `SmtpConfig`.
  - Если `config.starttls == true`: вызов `starttls_smtp` (EHLO → STARTTLS → TLS-рукопожатие через `async_native_tls::TlsConnector` → остаток диалога).
  - Если `config.tls == true`: TLS-рукопожатие сразу после TCP-подключения (`send_smtp` с TLS-потоком).
  - Иначе: `send_smtp` с открытым TCP-потоком.

Внутренние функции:

- `send_smtp_after_greeting` – выполняет полный SMTP-диалог после получения приветствия сервера:
  - `EHLO makosh`
  - `AUTH LOGIN`, передача base64-закодированных имени пользователя и пароля.
  - `MAIL FROM:<...>`
  - `RCPT TO:<...>` для каждого получателя (`to`, `cc`, `bcc`); адреса, принятые с кодом 250, записываются в `accepted_recipients`.
  - `DATA`, затем тело письма, завершаемое `\r\n.\r\n`.
  - `QUIT`.
- `read_line`, `write_cmd`, `read_ehlo_response` – вспомогательные async-функции ввода/вывода.
- `build_rfc2822_message` – формирует тело письма в формате RFC2822:
  - Заголовки `From`, `To`, `Cc`, `In-Reply-To`, `References`, `Date`, `Subject`, `MIME-Version: 1.0`.
  - Если `body_html` непустое: `Content-Type: multipart/alternative` с текстовой и HTML-частями.
  - Иначе: `Content-Type: text/plain; charset=utf-8`.
  - Граница multipart генерируется через SHA-256 от ключевых полей письма (префикс `makosh-alt-`).
- `base64` – кодирование в base64 (standard engine).

Тесты (в том же файле):

- `rfc2822_builder_sends_plain_only_messages_as_text_plain` – проверяет, что письмо без HTML имеет `Content-Type: text/plain`.
- `rfc2822_builder_preserves_html_body_as_multipart_alternative` – проверяет наличие заголовков `MIME-Version`, `multipart/alternative`, `In-Reply-To`, `References`, корректность частей.

### Синхронизация (`sync.rs` и подмодули)

#### Структура модуля

- `sync.rs` объявляет подмодули `errors`, `models`, `planning` и реэкспортирует их публичные элементы.
- Все экспортируемые типы и функции происходят из `crate::platform::communications`.

#### Экспортируемые сущности

- `EmailSyncPlanError` (реэкспорт из `crate::platform::communications`).
- `EmailSyncAdapterConfig`, `EmailSyncBatch`, `EmailSyncBlobImportReport`, `EmailSyncImportReport`, `EmailSyncPlan`, `FetchedCommunicationSourceMessage`.
- `imap_mailbox_stream_id`, `plan_email_sync`.

#### Провайдер синхронизации (`sync_provider.rs`)

`LiveEmailProviderSyncPort` – реализация trait `EmailProviderSyncPort`.

Конструктор `new` принимает:

- `pool: PgPool`
- `vault: HostVault`
- `provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>`
- `gmail_api_base_url: impl Into<String>`

Методы trait `EmailProviderSyncPort`:

- `fetch_gmail_message_list` – получает access token (через `gmail_access_token`), создаёт `GmailApiClient` с `user_id("me")`, вызывает `fetch_raw_messages` с опциями `GmailFetchOptions` (max_results, page_token).
- `fetch_gmail_history` – аналогично, но использует `GmailHistoryFetchOptions` (start_history_id, max_results, page_token). Обрабатывает ошибку 404 как признак истечения истории (флаг `history_expired`).
- `fetch_imap_messages` – получает пароль через `read_provider_secret` (с проверкой совместимости назначения и типа секрета), затем создаёт `ImapNetworkClient`, опции `ImapFetchOptions` (хост, порт, TLS, ящик, имя, provider_kind, max_messages, last_seen_uid) и вызывает `fetch_raw_messages`.

Вспомогательные методы:

- `gmail_access_token` – получает OAuth-токен через `EmailAccountSetupService` с возможностью рефреша.
- `read_provider_secret` (внешняя функция) – разрешает секрет по `ProviderAccountSecretPurpose`, проверяет соответствие kind и назначения, затем вызывает `SecretResolver::resolve`.

---

## Ollama Client

Модуль `backend/src/integrations/ollama/client` предоставляет HTTP-клиент для взаимодействия с Ollama API.

### Конфигурация (`OllamaClientConfig`)

Поля (доступны в крейте):

- `base_url: String`
- `chat_model: String`
- `embed_model: String`
- `timeout_seconds: u64` (по умолчанию 120)

Конструктор:

- `new(base_url, chat_model, embed_model)` – обрезает завершающие `/` у base_url.
- `with_timeout_seconds(self, secs) -> Self`

### `OllamaClient`

Поля (публичны в рамках `client`):

- `http: reqwest::Client` (с таймаутом из конфига)
- `base_url: Url`
- `chat_model: String`
- `embed_model: String`

Конструктор `new(config: OllamaClientConfig) -> Result<Self, OllamaError>` валидирует:

- `base_url` не пуст.
- `chat_model` не пуст.
- `embed_model` не пуст.
- `timeout_seconds` > 0.

Создаёт `reqwest::Client` с таймаутом.

Геттеры:

- `chat_model() -> &str`
- `embedding_model() -> &str`

### Каталог моделей (`catalog.rs`)

Методы:

- `version() -> Result<String, OllamaError>` – GET `/api/version`, возвращает поле `version`.
- `tags() -> Result<Vec<String>, OllamaError>` – GET `/api/tags`, возвращает имена моделей (непустые).
- `validate_required_models() -> Result<(), OllamaError>` – проверяет, что `chat_model` и `embed_model` присутствуют в списке `tags`. Иначе возвращает `OllamaError::MissingModel`.

### Чат (`chat.rs`)

Методы:

- `chat(&self, prompt: &str) -> Result<OllamaChatResult, OllamaError>` – вызывает `chat_with_model` с полем `self.chat_model`.
- `chat_with_model(&self, prompt: &str, model: &str) -> Result<OllamaChatResult, OllamaError>`
  - Отправляет POST `/api/chat` с телом JSON: `{ model, stream: false, think: false, messages: [{ role: "user", content: prompt }] }`.
  - Извлекает `content` из поля `message.content` ответа.
  - Пропускает результат через `strip_thinking_content`.
  - Возвращает ошибку, если контент пуст после очистки.

### Эмбеддинги (`embeddings.rs`)

Методы:

- `embed(&self, input: &str) -> Result<OllamaEmbedResult, OllamaError>`
- `embed_with_model(&self, input: &str, model: &str) -> Result<OllamaEmbedResult, OllamaError>`
  - POST `/api/embed` с `{ model, input }`.
  - Извлекает вектор из поля `embeddings` (первый элемент) или `embedding`.
  - Ошибка, если вектор пуст.

### Модели результатов (`models.rs`)

- `OllamaChatResult { model, content, total_duration_ns: Option<u64> }`
- `OllamaEmbedResult { model, embedding: Vec<f32>, total_duration_ns: Option<u64> }`

### Ошибки (`error.rs`)

`OllamaError` (derive `thiserror::Error`):

- `InvalidConfig(String)`
- `Endpoint { status: u16 }`
- `MissingModel { model: String }`
- `Protocol(String)`
- `Http(#[from] reqwest::Error)`

### Транспорт (`transport.rs`)

Внутренние методы `OllamaClient`:

- `endpoint(&self, path: &str) -> Result<Url, OllamaError>` – строит URL от `self.base_url` и `path`.
- `get_json<T>(&self, path: &str) -> Result<T, OllamaError>` – выполняет GET и десериализует ответ, проверяя статус.
- `post_json<T>(&self, path: &str, body: &serde_json::Value) -> Result<T, OllamaError>` – выполняет POST с JSON-телом, десериализует.

Функция `decode_response` проверяет успешность HTTP-статуса, иначе возвращает `OllamaError::Endpoint`.

### Санитизация (`sanitization.rs`)

Функция `strip_thinking_content(content: &str) -> String`:

- Удаляет все вхождения `<think>...</think>` (включая вложенные неполные).
- Если после удаления остаётся закрывающий `</think>` без открывающего, удаляет всё до него.
- Возвращает очищенную строку, обрезанную по краям.

---

## OmniRoute Client

Модуль `backend/src/integrations/omniroute/client` предоставляет клиент для OpenAI-совместимого API с аутентификацией по ключу.

### Конфигурация (`OmniRouteClientConfig`)

Поля (доступны в `super`):

- `base_url: String`
- `chat_model: String`
- `embed_model: String`
- `api_key: ResolvedSecret`
- `timeout_seconds: u64` (по умолчанию 120)

Конструктор `new(base_url, chat_model, embed_model, api_key)` удаляет завершающие `/` и добавляет `/` в конце для корректного построения URL. Создаёт `reqwest::Client` с таймаутом.

### `OmniRouteClient`

Поля:

- `http: reqwest::Client`
- `base_url: Url` (с завершающим `/`)
- `chat_model: String`
- `embed_model: String`
- `api_key: ResolvedSecret`

Конструктор валидирует непустоту строк и таймаут > 0, иначе `OmniRouteError::InvalidConfig`.

Геттеры: `chat_model()`, `embedding_model()`.

### Каталог моделей (`catalog.rs`)

Методы:

- `models() -> Result<Vec<String>, OmniRouteError>` – GET `models`, возвращает ID моделей (непустые).
- `validate_required_models() -> Result<(), OmniRouteError>` – проверяет наличие `chat_model` и `embed_model`.

### Чат (`chat.rs`)

Методы:

- `chat(&self, prompt: &str) -> Result<OmniRouteChatResult, OmniRouteError>`
- `chat_with_model(&self, prompt: &str, model: &str) -> Result<OmniRouteChatResult, OmniRouteError>`
  - POST `chat/completions` с JSON: `{ model, stream: false, messages: [{ role: "user", content: prompt }] }`.
  - Извлекает контент из первого `choices[].message.content`.
  - Применяет `strip_thinking_content` (аналог санитизации из Ollama, определена локально в этом файле).
  - Ошибка, если контент пуст.

### Эмбеддинги (`embeddings.rs`)

Методы:

- `embed(&self, input: &str) -> Result<OmniRouteEmbedResult, OmniRouteError>`
- `embed_with_model(&self, input: &str, model: &str) -> Result<OmniRouteEmbedResult, OmniRouteError>`
  - POST `embeddings` с `{ model, input }`.
  - Извлекает вектор из первого элемента `data[].embedding`.
  - Ошибка, если вектор пуст.

### Модели результатов (`models.rs`)

Точное содержимое `OmniRouteChatResult` и `OmniRouteEmbedResult` не встроено в контекст, но ожидается структура, аналогичная Ollama, с полями `model`, `content`/`embedding`.

### Ошибки (`error.rs`)

`OmniRouteError`:

- `InvalidConfig(String)`
- `MissingApiKey`
- `Endpoint { status: u16 }`
- `MissingModel { model: String }`
- `Protocol(String)`
- `Http(#[from] reqwest::Error)`

### Транспорт

Файл `transport.rs` присутствует в объявлениях `mod`, но текст не встроен. Подразумевается, что реализованы методы `get_json`, `post_json`, аналогичные Ollama, с проверкой статуса.

---

> **Примечание:** Детали других интеграций (`telegram`, `whatsapp`, `yandex_telemost`, `zoom`, `ai_runtime`) отсутствуют в предоставленном контексте и не могут быть документированы в рамках данного чанка.
```

## Source coverage / Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/integrations/mail/rfc822.rs` | Реэкспорт `parse_rfc822_message` и связанных типов из `crate::platform::communications::rfc822`. |
| `backend/src/integrations/mail/send.rs` | `LiveSmtpTransport`, `SmtpClient`, логика отправки SMTP (STARTTLS/TLS/plain), формирование RFC2822-сообщения с поддержкой multipart/alternative, base64-кодирование, тесты. |
| `backend/src/integrations/mail/sync.rs` | Объявление подмодулей `errors`, `models`, `planning`, реэкспорт типов и функций синхронизации из `crate::platform::communications`. |
| `backend/src/integrations/mail/sync/errors.rs` | Реэкспорт `EmailSyncPlanError`. |
| `backend/src/integrations/mail/sync/models.rs` | Реэкспорт `EmailSyncAdapterConfig`, `EmailSyncBatch` и других моделей синхронизации. |
| `backend/src/integrations/mail/sync/planning.rs` | Реэкспорт `imap_mailbox_stream_id`, `plan_email_sync`. |
| `backend/src/integrations/mail/sync_provider.rs` | `LiveEmailProviderSyncPort`, реализация `EmailProviderSyncPort`: `fetch_gmail_message_list`, `fetch_gmail_history`, `fetch_imap_messages`, работа с секретами через `HostVault` и `ProviderSecretBindingLookupPort`, токен-рефреш Gmail. |
| `backend/src/integrations/mod.rs` | Список публичных подмодулей: `ai_runtime`, `mail`, `ollama`, `omniroute`, `telegram`, `whatsapp`, `yandex_telemost`, `zoom`. |
| `backend/src/integrations/ollama/client.rs` | Структура `OllamaClient`, поля, конструктор, геттеры, перечень подмодулей. |
| `backend/src/integrations/ollama/client/catalog.rs` | Методы `version`, `tags`, `validate_required_models`. |
| `backend/src/integrations/ollama/client/chat.rs` | Методы `chat`, `chat_with_model`, POST на `/api/chat`, извлечение контента, санитизация. |
| `backend/src/integrations/ollama/client/config.rs` | `OllamaClientConfig`: поля, конструктор, `with_timeout_seconds`. |
| `backend/src/integrations/ollama/client/embeddings.rs` | `embed`, `embed_with_model`, POST на `/api/embed`, обработка ответов. |
| `backend/src/integrations/ollama/client/error.rs` | Варианты `OllamaError`: `InvalidConfig`, `Endpoint`, `MissingModel`, `Protocol`, `Http`. |
| `backend/src/integrations/ollama/client/models.rs` | `OllamaChatResult`, `OllamaEmbedResult`. |
| `backend/src/integrations/ollama/client/responses.rs` | Десериализуемые типы для ответов `/api/version`, `/api/tags`, `/api/chat`, `/api/embed`. |
| `backend/src/integrations/ollama/client/sanitization.rs` | Логика `strip_thinking_content` (удаление `<think>...</think>`). |
| `backend/src/integrations/ollama/client/transport.rs` | Методы `endpoint`, `get_json`, `post_json`, функция `decode_response` с проверкой статуса. |
| `backend/src/integrations/ollama/mod.rs` | Объявление `pub mod client`. |
| `backend/src/integrations/omniroute/client.rs` | `OmniRouteClient`: поля, конструктор, геттеры, перечень подмодулей. |
| `backend/src/integrations/omniroute/client/catalog.rs` | `models`, `validate_required_models`, типы ответов `models`. |
| `backend/src/integrations/omniroute/client/chat.rs` | `chat`, `chat_with_model`, POST на `chat/completions`, локальная `strip_thinking_content`, типы ответов. |
| `backend/src/integrations/omniroute/client/config.rs` | `OmniRouteClientConfig`: поля, конструктор, `with_timeout_seconds`. |
| `backend/src/integrations/omniroute/client/embeddings.rs` | `embed`, `embed_with_model`, POST на `embeddings`, типы ответов. |
| `backend/src/integrations/omniroute/client/error.rs` | Варианты `OmniRouteError`. |

## Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения кода, документации или ADR не видны. Файлы выглядят согласованными между собой (модульная структура, реэкспорты, trait-реализации). Нет признаков несоответствия типов или неиспользуемых импортов в видимых фрагментах.
