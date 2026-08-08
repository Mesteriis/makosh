---
chunk_id: 049-source-backend-part-029
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 049-source-backend-part-029 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Данный чанк покрывает доменную логику персон (`persons`) в бэкенде `makosh`: подсистемы идентификации (identity), интеллекта (intelligence), расследователя (investigator) и памяти (memory). Целевая страница `components/backend.md` должна быть наполнена документацией этих компонентов на русском языке с Obsidian-совместимой разметкой. Текущий контекст не содержит предыдущего состояния страницы — пишем её содержимое на основе предоставленных исходников.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Backend

Бэкенд-компоненты сервиса `makosh`, реализованные в модуле `backend/src/domains/persons/`.

## Домен персон (`persons`)

### Идентификация (`identity`)

Подсистема управления кандидатами на идентификацию персон: слияние, разделение и привязка email-адресов.

#### Модели

##### Типы кандидатов (`PersonIdentityCandidateKind`)

- `MergePersons` — слияние двух персон (`"merge_persons"`)
- `AttachEmailAddress` — прикрепление email-адреса к персоне (`"attach_email_address"`)
- `SplitPerson` — разделение ранее слитых персон (`"split_person"`)

##### Состояния проверки (`PersonIdentityReviewState`)

- `Suggested` — кандидат предложен системой (`"suggested"`)
- `UserConfirmed` — пользователь подтвердил (`"user_confirmed"`)
- `UserRejected` — пользователь отклонил (`"user_rejected"`)

Метод `parse` парсит строковое значение в enum, возвращая `PersonIdentityError::InvalidReviewState` при несовпадении. Видимость: `pub(super)`.

##### Кандидат (`PersonIdentityCandidate`)

Структура, сериализуемая через `Serialize`, содержит поля:

- `identity_candidate_id: String`
- `candidate_kind: String`
- `left_person_id: String`
- `right_person_id: Option<String>`
- `email_address: Option<String>`
- `evidence_summary: String`
- `confidence: f64`
- `review_state: String`
- `generated_at: DateTime<Utc>`
- `reviewed_at: Option<DateTime<Utc>>`
- `updated_at: DateTime<Utc>`

##### Детализация идентификации (`PersonIdentityDetail`)

Контейнер с полем `items: Vec<PersonIdentityCandidate>`.

##### Команда проверки (`PersonIdentityReviewCommand`)

- `command_id: String`
- `identity_candidate_id: String`
- `review_state: PersonIdentityReviewState`
- `actor_id: String`

##### Результат проверки (`PersonIdentityReviewCommandResult`)

- `identity_candidate_id: String`
- `review_state: PersonIdentityReviewState`
- `event_id: String`

##### Полезная нагрузка кандидата (`PersonIdentityCandidatePayload`)

Внутренняя (`pub(crate)`) структура:

- `candidate_kind: PersonIdentityCandidateKind`
- `left_person_id: String`
- `right_person_id: Option<String>`
- `email_address: Option<String>`
- `evidence_summary: String`
- `confidence: f64`

Генерирует `identity_candidate_id` по правилам:

- Для `MergePersons`: `"{PERSON_IDENTITY_ID_PREFIX}merge_persons:{left}:{right}"`
- Для `AttachEmailAddress`: `"{PERSON_IDENTITY_ID_PREFIX}attach_email_address:{left}:{email_len}:{email}"`
- Для `SplitPerson`: `"{PERSON_IDENTITY_ID_PREFIX}split_person:{left}:{right}"`

Значение константы `PERSON_IDENTITY_ID_PREFIX` не подтверждено данным контекстом (определено в `constants.rs`, не включённом в чанк).

##### Преобразование из БД (`row_to_person_identity_candidate`)

Функция в `rows.rs` маппит строку `PgRow` в `PersonIdentityCandidate`, извлекая все 11 полей через `row.try_get`.

#### Хранилище (`PersonIdentityStore`)

Структура с единственным полем `pool: PgPool`. Методы распределены по подмодулям:

- `candidates` — refresh-операции
- `name_merge_candidates` — поиск кандидатов на слияние по имени
- `split_candidates` — генерация кандидатов на разделение
- `queries` — запросы списков
- `review` — управление состояниями проверки
- `review_state` — применение состояний в транзакции

##### `refresh_candidates(limit: i64) -> Result<usize, PersonIdentityError>`

Обновляет кандидатов на слияние по имени и на разделение. Возвращает суммарное количество созданных/обновлённых кандидатов. Лимит валидируется через `validate_limit`.

##### `suggest_attach_email_candidates`

Ищет персоны по нормализованному `display_name` (trim + lowercase) без символа `@` в имени, у которых ещё нет активной email-идентичности с заданным адресом. Для каждой найденной персоны создаёт кандидат `AttachEmailAddress`. Возвращает количество созданных кандидатов.

Параметры: `display_name`, `email_address`, `evidence_summary`, `confidence`, `limit`. Если имя или email пусты, или email не содержит `@`, возвращает `0`.

Использует таблицы `persons` и `person_identities`. Проверяет `identity_type = 'email'` и `status = 'active'`.

##### `list_candidates(limit: Option<i64>) -> Result<Vec<PersonIdentityCandidate>, ...>`

Возвращает список кандидатов из таблицы `person_identity_candidates`, сортировка: `updated_at DESC, identity_candidate_id`. Лимит по умолчанию (`DEFAULT_LIMIT`) и границы (`MIN_LIMIT`, `MAX_LIMIT`) определены в модуле `constants`, не включённом в данный чанк.

##### `person_identity(person_id: &str) -> Result<PersonIdentityDetail, ...>`

Возвращает подтверждённые (`review_state = 'user_confirmed'`) слияния для персоны, исключая те, для которых существует подтверждённый кандидат на разделение. Проверяет пары `(left_person_id, right_person_id)` через `LEAST`/`GREATEST` для симметричного сравнения.

##### `set_review_state(command) -> Result<PersonIdentityReviewCommandResult, ...>`

Основной метод изменения состояния проверки кандидата. Делегирует в `set_review_state_with_observation` с `observation_id = None` и `metadata = None`.

##### `set_review_state_with_observation(command, observation_id, metadata) -> Result<...>`

Выполняет в транзакции:

1. Проверяет существование кандидата через `ensure_candidate_exists`
2. Формирует `event_id = "{PERSON_IDENTITY_REVIEW_PREFIX}{command_id}"` (константа `PERSON_IDENTITY_REVIEW_PREFIX` не подтверждена данным контекстом)
3. Создаёт событие `ReviewCommandEvent` и добавляет в `EventStore` в транзакции
4. Применяет состояние проверки к строке кандидата (`apply_review_state_in_transaction`)
5. Материализует кандидата на разделение при подтверждении слияния (`materialize_split_candidate_for_confirmed_merge_in_transaction`)
6. Создаёт связь перехода проверки с наблюдением через `materialize_review_transition_link_in_transaction` (параметры: `"persons"`, `"identity_candidate"`, `review_state.as_str()`)

Метаданные наблюдения включают `event_id` и, опционально, поле `context`.

##### `apply_review_event(event: &EventEnvelope) -> Result<(), ...>`

Применяет входящее событие проверки. Проверяет `event.event_type == PERSON_IDENTITY_REVIEW_EVENT_TYPE` (константа не подтверждена данным чанком). Парсит `ReviewEvent` из payload. Извлекает `actor_id` из `event.actor["actor_id"]`. Выполняет транзакцию: проверка существования кандидата, применение состояния, материализация кандидата на разделение.

##### `refresh_name_merge_candidates(pool, limit) -> Result<usize, ...>`

Ищет пары персон в таблице `persons`, у которых нормализованные имена совпадают (`lower(trim(display_name))`), исключая имена, содержащие `@`. Для каждой пары создаёт кандидат `MergePersons` с `confidence = 0.72` и `evidence_summary` вида `"Same normalized display name: {name}"`.

##### `refresh_split_candidates(pool, limit) -> Result<usize, ...>`

Ищет подтверждённые (`review_state = 'user_confirmed'`) слияния, для которых ещё нет кандидата на разделение. Для каждой такой пары создаёт кандидат `SplitPerson` с `confidence = 1.0` и `evidence_summary` вида `"Previously confirmed merge can be split: {left} and {right}"`.

##### `materialize_split_candidate_for_confirmed_merge_in_transaction`

Вызывается при подтверждении слияния. Если `review_state == UserConfirmed` и `candidate_kind == "merge_persons"`, создаёт соответствующий кандидат на разделение через `upsert_candidate_in_transaction`. Пропускает, если `right_person_id` отсутствует.

#### Upsert-логика (`upsert.rs`)

##### `upsert_candidate(pool, payload, identity_candidate_id, review_state)`

Создаёт транзакцию и делегирует в `upsert_candidate_in_transaction`.

##### `upsert_candidate_in_transaction(transaction, payload, identity_candidate_id, review_state)`

UPSERT в таблицу `person_identity_candidates`:

- При вставке: устанавливает все поля, `event_id = NULL`, `actor_id = NULL`, `reviewed_at = NULL`
- При конфликте по `identity_candidate_id`: обновляет поля, НО если текущий `review_state` уже `user_confirmed` или `user_rejected`, сохраняет текущие значения `review_state`, `event_id`, `actor_id`, `reviewed_at` (защита от перезаписи пользовательских решений)
- Всегда обновляет `updated_at = now()`
- После upsert вызывает `append_candidate_detected_event`

##### `append_candidate_detected_event`

Создаёт событие типа `"person_identity.candidate.detected"` с уникальным `event_instance_id` (UUID v7). Игнорирует ошибки уникальности (duplicate event). Публикуется через `EventStore::append_in_transaction`.

##### `load_identity_candidate_payload(transaction, identity_candidate_id)`

Загружает `PersonIdentityCandidatePayload` из таблицы по ID. Возвращает `IdentityCandidateNotFound`, если запись отсутствует. Парсит `candidate_kind` через `parse_person_identity_candidate_kind`.

##### Вспомогательные парсеры

- `parse_person_identity_candidate_kind(value: &str)` — парсит строку в `PersonIdentityCandidateKind`, возвращает `InvalidCandidateKind` при ошибке
- `parse_person_identity_review_state(value: &str)` — парсит строку в `PersonIdentityReviewState`, возвращает `InvalidReviewState` при ошибке

Оба парсера имеют видимость `pub(crate)`.

#### Валидация (`validation.rs`)

- `validate_limit(limit: i64)` — проверяет, что лимит в диапазоне `MIN_LIMIT..=MAX_LIMIT` (константы не подтверждены), иначе `InvalidLimit`
- `validate_optional_limit(limit: Option<i64>)` — применяет `DEFAULT_LIMIT` для `None`
- `validate_non_empty(field, value)` — обрезает пробелы, возвращает `EmptyField`, если пусто
- `required_payload_string(payload, field)` — извлекает строку из JSON-объекта, валидирует непустоту
- `as_object(value)` — проверяет, что JSON-значение является объектом

---

### Интеллект (`intelligence`)

Сервис анализа коммуникационных паттернов персон — эвристический и через LLM.

#### Модели

##### `CommunicationFingerprint`

Характеристики коммуникации персоны:

- `avg_message_length: Option<usize>` — средняя длина сообщения
- `avg_response_hours: Option<f64>` — среднее время ответа в часах (всегда `None` в эвристическом методе)
- `frequent_topics: Vec<String>` — частые темы
- `typical_tone: Option<String>` — типичный тон
- `detected_language: Option<String>` — определённый язык
- `writing_style: Option<String>` — стиль письма (`"verbose"`, `"concise"`, `"balanced"`)
- `preferred_time_of_day: Option<String>` — предпочитаемое время суток (всегда `None` в эвристическом методе)
- `trust_score: Option<i16>` — оценка доверия (0–100)

##### `PersonInsight`

Результат анализа:

- `person_id: String`
- `fingerprint: CommunicationFingerprint`
- `suggested_actions: Vec<String>`

##### `PersonMessage`

Входное сообщение:

- `subject: String`
- `body_text: String`
- `occurred_at: Option<DateTime<Utc>>`

#### Сервис `PersonIntelligenceService`

Хранит опциональный `runtime: Option<SharedAiRuntimePort>` для LLM-анализа.

##### `heuristic_fingerprint(messages: &[PersonMessage]) -> CommunicationFingerprint`

Эвристический анализ без LLM. Для пустого списка сообщений возвращает все поля `None`.

**Определение тем**: проверяет комбинированный текст (все `body_text`, объединённые в lowercase) на наличие ключевых слов:

- `"finance"`: `"invoice"`, `"payment"`, `"amount"`, `"tax"`
- `"legal"`: `"contract"`, `"nda"`, `"agreement"`, `"legal"`
- `"project"`: `"project"`, `"deadline"`, `"milestone"`, `"deliverable"`
- `"support"`: `"help"`, `"issue"`, `"problem"`, `"bug"`

**Определение тона** (по порядку приоритета):

- Содержит `"urgent"` или `"asap"` → `"urgent"`
- Содержит `"thanks"` или `"appreciate"` → `"friendly"`
- Содержит `"please"` и `"would"` → `"polite"`
- Иначе → `"neutral"`

**Определение языка** (функция `detect_language`):

- Символы кириллицы (U+0400–U+04FF): если есть `ї` или `є` → `"uk"`, иначе `"ru"`
- CJK иероглифы (U+4E00–U+9FFF) → `"zh"`
- Латинские ключевые слова (`"hola"`, `"gracias"`, `"para"`, `"como"`, `"que"`, `"por favor"`, `"saludos"`, `"adjunto"`) → `"es"`
- Транслит русских (`"privet"`, `"spasibo"`, `"pozhaluysta"`) → `"ru"`
- Немецкие (`"mit"`, `"und"`, `"der"`, `"die"`, `"das"`, `"ist"`, `"von"`, `"für"`, `"danke"`, `"bitte"`) → `"de"`
- ASCII-буквы → `"en"`
- Иначе → `"unknown"`

**Стиль письма**: по средней длине сообщения (`total_len / messages.len()`):

- `> 500` → `"verbose"`
- `< 100` → `"concise"`
- Иначе → `"balanced"`

**Оценка доверия** (`trust_score`): базовое значение 50, плюс `min(кол-во_сообщений * 2, 30)` (не более 30), плюс 10 при наличии тем. Зажимается в диапазон 0–100 через `clamp`.

##### `llm_fingerprint(messages: &[PersonMessage]) -> Result<Option<CommunicationFingerprint>, ...>`

Асинхронный LLM-анализ. Требует наличия `runtime` (возвращает `None`, если runtime отсутствует). Берёт до 5 сообщений, формирует промпт с просьбой вернуть JSON с полями: `frequent_topics`, `typical_tone`, `detected_language`, `writing_style`, `preferred_time_of_day`. Результат парсится из ответа LLM с очисткой маркеров ` ```json ` и ` ``` `. При ошибке парсинга возвращает `Ok(None)` (не паникует).

##### `suggested_actions(fingerprint: &CommunicationFingerprint) -> Vec<String>`

Генерирует рекомендации:

- Если тон определён — `"Person tends to be {tone} — match tone in replies"`
- Если язык не `"en"` — `"Person writes in {lang} — consider translating replies"`
- Стиль письма — `"Person style: {style}"`
- Если `trust_score < 30` — `"Low trust score — verify claims"`

#### Ошибки (`PersonIntelligenceError`)

- `Runtime(AiRuntimePortError)` — ошибка LLM-рантайма
- `Serde(serde_json::Error)` — ошибка десериализации

---

### Расследователь (`investigator`)

Сервис сборки досье на персону, подготовки к встречам и управления снапшотами досье.

#### Модели

##### `DossierSectionItem`

Элемент раздела досье:

- `label: String`
- `value: String`
- `source_refs: Vec<String>` — ссылки на источники
- `confidence: Option<f64>`

##### `PersonDossier`

Полное досье на персону:

- `person: EnrichedPerson`
- `facts: Vec<PersonFact>`
- `memory_cards: Vec<PersonMemoryCard>`
- `timeline: Vec<RelationshipEvent>`
- `identities: Vec<Value>`, `expertise: Vec<Value>`, `promises: Vec<Value>`, `risks: Vec<Value>` (пустые векторы при сборке)
- `summary: String` — сводка (формируется из `tone`, `language`, `interaction_count`, `frequent_topics` и карточек с `importance >= 7`)
- Секции: `interests`, `projects`, `organizations`, `skills`, `communication_patterns`, `ai_observations` (все `Vec<DossierSectionItem>`)
- `source_refs: Vec<String>` — все уникальные источники данных (сортированные, дедуплицированные через `BTreeSet`)
- `generated_at: DateTime<Utc>`

##### `DossierReviewState`

Состояния проверки досье (аналогично `PersonIdentityReviewState`):

- `Suggested` (`"suggested"`)
- `UserConfirmed` (`"user_confirmed"`)
- `UserRejected` (`"user_rejected"`)

Метод `parse` имеет видимость `pub`, возвращает `InvestigatorError::InvalidDossierReviewState` при несовпадении.

##### `DossierSnapshot`

Снапшот досье в БД:

- `dossier_snapshot_id: String` — идентификатор вида `"persona_dossier:v1:{person_id}"`
- `persona_id: String`
- `dossier: Value` — JSON-представление досье
- `source_refs: Value`
- `review_state: DossierReviewState`
- `reviewed_by: Option<String>` — при review всегда `"owner_persona"`
- `reviewed_at: Option<DateTime<Utc>>`
- `metadata: Value`
- `generated_at: DateTime<Utc>`
- `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`

##### `MeetingPrep`

Подготовка к встрече:

- `person_id: String`
- `display_name: String`
- `last_interaction_days: Option<i64>` — дней с последнего взаимодействия (`(now - last_interaction_at).num_days()`)
- `open_promises: i64` — количество открытых обещаний (status = `"pending"`)
- `open_risks: i64` — количество неразрешённых рисков (`resolved_at IS NULL`)
- `recent_topics: Vec<String>` — темы из `frequent_topics` персоны
- `communication_tips: Vec<String>` — советы: `"Discuss topic: {topic}"`, `"Match tone: {tone}"`, `"Style: {style}"`
- `shared_projects: Vec<String>` — проекты из `linked_projects` персоны

#### Сборка досье (`assembly.rs`)

Функция `assemble_dossier(pool, person_id)` собирает досье из шести источников:

1. `PersonEnrichmentStore` — обогащённая персона
2. `PersonFactStore` — факты о персоне
3. `PersonMemoryCardStore` — карточки памяти
4. `PersonPreferenceStore` — предпочтения
5. `RelationshipEventStore` — таймлайн (лимит 50 событий)
6. `PersonExpertiseStore` — экспертиза

Ошибки из `list`, `timeline` обрабатываются через `unwrap_or_default()` — сборка не прерывается при ошибках подсистем.

Секции формируются функциями из `sections.rs`:

- `fact_section` — фильтрует активные (`is_active == true`) факты по `fact_type`:
  - `"interest"` → секция `interests`
  - `"project"` → секция `projects`
  - `"organization"` → секция `organizations`
- `expertise_section` — преобразует записи экспертизы, используя `domain` как `label` (fallback: `"skill"`)
- `communication_pattern_section` — язык, тон, стиль письма из `EnrichedPerson`, а также предпочтения с префиксами `"communication:"` и `"interaction_context:"`
- `ai_observation_section` — карточки, где `source` содержит `"ai"` или `title` в lowercase содержит `"ai"`
- `dossier_source_refs` — собирает уникальные ссылки на источники из всех данных (`BTreeSet` для дедупликации и сортировки, пропускает пустые строки)

#### Сервис `PersonInvestigator`

Методы:

- `assemble_dossier(person_id)` — сборка досье без кэширования
- `assemble_and_cache_dossier(person_id)` — сборка + сохранение снапшота через `cache_dossier_snapshot`
- `assemble_cache_and_record_refresh(person_id, operation, captured_by, endpoint, source_ref)` — создаёт наблюдение (тип `"PERSON_MUTATION"`, `ObservationOriginKind::Manual`) через `ObservationStore::capture`, затем собирает и кэширует досье, привязывая снапшот к наблюдению через `link_persons_entity` с тегом `"dossier_refresh"`
- `cache_dossier_snapshot(dossier)` — сохраняет снапшот в таблицу `persona_dossier_snapshots`
- `review_dossier_snapshot(person_id, review_state)` — устанавливает состояние проверки
- `review_dossier_snapshot_with_observation(person_id, review_state, observation_id, metadata)` — то же с материализацией связи review-перехода через `materialize_review_transition_link` (параметры: `"persons"`, `"dossier_snapshot"`, `review_state.as_str()`)
- `meeting_prep(person_id)` — подготовка к встрече

#### Снапшоты (`snapshots.rs`)

**`cache_dossier_snapshot`**: UPSERT в `persona_dossier_snapshots` по ключу `persona_id`:

- При вставке: устанавливает `review_state = 'suggested'`
- При конфликте: обновляет `dossier`, `source_refs`, `generated_at`, `updated_at` (не меняет `review_state`)

**`review_dossier_snapshot`**: UPDATE снапшота по `persona_id`, устанавливает `review_state`, `reviewed_by = 'owner_persona'`, `reviewed_at = now()`. Возвращает `DossierSnapshotNotFound`, если персона не найдена.

---

### Память (`memory`)

Подсистема управления фактами, карточками памяти и предпочтениями персон. Экспортирует также `FieldChange`, `HistoryDiff`, `PersonSnapshot`, `PersonSnapshotStore` из подмодуля `snapshots` (содержимое не включено в данный чанк). `RelationshipEventStore` экспортируется под двумя именами: `RelationshipEventStore` и `RelationshipEventPort`.

#### Факты (`PersonFact`)

Структура:

- `id: String`, `person_id: String`
- `fact_type: String`, `value: String`
- `source: String`, `confidence: f64`
- `last_verified_at: Option<DateTime<Utc>>`
- `valid_from: Option<DateTime<Utc>>`, `valid_to: Option<DateTime<Utc>>`
- `is_active: bool`
- `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`

**`PersonFactStore`**:

- `list(person_id)` — все факты, сортировка `created_at DESC`
- `upsert(person_id, fact_type, value, source, confidence)` — создаёт факт через `MemoryEngine::persona_fact_memory`, вставляет в таблицу `person_facts` с `ON CONFLICT DO NOTHING`
- `upsert_with_observation(person_id, fact_type, value, source, confidence, observation_id)` — upsert + привязка к наблюдению через `link_persons_entity`
- `update_confidence(id, confidence)` — обновляет confidence и `last_verified_at`, `updated_at`
- `decay_unverified(threshold_days)` — умножает `confidence` на 0.5 для фактов, не проверенных указанное количество дней, либо с `last_verified_at IS NULL`. Возвращает количество затронутых строк (`rows_affected()`)

#### Карточки памяти (`PersonMemoryCard`)

Структура:

- `id: String`, `person_id: String`
- `title: String`, `description: String`
- `source: String`, `confidence: f64`
- `importance: i16`
- `created_at: DateTime<Utc>`, `last_verified_at: Option<DateTime<Utc>>`

**`PersonMemoryCardStore`**:

- `list(person_id)` — сортировка `importance DESC, created_at DESC`
- `upsert(person_id, title, description, source, importance)` — `ON CONFLICT DO NOTHING`
- `upsert_with_observation(person_id, title, description, source, importance, observation_id)` — upsert + привязка

#### Предпочтения (`PersonPreference`)

Структура:

- `id: String`, `person_id: String`
- `preference_type: String`, `value: String`
- `source: String`, `confidence: f64`
- `last_verified_at: Option<DateTime<Utc>>`
- `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`

**`PersonPreferenceStore`**:

- `list(person_id)` — сортировка `preference_type`
- `upsert(person_id, preference_type, value, source)` — `ON CONFLICT (person_id, preference_type) DO UPDATE SET value, source, updated_at` (в отличие от других сущностей — перезаписывает существующие)
- `upsert_with_observation(person_id, preference_type, value, source, observation_id)` — upsert + привязка

#### Ошибки памяти (`PersonMemoryError`)

- `Sqlx(sqlx::Error)` — ошибка БД
- `Memory(MemoryEngineError)` — ошибка движка памяти
- `Timeline(TimelineEngineError)` — ошибка таймлайна
- `ObservationStore(ObservationStoreError)` — ошибка хранилища наблюдений
- `NotFound` — сущность не найдена
```

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `identity/models.rs` | Enum `PersonIdentityCandidateKind` (3 варианта + `as_str`), enum `PersonIdentityReviewState` (3 варианта + `as_str` + `parse`), структуры `PersonIdentityReviewCommand`, `PersonIdentityReviewCommandResult`, `PersonIdentityCandidate` (11 полей), `PersonIdentityDetail`, `PersonIdentityCandidatePayload` (6 полей + логика генерации `identity_candidate_id`) |
| `identity/rows.rs` | Функция `row_to_person_identity_candidate` — маппинг 11 полей из `PgRow` |
| `identity/store.rs` | Структура `PersonIdentityStore` с `pool: PgPool`, модульная декомпозиция |
| `identity/store/candidates.rs` | Методы `refresh_candidates` (merge + split), `suggest_attach_email_candidates` (поиск по `display_name` + email, таблицы `persons`, `person_identities`) |
| `identity/store/name_merge_candidates.rs` | Функция `refresh_name_merge_candidates` — поиск пар по `lower(trim(display_name))`, исключение `@`, `confidence = 0.72` |
| `identity/store/queries.rs` | Методы `list_candidates` (сортировка, лимит), `person_identity` (подтверждённые слияния, исключение split, `LEAST`/`GREATEST`) |
| `identity/store/review.rs` | Методы `set_review_state`, `set_review_state_with_observation` (6 шагов в транзакции), `apply_review_event` (парсинг `actor_id` из `event.actor`) |
| `identity/store/review_state.rs` | Функции `apply_review_state_in_transaction` (UPDATE с вариантами для `Suggested` vs `UserConfirmed`/`UserRejected`), `ensure_candidate_exists` |
| `identity/store/split_candidates.rs` | Функции `refresh_split_candidates` (поиск confirmed merge без split, `confidence = 1.0`), `materialize_split_candidate_for_confirmed_merge_in_transaction` |
| `identity/upsert.rs` | Функции `upsert_candidate`, `upsert_candidate_in_transaction` (UPSERT с защитой user_confirmed/user_rejected), `append_candidate_detected_event` (UUID v7, игнорирование дубликатов), `load_identity_candidate_payload`, парсеры `parse_person_identity_candidate_kind`, `parse_person_identity_review_state` |
| `identity/validation.rs` | Функции `as_object`, `required_payload_string`, `validate_non_empty`, `validate_limit`, `validate_optional_limit` |
| `intelligence.rs` | Структуры `CommunicationFingerprint` (8 полей), `PersonInsight`, `PersonMessage`; сервис `PersonIntelligenceService` с методами `heuristic_fingerprint` (ключевые слова тем, тона, `detect_language`, расчёт trust_score), `llm_fingerprint` (промпт, парсинг JSON), `suggested_actions`; функция `detect_language` (кириллица, CJK, ключевые слова es/de/ru); enum `PersonIntelligenceError`; тесты `fingerprint_detects_topics`, `fingerprint_sets_trust`, `fingerprint_detects_tone`, `empty_messages_returns_none` |
| `investigator.rs` | Публичный интерфейс модуля: реэкспорт ошибок, моделей, сервиса |
| `investigator/assembly.rs` | Функция `assemble_dossier` — сборка из 6 источников, формирование `summary`, вызов секционных билдеров |
| `investigator/errors.rs` | Enum `InvestigatorError` (9 вариантов), реализации `From<PersonEnrichmentError>`, `From<PersonMemoryError>` |
| `investigator/meeting_prep.rs` | Функция `meeting_prep` — расчёт `last_interaction_days`, подсчёт `open_promises` (status=`"pending"`), `open_risks` (`resolved_at IS NULL`), формирование `communication_tips` |
| `investigator/models.rs` | Структуры `DossierSectionItem`, `PersonDossier` (17 полей), enum `DossierReviewState` (3 варианта + `as_str` + `parse`), `DossierSnapshot` (12 полей), `MeetingPrep` (8 полей) |
| `investigator/sections.rs` | Функции `fact_section` (фильтр `is_active`, `fact_type`), `expertise_section` (fallback label `"skill"`), `communication_pattern_section` (поля EnrichedPerson + предпочтения с префиксами), `ai_observation_section` (фильтр по `"ai"`), `dossier_source_refs` (BTreeSet, пропуск пустых) |
| `investigator/service.rs` | Сервис `PersonInvestigator` с методами `assemble_dossier`, `assemble_and_cache_dossier`, `assemble_cache_and_record_refresh` (наблюдение + привязка), `cache_dossier_snapshot`, `review_dossier_snapshot`, `review_dossier_snapshot_with_observation`, `meeting_prep` |
| `investigator/snapshots.rs` | Функции `cache_dossier_snapshot` (UPSERT по `persona_id`, `review_state = 'suggested'`), `review_dossier_snapshot` (UPDATE, `reviewed_by = 'owner_persona'`), `dossier_snapshot_id` (формат), `row_to_dossier_snapshot` |
| `memory.rs` | Публичный интерфейс модуля: реэкспорт всех подмодулей, алиас `RelationshipEventStore as RelationshipEventPort` |
| `memory/cards.rs` | Структура `PersonMemoryCard` (9 полей), `PersonMemoryCardStore` с методами `list` (сортировка), `upsert` (`ON CONFLICT DO NOTHING`), `upsert_with_observation` |
| `memory/errors.rs` | Enum `PersonMemoryError` (5 вариантов) |
| `memory/facts.rs` | Структура `PersonFact` (12 полей), `PersonFactStore` с методами `list`, `upsert` (через `MemoryEngine::persona_fact_memory`, `ON CONFLICT DO NOTHING`), `upsert_with_observation`, `update_confidence`, `decay_unverified` (умножение на 0.5) |
| `memory/preferences.rs` | Структура `PersonPreference` (9 полей), `PersonPreferenceStore` с методами `list`, `upsert` (`ON CONFLICT ... DO UPDATE`), `upsert_with_observation` |

## Исходные файлы

- [`backend/src/domains/persons/identity/models.rs`](../../../../backend/src/domains/persons/identity/models.rs)
- [`backend/src/domains/persons/identity/rows.rs`](../../../../backend/src/domains/persons/identity/rows.rs)
- [`backend/src/domains/persons/identity/store.rs`](../../../../backend/src/domains/persons/identity/store.rs)
- [`backend/src/domains/persons/identity/store/candidates.rs`](../../../../backend/src/domains/persons/identity/store/candidates.rs)
- [`backend/src/domains/persons/identity/store/name_merge_candidates.rs`](../../../../backend/src/domains/persons/identity/store/name_merge_candidates.rs)
- [`backend/src/domains/persons/identity/store/queries.rs`](../../../../backend/src/domains/persons/identity/store/queries.rs)
- [`backend/src/domains/persons/identity/store/review.rs`](../../../../backend/src/domains/persons/identity/store/review.rs)
- [`backend/src/domains/persons/identity/store/review_state.rs`](../../../../backend/src/domains/persons/identity/store/review_state.rs)
- [`backend/src/domains/persons/identity/store/split_candidates.rs`](../../../../backend/src/domains/persons/identity/store/split_candidates.rs)
- [`backend/src/domains/persons/identity/upsert.rs`](../../../../backend/src/domains/persons/identity/upsert.rs)
- [`backend/src/domains/persons/identity/validation.rs`](../../../../backend/src/domains/persons/identity/validation.rs)
- [`backend/src/domains/persons/intelligence.rs`](../../../../backend/src/domains/persons/intelligence.rs)
- [`backend/src/domains/persons/investigator.rs`](../../../../backend/src/domains/persons/investigator.rs)
- [`backend/src/domains/persons/investigator/assembly.rs`](../../../../backend/src/domains/persons/investigator/assembly.rs)
- [`backend/src/domains/persons/investigator/errors.rs`](../../../../backend/src/domains/persons/investigator/errors.rs)
- [`backend/src/domains/persons/investigator/meeting_prep.rs`](../../../../backend/src/domains/persons/investigator/meeting_prep.rs)
- [`backend/src/domains/persons/investigator/models.rs`](../../../../backend/src/domains/persons/investigator/models.rs)
- [`backend/src/domains/persons/investigator/sections.rs`](../../../../backend/src/domains/persons/investigator/sections.rs)
- [`backend/src/domains/persons/investigator/service.rs`](../../../../backend/src/domains/persons/investigator/service.rs)
- [`backend/src/domains/persons/investigator/snapshots.rs`](../../../../backend/src/domains/persons/investigator/snapshots.rs)
- [`backend/src/domains/persons/memory.rs`](../../../../backend/src/domains/persons/memory.rs)
- [`backend/src/domains/persons/memory/cards.rs`](../../../../backend/src/domains/persons/memory/cards.rs)
- [`backend/src/domains/persons/memory/errors.rs`](../../../../backend/src/domains/persons/memory/errors.rs)
- [`backend/src/domains/persons/memory/facts.rs`](../../../../backend/src/domains/persons/memory/facts.rs)
- [`backend/src/domains/persons/memory/preferences.rs`](../../../../backend/src/domains/persons/memory/preferences.rs)

## Кандидаты на drift

1. **Дублирование парсинга состояний**: `PersonIdentityReviewState::parse` (в `models.rs`, `pub(super)`) и `parse_person_identity_review_state` (в `upsert.rs`, `pub(crate)`) реализуют идентичную логику сопоставления строк `"suggested"`, `"user_confirmed"`, `"user_rejected"`. Аналогично для `PersonIdentityCandidateKind::as_str`/`parse_person_identity_candidate_kind`. Два источника истины для одного и того же маппинга — риск расхождения при расширении enum.

2. **Несоответствие видимости методов `parse`**: `DossierReviewState::parse` объявлен как `pub`, а `PersonIdentityReviewState::parse` — как `pub(super)`, хотя оба выполняют идентичную функцию парсинга строки в enum. Либо оба должны быть `pub`, либо оба `pub(super)`.

3. **Хардкод `reviewed_by` в снапшотах досье**: В `snapshots.rs`, `review_dossier_snapshot` всегда устанавливает `reviewed_by = 'owner_persona'`, тогда как модель `DossierSnapshot` имеет поле `reviewed_by: Option<String>`, что предполагает возможность указания идентификатора пользователя. При наличии multi-user сценария это может потребовать передачи `actor_id`.

4. **Неопределённые константы**: Значения `PERSON_IDENTITY_ID_PREFIX`, `PERSON_IDENTITY_REVIEW_PREFIX`, `PERSON_IDENTITY_REVIEW_EVENT_TYPE`, `MIN_LIMIT`, `MAX_LIMIT`, `DEFAULT_LIMIT` определены в модуле `constants.rs`, не включённом в данный чанк. Невозможно верифицировать их фактические значения и согласованность с форматами ID и границами лимитов, документированными в других частях системы.

5. **Разная стратегия upsert для сущностей памяти**: `PersonFactStore::upsert` и `PersonMemoryCardStore::upsert` используют `ON CONFLICT DO NOTHING`, а `PersonPreferenceStore::upsert` использует `ON CONFLICT (person_id, preference_type) DO UPDATE` — перезаписывает существующие предпочтения. Это может быть осознанным дизайном, но в коде не документировано различие в поведении.
