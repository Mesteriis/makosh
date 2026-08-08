## Summary / Резюме

Предлагается обновить страницу `components/backend.md` русской Obsidian‑wiki, отразив в ней новые компоненты бэкенда из группы `backend`, попавшие в чанк `036-source-backend-part-016`. В источники вошли исполняемые бинарники (`makosh_email_sync_dev`, `makosh_graph_project`, `makosh_migrate`, `makosh_whatsapp_business_cloud_edge_proxy`, `makosh_zoom_edge_proxy`), файл контрактов и ключевые модули домена «Календарь». Существующее содержимое wiki‑страницы недоступно в контексте, поэтому страница формируется заново.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты бэкенда Макошь

> **Чанк обновления:** `036-source-backend-part-016`
> **Дата контекста:** 2026-06-28

## Обзор

Бэкенд Макошь состоит из набора бинарных приложений (сервисов и утилит), генерируемых контрактов и модулей предметной логики. Основные слои: platform (конфигурация, БД), domains (календарь, коммуникации, граф), integrations (почта, мессенджеры), workflows (пайплайны синхронизации, проекции).

---

## Бинарные приложения (crate‑бинарники)

### `makosh_email_sync_dev`

Утилита разработческой синхронизации электронной почты через IMAP.

- **Конфигурация:** host, port, tls, mailbox, username, пароль, provider_kind, max_messages, blob_root (локальное хранилище блобов), account_id, import_batch_id.
- **Провайдеры:** поддерживаются `Icloud` и `Imap`; `Gmail`, `Telegram`, `WhatsApp`, `Zoom`, `YandexTelemost` отклоняются с ошибкой `UnsupportedProviderForDevSync`.
- **Хосты по умолчанию:** `Icloud` → `imap.mail.me.com`, `Imap` → `localhost`, остальные → пустая строка.
- **Выборка писем:**
  Формируются `ImapFetchOptions` с учётом последнего обработанного UID (`last_seen_uid`). Сообщения запрашиваются через `ImapNetworkClient::fetch_raw_messages`. Результат — `EmailSyncBatch`.
- **Пайплайн обработки:**
  Сначала в БД upsert‑ится аккаунт провайдера. Затем вызывается `project_email_sync_batch_with_mail_blobs` с `LocalCommunicationBlobStore` (файловая система по пути `blob_root`). Результат пайплайна фиксируется в `DevEmailSyncReport` (идентификатор аккаунта, провайдер, количество сообщений, чекпоинт и метрики пайплайна).

### `makosh_graph_project`

CLI‑утилита для выполнения проекции `v1` графа из БД и выдачи отчёта.

- Запускает `GraphProjectionService::project_from_v1()`.
- Получает `GraphSummary` через `GraphStore::summary()`.
- Выводит JSON‑отчёт с полями:
  - `projection` — количество upsert‑нутых узлов, рёбер, evidence.
  - `summary` — детальные счётчики узлов и рёбер по типам.
  - `total_nodes`, `total_edges` — агрегированные суммы по всем `GraphCount`.

### `makosh_migrate`

Утилита миграций и стартовых восстановлений.

- Загружает `AppConfig` из окружения.
- Подключается к БД через `Database::connect` (обязательно наличие `DATABASE_URL`).
- При успешном подключении выводит сообщение об окончании миграций и стартовых восстановлений.

### `makosh_whatsapp_business_cloud_edge_proxy`

Edge‑прокси (Axum‑сервер) для приёма вебхуков WhatsApp Business Cloud.

- **Назначение:** принимает публичные запросы из вне и пересылает их в защищённый контур Макошь.
- **Адрес по умолчанию:** `127.0.0.1:8787`.
- **Маршруты:**
  - `GET /healthz` — статус `ok`.
  - `GET /readyz` — проверяет доступность Макошь через `/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest`.
  - `GET /manifest` — возвращает конфигурацию прокси (пути, политики пересылки, наличие account_id).
  - `GET /webhooks/whatsapp/business-cloud` — пересылает GET‑запрос (query‑параметры) на защищённый путь `/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks`.
  - `POST /webhooks/whatsapp/business-cloud` — пересылает тело запроса и заголовок `X-Hub-Signature-256` на тот же защищённый путь.
- **Политики:** тело POST не парсится и не модифицируется; локальный API‑секрет передаётся только как заголовок `X-Макошь-Secret` и никогда не возвращается клиенту.
- **Конфигурация (переменные окружения):**
  - `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR` (по умолчанию `127.0.0.1:8787`)
  - `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL` (по умолчанию `http://127.0.0.1:8080`)
  - `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_SECRET` или `MAKOSH_LOCAL_API_SECRET` (обязательно)
  - `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID` (опционально, добавляется как query‑параметр `account_id` при пересылке)

### `makosh_zoom_edge_proxy`

Edge‑прокси для приёма вебхуков Zoom.

- **Адрес по умолчанию:** `127.0.0.1:8788`.
- **Маршруты:**
  - `GET /healthz` — статус `ok`.
  - `GET /readyz` — проверяет доступность Макошь через `/api/v1/integrations/zoom/capabilities`.
  - `GET /manifest` — возвращает конфигурацию прокси.
  - `POST /webhooks/zoom` — пересылает тело и заголовки `x-zm-signature`, `x-zm-request-timestamp` на `/api/v1/integrations/zoom/runtime-bridge/webhooks`.
- **Политики:** аналогичны WhatsApp‑прокси (тело не парсится, секрет не возвращается).
- **Конфигурация (переменные окружения):**
  - `MAKOSH_ZOOM_EDGE_BIND_ADDR` (по умолчанию `127.0.0.1:8788`)
  - `MAKOSH_ZOOM_EDGE_MAKOSH_BASE_URL` (по умолчанию `http://127.0.0.1:8080`)
  - `MAKOSH_ZOOM_EDGE_MAKOSH_SECRET` или `MAKOSH_LOCAL_API_SECRET` (обязательно)
  - `MAKOSH_ZOOM_EDGE_ACCOUNT_ID` (опционально)

---

## Генерируемые контракты

- **`backend/src/contracts.rs`** — подключает сгенерированные gRPC‑контракты через макрос `connectrpc::include_generated!();`. Детали генерации и конкретные RPC данным контекстом не подтверждены.

---

## Домен «Календарь» (domains/calendar)

### Архитектура

Домен построен по CQRS‑подобному принципу: команды выполняет `CalendarCommandService`, запросы (включая естественно‑языковые) — `CalendarBrainService`. Все операции изменяющие состояние фиксируются в таблице `observations` (платформенный компонент).

### `CalendarBrainService`

Сервис для обработки естественно‑языковых вопросов и генерации брифингов.

- **`answer(pool, question)`** — если вопрос содержит «недел», «week» или «brief», возвращает `weekly_overview`. Иначе выполняет `search_events`.
- **`weekly_overview(pool)`** — выбирает до 10 важных событий на ближайшие 7 дней (importance_score > 0.5 или типы meeting/deadline/tax/legal).
- **`search_events(pool, query)`** — поиск по ILIKE по title и description.
- **`meeting_brief(pool, event_id)`** — возвращает событие, участников и контекстный пакет (EventContextPackStore).
- **`generate_agenda(pool, event_id)`** — предлагает типовую повестку в зависимости от типа события (meeting / review / planning / прочее).

Тип ошибки: `CalendarBrainError` (Sqlx, CalendarCoreError, NotFound).

### `CalendarCommandService`

Сервис команд, каждая из которых:

1. Создаёт `observation` (с типом `CALENDAR_ACCOUNT_MUTATION`, `CALENDAR_EVENT`, `EVENT_AGENDA`, `EVENT_CHECKLIST`, `MEETING_NOTE`, …).
2. Передаёт `observation_id` в соответствующий store для привязки сущности к доказательной базе.

Методы (подтверждённые источниками):

- **CalendarAccount:** `create_calendar_account_manual`, `update_calendar_account_manual`, `delete_calendar_account_manual`.
- **CalendarSource:** `create_calendar_source_manual`.
- **EventAgenda:** `set_event_agenda_manual` (items – JSON, source).
- **EventChecklist:** `set_event_checklist_manual` (items – JSON, source).
- **EventParticipant:** `add_event_participant_manual` (email, display_name, role, …).
- **EventRelation:** `link_event_relation_manual` (entity_type, entity_id, relation_type).
- **MeetingNote:** `create_meeting_note_manual` (content, format, source).

> Получение/обновление событий, записей встреч, исходов, напоминаний, дедлайнов, фокус‑блоков и правил также реализованы аналогичным образом (файл обрезан, но сигнатуры методов вложены).

### Core‑сущности и их хранилища

#### EventAgenda (повестка события)

- Хранилище: `EventAgendaStore`.
- Методы: `get(event_id)`, `set`, `set_with_observation`.
- Свойства: `id`, `event_id`, `items` (JSON), `source`, `created_by`, `created_at`, `updated_at`.
- При `set_with_observation` вызывается `link_calendar_entity` для привязки к observation.

#### EventChecklist (чеклист события)

- Хранилище: `EventChecklistStore`.
- Аналогично `EventAgendaStore`, поле `items` (JSON). Привязка к observation.

#### EventContextPack (контекстный пакет события)

- Хранилище: `EventContextPackStore`.
- Использует общий движок `ContextPackStore` (`ContextPackKind::Calendar`).
- Поля: `summary`, `participants_summary`, `documents`, `tasks`, `open_questions`, `risks`, `suggested_agenda`, `suggested_actions`, `generated_at`, `model`.
- Методы: `get(event_id)`, `upsert(event_id, input)`. `upsert` создаёт `NewContextPack` с источником `ContextPackSourceKind::CalendarEvent`.

#### EventParticipant (участник события)

- Хранилище: `EventParticipantStore`.
- Методы: `list(event_id)`, `add`, `add_with_source`, `add_with_observation`.
- Поля: `person_id`, `email`, `display_name`, `role`, `response_status`, `source`, `organization_id`, `timezone`, `confidence`.

#### EventRelation (связь события)

- Хранилище: `EventRelationStore`.
- Методы: `list(event_id)`, `link`, `link_with_source`, `link_with_observation`.
- Поля: `entity_type`, `entity_id`, `relation_type`, `source`, `confidence`.
- Защита от дублирования: `get_by_identity` → `ON CONFLICT DO NOTHING`.

#### Вспомогательные функции

- **`link_calendar_entity`** в `core/evidence.rs` — делегирует в платформенную `link_domain_entity` с доменом `"calendar"`.

#### Ошибки core

- `CalendarCoreError` — объединяет `Sqlx`, `ObservationStoreError`, `ContextPackStoreError`, `NotFound`.

### Субдомен events (события и аккаунты)

#### CalendarAccount (аккаунт календаря)

- Хранилище: `CalendarAccountStore`.
- Методы: `create`, `create_with_observation`, `get`, `list`, `update`, `update_with_observation`, `upsert_google_workspace_account`, `upsert_apple_icloud_account`, `restore_google_workspace_account`, `restore_apple_icloud_account`, `delete`.
- Поля модели: `account_id` (префикс `cal:`), `provider`, `account_name`, `email`, `credentials_reference`, `sync_status`, `capabilities` (JSON), `created_at`, `updated_at`.
- Методы upsert/restore автоматически создают observation с типом `CALENDAR_ACCOUNT_LINK` и связывают аккаунт с учётной записью почты (`mail_account_id`).

#### CalendarSource (источник календаря)

- Хранилище: `CalendarSourceStore` (файл не включён, но модель экспортируется из `source_store.rs`).
- Поля модели: `source_id`, `account_id`, `provider_calendar_id`, `name`, `color`, `timezone`, `visibility`, `read_only`, `sync_enabled`, `capabilities`.

#### CalendarEvent (событие)

- Хранилище: `CalendarEventStore`.
- Ключевые методы создания:
  - `create_in_transaction` — для внутренней логики, с observation типа `LocalRuntime`.
  - `create_manual` / `create_manual_in_transaction` — ручное создание, тип `Manual`, привязка к source observation (опционально).
  - `create_file_import_in_transaction` — импорт из файла, тип `FileImport`.
- Каждое событие получает идентификатор формата `evt:v1:{timestamp_nanos_x}`.
- Модель `CalendarEvent` включает поля: `event_id`, `observation_id`, `source_event_id`, `account_id`, `source_id`, `title`, `description`, `location`, `start_at`, `end_at`, `timezone`, `all_day`, `recurrence_rule`, `status`, `visibility`, `event_type`, `importance_score`, `readiness_score`, `sync_status`, `conference_url`, `conference_provider`, `preparation_reminder_minutes`, `travel_buffer_minutes`.
- Для запросов используется `CalendarEventListQuery` с фильтрами: `account_id`, `source_id`, `from`, `to`, `status`, `event_type`, `limit`.

#### Ошибки events

- `CalendarError` — Sqlx, Observation, NotFound.

---

## Зависимости между модулями (кратко)

- `CalendarCommandService` → `CalendarAccountStore`, `CalendarSourceStore`, `EventAgendaStore`, `EventChecklistStore`, `EventParticipantStore`, `EventRelationStore`, `MeetingNoteStore`, `MeetingOutcomeStore`, `CalendarReminderStore`, `DeadlineStore`, `FocusBlockStore`, `CalendarRuleStore`.
- Все Store‑объекты принимают `PgPool` и реализуют методы с привязкой к `ObservationStore` (платформенный слой).
- `CalendarBrainService` работает напрямую с `PgPool` и `ContextPackStore`.

---

## Не отражено в данном чанке

- Полные реализации `event_store.rs`, `account_store.rs` и `command_service.rs` (файлы обрезаны до 12000 символов).
- Внутреннее устройство `source_store.rs`, `rows.rs`, `meetings/`, `reminders/`, `rules/`, `scheduling/`.
- Конкретные сгенерированные RPC из `contracts.rs`.
```

## Source coverage / Покрытие источников

| Source file (относительно `backend/src/`) | Факты, покрытые в `components/backend.md` |
| --- | --- |
| `bin/makosh_email_sync_dev/fetch.rs` | Использование `ImapFetchOptions`, `ImapNetworkClient::fetch_raw_messages` с паролем, поддержка `last_seen_uid` для инкрементальной синхронизации. |
| `bin/makosh_email_sync_dev/provider.rs` | Разбор `EmailProviderKind`: поддерживаемые (`Icloud`, `Imap`), неподдерживаемые; хосты по умолчанию. |
| `bin/makosh_email_sync_dev/report.rs` | Структура `DevEmailSyncReport`, включающая `account_id`, `provider`, `mailbox`, `fetched_messages`, `blob_root`, `checkpoint`, `pipeline`. |
| `bin/makosh_email_sync_dev/runner.rs` | Жизненный цикл запуска: подключение к БД, upsert аккаунта, получение чекпоинта, вызов `project_email_sync_batch_with_mail_blobs` с `LocalCommunicationBlobStore`, формирование отчёта. |
| `bin/makosh_graph_project.rs` | Запуск `project_from_v1`, получение `GraphSummary`, поля отчёта и подсчёт `total_nodes`/`total_edges`. |
| `bin/makosh_migrate.rs` | Подключение к БД и вывод сообщения о завершении миграций. |
| `bin/makosh_whatsapp_business_cloud_edge_proxy.rs` (обрезан) | Маршруты (`/healthz`, `/readyz`, `/manifest`, GET/POST `/webhooks/...`), конфигурация окружения, политики пересылки, передача `X-Макошь-Secret` и `X-Hub-Signature-256`, порт по умолчанию `8787`. |
| `bin/makosh_zoom_edge_proxy.rs` (обрезан) | Маршруты, переменные окружения, пересылка `x-zm-signature` и `x-zm-request-timestamp`, порт по умолчанию `8788`. |
| `contracts.rs` | Макрос `connectrpc::include_generated!();` для генерируемых контрактов. |
| `domains/calendar/brain.rs` | `CalendarBrainService::answer`, `weekly_overview`, `search_events`, `meeting_brief`, `generate_agenda`; логика обработки вопросов. |
| `domains/calendar/command_service.rs` (обрезан) | Наличие `CalendarCommandService`, фиксация observation, методы для аккаунтов, источников, повесток, чеклистов, участников, связей, заметок. |
| `domains/calendar/core.rs` | Экспортируемые сущности core: `EventAgenda`, `EventChecklist`, `EventContextPack`, `EventParticipant`, `EventRelation`, их store. |
| `domains/calendar/core/agendas.rs` | Методы `EventAgendaStore` (`get`, `set`, `set_with_observation`), поля, привязка к observation. |
| `domains/calendar/core/checklists.rs` | Аналогично для чеклистов. |
| `domains/calendar/core/context_packs.rs` | `EventContextPackStore` на основе `ContextPackStore`, преобразование `ContextPack` в `EventContextPack`. |
| `domains/calendar/core/errors.rs` | `CalendarCoreError` (Sqlx, Observation, ContextPack, NotFound). |
| `domains/calendar/core/evidence.rs` | Функция `link_calendar_entity`, делегирующая в `link_domain_entity`. |
| `domains/calendar/core/participants.rs` | Методы `EventParticipantStore` (`list`, `add`, `add_with_source`, `add_with_observation`), поля участника. |
| `domains/calendar/core/relations.rs` | Методы `EventRelationStore` (`list`, `link`, `link_with_source`, `link_with_observation`), защита от дублирования. |
| `domains/calendar/events.rs` | Экспорт моделей, хранилищ, запроса. |
| `domains/calendar/events/account_store.rs` (обрезан) | Сигнатуры создания, обновления, upsert/restore связанных аккаунтов (Google, iCloud); использование `ObservationStore`. |
| `domains/calendar/events/errors.rs` | `CalendarError` (Sqlx, Observation, NotFound). |
| `domains/calendar/events/event_store.rs` (обрезан) | Методы создания событий с разными origin‑типами, идентификатор события `evt:v1:{ts:x}`, поля `NewCalendarEvent`. |
| `domains/calendar/events/models.rs` | Структуры `CalendarAccount`, `CalendarSource`, `CalendarEvent`, `NewCalendarEvent`, `CalendarEventUpdate`, `CalendarAccountUpdate`. |
| `domains/calendar/events/queries.rs` | `CalendarEventListQuery` с фильтрами. |

## Drift candidates / Кандидаты на drift

Текущий контекст не содержит существующей версии страницы `components/backend.md` и не содержит ADR‑документов. Поэтому установить расхождение между предложенным текстом и текущей wiki‑документацией невозможно. Все утверждения в предложенной странице основаны исключительно на вложенных исходных файлах. Для выявления drift требуется сравнение с актуальным содержимым `docs/wiki/components/backend.md` и с проектными ADR.
