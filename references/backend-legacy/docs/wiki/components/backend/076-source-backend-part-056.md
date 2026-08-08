---
chunk_id: 076-source-backend-part-056
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 22
generated_by: code-wiki-ru
---

# 076-source-backend-part-056 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `22`

## Резюме

Страница `components/backend.md` должна быть создана или обновлена, чтобы задокументировать структуру и ответственность рабочих процессов (`workflows`) backend-компонента Макошь. На основе предоставленных исходных файлов описываются модули, их ключевые функции и роль в системе. Это улучшит архитектурную документацию русскоязычной Obsidian-вики.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend (Серверная часть)

## Обзор

Серверная часть Макошь реализует набор **рабочих процессов** (workflows), расположенных в `backend/src/workflows/`. Каждый процесс отвечает за определённую доменную операцию: обработку событий, синхронизацию данных, генерацию кандидатов для ревью, выполнение внешних команд и продвижение ревью в доменные сущности. Все процессы используют пул соединений PostgreSQL (`PgPool`) для взаимодействия с хранилищем.

## Состав модулей

В файле `backend/src/workflows/mod.rs` объявлены следующие публичные модули рабочих процессов:

- `consistency_review`
- `email_fixture_pipeline`
- `email_intelligence`
- `email_sync_pipeline`
- `graph_projection`
- `mail_background_sync`
- `person_derived_evidence`
- `project_link_review_effects`
- `realtime_conversation_memory_pipeline`
- `realtime_conversation_radar_projection`
- `realtime_conversation_transcript_execution`
- `realtime_conversation_transcript_projection`
- `review_inbox`
- `review_mirror`
- `review_promotion`
- `task_creation`
- `telegram_media_storage`
- `workflow_action_person_projection`
- `yandex_telemost_calendar_matching`
- `zoom_calendar_matching`
- `zoom_participant_identity`
- `zoom_signal_detection`

Далее детально описаны те модули, для которых доступны исходные файлы в текущем контексте.

## `mail_background_sync` — фоновая синхронизация почты

Управляет асинхронными циклами синхронизации почтовых аккаунтов (Gmail, iCloud, IMAP).

### Хранилище (`store`)

Точка входа — `MailSyncStore` (публичный алиас `MailSyncStatePort`), содержащий `PgPool`.

Подмодули и их публичные методы:

- **`account`** — `require_account(account_id)` проверяет существование аккаунта в `communication_provider_accounts`. При отсутствии возвращает `MailSyncError::AccountNotFound`.
- **`orphaned`** — `mark_orphaned_active_runs_failed(now)` переводит все активные (`queued`, `running`, `recoverable_full_resync_needed`) синхронизации в статус `failed` с кодом `backend_restarted` и создаёт observation-событие для каждого.
- **`run_start`** — `start_run(account_id, trigger, settings, checkpoint_before)` создаёт новый запуск синхронизации (`INSERT INTO communication_mail_sync_runs`) со статусом `running`, фазой `listing` и режимом прогресса `indeterminate`. При уникальном нарушении возвращает `RunAlreadyActive`. Генерирует событие `sync_run_started_event`.
- **`run_progress`** — `update_progress(ProgressUpdate)` обновляет статус, фазу, процент и счётчики сообщений для активного запуска, генерирует `sync_run_progress_event`. `mark_recoverable_full_resync(run_id, error_code)` переводит запуск в статус `recoverable_full_resync_needed` (например, при истечении истории Gmail).
- **`run_finish`** — `finish_run(run_id, FinishRun)` завершает запуск, устанавливает итоговые показатели, код ошибки, время завершения и `next_run_at`. Генерирует событие `sync_run_finished_event`.
- **`run_latest`** — `latest_run_response(account_id)` возвращает последний запуск синхронизации для аккаунта или `RunNotFound`.
- **`scheduling`** — `due_accounts(now, limit)` выбирает аккаунты, у которых синхронизация включена (`sync_enabled = true`), нет активных запусков и `next_run_at` ≤ `now`. Учитывает дефолтные параметры `DEFAULT_MAIL_SYNC_BATCH_SIZE` и `DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS`.
- **`settings`** — `settings_for_account(account_id)` возвращает (или инициализирует дефолтными) настройки синхронизации для аккаунта. `update_settings(account_id, update)` обновляет `sync_enabled`, `batch_size`, `poll_interval_seconds`, предварительно валидируя значения.
- **`statuses`** — `sync_statuses()` возвращает агрегированный статус синхронизации по всем аккаунтам (последний запуск, прогресс, ошибки).

### Валидация (`validation`)

Общие функции:

- `require_unlocked_vault(vault)` — проверяет, что `HostVault` разблокирован.
- `validate_account_id(account_id)` — требует непустой `account_id`.
- `validate_settings(batch_size, poll_interval_seconds)` — проверяет, что `batch_size` в диапазоне 1..500, а `poll_interval_seconds` — 60..86400.
- `next_run_at(settings)` — вычисляет время следующего запуска на основе `poll_interval_seconds`, если синхронизация включена.
- `mail_sync_run_id(account_id)` — генерирует уникальный идентификатор запуска (`mail-sync-run:v1:{account_id}:{timestamp_micros}`).

## `person_derived_evidence` — производные улики о персонах

Обрабатывает события из домена персон и материализует из них наблюдения, отношения и обязательства.

Константа потребителя: `PERSON_DERIVED_EVIDENCE_CONSUMER = "person_derived_evidence"`.

Функция `project_person_derived_evidence_event` диспетчеризует события по типу:

- `PERSON_ROLE_ASSIGNED_EVENT_TYPE` → `materialize_role_assigned`: создаёт observation `PERSON_ROLE` и relationship типа `has_role` между персоной и знанием (`role_knowledge_id`), с review state `UserConfirmed`.
- `PERSON_ROLE_REMOVED_EVENT_TYPE` → `materialize_role_removed`: переводит соответствующий relationship в `UserRejected`.
- `PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE` → `materialize_trust_score`: создаёт observation `PERSON_TRUST_SIGNAL`, вычисляет совместимость персон через `TrustEngine` и создаёт relationship между владельцем (self persona) и целевой персоной. Также вызывает `ensure_relationship_review_item`.
- `PERSON_PROMISE_CREATED_EVENT_TYPE` → `materialize_promise`: создаёт observation `PERSON_PROMISE` и obligation с entity kind `Persona`.

Вспомогательная функция `owner_persona_id` находит self-персону, отличную от указанной.

## `project_link_review_effects` — эффекты ревью связей проектов

Реагирует на событие `PROJECT_LINK_REVIEW_EVENT_TYPE` (`"project.link_review_state_changed"`) и выполняет:

1. Парсит `ProjectLinkReviewEffect` из payload события: `project_id`, `target_kind` (Message / Document), `target_id`, `review_state`.
2. Создаёт observation `PROJECT_LINK_REVIEW`.
3. Материализует relationship: для Message — `project_has_message`, для Document — `project_has_document`, с confidence, зависящей от review state (`Suggested` 0.65, `UserConfirmed` 1.0, `UserRejected` 0.0).
4. Синхронизирует состояние review item через `sync_relationship_review_item`.
5. Если `review_state == UserConfirmed`, дополнительно создаёт decision с impacted entities.

Степени уверенности и тексты улик жёстко закодированы в методе `ProjectLinkReviewEffect::evidence_text()`.

## `realtime_conversation_memory_pipeline` — планирование пайплайна памяти звонков

Функция `plan_memory_pipeline(manifest: &CallBundleManifest)` строит `RealtimeConversationMemoryPipelinePlan`, включающий:

- План для `CallIntelligenceEngine` (извлечение интеллекта из звонка).
- Ожидаемые follow-up события: `realtime_conversation.transcript.requested`, `realtime_conversation.knowledge.extracted`, `realtime_conversation.radar_signals.detected`.

Пайплайн запускается на стадии `"queued_after_local_recording"`.

## `realtime_conversation_radar_projection` — кандидаты радарных сигналов

`call_bundle_radar_candidates` генерирует список `RealtimeConversationRadarSignalCandidate` на основе `CallBundleManifest` и `RealtimeConversationRadarProjectionContext`.

Типы сигналов и условия:

- **`unmatched_meeting_link`** (confidence 0.72): если `join_url` задан, но отсутствуют `calendar_event_id`, `project_id`, `organization_id`.
- **`live_stream_reference`** (confidence 0.78): если в контексте передан `live_stream_watch_url`.
- **`unknown_cohosts`** (confidence 0.68): если в контексте передан непустой список `unknown_cohost_emails`.
- **`local_recording_artifact`** (confidence 0.88): всегда, включает пути к артефактам записи.

Каждый кандидат имеет `promotion_policy`, определяющий требования до продвижения.

## `realtime_conversation_transcript_execution` — выполнение расшифровки

Реагирует на событие `REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED` (только для провайдера `yandex_telemost`).

1. Извлекает `TranscriptExecutionPayload` из события: абсолютные пути `bundle_root`, `manifest_path`, `audio_path`.
2. Читает конфигурацию внешнего транскрайбера из переменных окружения:
   - `MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER` — путь к исполняемому файлу (обязательно).
   - `MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_ARGS_JSON` — JSON-массив строк с аргументами.
   - `MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_TIMEOUT_SECONDS` — таймаут (по умолчанию 900 с).
3. Запускает команду в блокирующем потоке (`spawn_blocking`) с таймаутом, передавая параметры через переменные окружения (`MAKOSH_TRANSCRIPT_*`).
4. Ожидает stdout в формате `LocalTranscriptCommandOutput` (JSON с полями `transcript_text`, `segments`, `language_code`, `stt_provider`, `summary`, `confidence`, `metadata`).
5. Вызывает `complete_realtime_conversation_transcript_bridge` для сохранения результата и публикации следующих событий.

Ошибки включают `CommandTimeout` и `CommandFailed` (ненулевой код возврата).

## `realtime_conversation_transcript_projection` — проекция расшифровки

Реагирует на событие `REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED`.

1. Читает файлы: `transcript_markdown_path`, `transcript_json_path`, `manifest_path`.
2. В транзакции:
   - Создаёт observation `MEETING_TRANSCRIPT` с полным JSON транскрипта.
   - Импортирует документ с расшифровкой (Markdown) через `DocumentImportPort`, связывая с observation.
   - Если в манифесте звонка есть `calendar_event_id`, находит или создаёт запись (`EventRecording`) для аудиофайла и прикрепляет транскрипт (`EventTranscriptPort::attach_transcript_in_transaction`).

Идентификатор документа формируется как `realtime-conversation-transcript:{bundle_id}`.

## `review_inbox` — наполнение инбокса ревью

Предоставляет функции для синхронизации элементов ревью с источниками:

- `refresh_message_task_candidates_into_review` — обновляет кандидатов задач для сообщений и синхронизирует их в обзор через `sync_task_candidates_to_review_for_observations`.
- `refresh_message_decisions_into_review` — аналогично для решений.
- `refresh_message_knowledge_candidates_into_review` — на основе `ProjectedMessage` и его `summary_contract` создаёт элементы ревью `KnowledgeCandidate` с evidence.
- `refresh_message_people_candidates_into_review` — создаёт элементы `NewPerson` и `NewOrganization` из кандидатов контракта сообщения.

Также обрабатывает событие идентификации персоны (`person_identity_candidate_detected_event_type` и `person_identity.review_state_changed`), синхронизируя их через `review_mirror`.

## `review_mirror` — зеркалирование состояний ревью

Содержит утилиты для отражения состояний ревью доменных сущностей в инбоксе ревью:

- `sync_decision_review_state_in_transaction` / `sync_decision_review_state_with_observation` — находит первое доказательство с observation_id, создаёт (или получает) review item решения, и переводит его статус: `Suggested` → `New`, `Rejected` → `Dismissed`, `Confirmed` → `Promote`.
- `sync_obligation_review_state_in_transaction` / `sync_obligation_review_state_with_observation` — аналогично для обязательств.
- `sync_relationship_review_state_in_transaction` / `sync_relationship_review_state_with_observation` — аналогично для отношений, с дополнительным состоянием `SystemAccepted` → `Approved`.
- `sync_identity_candidate_to_review` — создаёт review item для кандидата идентификации персоны и синхронизирует его статус.

## `review_promotion` — продвижение элементов ревью

`ReviewPromotionService` реализует продвижение элемента ревью в целевую доменную сущность.

Метод `promote` (и `promote_with_observation`) загружает `ReviewItem`, его evidence и в зависимости от `item_kind` выполняет:

- `NewPerson` — создаёт персону (`PersonProjectionPort`), возвращает идентификатор `"persons"` / `"persona"`.
- `NewOrganization` — создаёт организацию (`OrganizationCommandPort`).
- `IdentityCandidate` — вызывает `PersonIdentityPort::set_review_state` с `UserConfirmed`, фиксирует observation перехода.
- `ProjectLinkCandidate` — вызывает `ProjectLinkReviewPort::set_review_state`.
- `PotentialTask` — создаёт задачу (`TaskCommandPort`) с параметрами из элемента ревью.
- `ContradictionCandidate` — возвращает ошибку (такие элементы не продвигаются).

Для каждого типа создаётся observation перехода (`review_transition_observation`), который линкуется с результирующей доменной сущностью.

## `task_creation` — создание задач из workflows

`create_task_from_workflow_input` — утилита для создания задачи в рамках транзакции:

- Принимает `WorkflowTaskCreateInput` (заголовок, описание, источник, provenance, срок и др.).
- Создаёт задачу со статусом `"new"`, конфиденциальностью `"private_local"`.
- Материализует связь задачи с наблюдением через `materialize_task_observation_link_in_transaction`.

Используется для создания задач в результате проекций (например, `workflow_action_person_projection`).
```

## Покрытие источников

- `backend/src/workflows/mod.rs` — список всех модулей рабочих процессов.
- `backend/src/workflows/mail_background_sync/store.rs` — структура `MailSyncStore`, алиас `MailSyncStatePort`, конструктор `new`.
- `backend/src/workflows/mail_background_sync/store/account.rs` — метод `require_account`.
- `backend/src/workflows/mail_background_sync/store/orphaned.rs` — метод `mark_orphaned_active_runs_failed`.
- `backend/src/workflows/mail_background_sync/store/run_finish.rs` — метод `finish_run`.
- `backend/src/workflows/mail_background_sync/store/run_latest.rs` — метод `latest_run_response`.
- `backend/src/workflows/mail_background_sync/store/run_progress.rs` — методы `update_progress`, `mark_recoverable_full_resync`.
- `backend/src/workflows/mail_background_sync/store/run_start.rs` — метод `start_run`.
- `backend/src/workflows/mail_background_sync/store/scheduling.rs` — метод `due_accounts`, использование дефолтных констант.
- `backend/src/workflows/mail_background_sync/store/settings.rs` — методы `settings_for_account`, `update_settings`.
- `backend/src/workflows/mail_background_sync/store/statuses.rs` — метод `sync_statuses`.
- `backend/src/workflows/mail_background_sync/validation.rs` — функции `require_unlocked_vault`, `validate_account_id`, `validate_settings`, `next_run_at`, `mail_sync_run_id`; диапазоны валидации.
- `backend/src/workflows/person_derived_evidence.rs` — константа потребителя, функция `project_person_derived_evidence_event`, материализация ролей, trust score, промисов; `owner_persona_id`.
- `backend/src/workflows/project_link_review_effects.rs` — тип события `PROJECT_LINK_REVIEW_EVENT_TYPE`, разбор `ProjectLinkReviewEffect`, материализация связей и решений, маппинг степеней уверенности.
- `backend/src/workflows/realtime_conversation_memory_pipeline.rs` — структура `RealtimeConversationMemoryPipelinePlan`, функция `plan_memory_pipeline`.
- `backend/src/workflows/realtime_conversation_radar_projection.rs` — структуры `RealtimeConversationRadarSignalCandidate` и `RealtimeConversationRadarProjectionContext`, функция `call_bundle_radar_candidates`, условия для сигналов.
- `backend/src/workflows/realtime_conversation_transcript_execution.rs` — потребитель `execute_realtime_conversation_transcript_request_event`, конфигурация транскрайбера из env, запуск внешней команды, структура вывода.
- `backend/src/workflows/realtime_conversation_transcript_projection.rs` — потребитель `project_realtime_conversation_transcript_event`, формирование observation, импорт документа, привязка к календарю; `transcript_document_id`.
- `backend/src/workflows/review_inbox.rs` — функции `refresh_message_task_candidates_into_review`, `refresh_message_decisions_into_review`, `refresh_message_knowledge_candidates_into_review`, `refresh_message_people_candidates_into_review`; обработка событий идентификации персон.
- `backend/src/workflows/review_mirror.rs` — функции `sync_decision_review_state_*`, `sync_obligation_review_state_*`, `sync_relationship_review_state_*`, `sync_identity_candidate_to_review`.
- `backend/src/workflows/review_promotion/mod.rs` — структура `ReviewPromotionService`, метод `promote`, диспетчеризация по `item_kind`.
- `backend/src/workflows/task_creation.rs` — структура `WorkflowTaskCreateInput`, функция `create_task_from_workflow_input`.

## Исходные файлы

- [`backend/src/workflows/mail_background_sync/store.rs`](../../../../backend/src/workflows/mail_background_sync/store.rs)
- [`backend/src/workflows/mail_background_sync/store/account.rs`](../../../../backend/src/workflows/mail_background_sync/store/account.rs)
- [`backend/src/workflows/mail_background_sync/store/orphaned.rs`](../../../../backend/src/workflows/mail_background_sync/store/orphaned.rs)
- [`backend/src/workflows/mail_background_sync/store/run_finish.rs`](../../../../backend/src/workflows/mail_background_sync/store/run_finish.rs)
- [`backend/src/workflows/mail_background_sync/store/run_latest.rs`](../../../../backend/src/workflows/mail_background_sync/store/run_latest.rs)
- [`backend/src/workflows/mail_background_sync/store/run_progress.rs`](../../../../backend/src/workflows/mail_background_sync/store/run_progress.rs)
- [`backend/src/workflows/mail_background_sync/store/run_start.rs`](../../../../backend/src/workflows/mail_background_sync/store/run_start.rs)
- [`backend/src/workflows/mail_background_sync/store/scheduling.rs`](../../../../backend/src/workflows/mail_background_sync/store/scheduling.rs)
- [`backend/src/workflows/mail_background_sync/store/settings.rs`](../../../../backend/src/workflows/mail_background_sync/store/settings.rs)
- [`backend/src/workflows/mail_background_sync/store/statuses.rs`](../../../../backend/src/workflows/mail_background_sync/store/statuses.rs)
- [`backend/src/workflows/mail_background_sync/validation.rs`](../../../../backend/src/workflows/mail_background_sync/validation.rs)
- [`backend/src/workflows/mod.rs`](../../../../backend/src/workflows/mod.rs)
- [`backend/src/workflows/person_derived_evidence.rs`](../../../../backend/src/workflows/person_derived_evidence.rs)
- [`backend/src/workflows/project_link_review_effects.rs`](../../../../backend/src/workflows/project_link_review_effects.rs)
- [`backend/src/workflows/realtime_conversation_memory_pipeline.rs`](../../../../backend/src/workflows/realtime_conversation_memory_pipeline.rs)
- [`backend/src/workflows/realtime_conversation_radar_projection.rs`](../../../../backend/src/workflows/realtime_conversation_radar_projection.rs)
- [`backend/src/workflows/realtime_conversation_transcript_execution.rs`](../../../../backend/src/workflows/realtime_conversation_transcript_execution.rs)
- [`backend/src/workflows/realtime_conversation_transcript_projection.rs`](../../../../backend/src/workflows/realtime_conversation_transcript_projection.rs)
- [`backend/src/workflows/review_inbox.rs`](../../../../backend/src/workflows/review_inbox.rs)
- [`backend/src/workflows/review_mirror.rs`](../../../../backend/src/workflows/review_mirror.rs)
- [`backend/src/workflows/review_promotion/mod.rs`](../../../../backend/src/workflows/review_promotion/mod.rs)
- [`backend/src/workflows/task_creation.rs`](../../../../backend/src/workflows/task_creation.rs)

## Кандидаты на drift

Из предоставленного контекста расхождения между кодом, документацией и ADR не видны. Текущее содержимое вики-страницы `components/backend.md` не было предоставлено, поэтому сравнение невозможно.
