---
chunk_id: 053-source-backend-part-033
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 053-source-backend-part-033 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/backend.md` должна быть обновлена описанием ключевых backend-компонентов Макошь на русском языке. В данном чанке представлены исходники доменов `signal_hub` и `tasks` (частично). Необходимо отразить актуальную структуру: обработку сырых сигналов, диспатчинг для Telegram/WhatsApp, логику политик сигналов, управление задачами (создание, обновление, статусы, анализ), автоматическое извлечение кандидатов задач из сообщений и документов, процесс ревью кандидатов, а также вспомогательные core-модули (чек-листы, контекстные пакеты, evidence, связи). Всё описание должно опираться только на встроенные исходные тексты, без домыслов.

## Предложенные страницы

- `components/backend.md`:

```markdown
# Компоненты бэкенда

## Обзор

Макошь backend реализован на Rust (async, tokio, sqlx) и разделён на домены.
В данном контексте рассмотрены домены `signal_hub` и `tasks`.

Используемые паттерны: CQRS/Event Sourcing (через `EventStore`), наблюдения (`ObservationStore`),
политики (`SignalPolicyEvaluator`), idempotent-диспатчинг событий.

## Signal Hub

### Обработка сырых сигналов

`SignalHubSignalService` (`backend/src/domains/signal_hub/service.rs`) реализует:

- `process_raw_signal` — принимает `EventEnvelope`, разбирает через `ParsedRawSignal`,
  разрешает `connection_id` через `SignalHubStore::resolve_connection_id_for_event`,
  оценивает активные политики (`SignalPolicyEvaluator::decide`) и в зависимости от решения
  выполняет одно из действий:
  - **Allow** — эмитирует `signal.accepted.<source>.<kind>`, возвращает `Accepted { event_id }`.
  - **Rejected** — эмитирует `signal.rejected.<source>.<kind>`, возвращает `Rejected { reason }`.
  - **Muted** — эмитирует `signal.muted.<source>.<kind>`, возвращает `Muted { reason }`.
  - **Paused** — сохраняет paused-событие через `SignalHubStore::record_paused_event`,
    эмитирует `signal.paused.<source>.<kind>`, возвращает `Paused { reason }`.

- `replay_raw_signal` — безусловно эмитирует `signal.accepted.<source>.<kind>` с решением «replayed».

Производные события формируются функцией `build_derived_event`: `event_id` = `<derived_type>_<raw_event_id>`,
provenance включает поле `signal_hub` с решением, `causation_id` ссылается на исходное событие,
`correlation_id` пробрасывается из исходного.

Разбор сырого события (`ParsedRawSignal::parse`) требует, чтобы `event_type` имел вид
`signal.raw.<source_code>.<event_kind>.observed` (минимум 5 сегментов). `source_code`
извлекается из поля `source.source_code` либо из третьего сегмента `event_type`.

### SignalHubStore

`SignalHubStore` (`backend/src/domains/signal_hub/store.rs`, файл обрезан) управляет
источниками сигналов и профилями политик:

- `restore_system_sources` — восстанавливает системные fixture-записи (`system_source_fixtures`,
  `system_profile_fixtures`): создаёт отсутствующие, обновляет расходящиеся. Результат фиксируется в
  `FixtureRestoreReport` с полями `sources_created`, `sources_repaired`, `profiles_created`,
  `profiles_repaired`.

Ошибки `SignalHubError` каталогизируют невалидные запросы, ненайденные сущности и нарушенные предусловия
через методы `is_invalid_request()`, `is_not_found()`, `is_failed_precondition()`.

### Диспатчинг Telegram и WhatsApp

- `dispatch_telegram_raw_signal` (`backend/src/domains/signal_hub/telegram.rs`):
  строит `signal.raw.telegram.message.observed` из `StoredRawCommunicationRecord`, сохраняет
  идемпотентно, проверяет разрешён ли диспатчинг через `signal_hub_raw_dispatcher_allows_processing`,
  затем обрабатывает. Возвращает принятое событие при успехе, иначе `None`.
  Event ID вычисляется как `evt_signal_raw_telegram_<sha256(raw_record_id)>`.

- `dispatch_whatsapp_raw_signal` (`backend/src/domains/signal_hub/whatsapp.rs`):
  аналогично, но тип события динамический: `signal.raw.whatsapp.{event_kind}.observed`,
  где `event_kind` определяется по `record_kind` (например, `whatsapp_web_reaction` → `reaction`,
  `whatsapp_web_message_update` → `message_update`, по умолчанию `message`).

Оба диспатчера используют `signal_hub_raw_dispatcher_allows_processing`, который
восстанавливает системные источники и вызывает `runtime_allows_processing`
с конфигурацией `{ "label": "Signal Hub raw signal dispatcher", "scope": "consumer" }`.

## Tasks (Задачи)

### Модель задачи

`Task` (`backend/src/domains/tasks/api.rs`, файл обрезан) содержит более 30 полей, включая:

- `task_id` (строка, префикс `task:v1:`), `task_candidate_id`
- `title`, `description`, `provenance_*`, `source_*`
- `makosh_status` — строка статуса по модели Макошь (new, triaged, ready, in_progress, waiting, blocked, review, done, cancelled, archived)
- `priority_score`, `risk_score`, `readiness_score`, `area`, `why`, `outcome`
- `due_at`, `completed_at`, `archived_at`, `waiting_reason`, `energy_type`
- `confidentiality`, `tags`, `task_metadata`
- связи: `linked_person_id`, `linked_organization_id`

### TaskStore (порт команд и запросов)

`TaskStore` реализует:

- `create`, `get`, `list`, `update`, `set_status` и их вариации с observation-линковкой
  (`create_in_transaction`, `update_with_observation`, `set_status_with_observation`).

- `create_in_transaction` генерирует `task_id` как `task:v1:{ts:x}` (таймстемп в наносекундах),
  вставляет запись в `tasks`, при source_kind `"observation"` материализует связь
  `task_observation_link` с типом `task_create`.

- `list` принимает `TaskListQuery`: фильтры `status`, `project_id`, `source_type`, лимит от 1 до 500,
  сортировка по `priority_score DESC, due_at ASC NULLS LAST, created_at DESC`.

### TaskCommandService

`TaskCommandService` (`backend/src/domains/tasks/command_service.rs`, файл обрезан) — сервис
для мутирующих операций над задачами с автоматическим захватом наблюдений (`ObservationStore`):

- `create_task_manual` — создаёт задачу с разрешённым provenance.
- `update_task_manual` — обновляет поля, записывая observation с типом `task_update`.
- `set_status_manual` — меняет статус с observation типа `status_update`.
- `archive_manual` — архивирует задачу.
- `analyze_runtime` — вычисляет оценки `priority_score`, `risk_score`, `readiness_score`
  через `TaskIntelligenceService`, записывает observation `task analysis`,
  обновляет задачу.
- `add_evidence` — добавляет evidence; если `source_type` не указан или `manual`,
  создаёт observation с данными evidence, иначе использует переданный source_id.
- `add_relation_manual` — создаёт связь с другой сущностью через observation.

### TaskBrainService (аналитика)

`TaskBrainService` (`backend/src/domains/tasks/brain.rs`):

- `explain_task` — агрегирует данные задачи: поля из БД, контекстный пакет (`TaskContextPackStore`),
  evidence (LIMIT 5), возвращает JSON с ключами `what`, `why`, `status`, `context` (summary, blockers,
  risks, next_action), `evidence`.
- `search_tasks` — поиск по `title ILIKE '%query%'` или `description ILIKE '%query%'`,
  ограничение 20 записей.
- `daily_brief` — возвращает количество активных задач, количество просроченных (due_at < now и статус
  не финальный), список до 5 задач с `risk_score > 0.7`.

### Task Candidates (кандидаты задач)

Исходники: `backend/src/domains/tasks/candidates/`.

#### Извлечение кандидатов

- **Детерминированное извлечение** из сообщений (`communication_messages`) и документов (`documents`):
  `refresh_deterministic_candidates` перебирает записи, строит текст из `subject + body_text`
  (для сообщений) или `title + extracted_text` (для документов), и для каждого:
  - Пытается извлечь фрагмент через `extract_candidate_fragment`. Если текст содержит
    ключевые слова `action:`, `please `, `follow up`, `next step` (регистронезависимо),
    создаётся `CandidatePayload` с `confidence` 0.8 (сообщения) или 0.7 (документы).
  - Если фрагмент не найден, вызывается `ObligationEngine::detect_candidates`; для каждого
    обнаруженного обязательства формируется `CandidatePayload` с `confidence` (confidence_obligation - 0.08),
    минимально 0.0.

- **Upsert**: `upsert_task_candidate` вставляет или обновляет запись в `task_candidates`
  с уникальностью `(source_kind, source_id, lower(title))`. Если кандидат уже в состоянии
  `user_confirmed` или `user_rejected`, review_state не перезаписывается.

#### Типы и константы

- `TaskCandidateSourceKind` — только `Observation`.
- `TaskCandidateKind` — `Task` или `ObligationTask`.
- `TaskCandidateReviewState` — `Suggested`, `UserConfirmed`, `UserRejected`.
- Идентификатор кандидата: `task_candidate:v1:{source_kind}:{source_id}:{fnv1a64_hex(title)}`.
- Идентификатор задачи из кандидата: `task:v1:{fnv1a64_hex(task_candidate_id)}`.

#### Review flow (процесс подтверждения)

- `TaskCandidateReviewService::review_manual` — захватывает observation с типом `REVIEW_TRANSITION`,
  затем вызывает `TaskCandidateStore::set_review_state_with_observation`.

- `set_review_state_with_observation` создаёт событие с префиксом `task_candidate_review:`,
  тип события `task_candidate.review_state_changed`, добавляет в `EventStore`, применяет
  новое состояние в транзакции (`apply_review_state_in_transaction`), материализует
  `review_transition`-линк с observation.

- `apply_review_state_in_transaction`:
  - **UserConfirmed**:
    - Создаёт/обновляет задачу через `upsert_task_in_transaction` (статус `active`, makosh_status `ready`).
    - Обновляет `task_candidates`: review_state, event_id, actor_id, reviewed_at.
    - Синхронизирует обязательства, если `candidate_kind == obligation_task`.
  - **Suggested / UserRejected**:
    - Синхронизирует обязательства.
    - Обновляет review_state кандидата.
    - Удаляет задачу (если существовала) по `task_candidate_id`.

- `sync_obligation_candidate_in_transaction` для `ObligationTask`:
  - Подтверждение: создаёт `Obligation` с состоянием `UserConfirmed`, линкует `fulfillment_task`
    к созданной задаче.
  - Отклонение/Suggested: устанавливает `review_state` связанных obligation в соответствующий
    статус.

#### Вспомогательные модули

- **Validation**: `validate_non_empty`, `validate_limit` (1..100), `validate_optional_limit` (default 50),
  `text_preview` (обрезание с многоточием).
- **Extraction**: `title_from_fragment` (первые TITLE_PREVIEW_CHARS символов),
  `evidence_excerpt` (первый REVIEW_TEXT_SNIPPET_CHARS).
- **Persistence**: `row_task_candidate` возвращает `StoredCandidateRow` с блокировкой `FOR UPDATE` в транзакции.

### Core-модули задач (`backend/src/domains/tasks/core.rs`)

- `TaskChecklist` и `TaskChecklistStore` — хранение чек-листов; последний чек-лист доступен через `get`,
  новая версия создаётся через `set` (с линковкой observation, если source начинается с `"observation:"`).

- `TaskContextPack` и `TaskContextPackStore` — контекстные пакеты задачи:
  поля `summary`, `open_questions`, `blockers`, `risks`, `suggested_next_action`,
  `source_summary`, `model`. Создаются через движок `ContextPackStore` с kind `Task`.

- Прочие модули (упомянуты в `core.rs`, реализации не включены в чанк):
  `TaskEvidenceStore`, `TaskRelationStore`, `TaskSubtaskStore`, `ExternalTaskIdentityStore`,
  `ObligationTaskLinkStore`, `TaskProviderStore`.

### Обработка ошибок

- `SignalHubError` — бизнес-ошибки signal_hub: невалидный event_type, отсутствующий source_code,
  проблемы scope/mode, ненайденные сущности, проблемы фикстур и профилей, пустые поля.
  Методы классификации: `is_invalid_request()`, `is_not_found()`, `is_failed_precondition()`.

- `TaskError` (частично видно в `api.rs`), `TaskCandidateError`, `TaskCoreError`,
  `TaskCommandServiceError` — доменные ошибки задач с автоматическим преобразованием из
  sqlx, serde_json, ObservationStoreError и др.
```

## Покрытие источников

| Исходный файл | Факты, покрытые в предложенной странице |
|---|---|
| `backend/src/domains/signal_hub/service.rs` | `process_raw_signal` и `replay_raw_signal`, разбор `ParsedRawSignal`, производные события `signal.accepted/rejected/muted/paused`, `SignalProcessingOutcome`, `build_derived_event`, `signal_decision_payload`, `source_code_from_value`, функция `signal_hub_raw_dispatcher_allows_processing`, константы `SIGNAL_HUB_RAW_SIGNAL_CONSUMER` и `SIGNAL_HUB_RAW_SIGNAL_RUNTIME_SOURCE`. |
| `backend/src/domains/signal_hub/store.rs` | `SignalHubStore::restore_system_sources` (создание/восстановление fixture source и profile), `SignalHubError` и методы `is_invalid_request`/`is_not_found`/`is_failed_precondition`. Файл обрезан, остальное не покрыто. |
| `backend/src/domains/signal_hub/telegram.rs` | `dispatch_telegram_raw_signal` (сборка сырого сигнала, идемпотентная запись, проверка runtime, обработка), `build_telegram_raw_signal` (event_type, source, subject, provenance, causation, correlation), ID-функции. |
| `backend/src/domains/signal_hub/whatsapp.rs` | `dispatch_whatsapp_raw_signal` и `build_whatsapp_raw_signal` (маппинг `record_kind` в event_kind, формирование события). |
| `backend/src/domains/tasks/api.rs` | `Task`-структура, `TaskStore` с `create_in_transaction`, `get`, `list`, `update_with_observation` (частично). Файл обрезан, не покрыты детали `set_status` и `archive`. |
| `backend/src/domains/tasks/brain.rs` | `TaskBrainService::explain_task`, `search_tasks`, `daily_brief` и соответствующие SQL-запросы, структура ответа. |
| `backend/src/domains/tasks/candidates/` (все файлы) | Модели `TaskCandidate`, `TaskCandidateKind`, `TaskCandidateReviewState`, `CandidatePayload`, `StoredCandidateRow`; извлечение `CandidateFragment` и `ObligationCandidate`; upsert-логика; review flow (создание события, применение состояния, obligation-синхронизация); активация задачи; сервис `TaskCandidateReviewService`; `TaskCandidateStore` с refresh/list/review; валидация лимитов, непустых полей, text_preview; константы. |
| `backend/src/domains/tasks/command_service.rs` | `TaskCommandService` с методами `create_task_manual`, `update_task_manual`, `set_status_manual`, `archive_manual` (названия упомянуты), `analyze_runtime`, `add_evidence`, `add_relation_manual`. Файл обрезан, полная реализация `archive` и `add_relation` не видна. |
| `backend/src/domains/tasks/core.rs` | Перечень core-модулей (чек-листы, контекстные паки, evidence, relations, subtasks, external identities, obligation links, provider accounts). |
| `backend/src/domains/tasks/core/checklists.rs` | `TaskChecklist`, `TaskChecklistStore` с методами `get` и `set` (с observation-линком). |
| `backend/src/domains/tasks/core/context_packs.rs` | `TaskContextPack`, `TaskContextPackStore`, обёртка над `ContextPackStore`, поля `summary`, `open_questions`, `blockers`, `risks`, `suggested_next_action`. |

## Исходные файлы

- [`backend/src/domains/signal_hub/service.rs`](../../../../backend/src/domains/signal_hub/service.rs)
- [`backend/src/domains/signal_hub/store.rs`](../../../../backend/src/domains/signal_hub/store.rs)
- [`backend/src/domains/signal_hub/telegram.rs`](../../../../backend/src/domains/signal_hub/telegram.rs)
- [`backend/src/domains/signal_hub/whatsapp.rs`](../../../../backend/src/domains/signal_hub/whatsapp.rs)
- [`backend/src/domains/tasks/api.rs`](../../../../backend/src/domains/tasks/api.rs)
- [`backend/src/domains/tasks/brain.rs`](../../../../backend/src/domains/tasks/brain.rs)
- [`backend/src/domains/tasks/candidates.rs`](../../../../backend/src/domains/tasks/candidates.rs)
- [`backend/src/domains/tasks/candidates/constants.rs`](../../../../backend/src/domains/tasks/candidates/constants.rs)
- [`backend/src/domains/tasks/candidates/errors.rs`](../../../../backend/src/domains/tasks/candidates/errors.rs)
- [`backend/src/domains/tasks/candidates/events.rs`](../../../../backend/src/domains/tasks/candidates/events.rs)
- [`backend/src/domains/tasks/candidates/extraction.rs`](../../../../backend/src/domains/tasks/candidates/extraction.rs)
- [`backend/src/domains/tasks/candidates/ids.rs`](../../../../backend/src/domains/tasks/candidates/ids.rs)
- [`backend/src/domains/tasks/candidates/models.rs`](../../../../backend/src/domains/tasks/candidates/models.rs)
- [`backend/src/domains/tasks/candidates/persistence.rs`](../../../../backend/src/domains/tasks/candidates/persistence.rs)
- [`backend/src/domains/tasks/candidates/service.rs`](../../../../backend/src/domains/tasks/candidates/service.rs)
- [`backend/src/domains/tasks/candidates/store.rs`](../../../../backend/src/domains/tasks/candidates/store.rs)
- [`backend/src/domains/tasks/candidates/store/list.rs`](../../../../backend/src/domains/tasks/candidates/store/list.rs)
- [`backend/src/domains/tasks/candidates/store/refresh.rs`](../../../../backend/src/domains/tasks/candidates/store/refresh.rs)
- [`backend/src/domains/tasks/candidates/store/review.rs`](../../../../backend/src/domains/tasks/candidates/store/review.rs)
- [`backend/src/domains/tasks/candidates/store/task_activation.rs`](../../../../backend/src/domains/tasks/candidates/store/task_activation.rs)
- [`backend/src/domains/tasks/candidates/validation.rs`](../../../../backend/src/domains/tasks/candidates/validation.rs)
- [`backend/src/domains/tasks/command_service.rs`](../../../../backend/src/domains/tasks/command_service.rs)
- [`backend/src/domains/tasks/core.rs`](../../../../backend/src/domains/tasks/core.rs)
- [`backend/src/domains/tasks/core/checklists.rs`](../../../../backend/src/domains/tasks/core/checklists.rs)
- [`backend/src/domains/tasks/core/context_packs.rs`](../../../../backend/src/domains/tasks/core/context_packs.rs)

## Кандидаты на drift

Из предоставленного контекста (только исходные файлы) невозможно сравнить код с документацией или ADR, так как соответствующие документы не встроены в чанк. Видимых расхождений между файлами чанка не обнаружено.
