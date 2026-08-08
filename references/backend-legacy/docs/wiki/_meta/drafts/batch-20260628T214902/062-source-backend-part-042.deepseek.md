## Summary / Резюме

Предлагается дополнить или обновить страницу `components/backend.md` русской Obsidian-вики описанием модулей Telegram-интеграции: клиентской части (`client`) и рантайма (`runtime`). Включены факты о хранилище `TelegramStore`, работе с ссылками на сообщения (reply/forward), поиске, валидации, хранилище секретов, преобразовании строк БД, топиках, архитектуре актора TDLib, цикле обработки команд, авторизации, истории, участниках и ключевых константах. Основа – непосредственно встроенные исходные файлы.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компонент Telegram (backend)

Модуль `integrations::telegram` включает клиентскую логику (`client`) и рантайм на базе TDLib (`runtime`).

## `client` – доступ к данным и утилиты

### Хранилище `TelegramStore`

`TelegramStore` (файл `store.rs`) агрегирует:

- `PgPool` – подключение к PostgreSQL;
- `Arc<dyn ProviderAccountCommandPort>` – команды аккаунтов провайдера;
- `Arc<dyn ProviderSecretBindingCommandPort>` – привязка секретов;
- `Arc<dyn ProviderChannelMessageLookupPort>` – поиск сообщений и сводок;
- `Arc<dyn CommunicationRawRecordCommandPort>` – сырые записи;
- `Arc<dyn ProviderMessageObservationEventPort>` – события наблюдения.

Доступ к портам осуществляется через методы `pool()`, `provider_account_store()`, `provider_secret_binding_store()`, `provider_channel_message_store()`, `communication_raw_record_store()`, `provider_observation_events()`.

### Ссылки на сообщения (`references.rs`)

#### Reply-ссылки

- `insert_reply_ref()` – вставка в таблицу `telegram_message_reply_refs` с уникальным ключом `(source_message_id, target_message_id)`; при конфликте возвращается существующая запись.
- `reply_chain()` – сбор цепочки ответов: потомки (`collect_reply_descendants`) и предки (`collect_reply_ancestors`) через BFS.
  - Максимальная глубина: `MAX_REFERENCE_CHAIN_DEPTH = 16`.
  - Максимальное количество рёбер: `MAX_REFERENCE_CHAIN_EDGES = 128`.
  - К каждому звену прикрепляются сводки `TelegramMessageReferenceSummary` через `reference_message_summaries()`.

#### Forward-ссылки

- `insert_forward_ref()` – вставка в `telegram_message_forward_refs` с уникальностью по `(source_message_id, account_id)`.
- `forward_chain()` – сбор цепочки пересылок (`collect_forward_ancestors`), переход по `forward_origin_message_id`; для локальных сообщений используется `local_forward_origin_message_id()`.

#### Генерация идентификаторов

- `new_reply_ref_id()` формирует строку `tmsgreply_<timestamp_ms>_<sha256‑12>`.
- `new_forward_ref_id()` аналогично даёт `tmsgfwd_...`.
- `stable_short_hash()` вычисляет первые 12 hex-символов SHA-256 от входной строки.

#### Константа типа канала

`TELEGRAM_CHANNEL_KINDS = ["telegram_user", "telegram_bot"]` используется для фильтрации сообщений через `ProviderChannelMessageLookupPort`.

### Поиск (`search.rs`)

Методы `TelegramStore`:

- `pinned_messages(telegram_chat_id, limit)` – через `ProviderChannelMessageLookupPort::pinned_messages()`.
- `search_messages(account_id, provider_chat_id, query, limit)` – через `ProviderChannelMessageLookupPort::search_messages()`.
- `search_chats(account_id, query, limit)` – прямой SQL-запрос к `telegram_chats` с `WHERE title ILIKE $1`.

Все методы применяют `validate_message_list_limit` (1..5000) или эквивалентную проверку.

### Валидация (`validation.rs`)

- `validate_message_list_limit(limit)` и `validate_chat_list_limit(limit)` – допустимый диапазон **1–5000**.
- `validate_non_empty(field, value)` – обрезает пробелы, требует непустую строку.
- `required_optional_value(field, value)` – требует `Some`, затем валидирует как непустую.
- `validate_object(field, value)` – проверяет, что JSON-значение является объектом.

### Хранилище секретов (`vault.rs`)

`TelegramSecretVault` – enum с вариантами:

- `Database(DatabaseEncryptedSecretVault)`
- `Host(HostVault)`

`store_secret()` сохраняет учётные данные через соответствующий вариант, передавая контекст (`SecretEntryContext` с полями `entry_kind`, `account_id`, `purpose`, `secret_kind`, `label`, `metadata`).

### Преобразование строк БД (`rows.rs`)

Набор функций `row_to_telegram_*` маппит `PgRow` в доменные модели:

- `TelegramChat` ⟵ столбцы `telegram_chat_id`, `account_id`, `provider_chat_id`, `chat_kind`, `title`, `username`, `sync_state`, `last_message_at`, `metadata`, временные метки.
- `TelegramMessage` ⟵ `message_id`, `raw_record_id`, `account_id`, `provider_record_id`, `conversation_id`, `subject`, `sender`, `sender_display_name`, `body_text`, `occurred_at`, `projected_at`, `channel_kind`, `delivery_state`, `message_metadata`.
- `TelegramMessageVersion` ⟵ `version_id`, `message_id`, `account_id`, `provider_message_id`, `provider_chat_id`, `version_number`, `body_text`, `edit_timestamp`, `source_event`, `raw_diff_payload`, `provenance`, `created_at`.
- `TelegramMessageTombstone` ⟵ `tombstone_id`, `message_id`, `account_id`, `provider_message_id`, `provider_chat_id`, `reason_class`, `actor_class`, `observed_at`, `source_event`, `is_provider_delete`, `is_local_visible`, `metadata`, `provenance`, `created_at`.
- `TelegramProviderWriteCommand` ⟵ множество полей, включая `command_id`, `command_kind`, `idempotency_key`, `status`, `retry_count`, `locked_at`, `reconciliation_status`, и др.
- `TelegramReaction` ⟵ `reaction_id`, `message_id`, `account_id`, `provider_message_id`, `provider_chat_id`, `sender_id`, `sender_display_name`, `reaction_emoji`, `is_active`, `observed_at`, `source_event`, `provider_actor_id`, `metadata`, `provenance`, `created_at`, `updated_at`.
- `TelegramReplyRef` ⟵ `reply_ref_id`, `source_message_id`, `target_message_id`, `account_id`, `provider_chat_id`, `source_provider_id`, `target_provider_id`, `reply_depth`, `is_topic_reply`, `topic_id`, `metadata`, `provenance`, `created_at` (поля `source_message_summary` и `target_message_summary` из БД не читаются, заполняются отдельно).
- `TelegramForwardRef` ⟵ `forward_ref_id`, `source_message_id`, `account_id`, `provider_chat_id`, `source_provider_id`, `forward_origin_chat_id`, `forward_origin_message_id`, `forward_origin_sender_id`, `forward_origin_sender_name`, `forward_date`, `forward_depth`, `metadata`, `provenance`, `created_at` (поле `source_message_summary` заполняется отдельно).

Также `provider_channel_message_to_telegram_message()` конвертирует `ProviderChannelMessage` в `TelegramMessage`.

### Топики (`topics.rs`)

Работа с таблицей `telegram_topics`:

- `upsert_topic()` – `INSERT … ON CONFLICT (telegram_chat_id, provider_topic_id) DO UPDATE`. В рамках транзакции также создаётся observation `TELEGRAM_TOPIC` через `ObservationStore::capture_in_transaction`.
- `list_topics(telegram_chat_id, limit)` – сортировка: `is_pinned DESC`, `last_message_at DESC NULLS LAST`, `updated_at DESC`.
- `get_topic(topic_id)` – выборка по первичному ключу.
- `search_topics(telegram_chat_id, query, limit)` – `WHERE lower(title) LIKE $2`.
- `list_topic_message_ids(store, topic_id, limit)` – получает `message_id` через `ProviderChannelMessageLookupPort` по метаданным `forum_topic_id`.

### Тесты (`tests.rs`)

- Пустой текст (`"   "`) отклоняется с ошибкой `TelegramError::InvalidRequest` для runtime `"fixture"`.
- Для `"tdlib"` пустая строка разрешена (медиа-снимки могут не иметь текста).
- Лимиты сообщений/чатов: приемлемы 5000, отклоняется 5001.

## `runtime` – живое взаимодействие через TDLib

Модуль (`runtime.rs`) реэкспортирует публичные типы: `TelegramRuntimeManager`, `TelegramChatSyncRequest` / `TelegramChatSyncResponse`, `TelegramHistorySyncRequest` / `TelegramHistorySyncResponse`, `TelegramMediaDownloadRequest` / `TelegramMediaDownloadResponse`, `TelegramMediaSendRequest` / `TelegramMediaSendType`, `TelegramRuntimeStartRequest`, `TelegramRuntimeStopRequest`, `TelegramRuntimeRestartRequest`, `TelegramRuntimeStatus`, и внутренние контексты и утилиты.

Константы:

- `TDJSON_BOOTSTRAP_TIMEOUT = 30s`
- `TDJSON_COMMAND_TIMEOUT = 30s`
- `TDJSON_RECEIVE_POLL_SECONDS = 1.0`

### Архитектура актора (`actor.rs`, `spawn.rs`, `driver.rs`)

- `spawn_tdlib_actor()` проверяет доступность `libtdjson`, формирует `start_request`, создаёт поток с именем `telegram-tdlib-<sanitized_account_id>` и запускает `drive_tdlib_actor()`.
- Канал команд: `mpsc::channel()`, отправитель возвращается вызывающему.
- Опциональный канал событий: `UnboundedSender<TelegramRuntimeEvent>`.

#### Цикл `drive_tdlib_actor` (`driver.rs`)

1. Загрузка библиотеки TDLib, создание клиента.
2. `prepare_tdlib_client()`: создание директории БД, установка `logVerbosityLevel=1`, отправка `getAuthorizationState`.
3. `wait_for_tdlib_ready()`: ожидание `authorizationStateReady`.
4. Основной цикл с `recv_timeout(250ms)`:
   - При таймауте – вызов `drain_unsolicited_tdlib_events()`.
   - При получении команды – диспетчеризация по варианту `TelegramRuntimeCommand`.

**Известные варианты команд** (подтверждённые встроенным кодом):

- `LoadChats { limit, reply_tx }` → `actor_load_chats`
- `GetChatFolders { folder_ids, reply_tx }` → `actor_get_chat_folders`
- `SyncHistory { …, limit, mode, reply_tx }` → `actor_sync_history`
- `SendText { request, reply_tx }` → `actor_send_text`
- `SendMedia { request, reply_tx }` → `actor_send_media`
- `DownloadFile { file_id, priority, reply_tx }` → `actor_download_file`
- `EditMessage { … }` → `actor_edit_message`
- `DeleteMessage { … }` → `actor_delete_message`
- `SetReaction { … }` → `actor_set_reaction`
- `PinMessage { … }` → `actor_pin_message`
- `ToggleChatUnread { … }` → `actor_toggle_chat_unread`
- `ToggleChatArchive { … }` → `actor_toggle_chat_archive`
- `ToggleChatMute { … }` → `actor_toggle_chat_mute`
- `AddChatToFolder { … }` → `actor_add_chat_to_folder`
- `RemoveChatFromFolder { … }` → `actor_remove_chat_from_folder`
- `JoinChat { … }` → `actor_join_chat`
- `LeaveChat { … }` → `actor_leave_chat`
- `ReplyMessage { … }` → `actor_send_reply`
- `ForwardMessage { … }` → `actor_send_forward`
- `GetForumTopics { … }` → `actor_get_forum_topics`
- `CreateForumTopic { … }` → `actor_create_forum_topic`
- `ToggleForumTopicClosed { … }` → `actor_toggle_forum_topic_closed`
- `GetSupergroupMembers { … }` → `actor_get_supergroup_members`
- `GetSupergroupAdministrators { … }` → `actor_get_supergroup_administrators`
- `GetBasicGroupMembers { … }` → `actor_get_basic_group_members`
- `SearchMessages { … }` → `actor_search_messages`
- `SearchChatMessages { … }` → `actor_search_chat_messages`

**Примечание:** исходный файл `driver.rs` обрезан; полный перечень команд не может быть подтверждён из данного контекста.

### Авторизация (`authorization.rs`)

`wait_for_tdlib_ready()` обрабатывает состояния:

- `authorizationStateWaitTdlibParameters` – отправка `setTdlibParametersRequest`.
- `authorizationStateWaitEncryptionKey` – отправка `checkDatabaseEncryptionKeyRequest`.
- `authorizationStateReady` – успех.
- `authorizationStateClosed`, `authorizationStateClosing`, `authorizationStateLoggingOut` – ошибка "session is closed".
- Прочие `authorizationStateWait*` – ошибка "account is not authorized".

### Синхронизация истории (`history.rs`)

`actor_sync_history` загружает историю чата:

- Размер страницы: `limit.clamp(1, 100)`.
- Начальный курсор: `from_message_id` (опционально).
- Для режима `Full` пагинация продолжается до исчерпания сообщений или пока новый курсор не совпадёт с предыдущим.
- Курсор для следующей страницы: `oldest_tdlib_message_id()` (минимальный числовой `provider_message_id` среди полученных).

### Участники (`participants.rs`)

- `actor_get_supergroup_members` и `actor_get_supergroup_administrators` используют пагинацию с `offset` и `limit` (ограничение страницы `TDLIB_SUPERGROUP_MEMBER_PAGE_LIMIT = 100`). Дубликаты исключаются по `provider_member_id`.
- `actor_get_basic_group_members` загружает базовую группу и её полную информацию, затем парсит список участников.

### Отправка сообщений (`send.rs`)

- `actor_send_text` → `tdlib_send_text_message_request`
- `actor_send_media` → `tdlib_send_media_message_request` (предварительная валидация запроса)
- `actor_send_reply` → `tdlib_send_reply_request`
- `actor_send_forward` → `tdlib_send_forward_request`

Каждая функция возвращает `TelegramTdlibMessageSnapshot`.

### Редактирование и управление чатом (`edit.rs`)

- `actor_edit_message` – `editMessageText`.
- `actor_delete_message` – `deleteMessages`, `revoke` опционально.
- `actor_set_reaction` – `addMessageReaction` / `removeMessageReaction`.
- `actor_pin_message` – `pinChatMessage` / `unpinChatMessage`.
- `actor_toggle_chat_unread` – либо `toggleChatMarkedAsUnread`, либо `viewMessages` с указанием `read_through_provider_message_id`.
- `actor_toggle_chat_archive` – `addChatToList` с флагом архивации.
- `actor_toggle_chat_mute` – `setChatNotificationSettings`.
- `actor_add_chat_to_folder` – `addChatToFolder`.
- `actor_remove_chat_from_folder` – сначала `getChatFolder`, затем `editChatFolder` с удалением чата.
- `actor_join_chat` / `actor_leave_chat` – `joinChat` / `leaveChat`.

### Поиск в рантайме (`search.rs`)

- `actor_search_messages` – `searchMessages` с хешем запроса в `@extra`.
- `actor_search_chat_messages` – `searchChatMessages`.

### Загрузка файлов (`download.rs`)

`actor_download_file` вызывает `downloadFile` с `file_id` и `priority`, возвращает `TelegramTdlibFileSnapshot`.

### Сессии (`session.rs`)

`optional_telegram_session_key()` извлекает ключ шифрования сессии:

1. Получает `ProviderSecretBinding` с назначением `TelegramSessionKey`.
2. Разрешает `SecretReference` и проверяет совместимость `secret_kind`.
3. Возвращает `secret.expose_for_runtime()`.

### Стартовый запрос (`start_request.rs`)

`tdlib_start_request_from_account()` требует из `AppConfig`:

- `api_id` (MAKOSH_TELEGRAM_API_ID)
- `api_hash` (MAKOSH_TELEGRAM_API_HASH)
- `tdlib_data_path` из `account.config["tdlib_data_path"]`

Формирует `TelegramQrLoginStartRequest` с необязательным `session_encryption_key`.

### Вспомогательные функции (`support.rs`)

- `oldest_tdlib_message_id()` – минимальный числовой ID сообщения в снимке.
- `short_thread_suffix()` – санитизирует `account_id` для имени потока (только ASCII alphanumeric, до 32 символов).

### Вспомогательные функции ответов (`responses.rs`)

- `receive_tdlib_extra()` – опрашивает события TDLib до совпадения `@extra` с таймаутом.
- `tdlib_provider_chat_id()` – парсит числовой идентификатор чата.
- `tdlib_provider_message_id()` – извлекает числовой ID сообщения из строки вида `<chat>:<id>`.

## Ограничения и константы (сводка)

| Параметр | Значение |
|----------|----------|
| `MAX_REFERENCE_CHAIN_DEPTH` | 16 |
| `MAX_REFERENCE_CHAIN_EDGES` | 128 |
| Лимит сообщений/чатов | 1 – 5000 |
| `TELEGRAM_CHANNEL_KINDS` | `["telegram_user", "telegram_bot"]` |
| `TDLIB_SUPERGROUP_MEMBER_PAGE_LIMIT` | 100 |
| `TDJSON_BOOTSTRAP_TIMEOUT` | 30 с |
| `TDJSON_COMMAND_TIMEOUT` | 30 с |
| `TDJSON_RECEIVE_POLL_SECONDS` | 1.0 с |

## Связанные файлы

- `backend/src/integrations/telegram/mod.rs`
- `backend/src/integrations/telegram/client/references.rs`
- `backend/src/integrations/telegram/client/rows.rs`
- `backend/src/integrations/telegram/client/search.rs`
- `backend/src/integrations/telegram/client/store.rs`
- `backend/src/integrations/telegram/client/tests.rs`
- `backend/src/integrations/telegram/client/topics.rs`
- `backend/src/integrations/telegram/client/validation.rs`
- `backend/src/integrations/telegram/client/vault.rs`
- `backend/src/integrations/telegram/runtime.rs`
- `backend/src/integrations/telegram/runtime/actor.rs`
- `backend/src/integrations/telegram/runtime/actor/authorization.rs`
- `backend/src/integrations/telegram/runtime/actor/chats.rs`
- `backend/src/integrations/telegram/runtime/actor/download.rs`
- `backend/src/integrations/telegram/runtime/actor/driver.rs` (частично)
- `backend/src/integrations/telegram/runtime/actor/edit.rs`
- `backend/src/integrations/telegram/runtime/actor/history.rs`
- `backend/src/integrations/telegram/runtime/actor/participants.rs`
- `backend/src/integrations/telegram/runtime/actor/responses.rs`
- `backend/src/integrations/telegram/runtime/actor/search.rs`
- `backend/src/integrations/telegram/runtime/actor/send.rs`
- `backend/src/integrations/telegram/runtime/actor/session.rs`
- `backend/src/integrations/telegram/runtime/actor/spawn.rs`
- `backend/src/integrations/telegram/runtime/actor/start_request.rs`
- `backend/src/integrations/telegram/runtime/actor/support.rs`
```

## Source coverage / Покрытие источников

| Файл | Факты, отражённые в вики |
|------|---------------------------|
| `references.rs` | Структура reply/forward ссылок, глубина цепочки (16/128), алгоритмы BFS, генерация id, константа `TELEGRAM_CHANNEL_KINDS`, использование `ProviderChannelMessageLookupPort` для сводок. |
| `rows.rs` | Маппинг строк БД в модели: `TelegramChat`, `TelegramMessage`, `TelegramMessageVersion`, `TelegramMessageTombstone`, `TelegramProviderWriteCommand`, `TelegramReaction`, `TelegramReplyRef`, `TelegramForwardRef`. Конвертация `ProviderChannelMessage` → `TelegramMessage`. |
| `search.rs` | Методы `pinned_messages`, `search_messages`, `search_chats` в `TelegramStore`. Использование `ProviderChannelMessageLookupPort` и прямого SQL (`ILIKE`). |
| `store.rs` | Состав `TelegramStore`: пул БД и пять `Arc<dyn …Port>`. Методы доступа к каждому порту. |
| `tests.rs` | Валидация пустого текста для `"fixture"` и `"tdlib"`. Границы лимитов 5000/5001. |
| `topics.rs` | `upsert_topic` с `ON CONFLICT DO UPDATE`, observation `TELEGRAM_TOPIC`, сортировка `list_topics`, поиск `search_topics` через `ILIKE`, `list_topic_message_ids` через `forum_topic_id`. |
| `validation.rs` | Лимиты 1..5000 для сообщений и чатов, `validate_non_empty`, `required_optional_value`, `validate_object`. |
| `vault.rs` | Enum `TelegramSecretVault` (Database/Host), метод `store_secret` с контекстом. |
| `mod.rs` | Публичные модули: `client`, `runtime`, `tdjson`. |
| `runtime.rs` | Реэкспорт типов, константы `TDJSON_BOOTSTRAP_TIMEOUT`, `TDJSON_COMMAND_TIMEOUT`, `TDJSON_RECEIVE_POLL_SECONDS`. |
| `actor.rs` | Субмодули актора, реэкспорт `optional_telegram_session_key`, `spawn_tdlib_actor`, `oldest_tdlib_message_id`. |
| `authorization.rs` | `prepare_tdlib_client`, `wait_for_tdlib_ready` и обрабатываемые authorizationState. |
| `chats.rs` | `actor_load_chats`, `actor_get_chat_folders`. |
| `download.rs` | `actor_download_file`. |
| `driver.rs` (truncated) | Основной цикл, диспетчеризация всех перечисленных команд, `drain_unsolicited_tdlib_events`. |
| `edit.rs` | Все функции редактирования/управления: edit, delete, reaction, pin, toggle unread/archive/mute, add/remove folder, join/leave. |
| `history.rs` | `actor_sync_history`, пагинация с режимом `Full`, ограничение страницы 100. |
| `participants.rs` | `actor_get_supergroup_members`, `actor_get_supergroup_administrators` с пагинацией, `actor_get_basic_group_members`. |
| `responses.rs` | `receive_tdlib_extra`, `tdlib_provider_chat_id`, `tdlib_provider_message_id`. |
| `search.rs` (runtime) | `actor_search_messages`, `actor_search_chat_messages`. |
| `send.rs` | `actor_send_text`, `actor_send_media`, `actor_send_reply`, `actor_send_forward`. |
| `session.rs` | `optional_telegram_session_key` – разрешение сессионного ключа. |
| `spawn.rs` | `spawn_tdlib_actor` – создание потока, проверка доступности `libtdjson`. |
| `start_request.rs` | `tdlib_start_request_from_account` – требования `api_id`, `api_hash`, `tdlib_data_path`. |
| `support.rs` | `oldest_tdlib_message_id`, `short_thread_suffix`. |

## Drift candidates / Кандидаты на drift

1. **Полнота перечня команд в `driver.rs`** – исходный файл обрезан после частичной диспетчеризации `SearchChatMessages`, поэтому невозможно подтвердить, что все варианты `TelegramRuntimeCommand` перечислены. В вики явно указано, что полный перечень не гарантирован данным контекстом.

2. **Дублирование константы `TELEGRAM_CHANNEL_KINDS`** – одинаковый литерал `["telegram_user", "telegram_bot"]` определён в `references.rs`, `search.rs` и `topics.rs`. Это не drift между кодом и документацией, но потенциальный источник расхождения при изменении в одном месте без синхронизации остальных.

3. Отсутствуют расхождения между кодом и ADR/документацией, видимые из предоставленного контекста.
