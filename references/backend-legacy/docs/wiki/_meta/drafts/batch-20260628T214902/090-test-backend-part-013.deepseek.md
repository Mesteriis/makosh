### Summary / Резюме

Добавляется страница `operations/backend-tests.md` с описанием набора интеграционных тестов для бэкенда `makosh`, сфокусированных на Telegram. Страница отсутствовала в wiki, и её создание необходимо для документирования тестового покрытия, структуры тестов и проверяемых сценариев.

### Proposed pages / Предлагаемые страницы

- `operations/backend-tests.md`

```markdown
# Backend Tests

Страница описывает набор интеграционных тестов для бэкенда `makosh`. Все тесты написаны на Rust, используют `tokio`/`axum`/`sqlx` и работают с изолированной БД через `TestContext`.

## Общая структура тестов

- Тесты расположены в `backend/tests/`.
- Каждый файл содержит один или несколько `#[tokio::test]`.
- Для авторизации API-запросов используется заголовок `x-makosh-secret` с константой `LOCAL_API_TOKEN`.
- Основные утилиты:
  - `TestContext` из `testkit` предоставляет `connection_string()`, `pool()` и `database()`.
  - `build_router_with_database` собирает маршрутизатор `axum`.
  - Вспомогательные функции: `json_body`, `json_post_request_with_actor`, `get_request_with_token`, `delete_request_with_token`, `assert_ok`, `post_ok`.

## Категории тестов

### Синхронизация участников (`telegram_members_sync_private`, `telegram_participant_*`)

- `telegram_private_members_sync_uses_tdlib_chat_metadata_and_records_audit`
  - Создаёт учётную запись Telegram с конфигурацией `tdlib_qr_authorized`.
  - Импортирует фиктивное сообщение в приватный чат (`chat_kind: "private"`) через `POST /api/v1/integrations/telegram/fixtures/messages`.
  - Обновляет `telegram_chats.metadata` полями `tdlib_private_user_id` и `tdlib_chat_type`.
  - Вызывает `POST /api/v1/integrations/telegram/provider-sync/conversations/{telegram_chat_id}/members`.
  - Проверяет ответ: `telegram_chat_id`, `synced_count: 1`, `source: "tdlib"`, `provider_member_id: "user:888"`, `sender_display_name: "Alice"`, `role: "member"`, `status: "member"`, `permissions.observed_via: "tdlib.chat.metadata"`.
  - Проверяет, что члены доступны через `GET /api/v1/communications/conversations/{telegram_chat_id}/members?limit=10`.
  - Аудиторский лог (`api_audit_log`, операция `telegram.participants.sync`): `action_class: "read"`, `capability: "telegram.participants.sync"`, `decision: "allowed"`, `reason: "explicit_user_confirmation"`, поля `account_id`, `provider_chat_id`, `synced_count`.
  - Журнал событий (`event_log`):
    - `event_type: 'telegram.participant.updated'` (1 запись для `kind: 'telegram_chat_participant'`).
    - `event_type: 'telegram.sync.started'`, `payload.scope: 'members'` (1 запись).
    - `event_type: 'telegram.sync.progress'`, `payload.status: 'completed'` (1 запись).
    - `event_type: 'telegram.sync.completed'` (1 запись).

- `fixture_account_blocks_members_sync_before_audit_or_events` (`telegram_participant_capability_gates`)
  - Создаёт фиктивную учётную запись (`acct-1`, `provider_kind: "telegram_user"`) без `runtime: "tdlib_qr_authorized"`.
  - Вызывает синхронизацию участников — возвращается `400 BAD_REQUEST`.
  - Проверяет отсутствие записей в `api_audit_log` для операции `telegram.participants.sync`.
  - Проверяет отсутствие записей в `event_log` с типами `telegram.sync.*`.

- `fixture_account_blocks_join_and_leave_before_command_enqueue` (`telegram_participant_capability_gates`)
  - Создаёт фиктивную учётную запись.
  - Вызывает `POST /api/v1/integrations/telegram/provider-commands/conversations/join` — `400 BAD_REQUEST`.
  - Вызывает `POST .../conversations/{telegram_chat_id}/leave` — `400 BAD_REQUEST`.
  - Проверяет, что команды `join`/`leave` не попали в `telegram_provider_write_commands`.

- `telegram_exhaustive_roster_absence_reconciles_self_leave_command` (`telegram_participant_reconciliation_absence`)
  - Записывает команду `leave` со статусом `executing`, `reconciliation_status: "awaiting_provider"`.
  - Вызывает `reconcile_leave_commands_from_exhaustive_absence`.
  - Команда помечается как `completed` с `reconciliation_status: "observed"`.
  - `provider_state.membership_state`: `"absent_exhaustive"`, `observed_via`: `"tdlib.getSupergroupMembers.exhaustive_absence"`.
  - `result_payload.source`: `"tdlib.getSupergroupMembers.exhaustive_absence"`, поле `provider_member_id`.

- `telegram_basic_group_roster_reconciliation_records_observed_source` (`telegram_participant_reconciliation_sources`)
  - Создаёт учётную запись, импортирует сообщение.
  - Вставляет команду `join`.
  - Вызывает `reconcile_join_commands_from_provider_roster_with_source`.
  - Команда завершена, `reconciliation_status: "observed"`, `provider_state.observed_via: "tdlib.getBasicGroupFullInfo"`.

- `telegram_members_route_prefers_provider_roster_over_message_heuristic` (`telegram_participants`)
  - Импортирует три сообщения (два от одного отправителя).
  - Проверяет, что без ростер-участников `/members` возвращает 2 записи с `source: "message_heuristic"`, `message_count: 2` для дублирующегося отправителя.
  - После вставки участника через `insert_provider_participant` (с полями `provider_member_id: "user:42"`, `role: "owner"`, `status: "creator"`) запрос с `query=owner&role=owner` возвращает одну запись с `source: "tdlib"`, `provider_member_id: "user:42"`, `sender_display_name: "Owner User"`, `is_admin: true`, `is_owner: true`, `permissions.can_invite_users: true`, `message_count: 0`.

- `telegram_join_leave_routes_enqueue_provider_write_commands` (`telegram_participants`)
  - Создаёт учётную запись с `runtime: "tdlib_qr_authorized"`.
  - `POST /api/v1/integrations/telegram/provider-commands/conversations/join` возвращает `action: "join"`, `status: "queued"`.
  - `POST .../conversations/{telegram_chat_id}/leave` возвращает `action: "leave"`, `status: "queued"`, `telegram_chat_id` в ответе.
  - Команды записаны в `telegram_provider_write_commands` с соответствующими `command_kind` и `provider_chat_id`.

- `telegram_roster_sync_reconciles_join_only_after_self_member_is_observed` (`telegram_participants`)
  - Создаёт учётную запись, энквеит команду `join`.
  - Функция `telegram_self_provider_member_id` возвращает `Some("user:12345")` для `external_account_id: "telegram:12345"` и `None` для произвольной строки.
  - Вызов `reconcile_join_commands_from_provider_roster` завершает команду: `status: "completed"`, `reconciliation_status: "observed"`, `provider_state.observed_via: "tdlib.getSupergroupMembers"`, `membership_state: "present"`, `provider_member_id: "user:12345"`.

### Жизненный цикл сообщений и гейты возможностей (`telegram_message_lifecycle_*`, `telegram_messages_basic`, `telegram_message_links`)

- `removed_account_blocks_message_lifecycle_and_reaction_routes_before_side_effects` (`telegram_message_lifecycle_capability_gates`)
  - Создаёт учётную запись, сообщение, затем удаляет учётную запись (`DELETE /api/v1/integrations/telegram/accounts/{account_id}`).
  - После удаления аккаунта команды:
    - `edit` → 400
    - `delete` → 400
    - `restore-visibility` → 400
    - `pin` → 400
    - добавление/удаление реакции → 400
  - Проверяет нулевое количество записей в `telegram_provider_write_commands`, `api_audit_log`, `event_log`, `telegram_message_versions`, `telegram_message_tombstones`, `telegram_message_reactions`.
  - Метаданные сообщения (`pinned`/`is_pinned`) не изменились после попыток команд.

- `message_lifecycle_status_events_include_command_identity_for_realtime_command_inserts` (`telegram_message_lifecycle_capability_gates`) — файл обрезан.

- `fixture_account_blocks_message_mark_read_before_side_effects` (`telegram_message_mark_read_capability_gates`)
  - Фиктивная учётная запись (без авторизованного runtime).
  - Команда `mark-read` возвращает 400.
  - Нет записей в `telegram_provider_write_commands`, `event_log` с `event_type: 'telegram.command.status_changed'` для `command_kind: 'mark_read'`.
  - Нет записей в `api_audit_log` для операции `telegram.message.mark_read`.
  - Метаданные чата (`last_read_inbox_provider_message_id`, `unread_count`, `mention_count`) остались без изменений.

- `telegram_manual_send_records_sent_message_and_redacted_provider_write_audit` (`telegram_messages_basic`)
  - Отправка сообщения через `POST /api/v1/integrations/telegram/provider-commands/messages/send` с `actor_id: "legacy-telegram-test-actor"`.
  - Ответ: `account_id`, `provider_chat_id`, `delivery_state: "sent"`, `status: "sent"`, `runtime_kind: "fixture"`, `message_id` начинается с `"message:v4:telegram:"`, `rendered_preview_hash` начинается с `"sha256:"`.
  - Сообщение доступно через `GET /api/v1/communications/messages`.
  - Аудит (`api_audit_log`, операция `telegram.message.send`): `action_class: "provider_write"`, `capability: "telegram.message.send"`, `decision: "allowed"`, `reason: "explicit_user_confirmation"`, `confirmation_required: false`, поля `account_id`, `provider_chat_id`, `rendered_preview_hash`. Поля `text`, `message_text`, `rendered_text` отсутствуют (текст сообщения не записан в аудит).
  - Отправка через provider-neutral endpoint `POST /api/v1/communications/conversations/{chat_id}/messages` возвращает `message_id` с префиксом `"message:v4:telegram:"`, непустой `raw_record_id`, `channel_kind: "telegram"`, `provider: "telegram"`, `status: "sent"`.

- `telegram_raw_message_endpoint_returns_sanitized_source_evidence` (`telegram_messages_basic`)
  - Создаёт `raw_record` с полем `tdlib_raw`, содержащим `api_hash` и `token`, и `provenance` с полем `password`.
  - После диспатча и консумации сообщения запрос `GET /api/v1/communications/messages/{message_id}/raw-evidence` возвращает:
    - `raw_record_id`, `provider_record_id`, `payload.text`.
    - `tdlib_raw.@type: "message"`.
    - `tdlib_raw.nested.api_hash`: `"[redacted]"`, `tdlib_raw.nested.token`: `"[redacted]"`.
    - `provenance.password`: `"[redacted]"`.

- `telegram_fixture_sync_chats_returns_account_chat_metadata` (`telegram_messages_basic`) — файл обрезан.

- `telegram_message_ingestion_projects_public_message_link_without_erasing_chat_username` (`telegram_message_links`)
  - Создаёт публичный канал (`TelegramChatKind::Channel`) с `username: "МакошьPublicChannel"`.
  - Импортирует фиктивное сообщение.
  - Чат сохраняет `username` после импорта.
  - Запись `observations` типа `TELEGRAM_CHAT` с `relationship_kind: "upsert"` и `payload.username: "МакошьPublicChannel"` присутствует.
  - Спроецированное сообщение имеет `metadata.message_link: "https://t.me/МакошьPublicChannel/4242"` и `metadata.message_link_kind: "public_t_me"`.

### Реакции и ссылки (`telegram_reactions`, `telegram_reference_idempotency`)

- `telegram_provider_reactions_reconcile_react_and_unreact_commands` (`telegram_reactions`)
  - Вставляет команды `react` (👍), `unreact` (🔥), `react` (😎).
  - Вызывает `reconcile_reaction_commands_from_provider_reactions` с массивом реакций провайдера `["👍"]`.
  - Команда `tcmd_react_observed` → `completed` + `observed`.
  - Команда `tcmd_unreact_observed` → `completed` + `observed` (отсутствие реакции в списке провайдера трактуется как успех unreact).
  - Команда `tcmd_react_still_pending` → `failed` + `mismatch`, `last_error: "Provider observed a different reaction state than requested"`, `provider_state.expected_is_chosen: true`, `observed_is_chosen: false`.

- `telegram_reference_inserts_return_existing_rows_on_conflict` (`telegram_reference_idempotency`)
  - Создаёт root, reply и forward сообщения.
  - `insert_reply_ref` для одной пары сообщений дважды — возвращает один и тот же `reply_ref_id`.
  - `insert_forward_ref` с одинаковыми параметрами дважды — возвращает один и тот же `forward_ref_id`.

### Сверка (reconciliation) и outbox (`telegram_message_realtime`, `telegram_outbox`)

- `telegram_provider_delete_observation_is_idempotent_and_reconciles_delete_command` (`telegram_message_realtime`)
  - Вставляет команду `delete`.
  - Два вызова `record_provider_delete_observation` создают один `tombstone` (тот же `tombstone_id`, `reason_class: "deleted_by_provider"`, `actor_class: "provider"`, `is_local_visible: false`).
  - `reconcile_delete_commands_from_provider_state` завершает исходную команду: `status: "completed"`, `reconciliation_status: "observed"`.
  - Для `tombstone` создаётся наблюдение (`observation_links`) типа `TELEGRAM_MESSAGE_TOMBSTONE` с `relationship_kind: "provider_delete"`.

- `telegram_provider_edit_observation_is_idempotent_and_reconciles_edit_command` (`telegram_message_realtime`)
  - Вставляет команду `edit` с `new_text: "after"`.
  - Два вызова `record_provider_edit_observation` создают одну версию (`version_id` одинаковый, `body_text: "after"`, `source_event: "updateMessageContent"`).
  - `reconcile_edit_commands_from_provider_state` завершает команду: `status: "completed"`, `reconciliation_status: "observed"`.
  - Для версии создаётся наблюдение типа `TELEGRAM_MESSAGE_VERSION` с `relationship_kind: "insert"`.

- `telegram_provider_edit_observation_marks_mismatched_edit_command_failed` (`telegram_message_realtime`)
  - Вставляет команду `edit` с `new_text: "expected provider body"`.
  - `reconcile_edit_commands_from_provider_state` с фактическим текстом `"different provider body"` → `status: "failed"`, `reconciliation_status: "mismatch"`, `last_error: "Provider observed a different message body than requested"`.

- `telegram_outbox_claims_due_command_and_unlocks_while_awaiting_provider` (`telegram_outbox`)
  - Вставляет команду `edit`.
  - `claim_due_commands_for_execution` возвращает команду со статусом `executing`, `retry_count: 1`, `locked_by: "telegram-outbox-worker"`, `reconciliation_status: "awaiting_provider"`.
  - После `mark_command_awaiting_provider` команда сохраняет статус `executing`, `reconciliation_status: "awaiting_provider"`, но `locked_at` и `locked_by` становятся `None` (команда разблокирована).

- `telegram_outbox_recovers_stale_locked_execution_for_retry` (`telegram_outbox`)
  - Вставляет и блокирует команду, затем вручную сдвигает `locked_at` на 10 минут назад.
  - `recover_stale_executing_commands` с порогом 2 минуты возвращает команду в статусе `retrying` с установленным `next_attempt_at`, без блокировки.

- `telegram_outbox_dead_letter_can_be_manually_retried` (`telegram_outbox`)
  - Вставляет команду, вызывает `dead_letter_command` → статус `dead_letter`, установлено `dead_lettered_at`.
  - `manual_retry_command` переводит команду в `retrying` со сбросом `retry_count` в 0, без `dead_lettered_at`, `reconciliation_status: "not_observed"`.

### Логин и рантайм Telegram (`telegram_qr_login`)

- `telegram_qr_login_start_reports_tdlib_runtime_unavailable`
  - Приложение собрано с `with_test_tdjson_path` на несуществующий файл.
  - `POST /api/v1/integrations/telegram/login/qr/start` возвращает 503 с `error: "telegram_tdlib_runtime_unavailable"`.

- `telegram_qr_login_start_uses_configured_app_credentials_when_payload_omits_them`
  - Приложение собрано с `with_test_telegram_app_credentials(12345, "telegram-api-hash")`, в теле запроса нет `api_id`/`api_hash`.
  - Ответ также 503, ошибка `telegram_tdlib_runtime_unavailable`.

- `telegram_live_smoke_syncs_configured_account_when_explicitly_enabled`
  - Выполняется только при `MAKOSH_TELEGRAM_LIVE_SMOKE=1`.
  - Использует реальные TDLib-путь, API-ключи, `account_id`, `chat_id` из переменных окружения.
  - `POST /api/v1/integrations/telegram/runtime/start` возвращает `runtime_kind: "tdlib_qr_authorized"`, `status: "running"`.
  - `POST /api/v1/integrations/telegram/provider-sync/history` возвращает `status: "synced"`, `runtime_kind: "tdlib_qr_authorized"`.

- `telegram_qr_login_status_unknown_setup_returns_json_not_found`
  - `GET /api/v1/integrations/telegram/login/qr/missing-setup` → 404, `error: "telegram_qr_login_not_found"`.

- `telegram_qr_login_password_unknown_setup_returns_json_not_found`
  - `POST .../missing-setup/password` → 404, `error: "telegram_qr_login_not_found"`.

- `telegram_qr_login_cancel_unknown_setup_returns_json_not_found`
  - `DELETE .../missing-setup` → 404, `error: "telegram_qr_login_not_found"`.
```

### Source coverage / Покрытие источников

- `backend/tests/telegram_members_sync_private.rs`
  - Тест синхронизации приватных участников через TDLib: создание аккаунта, импорт сообщения, обновление `telegram_chats.metadata` полями `tdlib_private_user_id`/`tdlib_chat_type`, вызов sync endpoint, проверка ответа (`synced_count`, `source: "tdlib"`, `provider_member_id`, `sender_display_name`, `role`, `status`, `permissions.observed_via`), проверка наличия члена через GET `/members`, аудит `telegram.participants.sync`, события `telegram.participant.updated`, `telegram.sync.started`, `telegram.sync.progress`, `telegram.sync.completed`.

- `backend/tests/telegram_message_lifecycle_capability_gates.rs` (обрезан)
  - Удалённый аккаунт блокирует команды `edit`, `delete`, `restore-visibility`, `pin`, реакции — статус 400, отсутствие записей в `telegram_provider_write_commands`, `api_audit_log`, `event_log`, `telegram_message_versions`, `telegram_message_tombstones`, `telegram_message_reactions`. Метаданные сообщения не изменены.

- `backend/tests/telegram_message_links.rs`
  - Публичный канал: после импорта сохраняется `username`, создаётся observation `TELEGRAM_CHAT`. Сообщение получает `message_link` вида `https://t.me/МакошьPublicChannel/4242` и `message_link_kind: "public_t_me"`.

- `backend/tests/telegram_message_mark_read_capability_gates.rs`
  - Фиктивный аккаунт блокирует `mark-read`: 400, нет команд, нет событий, нет аудита, метаданные чата (`last_read_inbox_provider_message_id`, `unread_count`, `mention_count`) не изменяются.

- `backend/tests/telegram_message_realtime.rs` (обрезан)
  - Идемпотентность `record_provider_delete_observation` (тот же `tombstone_id`, `reason_class: "deleted_by_provider"`), сверка команды `delete` (completed, observed). Наблюдение для tombstone. Идемпотентность `record_provider_edit_observation` (одна версия, `body_text`), сверка команды `edit`. Mismatch edit: `failed`, ошибка о несовпадении тела сообщения.

- `backend/tests/telegram_messages_basic.rs` (обрезан)
  - Ручная отправка через `/provider-commands/messages/send`: ответ с `delivery_state: "sent"`, `status: "sent"`, `runtime_kind: "fixture"`, message id с префиксом `message:v4:telegram:`, `rendered_preview_hash`. Сообщение доступно в `/messages`. Аудит: операция `telegram.message.send`, `action_class: "provider_write"`, текст отсутствует в аудите. Provider-neutral send через `/communications/conversations/{chat_id}/messages`. Raw-evidence endpoint: `tdlib_raw.nested.api_hash`, `token`, `provenance.password` заменены на `[redacted]`.

- `backend/tests/telegram_outbox.rs`
  - `claim_due_commands_for_execution` блокирует команду (`executing`, `locked_by`), `mark_command_awaiting_provider` разблокирует. `recover_stale_executing_commands` переводит команду в `retrying`. `manual_retry_command` восстанавливает из `dead_letter`.

- `backend/tests/telegram_participant_capability_gates.rs`
  - Фиктивный аккаунт блокирует `members/sync` (400, нет аудита, нет событий `telegram.sync.*`) и команды `join`/`leave` (400, нет записей в `telegram_provider_write_commands`).

- `backend/tests/telegram_participant_reconciliation_absence.rs`
  - `reconcile_leave_commands_from_exhaustive_absence` завершает команду `leave`, `provider_state.membership_state: "absent_exhaustive"`, `observed_via: "tdlib.getSupergroupMembers.exhaustive_absence"`, `result_payload.source: "tdlib.getSupergroupMembers.exhaustive_absence"`.

- `backend/tests/telegram_participant_reconciliation_sources.rs`
  - `reconcile_join_commands_from_provider_roster_with_source` завершает команду, `provider_state.observed_via: "tdlib.getBasicGroupFullInfo"`, `result_payload.source: "tdlib.getBasicGroupFullInfo"`.

- `backend/tests/telegram_participants.rs` (обрезан)
  - Маршрут `/members` предпочитает provider roster (source: `tdlib`) над message heuristic. Join/leave энквеят команды (status: `queued`). Сверка join-команды через `reconcile_join_commands_from_provider_roster`; `telegram_self_provider_member_id` возвращает `user:{id}` для корректного `external_account_id`.

- `backend/tests/telegram_qr_login.rs`
  - Недоступность TDLib: 503, `telegram_tdlib_runtime_unavailable`. Использование сконфигурированных app credentials без передачи в payload. Live smoke test (включается переменной `MAKOSH_TELEGRAM_LIVE_SMOKE`): `runtime/start` → running, `provider-sync/history` → synced. Неизвестный setup: 404, `telegram_qr_login_not_found`.

- `backend/tests/telegram_reactions.rs`
  - Сверка реакций: команда `react` с совпадающей реакцией → completed/observed; `unreact` при отсутствии реакции в списке → completed/observed; `react` с несовпадающей → failed/mismatch с ошибкой `Provider observed a different reaction state than requested` и деталями `expected_is_chosen`/`observed_is_chosen`.

- `backend/tests/telegram_reference_idempotency.rs`
  - Повторные вызовы `insert_reply_ref` и `insert_forward_ref` с одинаковыми параметрами возвращают существующие `reply_ref_id`/`forward_ref_id`, а не создают дубликаты.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения кода, документации и ADR не видны. Файлы исходников не содержат комментариев или ссылок, позволяющих судить о рассинхроне с внешней документацией.
