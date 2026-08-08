---
chunk_id: 066-source-backend-part-046
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 066-source-backend-part-046 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

В русскую Obsidian wiki на страницу `components/backend.md` необходимо добавить описание компонентов бэкенда Макошь, основанное на предоставленных исходных файлах. Добавляемые разделы охватывают интеграцию Telegram через tdjson (QR-логин, снимки, конструкторы запросов) и интеграцию WhatsApp Web (клиент, хранилище, модели, идентификаторы, обработка ошибок). Изменение полностью базируется на встроенном коде.

## Предложенные страницы

### `components/backend.md`

```markdown
# Backend-компоненты

## Интеграция Telegram (tdjson)

### QR-логин

QR-логин реализован с использованием TDLib JSON-интерфейса (`TdJsonClient`). Основные модули находятся в `backend/src/integrations/telegram/tdjson/qr_login/` и `qr_login_support/`.

#### Контекст и состояние воркера (`worker_state.rs`)

- `QrLoginWorkerContext` — контекст, передаваемый воркеру: ссылка на клиент, карта ожидающих логинов (`PendingQrLoginMap`), идентификатор сессии (`setup_id`), запрос на старт (`TelegramQrLoginStartRequest`), канал команд (`Sender<TelegramQrLoginCommand>`), механизм завершения (`QrLoginWorkerCompletion`) и путь к директории БД.
- `QrLoginRuntimeState` — отслеживает фазы выполнения: отправлены ли параметры TDLib (`tdlib_parameters_sent`), проверен ключ шифрования БД (`database_encryption_key_checked`), запрошен QR (`qr_requested`), выдан линк (`qr_link_issued`), идёт ли проверка пароля (`password_check_in_flight`).
- `QrLoginEventOutcome` — исход обработки события: `Continue` (продолжить) или `Complete` (завершено).

#### Вспомогательные модули (`qr_login_support/`)

##### Авторизация (`authorization.rs`)

- `password_hint` извлекает подсказку пароля из состояния авторизации TDLib (поле `password_hint`).
- `state_allows_qr_request` возвращает `true` для состояний: `authorizationStateWaitPhoneNumber`, `WaitPremiumPurchase`, `WaitEmailAddress`, `WaitEmailCode`, `WaitCode`, `WaitRegistration`.

##### Завершение (`completion.rs`)

- `new_worker_completion` создаёт `QrLoginWorkerCompletion` (`Arc<(Mutex<bool>, Condvar)>`).
- `mark_worker_complete` устанавливает флаг завершения и оповещает ожидающих.
- `wait_for_worker_completion` ожидает с таймаутом `QR_CANCEL_WAIT_TIMEOUT` (5 секунд).

##### Константы (`constants.rs`)

- `QR_FIRST_LINK_TIMEOUT` — 20 с.
- `QR_SESSION_LIFETIME` — 10 мин.
- `QR_CANCEL_WAIT_TIMEOUT` — 5 с.
- `QR_GET_ME_TIMEOUT` — 5 с.
- `QR_POLL_AFTER_MS` — 2000 мс.

##### Идентификаторы (`identifiers.rs`)

- `new_setup_id` генерирует идентификатор сессии вида `telegram-qr-{safe_path_segment(account_id)}-{digest[..16]}`, где digest – первые 16 символов SHA-256 от `account_id`, нулевого байта и текущего timestamp в наносекундах.
- `short_thread_suffix` возвращает первые 32 символа от `safe_path_segment(account_id)`.

##### Идентичность (`identity.rs`)

- `fetch_authorized_user_identity` отправляет запрос `getMe` и ожидает ответ в течение `QR_GET_ME_TIMEOUT`; возвращает `Option<TelegramQrLoginIdentity>`.
- `parse_tdlib_user_identity` извлекает `user_id` (числовой идентификатор), `username` (из `username` или первого элемента `usernames.active_usernames`), формирует `TelegramQrLoginIdentity` с полями:
  - `suggested_account_id` (при наличии username – `{safe_user_id}_account_{safe_username}`, иначе `{safe_user_id}_account`),
  - `suggested_display_name` (`@{username}` или user_id),
  - `suggested_external_account_id` (`telegram:{user_id}`).
- `safe_account_identifier` нормализует строку: только ASCII буквы/цифры и `_`, в нижний регистр, с обрезкой висящих `_` (пустая строка заменяется на `"telegram"`).

##### Ожидающие сессии (`pending.rs`)

- `upsert_pending_response` вставляет `TelegramQrLoginSession` в `PendingQrLoginMap`.
- `mark_pending_status` обновляет статус и сообщение в сессии; сбрасывает `expires_at`, кроме состояний `WaitingQrScan` и `WaitingPassword`.
- `mark_pending_ready_status` устанавливает статус `Ready`, очищает срок действия, заполняет поля идентичности пользователя.

##### Генерация QR-кода (`qr.rs`)

- `render_qr_svg` создаёт SVG QR-кода с минимальными размерами 240×240 пикселей; при ошибке кодирования возвращает `TelegramError::QrGeneration`.

##### Ответы API (`responses.rs`)

- `qr_waiting_response` — ответ со статусом `WaitingQrScan`, ссылкой `qr_link` и SVG `qr_svg`, `poll_after_ms = 2000`.
- `qr_preparing_response` — статус `WaitingQrScan` без QR, `poll_after_ms = 1000`, сообщение `Preparing Telegram QR code.`.
- `password_waiting_response` — статус `WaitingPassword`, `poll_after_ms = 2000`.
- `ready_response` — статус `Ready`, `poll_after_ms = 0`, поля идентичности при наличии.

##### Типы (`types.rs`)

- `PendingQrLoginMap = Arc<Mutex<HashMap<String, TelegramQrLoginSession>>>`.
- `TelegramQrLoginSession` содержит `response`, `command_tx` (канал `Sender<TelegramQrLoginCommand>`) и `worker_completion`.
- `TelegramQrLoginCommand` — `CheckPassword(String)`, `Cancel`.
- `DrainedQrLoginCommand` — `None`, `PasswordSubmitted`, `Cancelled`.
- `TelegramQrLoginIdentity` — `user_id`, `username`, `suggested_account_id`, `suggested_display_name`, `suggested_external_account_id`.

### Снимки (`snapshots.rs`)

Определены структуры для представления обработанных данных Telegram:

- `TelegramTdlibTopicSnapshot` (`provider_topic_id`, `title`, `icon_emoji`, `is_pinned`, `is_closed`, `unread_count`, `last_message_at`).
- `TelegramTdlibChatSnapshot` (`provider_chat_id`, `chat_kind: TelegramChatKind`, `title`, `username`, `last_message_at`, `raw`).
- `TelegramTdlibChatFolderSnapshot` (`provider_folder_id`, `title`, `icon_name`, `color_id`, `raw`).
- `TelegramTdlibChatMemberSnapshot` (`provider_member_id`, `display_name`, `username`, `role`, `status`, `is_admin`, `is_owner`, `permissions`, `raw`).
- `TelegramTdlibMessageSnapshot` (`provider_chat_id`, `provider_message_id`, `sender_id`, `sender_display_name`, `text`, `occurred_at`, `delivery_state: TelegramDeliveryState`, `raw`).
- `TelegramTdlibMessageDeleteSnapshot`, `MessageInteractionInfoSnapshot`, `MessageContentSnapshot`, `MessageEditedSnapshot`, `MessagePinnedSnapshot` — каждая содержит идентификаторы и специфичные поля.
- `TelegramTdlibFileSnapshot` (`file_id`, `size_bytes`, `expected_size_bytes`, `local_path`, `is_downloading_active`, `is_downloading_completed`, `downloaded_size_bytes`, `remote_id`, `remote_unique_id`, `raw`).

### Запросы к TDLib (`requests.rs`)

Конструкторы JSON-запросов к TDLib (файл обрезан до 12000 символов; перечисленное ниже покрыто встроенным текстом):

- `set_tdlib_parameters_request` — параметры: `api_id`, `api_hash`, ключ шифрования БД (как base64 от `session_encryption_key`), `device_model: "Макошь"`, `system_version: std::env::consts::OS`, `application_version: CARGO_PKG_VERSION`, `enable_storage_optimizer: true`, `ignore_file_names: false` и другие.
- `tdlib_database_directory` — путь из запроса или `docker/data/telegram/{account_id}`.
- `check_database_encryption_key_request` — проверка ключа шифрования.
- Запросы чатов: `tdlib_load_chats_request`, `tdlib_get_chats_request`, `tdlib_get_chat_request`, `tdlib_get_basic_group_request`, `tdlib_get_basic_group_full_info_request`, `tdlib_get_chat_folder_request`, `tdlib_get_chat_history_request`.
- Отправка сообщений:
  - `tdlib_send_text_message_request` — `sendMessage` с `inputMessageText` (требует непустой текст).
  - `tdlib_send_media_message_request` — поддерживает типы `TelegramMediaSendType`: `Photo`, `Video`, `Document`, `Audio`, `Voice`, `Sticker`, `Animation`; каждый формирует соответствующий `inputMessage*`.
- `tdlib_edit_message_text_request` — редактирование текста (требует непустой текст).
- `tdlib_delete_messages_request` — удаление с флагом `revoke`.
- Реакции: `tdlib_add_message_reaction_request`, `tdlib_remove_message_reaction_request` (тип `reactionTypeEmoji`).
- `tdlib_pin_chat_message_request` — закрепление (с `disable_notification`, `only_for_self: false`).
- `tdlib_send_reply_request` — ответ с `inputMessageReplyToMessage` (требует непустой текст).
- `tdlib_send_forward_request` — пересылка (видна только сигнатура из-за обрезки).

Все запросы содержат поле `@extra` для отслеживания.

## Интеграция WhatsApp Web

### Клиент (`WhatsappWebStore` и модели)

Клиент WhatsApp Web построен вокруг хранилища `WhatsappWebStore`, которое агрегирует пул БД PostgreSQL (`PgPool`) и порты: `ProviderAccountCommandPort`, `ProviderSecretBindingCommandPort`, `ProviderChannelMessageLookupPort`.

#### Модели данных

##### Запросы настройки аккаунта

- `WhatsappWebAccountSetupRequest` (фикстура) — обязательные поля после валидации: `account_id`, `display_name`, `external_account_id`, `device_name`, `local_state_path`. `provider_shape` опционален, но если задан – непустой.
- `WhatsappLiveAccountSetupRequest` (live) — валидация зависит от `provider_shape`:
  - Поддерживаемые shapes: `whatsapp_web_companion`, `whatsapp_native_md`, `whatsapp_business_cloud`.
  - Для `whatsapp_business_cloud` обязательно `api_access_token`, запрещён `device_name`; `app_secret` и `webhook_verify_token` допустимы только для этой формы.

##### Сессия

- `WhatsappWebSession` (сериализуемая) — содержит `session_id`, `account_id`, `device_name`, `companion_runtime` (строка), `link_state` (строка), `local_state_path`, `last_sync_at`, `metadata`, `created_at`, `updated_at`.
- `NewWhatsappWebSession` (для создания) — те же поля, кроме временных меток создания/обновления, плюс обязательная валидация всех строковых полей.
- `WhatsappWebCompanionRuntime` — перечисление: `Fixture`, `ManualWebview`, `Blocked`, `ApiCredentials`.
- `WhatsappWebLinkState` — перечисление: `Fixture`, `QrPending`, `PairCodePending`, `Linked`, `Degraded`, `Revoked`, `Blocked`.

##### Сущности для импорта (fixture)

- `NewWhatsappWebMessage` — обязательные поля: `account_id`, `provider_chat_id`, `provider_message_id`, `chat_title`, `sender_id`, `sender_display_name`, `text`, `import_batch_id`, `occurred_at`, `delivery_state`. Имеет метод `source_fingerprint` на основе хеша конкатенации `account_id`, `provider_chat_id`, `provider_message_id`, `sender_id`, `text`.
- `NewWhatsappWebReaction` — поля: `provider_chat_id`, `provider_message_id`, `provider_actor_id`, `sender_display_name`, `reaction`, `is_active`, `observed_at`. `provider_record_id` = `{provider_message_id}:{provider_actor_id}:{reaction}`.
- `NewWhatsappWebMedia` — поля: `provider_chat_id`, `provider_message_id`, `provider_attachment_id`, `filename`, `content_type`, `size_bytes`, `sha256`, `storage_kind`, `storage_path`, `observed_at`. `provider_record_id` = `{provider_message_id}:{provider_attachment_id}`.
- `NewWhatsappWebStatus` — статусы (аналог Stories) с `provider_status_id`, `sender_id`, `sender_display_name`, `sender_identity_kind`, `sender_address`, `sender_push_name`, `sender_business_profile`, `sender_profile_photo_ref`, `text`.
- `NewWhatsappWebStatusView`, `NewWhatsappWebStatusDelete` — просмотры и удаления статусов.

#### Хранилище (`store`)

- **Аккаунты** (`store/accounts.rs`):
  - `setup_fixture_account` — настраивает аккаунт в режиме fixture: нормализует `provider_shape`, создаёт `NewProviderAccount` с конфигом (`runtime: fixture`, `setup_semantics`, `session_mode`), создаёт сессию с `companion_runtime=Fixture` и `link_state=Fixture`.
  - `setup_live_blocked_account` — аналогично для live blocked: `runtime: live_blocked`, `link_state=Blocked`, `companion_runtime=Blocked` (или `ApiCredentials` для `whatsapp_business_cloud`).
  - Поддерживаемые `provider_shape`: `whatsapp_web_companion`, `whatsapp_native_md`, `whatsapp_business_cloud`.
  - `provider_kind` должен соответствовать shape: для `whatsapp_business_cloud` — `WhatsappBusinessCloud`, иначе — `WhatsappWeb`.
- **Приём данных** (`store/ingestion.rs`) — файл обрезан; встроенный текст покрывает:
  - `ingest_fixture_message`, `ingest_fixture_reaction`, `ingest_fixture_media`, `ingest_fixture_status`, `ingest_fixture_status_view` — каждый метод проверяет входную модель, получает аккаунт провайдера, создаёт `NewRawCommunicationRecord` с метаданными и провенансом, обновляет `last_sync_at` у сессии. Для идентификации используются константы из `constants.rs` (например, `WHATSAPP_WEB_MESSAGE_RECORD_KIND`).

#### Вспомогательные модули

- **Идентификаторы** (`ids.rs`): все строятся через SHA-256 с префиксом версии `v5`:
  - `whatsapp_web_session_id(account_id)` — `whatsapp_web_session:v5:{hash}`.
  - `whatsapp_web_message_id(account_id, provider_message_id)` — `message:v5:whatsapp_web:{hash}`.
  - `whatsapp_web_raw_record_id(account_id, record_kind, provider_record_id)` — `raw:v5:whatsapp_web:{hash}`.
- **Ошибки** (`errors.rs`): `WhatsappWebError` — варианты `InvalidRequest`, `ProviderAccountStore`, `CommunicationMessagePort`, `ObservationStore`, `SecretReference`, `SecretResolution`, `HostVault`, `Sqlx` (с прозрачным преобразованием из соответствующих типов ошибок).
- **Константы записей** (`constants.rs`): 13 констант `*_RECORD_KIND` для типов записей (сообщение, реакция, медиа, статус, просмотр статуса, удаление статуса, присутствие, звонок, runtime-событие, диалог, участник, обновление сообщения, удаление сообщения, квитанция).
- **Строки БД** (`rows.rs`): функции `row_to_whatsapp_web_session` и `row_to_whatsapp_web_message` для маппинга `PgRow` в доменные типы; `provider_channel_message_to_whatsapp_web_message` для преобразования из общего `ProviderChannelMessage`.
- **Связывание сущностей** (`store/evidence.rs`): `link_whatsapp_entity_in_transaction` — делегирует в `link_domain_entity_in_transaction` с доменом `communications`.
- **Интеллект** (`store/intelligence.rs`): `refresh_message_intelligence_candidates` — заглушка, всегда возвращает `Ok(())`.

#### Тесты (`tests.rs`)

Определены тестовые модули: `environment`, `parsing_snapshots`, `qr_login_flows`, `request_builders`. Вспомогательная функция `test_qr_login_response` создаёт тестовый ответ QR-логина с заполненными полями `setup_id`, `qr_link`, `qr_svg`, `poll_after_ms: 2000`, статус параметризуется.
```

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/integrations/telegram/tdjson/qr_login/worker_state.rs` | Структуры `QrLoginWorkerContext`, `QrLoginRuntimeState` (поля и их смысл), перечисление `QrLoginEventOutcome`. |
| `backend/src/integrations/telegram/tdjson/qr_login_support.rs` | Декларация подмодулей и реэкспорты, архитектурная группировка функций и типов QR-логина. |
| `backend/src/integrations/telegram/tdjson/qr_login_support/authorization.rs` | `password_hint` (извлечение подсказки), `state_allows_qr_request` (разрешённые состояния авторизации). |
| `backend/src/integrations/telegram/tdjson/qr_login_support/completion.rs` | `new_worker_completion`, `mark_worker_complete`, `wait_for_worker_completion` и их поведение; использование `QR_CANCEL_WAIT_TIMEOUT`. |
| `backend/src/integrations/telegram/tdjson/qr_login_support/constants.rs` | Константы `QR_FIRST_LINK_TIMEOUT`, `QR_SESSION_LIFETIME`, `QR_CANCEL_WAIT_TIMEOUT`, `QR_GET_ME_TIMEOUT`, `QR_POLL_AFTER_MS` и их значения. |
| `backend/src/integrations/telegram/tdjson/qr_login_support/identifiers.rs` | `new_setup_id` (алгоритм генерации), `short_thread_suffix` (извлечение префикса), использование `safe_path_segment` и SHA-256. |
| `backend/src/integrations/telegram/tdjson/qr_login_support/identity.rs` | `fetch_authorized_user_identity` (запрос `getMe`, таймаут), `parse_tdlib_user_identity` (извлечение полей, `username` из `usernames.active_usernames`), `tdlib_user_username`, `safe_account_identifier`. |
| `backend/src/integrations/telegram/tdjson/qr_login_support/pending.rs` | `upsert_pending_response`, `mark_pending_status` (поведение с `expires_at`), `mark_pending_ready_status` (заполнение идентичности). |
| `backend/src/integrations/telegram/tdjson/qr_login_support/qr.rs` | `render_qr_svg` (размеры 240×240, ошибка `TelegramError::QrGeneration`). |
| `backend/src/integrations/telegram/tdjson/qr_login_support/responses.rs` | Фабрики ответов: `qr_waiting_response`, `qr_preparing_response`, `password_waiting_response`, `ready_response` — их статусы, интервалы опроса, поля. |
| `backend/src/integrations/telegram/tdjson/qr_login_support/types.rs` | `PendingQrLoginMap`, `QrLoginWorkerCompletion`, `TelegramQrLoginSession`, `TelegramQrLoginCommand`, `DrainedQrLoginCommand`, `TelegramQrLoginIdentity` — поля и назначение. |
| `backend/src/integrations/telegram/tdjson/requests.rs` (truncated) | Конструкторы запросов: `set_tdlib_parameters_request` (параметры, `device_model: "Макошь"`), `tdlib_database_directory`, `check_database_encryption_key_request`, запросы чатов, отправки/редактирования/удаления сообщений, реакции, закрепление, ответы, forward (сигнатура). |
| `backend/src/integrations/telegram/tdjson/snapshots.rs` | Перечень и поля всех snapshot-структур (`TelegramTdlibTopicSnapshot`, `ChatSnapshot`, `ChatFolderSnapshot`, `ChatMemberSnapshot`, `MessageSnapshot`, `MessageDeleteSnapshot`, `MessageInteractionInfoSnapshot`, `MessageContentSnapshot`, `MessageEditedSnapshot`, `MessagePinnedSnapshot`, `FileSnapshot`). |
| `backend/src/integrations/telegram/tdjson/tests.rs` | Тестовые модули (`environment`, `parsing_snapshots`, `qr_login_flows`, `request_builders`), вспомогательная функция `test_qr_login_response`. |
| `backend/src/integrations/whatsapp/client.rs` | Публичный интерфейс клиента: реэкспорт модулей и типов. |
| `backend/src/integrations/whatsapp/client/constants.rs` | Константы `WHATSAPP_WEB_*_RECORD_KIND` (13 штук). |
| `backend/src/integrations/whatsapp/client/errors.rs` | Перечисление `WhatsappWebError` и все его варианты (включая прозрачные преобразования). |
| `backend/src/integrations/whatsapp/client/ids.rs` | Функции `whatsapp_web_session_id`, `whatsapp_web_message_id`, `whatsapp_web_raw_record_id` — алгоритм на основе SHA-256 и префиксов версии. |
| `backend/src/integrations/whatsapp/client/models.rs` (truncated) | Модели запросов (`WhatsappWebAccountSetupRequest`, `WhatsappLiveAccountSetupRequest` — валидация), модель сессии (`WhatsappWebSession`, `NewWhatsappWebSession`), перечисления `WhatsappWebCompanionRuntime`, `WhatsappWebLinkState`, сущности `NewWhatsappWebMessage`, `NewWhatsappWebReaction`, `NewWhatsappWebMedia`, `NewWhatsappWebStatus`, `NewWhatsappWebStatusView`, `NewWhatsappWebStatusDelete` и их методы. |
| `backend/src/integrations/whatsapp/client/rows.rs` | `row_to_whatsapp_web_session`, `row_to_whatsapp_web_message`, `provider_channel_message_to_whatsapp_web_message`. |
| `backend/src/integrations/whatsapp/client/store.rs` | Структура `WhatsappWebStore` (поля, конструктор, методы доступа к портам). |
| `backend/src/integrations/whatsapp/client/store/accounts.rs` | `setup_fixture_account`, `setup_live_blocked_account` — логика настройки, `normalize_provider_shape`, `validate_live_provider_kind`, генерация `provider_shape` по умолчанию, `setup_semantics`, `session_mode`, `companion_runtime`. |
| `backend/src/integrations/whatsapp/client/store/evidence.rs` | `link_whatsapp_entity_in_transaction` (делегирование). |
| `backend/src/integrations/whatsapp/client/store/ingestion.rs` (truncated) | Методы `ingest_fixture_message`, `ingest_fixture_reaction`, `ingest_fixture_media`, `ingest_fixture_status`, `ingest_fixture_status_view` — валидация, создание `NewRawCommunicationRecord`, провенанс, обновление `last_sync_at`. |
| `backend/src/integrations/whatsapp/client/store/intelligence.rs` | Заглушка `refresh_message_intelligence_candidates`. |

## Исходные файлы

- [`backend/src/integrations/telegram/tdjson/qr_login/worker_state.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login/worker_state.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/authorization.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/authorization.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/completion.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/completion.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/constants.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/constants.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/identifiers.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/identifiers.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/identity.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/identity.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/pending.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/pending.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/qr.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/qr.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/responses.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/responses.rs)
- [`backend/src/integrations/telegram/tdjson/qr_login_support/types.rs`](../../../../backend/src/integrations/telegram/tdjson/qr_login_support/types.rs)
- [`backend/src/integrations/telegram/tdjson/requests.rs`](../../../../backend/src/integrations/telegram/tdjson/requests.rs)
- [`backend/src/integrations/telegram/tdjson/snapshots.rs`](../../../../backend/src/integrations/telegram/tdjson/snapshots.rs)
- [`backend/src/integrations/telegram/tdjson/tests.rs`](../../../../backend/src/integrations/telegram/tdjson/tests.rs)
- [`backend/src/integrations/whatsapp/client.rs`](../../../../backend/src/integrations/whatsapp/client.rs)
- [`backend/src/integrations/whatsapp/client/constants.rs`](../../../../backend/src/integrations/whatsapp/client/constants.rs)
- [`backend/src/integrations/whatsapp/client/errors.rs`](../../../../backend/src/integrations/whatsapp/client/errors.rs)
- [`backend/src/integrations/whatsapp/client/ids.rs`](../../../../backend/src/integrations/whatsapp/client/ids.rs)
- [`backend/src/integrations/whatsapp/client/models.rs`](../../../../backend/src/integrations/whatsapp/client/models.rs)
- [`backend/src/integrations/whatsapp/client/rows.rs`](../../../../backend/src/integrations/whatsapp/client/rows.rs)
- [`backend/src/integrations/whatsapp/client/store.rs`](../../../../backend/src/integrations/whatsapp/client/store.rs)
- [`backend/src/integrations/whatsapp/client/store/accounts.rs`](../../../../backend/src/integrations/whatsapp/client/store/accounts.rs)
- [`backend/src/integrations/whatsapp/client/store/evidence.rs`](../../../../backend/src/integrations/whatsapp/client/store/evidence.rs)
- [`backend/src/integrations/whatsapp/client/store/ingestion.rs`](../../../../backend/src/integrations/whatsapp/client/store/ingestion.rs)
- [`backend/src/integrations/whatsapp/client/store/intelligence.rs`](../../../../backend/src/integrations/whatsapp/client/store/intelligence.rs)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией или ADR не видно. Встроенные файлы не содержат существующей wiki-документации для сравнения, и внешние артефакты не доступны.
