### Summary / Резюме

В русскую wiki встраивается страница `components/backend.md`, описывающая реализацию интеграций Макошь на основе исходного кода из чанка `060-source-backend-part-040`. Добавляются разделы по клиентам **OmniRoute** (модели, транспортный слой) и **Telegram** (управление аккаунтами, чатами, метаданными чатов, очередью команд, жизненным циклом сообщений, сверкой состояний, ошибками, наблюдениями и идентификаторами). Каждое утверждение подтверждено встроенными исходными файлами.

### Proposed pages / Предлагаемые страницы

- `components/backend.md`

```markdown
# Backend (Интеграции)

Backend-компонент Макошь обеспечивает взаимодействие с внешними провайдерами через интеграционные модули. Ниже описана реализация клиентов **OmniRoute** и **Telegram**.

## Интеграции

### OmniRoute

Расположение: `backend/src/integrations/omniroute/`

- **Модели** (`client/models.rs`):
  - `OmniRouteChatResult` — результат чат-запроса: поля `model: String`, `content: String`.
  - `OmniRouteEmbedResult` — результат эмбеддинга: поля `model: String`, `embedding: Vec<f32>`.

- **Транспорт** (`client/transport.rs`):
  - `OmniRouteClient` формирует URL через `endpoint(path)` (склеивание с `base_url`).
  - `get_json<T>(path)` выполняет GET с `bearer_auth` (ключ `api_key`), десериализует ответ.
  - `post_json<T>(path, body)` выполняет POST с JSON-телом.
  - `decode_response` проверяет статус HTTP: при неуспехе возвращает `OmniRouteError::Endpoint { status }`, при ошибке десериализации — `OmniRouteError::Protocol`.

- **Корневой модуль** (`mod.rs`): реэкспортирует `pub mod client;`.

### Telegram

Расположение: `backend/src/integrations/telegram/client/`

#### Аккаунты (`accounts/`)

- **Хранение секретов** (`credential_bindings.rs`):
  - `TelegramStore::store_account_credential` сохраняет учётные данные (API hash, bot token, session key) через `SecretReferenceStore` и `TelegramSecretVault`. Формирует `telegram_secret_ref`, upsert-ит ссылку на секрет и привязывает к аккаунту через `provider_secret_binding_store`.

- **Фикстурные аккаунты** (`fixture_setup.rs`):
  - `setup_fixture_account(request)` валидирует провайдера (должен быть `telegram_user` или `telegram_bot`), создаёт `NewProviderAccount` с `runtime: "fixture"` и сохраняет через `provider_account_store`.

- **Жизненный цикл** (`lifecycle.rs`):
  - `list_accounts(include_removed)` — возвращает все Telegram-аккаунты, фильтруя по `provider_kind.is_telegram()`. По умолчанию исключает удалённые (`lifecycle_state != TELEGRAM_ACCOUNT_REMOVED`).
  - `logout_account(account_id)` — устанавливает `TELEGRAM_ACCOUNT_LOGGED_OUT`.
  - `remove_account(account_id)` — устанавливает `TELEGRAM_ACCOUNT_REMOVED`. При попытке изменить удалённый аккаунт возвращается ошибка `InvalidRequest`.
  - `update_account_lifecycle` обновляет `lifecycle_state`, `lifecycle_updated_at`, а также `logged_out_at`/`removed_at` в конфиге аккаунта через `provider_account_store.update_config_with_origin`.

- **Живые аккаунты – секреты** (`live_credentials.rs`):
  - `store_live_account_credentials(request)` для `TelegramUser` сохраняет `TelegramApiHash` (или пропускает для QR-авторизации) и `TelegramSessionKey` (опционально). Для `TelegramBot` — только `TelegramBotToken`.

- **Живые аккаунты – настройка** (`live_setup.rs`):
  - `setup_live_blocked_account(secret_store, vault, request)` валидирует провайдера, определяет `runtime` (`tdlib_qr_authorized` для QR, иначе `live_blocked`), сохраняет аккаунт с конфигом (включая `tdlib_data_path`, `api_id`), затем сохраняет credentials.

#### Чаты (`chats.rs`, `chat_metadata.rs`, `chat_state.rs`, `chat_reconciliation.rs`, `chats/metadata_flags.rs`)

- **Хранение чатов** (`chats.rs`):
  - `TelegramStore::upsert_chat(chat)` вставляет или обновляет запись в `telegram_chats`. При конфликте (account_id, provider_chat_id) обновляет `chat_kind`, `title`, `username` (COALESCE), `sync_state`, `last_message_at`, `metadata` (объединение с существующим). Генерируется observation.
  - `list_chats(account_id, limit)`, `list_chat_group_filters(account_id)` — выборка и агрегация по папкам (из `folder_labels` и `folder_name`).
  - `apply_provider_chat_folders(account_id, folders)` связывает папки TDLib с чатами, обновляя `folder_labels`.

- **Метаданные чата** (`chat_metadata.rs`):
  - `tdlib_chat_projection_metadata(snapshot, raw_record_id, owner_provider_user_id)` извлекает из снапшота TDLib:
    - `tdlib_permissions` (can_send_messages, can_send_polls, …)
    - `tdlib_notification_settings`, вычисляет `is_muted` (если `!use_default_mute_for && mute_for > 0`)
    - `tdlib_chat_positions` (main, archive, folder_ids), вычисляет `is_archived`, `is_pinned`
    - Тип чата: идентифицирует `chatTypePrivate`, `chatTypeBasicGroup`, `chatTypeSupergroup`. Для приватного self-чата выставляет `is_saved_messages = true`. Для супергрупп — `is_supergroup`, `is_channel_supergroup`, `is_forum`, `tdlib_supergroup_id`.

- **Флаги метаданных** (`chats/metadata_flags.rs`):
  - `set_chat_metadata_bool`, `set_chat_metadata_number`, `set_chat_last_read_at` — атомарная установка значений в метаданных чата.
  - `apply_provider_unread_counts` — запись `unread_count`, `mention_count`, `last_read_inbox_provider_message_id`.
  - `recompute_chat_unread_count` — пересчитывает unread/mention из `provider_channel_message_store` с учётом `last_read_at`.

- **Состояния чата от провайдера** (`chat_state.rs`):
  - `apply_provider_marked_as_unread` — сохраняет `is_marked_as_unread` и источник.
  - `apply_provider_notification_settings` — сохраняет `is_muted` и `tdlib_notification_settings`.
  - `apply_provider_chat_position` — обновляет `tdlib_chat_positions`, пересчитывает `is_archived`, `is_pinned`, обновляет информацию о папках (`folder_ids`, `folder_name`, `folder_labels`).

- **Сверка состояний** (`chat_reconciliation.rs`):
  - `reconcile_dialog_boolean_commands_from_provider_state` сверяет ожидаемое состояние (архив, mute, pin, mark_unread) с наблюдаемым. При несовпадении помечает команду как `mismatch`, иначе — `reconciled`. Используется константа `PROVIDER_RECONCILIATION_CLOCK_SKEW = 5 сек`.
  - Вспомогательные функции: `expected_archive_state_for_command_kind`, `expected_muted_state_for_command_kind` и т.д.

#### Команды (`commands.rs`, `commands/queries.rs`)

- **Модель команды** (`TelegramProviderWriteCommand`): поля `command_id`, `account_id`, `command_kind`, `idempotency_key`, `status`, `provider_chat_id`, `provider_message_id`, `capability_state`, `action_class`, `confirmation_decision`, `retry_count`, `max_retries`, и др.
- **Виды команд**: `send_text`, `send_media`, `reply`, `forward`, `edit`, `delete`, `react`, `unreact`, `pin`, `unpin`, `mark_read`, `mark_unread`, `archive`, `unarchive`, `mute`, `unmute`, `join`, `leave`, `folder_add`, `folder_remove`, `admin_action`.
- **Статусная машина**: `queued` → `retrying` / `executing` → `completed` / `dead_letter`. При старте воркер помечает как `executing` (захват через `CLAIM` в `claim_due_commands_for_execution`).
- **Операции**:
  - `insert_command` — вставка с идемпотентностью, начальный статус `queued`.
  - `update_command_status` — обновление статуса, ошибки, результата.
  - `retry_command`, `schedule_command_retry` — повтор через 30 сек.
  - `dead_letter_command` — перевод в dead letter.
  - `mark_command_awaiting_provider` — перевод в `executing` с `reconciliation_status = awaiting_provider`.
  - `mark_command_reconciled` — завершение с `status = completed`, `reconciliation_status = observed`.
  - `mark_command_mismatch` — фиксация расхождения с провайдером (не включён в чанк, но упоминается).
- **Запросы** (`queries.rs`):
  - `find_command_by_idempotency`, `list_commands`, `list_commands_filtered`, `list_queued_commands_for_execution` (фильтр по статусам и видам команд).

#### Жизненный цикл сообщений (`lifecycle/`)

- **Версии сообщений** (`message_versions.rs`):
  - Таблица `telegram_message_versions`.
  - `insert_message_version` создаёт новую версию с инкрементом `version_number`, `body_text`, `edit_timestamp`, `raw_diff_payload`.
  - `latest_message_version`, `list_message_versions`, `latest_version_number`.
  - `record_provider_edit_observation` — при получении редактирования от провайдера проверяет дубликат, иначе создаёт версию и вызывает `insert_message_version`.
  - `local_edit_diff` вычисляет дифф: предыдущая/новая длина, превью (до 160 символов), SHA-256.

- **Операции** (`operations.rs`):
  - `record_edit` — создаёт версию сообщения, идентификатор идемпотентности `edit:{provider_message_id}:{version_number}`, вставляет команду `edit`.
  - `record_delete` — создаёт tombstone (причина, актор), вставляет команду `delete`.
  - `record_restore_visibility` — создаёт tombstone с `is_local_visible = true`, вставляет команду `restore_visibility`.
  - `record_pin_state` — вызывает `append_message_pin_observation`, вставляет команду `pin`/`unpin`.

- **Tombstones** (`tombstones.rs`):
  - Таблица `telegram_message_tombstones`.
  - `insert_tombstone` — запись удаления/восстановления с полями `reason_class`, `actor_class`, `is_provider_delete`, `is_local_visible`.
  - `list_tombstones`, `is_message_visible` (возвращает последнее `is_local_visible`).
  - `record_provider_delete_observation` — регистрирует удаление провайдером, избегая дубликатов, выставляет `is_local_visible = false`.

- **Сверки сообщений** (`provider_reconciliation.rs`):
  - `reconcile_edit_commands_from_provider_state` — сравнивает `new_text` из команды с текущим текстом провайдера.
  - `reconcile_message_pin_commands_from_provider_state` — сравнивает ожидаемое состояние pin с фактическим.
  - `reconcile_delete_commands_from_provider_state` — считает команду завершённой при наличии удаления на стороне провайдера.

- **Идентификаторы версий/томбстоунов** (`ids.rs`):
  - `new_version_id` — формирует `"tmsgver_{timestamp_ms}_{short_hash}"`.
  - `new_tombstone_id` — `"tmsgtomb_{timestamp_ms}_{short_hash}"`.

#### Ошибки (`errors.rs`)

`TelegramError` — перечисление с вариантами: `InvalidRequest`, `TdlibRuntimeUnavailable`, `TdlibRuntime`, `QrGeneration`, `QrLoginNotFound`, `ProviderAccountStore`, `MediaStorage`, `SecretReference` (transparent), `DatabaseVault` (transparent), `HostVault` (transparent), `CommunicationMessagePort` (transparent), `ObservationStore` (transparent), `Sqlx` (transparent).

#### Наблюдения (`evidence.rs`)

- `link_telegram_entity_in_transaction` — связывает observation с доменной сущностью в домене `"telegram"`.
- `link_communication_entity_in_transaction` — то же для домена `"communications"`.

#### Идентификаторы (`identifiers.rs`)

- `telegram_chat_id(account_id, provider_chat_id)` — SHA-256 от соединённых строк.
- `telegram_message_id(account_id, provider_message_id)`.
- `telegram_raw_record_id(account_id, record_kind, provider_record_id)`.
- `telegram_text_preview_hash(text)` — `"sha256:{hash}"`.
- `telegram_secret_ref(account_id, secret_purpose)` — `"secret:provider-account:{account_id}:{purpose}"`.
- `telegram_account_runtime`, `telegram_account_lifecycle_state`, `ensure_telegram_account_active` — извлечение runtime и состояния из конфига провайдера; проверка, что аккаунт активен (`TELEGRAM_ACCOUNT_ACTIVE`).
- `telegram_account_from_provider_account` — преобразование `ProviderAccount` в `TelegramAccount`.
```

### Source coverage / Покрытие источников

- `backend/src/integrations/omniroute/client/models.rs` — структуры `OmniRouteChatResult` (model, content) и `OmniRouteEmbedResult` (model, embedding: Vec<f32>).
- `backend/src/integrations/omniroute/client/transport.rs` — методы `OmniRouteClient::get_json`, `post_json`, `endpoint`, `decode_response`, маппинг ошибок (`Endpoint`, `Protocol`).
- `backend/src/integrations/omniroute/mod.rs` — public module `client`.
- `backend/src/integrations/telegram/client/accounts.rs` — модульная декларация `credential_bindings`, `fixture_setup`, `lifecycle`, `live_credentials`, `live_setup`.
- `backend/src/integrations/telegram/client/accounts/credential_bindings.rs` — `store_account_credential` с `SecretReferenceStore`, `TelegramSecretVault`, `provider_secret_binding_store`.
- `backend/src/integrations/telegram/client/accounts/fixture_setup.rs` — `setup_fixture_account` с валидацией `provider_kind`, `runtime: "fixture"`.
- `backend/src/integrations/telegram/client/accounts/lifecycle.rs` — `list_accounts`, `logout_account`, `remove_account`, `update_account_lifecycle`, состояния `TELEGRAM_ACCOUNT_LOGGED_OUT`, `TELEGRAM_ACCOUNT_REMOVED`.
- `backend/src/integrations/telegram/client/accounts/live_credentials.rs` — `store_live_account_credentials`, `store_session_key`, поддержка `TelegramUser` (API hash, session key) и `TelegramBot` (bot token).
- `backend/src/integrations/telegram/client/accounts/live_setup.rs` — `setup_live_blocked_account`, runtime `tdlib_qr_authorized`/`live_blocked`, конфиг с `tdlib_data_path`, `api_id`.
- `backend/src/integrations/telegram/client/chat_metadata.rs` — `tdlib_chat_projection_metadata`: извлечение permissions, notification settings, positions, `is_muted`, `is_archived`, `is_pinned`, типов чатов, `is_saved_messages` для self-чата.
- `backend/src/integrations/telegram/client/chat_reconciliation.rs` — `reconcile_dialog_boolean_commands_from_provider_state`, `PROVIDER_RECONCILIATION_CLOCK_SKEW`, функции `expected_*_state_for_command_kind`.
- `backend/src/integrations/telegram/client/chat_state.rs` — `apply_provider_marked_as_unread`, `apply_provider_notification_settings`, `apply_provider_chat_position`, `TelegramProviderChatPositionUpdate`, сверки `reconcile_marked_as_unread_commands_*`, `reconcile_mark_read_commands_*`, `reconcile_mute_commands_*`.
- `backend/src/integrations/telegram/client/chats.rs` — `upsert_chat` (INSERT … ON CONFLICT DO UPDATE, слияние метаданных), `list_chats`, `list_chat_group_filters` (UNION ALL для папок), `apply_provider_chat_folders`.
- `backend/src/integrations/telegram/client/chats/metadata_flags.rs` — `set_chat_metadata_bool`, `set_chat_metadata_number`, `set_chat_last_read_at`, `apply_provider_unread_counts`, `recompute_chat_unread_count` (через `provider_channel_message_store`).
- `backend/src/integrations/telegram/client/commands.rs` — `TelegramProviderWriteCommand`, статусная машина, `insert_command` (idempotency), `update_command_status`, `retry_command`, `dead_letter_command`, `mark_command_awaiting_provider`, `mark_command_reconciled`, `claim_due_commands_for_execution` (упоминание `TELEGRAM_OUTBOX_WORKER_ID`), `list_queued_commands_for_execution`.
- `backend/src/integrations/telegram/client/commands/queries.rs` — `find_command_by_idempotency`, `list_commands`, `list_commands_filtered`, `list_queued_commands_for_execution` (фильтр по статусам и перечню command_kind).
- `backend/src/integrations/telegram/client/errors.rs` — enum `TelegramError` с вариантами.
- `backend/src/integrations/telegram/client/evidence.rs` — `link_telegram_entity_in_transaction`, `link_communication_entity_in_transaction`.
- `backend/src/integrations/telegram/client/identifiers.rs` — хеш-функции для ID, `telegram_secret_ref`, `ensure_telegram_account_active`, `telegram_account_from_provider_account`.
- `backend/src/integrations/telegram/client/lifecycle.rs` — реэкспорт модулей `message_versions`, `operations`, `provider_reconciliation`, `tombstones`, а также `commands` и `reactions`, `references`.
- `backend/src/integrations/telegram/client/lifecycle/ids.rs` — `new_version_id`, `new_tombstone_id` (формат с timestamp и short hash).
- `backend/src/integrations/telegram/client/lifecycle/message_versions.rs` — `insert_message_version`, `latest_message_version`, `list_message_versions`, `latest_version_number`, `record_provider_edit_observation`, `local_edit_diff` (длины, preview, SHA-256).
- `backend/src/integrations/telegram/client/lifecycle/operations.rs` — `record_edit`, `record_delete`, `record_restore_visibility`, `record_pin_state`, взаимодействие с `TelegramStore` и `insert_command`.
- `backend/src/integrations/telegram/client/lifecycle/provider_reconciliation.rs` — `reconcile_edit_commands_from_provider_state`, `reconcile_message_pin_commands_from_provider_state`, `reconcile_delete_commands_from_provider_state`.
- `backend/src/integrations/telegram/client/lifecycle/tombstones.rs` — `insert_tombstone`, `list_tombstones`, `is_message_visible`, `record_provider_delete_observation`.

### Drift candidates / Кандидаты на drift

На основе предоставленного контекста расхождений между кодом, документацией и ADR не выявлено. Исходный контекст не содержит содержимого существующих wiki-страниц или ADR, поэтому сравнение невозможно.
