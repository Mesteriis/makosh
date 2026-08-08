---
chunk_id: 030-source-backend-part-010
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 030-source-backend-part-010 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

В wiki-страницу `components/backend.md` необходимо добавить описание слоя HTTP-обработчиков backend-приложения (`backend/src/app/handlers`), основанное исключительно на предоставленных исходных файлах. Предлагаемое содержимое документирует каждый доменный модуль обработчиков, их публичные функции, модели запросов/ответов, валидацию, аудит и основные паттерны использования доменных сервисов и хранилищ. Цель — актуализировать страницу, отразив API-поверхность, видимую в данном чанке.

## Предложенные страницы

**Страница:** `components/backend.md`

```markdown
# Backend (API-обработчики)

## Обзор

Данный документ описывает слой HTTP-обработчиков backend-приложения Макошь,
расположенный в `backend/src/app/handlers`. Обработчики сгруппированы по доменным
областям и реализованы с использованием фреймворка Axum.

Каждый обработчик получает состояние приложения (`AppState`) через extractor `State`,
извлекает пул соединений с базой данных и делегирует бизнес-логику
доменным сервисам или хранилищам (stores). Ошибки преобразуются в `ApiError`.

## Структура модулей

Обработчики разбиты на следующие модули:

- **decisions** – решения
- **documents** – обработка документов
- **events** – события и аудит
- **graph** – графовая модель
- **obligations** – обязательства
- **organizations** – организации
- **persons** – лица (контакты, экспертиза, аналитика)

## Decisions (решения)

Источник: `backend/src/app/handlers/decisions/`

### Модели запросов и ответов

- `DecisionListQuery` – параметры запроса списка решений: `entity_kind`, `entity_id`, `review_state`, `limit`.
- `DecisionReviewApiRequest` – тело запроса на изменение review-статуса: `review_state`.
- `DecisionListResponse` – ответ со списком элементов `Decision[]`.

### Обработчики

- `get_v1_decisions` – получить список решений.
- `put_v1_decision_review` – установить review-статус конкретного решения.

*Детали реализации обработчиков не входят в данный контекст; известны только сигнатуры.*

## Documents (документы)

Источник: `backend/src/app/handlers/documents/mod.rs`

### Обработчики

- `get_document_processing` – получить запись обработки документа по `document_id`; возвращает `DocumentProcessingRecord`.
- `get_document_processing_jobs` – список задач обработки документов; принимает `RawQuery` для извлечения параметра `limit`; возвращает `DocumentProcessingJobsResponse`.
- `post_document_processing_job_retry` – повторный запуск упавшей задачи обработки; путь содержит `job_id`; тело запроса `DocumentProcessingRetryApiRequest` содержит команду; выполняется аудит через `api_audit_log`; используется `DocumentProcessingCommandService` для выполнения; возвращает результат.

*В файле присутствуют многочисленные импорты, но определены только три перечисленных функции.*

## Events (события)

Источник: `backend/src/app/handlers/events/handlers.rs` (файл обрезан на 12000 символов; показаны не все обработчики)

### Обработчики событий

- `post_event` – добавить событие; тело `AppendEventRequest`; возвращает `AppendEventResponse` (event_id, position); записывает аудит.
- `get_event` – получить событие по `event_id`; возвращает `EventEnvelope`; записывает аудит.
- `get_event_trace` – трассировка события по `event_id`; параметр `limit` (default 1000); возвращает `EventTrace`, payloads санитизируются.
- `get_event_trace_by_correlation` – трассировка по `correlation_id`; аналогичен предыдущему.
- `get_event_children` – дочерние события по `event_id`; `limit` (default 1000); payloads санитизируются.
- `get_events` – список событий с пагинацией: `after_position` (default 0, >=0), `limit` (1..1000, clamp), `wait_seconds` (0..30, clamp). Реализует long-polling: если событий нет, ждёт до deadline с шагом 500мс.
- `get_events_stream` – SSE-поток событий: `after_position`, `batch_size` (1..1000), `heartbeat_seconds` (1..60). Отправляет события по мере появления; при отсутствии — heartbeat; при ошибке опроса — ошибка в стриме.
- `get_events_websocket` – WebSocket-аналог SSE-потока; использует `EventStreamState` и цикл `event_websocket_loop` с сообщениями типа `event`, `heartbeat`, `error`.

### Аудит

- `get_audit_events` – список записей аудита: фильтры `target_id`, `actor_id`, `after_audit_id`, `limit`.

### Внутренние детали

- `EventStreamState` – структура для поддержки стриминга.
- `stored_event_to_sse` – сериализация события в SSE Event.
- `event_websocket_loop` – основной цикл WebSocket.
- `send_ws_json` – отправка JSON-фрейма через WebSocket.
- `event_list_response` – формирование ответа с пагинацией.
- `heartbeat_event`, `stream_error_event` – вспомогательные SSE-события.

*Обработчики из оставшейся части файла не видны из-за обрезки.*

## Graph (граф)

Источник: `backend/src/app/handlers/graph/mod.rs`

### Обработчики

- `get_graph_summary` – сводка графа; возвращает `GraphSummary`.
- `get_graph_nodes` – узлы для выбора (picker); `RawQuery` извлекает `limit` (1..50); возвращает `Vec<GraphNode>`.
- `get_graph_neighborhood` – соседи узла: параметры `node_id` (обязателен), `depth` (только 1, иначе ошибка). Возвращает `GraphNeighborhood` или 404.
- `get_graph_search` – поиск узлов: `q` (обязателен, непустой), `limit` (1..50). Возвращает `Vec<GraphNode>`.

## Obligations (обязательства)

Источник: `backend/src/app/handlers/obligations/`

### Модели

- `ObligationListQuery`: фильтры `entity_kind`, `entity_id`, `review_state`, `limit`.
- `ObligationReviewApiRequest`: `review_state`.
- `ObligationListResponse`: `items: Vec<Obligation>`.

### Обработчики

- `get_v1_obligations` – список обязательств:
  - Если передан только `review_state` – фильтр по статусу; limit проверяется (1–100).
  - Если переданы `entity_kind` и `entity_id` – фильтр по сущности.
  - Комбинация `review_state` с entity-фильтрами запрещена (ошибка).
  - Отсутствие всех фильтров – ошибка "missing required obligation query field".
- `put_v1_obligation_review` – установить review-статус обязательства:
  - Использует `ObligationReviewApplicationService` для ручного review.
  - Записывает аудит (`NewApiAuditRecord::obligation_review_set`).
  - Возвращает обновлённый `Obligation`.

### Константы

- Лимит по умолчанию: 50, минимум 1, максимум 100.
- Actor ID для аудита: `"makosh-frontend"`.

## Organizations (организации)

Источник: `backend/src/app/handlers/organizations/`

Модуль разделён на файлы: `directory`, `core_records`, `enrichment`, `finance`, `health`, `investigator`, `workflows`, `support`.

### Directory (управление профилями)

- `get_organizations` – список организаций: фильтр `org_type`, `limit` (default 50).
- `post_organization` – создать организацию: `display_name`, `org_type`; использует `OrganizationCommandService`.
- `get_organization` – получить организацию по `org_id`; 404 если не найдено.
- `put_organization` – обновить организацию: принимает `OrganizationUpdate`; использует `OrganizationCommandService`.
- `get_organization_search` – текстовый поиск по `display_name`, `legal_name`, `website`; `q` (запрос), `limit` (1..100). Поиск регистронечувствительный, перебирает все организации (лимит выборки 200).
- `post_organization_archive` – архивировать организацию.

### Core Records (основные записи)

- **Identities**: `get_org_identities`, `post_org_identity` (добавление identity через `add_identity_manual`).
- **Aliases**: `get_org_aliases`, `post_org_alias` (добавление псевдонима через `add_alias_manual`).
- **Domains**: `get_org_domains` (список доменов).
- **Departments**: `get_org_departments`, `post_org_department` (добавление подразделения через `add_department_manual`).
- **Contacts**: `get_org_contacts`, `post_org_contact_link` (привязка контакта через `OrganizationContactLinkApplicationService`).
- **Related Orgs**: `get_org_related` (связанные организации).

### Enrichment (обогащение)

- `get_org_enrichment` – список результатов обогащения (`OrgEnrichmentResult`).
- `post_org_enrich_apply` – применить результат обогащения по `rid`; возвращает `{"applied": true}`.

### Finance (финансы)

- `get_org_financial` – финансовая информация.
- `get_org_contracts` – контракты.
- `get_org_compliance` – статусы комплаенса.
- `get_org_services` – услуги.
- `get_org_products` – продукты.

### Health (здоровье)

- `get_org_risks` – риски.
- `get_org_health` – состояние здоровья организации.
- `post_org_watchlist_toggle` – переключить наблюдение (watchlist).

### Investigator (расследователь)

- `get_org_dossier` – агрегированное досье.
- `get_org_brief` – краткая сводка.
- `get_org_context_pack` – контекстный пакет (context pack).

### Workflows (рабочие процессы)

- `get_org_timeline` – хронология событий с параметром `limit` (default 50).
- `get_org_portals` – порталы.
- `get_org_procedures` – процедуры.
- `get_org_playbooks` – playbook'и.
- `get_org_templates` – шаблоны.

## Persons (лица)

Источник: `backend/src/app/handlers/persons/`

Модуль содержит файлы: `compatibility`, `errors`, `health`, `history`, `identity`, `intelligence`, `investigator`, `support` (не показан).

### Compatibility (роли, персоны)

- **Roles**:
  - `get_person_roles` – роли лица.
  - `post_person_role` – назначить роль (`PersonCommandService::assign_role_manual`).
  - `delete_person_role` – удалить роль; возвращает `{"deleted": bool}`.
- **Personas**:
  - `get_person_personas` – персоны лица.
  - `post_person_persona` – создать/обновить персону (`PersonCommandService::upsert_person_persona_manual`).
  - `delete_person_persona` – удалить персону; возвращает `{"deleted": bool}`.

### Errors (маппинг ошибок)

Преобразования ошибок из доменных типов в `ApiError`:

- `EnrichmentEngineError` → `InvalidCommunicationQuery("enrichment engine operation failed")`.
- `PersonExpertiseError` → аналогично.
- `PersonTrustError` → аналогично.
- `PersonHealthError` → аналогично.
- `InvestigatorError` → детальный маппинг: `PersonNotFound`/`DossierSnapshotNotFound` → `PersonIdentityNotFound`; `InvalidDossierReviewState` → `InvalidCommunicationQuery` с сообщением о допустимых состояниях; прочие → общая ошибка.
- `AnalyticsError` → общая ошибка.
- `ExportError` → общая ошибка.

Все ошибки логируются через `tracing::error!`.

### Health (здоровье)

- `get_person_health` – здоровье конкретного лица; 404 если не найдено.
- `get_persons_health` – список здоровья всех лиц.
- `get_persons_watchlist` – список watchlist.
- `post_person_watchlist_toggle` – переключить наблюдение.

### History (история, аналитика)

- `get_person_analytics` – аналитика по лицу (через `PersonAnalyticsService`).
- `get_person_export_handler` – экспорт данных лица: формат из параметра `format` (по умолчанию JSON); возвращает файл с Content-Disposition.
- `get_person_snapshots` – снапшоты истории.
- `get_person_history_diff` – разница между двумя снапшотами: параметры `from`, `to` (RFC 3339).
- `get_identity_candidates` – кандидаты на идентификацию: `RawQuery` с `limit`.
- `put_identity_candidate_review` – ручной review кандидата: через `PersonCommandService::review_identity_candidate_manual`; аудит.
- `get_person_identity` – полная идентичность лица (`PersonIdentityDetail`).

### Identity (идентичности)

- `get_person_identities` – идентичности лица.
- `get_identity_traces` – неприкреплённые identity traces: статус должен быть `unattached` (иначе ошибка).
- `post_identity_trace` – создать identity trace вручную (`PersonCommandService::create_identity_trace_manual`).
- `put_identity_trace_assignment` – привязать trace к лицу (`PersonCommandService::assign_identity_trace_manual`).
- `post_person_identity` – добавить идентичность лицу (`PersonCommandService::upsert_person_identity_manual`).
- `delete_person_identity` – удалить идентичность; возвращает `{"deleted": bool}`.

### Intelligence (разведка, экспертиза)

- **Enrichment (обогащение)**:
  - `get_person_enrichment` – список результатов обогащения.
  - `post_person_enrichment_apply` – применить результат.
  - `post_person_enrichment_reject` – отклонить результат.
- **Expertise (экспертиза)**:
  - `get_person_expertise` – список компетенций лица.
  - `get_person_expertise_search` – поиск по навыку (`skill`, `limit`).
- **Promises (обещания)**:
  - `get_person_promises` – обещания.
- **Risks (риски)**:
  - `get_person_risks` – риски.

### Investigator (расследователь)

- `post_person_investigate` – запуск обновления кеша досье (refresh) и возврат досье и снапшота.
- `get_person_dossier` – получить досье с кешированием.
- `put_person_dossier_review` – установить review-статус досье; принимает `review_state`, парсится через `DossierReviewState::parse`.
- `get_person_meeting_prep` – подготовка к встрече (meeting prep).

Вспомогательные функции формируют JSON-ответы с дополнительной метаинформацией (dossier_snapshot_id, review_state, reviewed_by, reviewed_at).

## Общие паттерны

- Все обработчики получают состояние `State<AppState>`.
- Доступ к БД через `state.database.pool().ok_or(ApiError::DatabaseNotConfigured)`.
- Доменные хранилища создаются через `crate::app::api_support::app_store::<Store>(pool)`.
- Доменные сервисы создаются напрямую через конструктор (например, `PersonCommandService::new(pool)`, `ObligationReviewApplicationService::new(pool)`).
- Операции, изменяющие состояние, записываются в аудит-лог (`api_audit_log`).
- Валидация входных параметров выполняется в обработчиках: проверка на пустоту, допустимые диапазоны, допустимые комбинации фильтров.
- Ошибки логируются через `tracing::error!` при маппинге.

## Примечание

Данный документ основан исключительно на предоставленных файлах обработчиков.
Обработчики, определённые в других модулях (например, AI, задачи, календарь),
не показаны в данном контексте, но могут существовать в проекте.
```

## Покрытие источников

- `backend/src/app/handlers/decisions/mod.rs` – реэкспорт `get_v1_decisions`, `put_v1_decision_review`, `DecisionListQuery`, `DecisionReviewApiRequest`, `DecisionListResponse`.
- `backend/src/app/handlers/decisions/models.rs` – структуры `DecisionListQuery`, `DecisionReviewApiRequest`, `DecisionListResponse`.
- `backend/src/app/handlers/documents/mod.rs` – функции `get_document_processing`, `get_document_processing_jobs`, `post_document_processing_job_retry`; использование `api_audit_log`, `DocumentProcessingCommandService`, `RawQuery`; импорты доменных типов.
- `backend/src/app/handlers/events/handlers.rs` – функции `post_event`, `get_event`, `get_event_trace`, `get_event_trace_by_correlation`, `get_event_children`, `get_events`, `get_events_stream`, `get_events_websocket`, `get_audit_events`; структуры `EventStreamState`, `EventListQuery`, `EventStreamQuery`, `EventListResponse`; long-polling логика; SSE и WebSocket стриминг; санитизация payloads; константы валидации; вспомогательные функции `send_ws_json`, `event_list_response`, `heartbeat_event`, `stream_error_event`.
- `backend/src/app/handlers/events/mod.rs` – реэкспорт.
- `backend/src/app/handlers/graph/mod.rs` – функции `get_graph_summary`, `get_graph_nodes`, `get_graph_neighborhood`, `get_graph_search`; валидация параметров (`depth=1`, `node_id` обязателен, `q` непуст); лимиты 1..50.
- `backend/src/app/handlers/obligations/handlers.rs` – функции `get_v1_obligations`, `put_v1_obligation_review`; валидация комбинаций фильтров; константы `DEFAULT_OBLIGATION_LIMIT`, `MIN_OBLIGATION_LIMIT`, `MAX_OBLIGATION_LIMIT`; actor ID `OBLIGATION_API_ACTOR_ID`; использование `ObligationReviewApplicationService`, `ObligationStore`, `ApiAuditLog`.
- `backend/src/app/handlers/obligations/mod.rs` – реэкспорт.
- `backend/src/app/handlers/obligations/models.rs` – `ObligationListQuery`, `ObligationReviewApiRequest`, `ObligationListResponse`.
- `backend/src/app/handlers/organizations/directory.rs` – CRUD организаций: `get_organizations`, `post_organization`, `get_organization`, `put_organization`, `get_organization_search`, `post_organization_archive`; поиск по display_name/legal_name/website; лимит списка 200.
- `backend/src/app/handlers/organizations/core_records.rs` – подресурсы: identities, aliases, domains, departments, contacts, related orgs; создание через `add_identity_manual`, `add_alias_manual`, `add_department_manual`, `OrganizationContactLinkApplicationService`.
- `backend/src/app/handlers/organizations/enrichment.rs` – `get_org_enrichment`, `post_org_enrich_apply`.
- `backend/src/app/handlers/organizations/finance.rs` – `get_org_financial`, `get_org_contracts`, `get_org_compliance`, `get_org_services`, `get_org_products`.
- `backend/src/app/handlers/organizations/health.rs` – `get_org_risks`, `get_org_health`, `post_org_watchlist_toggle`.
- `backend/src/app/handlers/organizations/investigator.rs` – `get_org_dossier`, `get_org_brief`, `get_org_context_pack`.
- `backend/src/app/handlers/organizations/mod.rs` – объявление подмодулей.
- `backend/src/app/handlers/organizations/support.rs` – `database_pool`, `observation_store`.
- `backend/src/app/handlers/organizations/workflows.rs` – `get_org_timeline`, `get_org_portals`, `get_org_procedures`, `get_org_playbooks`, `get_org_templates`.
- `backend/src/app/handlers/persons/compatibility.rs` – роли: `get_person_roles`, `post_person_role`, `delete_person_role`; персоны: `get_person_personas`, `post_person_persona`, `delete_person_persona`.
- `backend/src/app/handlers/persons/errors.rs` – маппинг `EnrichmentEngineError`, `PersonExpertiseError`, `PersonTrustError`, `PersonHealthError`, `InvestigatorError`, `AnalyticsError`, `ExportError` в `ApiError`; логирование ошибок.
- `backend/src/app/handlers/persons/health.rs` – `get_person_health`, `get_persons_health`, `get_persons_watchlist`, `post_person_watchlist_toggle`.
- `backend/src/app/handlers/persons/history.rs` – `get_person_analytics`, `get_person_export_handler` (форматы экспорта, Content-Disposition), `get_person_snapshots`, `get_person_history_diff` (from/to RFC3339), `get_identity_candidates`, `put_identity_candidate_review`, `get_person_identity`.
- `backend/src/app/handlers/persons/identity.rs` – `get_person_identities`, `get_identity_traces` (только unattached), `post_identity_trace`, `put_identity_trace_assignment`, `post_person_identity`, `delete_person_identity`.
- `backend/src/app/handlers/persons/intelligence.rs` – enrichment: `get_person_enrichment`, `post_person_enrichment_apply`, `post_person_enrichment_reject`; expertise: `get_person_expertise`, `get_person_expertise_search`; promises: `get_person_promises`; risks: `get_person_risks`.
- `backend/src/app/handlers/persons/investigator.rs` – `post_person_investigate`, `get_person_dossier`, `put_person_dossier_review`, `get_person_meeting_prep`; вспомогательные функции `dossier_snapshot_response`, `dossier_snapshot_only_response`.

## Исходные файлы

- [`backend/src/app/handlers/decisions/mod.rs`](../../../../backend/src/app/handlers/decisions/mod.rs)
- [`backend/src/app/handlers/decisions/models.rs`](../../../../backend/src/app/handlers/decisions/models.rs)
- [`backend/src/app/handlers/documents/mod.rs`](../../../../backend/src/app/handlers/documents/mod.rs)
- [`backend/src/app/handlers/events/handlers.rs`](../../../../backend/src/app/handlers/events/handlers.rs)
- [`backend/src/app/handlers/events/mod.rs`](../../../../backend/src/app/handlers/events/mod.rs)
- [`backend/src/app/handlers/graph/mod.rs`](../../../../backend/src/app/handlers/graph/mod.rs)
- [`backend/src/app/handlers/obligations/handlers.rs`](../../../../backend/src/app/handlers/obligations/handlers.rs)
- [`backend/src/app/handlers/obligations/mod.rs`](../../../../backend/src/app/handlers/obligations/mod.rs)
- [`backend/src/app/handlers/obligations/models.rs`](../../../../backend/src/app/handlers/obligations/models.rs)
- [`backend/src/app/handlers/organizations/core_records.rs`](../../../../backend/src/app/handlers/organizations/core_records.rs)
- [`backend/src/app/handlers/organizations/directory.rs`](../../../../backend/src/app/handlers/organizations/directory.rs)
- [`backend/src/app/handlers/organizations/enrichment.rs`](../../../../backend/src/app/handlers/organizations/enrichment.rs)
- [`backend/src/app/handlers/organizations/finance.rs`](../../../../backend/src/app/handlers/organizations/finance.rs)
- [`backend/src/app/handlers/organizations/health.rs`](../../../../backend/src/app/handlers/organizations/health.rs)
- [`backend/src/app/handlers/organizations/investigator.rs`](../../../../backend/src/app/handlers/organizations/investigator.rs)
- [`backend/src/app/handlers/organizations/mod.rs`](../../../../backend/src/app/handlers/organizations/mod.rs)
- [`backend/src/app/handlers/organizations/support.rs`](../../../../backend/src/app/handlers/organizations/support.rs)
- [`backend/src/app/handlers/organizations/workflows.rs`](../../../../backend/src/app/handlers/organizations/workflows.rs)
- [`backend/src/app/handlers/persons/compatibility.rs`](../../../../backend/src/app/handlers/persons/compatibility.rs)
- [`backend/src/app/handlers/persons/errors.rs`](../../../../backend/src/app/handlers/persons/errors.rs)
- [`backend/src/app/handlers/persons/health.rs`](../../../../backend/src/app/handlers/persons/health.rs)
- [`backend/src/app/handlers/persons/history.rs`](../../../../backend/src/app/handlers/persons/history.rs)
- [`backend/src/app/handlers/persons/identity.rs`](../../../../backend/src/app/handlers/persons/identity.rs)
- [`backend/src/app/handlers/persons/intelligence.rs`](../../../../backend/src/app/handlers/persons/intelligence.rs)
- [`backend/src/app/handlers/persons/investigator.rs`](../../../../backend/src/app/handlers/persons/investigator.rs)

## Кандидаты на drift

Из предоставленного контекста расхождения кода, документации и ADR не видны.
