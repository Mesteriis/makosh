---
chunk_id: 087-test-backend-part-010
batch_id: batch-20260628T214902
group: backend
role: test
source_status: pending
source_count: 12
generated_by: code-wiki-ru
---

# 087-test-backend-part-010 — backend/test

- Target index: [[operations/backend-tests]]
- Batch: `batch-20260628T214902`
- Source files: `12`

## Резюме

Создана страница `operations/backend-tests.md` — обзор бэкенд-тестов из `backend/tests/`. Документ описывает структуру тестов, перечисляет модули и ключевые проверяемые сценарии. Основан исключительно на предоставленных исходных файлах.

## Предложенные страницы

**Path:** `operations/backend-tests.md`

```markdown
# Бэкенд-тесты

## Обзор

Этот раздел описывает тесты, расположенные в `backend/tests/`. Тесты делятся на:

- **Интеграционные** с реальной PostgreSQL (через `testkit::TestContext`), использующие `tokio::test`.
- **Модульные** тесты для движков, работающих без БД (поиск, риск-движок).

Каждый файл фокусируется на конкретной подсистеме Макошь: проекты, отношения, ревью, секреты, настройки, поиск и др.

Тесты используют уникальный суффикс (`unique_suffix()`) для предотвращения конфликтов между запусками.

## Структура тестового модуля

- Файлы в `backend/tests/` содержат тестовые функции, вспомогательные структуры (например, `LiveProjectContext`) и seed-функции (`seed_message`, `seed_document`).
- Тесты API взаимодействуют с `axum::Router` через `tower::ServiceExt`.
- Для работы с БД используется `sqlx::PgPool`, полученный из `Database::connect`.

## Тестовые модули

### project_link_reviews

- `project_link_review_command_appends_event_and_updates_review_against_postgres` — команда `set_review_state` с `UserConfirmed` добавляет событие и обновляет строку `project_link_reviews`.
- `project_link_review_confirm_materializes_user_confirmed_decision_against_postgres` — подтверждение создаёт Decision (`DecisionReviewState::UserConfirmed`), запись в `decision_impacted_entities` (project + communication), evidence с source_kind `observation` и observation типа `PROJECT_LINK_REVIEW`.
- `project_link_review_confirm_materializes_relationship_against_postgres` — подтверждение создаёт Relationship типа `project_has_message` с `UserConfirmed`, confidence 1.0, evidence с observation типа `PROJECT_LINK_REVIEW`.
- `project_link_review_reset_clears_review_and_demotes_relationship_against_postgres` (обрезано) — сброс ревью очищает состояние и понижает связанные отношения.

### projection_runner

- `projection_runner_processes_batch_and_advances_cursor_against_postgres` — `run_projection_batch` обрабатывает события из `EventStore`, вызывает обработчик для каждого и обновляет позицию в `ProjectionCursorStore`.
- `projection_runner_stops_on_handler_error_without_advancing_failed_event_against_postgres` — при ошибке обработчика позиция не продвигается за сбойное событие; при повторном запуске сбойное событие обрабатывается заново, после успеха курсор обновляется.

### projects

- `project_detail_links_keyword_messages_documents_and_people_against_postgres` — метод `project_detail` возвращает проект с подсчётом сообщений, документов, людей, временной шкалой и списком ключевых персон, отфильтрованных по ключевым словам.
- `project_detail_excludes_rejected_keyword_message_against_postgres` — сообщение, сопоставленное по ключевому слову, но с review_state `UserRejected`, исключается из детализации.
- `project_detail_includes_confirmed_non_keyword_message_against_postgres` — сообщение без ключевого слова, но подтверждённое вручную (`UserConfirmed`), включается в детализацию.

### projects_api

- `projects_rejects_missing_local_api_secret` — запрос без `x-makosh-secret` возвращает 403 с телом `{"error":"invalid_api_secret","message":"missing or invalid x-makosh-secret header"}`.
- `project_detail_returns_live_project_payload` — `GET /api/v1/projects/{id}` с валидным токеном возвращает 200 с полями проекта.
- `project_link_candidates_rejects_missing_local_api_secret` — аналогичная проверка для `link-candidates`.
- `project_link_candidates_return_safe_message_and_document_candidates` — `GET .../link-candidates` возвращает список кандидатов (`review_state: "suggested"`) и создаёт записи в `review_items` с `item_kind = "project_link_candidate"` и `mirrored_from = "project_link_candidates"`.
- `put_project_link_review_updates_review_state` — `PUT /api/v1/projects/{id}/link-reviews` применяет review_state, создаёт observation link с relationship_kind `review_transition` и обновляет статус review item до `promoted`.
- `put_project_link_review_rejects_missing_target` (обрезано) — проверка валидации отсутствующего target.

### relationships

- `relationship_store_upserts_persona_relationship_with_evidence_against_postgres` — `RelationshipStore::upsert_with_evidence` создаёт или обновляет связь между персонами и доказательства; повторный вызов заменяет evidence.
- `relationship_store_projects_persona_relationship_into_graph_against_postgres` — связь проецируется в таблицу `graph_edges` с атрибутами confidence, review_state, properties и в `graph_evidence`.
- `relationship_store_projects_supported_cross_domain_relationship_into_graph_against_postgres` — связь между Decision и Project также создаёт графовое ребро.
- `relationship_store_projects_organization_task_relationship_into_graph_against_postgres` (обрезано) — вероятно, аналогичная проекция для организации и задачи.

### relationships_api

- `relationships_list_returns_entity_scoped_relationships` — `GET /api/v1/relationships?entity_kind=persona&entity_id=...` возвращает только связи, касающиеся указанной сущности.
- `relationships_list_returns_global_suggested_review_items` — фильтр `review_state=suggested` возвращает только связи с этим состоянием.
- `put_relationship_review_updates_relationship_and_graph_projection` — `PUT /api/v1/relationships/{id}/review` изменяет review_state, обновляет графовое ребро, создаёт observation link с origin_kind `manual` и review item со статусом `promoted`.

### review_inbox

- `review_inbox_creates_evidence_backed_item_against_postgres` — `ReviewInboxStore::create_with_evidence` создаёт элемент (например, `PotentialTask`) с ссылкой на observation; генерируются события `task.candidate.detected.v1` и `review.item.available.v1`.
- `review_inbox_filters_active_and_all_lists_against_postgres` — `list_open` исключает Dismissed элементы; `list_all` включает все.
- `review_inbox_lifecycle_approves_promotes_dismisses_and_archives_against_postgres` — проверяет переходы: New → InReview → Approved → Promoted, а также New → Dismissed → Archived. Каждый переход сопровождается событием (`decision.candidate.detected.v1`, `review.item.available.v1`, `review.item.approved.v1`, `review.item.promoted.v1`, `review.item.dismissed.v1`).
- `review_inbox_status_with_observation_materializes_transition_link_against_postgres` (обрезано) — вероятно, переход статуса с observation создает observation link.

### risk_engine

- `risk_engine_derives_attention_status_from_unresolved_severity` — `RiskEngine::derive_attention_status` на основе неразрешённых сигналов определяет `RiskAttentionStatus` (Healthy/NeedsAttention/AtRisk).
- `risk_severity_rejects_unknown_compatibility_values` — `RiskSeverity::parse` отклоняет недопустимые строки.
- `risk_engine_builds_source_backed_persona_observation_draft` — `RiskEngine::persona_observation` формирует черновик observation с полями `risk_type`, `source`, `severity`, `suggested_handling_state` и др.
- `risk_engine_rejects_unsourced_persona_observation` — пустой источник приводит к ошибке.

### search

- `search_index_returns_message_by_body_term` — `SearchIndex` индексирует документы и возвращает совпадения по полю `body`.
- `search_index_rejects_blank_required_document_fields` — обязательные поля `object_id`, `object_kind`, `title` не могут быть пустыми.
- `search_index_rejects_blank_query` — пустой поисковый запрос вызывает ошибку.
- `search_index_rejects_zero_limit` — лимит 0 вызывает ошибку.
- `search_index_replaces_existing_document_identity` — повторный `upsert_document` с тем же `object_id` заменяет содержимое; старые термины не находятся.
- `search_index_accepts_blank_body_for_title_only_documents` — допускается пустое тело, поиск ведётся только по заголовку.
- `search_index_distinguishes_delimiter_bearing_document_identities` — идентификаторы, содержащие разделители (двоеточия), корректно различаются.

### secret_vault

- `encrypted_vault_persists_secrets_without_plaintext_leakage` — `EncryptedSecretVault` шифрует секреты; файл не содержит открытого текста; `ResolvedSecret` не раскрывает значение в отладочном выводе.
- `encrypted_vault_rejects_wrong_master_key` — неверный мастер-ключ вызывает `SecretResolutionError::StoreFailure`.
- `database_encrypted_vault_persists_ciphertext_against_postgres` — `DatabaseEncryptedSecretVault` хранит только шифротекст в колонке `ciphertext`.
- `database_encrypted_vault_rejects_wrong_master_key_against_postgres` — неверный ключ базы данных приводит к ошибке.
- `host_vault_requires_entropy_threshold_before_create` — для создания `HostVault` требуется не менее 2000 событий энтропии.
- `host_vault_create_unlock_store_and_resolve_secret` — цикл: создание, сохранение, разрешение, блокировка, разблокировка, чтение.
- `host_vault_unlock_existing_reopens_session_after_runtime_restart` — `HostVault::unlock_existing` восстанавливает состояние после перезапуска.
- `host_vault_delete_removes_secret_and_manifest` — удаление секрета очищает манифест и возвращает ошибку при чтении.
- `host_vault_rejects_tampered_ciphertext` (обрезано) — вероятно, проверка целостности шифротекста.

### secrets

- `secret_reference_enums_reject_unsupported_values` — `SecretKind::try_from` и `SecretStoreKind::try_from` поддерживают только определённые строки.
- `in_memory_secret_resolver_resolves_test_double_references_without_debug_leaking_value` — `InMemorySecretResolver` возвращает секреты только для `TestDouble`; значение скрыто в отладке.
- `in_memory_secret_resolver_reports_missing_test_double_references` — отсутствующий секрет вызывает `SecretResolutionError::MissingSecret`.
- `in_memory_secret_resolver_rejects_non_test_double_store_kinds` — резольвер не обслуживает другие виды хранилищ.
- `resolved_secret_rejects_empty_values` — вставка пустого значения в резольвер возвращает `SecretResolutionError::EmptySecretValue`.
- `secret_references_store_only_metadata_against_postgres` — `SecretReferenceStore` сохраняет только метаданные ссылки на секрет, без самого секрета.

### settings

- `application_settings_store_lists_seeded_settings_against_postgres` — `list_settings` возвращает предустановленные ключи, не содержащие "password".
- `application_settings_include_frontend_layout_against_postgres` — настройка `frontend.layout` с category `frontend`, value_kind `Json`, содержит `schemaVersion: 2` и объект `views`.
- `application_settings_include_frontend_sidebar_against_postgres` — `frontend.sidebar` содержит группы, элементы, `rootItemIds`.
- `application_settings_include_frontend_theme_against_postgres` — `frontend.theme` определяет shellBackground, accentColor, panelOpacity и другие визуальные параметры.
- `application_settings_include_frontend_ui_state_against_postgres` — `frontend.ui_state` помечена `ui_control: "hidden"` и `stores_private_content: false`.
- `application_settings_update_repairs_missing_declared_setting_against_postgres` — при обновлении ранее удалённой настройки она восстанавливается (repair).
- `database_startup_repairs_declared_application_settings_against_postgres` — при инициализации базы данных недостающие настройки восстанавливаются, повреждённые значения корректируются, а кастомные (не объявленные) строки не затрагиваются.
```

## Покрытие источников

- `backend/tests/project_link_reviews.rs` (truncated) — в разделе «project_link_reviews» описаны тесты команды, материализации решения и отношения, сброса.
- `backend/tests/projection_runner.rs` — «projection_runner»: поведение пакетной обработки событий и обработка ошибок.
- `backend/tests/projects.rs` — «projects»: фильтрация по ключевым словам, исключение отклонённых сообщений, включение подтверждённых сообщений.
- `backend/tests/projects_api.rs` (truncated) — «projects_api»: проверка секрета, получение деталей проекта, кандидатов, обновление ревью.
- `backend/tests/relationships.rs` (truncated) — «relationships»: upsert связи с evidence, проекция в граф для персона-персона и кросс-доменных связей.
- `backend/tests/relationships_api.rs` — «relationships_api»: списочные эндпоинты, обновление ревью, обновление графа.
- `backend/tests/review_inbox.rs` (truncated) — «review_inbox»: создание элемента с evidence, фильтрация, полный жизненный цикл.
- `backend/tests/risk_engine.rs` — «risk_engine»: расчёт статуса внимания, валидация severity, формирование наблюдений.
- `backend/tests/search.rs` — «search»: crud индекса, валидация полей, замена документов.
- `backend/tests/secret_vault.rs` (truncated) — «secret_vault»: шифрованное хранение, устойчивость к неверному ключу, HostVault.
- `backend/tests/secrets.rs` — «secrets»: enum-валидация, InMemorySecretResolver, хранение метаданных.
- `backend/tests/settings.rs` (truncated) — «settings»: списки настроек, frontend-компоненты, repair-механизмы.

## Исходные файлы

- [`backend/tests/project_link_reviews.rs`](../../../../backend/tests/project_link_reviews.rs)
- [`backend/tests/projection_runner.rs`](../../../../backend/tests/projection_runner.rs)
- [`backend/tests/projects.rs`](../../../../backend/tests/projects.rs)
- [`backend/tests/projects_api.rs`](../../../../backend/tests/projects_api.rs)
- [`backend/tests/relationships.rs`](../../../../backend/tests/relationships.rs)
- [`backend/tests/relationships_api.rs`](../../../../backend/tests/relationships_api.rs)
- [`backend/tests/review_inbox.rs`](../../../../backend/tests/review_inbox.rs)
- [`backend/tests/risk_engine.rs`](../../../../backend/tests/risk_engine.rs)
- [`backend/tests/search.rs`](../../../../backend/tests/search.rs)
- [`backend/tests/secret_vault.rs`](../../../../backend/tests/secret_vault.rs)
- [`backend/tests/secrets.rs`](../../../../backend/tests/secrets.rs)
- [`backend/tests/settings.rs`](../../../../backend/tests/settings.rs)

## Кандидаты на drift

Никаких расхождений между кодом и документацией в данном контексте не выявлено — предоставленный контекст не включает ADR, спецификации или другие артефакты для сравнения.
