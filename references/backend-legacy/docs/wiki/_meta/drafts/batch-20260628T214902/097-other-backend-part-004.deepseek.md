### Summary / Резюме

Добавить в страницу `components/backend.md` документацию схемы базы данных, вводимой миграциями 0076–0100. В контексте присутствуют только SQL‑миграции; страница будет описывать объявленные таблицы, колонки, ограничения, индексы и связи, без домысливания поведения сервисов, API или конвенций вне этих миграций. Текст на русском, в формате Obsidian‑совместимого Markdown.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend

Компонент `backend` — серверная часть Макошь.
В файлах миграций (0076–0100) определена структура базы данных: таблицы для исходящей электронной почты, сохранённых поисков, вложений, пользовательских папок, AI‑состояний почты, уведомлений о прочтении, жизненного цикла сообщений Telegram, реакций, ссылок на ответы/пересылки, тем форума, участников чатов, очередей событий, канонической системы evidence‑review, а также связи между коммуникационными записями и наблюдениями (observations).

## Схема базы данных (миграции 0076–0100)

### Обзор миграций

| Миграция | Назначение |
|---|---|
| 0076 | Таблица `email_outbox_tracking` — отслеживание исходящих писем |
| 0077 | `mail_saved_searches` — сохранённые поиски и умные папки |
| 0078 | Индексы для поиска вложений (`communication_attachments`) |
| 0079 | `mail_folders` + `mail_folder_messages` — пользовательские папки с операциями copy/move |
| 0080 | `mail_ai_states` — AI‑состояния для сообщений (NEW, PROCESSING, …) |
| 0081 | `mail_read_receipts` — квитанции о прочтении почты |
| 0082 | Telegram: версии сообщений, надгробия (`tombstones`), команды записи провайдера |
| 0083 | `telegram_message_reactions` — реакции на сообщения Telegram |
| 0084 | `telegram_message_reply_refs` и `telegram_message_forward_refs` — ссылки на ответы и пересылки |
| 0085–0092 | Расширение допустимых значений `command_kind` в таблице `telegram_provider_write_commands` (mark_unread, topic_create/close/reopen, folder_add, folder_remove) |
| 0086 | `telegram_topics` — темы форума Telegram |
| 0087 | Расширение `telegram_provider_write_commands` полями для выполнения, сверки и dead‑letter |
| 0088 | `communication_attachment_imports` — локальный импорт вложений до появления сообщения провайдера |
| 0089 | `telegram_chat_participants` — проекция участников чатов Telegram |
| 0093 | `event_consumers`, `event_consumer_failures`, `event_consumer_processed_events`, `event_dead_letters` — инфраструктура событийной очереди с dead‑letter |
| 0094 | Таблицы системы review‑evidence: `observation_kind_definitions`, `observations`, `observation_links`, `observation_ingestion_runs`, `review_items`, `review_item_evidence`, `context_packs` (частично) |
| 0095 | Добавление `provenance_kind` и `provenance_id` в таблицу `tasks` |
| 0096 | Расширение допустимых `source_type` в `tasks` |
| 0097–0100 | Связывание `communication_raw_records`, `communication_messages`, `decision_evidence`, `obligation_evidence`, `relationship_evidence`, `task_candidates` с таблицей `observations` через внешний ключ `observation_id` |

### Электронная почта

#### `email_outbox_tracking`
Отслеживание исходящих писем.
Ключевые колонки: `outbox_id` (PK), ссылка на аккаунт (`account_id`), проект письма (`draft_id`), получатели (JSONB‑массивы `to_recipients`, `cc_recipients`, `bcc_recipients`), `subject`, тела (`body_text`, `body_html`), `status` (queued, scheduled, sending, sent, failed, canceled), времена плановой отправки и окна отмены (`scheduled_send_at`, `undo_deadline_at`), счётчик попыток, время захвата и отправки, `provider_message_id`, `last_error`, `metadata`, временные штампы.
Индексы: `email_outbox_tracking_account_status_idx`, `email_outbox_tracking_due_idx`.

#### `mail_saved_searches`
Сохранённые поиски и умные папки.
Колонки: `saved_search_id` (PK), имя, описание, аккаунт, `query_text`, `workflow_state` (new, reviewed, needs_action, …), `local_state` (active, trash, all), `channel_kind`, `is_smart_folder`, `sort_order`.
Индексы: `mail_saved_searches_account_smart_idx`, `mail_saved_searches_smart_idx`.

#### Индексы вложений
Добавлены индексы для таблицы `communication_attachments`: `communication_attachments_search_order_idx`, `communication_attachments_scan_status_idx`, `communication_attachments_content_type_idx`.

#### `mail_folders` и `mail_folder_messages`
Пользовательские папки (`mail_folders`): `folder_id` (PK), `account_id`, `name` (уникальность с учётом аккаунта), `description`, `color`, `sort_order`.
Связка папок и сообщений (`mail_folder_messages`): составной первичный ключ `(folder_id, message_id)`, `added_at`, `last_operation` (copy или move). Сообщения удаляются каскадно при удалении ссылочных записей.
Индексы: `mail_folder_messages_message_idx`, `mail_folder_messages_folder_order_idx`.

#### `mail_ai_states`
AI‑состояния сообщений (по связи с `communication_messages` через `message_id`).
Состояния: `NEW`, `PROCESSING`, `PROCESSED`, `REVIEW_REQUIRED`, `FAILED`, `ARCHIVED`.
Индекс: `mail_ai_states_state_updated_idx`.
При создании таблицы производится заполнение всех существующих сообщений состоянием `'NEW'`.

#### `mail_read_receipts`
Квитанции о прочтении почты.
Колонки: `receipt_id` (PK), `account_id`, `outbox_id`, `provider_message_id`, `recipient`, `receipt_kind` (read), `read_at`, `source_kind` (mdn), `provider_record_id` (уникальный при ненулевом значении на аккаунт), ссылка на сырую запись, метаданные.
Индексы: `mail_read_receipts_provider_record_unique_idx`, `mail_read_receipts_outbox_read_at_idx`, `mail_read_receipts_provider_message_idx`.

### Telegram

#### `telegram_message_versions`
История версий сообщений — только добавление.
Ключевые колонки: `version_id` (PK), `message_id`, `account_id`, `provider_message_id`, `provider_chat_id`, `version_number`, `body_text`, `edit_timestamp`, `source_event`, `raw_diff_payload`, `provenance`.
Ограничение `version_number > 0`. Уникальность на `(message_id, version_number)`.
Индексы: по `message_id`, по `(account_id, provider_chat_id, provider_message_id)`.

#### `telegram_message_tombstones`
Надгробия для удалённых или скрытых сообщений.
Колонки: `tombstone_id` (PK), `message_id`, `account_id`, `provider_message_id`, `provider_chat_id`, `reason_class` (deleted_by_owner, …, unknown), `actor_class` (owner, provider, automation, system, unknown), `observed_at`, `is_provider_delete`, `is_local_visible`, метаданные.
Индексы: по `message_id`, по `(account_id, provider_chat_id)`.

#### `telegram_provider_write_commands`
Команды записи к провайдеру Telegram.
Версия после миграции 0087 включает расширенные поля: `next_attempt_at`, `last_attempt_at`, `locked_at`, `locked_by`, `provider_observed_at`, `provider_state` (JSONB), `reconciliation_status` (not_observed, awaiting_provider, observed, mismatch, not_required), `reconciled_at`, `dead_lettered_at`.
Допустимые значения `status`: queued, executing, completed, failed, retrying, cancelled, dead_letter.
`command_kind` допускает: send_text, send_media, edit, delete, restore_visibility, mark_read, mark_unread, pin, unpin, archive, unarchive, mute, unmute, react, unreact, reply, forward, join, leave, folder_add, folder_remove, topic_create, topic_close, topic_reopen, admin_action.
Уникальность на `(account_id, idempotency_key)`.
Индексы: `telegram_provider_write_commands_account_idx`, `telegram_provider_write_commands_chat_idx`, `telegram_provider_write_commands_idempotency_idx`, а также `due_idx` и `reconciliation_idx`.

#### `telegram_message_reactions`
Реакции на сообщения.
Колонки: `reaction_id` (PK), `message_id`, `account_id`, `provider_message_id`, `provider_chat_id`, `sender_id`, `sender_display_name`, `reaction_emoji`, `is_active`, `observed_at`, `source_event`, `provider_actor_id`, метаданные.
Уникальное ограничение на `(message_id, sender_id, reaction_emoji)`.
Индексы: по `message_id`, по `(account_id, provider_chat_id, provider_message_id)`.

#### `telegram_message_reply_refs` и `telegram_message_forward_refs`
Ссылки на ответы (`telegram_message_reply_refs`): источник и цель (`source_message_id`, `target_message_id`), `reply_depth` (>0), `is_topic_reply`, `topic_id`. Уникальность на пару источник‑цель.
Ссылки на пересылки (`telegram_message_forward_refs`): `source_message_id`, `forward_origin_*` (чат, сообщение, отправитель), `forward_date`, `forward_depth` (>0). Уникальность на `(source_message_id, account_id)`.
Индексы для обоих таблиц по ключевым колонкам.

#### `telegram_topics`
Темы форума Telegram.
Колонки: `topic_id` (PK), `telegram_chat_id` (ссылка на `telegram_chats`), `account_id`, `provider_topic_id` (BIGINT), `provider_chat_id`, `title`, `icon_emoji`, `is_pinned`, `is_closed`, `unread_count`, `last_message_at`, метаданные.
Уникальность на `(telegram_chat_id, provider_topic_id)`.
Индексы: по чату, по аккаунту, по `forum_topic_id` в метаданных сообщений.

#### `telegram_chat_participants`
Проекция участников чата Telegram.
Колонки: `participant_id` (PK), `telegram_chat_id`, `account_id`, `provider_chat_id`, `provider_member_id`, `display_name`, `username`, `role`, `status`, `is_admin`, `is_owner`, `permissions` (JSONB), `raw_payload` (JSONB), `source` (tdlib или bot_api), `observed_at`.
Уникальность на `(telegram_chat_id, provider_member_id)`.
Индексы: по чату, по аккаунту, поисковый по имени/username/идентификатору.

### Импорт вложений

#### `communication_attachment_imports`
Локальный импорт вложений до создания сообщения провайдера.
Колонки: `attachment_id` (PK), `account_id`, `channel_kind`, `blob_id` (ссылка на `communication_mail_blobs`), `filename`, `content_type`, `size_bytes` (>0), `sha256` (регулярное выражение `sha256:[0-9a-f]{64}`), `source_kind` (по умолчанию `local_import`), `imported_by`, `scan_status` (not_scanned, clean, suspicious, malicious, failed), поля сканирования, метаданные.
Индексы: по аккаунту, по `blob_id`, по `sha256`.

### Событийная инфраструктура

#### `event_consumers`, `event_consumer_failures`, `event_consumer_processed_events`, `event_dead_letters`
- **event_consumers** — потребители событий: `consumer_name` (PK), `last_processed_position`, `status` (active, paused, disabled), блокировка.
- **event_consumer_failures** — неудачные обработки: составной PK `(consumer_name, event_position)`, `event_id`, `event_type`, счётчик попыток (>0), время следующей попытки, ошибка.
- **event_consumer_processed_events** — успешно обработанные события: составной PK `(consumer_name, event_position)`, уникальность на `(consumer_name, event_id)`.
- **event_dead_letters** — dead‑letter очередь: `dead_letter_id` (PK), `consumer_name`, `event_position` (уникальность на потребителе+позиции), `event_id`, `event_type`, количество попыток, ошибка, `event_payload` (JSONB), `review_state` (open, replay_requested, replayed, dismissed), поля повторного воспроизведения.

### Каноническая система evidence‑review

#### `observation_kind_definitions`
Справочник видов наблюдений.
Колонки: `kind_definition_id` (PK), код (верхний регистр), имя, версия (>0), категория, описание. Уникальность на `(code, version)`.
Предзаполненные значения: `COMMUNICATION_MESSAGE`, `COMMUNICATION_MESSAGE_DELETED`, `COMMUNICATION_ATTACHMENT`, `MEETING`, `MEETING_RECORDING`, `MEETING_TRANSCRIPT`, `DOCUMENT`, `VOICE_RECORDING`, `BROWSER_CAPTURE`, `CONTACT_RECORD`, `CALENDAR_EVENT`, `CALENDAR_EVENT_DELETED`.

#### `observations`
Наблюдения — только добавление (триггеры `observations_append_only_update`/`_delete`).
Колонки: `observation_id` (PK), ссылка на `kind_definition_id`, `origin_kind` (vault_source, manual, browser_capture, voice_memo, file_import, local_runtime, test_fixture), `vault_source_id`, `observed_at`, `captured_at`, `payload` (JSONB), `confidence` (0..1), `content_hash`, `source_ref`, `provenance` (JSONB).
Индексы: по определению вида и времени фиксации, по `vault_source_id`, по `source_ref`, по `content_hash`.

#### `observation_links`
Связи наблюдений с другими сущностями.
Составной PK `(observation_id, domain, entity_kind, entity_id, relationship_kind)`. Поле `relationship_kind` по умолчанию `'evidence_for'`. `confidence` в диапазоне 0..1.

#### `observation_ingestion_runs`
Запуски обработки наблюдения: `ingestion_run_id` (PK), пайплайн, статус (running, succeeded, failed, skipped), начало/окончание, выходные данные, ошибка.

#### `review_items`
Объекты на ревью: `review_item_id` (PK), `item_kind` (new_person, new_organization, potential_task, potential_obligation, potential_decision, potential_relationship, potential_project, knowledge_candidate), заголовок, краткое описание, статус (new, in_review, approved, promoted, dismissed, archived), целевая сущность (три поля: domain, entity_kind, entity_id — либо все заданы, либо все NULL), `confidence` (0..1), метаданные.

#### `review_item_evidence`
Связь объекта ревью с наблюдением: составной PK `(review_item_id, observation_id, evidence_role)`. `evidence_role` по умолчанию `'primary'`.

Таблица `context_packs` обрезана в предоставленном контексте; детали не подтверждены.

### Задачи (tasks)

Миграции 0095 и 0096 изменяют таблицу `tasks`:
- Добавлены `provenance_kind` (observation, review_item, decision, obligation) и `provenance_id`. Выполнено заполнение на основе `source_kind`/`source_id`.
- Расширен набор допустимых значений `source_type` (добавлены `communication`, `email`, `telegram`, `whatsapp`, и т.д.; итоговый список в чек‑констрейнте).

### Связи с observations (миграции 0097–0100)

- `communication_raw_records`: добавлена колонка `observation_id` (NOT NULL, внешний ключ к `observations`), заполнена через генерацию `observation_id` из `raw_record_id` и создание записей в `observations` (со ссылкой на `observation_kind_definitions` с кодом `COMMUNICATION_MESSAGE` или `COMMUNICATION_ATTACHMENT`).
- `communication_messages`: добавлена `observation_id` (NOT NULL) через связь с `raw_records`, внешний ключ.
- `decision_evidence`, `obligation_evidence`, `relationship_evidence`: добавлен `observation_id` (внешний ключ), заполнен по соответствию `source_id == observation.observation_id`, добавлены чек‑ограничения для согласованности `source_kind = 'observation'` с `observation_id`.
- `task_candidates`: добавлен `observation_id` (внешний ключ), заполнен для `source_kind = 'message'` через `communication_messages.observation_id`, чек‑ограничение: если `source_kind = 'message'`, то `observation_id` обязателен.

## Примечание

Поведение сервисов, API и конвенции, выходящие за рамки приведённых SQL‑определений, не подтверждены данным контекстом.
```

### Source coverage / Покрытие источников

| Исходный файл | Упомянутые факты |
|---|---|
| `backend/migrations/0076_create_email_outbox_tracking.sql` | Таблица `email_outbox_tracking`, её колонки, ограничения статуса и JSONB-массивов, индексы |
| `backend/migrations/0077_create_mail_saved_searches.sql` | Таблица `mail_saved_searches`, колонки, ограничения `query_not_blank_or_filters`, `workflow_state`, `local_state`, индексы |
| `backend/migrations/0078_add_attachment_search_indexes.sql` | Три индекса на `communication_attachments` |
| `backend/migrations/0079_create_mail_custom_folders.sql` | Таблицы `mail_folders`, `mail_folder_messages`, их колонки, ограничения, индексы |
| `backend/migrations/0080_create_mail_ai_states.sql` | Таблица `mail_ai_states`, состояния, заполнение начальными значениями |
| `backend/migrations/0081_create_mail_read_receipts.sql` | Таблица `mail_read_receipts`, колонки, ограничения, индексы |
| `backend/migrations/0082_create_telegram_message_lifecycle_schema.sql` | `telegram_message_versions`, `telegram_message_tombstones`, `telegram_provider_write_commands` (исходная версия) |
| `backend/migrations/0083_create_telegram_reactions.sql` | `telegram_message_reactions` |
| `backend/migrations/0084_create_telegram_reply_forward_refs.sql` | `telegram_message_reply_refs`, `telegram_message_forward_refs` |
| `backend/migrations/0085_allow_mark_unread_telegram_command.sql` | Добавление `mark_unread` в `command_kind` check |
| `backend/migrations/0086_create_telegram_topics.sql` | `telegram_topics`, индекс на `communication_messages` по `forum_topic_id` |
| `backend/migrations/0087_extend_telegram_provider_write_outbox.sql` | Расширение `telegram_provider_write_commands` полями выполнения/выверки, добавление `dead_letter` в статусы, новые индексы |
| `backend/migrations/0088_create_communication_attachment_imports.sql` | `communication_attachment_imports` |
| `backend/migrations/0089_create_telegram_chat_participants.sql` | `telegram_chat_participants` |
| `backend/migrations/0090_restore_topic_telegram_command_kinds.sql` | Добавление `topic_create`, `topic_close`, `topic_reopen` в `command_kind` |
| `backend/migrations/0091_add_telegram_folder_add_command_kind.sql` | `folder_add` |
| `backend/migrations/0092_add_telegram_folder_remove_command_kind.sql` | `folder_remove` |
| `backend/migrations/0093_create_event_consumers_dlq.sql` | `event_consumers`, `event_consumer_failures`, `event_consumer_processed_events`, `event_dead_letters` |
| `backend/migrations/0094_create_canonical_evidence_review_context.sql` (первые 12000 симв.) | `observation_kind_definitions`, `observations`, триггеры append-only, `observation_links`, `observation_ingestion_runs`, `review_items`, `review_item_evidence`; `context_packs` обрезана – детали не подтверждены |
| `backend/migrations/0095_add_task_provenance.sql` | `provenance_kind`/`provenance_id` в `tasks`, заполнение, чек‑ограничения |
| `backend/migrations/0096_expand_task_source_type_for_observation_spine.sql` | Расширение `source_type` в `tasks` |
| `backend/migrations/0097_link_communication_raw_records_to_observations.sql` | Добавление `observation_id` в `communication_raw_records`, заполнение, внешний ключ |
| `backend/migrations/0098_link_communication_messages_to_observations.sql` | Добавление `observation_id` в `communication_messages`, заполнение через `raw_records`, внешний ключ |
| `backend/migrations/0099_link_domain_evidence_to_observations.sql` | Добавление `observation_id` в `decision_evidence`, `obligation_evidence`, `relationship_evidence`, заполнение, чек‑ограничения, внешние ключи |
| `backend/migrations/0100_link_task_candidates_to_observations.sql` | Добавление `observation_id` в `task_candidates`, заполнение для `source_kind = 'message'`, чек‑ограничение, внешний ключ |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста (SQL‑миграции 0076–0100) несоответствий между кодом миграций и документацией не выявлено. Все миграции последовательны: расширения таблицы `telegram_provider_write_commands` (0085–0092) постепенно добавляют новые допустимые значения `command_kind` и колонки (0087), не противореча друг другу; ссылочная целостность на `observation_kind_definitions`, `observations` и другие таблицы соблюдена. Таблица `context_packs` (миграция 0094) обрезана — при наличии полного определения возможен дополнительный дрифт, но в рамках видимого текста он не обнаружен.
```
