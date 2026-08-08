### Summary / Резюме

Страница `components/backend.md` должна быть актуализирована на основе встроенных исходных файлов. Предлагаемое обновление описывает структуру модуля `app`, паттерн регистрации обработчиков провайдеров, доступные категории обработчиков Telegram, а также ключевые вспомогательные функции и утилиты. Для провайдеров WhatsApp, Yandex Telemost и Zoom детализация не приводится, так как их реализация не включена в контекст. Для обрезанных файлов указано, что полное содержимое недоступно.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend

Компонент `backend` — это Rust-приложение (фреймворк `axum`, работа с БД через `sqlx`), реализующее API-сервер платформы `makosh`.

## Структура модуля `app`

Основной модуль приложения — `backend/src/app`. Он объявлен в `backend/src/app/mod.rs` и включает следующие подмодули:

- `api_support`
- `connectrpc`
- `error`
- `guard`
- `handlers`
- `provider_runtime_handlers`
- `router`
- `signal_hub_support`
- `state`
- `vault_reconciliation`

Из модуля переэкспортируются:

- типы ошибок `ApiError` и `AppError` (из `error`);
- публичные функции `build_router`, `build_router_with_database`, `init_tracing`, `run` (из `router`);
- типы состояния `AccountSetupState` и `AppState` (из `state`).

## Обработчики провайдеров

Обработчики API сгруппированы по провайдерам. Файлы в `backend/src/app/handlers/` (например, `telegram.rs`, `whatsapp.rs`) являются тонкими реэкспортами из соответствующих модулей `provider_runtime_handlers`:

- `backend/src/app/handlers/telegram.rs` — `pub(crate) use crate::app::provider_runtime_handlers::telegram::*;`
- `backend/src/app/handlers/whatsapp.rs` — `pub(crate) use crate::app::provider_runtime_handlers::whatsapp::*;`
- `backend/src/app/handlers/yandex_telemost.rs` — `pub(crate) use crate::app::provider_runtime_handlers::yandex_telemost::*;`
- `backend/src/app/handlers/zoom.rs` — `pub(crate) use crate::app::provider_runtime_handlers::zoom::*;`

Сами реализации находятся в `backend/src/app/provider_runtime_handlers/`. Модульный файл `provider_runtime_handlers.rs` объявляет подмодули:

```rust
pub(crate) mod telegram;
pub(crate) mod whatsapp;
pub(crate) mod yandex_telemost;
pub(crate) mod zoom;
```

> **Примечание:** исходные файлы для WhatsApp, Yandex Telemost и Zoom в данном контексте не встроены, поэтому их содержание не документировано.

## Telegram-обработчики

Обработчики Telegram — наиболее детально представленная группа. Модуль `provider_runtime_handlers/telegram` содержит подмодули, каждый из которых публикует функции-обработчики через родительский `telegram.rs`. Ниже перечислены категории и примеры эндпоинтов (на основе видимых реэкспортов).

### Управление аккаунтами (`accounts`)
- `delete_telegram_account`
- `get_telegram_accounts`
- `post_telegram_account`
- `post_telegram_account_logout`
- `post_telegram_fixture_account`

Реализация (`accounts.rs`) использует:
- метод `state.config.telegram_api_id()` и хелпер `telegram_api_hash_from_config` для получения учётных данных приложения;
- `provider_account_or_not_found` и `sync_provider_account_signal_connection` для работы с signal-подключениями;
- `telegram_provider_runtime_service` для вызовов runtime-сервиса;
- `telegram_secret_store` и `TelegramSecretVault` для работы с секретами.

### Возможности (`capabilities`)
- `get_telegram_capabilities` — возвращает общие возможности Telegram, определённые конфигурацией.
- `get_telegram_account_capabilities` — возвращает возможности для конкретного аккаунта.

Реализация (`capabilities.rs`) использует `TelegramCapabilitiesResponse::current` и `::current_for_account`.

### Действия с чатами (`chat_actions`)
- `post_telegram_chat_archive`, `post_telegram_chat_unarchive`
- `post_telegram_chat_join`, `post_telegram_chat_leave`
- `post_telegram_chat_mark_read`, `post_telegram_chat_mark_unread`
- `post_telegram_chat_mute`, `post_telegram_chat_unmute`
- `post_telegram_chat_pin`, `post_telegram_chat_unpin`

Файл `chat_actions.rs` обрезан после 12000 символов. В видимой части определены:
- структуры `TelegramChatActionRequest`, `TelegramChatActionResponse`, `TelegramChatLifecycleCommandResponse`;
- функции `record_dialog_command`, `record_chat_lifecycle_command`, `record_chat_lifecycle_command_with_payload` для записи команд жизненного цикла;
- функции публикации событий (`build_event`, `build_command_event`, `build_chat_flag_event`, `build_chat_updated_event`);
- проверка прав через `ensure_telegram_account_operation_allowed`.

### Действия с папками чатов (`chat_folder_actions`)
- `post_telegram_chat_add_folder`
- `post_telegram_chat_reassign_folders`
- `post_telegram_chat_remove_folder`

Реализация (`chat_folder_actions.rs`) полностью поместилась:
- `post_telegram_chat_reassign_folders` вычисляет добавляемые и удаляемые `provider_folder_id` относительно текущих позиций (из метаданных чата), после чего для каждого создает отдельную lifecycle-команду (`folder_add`/`folder_remove`).
- `post_telegram_chat_add_folder` и `post_telegram_chat_remove_folder` создают по одной команде.
- При пустом списке изменений `reassign` возвращает статус `"noop"`.

### Чаты (`chats`)
- `get_telegram_chats` — список чатов с опциональной фильтрацией по `channel_kind`.
- `get_telegram_chat_detail` — детализация чата (поиск как в Telegram-хранилище, так и в `communication_conversations`).
- `get_telegram_chat_members` — участники чата с пагинацией.
- `post_telegram_chat_members_sync` — синхронизация участников через TDLib с публикацией событий (`SYNC_STARTED`, `SYNC_PROGRESS`, `SYNC_COMPLETED`, `SYNC_FAILED`).
- `get_telegram_folders` — список групповых фильтров (папок) чатов.

Файл `chats.rs` обрезан после 12000 символов. Видимая часть также включает функции для работы с каноническими коммуникационными беседами (`list_canonical_communication_conversations`, `canonical_communication_conversation`), которые обращаются к таблице `communication_conversations` и `communication_messages`.

### Команды (`commands`)
- `get_telegram_commands` — список записанных команд с фильтрацией по `account_id`, `provider_chat_id`, `provider_message_id`, `command_kinds`.

### Медиа (`media`)
- `post_telegram_media_download`
- `post_telegram_media_upload`

Файл `media.rs` обрезан. В видимой части:
- обработка загрузки (`post_telegram_media_upload`) включает валидацию запроса (`validate_media_upload_request`), проверку аккаунта и runtime-типа (`tdlib_qr_authorized`), разрешение вложения (`resolve_upload_attachment`), запись команды с идемпотентностью (`media_upload_idempotency_key`), аудит и публикацию событий (`MEDIA_UPLOAD_STARTED`, `COMMAND_STATUS_CHANGED`).

### Сообщения (`messages`)
Обработчики сообщений разделены на основной модуль и подмодули:
- `messages/mark_read.rs` — `post_telegram_message_mark_read`
- `messages/reactions.rs` — `post_telegram_reaction`, `delete_telegram_reaction`, `get_telegram_reactions`

Основной модуль (`messages.rs`) обрезан. Видимые функции:
- `post_telegram_fixture_message`
- `post_telegram_manual_send`
- `post_communication_conversation_message` — унифицированная отправка (роутинг на WhatsApp или Telegram в зависимости от провайдера аккаунта).
- `post_telegram_message_reply`
- `post_telegram_message_forward`
- `post_telegram_message_edit`, `post_telegram_message_delete`, `post_telegram_message_restore_visibility`, `post_telegram_message_pin` — операции жизненного цикла (ADR-0091), также поддерживают роутинг на WhatsApp.
- `post_communication_conversation_pin` и другие унифицированные операции для бесед.

### Исходящие команды (`outbox`)
- `post_telegram_command_retry` — ручной повтор команды с публикацией события `COMMAND_STATUS_CHANGED`.

### QR-вход (`qr_login`)
- `post_telegram_qr_login_start`
- `get_telegram_qr_login_status`
- `delete_telegram_qr_login`
- `post_telegram_qr_login_password`

Используют `AccountSetupState::pending_telegram_qr_login`, конфигурационные `telegram_api_id` и `telegram_api_hash`.

### Сырые данные (`raw`)
- `get_telegram_message_raw` — возвращает сырую запись сообщения с редактированием секретных полей (`access_token`, `api_hash`, `password`, `session_key` и т.п.). Поддерживаются каналы: `telegram_user`, `telegram_bot`, `whatsapp_web`, `whatsapp_business_cloud`.

### Рантайм (`runtime`)
- `get_telegram_runtime_status`
- `post_telegram_runtime_start`
- `post_telegram_runtime_stop`
- `post_telegram_runtime_restart`

Операции `stop` и `restart` записывают аудит-события.

### Поиск (`search`)
Файл `search.rs` обрезан. Видимые обработчики:
- `search_telegram_messages` (GET) — поиск по сохранённым сообщениям (через `ProviderChannelMessageStore`).
- `post_telegram_provider_search` (POST) — запуск поиска через TDLib.
- `search_telegram_chats` (GET) — поиск чатов (Telegram + канонические беседы WhatsApp).
- `get_telegram_pinned_messages` (GET) — закреплённые сообщения.
- `search_telegram_media` (GET) — медиа-поиск.

### Топики (`topics`)
- `get_telegram_topics` — обновление топиков через TDLib с fallback на БД.
- `post_telegram_topic_create`
- `get_telegram_topic_detail`
- `post_telegram_topic_close` — закрытие/переоткрытие топика.
- `get_telegram_topic_messages`
- `search_telegram_topics`

## Общие компоненты

### Состояние приложения (`AppState`)
Доступ к `AppState` осуществляется через `State(state): State<AppState>`. Включает:
- `database` — пул подключений (PostgreSQL);
- `config` — конфигурация приложения (`AppConfig`);
- `event_bus` — шина событий;
- `vault` — хранилище секретов;
- `telegram_runtime` — управление рантаймами Telegram;
- `account_setup` — состояние настройки аккаунтов (в т.ч. QR-сессии).

### Доступ к БД
Функция `database_pool` (`tasks/support.rs`) извлекает `PgPool` из `AppState`:
```rust
pub(super) fn database_pool(state: &AppState) -> Result<PgPool, ApiError> {
    state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)
        .cloned()
}
```

### Проверка возможностей
Функция `ensure_telegram_account_operation_allowed` (`helpers.rs`) проверяет, разрешена ли операция для аккаунта, анализируя его `TelegramCapabilitiesResponse`. Статусы `"available"` или `"degraded"` разрешают выполнение.

### Публикация событий
Функция `publish_telegram_event` (`helpers.rs`) пытается записать событие в `event_store` (если доступен пул БД), затем всегда отправляет в `event_bus.broadcast`.

### Аудит
Большинство мутирующих операций записывают аудит-записи через `api_audit_log` с актором `"makosh-frontend"` (константа `AUDIT_ACTOR_ID`).

### Конфигурация Telegram
Хеш API (`telegram_api_hash`) извлекается из `AppConfig` через метод `telegram_api_hash()` с преобразованием секрета. Идентификатор API (`telegram_api_id`) — напрямую из конфига.

## Ограничения контекста

- Исходные файлы `chat_actions.rs`, `chats.rs`, `media.rs`, `messages.rs`, `search.rs` обрезаны после 12000 символов; полное содержимое не включено.
- Реализации обработчиков WhatsApp, Yandex Telemost и Zoom не встроены в данный контекст — перечислены только их реэкспорты.
- Семантика профилей, флагов, аннотаций и переменных окружения не выводится, если не подтверждена встроенными исходниками.
```

### Source coverage / Покрытие источников

Файлы и покрываемые ими факты:

- **`backend/src/app/handlers/tasks/support.rs`**: функция `database_pool`, извлечение `PgPool` из `AppState`, возврат `ApiError::DatabaseNotConfigured`.
- **`backend/src/app/handlers/telegram.rs`**: реэкспорт из `crate::app::provider_runtime_handlers::telegram`.
- **`backend/src/app/handlers/whatsapp.rs`**: реэкспорт из `crate::app::provider_runtime_handlers::whatsapp`.
- **`backend/src/app/handlers/yandex_telemost.rs`**: реэкспорт из `crate::app::provider_runtime_handlers::yandex_telemost`.
- **`backend/src/app/handlers/zoom.rs`**: реэкспорт из `crate::app::provider_runtime_handlers::zoom`.
- **`backend/src/app/mod.rs`**: структура подмодулей `api_support`, `connectrpc`, `error`, `guard`, `handlers`, `provider_runtime_handlers`, `router`, `signal_hub_support`, `state`, `vault_reconciliation`; переэкспорты `ApiError`, `AppError`, `build_router`, `build_router_with_database`, `init_tracing`, `run`, `AccountSetupState`, `AppState`.
- **`backend/src/app/provider_runtime_handlers.rs`**: объявление подмодулей `telegram`, `whatsapp`, `yandex_telemost`, `zoom`.
- **`backend/src/app/provider_runtime_handlers/telegram.rs`**: реэкспорты всех публичных функций из подмодулей `accounts`, `capabilities`, `chat_actions`, `chat_folder_actions`, `chats`, `commands`, `helpers`, `media`, `messages`, `outbox`, `qr_login`, `raw`, `runtime`, `search`, `topics`.
- **`backend/src/app/provider_runtime_handlers/telegram/accounts.rs`**: обработчики `post_telegram_fixture_account`, `post_telegram_account`, `get_telegram_accounts`, `post_telegram_account_logout`, `delete_telegram_account`; использование `telegram_api_id`, `telegram_api_hash_from_config`, `telegram_secret_store`, `ensure_fixture_routes_enabled`, `telegram_provider_runtime_service`, `provider_account_or_not_found`, `sync_provider_account_signal_connection`; запись аудита.
- **`backend/src/app/provider_runtime_handlers/telegram/capabilities.rs`**: обработчики `get_telegram_capabilities` и `get_telegram_account_capabilities`; использование `TelegramCapabilitiesResponse::current` и `::current_for_account`.
- **`backend/src/app/provider_runtime_handlers/telegram/chat_actions.rs`** (обрезан): видимая часть содержит структуры запросов/ответов (`TelegramChatActionRequest`, `TelegramChatActionResponse`, `TelegramChatLifecycleCommandResponse`), функции записи команд (`record_dialog_command`, `record_chat_lifecycle_command`, `record_chat_lifecycle_command_with_payload`), шины событий (`build_event`, `build_command_event`, `build_chat_flag_event`, `build_chat_updated_event`), проверки прав (`ensure_telegram_account_operation_allowed`), обработчики `post_telegram_chat_join`, `post_telegram_chat_leave`, `post_telegram_chat_pin`, `post_telegram_chat_unpin`, `post_telegram_chat_archive` (частично).
- **`backend/src/app/provider_runtime_handlers/telegram/chat_folder_actions.rs`**: обработчики `post_telegram_chat_add_folder`, `post_telegram_chat_remove_folder`, `post_telegram_chat_reassign_folders`; вычисление добавляемых/удаляемых folder_id; запись команд с payload; аудит.
- **`backend/src/app/provider_runtime_handlers/telegram/chats.rs`** (обрезан): видимая часть содержит `get_telegram_chats`, `get_telegram_folders`, `get_telegram_chat_detail`, `get_telegram_chat_members`, `post_telegram_chat_members_sync`, вспомогательные функции `list_canonical_communication_conversations`, `canonical_communication_conversation`, `canonical_message_row_to_chat`; SQL-запросы к `communication_conversations` и `communication_messages`.
- **`backend/src/app/provider_runtime_handlers/telegram/commands.rs`**: обработчик `get_telegram_commands` с фильтрацией по `command_kinds`.
- **`backend/src/app/provider_runtime_handlers/telegram/helpers.rs`**: константа `AUDIT_ACTOR_ID = "makosh-frontend"`, функции `telegram_api_hash_from_config`, `telegram_secret_store`, `publish_telegram_event`, `ensure_telegram_account_operation_allowed`.
- **`backend/src/app/provider_runtime_handlers/telegram/media.rs`** (обрезан): видимая часть содержит `post_telegram_media_upload` с валидацией, проверкой runtime-типа, разрешением вложения, идемпотентной записью команды, аудитом и публикацией событий.
- **`backend/src/app/provider_runtime_handlers/telegram/messages.rs`** (обрезан): видимая часть содержит `post_telegram_fixture_message`, `post_telegram_manual_send`, `post_communication_conversation_message`, `post_telegram_message_reply`, `post_telegram_message_forward`, `post_telegram_message_edit`, `post_telegram_message_delete`, `post_telegram_message_restore_visibility`, `post_telegram_message_pin`; роутинг на WhatsApp для универсальных операций.
- **`backend/src/app/provider_runtime_handlers/telegram/messages/mark_read.rs`**: обработчик `post_telegram_message_mark_read`.
- **`backend/src/app/provider_runtime_handlers/telegram/messages/reactions.rs`**: обработчики `post_telegram_reaction`, `delete_telegram_reaction`, `get_telegram_reactions`; роутинг на WhatsApp для реакций; вспомогательные функции формирования idempotency key и маппинга ответов.
- **`backend/src/app/provider_runtime_handlers/telegram/outbox.rs`**: обработчик `post_telegram_command_retry`; ручной повтор с публикацией события.
- **`backend/src/app/provider_runtime_handlers/telegram/qr_login.rs`**: обработчики `post_telegram_qr_login_start`, `get_telegram_qr_login_status`, `delete_telegram_qr_login`, `post_telegram_qr_login_password`; использование `qr_login` use-case и `pending_telegram_qr_login`.
- **`backend/src/app/provider_runtime_handlers/telegram/raw.rs`**: обработчик `get_telegram_message_raw`; редактирование секретных полей; структура `TelegramRawMessageRecord`.
- **`backend/src/app/provider_runtime_handlers/telegram/runtime.rs`**: обработчики `get_telegram_runtime_status`, `post_telegram_runtime_start`, `post_telegram_runtime_stop`, `post_telegram_runtime_restart`; аудит для stop/restart.
- **`backend/src/app/provider_runtime_handlers/telegram/search.rs`** (обрезан): видимая часть содержит `search_telegram_messages`, `post_telegram_provider_search`, `search_telegram_chats`, `get_telegram_pinned_messages`, `search_telegram_media`; запросы к `ProviderChannelMessageStore`, обход `communication_conversations`.
- **`backend/src/app/provider_runtime_handlers/telegram/topics.rs`**: все обработчики (`get_telegram_topics`, `post_telegram_topic_create`, `get_telegram_topic_detail`, `post_telegram_topic_close`, `get_telegram_topic_messages`, `search_telegram_topics`); логика fallback на БД при недоступности TDLib; публикация командных событий и аудита.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения не видны. Текущая wiki-страница `components/backend.md` не встроена, поэтому сравнение невозможно. Видимые исходные файлы внутренне согласованы. При последующем обновлении wiki рекомендуется сверить с актуальным деревом модулей, так как некоторые файлы обрезаны, а реализации WhatsApp, Yandex Telemost и Zoom отсутствуют в контексте.
