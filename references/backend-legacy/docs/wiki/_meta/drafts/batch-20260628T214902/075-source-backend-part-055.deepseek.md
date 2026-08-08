## Summary / Резюме

Предлагается создать (или обновить) страницу `components/backend.md` в русской wiki. Она описывает два ключевых backend‑компонента, видимых из предоставленных исходных файлов: сервис проецирования графа (`GraphProjectionService`) и службу фоновой синхронизации почты (`MailBackgroundSyncService`). Страница содержит верхнеуровневое описание их назначения, основных моделей и поведения, подтверждённое исходным кодом.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты бэкенда Макошь

На основе доступных исходных файлов можно выделить два основных компонента бэкенда:

- **GraphProjectionService** – проецирует данные из реляционного хранилища в граф знаний.
- **MailBackgroundSyncService** – выполняет фоновую синхронизацию почтовых ящиков с последующей проекцией в граф.

## Сервис проецирования графа (GraphProjectionService)

Сервис `GraphProjectionService` описан в модуле `graph_projection`. Он зависит от пула соединений (`PgPool`), порта `GraphProjectionPort` и порта `ProjectCommandPort`.

- `new(pool)` – создаёт экземпляр, инициализируя все порты.
- `project_from_v1()` – выполняет полное проецирование, поочерёдно обрабатывая:
  1. **Персоны** (вызовы `list_persons` → `project_person`)
  2. **Сообщения**
  3. **Документы**
  4. **Проекты** (через `projects.graph_projection_projects()`)
  5. **Решения**
  6. **Обязательства**

Результат работы – `GraphProjectionReport` со счётчиками созданных/обновлённых узлов, рёбер и свидетельств.

### Проекция персоны

- **Чтение:** SQL‑запрос `SELECT person_id, display_name, email_address FROM persons ORDER BY person_id`.
- **Обработка одной персоны:**
  - Нормализуется email‑адрес (функция `normalize_email_address`).
  - Создаётся (или перезаписывается) узел `Person` с идентификатором `person_id`, именем `display_name` и свойством `email_address`.
  - Создаётся узел `EmailAddress` с нормализованным email.
  - Между ними создаётся ребро `PersonHasEmailAddress` с весом `1.0` и состоянием `SystemAccepted`. К ребру добавляется свидетельство (`GraphEvidence`) с типом источника `Person` и идентификатором `person_id`.

### Проекция проекта

- **Входные данные:** `ProjectProjectionSource` (проект + ключевые слова).
- **Шаги:**
  1. Получение связанных сообщений (`matching_project_messages`) и документов (`matching_project_documents`).
  2. Начало транзакции.
  3. Создание/обновление узла `Project` со свойствами: `kind`, `status`, `description`, `owner_display_name`, `progress_percent`, `start_date`, `target_date`, `keywords`.
  4. Удаление старых рёбер, исходящих от узла проекта, с типами `project_has_message`, `project_has_document`, `project_involves_person`, `project_involves_email_address`.
  5. Для каждого сообщения:
     - Ребро `ProjectHasMessage` с признаком `match_rule = "project_keyword"`, уверенность и состояние определяются `review_state` сообщения.
     - Проекция участников сообщения: из полей `sender` и `recipients` собираются email‑адреса, для каждого разрешается endpoint (персона или email‑адрес), и создаётся соответствующее ребро (`project_involves_person` / `project_involves_email_address`).
  6. Для каждого документа:
     - Ребро `ProjectHasDocument` с признаком `match_rule = "project_keyword"`.
  7. Фиксация транзакции.

Все операции в транзакции используют `GraphProjectionPort::upsert_edge_with_evidence_in_transaction`.

### Вспомогательные структуры строк

Функции `row_to_person`, `row_to_message`, `row_to_document` преобразуют строки БД в соответствующие типы. `recipients_from_value` извлекает массив строк из JSON‑значения `recipients`.

## Служба фоновой синхронизации почты (MailBackgroundSyncService)

`MailBackgroundSyncService` управляет периодическим и ручным получением почты, её сохранением и проекцией в граф. Используются следующие внешние зависимости:

- `HostVault` для проверки доступности хранилища учётных данных.
- `SharedEmailProviderSyncPort` для непосредственного обращения к Gmail/IMAP.
- `CommunicationIngestionPort` и `CommunicationProviderAccountPort` для работы с аккаунтами и чекпоинтами.
- Локальное файловое хранилище BLOB (`blob_root`).

### Основные константы

| Константа | Значение |
|-----------|----------|
| `DEFAULT_MAIL_SYNC_BATCH_SIZE` | 100 |
| `DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS` | 300 |
| `MAX_BATCH_SIZE` | 500 |
| `MIN_POLL_INTERVAL_SECONDS` | 60 |
| `MAX_POLL_INTERVAL_SECONDS` | 86 400 |
| `DEFAULT_GMAIL_API_BASE_URL` | `https://www.googleapis.com` |

### Модели данных

- **MailSyncRun** – детальное состояние одного запуска синхронизации: идентификатор запуска (`run_id`), аккаунт, триггер, статус, фаза, режим прогресса, процент, счётчики обработанных/полученных/спроецированных сообщений, чекпоинты, коды ошибок, временные метки.
- **MailSyncRunResponse** – ответ API, производный от `MailSyncRun`. Содержит `failure_reason` типа `MailSyncFailureReason` (код + сообщение) и булевы признаки наличия чекпоинтов.
- **MailSyncStatus** – сводный статус аккаунта: текущие статус/фаза/прогресс, время последнего запуска/завершения, время следующего запуска, последняя ошибка.
- **MailSyncSettings** – настройки синхронизации: `account_id`, `sync_enabled`, `batch_size`, `poll_interval_seconds`, `updated_at`.
- **MailSyncSettingsUpdate** – данные для обновления настроек (те же поля, кроме `account_id` и `updated_at`).
- **MailSyncDueAccount** – аккаунт, готовый к синхронизации (содержит `batch_size` и `poll_interval_seconds`).
- **MailSyncTrigger** – тип триггера: `Scheduled` (`"scheduled"`) или `Manual` (`"manual"`).
- Внутренние перечисления (не экспортируются):
  - `MailSyncPhase`: `Listing`, `Fetching`, `Projecting`, `PersonsGraph`, `Completed`, `Failed`.
  - `MailSyncRunStatus`: `Completed`, `Failed`, `Skipped`.
  - `ProgressMode`: `None`, `Determinate`, `Indeterminate`.
- **ProgressUpdate** – структура для передачи информации о прогрессе (run_id, фаза, режим, процент, счётчики, размер батча).
- **FinishRun** – состояние завершённого запуска для внутреннего учёта.

### Обработка ошибок

Ошибки синхронизации агрегируются в `MailSyncError`, который включает:

- Ошибки БД (`Sqlx`), событий (`EventEnvelope`, `EventLogPort`), наблюдений (`ObservationPort`), коммуникаций.
- Специфичные: `AccountNotFound`, `RunAlreadyActive`, `RunNotFound`, `InvalidSetting`.
- `ProviderSyncError` (внутренний) объединяет ошибки провайдера, пайплайна, графа, состояния синхронизации.

`SanitizedSyncFailure` преобразует низкоуровневые ошибки в безопасные для пользователя коды:

| Код ошибки | Причина |
|------------|---------|
| `provider_config_invalid` | Некорректная конфигурация провайдера |
| `vault_locked` | Хранилище учётных данных заблокировано |
| `vault_uninitialized` | Хранилище не инициализировано |
| `vault_unavailable` | Хранилище недоступно |
| `credential_unavailable` | Отсутствует или некорректна учётная запись провайдера |
| `oauth_refresh_failed` | Ошибка обновления OAuth‑токена |
| `provider_network_error` | Сетевая ошибка при обращении к провайдеру |
| `projection_failed` | Ошибка пайплайна проекции почты |
| `graph_projection_failed` | Ошибка проекции в граф |
| `communication_store_error` | Ошибка хранилища коммуникаций |
| `sync_store_error` | Ошибка хранилища состояния синхронизации |

### Жизненный цикл синхронизации

1. **Запуск по расписанию:** `run_due_accounts()` запрашивает до 20 аккаунтов, готовых к синхронизации (через `store.due_accounts(now, 20)`), и для каждого вызывает `run_account` с триггером `Scheduled`.
2. **Запуск конкретного аккаунта (`run_account`):**
   - Если синхронизация отключена (`sync_enabled == false`) – запуск помечается как skipped с кодом `sync_disabled`.
   - Получение плана синхронизации (`plan_email_sync(&account)`). При ошибке – немедленный сбой без вызова провайдера.
   - Извлечение последнего чекпоинта из `CommunicationIngestionPort`.
   - Проверка разблокирован ли Vault (`require_unlocked_vault`). При ошибке – сбой с кодом `vault_locked` и т.п.
   - Выполнение провайдер-специфичной синхронизации через `execute_provider_sync`.
   - По результатам – завершение с заполнением `FinishRun` (успех) или `FinishRun::failed` (ошибка).
3. **Полная ресинхронизация:** `run_account_full_resync` удаляет чекпоинт для потока (`delete_checkpoint`), затем вызывает `run_account` с триггером `Manual`.

### Провайдер‑специфичная синхронизация

Метод `execute_provider_sync` получает конфигурацию адаптера (`EmailSyncAdapterConfig`) и контекст `ProviderSyncContext`, и вызывает:

- `sync_gmail` для Gmail.
- `sync_imap` с конфигурацией `ImapAccountConfig` (хост, порт, TLS, почтовый ящик) для IMAP.

**Gmail‑синхронизация:**
- Проверяет чекпоинт: `next_page_token` и `page_kind`. Если есть токен и `page_kind != "history"` – продолжает перебор страниц списка сообщений.
- Иначе пытается получить историю изменений (Gmail History API), начиная с `start_history_id`. Если история истекла (`history_expired`), происходит полная ресинхронизация списка сообщений.
- Перебор страниц (`sync_gmail_message_list_pages` и `sync_gmail_history_pages`) выполняется в цикле с обновлением прогресса (`MailSyncPhase::Listing`, `ProgressMode::Indeterminate`). Каждый полученный батч сразу передаётся в `project_batch`.

**IMAP‑синхронизация:**
- Работает в цикле, используя `last_seen_uid` из чекпоинта.
- При обнаружении смены `uid_validity` чекпоинт сбрасывается (`last_seen_uid = None`) и синхронизация повторяется (однократно).
- Батчи запрашиваются через `fetch_imap_messages` с параметрами `max_messages = settings.batch_size`, `last_seen_uid`.
- После получения каждого батча вызывается `project_batch`.

### Обработка батча сообщений

Метод `project_batch`:
1. Обновляет прогресс (`MailSyncPhase::Projecting`).
2. Создаёт `LocalCommunicationBlobPort` и вызывает `project_email_sync_batch_with_mail_blobs`, передавая пул, хранилище BLOB, account_id и идентификатор батча.
3. Применяет отчёт пайплайна (`apply_pipeline_report`) к `ProviderSyncSummary`: счётчики `projected_messages`, `upserted_persons`, `upserted_organizations`.
4. Обновляет прогресс (`MailSyncPhase::PersonsGraph`).
5. Запускает полное проецирование в граф: `GraphProjectionService::new(pool).project_from_v1()`.
6. Если в батче присутствует чекпоинт, обновляет `checkpoint_saved` в сводке.

### События и наблюдения

- При старте, прогрессе и завершении синхронизации генерируются события с типами:
  - `mail.sync.started`
  - `mail.sync.progress`
  - `mail.sync.completed`
  - `mail.sync.failed`
  - `mail.sync.skipped`
- Событие содержит уникальный идентификатор, временную метку, данные о запуске (run_id, account_id, trigger, status, phase, progress, счётчики, ошибки), а также provenance с `source_kind = "mail_sync_run"`.
- Для каждого запуска (`MailSyncRun`) создаётся `Observation` с полным слепком состояния запуска, устанавливается связь с сущностью типа `mail_sync_run` через `link_mail_entity_in_transaction` (в рамках транзакции).

### Вспомогательные функции чтения из БД

`row_to_settings`, `row_to_status`, `row_to_due_account`, `row_to_run` – преобразуют строки `PgRow` в соответствующие структуры. Поля извлекаются по именам колонок (например, `account_id`, `batch_size`, `poll_interval_seconds`, `status`, `phase`, и т.д.), точный список колонок виден в коде этих функций.

## Взаимодействие между компонентами

`MailBackgroundSyncService` после обработки каждого батча вызывает `GraphProjectionService::project_from_v1()` для обновления графа знаний свежими данными. Таким образом, фоновая синхронизация почты автоматически поддерживает актуальность графа.
```

## Source coverage / Покрытие источников

- **`backend/src/workflows/graph_projection/persons.rs`** – чтение персон из БД, нормализация email, создание узлов Person/EmailAddress и ребра PersonHasEmailAddress с evidence.
- **`backend/src/workflows/graph_projection/projects.rs`** – проекция проектов: получение сообщений и документов, транзакционное обновление узла Project и рёбер, удаление устаревших рёбер, проекция участников.
- **`backend/src/workflows/graph_projection/rows.rs`** – функции маппинга строк БД в PersonRow, MessageRow, DocumentRow; разбор recipients из JSON.
- **`backend/src/workflows/graph_projection/service.rs`** – структура GraphProjectionService и метод `project_from_v1`, итерация по сущностям и вызов методов проекции.
- **`backend/src/workflows/mail_background_sync.rs`** (корневой модуль) – список констант, подмодули, публичный экспорт основных типов.
- **`backend/src/workflows/mail_background_sync/errors.rs`** – перечисление ошибок MailSyncError и ProviderSyncError.
- **`backend/src/workflows/mail_background_sync/events.rs`** – типы событий и функция создания NewEventEnvelope для стадий синхронизации.
- **`backend/src/workflows/mail_background_sync/evidence.rs`** – создание Observation для запуска синхронизации и привязка к mail entity.
- **`backend/src/workflows/mail_background_sync/models.rs`** – реэкспорт основных и внутренних моделей.
- **`backend/src/workflows/mail_background_sync/models/failures.rs`** – SanitizedSyncFailure и отображение ошибок в пользовательские коды.
- **`backend/src/workflows/mail_background_sync/models/finish.rs`** – структура FinishRun и фабрика failed.
- **`backend/src/workflows/mail_background_sync/models/progress.rs`** – MailSyncTrigger, ProgressUpdate, MailSyncPhase, MailSyncRunStatus, ProgressMode.
- **`backend/src/workflows/mail_background_sync/models/runs.rs`** – MailSyncRun и MailSyncRunResponse.
- **`backend/src/workflows/mail_background_sync/models/settings.rs`** – MailSyncSettings, MailSyncSettingsUpdate, MailSyncDueAccount.
- **`backend/src/workflows/mail_background_sync/models/status.rs`** – MailSyncStatus.
- **`backend/src/workflows/mail_background_sync/provider.rs`** – диспетчеризация execute_provider_sync на Gmail/IMAP.
- **`backend/src/workflows/mail_background_sync/provider/gmail.rs`** – логика Gmail‑синхронизации с чекпоинтами и переключением history/message list.
- **`backend/src/workflows/mail_background_sync/provider/gmail/history.rs`** – перебор страниц истории Gmail.
- **`backend/src/workflows/mail_background_sync/provider/gmail/message_list.rs`** – перебор страниц списка сообщений Gmail.
- **`backend/src/workflows/mail_background_sync/provider/imap.rs`** – IMAP‑синхронизация с UID и обработкой смены uid_validity.
- **`backend/src/workflows/mail_background_sync/provider/projection.rs`** – project_batch: вызов пайплайна и GraphProjectionService.
- **`backend/src/workflows/mail_background_sync/provider/summary.rs`** – ProviderSyncSummary.
- **`backend/src/workflows/mail_background_sync/provider/types.rs`** – ProviderSyncContext, ImapAccountConfig.
- **`backend/src/workflows/mail_background_sync/rows.rs`** – маппинг строк БД в MailSyncSettings, MailSyncStatus, MailSyncDueAccount, MailSyncRun.
- **`backend/src/workflows/mail_background_sync/service.rs`** – MailBackgroundSyncService: run_due_accounts, run_account, run_account_full_resync, fail_without_provider_io.

## Drift candidates / Кандидаты на drift

В предоставленном контексте нет встроенных ранее написанных wiki‑страниц, документации или ADR для сравнения. Поэтому расхождения кода/документации не видны.
