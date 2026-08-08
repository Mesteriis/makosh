## Summary / Резюме

Предлагается создать страницу `components/backend.md` в русской Obsidian‑wiki на основе встроенных исходников. Страница описывает слой приложений (`application`) и бинарные утилиты (`bin`) backend‑крейта Макошь, фиксируя реальные структуры, функции, конвейеры команд и связи между модулями так, как они представлены в предоставленном контексте.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты: backend

## Слой приложений (`application`)

Модуль `backend::application` собирает сервисы уровня приложения (application
services), которые координируют доменную логику, хранилища и подсистемы
наблюдений. Многие из них реализованы прямым делегированием в модули
`workflows`.

### Переиспользование рабочих процессов (`workflows`)

Ряд файлов слоя `application` содержат только ре‑экспорт из `crate::workflows`:

- `backend/src/application/review_inbox.rs` → `workflows::review_inbox`
- `backend/src/application/review_promotion.rs` → `workflows::review_promotion`
- `backend/src/application/task_creation.rs` → `workflows::task_creation`
- `backend/src/application/workflow_action_person_projection.rs` → `workflows::workflow_action_person_projection`
- `backend/src/application/yandex_telemost_calendar_matching.rs` → `workflows::yandex_telemost_calendar_matching`
- `backend/src/application/zoom_calendar_matching.rs` → `workflows::zoom_calendar_matching`
- `backend/src/application/zoom_participant_identity.rs` → `workflows::zoom_participant_identity`
- `backend/src/application/zoom_signal_detection.rs` → `workflows::zoom_signal_detection`

Поведение этих модулей не раскрывается в данном контексте.

### Сервисы проверки (review)

Файл `backend/src/application/review_transitions.rs` содержит четыре
прикладных сервиса, каждый из которых принимает `PgPool`:

- `DecisionReviewApplicationService`
- `ObligationReviewApplicationService`
- `RelationshipReviewApplicationService`
- `TaskCandidateReviewApplicationService`

Каждый сервис реализует метод `review_manual`, который:

1. Фиксирует наблюдение (`Observation`) типа `"REVIEW_TRANSITION"` с
   происхождением `Manual` через `ObservationStore::capture`.
2. Вызывает соответствующий доменный store (`DecisionStore`,
   `ObligationStore`, `RelationshipStore` или `TaskCandidateReviewService`)
   для установки нового review‑состояния.
3. Синхронизирует review‑состояние через функции модуля `workflows::review_mirror`
   (например, `sync_decision_review_state_with_observation`).

Для `TaskCandidateReviewApplicationService` дополнительно внутри явной
транзакции выполняется `SELECT … FOR UPDATE` из таблицы `task_candidates` и
вызов `sync_task_candidate_review_state_in_transaction`.

Типы ошибок: `DecisionReviewApplicationError`, `ObligationReviewApplicationError`,
`RelationshipReviewApplicationError`, `TaskCandidateReviewApplicationError`.
Все они агрегируют ошибки хранилищ и `workflows::review_mirror::ReviewMirrorError`.

### Переигрывание сигналов (signal hub replay)

Файл `backend/src/application/signal_hub_replay.rs` определяет сервис
`SignalHubReplayService` (композиция `SignalHubStore`,
`SignalHubSignalService`, `EventStore`). Основные операции:

- **request_replay** – создаёт запрос на переигрывание, добавляет событие
  жизненного цикла `signal.replay.requested`.
- **process_next_request** – забирает (claim) первый ожидающий запрос и
  вызывает `process_claimed_request`; при ошибке помечает запрос как
  failed и публикует `signal.replay.failed`, при успехе – `signal.replay.completed`.
- **process_claimed_request** – маршрутизирует выполнение по следующим
  веткам:
  - Если задан `target_projection` → `rebuild_projection`.
  - Если задан `target_consumer` → список событий для потребителя и
    `prepare_consumer_replay`.
  - Если `uses_event_log_replay` → события из event log, каждое
    проигрывается через `signal_service.replay_raw_signal`.
  - Иначе – список приостановленных событий из `signal_store` с
    последующей разблокировкой (`release_paused_event`).

`rebuild_projection` поддерживает проекции:
`communication_messages`, `person_derived_evidence`,
`project_link_review_effects`, `realtime_conversation_transcript_projection`,
`yandex_telemost_calendar_matching`, `zoom_calendar_matching`,
`timeline_event_log`.

Дополнительно имеются:

- `append_replay_lifecycle_event` – запись событий жизненного цикла в
  EventStore.
- `list_event_log_events_for_replay` – фильтрация событий, начинающихся с
  `signal.raw.`.
- `list_matching_signal_events` – построение запроса `EventLogQuery` с
  учётом `source_code`, `from_position`/`to_position`,
  `from_time`/`to_time` и фильтр по паттерну.

Глубинная логика перестроения потребителей и дополнительные детали
рантайма Signal Hub обрезаны в предоставленном контексте.

### Управление рантаймом Telegram

Файл `backend/src/application/telegram_runtime.rs` предоставляет контекст
`TelegramRuntimeUseCaseContext` и набор `pub(crate)`‑асинхронных функций,
образующих use‑case слой для взаимодействия с Telegram‑рантаймом:

| Функция | Назначение |
|---------|------------|
| `runtime_status` | Запрос статуса рантайма аккаунта |
| `start_runtime` | Запуск рантайма |
| `stop_runtime` | Остановка рантайма |
| `restart_runtime` | Перезапуск рантайма |
| `sync_chat_members` | Синхронизация участников чата |
| `sync_chats` | Синхронизация списка чатов |
| `sync_history` | Синхронизация истории сообщений |
| `send_manual_message` | Отправка произвольного сообщения |
| `send_reply_message` | Отправка ответа |
| `send_forward_message` | Пересылка сообщения |
| `refresh_provider_search` | Поиск сообщений провайдера |
| `refresh_forum_topics` | Синхронизация тем форума |
| `download_media` | Загрузка медиафайлов |

Все функции делегируют реальную работу `TelegramRuntimeManager`.
Контекст собирается из stores (`CommunicationProviderAccountStore`,
`CommunicationProviderSecretBindingStore`, `TelegramStore`,
`SecretReferenceStore`) и runtime‑зависимостей (`HostVault`, `AppConfig`,
`EventBus`, `TelegramRuntimeManager`).

### Выполнение команд WhatsApp

Файл `backend/src/application/whatsapp_command_executor.rs` (обрезан)
содержит два конвейера выполнения команд WhatsApp – для фикстур и для
«живого» native‑MD‑драйвера.

**Конвейер фикстур** – `execute_due_fixture_commands`:

1. Импорт канонических команд провайдера (`import_canonical_provider_commands`).
2. Восстановление «зависших» исполняющихся команд
   (`recover_stale_fixture_executing_commands`).
3. Забор ожидающих команд (`claim_due_commands_for_execution`).
4. Для каждой команды публикуются события статуса, запускается выполнение через
   `WhatsappFixtureIngestApplicationService`; при неудаче –
   перепланирование (`reschedule_failed_command`) и событие об ошибке.

**Конвейер live native MD** – `execute_due_live_native_md_commands`:

- Восстановление «зависших» live‑команд.
- Забор команд через `claim_due_native_md_commands_for_execution`.
- Для каждой команды выполняется подготовка загрузки/скачивания медиа
  (функции `prepare_live_native_md_media_upload`,
  `prepare_live_native_md_media_download`), затем выполнение через
  `runtime.execute_live_provider_command`.
- В зависимости от результата либо сохраняется подтверждение выполнения
  (`record_live_provider_command_submitted`), либо фиксируется ошибка с
  публикацией событий.

Детали обработки ответов провайдера, пагинации и вспомогательных структур
не полностью доступны из‑за обрезания исходника.

### Сверка наблюдений провайдера WhatsApp

Файл `backend/src/application/whatsapp_provider_observation_reconciliation.rs`
(обрезан) содержит потребителя событий с идентификатором
`"whatsapp_provider_observation_reconciliation"` и функцию
`reconcile_whatsapp_provider_observation_event`.

Логика:

1. Фильтрация по типам событий `signal.accepted.whatsapp.*` (message,
   reaction, receipt, media, status, dialog, participant, message_update,
   message_delete).
2. Извлечение `raw_record_id` из `event.subject`, загрузка сырой записи
   из `CommunicationIngestionStore`.
3. Проверка, что аккаунт имеет `provider_kind` WhatsApp.
4. Конвертация сырой записи в доменную модель (с помощью функций
   `raw_record_to_whatsapp_message`, `raw_record_to_whatsapp_reaction`,
   …) и вызов соответствующего метода `runtime.reconcile_fixture_*_commands`
   (например, `reconcile_fixture_message_commands`).
5. Для каждой полученной команды публикуются события
   `COMMAND_STATUS_CHANGED` и `COMMAND_RECONCILED`.

Полное определение всех моделей (reaction, receipt и т.д.) находится за
пределами переданного фрагмента.

### Проекция событий рантайма WhatsApp

Файл `backend/src/application/whatsapp_runtime_event_projection.rs`
(обрезан) содержит потребителя `"whatsapp_runtime_event_projection"` и
функцию `project_whatsapp_runtime_event`, обрабатывающую события
`signal.accepted.whatsapp.runtime_event`.

Логика проекции:

1. Загрузка сырой записи по `raw_record_id`.
2. Построение `RuntimeEventReconcileDecision` из `lifecycle_state` или
   `runtime_status` сырой записи.
3. При необходимости – очистка восстанавливаемой сессии WhatsApp
   (`clear_whatsapp_restorable_session`).
4. Обновление `lifecycle_state` аккаунта в конфигурации провайдера.
5. Обновление `link_state` сессии в таблице `whatsapp_web_sessions`.
6. Получение актуального статуса рантайма через `runtime.runtime_status`.
7. Если статус `"removed"` – удаление signal‑подключения; иначе –
   синхронизация подключения через
   `sync_whatsapp_runtime_signal_connection_for_pool`.

Функция `merged_whatsapp_runtime_connection_settings` обрезана в
предоставленном контексте.

### Приём сигналов рантайма WhatsApp

Файл `backend/src/application/whatsapp_runtime_signal_ingest.rs` (обрезан)
содержит сервис `WhatsappRuntimeSignalIngestService` и его реализацию
трейта `WhatsAppRuntimeEventSink`.

Основной метод `ingest_sanitized_runtime_event`:

- Проверяет контракт `assert_event_spine_contract` на DTO.
- Создаёт `NewRawCommunicationRecord` через
  `CommunicationIngestionPort::record_raw_source`.
- Вызывает `dispatch_whatsapp_raw_signal` для порождения принятого события.
- Если принятое событие не соответствует ожидаемому типу, возвращает ошибку
  `AcceptedEventKindMismatch`.

DTO `WhatsAppSanitizedRuntimeEventDto` содержит метаданные, из которых
извлекаются `runtime_status`, `lifecycle_state`, `severity` через функцию
`runtime_event_state` и опциональные переопределения
`sanitized_runtime_state_override`. Ключи, похожие на секреты, редактуются
функцией `redact_secret_like_metadata` (список ключей включает:
`access_token`, `session_key`, `cookie`, `token`, `secret`, `password`,
`url` и другие).

В конце файла присутствует тест
`sanitized_native_runtime_event_enters_raw_evidence_and_signal_hub_idempotently`
(обрезан), подтверждающий сценарий идемпотентной записи.

### Прочие делегирующие модули

Оставшиеся файлы `backend/src/application/*.rs` (см. секцию
«Переиспользование рабочих процессов») не содержат собственной логики в
переданном контексте.

---

## Бинарные утилиты (`bin/`)

Все бинарные крейты используют `makosh_hub_backend::app::init_tracing()` и
конфигурируются через переменные окружения с префиксом `MAKOSH_*`.

### `makosh_document_process`

- Источник: `backend/src/bin/makosh_document_process.rs`
- Назначение: запуск очереди обработки документов.
- Параметры: `DATABASE_URL` (обязательно), аргумент командной строки `limit`
  (по умолчанию 25).
- Использует `DocumentProcessingStore::run_queued_jobs(limit)`, выводит
  JSON‑отчёт.

### `makosh_email_fixture_dev`

- Источник: `backend/src/bin/makosh_email_fixture_dev.rs`
- Режимы: `import` и `project` (переменная `MAKOSH_EMAIL_FIXTURE_MODE`,
  по умолчанию `project`).
- Провайдер: `MAKOSH_EMAIL_FIXTURE_PROVIDER` (`gmail`, `icloud`, `imap`;
  по умолчанию `icloud`).
- Другие переменные: путь к фикстуре, ID аккаунта, отображаемое имя,
  внешний email, batch ID.
- В режиме `import` вызывает `import_fixture_email_messages_for_dev`, в
  режиме `project` – `project_fixture_email_messages`.

### `makosh_email_fixture_export`

- Источник: `backend/src/bin/makosh_email_fixture_export.rs`
- Назначение: экспорт сырых email‑сообщений из iCloud (IMAP) в
  редактуированный JSON‑фикстурный файл.
- Переменные окружения: `MAKOSH_IMAP_FIXTURE_{USERNAME,PASSWORD,HOST,PORT,TLS,MAILBOX,MAX_MESSAGES,LAST_SEEN_UID,OUTPUT}`
  с резервными значениями (`ICLOUD_LOGIN`, `ICLOUD_2FA`).
- Использует `ImapNetworkClient`, `EmailFixtureExportOptions` и
  `export_fixture_messages_from_sync_batch`. Результат сохраняется в
  файл, в stdout выводится JSON‑отчёт.

### `makosh_email_sync_dev`

- Источник: `backend/src/bin/makosh_email_sync_dev.rs` и подмодули в
  `backend/src/bin/makosh_email_sync_dev/` (`account.rs`,
  `checkpoint.rs`, `config.rs`, `env.rs`, `errors.rs`, `provider.rs` и др.).
- Назначение: полный цикл IMAP‑синхронизации почты для разработки.
- Конфигурация из переменных `MAKOSH_EMAIL_SYNC_*` (провайдер, учётные
  данные, хост, порт, mailbox, лимит сообщений, blob‑root, import batch).
- Поддерживаемые провайдеры: `icloud` и `imap`; Gmail явно не
  поддерживается в данной утилите.
- Основной запуск через `run_dev_email_sync(config)`, который внутри
  создаёт/обновляет провайдер‑аккаунт, читает последний `last_seen_uid` из
  чекпойнта, выполняет сетевую выборку и передаёт данные в конвейер
  синхронизации.

### `makosh_email_reproject_dev`

- Источник: `backend/src/bin/makosh_email_reproject_dev.rs`
- Назначение: повторная проекция email‑сообщений из сырых записей,
  например, для исправления проблем с кодировкой.
- Конфигурация: `MAKOSH_EMAIL_REPROJECT_ACCOUNT_ID` (опционально),
  `MAKOSH_EMAIL_REPROJECT_ONLY_CORRUPT` (по умолчанию `true`),
  `MAKOSH_EMAIL_REPROJECT_BLOB_ROOT` (по умолчанию
  `docker/data/mail`).
- Запрашивает записи `communication_raw_records` со связью в
  `communication_messages`; при `only_corrupt=true` фильтрует сообщения с
  символами замены (`�`) в subject, sender или body.
- Для каждой записи вызывает `parse_raw_email_message_from_blob`, затем
  `project_parsed_raw_email_message`. Выводит JSON‑отчёт и завершается
  ошибкой, если были неудачные записи.
```

## Source coverage / Покрытие источников

- `backend/src/application/review_inbox.rs` – факт делегирования в `workflows::review_inbox`.
- `backend/src/application/review_promotion.rs` – факт делегирования в `workflows::review_promotion`.
- `backend/src/application/review_transitions.rs` – четыре application service, их метод `review_manual`, использование `ObservationStore`, доменных store, синхронизация через `review_mirror`, структура ошибок, транзакционная обработка `TaskCandidateReviewApplicationService`.
- `backend/src/application/signal_hub_replay.rs` – `SignalHubReplayService`, методы `request_replay`, `process_next_request`, `process_claimed_request`, `rebuild_projection`; список поддерживаемых проекций; функции `append_replay_lifecycle_event`, `list_event_log_events_for_replay`, `list_matching_signal_events`; маршрутизация replay по `target_projection`, `target_consumer`, event log и paused events.
- `backend/src/application/task_creation.rs` – делегирование в `workflows::task_creation`.
- `backend/src/application/telegram_runtime.rs` – `TelegramRuntimeUseCaseContext`, все pub‑функции use‑case слоя (runtime_status, start/stop/restart_runtime, sync_chat_members, sync_chats, sync_history, send_manual_message, send_reply_message, send_forward_message, refresh_provider_search, refresh_forum_topics, download_media), используемые stores и runtime‑зависимости.
- `backend/src/application/whatsapp_command_executor.rs` – две функции конвейеров: `execute_due_fixture_commands` и `execute_due_live_native_md_commands`; этапы импорта, восстановления, забора, выполнения, перепланирования; подготовка медиа‑загрузки/скачивания для native MD; публикация событий.
- `backend/src/application/whatsapp_provider_observation_reconciliation.rs` – потребитель `whatsapp_provider_observation_reconciliation`, функция `reconcile_whatsapp_provider_observation_event`, поддерживаемые типы событий, преобразования сырых записей в доменные модели, публикация командных событий.
- `backend/src/application/whatsapp_runtime_event_projection.rs` – потребитель `whatsapp_runtime_event_projection`, `project_whatsapp_runtime_event`, `RuntimeEventReconcileDecision`, обновление lifecycle_state, link_state, очистка сессии, синхронизация signal‑подключения.
- `backend/src/application/whatsapp_runtime_signal_ingest.rs` – `WhatsappRuntimeSignalIngestService`, `ingest_sanitized_runtime_event`, реализация `WhatsAppRuntimeEventSink`, определение `runtime_event_state`, редакция секретов, присутствие теста.
- `backend/src/application/workflow_action_person_projection.rs` – делегирование в workflows.
- `backend/src/application/yandex_telemost_calendar_matching.rs` – делегирование в workflows.
- `backend/src/application/zoom_calendar_matching.rs` – делегирование в workflows.
- `backend/src/application/zoom_participant_identity.rs` – делегирование в workflows.
- `backend/src/application/zoom_signal_detection.rs` – делегирование в workflows.
- `backend/src/bin/makosh_document_process.rs` – утилита, переменные окружения, `DocumentProcessingStore`, вывод отчёта.
- `backend/src/bin/makosh_email_fixture_dev.rs` – режимы `import`/`project`, конфигурация, функции `import_fixture_email_messages_for_dev` / `project_fixture_email_messages`.
- `backend/src/bin/makosh_email_fixture_export.rs` – IMAP‑экспорт фикстур, переменные окружения, `ImapNetworkClient`, `EmailFixtureExportOptions`, редакция.
- `backend/src/bin/makosh_email_reproject_dev.rs` – репроекция email, фильтрация по‑аккаунту и corrupt‑сообщениям, функции `parse_raw_email_message_from_blob`, `project_parsed_raw_email_message`, отчёт.
- `backend/src/bin/makosh_email_sync_dev.rs` – структура утилиты, подмодули, `DevEmailSyncConfig`, `run_dev_email_sync`.
- `backend/src/bin/makosh_email_sync_dev/account.rs` – `upsert_dev_provider_account`.
- `backend/src/bin/makosh_email_sync_dev/checkpoint.rs` – `last_seen_uid`.
- `backend/src/bin/makosh_email_sync_dev/config.rs` – `DevEmailSyncConfig`, переменные окружения, провайдеры.
- `backend/src/bin/makosh_email_sync_dev/env.rs` – вспомогательные функции для переменных окружения.
- `backend/src/bin/makosh_email_sync_dev/errors.rs` – `DevEmailSyncError` и его варианты.

## Drift candidates / Кандидаты на drift

Из предоставленного контекста не видны расхождения между кодом и документацией или ADR, поскольку существующая wiki‑страница `components/backend.md` не была включена в данный чанк. Наблюдаемое содержимое исходников не содержит явных признаков drift.
