---
chunk_id: 086-test-backend-part-009
batch_id: batch-20260628T214902
group: backend
role: test
source_status: pending
source_count: 19
generated_by: code-wiki-ru
---

# 086-test-backend-part-009 — backend/test

- Target index: [[operations/backend-tests]]
- Batch: `batch-20260628T214902`
- Source files: `19`

## Резюме

Создать новую русскую wiki-страницу `operations/backend-tests.md` с документированием структуры, паттернов, покрытия и архитектурных ограничений тестового набора бэкенда Макошь (доменные интеграционные тесты `persons`, HTTP-API тесты `persons_api` и проверки лимитов строк). Описание формируется исключительно на основе предоставленных исходных файлов тестов.

## Предложенные страницы

#### `operations/backend-tests.md`

```markdown
# Бэкенд-тесты

Тестовый набор backend-тестов находится в `backend/tests/` и организован по двум направлениям:

- **`persons/`** — интеграционные тесты доменной логики персон (projection, identities, memory/preferences, health/dossier, relationships).
- **`persons_api/`** — тесты HTTP-API эндпоинтов для персон и связанных сущностей (personas, identity traces, write-операции, observation-пинги).
- **`persons_architecture.rs`** и **`persons_api_architecture.rs`** — проверяют, что файлы тестов не превышают лимит в **700 строк**.

Все интеграционные и API-тесты используют `#[tokio::test]` с асинхронным рантаймом и подключаются к реальной базе данных PostgreSQL через `testkit::TestContext`.

## Вспомогательные утилиты

- **`live_persons_pool(test_name)`** — возвращает `Option<PgPool>`; если контекст не доступен, тест завершается рано (`return`).
- **`live_persons_store(test_name)`** — возвращает `Option<PersonProjectionStore>`.
- **`disconnected_persons_store()`** — создаёт "ленивый" пул для проверки ошибок валидации на входе без реальной БД.
- **`unique_suffix()`** — генерирует суффикс на основе `SystemTime::nanos`.
- **`run_person_derived_evidence_consumer(pool)`** — запускает Consumer `person_derived_evidence`, обрабатывая события через `project_person_derived_evidence_event`.

Тесты, проверяющие порождение derived evidence (взаимоотношения, обязательства), принудительно дёргают этот consumer.

В API-тестах используется константа `LOCAL_API_TOKEN = "persons-api-test-token"` и помощники для построения приложения (`build_persons_app`, `build_persons_app_with_database`, `build_persons_app_without_database`).

## Persons: доменные интеграционные тесты

### Projection (`backend/tests/persons/projection.rs`)

- **`persons_projection_upserts_email_identities_against_postgres`** — `upsert_persons_from_message_participants` создаёт персоны по email.
- **`persons_projection_normalizes_and_deduplicates_participants_against_postgres`** — дубликаты и пробельные адреса схлопываются в одну нормированную запись.
- **`persons_projection_rejects_blank_email_participant`** — пустой email вызывает ошибку `EmptyEmailAddress`.
- **`persons_projection_rejects_invalid_batch_before_writing_against_postgres`** — некорректная партия не приводит к частичной записи (atomicity).
- **`persons_projection_distinguishes_delimiter_bearing_email_identities_against_postgres`** — идентификаторы с разделителями (`:` vs `-`) не пересекаются.
- **`persons_projection_defaults_to_human_non_owner_persona_against_postgres`** — персоны по умолчанию имеют `PersonaType::Human` и `is_self == false`.
- **`persons_projection_tracks_single_owner_persona_against_postgres`** — `set_owner_persona` перемещает флаг `is_self` на одного владельца.
- **`persons_projection_sets_supported_persona_type_against_postgres`** — тип персоны можно сменить на `AiAgent`.
- **`persons_schema_rejects_invalid_persona_type_against_postgres`** — недопустимый тип вызывает нарушение check-ограничения `persons_person_type_check`.

### Identities (`backend/tests/persons/identities.rs`)

- **`person_identities_accept_document_and_message_traces_against_postgres`** — сохраняются identity-следы типов `document_mention` и `message_participant`.
- **`person_identities_accept_disputed_status_against_postgres`** — статус identity можно перевести в `disputed`.
- **`person_identities_support_unattached_trace_assignment_against_postgres`** — непривязанный след (`create_unattached`) можно позже закрепить за персоной через `attach_to_persona`.

### Memory, Preferences, Enrichment (`backend/tests/persons/memory_preferences.rs`)

- **`person_persona_upsert_and_delete_materializes_interaction_preferences_against_postgres`** — upsert персоны-контекста создаёт записи в `person_preferences` типа `interaction_context:<id>:<field>`, удаление контекста очищает их.
- **`person_notes_materialize_persona_memory_card_against_postgres`** — заметки (`set_notes`) сохраняются в columns `persons.notes` и в `person_memory_cards` с importance=5.
- **`person_fact_upsert_uses_memory_engine_source_backed_draft_against_postgres`** — факты тримятся, сохраняют `fact_type`, `value`, `source`, `confidence`.
- **`person_favorite_toggle_materializes_ui_preference_against_postgres`** — тоггл `toggle_favorite` создаёт / удаляет preference `ui:favorite` с source `persons.is_favorite:<person_id>`.
- **`person_watchlist_toggle_materializes_ui_preference_against_postgres`** — тоггл `toggle_watchlist` создаёт / удаляет preference `ui:watchlist` с source `persons.watchlist:<person_id>`.
- **`person_enrichment_result_upsert_materializes_pending_source_backed_candidate_against_postgres`** — enrichment-результаты сохраняются в статусе `pending`, содержат поля `_enrichment` (affected_entity_kind/id, review_state, freshness, conflict_marker).

### Health & Dossier (`backend/tests/persons/health_dossier.rs`)

- **`person_risk_report_and_resolve_materializes_health_status_cache_against_postgres`** — риск `relationship_attention` с severity=high, confidence=0.5 переводит `health_status` в `at_risk` (open_risks=1); resolve возвращает `healthy` (open_risks=0).
- **`person_dossier_includes_target_sections_and_source_refs_against_postgres`** — досье собирает секции: interests, projects, organizations, skills, communication_patterns; каждое свойство имеет `source_refs`, досье содержит `generated_at` и `ai_observations`.

### Relationships (`backend/tests/persons/relationships.rs`)

- **`person_role_assign_and_remove_materializes_relationship_against_postgres`** — назначение роли создаёт relationship `has_role` с review_state=`user_confirmed`, evidence через observation типа `PERSON_ROLE`; удаление переводит в `user_rejected`.
- **`person_enrichment_trust_score_materializes_owner_relationship_against_postgres`** — enrichment с trust_score=82 порождает relationship `trusts` (source_entity=owner persona, target=target persona) с review_state=`suggested`, trust_score=0.82, evidence типа `PERSON_TRUST_SIGNAL` и review-зеркало в `review_items`.
- **`person_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres`** — обещание создаёт Obligation со статусом `Open`, `UserConfirmed`, с метадатой `person_promise_id` и evidence-наблюдением.

## Persons API: тесты HTTP-эндпоинтов

Модуль собран в `backend/tests/persons_api.rs` и включает:

- `auth`
- `dossier_owner`
- `identity_traces`
- `persona_routes`
- `read_endpoints`
- `support`
- `write_entrypoints_basic`
- `write_identity_timeline`
- `write_memory_observations`
- `write_review_observations`

### Auth (`backend/tests/persons_api/auth.rs`)

- **`persons_rejects_missing_local_api_secret`** — запрос без заголовка `x-makosh-secret` возвращает 403 с телом `{"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"}`.

### Read endpoints (`backend/tests/persons_api/read_endpoints.rs`)

С помощью макроса `person_endpoint_test!` для несуществующей персоны проверяется, что каждый GET-эндпоинт не возвращает 5xx ошибку:
- `/persons/{id}/identities`, `/roles`, `/personas`, `/facts`, `/memory-cards`, `/preferences`, `/timeline`, `/snapshots`, `/history-diff`, `/enrichment`, `/expertise`, `/promises`, `/risks`, `/health`, `/dossier`, `/meeting-prep`, `/analytics`, `/export`, `/identity`.
- `/persons/search?q=alex` возвращает 200.
- `/persons/health` возвращает 200.
- `/persons/watchlist` возвращает 200.
- `/identity-candidates` возвращает 200.
- `/persons/search/expertise?q=rust` не является 5xx.

### Persona native schema (`backend/tests/persons_api/persona_routes.rs`)

- **`persons_list_returns_ok`** — `GET /api/v1/persons` отдаёт 200.
- **`personas_routes_return_persona_native_schema_against_postgres`** — `GET /api/v1/personas?limit=20` включает owner-персону; `GET /api/v1/personas/{id}` отдаёт поля `persona_id`, `persona_type`, `is_self`, `identity`, `communication`, `compatibility` с `legacy_route`.
- **`personas_put_updates_compatibility_projection_against_postgres`** — `PUT /api/v1/personas/{id}` обновляет `display_name` и `is_self` (только owner). Попытка снять `is_self` у owner возвращает 400. Обновление создаёт observation `persona_update` типа `PERSON_MUTATION`.

### Owner persona & Dossier (`backend/tests/persons_api/dossier_owner.rs`)

- **`person_owner_get_and_put_uses_owner_persona_against_postgres`** — `GET /api/v1/persons/owner` изначально возвращает null; `PUT` устанавливает владельца и порождает observation `owner_assignment` типа `PERSON_MUTATION`.
- **`person_dossier_get_persists_snapshot_and_review_state_against_postgres`** — `GET /api/v1/persons/{id}/dossier` создаёт снапшот досье с `review_state=suggested`, сохраняет в `persona_dossier_snapshots` и создаёт `dossier_refresh` observation-ссылку. `PUT /persons/{id}/dossier/review` с `review_state=user_confirmed` обновляет состояние и создаёт `review_transition` ссылку.
- **`person_investigate_captures_observation_and_links_snapshot_against_postgres`** — `POST /persons/{id}/investigate` создаёт observation `PERSON_MUTATION` и привязывает к снапшоту как `dossier_refresh`.
- **`person_detail_not_found_returns_404`** — несуществующая персона получает 404.

### Identity traces API (`backend/tests/persons_api/identity_traces.rs`)

- **`identity_traces_create_list_and_attach_unattached_trace`** — создание трейса `POST /api/v1/identity-traces` возвращает `person_id = null`, создаёт observation `PERSON_RECORD_MUTATION` с `origin_kind=manual`. Список `GET /api/v1/identity-traces?status=unattached` содержит трейс. Привязка `PUT /identity-traces/{id}/assignment` связывает трейс с персоной и создаёт observation `trace_assignment` типа `PERSON_RECORD_MUTATION`.

### Write entrypoints (basic) (`backend/tests/persons_api/write_entrypoints_basic.rs`)

Макрос `person_post_test!` покрывает базовое отсутствие 5xx для POST:
- `POST /persons/{id}/fingerprint`
- `POST /persons/{id}/favorite`
- `POST /persons/{id}/investigate`
- `POST /persons/{id}/facts`
- `POST /persons/{id}/memory-cards`
- `POST /persons/{id}/preferences`
- `POST /persons/{id}/timeline`

Дополнительно:
- **`person_put_notes`** — `PUT /persons/{id}/notes` не падает с 5xx.
- **`person_roles_post_and_delete`** — `POST /persons/{id}/roles` и `DELETE /persons/{id}/roles/colleague` не падают с 5xx.
- **`person_persona_post_and_delete`** — `POST /persons/{id}/personas` и `DELETE /persons/{id}/personas/pers:fake` не падают с 5xx.
- **`person_watchlist_toggle`** — `POST /persons/{id}/watchlist` не падает с 5xx.

### Write identity timeline (`backend/tests/persons_api/write_identity_timeline.rs`)

- **`person_identity_post_and_delete`** — `POST /persons/{id}/identities` создаёт identity; `DELETE` возвращает `{"deleted": true}` и создаёт observation `identity_delete` типа `PERSON_RECORD_MUTATION` с metadata `deleted=true`.
- **`person_relationship_timeline_entrypoint_captures_observation_against_postgres`** — `POST /persons/{id}/timeline` создаёт relationship event, сохраняя source вида `observation:<id>`. Observation имеет `origin_kind=manual`, kind `PERSON_RECORD_MUTATION`, и привязан к событию через ссылку.

### Write memory observations (`backend/tests/persons_api/write_memory_observations.rs`)

- **`person_manual_memory_entrypoints_capture_observations_against_postgres`** — ручные операции:
  - `PUT /persons/{id}/notes` — memory card с source `observation:<id>`.
  - `POST /persons/{id}/facts` — fact с source `observation:<id>`, observation типа `PERSON_RECORD_MUTATION`.
  - `POST /persons/{id}/memory-cards` — card с source `observation:<id>`, observation типа `PERSON_MEMORY_CARD`.
  - `POST /persons/{id}/preferences` — preference с source `observation:<id>`, observation типа `PERSON_RECORD_MUTATION`.
  - `POST /persons/{id}/fingerprint` — создаёт observation `profile_enrichment` типа `PERSON_MUTATION`, а также порождает trust signal типа `PERSON_TRUST_SIGNAL`.
  - `POST /persons/{id}/favorite` — preference `ui:favorite` с source `observation:<id>`, observation link `favorite_toggle`.
  - `POST /persons/{id}/watchlist` — preference `ui:watchlist` с source `observation:<id>`, observation link `watchlist_toggle`.
- Все observation имеют `origin_kind=manual`.

### Write review observations (`backend/tests/persons_api/write_review_observations.rs`)

- **`person_enrichment_review_entrypoints_capture_observations_against_postgres`** — применение/отклонение enrichment через `POST /persons/{id}/enrichment/{eid}/apply` и `/reject` создаёт `review_transition` observation-ссылку.
- **`person_compatibility_entrypoints_capture_observations_against_postgres`** — создание/удаление ролей и personas создаёт observation с kind `PERSON_RECORD_MUTATION`, origin `manual`. Preference persona также имеет source `observation:<id>`.
- **`identity_candidate_review_captures_observation_against_postgres`** — ревью identity-кандидата порождает observation. (Файл обрезан, полные детали не подтверждены контекстом.)

## Событийный consumer

Многие тесты, затрагивающие производимые отношения и обязательства, используют `run_person_derived_evidence_consumer(pool)` для обработки событий через конфигурацию `PERSON_DERIVED_EVIDENCE_CONSUMER` и функцию `project_person_derived_evidence_event`.

## Архитектурные ограничения

- **`persons_architecture.rs`** — файлы тестов в папке `persons/` не должны превышать 700 строк.
- **`persons_api_architecture.rs`** — файлы тестов в папке `persons_api/` и сами файлы `persons_api.rs` / `persons_api_architecture.rs` не должны превышать 700 строк.
```

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/tests/persons/health_dossier.rs` | риск: report/resolve и статус здоровья (`at_risk`/`healthy`), разделы досье (interests, projects, organizations, skills, communication_patterns), source_refs, generated_at, ai_observations |
| `backend/tests/persons/identities.rs` | identity-следы типов `document_mention` и `message_participant`, статус `disputed`, создание непривязанного следа (`create_unattached`) и последующее `attach_to_persona` |
| `backend/tests/persons/memory_preferences.rs` | материализация preferences для interaction-контекстов, notes → memory card с importance=5, upsert факта (тримминг, source, confidence), toggle `ui:favorite` / `ui:watchlist`, enrichment-результаты со статусом `pending` и метаданными `_enrichment` (файл обрезан, полные детали не подтверждены) |
| `backend/tests/persons/projection.rs` | upsert participants, нормализация/дедупликация, пустой email → `EmptyEmailAddress`, атомарность батча, отличие delimiter-bearing email, тип по умолчанию `Human`/`is_self=false`, единственный owner, смена persona_type (включая `AiAgent`), нарушение check-ограничения `persons_person_type_check` для невалидного типа |
| `backend/tests/persons/relationships.rs` | роль → relationship `has_role` + evidence `PERSON_ROLE`, удаление роли → `user_rejected`; trust-score enrichment → relationship `trusts` + evidence `PERSON_TRUST_SIGNAL` + review-зеркало; promise → Obligation со статусом `Open`/`UserConfirmed` + evidence (файл обрезан) |
| `backend/tests/persons/support.rs` | хелперы: `live_persons_pool`, `live_persons_store`, `disconnected_persons_store`, `unique_suffix`, `run_person_derived_evidence_consumer` |
| `backend/tests/persons_api.rs` | перечень подключённых подмодулей API-тестов |
| `backend/tests/persons_api/auth.rs` | 403 при отсутствии `x-makosh-secret` |
| `backend/tests/persons_api/dossier_owner.rs` | снапшот досье и review-состояния, `dossier_refresh`/`review_transition` observation-ссылки; `/investigate` → observation `PERSON_MUTATION`; `/owner` GET/PUT и observation `owner_assignment`; 404 для несуществующей персоны |
| `backend/tests/persons_api/identity_traces.rs` | создание непривязанного следа (`POST /api/v1/identity-traces`), список с фильтром `status=unattached`, привязка (`PUT .../assignment`), observation `trace_assignment` и `PERSON_RECORD_MUTATION` |
| `backend/tests/persons_api/persona_routes.rs` | `GET /api/v1/persons`, `GET /api/v1/personas?limit=20` и `GET /api/v1/personas/{id}` с native schema (identity, communication, compatibility), `PUT persona` обновляет `display_name`, `is_self`, не позволяет снять `is_self` (400), создаёт observation `persona_update` типа `PERSON_MUTATION` |
| `backend/tests/persons_api/read_endpoints.rs` | smoke-тесты (отсутствие 5xx) для множества GET-эндпоинтов (включая search, health, watchlist, identity-candidates, expertise search) |
| `backend/tests/persons_api/support.rs` | хелперы для API-тестов: конфиг с API-токеном, конструкторы запросов (get/post/put/delete), `json_body`, `urlencoding_percent_encode`, `live_database_url`, `build_persons_app*`, `run_person_derived_evidence_consumer` |
| `backend/tests/persons_api/write_entrypoints_basic.rs` | smoke-тесты для POST `fingerprint`, `favorite`, `investigate`, `facts`, `memory-cards`, `preferences`, `timeline`; PUT `notes`; POST+DELETE для `roles` и `personas`; POST `watchlist` toggle |
| `backend/tests/persons_api/write_identity_timeline.rs` | identity POST+DELETE с observation `identity_delete` (metadata `deleted=true`); timeline POST создаёт relationship event с source `observation:<id>`, observation вида `PERSON_RECORD_MUTATION` и origin `manual` |
| `backend/tests/persons_api/write_memory_observations.rs` | ручные memory-записи (notes, facts, memory-cards, preferences, fingerprint, favorite, watchlist) генерируют observation с соответствующими kind (`PERSON_RECORD_MUTATION`, `PERSON_MEMORY_CARD`, `PERSON_MUTATION`, `PERSON_TRUST_SIGNAL`) и origin=`manual` (файл обрезан) |
| `backend/tests/persons_api/write_review_observations.rs` | enrichment apply/reject → `review_transition` ссылки; compatibility-операции (roles/personas create/delete) создают observation `PERSON_RECORD_MUTATION` с origin `manual`; identity candidate review порождает observation (файл обрезан) |
| `backend/tests/persons_api_architecture.rs` | ограничение 700 строк для файлов в `persons_api/` и файлов `persons_api.rs`/`persons_api_architecture.rs` |
| `backend/tests/persons_architecture.rs` | ограничение 700 строк для файлов в `persons/` и файлов `persons.rs`/`persons_architecture.rs` |

## Исходные файлы

- [`backend/tests/persons/health_dossier.rs`](../../../../backend/tests/persons/health_dossier.rs)
- [`backend/tests/persons/identities.rs`](../../../../backend/tests/persons/identities.rs)
- [`backend/tests/persons/memory_preferences.rs`](../../../../backend/tests/persons/memory_preferences.rs)
- [`backend/tests/persons/projection.rs`](../../../../backend/tests/persons/projection.rs)
- [`backend/tests/persons/relationships.rs`](../../../../backend/tests/persons/relationships.rs)
- [`backend/tests/persons/support.rs`](../../../../backend/tests/persons/support.rs)
- [`backend/tests/persons_api.rs`](../../../../backend/tests/persons_api.rs)
- [`backend/tests/persons_api/auth.rs`](../../../../backend/tests/persons_api/auth.rs)
- [`backend/tests/persons_api/dossier_owner.rs`](../../../../backend/tests/persons_api/dossier_owner.rs)
- [`backend/tests/persons_api/identity_traces.rs`](../../../../backend/tests/persons_api/identity_traces.rs)
- [`backend/tests/persons_api/persona_routes.rs`](../../../../backend/tests/persons_api/persona_routes.rs)
- [`backend/tests/persons_api/read_endpoints.rs`](../../../../backend/tests/persons_api/read_endpoints.rs)
- [`backend/tests/persons_api/support.rs`](../../../../backend/tests/persons_api/support.rs)
- [`backend/tests/persons_api/write_entrypoints_basic.rs`](../../../../backend/tests/persons_api/write_entrypoints_basic.rs)
- [`backend/tests/persons_api/write_identity_timeline.rs`](../../../../backend/tests/persons_api/write_identity_timeline.rs)
- [`backend/tests/persons_api/write_memory_observations.rs`](../../../../backend/tests/persons_api/write_memory_observations.rs)
- [`backend/tests/persons_api/write_review_observations.rs`](../../../../backend/tests/persons_api/write_review_observations.rs)
- [`backend/tests/persons_api_architecture.rs`](../../../../backend/tests/persons_api_architecture.rs)
- [`backend/tests/persons_architecture.rs`](../../../../backend/tests/persons_architecture.rs)

## Кандидаты на drift

Из предоставленного контекста (только исходные файлы тестов) расхождений между кодом, документацией и ADR не обнаружено. Файлы `memory_preferences.rs`, `relationships.rs`, `write_memory_observations.rs` и `write_review_observations.rs` обрезаны, поэтому полное покрытие их утверждений не может быть подтверждено — но это не является дрифтом.
