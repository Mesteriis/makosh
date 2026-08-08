### Summary / Резюме

Обновление страницы `components/backend.md` в русской Obsidian‑wiki. На основе предоставленных исходных файлов добавляется описание структуры прикладного слоя `application`, фоновых задач (`bootstrap`), механизма согласования хранилища (`vault_reconciliation`), ключевых прикладных сервисов (календарь, фикстуры, отправка почты, контакты организаций, AI‑диспетчеризация, контракты и сервисы провайдеров) и обработки ошибок. Все утверждения опираются исключительно на встроенный код.

### Proposed pages / Предлагаемые страницы

**Путь:** `components/backend.md`

```markdown
# Бэкенд (backend)

Компонент `backend` — прикладной слой системы makosh‑hub, реализованный на Rust. Он объединяет бизнес‑логику, планирование фоновых задач, интеграцию с провайдерами (Telegram, WhatsApp, Zoom, Яндекс.Телемост) и управление данными через доменные сервисы.

## Структура модулей приложения

Модуль `application` содержит следующие подмодули (см. `mod.rs`):

- `bootstrap` — запуск фоновых сервисов.
- `ai_signal_dispatch` — диспетчеризация сигналов AI‑помощника.
- `calendar_meeting_outcomes` — обработка результатов встреч календаря.
- `communication_fixture_ingest` — приём фикстурных сообщений Telegram/WhatsApp.
- `communication_provider_writes` — канонические операции чтения/записи данных коммуникаций.
- `communication_send` — отправка электронной почты.
- `consistency_review`, `email_intelligence`, `mail_background_sync`, `person_derived_evidence`, `project_link_review_effects` — делегирование в `crate::workflows`.
- `organization_contact_links` — ручная привязка контактов к организациям.
- `project_link_review_mirror` — зеркалирование состояний ревью связей проектов.
- `provider_runtime_contracts` — контракты (типы, трейты) для провайдеров связи.
- `provider_runtime_services` — прикладные сервисы управления провайдерами.
- `realtime_conversation_transcript_execution` — выполнение действий по транскриптам разговоров.
- `realtime_conversation_transcript_projection` — проекция транскриптов.
- `review_inbox`, `review_promotion`, `review_transitions` — ревью‑инбокс и продвижение.
- `signal_hub_replay` — воспроизведение сигналов.
- `task_creation` — создание задач.
- `telegram_runtime` — управление Telegram‑рантаймом.
- `whatsapp_command_executor` — выполнение команд WhatsApp.
- `whatsapp_provider_observation_reconciliation` — согласование наблюдений WhatsApp.
- `whatsapp_runtime_event_projection` — проекция событий WhatsApp.
- `whatsapp_runtime_signal_ingest` — приём сигналов WhatsApp.
- `workflow_action_person_projection` — проекция действий рабочих процессов на персоны.
- `yandex_telemost_calendar_matching` — сопоставление встреч Яндекс.Телемост.
- `zoom_calendar_matching` — сопоставление встреч Zoom.
- `zoom_participant_identity` — идентификация участников Zoom.
- `zoom_signal_detection` — детектирование сигналов Zoom.

## Фоновые задачи (bootstrap)

Функция `start_background_services` (файл `bootstrap.rs`) запускает долгоживущие асинхронные задачи, каждая из которых выполняется в цикле с заданным интервалом. Для предотвращения повторного запуска одной задачи для одной базы данных используются глобальные статические `LazyLock<Mutex<HashSet<String>>>` (например, `MAIL_BACKGROUND_SYNC_DATABASES`) и вызовы `register_*_scheduler`.

Контекст `ApplicationBootstrapContext` содержит:

- `pool: Option<PgPool>` — подключение к PostgreSQL;
- `database_url: Option<String>` — URL базы;
- `nats_server_url: Option<String>` — URL NATS (если используется);
- `vault: HostVault` — хранилище секретов;
- `telegram_runtime: TelegramRuntimeManager` — менеджер Telegram‑рантайма;
- `event_bus: EventBus` — шина событий;
- флаги `zoom_token_maintenance_scheduler_enabled`, `zoom_recording_sync_scheduler_enabled`, `zoom_retention_cleanup_scheduler_enabled`.

Список запускаемых фоновых сервисов (каждый со своей константой идентификатора):

| Сервис | Интервал | Назначение |
|--------|----------|------------|
| `mail_background_sync` | 30 с | Фоновая синхронизация почты |
| `mail_outbox_delivery` | 10 с | Отправка писем из outbox |
| `telegram_command_executor` | (не показан) | Выполнение команд Telegram |
| `whatsapp_command_executor` | – | Выполнение команд WhatsApp |
| `whatsapp_runtime_restore_reconciliation` | – | Восстановление/согласование WhatsApp‑рантайма |
| `zoom_token_maintenance` | 60 с | Обновление токенов Zoom (refresh за 300 с до истечения) |
| `zoom_recording_sync` | 300 с | Синхронизация записей Zoom (lookback 7 дней) |
| `zoom_retention_cleanup` | 3600 с | Очистка по retention (до 100 записей на аккаунт) |
| `yandex_telemost_retention_cleanup` | 3600 с | Очистка Яндекс.Телемост (до 100 записей на аккаунт) |
| `whatsapp_runtime_event_projection` | – | Проекция событий WhatsApp‑рантайма |
| `whatsapp_provider_observation_reconciliation` | – | Согласование провайдерских наблюдений WhatsApp |
| `communication_provider_observation_projection` | – | Проекция наблюдений провайдеров связи |
| `person_derived_evidence_projection` | – | Проекция производных свидетельств о персонах |
| `zoom_signal_detection_projection` | – | Проекция обнаружения сигналов Zoom |
| `zoom_calendar_matching_projection` | – | Проекция сопоставления календарных событий Zoom |
| `zoom_participant_identity_projection` | – | Проекция идентификации участников Zoom |
| `yandex_telemost_calendar_matching_projection` | – | Проекция сопоставления Яндекс.Телемост |
| `realtime_conversation_transcript_execution` | – | Выполнение по транскриптам |
| `person_identity_review_inbox_projection` | – | Проекция инбокса идентификации персон |
| `project_link_review_effects_projection` | – | Проекция эффектов ревью связей проектов |
| `realtime_conversation_transcript_projection` | – | Проекция транскриптов |
| `signal_hub_raw_signal_dispatcher` | – | Диспетчер сырых сигналов |
| `event_outbox_dispatcher` | – | Диспетчер исходящих событий |
| `signal_replay_dispatcher` | – | Диспетчер воспроизведения сигналов |

*Примечание:* Интервалы, помеченные «–» или «(не показан)», не представлены в данном фрагменте, но присутствуют в полном `bootstrap.rs`.

Каждая задача проверяет доступность подключения к БД, а также (для некоторых) состояние хранилища: если vault не разблокирован (`Unlocked`), выполнение пропускается. Часть задач использует функцию `runtime_allows_processing` для динамического управления.

## Согласование хранилища (vault reconciliation)

Модуль `vault_reconciliation` обеспечивает синхронизацию манифеста учётных записей провайдеров, хранящегося в host‑хранилище (`HostVault`), с базой данных приложения.

### Ключевые структуры

- `HostVaultReconciliationError` (`errors.rs`) — объединяет ошибки `HostVaultError`, `SecretReferenceError`, `CommunicationIngestionError`, `CalendarError` и `sqlx::Error`.
- `HostVaultReconciliationSummary` (`summary.rs`) — счётчики восстановленных аккаунтов:
    - `restored_accounts: usize` — количество восстановленных учётных записей провайдеров;
    - `restored_calendar_accounts: usize` — количество связанных календарных аккаунтов.
- `RecoverableProviderSecret` (`provider_recovery.rs`) — представление секрета провайдера, извлечённое из записи манифеста. Содержит `account_id`, `provider_kind`, `display_name`, `external_account_id`, `secret_ref`, `secret_kind`, `store_kind`, `secret_purpose`, `label`, `secret_metadata` и `provider_account_config`.

### Процесс согласования

Основная функция `reconcile_host_vault_manifest` (`service.rs`):

1. Получает манифест учётных записей из хранилища (`vault.account_secret_manifest()`).
2. Для каждой записи манифеста:
    - Обогащает запись метаданными из PostgreSQL (функция `enrich_manifest_entry_from_postgres`). Это применяется только к записям типа `provider_credential`. Выполняется запрос к таблицам `communication_provider_accounts` и `communication_provider_account_secret_refs`, формируется метаданные (провайдер, display_name, external_account_id, config), которые записываются в запись манифеста через `vault.upsert_account_secret_manifest_entry(...)`.
    - Преобразует запись в `RecoverableProviderSecret`, если это возможно. Условия: entry_kind = `provider_credential`, provider_kind — Gmail, iCloud или Imap, secret_kind и store_kind совместимы с purpose.
    - Вызывает `restore_secret_reference` — upsert записи в таблицу секретных ссылок.
    - Вызывает `restore_provider_account` — если учётная запись провайдера отсутствует в БД, создаёт её, увеличивая счётчик `restored_accounts`.
    - Вызывает `restore_provider_account_secret_binding` — восстанавливает привязку секрета к аккаунту.
    - Вызывает `restore_linked_calendar_account` — при необходимости восстанавливает календарный аккаунт, увеличивая `restored_calendar_accounts`.
3. Возвращает сводку `HostVaultReconciliationSummary`.

### Запуск согласования

Функция `spawn_host_vault_manifest_reconciliation` (`lifecycle.rs`) проверяет:
- наличие `database_url` в конфигурации;
- доступность статуса хранилища;
- что хранилище находится в состоянии `Unlocked`;
- наличие пула подключений;
- наличие активного Tokio‑рантайма.

При соблюдении всех условий в отдельной задаче выполняется `reconcile_host_vault_manifest`. Успешное восстановление аккаунтов логируется с тегом `info`, ошибки — с тегом `warn`.

### Вспомогательные функции (`metadata.rs`)

- `fallback_provider_account_config(provider_kind, metadata, external_account_id)` — возвращает конфигурацию провайдера по умолчанию, если она отсутствует в метаданных. Для Gmail — OAuth (`{"auth":"oauth","api":"gmail",...}`), для iCloud — IMAP (`"host":"imap.mail.me.com","port":993,"tls":true`), для IMAP — username.
- `fallback_display_name(provider_kind, label, account_id)` — возвращает отображаемое имя: Google Workspace, iCloud, либо account_id.
- `metadata_string(metadata, key)` — извлечение строки из JSON‑метаданных.
- `non_empty(Option<String>)` — фильтрация пустых строк.

## Ключевые прикладные сервисы

### CalendarMeetingOutcomeApplicationService

Сервис `add_manual` (`calendar_meeting_outcomes.rs`) добавляет исход встречи вручную. Операция:
- Захватывает observation (тип `MEETING`, источник `Manual`).
- В транзакции создаёт `MeetingOutcome`, связывает его с observation.
- Если тип исхода — `decision`, создаёт решение (`Decision`) с evidence и impacted entity; синхронизирует состояние ревью.
- Если тип — `promise`, создаёт обязательство (`Obligation`) с привязкой к персоне; синхронизирует ревью.
- Возвращает созданный `MeetingOutcome`.

### CommunicationFixtureIngest

Сервисы `TelegramFixtureIngestApplicationService` и `WhatsappFixtureIngestApplicationService` (`communication_fixture_ingest.rs`) обрабатывают входящие фикстурные сообщения.

**Telegram**:
- `ingest_message` сохраняет сообщение в хранилище, диспетчеризует сигнал через `dispatch_telegram_raw_signal`, выполняет проекцию через `project_accepted_signal_if_runtime_allows`, пересчитывает количество непрочитанных сообщений в чате, запускает обновление ревью‑кандидатов (`refresh_message_decisions_into_review`, `refresh_message_task_candidates_into_review`), публикует событие `MESSAGE_CREATED` в event bus.

**WhatsApp** (методы с постфиксом `_with_reconciliation_source`):
- Поддерживают несколько источников: `provider_observed.fixture_message`, `provider_observed.runtime_bridge_message` (аналогично для реакций).
- Выполняют сохранение, диспетчеризацию, проекцию, а также дополнительно:
    - Проекцию ссылок сообщений (`project_whatsapp_message_refs`).
    - Обновление трейсов идентификации персон (`upsert_whatsapp_person_identity_traces_for_message`).
    - Публикацию событий команд согласования.
    - Обновление ревью‑кандидатов (decisions, tasks, people, knowledge).

### CommunicationSend

Функция `send_email` (`communication_send.rs`) принимает запрос `CommunicationSendRequest`, проверяет наличие провайдерского аккаунта, формирует `OutgoingEmail` и ставит его в очередь outbox через `CommunicationCommandService::enqueue_outbox_send`. Возвращает `CommunicationSendResult` с `outbox_id`, статусом, списком получателей. Фиксирует аудит‑запись через `ApiAuditLog`.

### OrganizationContactLinkApplicationService

Метод `link_contact_manual` (`organization_contact_links.rs`) привязывает контакт (персону) к организации через `OrganizationCommandService::link_contact_manual`, после чего создаёт связь `member_of` в графе отношений с состоянием `UserConfirmed` и соответствующими evidence.

### Диспетчеризация AI‑сигналов

`dispatch_ai_runtime_signal` (`ai_signal_dispatch.rs`) принимает параметры сигнала (event_kind, source_id, subject, payload, provenance, correlation_id) и делегирует в `signal_hub::dispatch_ai_helper_signal`. При ошибке возвращает `EventStoreError::ConsumerHandlerFailed`.

### Provider Runtime Contracts

Модуль `provider_runtime_contracts` ре‑экспортирует типы и трейты для интеграций:
- Telegram: `TelegramStore`, команды, модели сообщений, топики, QR‑логин.
- WhatsApp: `WhatsAppProviderRuntime` (trait), сессии, команды, константы.
- Zoom: `ZoomStore`, OAuth, токены, записи, транскрипты, webhook'и.
- Yandex Telemost: `YandexTelemostStore`, очистка, транскрипты.

Также предоставляет конструкторы хранилищ (`telegram_provider_runtime_store`, `whatsapp_provider_runtime`, `zoom_provider_runtime_store`) с внедрением зависимостей.

### Provider Runtime Services

`ZoomProviderRuntimeApplicationService` предоставляет высокоуровневые методы: настройка аккаунтов, OAuth, обновление токенов, управление рантаймом, наблюдение встреч и записей, импорт медиа, синхронизация записей, подписки webhook и т.д.

`YandexTelemostProviderRuntimeApplicationService` предоставляет список аккаунтов, очистку и другие операции (подробности в усечённом фрагменте).

*Примечание:* Полный перечень методов `TelegramProviderRuntimeApplicationService` не входит в данный фрагмент.

## Обработка ошибок

Прикладной слой определяет несколько перечислений ошибок, объединяющих ошибки нижележащих доменов и инфраструктуры:

- `HostVaultReconciliationError`
- `CalendarMeetingOutcomeApplicationError` (`Sqlx`, `ObservationStoreError`, `MeetingsError`, `DecisionStoreError`, `ObligationStoreError`, `ReviewMirrorError`)
- `CommunicationSendError` (`InvalidRequest`, `ProviderAccountNotFound`, `CommunicationIngestionError`, `CommunicationCommandServiceError`, `ApiAuditError`)
- Многие модули используют `thiserror::Error` и `#[error(transparent)]` для проброса ошибок.

## Связанные страницы

*Другие страницы документации не включены в данный контекст, поэтому точные ссылки не указаны.*
```

### Source coverage / Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `backend/src/app/vault_reconciliation/errors.rs` | Описание перечисления `HostVaultReconciliationError` и входящих в него вариантов ошибок. |
| `backend/src/app/vault_reconciliation/lifecycle.rs` | Функция `spawn_host_vault_manifest_reconciliation`, условия её запуска, логирование результатов/ошибок. |
| `backend/src/app/vault_reconciliation/manifest_enrichment.rs` | `enrich_manifest_entry_from_postgres` — запрос к `communication_provider_accounts`/`communication_provider_account_secret_refs`, обогащение метаданных и вызов `vault.upsert_account_secret_manifest_entry`. |
| `backend/src/app/vault_reconciliation/metadata.rs` | Функции `fallback_provider_account_config`, `fallback_display_name`, `metadata_string`, `non_empty`. |
| `backend/src/app/vault_reconciliation/provider_recovery.rs` | Структура `RecoverableProviderSecret`, её построение из `HostVaultManifestEntry`, условия фильтрации (provider_credential, Gmail/iCloud/Imap, совместимость secret_kind/purpose). |
| `backend/src/app/vault_reconciliation/service.rs` | `reconcile_host_vault_manifest` и вспомогательные функции восстановления: `restore_secret_reference`, `restore_provider_account`, `restore_provider_account_secret_binding`, интеграция `restore_linked_calendar_account`. |
| `backend/src/app/vault_reconciliation/summary.rs` | Структура `HostVaultReconciliationSummary` с полями `restored_accounts` и `restored_calendar_accounts`. |
| `backend/src/application/ai_signal_dispatch.rs` | `dispatch_ai_runtime_signal` — делегирование в `signal_hub::dispatch_ai_helper_signal`, маппинг ошибок. |
| `backend/src/application/bootstrap.rs` (truncated) | `start_background_services`, `ApplicationBootstrapContext`, глобальные статические переменные для дедупликации, список сервисов, константы интервалов (`30 с`, `10 с`, `60 с`, `300 с`, `3600 с`) и параметры (`ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT = 100`, `ZOOM_RECORDING_SYNC_LOOKBACK_DAYS = 7` и др.). |
| `backend/src/application/calendar_meeting_outcomes.rs` | `CalendarMeetingOutcomeApplicationService::add_manual`, захват observation, создание `MeetingOutcome` в транзакции, формирование решений/обязательств в зависимости от `outcome_type` (`"decision"`, `"promise"`). |
| `backend/src/application/communication_fixture_ingest.rs` (truncated) | `TelegramFixtureIngestApplicationService::ingest_message` и `WhatsappFixtureIngestApplicationService::ingest_message_with_reconciliation_source` / `ingest_reaction_with_reconciliation_source` — сохранение, диспетчеризация, проекция, обновление ревью‑кандидатов. |
| `backend/src/application/communication_provider_writes.rs` (truncated) | Упоминание в описании модуля (канонические версии, tombstone, реакции) на основе видимых сигнатур функций `list_canonical_message_versions`, `list_canonical_message_tombstones`, `list_canonical_reactions` и т.п. |
| `backend/src/application/communication_send.rs` | `send_email` — проверка аккаунта, формирование `OutgoingEmail`, постановка в outbox, аудит. |
| `backend/src/application/consistency_review.rs` | Ре‑экспорт `crate::workflows::consistency_review::*`. |
| `backend/src/application/email_intelligence.rs` | Ре‑экспорт `crate::workflows::email_intelligence::*`. |
| `backend/src/application/mail_background_sync.rs` | Ре‑экспорт `crate::workflows::mail_background_sync::*`. |
| `backend/src/application/mod.rs` | Полный список подмодулей `application` и публичные/приватные ре‑экспорты. |
| `backend/src/application/organization_contact_links.rs` | `OrganizationContactLinkApplicationService::link_contact_manual`, создание связи `member_of` через `RelationshipReviewPort`, evidence. |
| `backend/src/application/person_derived_evidence.rs` | Ре‑экспорт `crate::workflows::person_derived_evidence::*`. |
| `backend/src/application/project_link_review_effects.rs` | Ре‑экспорт `crate::workflows::project_link_review_effects::*`. |
| `backend/src/application/project_link_review_mirror.rs` | Функции `ensure_project_link_candidate_review_item` и `sync_project_link_review_state_in_transaction`, делегирующие в `workflows::review_mirror`. |
| `backend/src/application/provider_runtime_contracts.rs` | Ре‑экспорт контрактов Telegram, WhatsApp, Zoom, Yandex Telemost; конструкторы хранилищ `telegram_provider_runtime_store`, `whatsapp_provider_runtime`, `zoom_provider_runtime_store`. |
| `backend/src/application/provider_runtime_services.rs` (truncated) | `ZoomProviderRuntimeApplicationService` и его методы (setup, OAuth, tokens, runtime lifecycle, recordings, transcripts, webhooks); `YandexTelemostProviderRuntimeApplicationService` (list_accounts). |
| `backend/src/application/realtime_conversation_transcript_execution.rs` | `complete_realtime_conversation_transcript_bridge` — вызов `integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge`. |
| `backend/src/application/realtime_conversation_transcript_projection.rs` | Ре‑экспорт `crate::workflows::realtime_conversation_transcript_projection::*`. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста (только исходные файлы, без существующих wiki‑страниц, ADR или другой документации) расхождений между кодом и документацией не выявлено.
