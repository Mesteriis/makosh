### Summary / Резюме

Создать страницу `operations/backend-tests.md`, описывающую организацию, архитектурные ограничения и ключевые интеграционные тесты backend-части проекта `makosh`. Документация основывается исключительно на встроенных в контекст файлах тестов (Rust, `tokio::test` и синхронные тесты). Страница фиксирует покрытые области, архитектурные правила (максимальное количество строк в тестовых файлах), а также поведение, подтверждённое в конкретных тестовых функциях.

### Proposed pages / Предлагаемые страницы

#### `operations/backend-tests.md`

```markdown
# Backend Tests (Тестирование backend)

## Обзор

Тесты backend-части `makosh` написаны на Rust с использованием фреймворка `tokio::test` для асинхронных тестов и `#[test]` для синхронных. Интеграционные тесты поднимают изолированный экземпляр PostgreSQL через `testkit::context::TestContext`, а для внешних интеграций (Ollama, OmniRoute) используются подставные HTTP-сервера (`axum`).

Архитектурные тесты проверяют соблюдение ограничения: **ни один файл с тестами для определённого домена не должен превышать 700 строк**. Это правило реализовано для групп `messages` и `person_identity`.

Основные проверяемые домены:
- Сообщения и их рабочие состояния (Messages/Workflow)
- Обязательства (Obligations) – движок, хранилище, API
- Наблюдения (Observations) – захват, хранение, связи, ingestion-пайплайны
- Интеграции с AI (Ollama, OmniRoute)
- Организации (Organizations) – CRUD, подресурсы, enrichment, наблюдения
- Идентификация персон (Person Identity) – кандидаты на слияние/разделение, события, API

## Архитектурные ограничения

- **`messages_architecture.rs`** – ищет все файлы в директории тестов, содержащие `"messages"` в пути и расширение `.rs`. Если число строк > 700, тест падает с перечислением нарушителей.
- **`person_identity_architecture.rs`** – аналогично для файлов, связанных с `person_identity` (включая `person_identity.rs`, `person_identity_architecture.rs` и файлы в поддиректории `person_identity/`).

Эти тесты гарантируют, что тестовые файлы остаются обозримыми и не превращаются в монолиты.

## Тесты доменов

### Messages / Workflow (`messages/workflow.rs`)

Тесты перечислений `WorkflowState` и операций с сообщениями в PostgreSQL.

- **`workflow_state_from_str_all_valid`** – парсинг всех 8 допустимых строковых представлений: `"new" → WorkflowState::New`, `"reviewed"`, `"needs_action"`, `"waiting"`, `"done"`, `"archived"`, `"muted"`, `"spam"`.
- **`workflow_state_from_str_invalid`** – отклонение пустой строки, `"invalid_state"` и `"NEW"`.
- **`workflow_state_as_str_roundtrips`** – для каждого варианта `WorkflowState` преобразование в строку и обратно даёт исходное значение.
- **`workflow_state_valid_transitions`** – проверка допустимых переходов:
  - `New → Reviewed`, `New → NeedsAction`, `New → Archived`, `New → Muted`, `New → Spam` — разрешены.
  - `New → Done`, `New → Waiting` — запрещены.
  - `Reviewed → New` — разрешён.
  - `NeedsAction → Done`, `NeedsAction → Waiting`, `NeedsAction → Archived` — разрешены.
  - `Spam → New`, `Done → Archived`, `Archived → Reviewed`, `Archived → NeedsAction` — разрешены.
  - `New → New`, `Done → Done` — запрещены (переход в себя недопустим).
- **`workflow_state_serde_roundtrips`** – сериализация `WorkflowState::NeedsAction` в JSON-строку `"needs_action"` и обратная десериализация.
- **`message_workflow_state_transition_against_postgres`** – сквозной тест: создание учётной записи провайдера, запись сырого email-сообщения, проекция в `message_store`, последовательный перевод состояний (`New → NeedsAction → Done → Archived`), проверка итогового состояния.
- **`message_state_counts_against_postgres`** – подсчёт сообщений по состояниям для аккаунта; после создания двух сообщений и перевода одного в `Done` проверяется счётчик.
- **`message_list_filtering_by_state_against_postgres`** – фильтрация списка сообщений по `WorkflowState::New` и `WorkflowState::Done`.
- **`workflow_state_count_serialization`** – сериализация структуры `WorkflowStateCount` (состояние + количество) в JSON.

### Obligations Engine (`obligation_engine.rs`)

Тесты движка извлечения обязательств (`ObligationEngine`) из текста коммуникаций.

- **`obligation_engine_detects_owner_promise_from_communication`** – на входе текст `"I will send the revised project proposal by Friday 5pm."`. Движок определяет:
  - `kind = Commitment`
  - `statement = "send the revised project proposal"`
  - `quote = "I will send the revised project proposal by Friday 5pm."`
  - `due_text = Some("Friday 5pm")`
  - `confidence = 0.84`
  - `review_state = Suggested`
  - Сущности: obligated – Persona, beneficiary – Project; evidence source – Communication.
  - Также генерируются `task_candidates` и `follow_ups`.
- **`obligation_engine_detects_request_to_owner_without_autoconfirming`** – текст `"Please send the signed agreement before Monday morning."` даёт `kind = Request`, `confidence = 0.76`, `review_state = Suggested`.
- **`obligation_engine_ignores_deadline_without_commitment_language`** – фраза `"The office closes by Friday 5pm. The report was already sent."` не порождает ни обязательств, ни задач, ни follow-up.
- **`obligation_engine_rejects_empty_source_evidence_before_detection`** – пустой `source_id` вызывает ошибку `"source_id must not be empty"`.

### Obligations Store & API (`obligations.rs`, `obligations_api.rs`)

#### Хранилище (`obligations.rs`)

Файл обрезан после 12000 символов; ниже отражены видимые тесты.

- **`obligation_store_upserts_evidence_backed_obligation_without_creating_task_against_postgres`**
  - Создание обязательства с одним свидетельством, затем идемпотентный upsert с новым свидетельством.
  - Поля обязательства сохраняются: `status = Open`, `review_state = UserConfirmed`, `risk_state = Watch`, `confidence = 0.88`, `condition = "before the stakeholder review"`, beneficiary и obligated entity.
  - Свидетельство в БД содержит последнюю переданную цитату и confidence.
  - Проекция графа (graph projection) создаёт узлы `obligation`, `person`, `project` и рёбра `entity_relationship` с ролями `obligated_entity` и `beneficiary_entity`, со статусом `user_confirmed` и привязанным graph evidence.
  - Связи с задачами не создаются (`obligation_task_links` и `tasks` пусты).
- **`obligation_store_rejects_missing_evidence_before_database_write`** – ошибка `ObligationStoreError::MissingEvidence` при попытке сохранить обязательство без свидетельств.
- **`obligation_store_rejects_invalid_confidence_before_database_write`** – ошибка `InvalidScore("confidence", _)` при confidence > 1.0.
- **`obligation_store_rejects_partial_beneficiary_before_database_write`** – ошибка `PartialBeneficiary` при заданном `beneficiary_entity_kind`, но отсутствующем `beneficiary_entity_id`.
- **`obligation_store_rejects_missing_observation_evidence_against_postgres`** – (тест обрезан) проверяет, что ссылка на несуществующее observation-свидетельство отвергается.

#### API (`obligations_api.rs`)

- **`obligations_list_returns_entity_scoped_obligations`** – `GET /api/v1/obligations?entity_kind=persona&entity_id=...` возвращает обязательства, относящиеся к заданной сущности.
- **`obligations_list_returns_global_suggested_review_items`** – фильтр `review_state=suggested` возвращает только обязательства с этим статусом, исключая `user_confirmed`.
- **`put_obligation_review_updates_review_state_with_observation_trail`** – `PUT /api/v1/obligations/{obligation_id}/review` с `review_state: "user_confirmed"` обновляет состояние, создаёт observation-link типа `review_transition`, запись в review_items со статусом `promoted`, и observation с origin_kind=manual.

### Observations (`observations.rs`)

Файл обрезан после 12000 символов; ниже видимые тесты.

- **`manual_capture_creates_observation_without_vault_source_against_postgres`**
  - Создание наблюдения вручную (`VOICE_RECORDING`, origin `Manual`). Поля `kind_code`, `origin_kind`, `content_hash` (начинается с `sha256:`) сохранены. `vault_source_id = None`.
  - В `event_log` появляется событие `observation.captured.v1` с полями `event_id`, `correlation_id`, `causation_id`, `subject`, соответствующими созданному наблюдению.
- **`manual_note_creates_observation_without_vault_source_against_postgres`** – аналогично для вида `DOCUMENT`.
- **`observations_are_append_only_and_survive_provider_deletion_against_postgres`**
  - Попытки `UPDATE` и `DELETE` строки наблюдения блокируются (содержат в ошибке `"append-only"`).
  - Удаление на стороне провайдера моделируется новым наблюдением с кодом `COMMUNICATION_MESSAGE_DELETED`; старое наблюдение остаётся.
- **`observation_platform_persists_links_and_ingestion_runs_against_postgres`**
  - Создание наблюдения, upsert связи (`NewObservationLink`) с доменом и relation, проверка списка связей.
  - Запуск и завершение ingestion-run: статусы `Running` → `Succeeded`, фиксация `finished_at` и output.
- **`canonical_observation_kind_definitions_are_seeded_against_postgres`** – проверяет наличие обязательных кодов видов наблюдений: `COMMUNICATION_MESSAGE`, `COMMUNICATION_DRAFT`, `COMMUNICATION_FOLDER`, `COMMUNICATION_SAVED_SEARCH`, `COMMUNICATION_OUTBOX`, `COMMUNICATION_DELIVERY_STATUS`, `COMMUNICATION_READ_RECEIPT`, `CONTRADICTION_OBSERVATION`, `COMMUNICATION_MESSAGE_DELETED`, `COMMUNICATION_ATTACHMENT`, `MEETING`, `MEETING_RECORDING`, `MEETING_TRANSCRIPT`, `DOCUMENT`, `VOICE_RECORDING`, `BROWSER_CAPTURE`, `CONTACT_RECORD`, `CALENDAR_EVENT`, `CALENDAR_EVENT_DELETED`.
- **`browser_capture_creates_observation_without_vault_source_against_postgres`** – (тест обрезан).

### AI интеграции: Ollama и OmniRoute

#### Ollama (`ollama.rs`)

Используется подставной Ollama-сервер. Конфигурация клиента: `OllamaClientConfig` с базовым URL, моделями для чата (`qwen3:4b`) и эмбеддингов (`qwen3-embedding:4b`), таймаутом 5 секунд.

- **`ollama_client_round_trips_chat_embed_tags_and_version`**
  - `version()` возвращает `"0.17.4"`.
  - `tags()` содержит обе модели.
  - `chat("Return exactly: makosh-ai-ok")` возвращает `content = "makosh-ai-ok"`.
  - `embed("Макошь memory retrieval")` возвращает вектор размером 2560.
- **`ollama_client_strips_qwen_thinking_blocks_from_chat_content`** – контент ответа, содержащий `<think>...</think>`, очищается до `"Final cited answer."`.
- **`ollama_client_reports_missing_models_and_malformed_json`**
  - При отсутствии моделей в `tags` – ошибка `OllamaError::MissingModel`.
  - При невалидном JSON – `OllamaError::Protocol(_)`.

#### OmniRoute (`omniroute.rs`)

Подставной OmniRoute-сервер с эндпоинтами `/v1/models`, `/v1/chat/completions`, `/v1/embeddings`. Авторизация: `Bearer test-omniroute-key`.

- **`omniroute_client_round_trips_openai_compatible_models_chat_and_embeddings`**
  - `models()` содержит `codex/gpt-5.5` и `openai-compatible-chat-ollama-pve/qwen3-embedding:4b`.
  - `validate_required_models()` успешен.
  - `chat("Return exactly: makosh-omniroute-ok")` возвращает `content = "makosh-omniroute-ok"` (после удаления `<think>` блоков).
  - `embed("Макошь source-backed retrieval")` возвращает вектор размером 2560.
- **`omniroute_client_reports_auth_missing_models_and_malformed_json`**
  - Неавторизованный запрос (401) – `OmniRouteError::Endpoint { status: 401 }`.
  - Отсутствие моделей – `OmniRouteError::MissingModel`.
  - Невалидный JSON – `OmniRouteError::Protocol(_)`.

### Organizations API (`organizations_api.rs`)

Файл обрезан после 12000 символов; тесты используют общий токен `"orgs-test-token"`.

- **`orgs_auth_reject`** – запрос без заголовка `x-makosh-secret` возвращает `403 FORBIDDEN`.
- **`orgs_crud`** – создание, получение, обновление и архивирование организации. Проверяются статусы ответов.
- **`orgs_list`** – `GET /api/v1/organizations` возвращает успешный ответ.
- **`orgs_search`** – `GET /api/v1/organizations/search?q=test` возвращает успешный ответ.
- **`orgs_not_found_404`** – запрос несуществующей организации возвращает `404 NOT FOUND`.
- **Sub-resource endpoints** – макрос `org_test!` проверяет, что следующие endpoint-ы не возвращают 5xx ошибок:
  `identities`, `aliases`, `domains`, `departments`, `contacts`, `related`, `timeline`, `portals`, `procedures`, `playbooks`, `templates`, `financial`, `contracts`, `compliance`, `services`, `products`, `enrichment`, `risks`, `health`, `dossier`, `brief`, `context-pack`.
- **`orgs_enrichment_apply_captures_observation_against_postgres`** – применение enrichment-результата создаёт связь `observation_links` с `relationship_kind = 'review_transition'` для домена `organizations`.
- **`organization_manual_entrypoints_capture_observations_against_postgres`** – (тест обрезан) проверяет, что создание организации и добавление identity фиксируются через observation-связи.

Макросы `org_post_test!` проверяют создание identity, alias и department через POST без 5xx ошибок. Дополнительный тест проверяет переключение watchlist.

### Person Identity (`person_identity/`)

Тесты модуля идентификации персон (слияние и разделение кандидатов) и их API.

#### Основные тесты

- **`person_identity_reject_suppresses_candidate_against_postgres`** (`events.rs`) – после отклонения кандидата (review_state = `user_rejected`) повторный `refresh_candidates` не восстанавливает его.
- **`person_identity_review_event_rebuilds_state_against_postgres`** (`events.rs`) – последовательное применение событий (confirm → reject) через `apply_review_event` устанавливает итоговое состояние `user_rejected`; в таблице фиксируется event_id последнего события.
- **`person_identity_refresh_creates_conservative_merge_candidate_against_postgres`** (`merge_split.rs`) – `refresh_candidates` создаёт кандидата типа `merge_persons` со статусом `suggested`.
- **`person_identity_confirm_records_review_without_mutating_persons_against_postgres`** (`merge_split.rs`) – подтверждение кандидата меняет `review_state` на `user_confirmed`, но не удаляет и не изменяет записи в таблице `persons`.
- **`person_identity_confirm_materializes_split_candidate_against_postgres`** (`merge_split.rs`) – после подтверждения merge-кандидата появляется split-кандидат с `candidate_kind = "split_person"`, `confidence = 1.0`, `review_state = "suggested"`, и evidence_summary, начинающийся с `"Previously confirmed merge can be split:"`.
- **`person_identity_confirmed_split_removes_merge_from_detail_against_postgres`** (`merge_split.rs`) – после подтверждения split-кандидата merge-кандидат больше не отображается в детализации идентификации персоны.
- **`person_identity_refresh_skips_existing_split_when_generating_next_split_against_postgres`** (`refresh_ordering.rs`) – при наличии существующего split-кандидата и последующем создании следующего, `updated_at` первого split-кандидата не изменяется.

#### Person Identity API (`person_identity_api.rs`)

Файл обрезан; токен `"person-identity-api-test-token"`.

- **`identity_candidates_reject_missing_local_api_secret`** – запрос без токена возвращает `403 FORBIDDEN` и тело `{"error":"invalid_api_secret", ...}`.
- **`identity_candidates_returns_safe_candidate_payload`** – `GET /api/v1/identity-candidates` возвращает кандидатов с полями `candidate_kind`, `review_state`, `left_person_id`, `right_person_id`, `evidence_summary`, `confidence`. В review_items создаётся запись с `item_kind = "identity_candidate"` и `mirrored_from = "identity_candidates"`, а в review_item_evidence — observation с кодом `PERSON_IDENTITY_CANDIDATE`.
- **`identity_candidates_returns_split_candidate_for_confirmed_merge`** – после подтверждения merge через API появляется split-кандидат с соответствующим `evidence_summary`.
- **`put_identity_candidate_review_confirms_candidate`** – `PUT /api/v1/identity-candidates/{id}/review` с `review_state: "user_confirmed"` возвращает подтверждённый статус и `event_id`, создаёт review_item со статусом `promoted`.
- **`run_person_identity_review_inbox_consumer`** – вспомогательная функция для обработки событий review inbox consumer (до 10 попыток, пока есть события).

### Прочие тесты

- **`persons.rs`** – модуль-агрегатор, подключающий файлы `health_dossier.rs`, `identities.rs`, `memory_preferences.rs`, `projection.rs`, `relationships.rs`, `support.rs`. Сами тесты в данном контексте не раскрыты.

> **Примечание:** Для файлов, обрезанных после 12000 символов (`obligations.rs`, `observations.rs`, `organizations_api.rs`, `person_identity_api.rs`), приведённое описание основывается только на видимой части. Полное поведение может включать дополнительные сценарии, не попавшие в контекст.
```

### Source coverage / Покрытие источников

- **`backend/tests/messages/workflow.rs`**
  — парсинг/валидация строк `WorkflowState`; раундтрип `as_str() -> parse()`; допустимые/недопустимые переходы; сериализация/десериализация `WorkflowState` и `WorkflowStateCount`; интеграционные тесты переходов состояний сообщения в PostgreSQL; подсчёт сообщений по состояниям; фильтрация списка сообщений по `WorkflowState`.

- **`backend/tests/messages_architecture.rs`**
  — архитектурный тест: ни один файл тестов сообщений не превышает 700 строк.

- **`backend/tests/obligation_engine.rs`**
  — обнаружение commitment из коммуникации (поля statement, quote, due_text, confidence, review_state, сущности); обнаружение request без автоподтверждения; игнорирование дедлайна без маркеров обязательства; отклонение пустого source_id.

- **`backend/tests/obligations.rs`** (truncated)
  — upsert обязательства с evidence (поля, перезапись свидетельства, отсутствие связей с задачами); проекция графа с узлами и рёбрами, graph evidence; ошибки при отсутствии evidence, некорректном confidence, частичном beneficiary.

- **`backend/tests/obligations_api.rs`**
  — `GET /api/v1/obligations` с фильтрацией по entity и review_state; `PUT .../review` с созданием observation-link, review_item и observation (origin manual).

- **`backend/tests/observations.rs`** (truncated)
  — захват manual observation (поля, событие в event_log); append-only характер (блокировка UPDATE/DELETE, моделирование удаления новым наблюдением); связи и ingestion runs; канонические коды видов наблюдений.

- **`backend/tests/ollama.rs`**
  — round-trip клиента Ollama (version, tags, chat, embed); удаление блоков `<think>`; ошибки при отсутствии моделей и невалидном JSON.

- **`backend/tests/omniroute.rs`**
  — round-trip клиента OmniRoute (models, chat, embed размер 2560); ошибки 401, MissingModel, Protocol.

- **`backend/tests/organizations_api.rs`** (truncated)
  — авторизация (403); CRUD (create, get, update, archive); list, search, 404; sub-resource endpoints (21 штука) без 5xx; enrichment apply создаёт observation link; создание identity, alias, department; watchlist toggle; фиксация observation при ручных действиях.

- **`backend/tests/person_identity.rs`**
  — объявление подмодулей (events, merge_split, refresh_ordering, support).

- **`backend/tests/person_identity/events.rs`**
  — подавление кандидата после reject; восстановление состояния через apply_review_event (последний reject выигрывает).

- **`backend/tests/person_identity/merge_split.rs`**
  — создание merge_persons кандидата; подтверждение без удаления persons; появление split_person кандидата после подтверждения merge; удаление merge из detail после подтверждения split.

- **`backend/tests/person_identity/refresh_ordering.rs`**
  — сохранение updated_at существующего split-кандидата при генерации следующего.

- **`backend/tests/person_identity/support.rs`**
  — вспомогательные функции (контекст, посев имён, подтверждение/промоут/aging кандидатов, проверка существования, построение ID, события).

- **`backend/tests/person_identity_api.rs`** (truncated)
  — отклонение без токена; содержимое ответа GET кандидатов (поля, review_item, observation PERSON_IDENTITY_CANDIDATE); split-кандидат после confirmed merge; PUT review подтверждение.

- **`backend/tests/person_identity_architecture.rs`**
  — архитектурный тест: ограничение 700 строк для файлов person_identity.

- **`backend/tests/persons.rs`**
  — объявление подмодулей (health_dossier, identities, memory_preferences, projection, relationships, support) без раскрытия содержимого.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно. Предлагаемая страница создаётся на основе исходного кода и отражает актуальное поведение, зафиксированное в тестах. Для проверки полноты описания обрезанных файлов потребуется их полное содержимое.
