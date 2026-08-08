---
chunk_id: 043-source-backend-part-023
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 043-source-backend-part-023 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Обновить страницу `components/backend.md` в русской Obsidian‑wiki проекта `makosh‑hub`.
Страница должна документировать два домена, исходный код которых включён в этот чанк:

- **Communications** – хранилище блобов/вложений, детектирование подписок, шаблоны сообщений с рендерингом, трединг почтовых сообщений.
- **Decisions** – правило‑базированное извлечение решений из текстов, сохранение кандидатов, модели и идентификаторы.

Описание должно опираться только на встроенные исходники, сохраняя идентификаторы Rust‑кода (структуры, енамы, имена функций) точно так, как они приведены.

## Предложенные страницы

### `components/backend.md`

```markdown
# Компоненты бэкенда

Обзор доменов `communications` и `decisions` в кодовой базе Макошь.
Каждый компонент описан на основе исходных файлов, включённых в контекст.

---

## Домен «Коммуникации» (`domains/communications`)

### Хранилище блобов и вложений (`CommunicationStorageStore`)

**Источник:** `backend/src/domains/communications/storage/store.rs`

Хранилище взаимодействует с таблицами `communication_mail_blobs` и `communication_attachments`.

- `upsert_blob` – вставляет или обновляет запись в `communication_mail_blobs`.
  - Первичный идентификатор: `mail_blob_id(&blob.sha256)`.
  - При конфликте по колонке `sha256` (ON CONFLICT) обновляется только `content_type`, если он не `NULL`.
  - Возвращает `StoredCommunicationBlob`.

- `upsert_attachment` – вставляет или обновляет запись в `communication_attachments`.
  - Идентификатор: `mail_attachment_id(&attachment.message_id, &attachment.provider_attachment_id)`.
  - При конфликте по `(message_id, provider_attachment_id)` обновляются все поля, кроме `created_at`, а `updated_at` устанавливается в `now()`.
  - Возвращает `StoredCommunicationAttachment`.

- `attachments_for_message` – выбирает все вложения для заданного `message_id` с JOIN к `communication_mail_blobs`. Результат: `Vec<StoredCommunicationAttachmentWithBlob>`, отсортирован по `created_at ASC, attachment_id ASC`.

- `attachment_by_id` – ищет одно вложение по `attachment_id`. Возвращает `Option<StoredCommunicationAttachmentWithBlob>`.

Все публичные методы предварительно валидируют входные данные, используя функции из `validation.rs`.

**Валидация** (`backend/src/domains/communications/storage/validation.rs`):
- `validate_storage_kind` – допускается только значение, равное `LOCAL_FS_STORAGE_KIND` (константа из `constants.rs`).
- `validate_storage_path` – путь не должен быть абсолютным, содержать обратную косую черту (`\`) или компоненты, отличные от `Normal` (например, родительские ссылки).
- `validate_sha256` – строка должна начинаться с префикса `SHA256_PREFIX`, содержать ровно 64 hex-цифры, приводится к нижнему регистру.
- `validate_size_bytes` – значение должно быть ≥ 0.
- `validate_non_empty` – общая проверка, что строка после trim не пуста.

---

### Подписки (`SubscriptionStore`)

**Источник:** `backend/src/domains/communications/subscriptions.rs`

Хранилище определяет отправителей, чьи письма с большой вероятностью являются рассылками, на основе таблицы `communication_messages`.

- `detect_subscriptions` – возвращает `Vec<SubscriptionSource>`. Внутренне делегирует `detect_subscriptions_page` без курсора.
- `detect_subscriptions_page` – пагинированный вариант. Принимает необязательные `account_id`, `limit` (clamp 1..100) и `cursor`.

**Логика запроса:**
- Фильтр: `channel_kind = 'email'` AND `local_state = 'active'` (плюс опциональный `account_id`).
- Группировка по `sender` с `HAVING count(*) > 1`.
- Для каждого отправителя вычисляются:
  - `message_count` – общее количество сообщений.
  - `first_seen`, `last_seen` – минимальная/максимальная `occurred_at`.
  - `has_unsubscribe` – `bool_or` проверки на наличие в `body_text` (lower) слов `unsubscribe`, `opt out` или `manage preferences`.
  - `is_newsletter` – `bool_or` проверки на `newsletter` или `digest` в `subject` (lower) или `body_text` (lower).

**Пагинация:**
- Курсор представляет собой base64 (URL-safe, без padding) от JSON-объекта `{ message_count: i64, sender: String }`.
- Декодирует курсор функция `decode_subscription_cursor`, кодирует – `encode_subscription_cursor`.
- Запрос возвращает `limit + 1` записей, последняя используется как курсор, а флаг `has_more` выставляется, если записей больше лимита.
- Направление пагинации: `ORDER BY message_count DESC, sender ASC`.

---

### Шаблоны сообщений (`CommunicationTemplateStore`)

**Источник:** `backend/src/domains/communications/templates.rs` (файл обрезан, включено 12000 символов)

Хранилище управляет шаблонами с переменными в таблице `communication_templates`.

**CRUD-операции:**
- `upsert` – вставляет `NewCommunicationTemplate`. При конфликте по `template_id` обновляет все поля (кроме `created_at`), `updated_at` = `now()`.
- `list` – возвращает все шаблоны, отсортированные по `name`.
- `get` – возвращает один шаблон по `template_id`.
- `delete` – удаляет шаблон, возвращает `true`, если удалена хотя бы одна строка.

**Проверка шаблона:**
При создании/чтении для каждого шаблона вычисляются поля диагностики:
- `placeholder_variables` – все имена переменных, найденные в `subject_template` и `body_template` (рендеринг с пустым словарём).
- `undeclared_variables` – переменные, присутствующие в шаблоне, но не перечисленные в `variables`.
- `unused_variables` – переменные из `variables`, не используемые в шаблоне.
- `malformed_placeholders` – некорректные конструкции (например, открывающая `{{` без закрывающей `}}`).

**Рендеринг:**
- `render` – заменяет объявленные переменные на переданные значения. Возвращает `RenderedTemplate` с готовым `subject`, `body`, а также списками `missing_variables` (не переданы или пустые), `unresolved_variables` (отсутствуют в переданном словаре) и `malformed_placeholders`.
- `render_mail_merge_preview` – принимает вектор `CommunicationMergePreviewRow` (каждый – `row_id` + `HashMap<String, String>`). Для каждой строки вызывается `render`. Агрегация: `row_count`, `ready_count` (строки без диагностических проблем и с полным заполнением), `blocked_count = row_count - ready_count`. Возвращает `CommunicationMergePreview`.

Функция `render_template_text` (полный код обрезан) использует поиск шаблонных конструкций `{{` и `}}`, заменяя их значениями из словаря переменных, и собирает неразрешённые переменные и некорректные конструкции.

---

### Треды сообщений (`CommunicationThreadStore`)

**Источник:** `backend/src/domains/communications/threads.rs` (файл обрезан, включено 12000 символов)

Предоставляет группировку email-сообщений в треды по нормализованной теме.

**Нормализация темы** (`normalize_subject_for_thread`, публичная):
- Многократно удаляет префиксы `Re:`, `AW:`, `WG:`, `Fwd:`, `FW:` (без учёта регистра) и обрезает пробелы.

**Идентификатор треда** (`thread_id`, публичная):
- Конкатенация `account_id` и нормализованной темы (в нижнем регистре), затем хеширование через `DefaultHasher`. Результат – строка `thread:{:016x}`.

**Список тредов:**
- `list_threads` – возвращает `Vec<CommunicationThread>`.
- `list_threads_page` – пагинированный вариант. Принимает `account_id` (опционально), `cursor`, `limit` (clamp 1..100, по умолчанию 50).

Запрос использует CTE `grouped_threads`:
- `thread_id` – `COALESCE(m.conversation_id, md5(account_id || ':' || normalized_subject))`.
- Для каждого треда вычисляются:
  - `message_count`, `participant_count` (число уникальных отправителей)
  - `first_message_at`, `last_message_at`, `last_activity_at` (максимум из `occurred_at` и `projected_at`)
  - `has_open_action` – `true`, если есть сообщения с `workflow_state` IN `('needs_action', 'new')`
  - `has_attachments` – `true`, если существуют присоединённые записи в `communication_attachments`
  - `dominant_workflow_state` – мода `workflow_state` средствами PostgreSQL `mode() WITHIN GROUP`
- Пагинация по `last_activity_at DESC, thread_id ASC`. Курсор декодируется/кодируется функциями `decode_thread_list_cursor` / `encode_thread_list_cursor` (детали кодирования обрезаны, но логика аналогична подпискам).

**Сообщения треда** (`thread_messages`):
- Возвращает сообщения, относящиеся к нормализованной теме, с агрегированными вложениями.
- Вложения собираются через LEFT JOIN `communication_attachments` и `communication_mail_blobs` в JSON-массив (функции `jsonb_agg` / `jsonb_build_object`).
- Структура `ThreadMessage` включает поля: `message_id`, `provider_record_id`, `account_id`, `subject`, `sender`, `sender_display_name`, `body_text`, `occurred_at`, `projected_at`, `workflow_state`, `importance_score`, `ai_category`, `ai_summary`, `delivery_state`, `attachment_count`, `attachments` (вектор `ThreadMessageAttachment`).
- Сортировка: `COALESCE(occurred_at, projected_at) ASC`.

---

## Домен «Решения» (`domains/decisions`)

### Извлечение решений (`DecisionEngine`)

**Источник:** `backend/src/domains/decisions/extraction/engine.rs`, `detection.rs`, `models.rs`, `errors.rs`

Движок реализует детерминированное правило‑базированное извлечение решений из произвольного текста.

- `DecisionEngine::detect_candidates(input: &DecisionExtractionInput) -> Result<DecisionExtractionResult, DecisionEngineError>`
  1. Проверяет вход (непустые `source_id`, `text`, `impacted_entity_id`; если задан `decided_by_entity_kind`, то и `decided_by_entity_id`).
  2. Разбивает `text` на предложения функцией `sentences` (разделители: `\n`, `.`, `!`, `?`).
  3. Для каждого предложения вызывает `detect_decision`.

**Функция `detect_decision`:**
- Приводит предложение к нижнему регистру и ищет совпадение с префиксами:
  - `decision:` → `DecisionCandidateKind::ExplicitDecision`, уверенность `0.83`
  - `we decided to ` → `ExplicitDecision`, уверенность `0.78`
  - `approved:` → `Approval`, уверенность `0.74`
  - `confirmed:` → `Confirmation`, уверенность `0.72`
- Если совпадение найдено, оставшаяся часть (body) очищается от завершающих `.`, `!`, `?`.
- Из body извлекаются `title` и `rationale`:
  - Ищется маркер ` because ` или ` so that `.
  - Если маркер найден, title – часть до него, rationale – после.
  - Если маркер не найден, и title, и rationale равны полному тексту body.
- Кандидат создаётся только если title и rationale не короче 3 символов.
- `review_state` всегда устанавливается в `DecisionReviewState::Suggested`.
- Также заполняются `evidence_source_kind`, `evidence_source_id`, `evidence_observation_id` (из input`а) и `impacted_entities` с одним элементом, чьи `entity_kind` и `entity_id` берутся из input`а, а `impact_type` = `"decision_context"`.

**`DecisionCandidate::to_decision_draft`:**
- Создаёт `NewDecision` с метаданными `{ "engine": "decision", "candidate_kind": "<kind>" }`.
- Создаёт `NewDecisionEvidence` с той же цитатой и уверенностью.
- Создаёт вектор `NewDecisionImpactedEntity`.

Допустимые виды кандидатов (`DecisionCandidateKind`): `ExplicitDecision`, `Approval`, `Confirmation`.

---

### Обновление кандидатов (`candidate_refresh`)

**Источник:** `backend/src/domains/decisions/candidate_refresh.rs`, `constants.rs`

Расширение `DecisionStore` методами, запускающими процесс извлечения на реальных данных.

- `refresh_deterministic_candidates(limit: i64)` – обновляет кандидатов из наиболее свежих сообщений (`communication_messages`) и документов (`documents`).
  - `limit` ограничен диапазоном [1, 100] (константы `MIN_REFRESH_LIMIT` и `MAX_REFRESH_LIMIT` из `constants.rs`).
  - Для сообщений: `ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id LIMIT $limit`.
  - Для документов: `ORDER BY imported_at DESC, document_id LIMIT $limit`.

- `refresh_message_candidates_for_ids(message_ids: &[String])` – обновляет кандидатов только для указанных идентификаторов сообщений (пустой слайс возвращает 0).

- Внутренний метод `refresh_communication_decision_candidates` формирует `DecisionExtractionInput::communication` (с `impacted_entity_kind = DecisionEntityKind::Communication` и `impacted_entity_id = source_id`), вызывает `DecisionEngine::detect_candidates` и сохраняет результат.

- `persist_decision_extraction` – обходит все `DecisionCandidate`, для каждого:
  - Собирает `NewDecision`, `NewDecisionEvidence`, `Vec<NewDecisionImpactedEntity>` через `to_decision_draft`.
  - Вызывает `preserve_existing_review_state` (сохраняет текущее состояние ревью, если решение уже существует – детали не в этом контексте).
  - Вызывает `self.upsert_with_evidence(…)` для фактического сохранения в БД.

---

### Модели и перечисления

**Источник:** `backend/src/domains/decisions/models/*.rs`, `states.rs`, `entity_kind.rs`, `source_kind.rs`, `decision.rs`, `evidence.rs`, `impacted_entity.rs`

**`DecisionEntityKind`** – виды сущностей, на которые влияет решение, или которые его приняли:
`Persona`, `Organization`, `Project`, `Communication`, `Document`, `Task`, `Event`, `Decision`, `Obligation`, `Knowledge`.
Каждый вариант имеет строковое представление (`persona`, `organization`, …) через `as_str()`, и разбор из строки через `parse()` (при несовпадении – ошибка `UnknownEntityKind`).

**`DecisionEvidenceSourceKind`** – источник свидетельства:
`Observation`, `Communication`, `Document`, `Event`, `Memory`, `Knowledge`, `Decision`, `Obligation`, `Task`, `Relationship`, `Project`, `Organization`, `Persona`.
Аналогично имеет `as_str()`.

**`DecisionStatus`** – статус жизненного цикла:
`Active`, `Superseded`, `Reversed`, `Deprecated` (строки: `active`, `superseded`, `reversed`, `deprecated`). Разбирается в `row_mapping.rs`.

**`DecisionReviewState`** – состояние ревью:
`Suggested`, `UserConfirmed`, `UserRejected` (строки: `suggested`, `user_confirmed`, `user_rejected`). Разбирается через `parse()`.

**`NewDecision`** – конструктор для создания/обновления решения:
- Обязательные поля: `title`, `rationale`, `confidence` (f64), `review_state`.
- Builder-методы: `status`, `alternatives` (JSON массив), `decided_by(kind, id)`, `decided_at(DateTime<Utc>)`, `metadata` (JSON объект).
- Валидация: `title` и `rationale` непустые; `confidence` в диапазоне 0.0–1.0; `alternatives` – JSON массив; `metadata` – JSON объект; если задан `decided_by_entity_kind`, то и `decided_by_entity_id` непустой – иначе ошибка `PartialDecider`.

**`Decision`** – хранимая модель, дополнительно содержит `decision_id`, `created_at`, `updated_at`.

**`NewDecisionEvidence`** – свидетельство для решения:
- Источник: `source_kind` + `source_id` (обязательные).
- Может быть observation: `observation_id` должно совпадать с `source_id`, если `source_kind == Observation`, иначе ошибка `InvalidObservationEvidenceSource`.
- Может иметь `quote`, `confidence` (по умолчанию 1.0), `metadata`.

**`NewDecisionImpactedEntity`** – связь решения с затрагиваемой сущностью:
- `entity_kind` + `entity_id` + `impact_type` (по умолчанию `"related"`) + `metadata`.

---

### Идентификаторы решений

**Источник:** `backend/src/domains/decisions/ids.rs`

- `decision_id(decision: &NewDecision)` – детерминированная строка формата:
  ```
  decision:v1:{len_title}:{normalized_title}:{len_kind}:{kind}:{len_id}:{id}:{len_date}:{date}
  ```
  где title нормализуется (схлопываются множественные пробелы), а дата – `decided_at.to_rfc3339()`.

- `evidence_id(decision_id, source_kind, source_id)` – строка формата:
  ```
  decision:evidence:v1:{len_decision_id}:{decision_id}:{len_kind}:{kind}:{len_source_id}:{source_id}
  ```

---

### Связи с наблюдениями

**Источник:** `backend/src/domains/decisions/evidence.rs`

Вспомогательные функции для работы в транзакциях:

- `link_decision_support_in_transaction` – связывает наблюдение с решением как `"supports"`. Использует обобщённую функцию `link_domain_entity_in_transaction` из `platform::observations`, передавая `domain = "decisions"`, `entity_type = "decision"`.
- `link_decision_review_transition_in_transaction` – фиксирует переход состояния ревью для решения. Вызывает `materialize_review_transition_link_in_transaction` с типом связи `"review_state"`.

Обе функции работают через `ObservationStoreError`.

---

### Переиспользование порта

**Источник:** `backend/src/domains/decisions/ports.rs`

`DecisionStore` реэкспортируется как `DecisionReviewPort` для использования в других слоях приложения (например, для обновления состояния ревью).
```

---

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/domains/communications/storage/store.rs` | Методы `upsert_blob`, `upsert_attachment`, `attachments_for_message`, `attachment_by_id`; таблицы `communication_mail_blobs`, `communication_attachments`; стратегия ON CONFLICT; использование идентификаторов `mail_blob_id`, `mail_attachment_id`. |
| `backend/src/domains/communications/storage/validation.rs` | Правила валидации: `storage_kind` (только `LOCAL_FS_STORAGE_KIND`), `storage_path` (не абсолютный, без обратных слешей, только Normal-компоненты), `sha256` (префикс + 64 hex), `size_bytes` (≥ 0), `validate_non_empty`. |
| `backend/src/domains/communications/subscriptions.rs` | `SubscriptionStore` с методами `detect_subscriptions`, `detect_subscriptions_page`; логика определения `has_unsubscribe` и `is_newsletter`; пагинация через base64‑кодированный JSON‑курсор (`SubscriptionCursor`). |
| `backend/src/domains/communications/templates.rs` (truncated) | `CommunicationTemplateStore` с `upsert`, `list`, `get`, `delete`; вычисляемые поля диагностики (`placeholder_variables`, `undeclared_variables` и т.д.); рендеринг `render` и `render_mail_merge_preview`; поиск шаблонных конструкций `{{...}}`. |
| `backend/src/domains/communications/threads.rs` (truncated) | `CommunicationThreadStore` с `list_threads`, `list_threads_page`, `thread_messages`; нормализация темы `normalize_subject_for_thread`; идентификатор треда `thread_id`; группировка через CTE, поля `has_open_action`, `has_attachments`, `dominant_workflow_state`; структуры `CommunicationThread`, `ThreadMessage`. |
| `backend/src/domains/decisions/candidate_refresh.rs` | `refresh_deterministic_candidates`, `refresh_message_candidates_for_ids`; использование `DecisionExtractionInput::communication` / `::document`; сохранение через `persist_decision_extraction` и `preserve_existing_review_state`. |
| `backend/src/domains/decisions/constants.rs` | `MAX_REFRESH_LIMIT = 100`, `MIN_REFRESH_LIMIT = 1`. |
| `backend/src/domains/decisions/errors.rs` | Перечисление `DecisionStoreError` – все варианты ошибок (Sqlx, Observation, валидация, неизвестные enum-значения и т.д.). |
| `backend/src/domains/decisions/evidence.rs` | Функции `link_decision_support_in_transaction` и `link_decision_review_transition_in_transaction`. |
| `backend/src/domains/decisions/extraction.rs` | Публичный API extraction-модуля: `DecisionEngine`, `DecisionEngineError`, модели `DecisionCandidate`, `DecisionCandidateKind`, `DecisionExtractionInput`, `DecisionExtractionResult`, `DecisionImpactedEntityCandidate`. |
| `backend/src/domains/decisions/extraction/detection.rs` | `detect_decision` – префиксы, уверенности, разбор `because`/`so that`; `sentences`; `ensure_sentence_terminator`. |
| `backend/src/domains/decisions/extraction/engine.rs` | `DecisionEngine::detect_candidates` – разбиение на предложения, вызов `detect_decision`, агрегация в `DecisionExtractionResult`. |
| `backend/src/domains/decisions/extraction/errors.rs` | `DecisionEngineError` – `EmptyField` и `PartialDecider`. |
| `backend/src/domains/decisions/extraction/models.rs` | `DecisionExtractionInput` (конструкторы `communication`, `document`, `decided_by`, `with_observation_id`, `validate`); `DecisionExtractionResult`; `DecisionCandidateKind`; `DecisionCandidate` (поля и `to_decision_draft`); `DecisionImpactedEntityCandidate`. |
| `backend/src/domains/decisions/ids.rs` | `decision_id` (формат `decision:v1:…` с нормализацией title), `evidence_id`. |
| `backend/src/domains/decisions/mod.rs` | Публичный API домена, включая экспорт `DecisionStore` как `DecisionReviewPort`. |
| `backend/src/domains/decisions/models.rs` | Структура модуля моделей. |
| `backend/src/domains/decisions/models/decision.rs` | `NewDecision` (builder, валидация), `Decision` (хранимая модель). |
| `backend/src/domains/decisions/models/entity_kind.rs` | `DecisionEntityKind` – перечисление, `as_str()`, `parse()`. |
| `backend/src/domains/decisions/models/evidence.rs` | `NewDecisionEvidence` (конструкторы, валидация). |
| `backend/src/domains/decisions/models/impacted_entity.rs` | `NewDecisionImpactedEntity` (builder, валидация). |
| `backend/src/domains/decisions/models/source_kind.rs` | `DecisionEvidenceSourceKind` – перечисление, `as_str()`. |
| `backend/src/domains/decisions/models/states.rs` | `DecisionStatus` (варианты, `as_str()`), `DecisionReviewState` (варианты, `as_str()`, `parse()`). |
| `backend/src/domains/decisions/ports.rs` | Реэкспорт `DecisionStore` как `DecisionReviewPort`. |
| `backend/src/domains/decisions/row_mapping.rs` | `row_to_decision` – преобразование строки БД в `Decision`, разбор `status`, `review_state`, `decided_by_entity_kind`. |

## Исходные файлы

- [`backend/src/domains/communications/storage/store.rs`](../../../../backend/src/domains/communications/storage/store.rs)
- [`backend/src/domains/communications/storage/validation.rs`](../../../../backend/src/domains/communications/storage/validation.rs)
- [`backend/src/domains/communications/subscriptions.rs`](../../../../backend/src/domains/communications/subscriptions.rs)
- [`backend/src/domains/communications/templates.rs`](../../../../backend/src/domains/communications/templates.rs)
- [`backend/src/domains/communications/threads.rs`](../../../../backend/src/domains/communications/threads.rs)
- [`backend/src/domains/decisions/candidate_refresh.rs`](../../../../backend/src/domains/decisions/candidate_refresh.rs)
- [`backend/src/domains/decisions/constants.rs`](../../../../backend/src/domains/decisions/constants.rs)
- [`backend/src/domains/decisions/errors.rs`](../../../../backend/src/domains/decisions/errors.rs)
- [`backend/src/domains/decisions/evidence.rs`](../../../../backend/src/domains/decisions/evidence.rs)
- [`backend/src/domains/decisions/extraction.rs`](../../../../backend/src/domains/decisions/extraction.rs)
- [`backend/src/domains/decisions/extraction/detection.rs`](../../../../backend/src/domains/decisions/extraction/detection.rs)
- [`backend/src/domains/decisions/extraction/engine.rs`](../../../../backend/src/domains/decisions/extraction/engine.rs)
- [`backend/src/domains/decisions/extraction/errors.rs`](../../../../backend/src/domains/decisions/extraction/errors.rs)
- [`backend/src/domains/decisions/extraction/models.rs`](../../../../backend/src/domains/decisions/extraction/models.rs)
- [`backend/src/domains/decisions/ids.rs`](../../../../backend/src/domains/decisions/ids.rs)
- [`backend/src/domains/decisions/mod.rs`](../../../../backend/src/domains/decisions/mod.rs)
- [`backend/src/domains/decisions/models.rs`](../../../../backend/src/domains/decisions/models.rs)
- [`backend/src/domains/decisions/models/decision.rs`](../../../../backend/src/domains/decisions/models/decision.rs)
- [`backend/src/domains/decisions/models/entity_kind.rs`](../../../../backend/src/domains/decisions/models/entity_kind.rs)
- [`backend/src/domains/decisions/models/evidence.rs`](../../../../backend/src/domains/decisions/models/evidence.rs)
- [`backend/src/domains/decisions/models/impacted_entity.rs`](../../../../backend/src/domains/decisions/models/impacted_entity.rs)
- [`backend/src/domains/decisions/models/source_kind.rs`](../../../../backend/src/domains/decisions/models/source_kind.rs)
- [`backend/src/domains/decisions/models/states.rs`](../../../../backend/src/domains/decisions/models/states.rs)
- [`backend/src/domains/decisions/ports.rs`](../../../../backend/src/domains/decisions/ports.rs)
- [`backend/src/domains/decisions/row_mapping.rs`](../../../../backend/src/domains/decisions/row_mapping.rs)

## Кандидаты на drift

- Файлы `templates.rs` и `threads.rs` в контексте обрезаны (показано только 12000 символов из 19000 и 15363 соответственно). Поэтому документация описывает только ту часть, которая присутствует в чанке. Полная реализация функций `render_template_text`, а также кодирование/декодирование курсора списка тредов (`encode_thread_list_cursor`, `decode_thread_list_cursor`) остаются **не подтверждёнными данным контекстом**.
- В чанке не предоставлены другие файлы доменов `communications` и `decisions`, которые могут содержать дополнительную логику (например, полные реализации `store.rs`, сервисный слой, HTTP-ручки). Дрейф между текущим описанием и реальным кодом возможен, но из предоставленных исходников не виден.
- Никаких иных расхождений между кодом, документацией и ADR в данном контексте не обнаружено.
