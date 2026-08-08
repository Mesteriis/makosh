# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `081-test-backend-part-004`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/tests/contradictions_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/contradictions_api.rs`
- Size bytes / Размер в байтах: `10241`
- Included characters / Включено символов: `10241`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::engines::consistency::{
    ContradictionObservation, ContradictionObservationStore, ContradictionReviewState,
    ContradictionSeverity, ContradictionSourceKind, NewContradictionObservation,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::consistency_review::sync_contradiction_review_item;

const LOCAL_API_TOKEN: &str = "contradictions-api-test-token";

#[tokio::test]
async fn contradictions_list_returns_open_reviewable_observations() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_contradiction_observation(&pool, suffix).await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/contradictions?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["observation_id"] == json!(stored.observation_id))
        .expect("seeded contradiction observation");

    assert_eq!(item["conflict_type"], "direct_contradiction");
    assert_eq!(item["old_claim"], stored.old_claim);
    assert_eq!(item["new_claim"], stored.new_claim);
    assert_eq!(item["review_state"], "suggested");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT item_kind, status, metadata->>'contradiction_observation_id'
        FROM review_items
        WHERE metadata->>'contradiction_observation_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("contradiction review item");
    assert_eq!(review_item.0, "contradiction_candidate");
    assert_eq!(review_item.1, "new");
    assert_eq!(review_item.2, stored.observation_id);

    let materialized_link: (String, String, Value, String) = sqlx::query_as(
        r#"
        SELECT
            link.observation_id,
            link.relationship_kind,
            link.metadata,
            kind.code
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'consistency'
          AND link.entity_kind = 'contradiction_observation'
          AND link.entity_id = $1
          AND link.relationship_kind = 'upsert'
        ORDER BY link.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("contradiction materialized link");
    assert!(materialized_link.0.starts_with("observation:v1:"));
    assert_eq!(materialized_link.1, "upsert");
    assert_eq!(
        materialized_link.2["conflict_type"],
        json!("direct_contradiction")
    );
    assert_eq!(materialized_link.3, "CONTRADICTION_OBSERVATION");
}

#[tokio::test]
async fn put_contradiction_review_updates_review_state_with_observation_trail() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_contradiction_observation(&pool, suffix).await;
    let observation_id = path_segment(&stored.observation_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/contradictions/{observation_id}/review"),
            json!({
                "review_state": "user_confirmed",
                "resolution": "confirmed from source review"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["observation_id"], stored.observation_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert_eq!(body["reviewed_by"], "makosh-frontend");
    assert_eq!(body["resolution"], "confirmed from source review");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'consistency'
           AND entity_kind = 'contradiction_observation'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("contradiction observation link");
    let review_observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(metadata["resolution"], "confirmed from source review");

    let row: (String, Option<String>, i64) = sqlx::query_as(
        r#"
        SELECT
            review_state,
            resolution,
            (
                SELECT count(*)
                FROM person_facts
                WHERE value = $2
            ) AS memory_overwrite_count
        FROM contradiction_observations
        WHERE observation_id = $1
        "#,
    )
    .bind(&stored.observation_id)
    .bind(&stored.new_claim)
    .fetch_one(&pool)
    .await
    .expect("stored contradiction review");

    assert_eq!(row.0, "user_confirmed");
    assert_eq!(row.1.as_deref(), Some("confirmed from source review"));
    assert_eq!(row.2, 0);

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&review_observation_id)
            .fetch_one(&pool)
            .await
            .expect("review observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(
        payload["contradiction_observation_id"],
        json!(stored.observation_id)
    );
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT item_kind, status, metadata->>'contradiction_observation_id'
        FROM review_items
        WHERE metadata->>'contradiction_observation_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("updated contradiction review item");
    assert_eq!(review_item.0, "contradiction_candidate");
    assert_eq!(review_item.1, "approved");
    assert_eq!(review_item.2, stored.observation_id);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    );

    (app, pool)
}

async fn seed_contradiction_observation(pool: &PgPool, suffix: u128) -> ContradictionObservation {
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: format!("memory:contradiction-api:{suffix}"),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: format!("message:contradiction-api:{suffix}"),
        affected_entities: json!([
            {"entity_kind": "persona", "entity_id": format!("person:v1:email:polygraph-{suffix}@example.com")}
        ]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "status=available".to_owned(),
        new_claim: format!("status=unavailable-{suffix}"),
        confidence: 0.86,
        severity: ContradictionSeverity::Medium,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({"source": "contradictions_api_test"}),
    };

    let stored = ContradictionObservationStore::new(pool.clone())
        .upsert(&observation)
        .await
        .expect("seed contradiction observation");
    sync_contradiction_review_item(pool, &stored)
        .await
        .expect("seed contradiction review item");
    stored
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
```

### `backend/tests/decision_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/decision_engine.rs`
- Size bytes / Размер в байтах: `3874`
- Included characters / Включено символов: `3874`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::decisions::{
    DecisionCandidateKind, DecisionEngine, DecisionEngineError, DecisionExtractionInput,
};
use makosh_hub_backend::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState,
};
use serde_json::json;

#[test]
fn decision_engine_detects_explicit_communication_decision_candidate() {
    let input = DecisionExtractionInput::communication(
        "message:decision-engine",
        "Decision: Use local-first storage because private context must work offline.",
        DecisionEntityKind::Project,
        "project:v1:makosh",
    )
    .decided_by(
        DecisionEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let result = DecisionEngine::detect_candidates(&input).expect("detect decisions");

    assert_eq!(result.decisions.len(), 1);
    let candidate = &result.decisions[0];
    assert_eq!(candidate.kind, DecisionCandidateKind::ExplicitDecision);
    assert_eq!(candidate.title, "Use local-first storage");
    assert_eq!(candidate.rationale, "private context must work offline");
    assert_eq!(
        candidate.quote,
        "Decision: Use local-first storage because private context must work offline."
    );
    assert_eq!(
        candidate.decided_by_entity_kind,
        Some(DecisionEntityKind::Persona)
    );
    assert_eq!(
        candidate.decided_by_entity_id.as_deref(),
        Some("person:v1:email:owner@example.com")
    );
    assert_eq!(candidate.confidence, 0.83);
    assert_eq!(candidate.review_state, DecisionReviewState::Suggested);
    assert_eq!(
        candidate.evidence_source_kind,
        DecisionEvidenceSourceKind::Communication
    );
    assert_eq!(candidate.evidence_source_id, "message:decision-engine");
    assert_eq!(candidate.impacted_entities.len(), 1);
    assert_eq!(
        candidate.impacted_entities[0].entity_kind,
        DecisionEntityKind::Project
    );
    assert_eq!(
        candidate.impacted_entities[0].entity_id,
        "project:v1:makosh"
    );

    let (decision, evidence, impacted_entities) = candidate.to_decision_draft();

    assert_eq!(decision.title, "Use local-first storage");
    assert_eq!(decision.rationale, "private context must work offline");
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(decision.confidence, 0.83);
    assert_eq!(
        decision.metadata,
        json!({
            "engine": "decision",
            "candidate_kind": "explicit_decision"
        })
    );
    assert_eq!(
        evidence.source_kind,
        DecisionEvidenceSourceKind::Communication
    );
    assert_eq!(evidence.source_id, "message:decision-engine");
    assert_eq!(
        evidence.quote.as_deref(),
        Some("Decision: Use local-first storage because private context must work offline.")
    );
    assert_eq!(evidence.confidence, 0.83);
    assert_eq!(impacted_entities.len(), 1);
    assert_eq!(impacted_entities[0].impact_type, "decision_context");
}

#[test]
fn decision_engine_ignores_non_decision_evidence() {
    let input = DecisionExtractionInput::document(
        "document:status-note",
        "The team discussed storage options but no decision was made.",
        DecisionEntityKind::Project,
        "project:v1:makosh",
    );

    let result = DecisionEngine::detect_candidates(&input).expect("detect decisions");

    assert!(result.decisions.is_empty());
}

#[test]
fn decision_engine_rejects_empty_source_evidence_before_detection() {
    let input = DecisionExtractionInput::communication(
        "message:empty-decision",
        " ",
        DecisionEntityKind::Project,
        "project:v1:makosh",
    );

    let error = DecisionEngine::detect_candidates(&input)
        .expect_err("empty evidence text must be rejected");

    assert!(matches!(error, DecisionEngineError::EmptyField("text")));
}
```

### `backend/tests/decisions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/decisions.rs`
- Size bytes / Размер в байтах: `24046`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{TimeZone, Utc};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStatus,
    DecisionStore, DecisionStoreError, NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::graph_projection::GraphProjectionService;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::test]
async fn decision_store_upserts_evidence_backed_decision_without_creating_work_against_postgres() {
    let Some((pool, decision_store)) = live_decision_context("decision upsert").await else {
        return;
    };
    let suffix = unique_suffix();
    let decided_by_persona_id = format!("person:v1:email:owner-{suffix}@example.com");
    let project_id = format!("project:v1:decision-{suffix}");
    let evidence_source_id = format!("meeting:decision:{suffix}");

    let decision = NewDecision::new(
        "Use local-first storage for client dossier",
        "Private communication context must remain available offline and under owner control.",
        0.9,
        DecisionReviewState::UserConfirmed,
    )
    .decided_by(DecisionEntityKind::Persona, decided_by_persona_id.clone())
    .decided_at(Utc.with_ymd_and_hms(2026, 6, 12, 10, 30, 0).unwrap())
    .alternatives(json!([
        "remote CRM-backed dossier",
        "provider-only search without local memory"
    ]))
    .metadata(json!({"source": "architecture-review", "scope": "dossier"}));
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, project_id.clone())
        .impact_type("architecture_direction")
        .metadata(json!({"component": "dossier"}));
    let first_evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Event,
        evidence_source_id.clone(),
    )
    .quote("We will use local-first storage for the client dossier.")
    .confidence(0.92)
    .metadata(json!({"meeting_section": "architecture", "revision": 1}));
    let second_evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Event,
        evidence_source_id.clone(),
    )
    .quote("Updated decision evidence for local-first dossier storage.")
    .confidence(0.94)
    .metadata(json!({"meeting_section": "architecture", "revision": 2}));

    let first = decision_store
        .upsert_with_evidence(
            &decision,
            std::slice::from_ref(&first_evidence),
            std::slice::from_ref(&impact),
        )
        .await
        .expect("first decision upsert");
    let second = decision_store
        .upsert_with_evidence(&decision, &[second_evidence], &[impact])
        .await
        .expect("idempotent decision upsert");

    assert_eq!(first.decision_id, second.decision_id);
    assert_eq!(first.title, "Use local-first storage for client dossier");
    assert_eq!(
        first.rationale,
        "Private communication context must remain available offline and under owner control."
    );
    assert_eq!(first.status, DecisionStatus::Active);
    assert_eq!(first.review_state, DecisionReviewState::UserConfirmed);
    assert_eq!(first.confidence, 0.9);
    assert_eq!(
        first.decided_by_entity_kind,
        Some(DecisionEntityKind::Persona)
    );
    assert_eq!(first.decided_by_entity_id, Some(decided_by_persona_id));
    assert_eq!(
        first.alternatives,
        json!([
            "remote CRM-backed dossier",
            "provider-only search without local memory"
        ])
    );

    let evidence_row = sqlx::query(
        r#"
        SELECT quote, confidence::float8 AS confidence, metadata
        FROM decision_evidence
        WHERE decision_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.decision_id)
    .bind(DecisionEvidenceSourceKind::Event.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored decision evidence");
    let quote: Option<String> = evidence_row.try_get("quote").expect("evidence quote");
    let confidence: f64 = evidence_row
        .try_get("confidence")
        .expect("evidence confidence");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(
        quote.as_deref(),
        Some("Updated decision evidence for local-first dossier storage.")
    );
    assert_eq!(confidence, 0.94);
    assert_eq!(
        metadata,
        json!({"meeting_section": "architecture", "revision": 2})
    );

    let impact_row = sqlx::query(
        r#"
        SELECT impact_type, metadata
        FROM decision_impacted_entities
        WHERE decision_id = $1
          AND entity_kind = $2
          AND entity_id = $3
        "#,
    )
    .bind(&first.decision_id)
    .bind(DecisionEntityKind::Project.as_str())
    .bind(&project_id)
    .fetch_one(&pool)
    .await
    .expect("stored impacted entity");
    let impact_type: String = impact_row.try_get("impact_type").expect("impact type");
    let impact_metadata: Value = impact_row.try_get("metadata").expect("impact metadata");
    assert_eq!(impact_type, "architecture_direction");
    assert_eq!(impact_metadata, json!({"component": "dossier"}));

    let project_decisions = decision_store
        .list_for_entity(DecisionEntityKind::Project, &project_id, 10)
        .await
        .expect("project decisions");
    assert!(
        project_decisions
            .iter()
            .any(|item| item.decision_id == first.decision_id)
    );

    GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await
        .expect("project decision graph");

    let decision_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'decision' AND stable_key = $1",
    )
    .bind(&first.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision graph node");
    let project_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'project' AND stable_key = $1",
    )
    .bind(&project_id)
    .fetch_one(&pool)
    .await
    .expect("project graph node");
    let graph_edge_row = sqlx::query(
        r#"
        SELECT edge_id, relationship_type, confidence::float8 AS confidence, review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(&decision_node_id)
    .bind(&project_node_id)
    .fetch_one(&pool)
    .await
    .expect("decision graph edge");
    let graph_edge_id: String = graph_edge_row.try_get("edge_id").expect("edge id");
    let graph_relationship_type: String = graph_edge_row
        .try_get("relationship_type")
        .expect("relationship type");
    let graph_confidence: f64 = graph_edge_row
        .try_get("confidence")
        .expect("graph confidence");
    let graph_review_state: String = graph_edge_row
        .try_get("review_state")
        .expect("graph review state");
    let graph_properties: Value = graph_edge_row
        .try_get("properties")
        .expect("graph properties");

    assert_eq!(graph_relationship_type, "entity_relationship");
    assert_eq!(graph_confidence, 0.9);
    assert_eq!(graph_review_state, "user_confirmed");
    assert_eq!(
        graph_properties,
        json!({
            "domain": "decision",
            "decision_id": first.decision_id,
            "impact_type": "architecture_direction"
        })
    );

    let graph_evidence_row = sqlx::query(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM graph_evidence
        WHERE edge_id = $1
        "#,
    )
    .bind(&graph_edge_id)
    .fetch_one(&pool)
    .await
    .expect("decision graph evidence");
    let graph_source_kind: String = graph_evidence_row
        .try_get("source_kind")
        .expect("graph evidence source kind");
    let graph_source_id: String = graph_evidence_row
        .try_get("source_id")
        .expect("graph evidence source id");
    let graph_excerpt: Option<String> = graph_evidence_row
        .try_get("excerpt")
        .expect("graph evidence excerpt");
    let graph_evidence_metadata: Value = graph_evidence_row
        .try_get("metadata")
        .expect("graph evidence metadata");

    assert_eq!(graph_source_kind, "decision");
    assert_eq!(graph_source_id, first.decision_id);
    assert_eq!(
        graph_excerpt.as_deref(),
        Some("Updated decision evidence for local-first dossier storage.")
    );
    assert_eq!(
        graph_evidence_metadata,
        json!({
            "domain": "decision",
            "source_kind": "event",
            "source_id": evidence_source_id
        })
    );

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&evidence_source_id)
            .fetch_one(&pool)
            .await
            .expect("task count for decision source");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE metadata @> $1")
            .bind(json!({"decision_source_id": evidence_source_id}))
            .fetch_one(&pool)
            .await
            .expect("obligation count for decision source");

    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn decision_store_refresh_persists_explicit_message_decision_candidate_against_postgres() {
    let Some((pool, decision_store)) = live_decision_context("decision candidate refresh").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let title = format!("Use persona dossiers {suffix}");
    let rationale = "relationship context must survive channel changes";
    let quote = format!("Decision: {title} because {rationale}.");
    let message_id = seed_decision_message(
        &pool,
        suffix,
        &format!("decision-candidate-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-decision-candidate-{suffix}"),
        &format!("Decision candidate {suffix}"),
        &quote,
    )
    .await;

    let refreshed = decision_store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh decision candidates");
    assert!(refreshed >= 1);

    let decisions = decision_store
        .list_for_entity(DecisionEntityKind::Communication, &message_id, 10)
        .await
        .expect("communication decisions");
    let decision = decisions
        .iter()
        .find(|item| item.title == title)
        .expect("refreshed decision candidate");

    assert_eq!(decision.rationale, rationale);
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(decision.confidence, 0.83);

    let evidence_row: (String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, quote
        FROM decision_evidence
        WHERE decision_id = $1
        "#,
    )
    .bind(&decision.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence_row.0, "communication");
    assert_eq!(evidence_row.1, message_id);
    assert_eq!(evidence_row.2.as_deref(), Some(quote.as_str()));
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/decisions_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/decisions_api.rs`
- Size bytes / Размер в байтах: `10284`
- Included characters / Включено символов: `10284`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::decisions::{
    Decision, DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore,
    NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "decisions-api-test-token";

#[tokio::test]
async fn decisions_list_returns_entity_scoped_decisions() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let project_id = format!("project:v1:decision-api-{suffix}");
    let stored = seed_decision(&pool, suffix, &project_id).await;

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/decisions?entity_kind=project&entity_id={project_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["decision_id"] == json!(stored.decision_id))
        .expect("seeded decision");

    assert_eq!(item["title"], stored.title);
    assert_eq!(item["status"], "active");
    assert_eq!(item["review_state"], "suggested");
    assert_eq!(item["decided_by_entity_kind"], "persona");
}

#[tokio::test]
async fn decisions_list_returns_global_suggested_review_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested_project_id = format!("project:v1:decision-global-suggested-{suffix}");
    let confirmed_project_id = format!("project:v1:decision-global-confirmed-{suffix}");
    let suggested = seed_decision_with_review_state(
        &pool,
        suffix,
        &suggested_project_id,
        DecisionReviewState::Suggested,
    )
    .await;
    let confirmed = seed_decision_with_review_state(
        &pool,
        suffix + 1,
        &confirmed_project_id,
        DecisionReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/decisions?review_state=suggested&limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(
        items
            .iter()
            .any(|item| item["decision_id"] == json!(suggested.decision_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["decision_id"] != json!(confirmed.decision_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_decision_review_updates_review_state_with_observation_trail() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let project_id = format!("project:v1:decision-review-{suffix}");
    let stored = seed_decision(&pool, suffix, &project_id).await;
    let decision_id = path_segment(&stored.decision_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/decisions/{decision_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["decision_id"], stored.decision_id);
    assert_eq!(body["review_state"], "user_confirmed");

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(&stored.decision_id)
            .fetch_one(&pool)
            .await
            .expect("stored review state");
    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'decisions'
           AND entity_kind = 'decision'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&stored.decision_id)
            .fetch_one(&pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE metadata @> $1")
            .bind(json!({"decision_id": stored.decision_id}))
            .fetch_one(&pool)
            .await
            .expect("obligation count");

    assert_eq!(review_state, "user_confirmed");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("decision observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["decision_id"], json!(stored.decision_id));
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'decision_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "decision");
    assert_eq!(review_item.2, stored.decision_id);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    );

    (app, pool)
}

async fn seed_decision(pool: &PgPool, suffix: u128, project_id: &str) -> Decision {
    seed_decision_with_review_state(pool, suffix, project_id, DecisionReviewState::Suggested).await
}

async fn seed_decision_with_review_state(
    pool: &PgPool,
    suffix: u128,
    project_id: &str,
    review_state: DecisionReviewState,
) -> Decision {
    let decision = NewDecision::new(
        format!("Adopt decision API route {suffix}"),
        "Accepted decisions need a guarded backend review surface.",
        0.84,
        review_state,
    )
    .decided_by(
        DecisionEntityKind::Persona,
        format!("person:v1:email:decision-api-{suffix}@example.com"),
    )
    .decided_at(Utc.with_ymd_and_hms(2026, 6, 12, 11, 0, 0).unwrap())
    .alternatives(json!([
        "store decisions only in meeting outcomes",
        "hide decisions in project notes"
    ]))
    .metadata(json!({"source": "decisions_api_test"}));
    let evidence = NewDecisionEvidence::new(
        DecisionEvidenceSourceKind::Event,
        format!("meeting:decision-api:{suffix}"),
    )
    .quote("We decided to expose accepted decisions through guarded backend routes.")
    .confidence(0.91)
    .metadata(json!({"source": "decisions_api_test"}));
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Project, project_id)
        .impact_type("architecture_direction")
        .metadata(json!({"source": "decisions_api_test"}));

    DecisionStore::new(pool.clone())
        .upsert_with_evidence(&decision, &[evidence], &[impact])
        .await
        .expect("seed decision")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
```

### `backend/tests/document_processing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/document_processing.rs`
- Size bytes / Размер в байтах: `172`
- Included characters / Включено символов: `172`
- Truncated / Обрезано: `no`

```rust
#[path = "document_processing/enqueue_run.rs"]
mod enqueue_run;
#[path = "document_processing/retry.rs"]
mod retry;
#[path = "document_processing/support.rs"]
mod support;
```

### `backend/tests/document_processing/enqueue_run.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/document_processing/enqueue_run.rs`
- Size bytes / Размер в байтах: `6836`
- Included characters / Включено символов: `6836`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;
use testkit::context::TestContext;

#[tokio::test]
async fn enqueue_for_document_creates_extract_text_and_ocr_jobs() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("enqueue both processing jobs").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_enqueue_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "pipeline.md",
            "# Draft\n\nRun processing queue",
        ))
        .await
        .expect("import markdown document");

    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");

    assert_eq!(jobs.len(), 2);
    assert!(
        jobs.iter()
            .any(|job| step_name(&job.step) == "extract_text")
    );
    assert!(jobs.iter().any(|job| step_name(&job.step) == "ocr"));
    let observation_links: i64 = query_scalar::<_, i64>(
        "SELECT count(*) FROM observation_links WHERE domain = 'documents' AND entity_kind = 'document_processing_job' AND entity_id = ANY($1)",
    )
    .bind(jobs.iter().map(|job| job.job_id.clone()).collect::<Vec<_>>())
    .fetch_one(&pool)
    .await
    .expect("document processing observation links");
    assert!(observation_links >= 2);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn enqueue_for_document_does_not_reset_terminal_jobs() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("terminal job reset protection").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_terminal_{suffix}");

    document_store
        .import_document(&NewDocumentImport::pdf_metadata(
            &document_id,
            "pipeline.pdf",
            "sha256:processing-terminal",
        ))
        .await
        .expect("import pdf document");

    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs");
    assert_eq!(report.jobs_skipped, 2);
    let terminal_state_before = terminal_state_for_document(&pool, &document_id).await;

    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs again");
    let terminal_state_after = terminal_state_for_document(&pool, &document_id).await;

    assert_eq!(terminal_state_before, terminal_state_after);
}

#[tokio::test]
async fn run_queued_jobs_for_markdown_populates_extracted_text_artifact() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("markdown run generates artifact").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_run_markdown_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "notes.md",
            "First line\n\nExtracted body text.",
        ))
        .await
        .expect("import markdown document");
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs");

    assert_eq!(report.jobs_queued, 2);
    let extract_status: String = query_scalar::<_, String>(
        "SELECT status FROM document_processing_jobs WHERE document_id = $1 AND step = 'extract_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("extract status");
    let artifact_count: i64 = query_scalar::<_, i64>(
        "SELECT count(*) FROM document_artifacts WHERE document_id = $1 AND artifact_kind = 'extracted_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("artifact count");

    assert_eq!(extract_status, "succeeded");
    assert_eq!(artifact_count, 1);
    let extract_job_id: String = query_scalar::<_, String>(
        "SELECT job_id FROM document_processing_jobs WHERE document_id = $1 AND step = 'extract_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("extract job id");
    let status_observations: i64 = query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'documents'
          AND link.entity_kind = 'document_processing_job'
          AND link.entity_id = $1
          AND kind.code = 'DOCUMENT_PROCESSING_JOB_STATUS'
        "#,
    )
    .bind(&extract_job_id)
    .fetch_one(&pool)
    .await
    .expect("extract status observations");
    assert!(status_observations >= 2);
}

#[tokio::test]
async fn run_queued_jobs_skips_non_markdown_text_extraction_with_summary() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("non-markdown extract skip").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;

    let suffix = unique_suffix();
    let document_id = format!("doc_processing_non_markdown_{suffix}");

    document_store
        .import_document(&NewDocumentImport::pdf_metadata(
            &document_id,
            "scan.pdf",
            "sha256:processing-non-markdown",
        ))
        .await
        .expect("import pdf document");
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs");

    let summary: Option<String> = query_scalar::<_, Option<String>>(
        "SELECT last_error_summary FROM document_processing_jobs WHERE document_id = $1 AND step = 'extract_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("extract skip summary");

    assert!(matches!(summary, Some(value) if !value.is_empty()));
}
```

### `backend/tests/document_processing/retry.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/document_processing/retry.rs`
- Size bytes / Размер в байтах: `11131`
- Included characters / Включено символов: `11131`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;
use testkit::context::TestContext;

#[tokio::test]
async fn document_processing_retry_failed_job_requeues_job_against_postgres() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("retry failed processing job").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry.md",
            "# Retry\n\nProcessing retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");
    let extract_job = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job");

    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .execute(&pool)
    .await
    .expect("mark extract job failed");

    let command_id = format!("document-processing-retry-{suffix}");
    let result = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: command_id.clone(),
            job_id: extract_job.job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect("retry failed job");

    assert_eq!(result.job_id, extract_job.job_id);
    assert_eq!(result.status, DocumentProcessingStatus::Queued);
    assert_eq!(
        result.event_id,
        format!("document_processing_retry:{command_id}")
    );

    let persisted = sqlx::query_as::<_, (String, i32, Option<String>)>(
        r#"
        SELECT status, attempts, last_error_summary
        FROM document_processing_jobs
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("persisted retried job");

    assert_eq!(persisted.0, "queued");
    assert_eq!(persisted.1, 0);
    assert_eq!(persisted.2, None);
    let requeue_observations: i64 = query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'documents'
          AND link.entity_kind = 'document_processing_job'
          AND link.entity_id = $1
          AND kind.code = 'DOCUMENT_PROCESSING_JOB_STATUS'
          AND link.relationship_kind = 'requeued'
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("requeue observations");
    assert!(requeue_observations >= 1);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn run_queued_jobs_requires_retry_command_for_failed_jobs() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("failed job requires retry command").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_runner_{suffix}");
    let job_id =
        create_failed_extract_text_job(&pool, &document_store, &processing_store, &document_id)
            .await;
    quiesce_document_processing_jobs_except(&pool, &document_id, &job_id).await;

    let skipped_report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs without retry command");
    let failed_state = job_retry_state(&pool, &job_id).await;
    let artifact_count_before_retry = extracted_text_artifact_count(&pool, &document_id).await;

    assert_eq!(skipped_report.jobs_seen, 0);
    assert_eq!(skipped_report.jobs_queued, 0);
    assert_eq!(failed_state.0, "failed");
    assert_eq!(failed_state.1, 2);
    assert!(failed_state.2.is_some());
    assert_eq!(artifact_count_before_retry, 0);

    processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-runner-{suffix}"),
            job_id: job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect("retry failed job");
    let retried_report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run retried job");
    let retried_state = job_retry_state(&pool, &job_id).await;
    let artifact_count_after_retry = extracted_text_artifact_count(&pool, &document_id).await;

    assert_eq!(retried_report.jobs_seen, 1);
    assert_eq!(retried_report.jobs_queued, 1);
    assert_eq!(retried_report.jobs_succeeded, 1);
    assert_eq!(retried_state.0, "succeeded");
    assert_eq!(artifact_count_after_retry, 1);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_duplicate_same_command_is_idempotent() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("duplicate retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_idempotent_{suffix}");
    let job_id =
        create_failed_extract_text_job(&pool, &document_store, &processing_store, &document_id)
            .await;
    let command = DocumentProcessingRetryCommand {
        command_id: format!("document-processing-retry-idempotent-{suffix}"),
        job_id: job_id.clone(),
        actor_id: "document-processing-test-actor".to_owned(),
    };

    let first = processing_store
        .retry_failed_job(&command)
        .await
        .expect("first retry succeeds");
    let second = processing_store
        .retry_failed_job(&command)
        .await
        .expect("duplicate retry is idempotent");

    assert_eq!(first, second);
    assert_eq!(second.job_id, job_id);
    assert_eq!(second.status, DocumentProcessingStatus::Queued);
    assert_eq!(
        second.event_id,
        format!("document_processing_retry:{}", command.command_id)
    );
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_duplicate_command_for_different_job_is_rejected() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("duplicate retry command collision").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let existing_document_id = format!("doc_processing_retry_collision_existing_{suffix}");
    let target_document_id = format!("doc_processing_retry_collision_target_{suffix}");
    let existing_job_id = create_failed_extract_text_job(
        &pool,
        &document_store,
        &processing_store,
        &existing_document_id,
    )
    .await;
    let target_job_id = create_failed_extract_text_job(
        &pool,
        &document_store,
        &processing_store,
        &target_document_id,
    )
    .await;
    let command_id = format!("document-processing-retry-collision-{suffix}");
    append_retry_event_for_job(&pool, &command_id, &existing_job_id).await;

    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id,
            job_id: target_job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("command collision must be rejected");

    assert!(matches!(
        error,
        DocumentProcessingError::RetryCommandConflict
    ));
    let persisted = job_retry_state(&pool, &target_job_id).await;
    assert_eq!(persisted.0, "failed");
    assert_eq!(persisted.1, 2);
    assert!(persisted.2.is_some());

    quiesce_processing_jobs_for_document(&pool, &existing_document_id).await;
    quiesce_processing_jobs_for_document(&pool, &target_document_id).await;
}

#[tokio::test]
async fn document_processing_retry_non_failed_job_requires_failed_status() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("non-failed retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_non_failed_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry-non-failed.md",
            "# Retry\n\nQueued retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");
    let extract_job = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job");

    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-non-failed-{suffix}"),
            job_id: extract_job.job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("queued job retry must be rejected");

    assert!(matches!(
        error,
        DocumentProcessingError::RetryRequiresFailedJob
    ));
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_missing_job_returns_job_not_found() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((_pool, _document_store, processing_store)) =
        live_context("missing retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-missing-{suffix}"),
            job_id: format!("document_processing_job:v1:missing-{suffix}:extract_text"),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("missing job retry must be rejected");

    assert!(matches!(error, DocumentProcessingError::JobNotFound));
}
```

### `backend/tests/document_processing/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/document_processing/support.rs`
- Size bytes / Размер в байтах: `7092`
- Included characters / Включено символов: `7092`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

pub(crate) use chrono::Utc;
pub(crate) use makosh_hub_backend::domains::documents::core::{
    DocumentImportStore, NewDocumentImport,
};
pub(crate) use makosh_hub_backend::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingRetryCommand, DocumentProcessingStatus,
    DocumentProcessingStore,
};
pub(crate) use makosh_hub_backend::platform::events::{EventStore, NewEventEnvelope};
pub(crate) use makosh_hub_backend::platform::storage::Database;
pub(crate) use serde_json::json;
pub(crate) use sqlx::postgres::PgPool;
pub(crate) use sqlx::query_scalar;

pub(crate) async fn live_context(
    _test_name: &str,
) -> Option<(PgPool, DocumentImportStore, DocumentProcessingStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((
        pool.clone(),
        DocumentImportStore::new(pool.clone()),
        DocumentProcessingStore::new(pool),
    ))
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub(crate) fn step_name(
    step: &makosh_hub_backend::domains::documents::processing::DocumentProcessingStep,
) -> &'static str {
    match step {
        makosh_hub_backend::domains::documents::processing::DocumentProcessingStep::ExtractText => {
            "extract_text"
        }
        makosh_hub_backend::domains::documents::processing::DocumentProcessingStep::Ocr => "ocr",
    }
}

pub(crate) async fn terminal_state_for_document(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) -> Vec<(String, String, i32)> {
    sqlx::query_as::<_, (String, String, i32)>(
        "SELECT step, status, attempts FROM document_processing_jobs WHERE document_id = $1 ORDER BY step",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await
    .expect("terminal state")
}

pub(crate) async fn create_failed_extract_text_job(
    pool: &sqlx::postgres::PgPool,
    document_store: &DocumentImportStore,
    processing_store: &DocumentProcessingStore,
    document_id: &str,
) -> String {
    document_store
        .import_document(&NewDocumentImport::markdown(
            document_id,
            "retry-collision.md",
            "# Retry\n\nProcessing retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(document_id)
        .await
        .expect("enqueue processing jobs");
    let job_id = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job")
        .job_id
        .clone();

    fail_processing_job(pool, &job_id).await;

    job_id
}

pub(crate) async fn fail_processing_job(pool: &sqlx::postgres::PgPool, job_id: &str) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .execute(pool)
    .await
    .expect("mark extract job failed");
}

pub(crate) async fn append_retry_event_for_job(
    pool: &sqlx::postgres::PgPool,
    command_id: &str,
    job_id: &str,
) {
    let event = NewEventEnvelope::builder(
        format!("document_processing_retry:{command_id}"),
        "document_processing.retry_requested",
        Utc::now(),
        json!({
            "kind": "document_processing_retry",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({
            "kind": "document_processing_job",
            "job_id": job_id,
        }),
    )
    .actor(json!({ "actor_id": "document-processing-test-actor" }))
    .payload(json!({ "job_id": job_id }))
    .build()
    .expect("retry event envelope");

    EventStore::new(pool.clone())
        .append(&event)
        .await
        .expect("append retry collision event");
}

pub(crate) async fn job_retry_state(
    pool: &sqlx::postgres::PgPool,
    job_id: &str,
) -> (String, i32, Option<String>) {
    sqlx::query_as::<_, (String, i32, Option<String>)>(
        r#"
        SELECT status, attempts, last_error_summary
        FROM document_processing_jobs
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .expect("job retry state")
}

pub(crate) async fn extracted_text_artifact_count(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) -> i64 {
    query_scalar::<_, i64>(
        "SELECT count(*) FROM document_artifacts WHERE document_id = $1 AND artifact_kind = 'extracted_text'",
    )
    .bind(document_id)
    .fetch_one(pool)
    .await
    .expect("extracted text artifact count")
}

pub(crate) async fn quiesce_document_processing_jobs_except(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
    active_job_id: &str,
) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id = $1
          AND job_id <> $2
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .bind(document_id)
    .bind(active_job_id)
    .execute(pool)
    .await
    .expect("quiesce non-target document processing jobs");
}

pub(crate) async fn quiesce_retryable_test_processing_jobs(pool: &sqlx::postgres::PgPool) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id LIKE 'doc_processing_%'
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .execute(pool)
    .await
    .expect("quiesce retryable test processing jobs");
}

pub(crate) async fn quiesce_processing_jobs_for_document(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id = $1
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .bind(document_id)
    .execute(pool)
    .await
    .expect("quiesce document processing jobs for test document");
}
```

### `backend/tests/document_processing_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/document_processing_api.rs`
- Size bytes / Размер в байтах: `18927`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::documents::processing::DocumentProcessingStore;
use makosh_hub_backend::platform::events::{EventStore, NewEventEnvelope};
use makosh_hub_backend::platform::storage::Database;
use serde_json::Value;
use sqlx::query_scalar;
use tower::ServiceExt;

const LOCAL_API_TOKEN: &str = "document-processing-api-test-token";

#[tokio::test]
async fn get_document_processing_jobs_rejects_missing_local_api_secret() {
    let app =
        makosh_hub_backend::app::build_router(testkit::app::config_with_secret(LOCAL_API_TOKEN));

    let response = app
        .oneshot(get_request("/api/v1/document-processing/jobs"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn get_document_processing_for_missing_document_returns_404() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );
    let missing_document_id = format!("doc_processing_api_missing_{:x}", unique_suffix());

    let response = app
        .oneshot(get_request_with_actor(
            &format!("/api/v1/documents/{missing_document_id}/processing"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "document_processing_store_error",
            "message": "document processing job was not found"
        })
    );
}

#[tokio::test]
async fn document_processing_api_returns_expected_payloads() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let document_id = format!("doc_processing_api_{:x}", unique_suffix());

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "pipeline api doc",
            "# Pipeline API\nMarkdown body for API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let jobs_response = app
        .clone()
        .oneshot(get_request_with_actor(
            "/api/v1/document-processing/jobs?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(jobs_response.status(), StatusCode::OK);
    let jobs_body = json_body(jobs_response).await;
    let items = jobs_body["items"]
        .as_array()
        .expect("document processing jobs");
    assert!(!items.is_empty());
    let has_target = items
        .iter()
        .any(|item| item["document_id"] == Value::String(document_id.clone()));
    assert!(has_target, "jobs should include enqueued document");

    let detail_response = app
        .oneshot(get_request_with_actor(
            &format!("/api/v1/documents/{document_id}/processing"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(
        detail_body["document_id"],
        Value::String(document_id.clone())
    );
    assert!(detail_body["jobs"].is_array());
    assert!(detail_body["jobs"].as_array().unwrap().len() >= 2);

    let _ = query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM document_processing_jobs WHERE document_id = $1)",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("jobs inserted for processing document");
}

#[tokio::test]
async fn post_document_processing_job_retry_requeues_failed_job() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_api_retry_{suffix:x}");

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry api doc",
            "# Retry API\nMarkdown body for retry API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let extract_job = jobs
        .iter()
        .find(|job| {
            job.step == makosh_hub_backend::domains::documents::processing::DocumentProcessingStep::ExtractText
        })
        .expect("extract text job");

    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .execute(&pool)
    .await
    .expect("mark extract job failed");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );
    let command_id = format!("document-processing-retry-{suffix:x}");
    let retry_path = format!(
        "/api/v1/document-processing/jobs/{}/retry",
        extract_job.job_id
    );
    let request_body = serde_json::json!({ "command_id": command_id });

    let retry_response = app
        .oneshot(post_json_request(
            &retry_path,
            LOCAL_API_TOKEN,
            request_body.clone(),
        ))
        .await
        .expect("retry response");
    assert_eq!(retry_response.status(), StatusCode::OK);
    let retry_body = json_body(retry_response).await;
    assert_eq!(
        retry_body,
        serde_json::json!({
            "job_id": extract_job.job_id,
            "status": "queued",
            "event_id": format!("document_processing_retry:{}", request_body["command_id"].as_str().unwrap())
        })
    );
    let audit_record =
        sqlx::query_as::<_, (String, String, String, String, String, Option<String>)>(
            r#"
            SELECT operation, actor_id, method, path_template, target_kind, target_id
            FROM api_audit_log
            WHERE target_kind = 'document_processing_job'
              AND target_id = $1
            ORDER BY audit_id ASC
            "#,
        )
        .bind(&extract_job.job_id)
        .fetch_one(&pool)
        .await
        .expect("document processing retry audit record");
    assert_eq!(audit_record.0, "document_processing.job.retry");
    assert_eq!(audit_record.1, "makosh-frontend");
    assert_eq!(audit_record.2, "POST");
    assert_eq!(
        audit_record.3,
        "/api/v1/document-processing/jobs/{job_id}/retry"
    );
    assert_eq!(audit_record.4, "document_processing_job");
    assert_eq!(audit_record.5.as_deref(), Some(extract_job.job_id.as_str()));

    let retry_observation_link_count: i64 = query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'documents'
           AND entity_kind = 'document_processing_job'
           AND entity_id = $1
           AND relationship_kind = 'retry_command'",
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("document processing retry observation link count");
    assert_eq!(retry_observation_link_count, 1);
    let retry_observation_kind: String = query_scalar(
        r#"
        SELECT kinds.code
        FROM observation_links links
        JOIN observations observation
          ON observation.observation_id = links.observation_id
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE links.domain = 'documents'
          AND links.entity_kind = 'document_processing_job'
          AND links.entity_id = $1
          AND links.relationship_kind = 'retry_command'
        ORDER BY links.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("document processing retry observation kind");
    assert_eq!(retry_observation_kind, "DOCUMENT_PROCESSING_JOB_STATUS");
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn post_document_processing_job_retry_rejects_non_failed_job_with_stable_body() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_api_retry_non_failed_{suffix:x}");

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry api non failed doc",
            "# Retry API\nMarkdown body for non-failed retry API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let extract_job = jobs
        .iter()
        .find(|job| {
            job.step == makosh_hub_backend::domains::documents::processing::DocumentProcessingStep::ExtractText
        })
        .expect("extract text job");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );
    let retry_path = format!(
        "/api/v1/document-processing/jobs/{}/retry",
        extract_job.job_id
    );

    let response = app
        .oneshot(post_json_request(
            &retry_path,
            LOCAL_API_TOKEN,
            serde_json::json!({
                "command_id": format!("document-processing-retry-non-failed-{suffix:x}")
            }),
        ))
        .await
        .expect("retry response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "document_processing_store_error",

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/document_processing_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/document_processing_architecture.rs`
- Size bytes / Размер в байтах: `2042`
- Included characters / Включено символов: `2042`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn document_processing_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_document_processing_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "document processing test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_document_processing_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_document_processing_test_violations(&path, violations);
            continue;
        }
        if !is_document_processing_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_document_processing_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "document_processing.rs" || file_name == "document_processing_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "document_processing")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/documents.rs`
- Size bytes / Размер в байтах: `11773`
- Included characters / Включено символов: `11773`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use makosh_hub_backend::domains::documents::core::{
    DocumentImportError, DocumentImportStore, NewDocumentImport,
};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn document_import_stores_markdown_text_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = DocumentImportStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_markdown_{suffix}"),
            "notes.md",
            "# Notes\n\nBudget review notes.",
        ))
        .await
        .expect("import markdown");

    assert_eq!(imported.document_kind, "markdown");
    assert_eq!(imported.title, "notes.md");
    assert_eq!(imported.extracted_text, "Notes\n\nBudget review notes.");

    let observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'documents'
          AND entity_kind = 'document'
          AND entity_id = $2
          AND relationship_kind = 'import'
        "#,
    )
    .bind(&imported.observation_id)
    .bind(&imported.document_id)
    .fetch_one(database.pool().expect("configured pool"))
    .await
    .expect("document import observation links");
    assert_eq!(observation_link_count, 1);
}

#[tokio::test]
async fn document_import_stores_pdf_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = DocumentImportStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::pdf_metadata(
            format!("doc_pdf_{suffix}"),
            "contract.pdf",
            "sha256:contract",
        ))
        .await
        .expect("import pdf metadata");

    assert_eq!(imported.document_kind, "pdf");
    assert_eq!(imported.title, "contract.pdf");
    assert_eq!(imported.extracted_text, "");
    assert_eq!(imported.source_fingerprint, "sha256:contract");
}

#[tokio::test]
async fn document_import_rejects_blank_required_fields() {
    let store = disconnected_document_store();

    for (field_name, document) in [
        (
            "document_id",
            NewDocumentImport {
                document_id: "   ".to_owned(),
                document_kind: "markdown".to_owned(),
                title: "notes.md".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "Notes".to_owned(),
            },
        ),
        (
            "document_kind",
            NewDocumentImport {
                document_id: "doc_blank_kind".to_owned(),
                document_kind: "   ".to_owned(),
                title: "notes.md".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "Notes".to_owned(),
            },
        ),
        (
            "title",
            NewDocumentImport {
                document_id: "doc_blank_title".to_owned(),
                document_kind: "markdown".to_owned(),
                title: "   ".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "Notes".to_owned(),
            },
        ),
        (
            "source_fingerprint",
            NewDocumentImport {
                document_id: "doc_blank_fingerprint".to_owned(),
                document_kind: "pdf".to_owned(),
                title: "contract.pdf".to_owned(),
                source_fingerprint: "   ".to_owned(),
                extracted_text: String::new(),
            },
        ),
        (
            "extracted_text",
            NewDocumentImport {
                document_id: "doc_blank_extracted_text".to_owned(),
                document_kind: "markdown".to_owned(),
                title: "notes.md".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "   ".to_owned(),
            },
        ),
    ] {
        let error = store
            .import_document(&document)
            .await
            .expect_err("blank document field must fail");

        assert!(
            matches!(error, DocumentImportError::EmptyField(actual) if actual == field_name),
            "expected EmptyField({field_name}), got {error:?}"
        );
    }
}

#[tokio::test]
async fn document_import_rejects_invalid_kind() {
    let store = disconnected_document_store();
    let document = NewDocumentImport {
        document_id: "doc_invalid_kind".to_owned(),
        document_kind: "docx".to_owned(),
        title: "notes.docx".to_owned(),
        source_fingerprint: "sha256:notes".to_owned(),
        extracted_text: "Notes".to_owned(),
    };

    let error = store
        .import_document(&document)
        .await
        .expect_err("invalid document kind must fail");

    assert!(
        matches!(error, DocumentImportError::InvalidDocumentKind(ref kind) if kind == "docx"),
        "expected InvalidDocumentKind(docx), got {error:?}"
    );
}

#[test]
fn markdown_import_helper_derives_deterministic_local_fingerprint() {
    let first = NewDocumentImport::markdown("doc_fingerprint", "notes.md", "# Notes\n\nBody.");
    let second = NewDocumentImport::markdown("doc_fingerprint", "notes.md", "# Notes\n\nBody.");

    assert_eq!(first.source_fingerprint, second.source_fingerprint);
    assert_eq!(first.extracted_text, "Notes\n\nBody.");
    assert!(first.source_fingerprint.starts_with("local-v1:markdown:"));
}

#[tokio::test]
async fn document_import_extracts_multiple_markdown_heading_levels_against_postgres() {
    let Some(store) = live_document_store("markdown heading extraction").await else {
        return;
    };
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_headings_{suffix}"),
            "headings.md",
            "# Title\n\n## Section\n\n### Detail\n\nBody.   \n",
        ))
        .await
        .expect("import markdown headings");

    assert_eq!(
        imported.extracted_text,
        "Title\n\nSection\n\nDetail\n\nBody."
    );
    assert!(
        imported
            .source_fingerprint
            .starts_with("local-v1:markdown:")
    );
}

#[tokio::test]
async fn document_import_preserves_hash_prefixed_non_headings_against_postgres() {
    let Some(store) = live_document_store("markdown non-heading hash-prefixed lines").await else {
        return;
    };
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_hash_prefixed_{suffix}"),
            "hash-prefixed.md",
            "# Heading\n\n#hashtag\n#include <x>\n###not heading\n####### Too many hashes",
        ))
        .await
        .expect("import markdown hash-prefixed lines");

    assert_eq!(
        imported.extracted_text,
        "Heading\n\n#hashtag\n#include <x>\n###not heading\n####### Too many hashes"
    );
}

#[tokio::test]
async fn document_import_reimports_same_kind_idempotently_against_postgres() {
    let Some((pool, store)) = live_document_context("same-kind idempotent document import").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_idempotent_{suffix}");

    let first = store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            "draft.md",
            "# Draft\n\nInitial text.",
        ))
        .await
        .expect("first import");
    let updated = store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            "draft-renamed.md",
            "# Draft\n\nUpdated text.",
        ))
        .await
        .expect("second import");

    assert_eq!(updated.document_id, document_id);
    assert_eq!(updated.document_kind, "markdown");
    assert_eq!(updated.title, "draft-renamed.md");
    assert_ne!(updated.source_fingerprint, first.source_fingerprint);
    assert_eq!(updated.extracted_text, "Draft\n\nUpdated text.");
    assert_eq!(updated.imported_at, first.imported_at);

    let count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM documents WHERE document_id = $1")
            .bind(&updated.document_id)
            .fetch_one(&pool)
            .await
            .expect("idempotent document count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn document_import_rejects_existing_document_kind_change_against_postgres() {
    let Some((pool, store)) = live_document_context("document kind change rejection").await else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_kind_change_{suffix}");

    let first = store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            "notes.md",
            "# Notes\n\nInitial text.",
        ))
        .await
        .expect("first import");

    let error = store
        .import_document(&NewDocumentImport::pdf_metadata(
            document_id.clone(),
            "notes.pdf",
            "sha256:notes-pdf",
        ))
        .await
        .expect_err("document kind changes must fail");

    assert!(
        matches!(
            error,
            DocumentImportError::DocumentKindChange {
                ref document_id,
                ref existing_kind,
                ref new_kind,
            } if document_id == &first.document_id
                && existing_kind == "markdown"
                && new_kind == "pdf"
        ),
        "expected DocumentKindChange, got {error:?}"
    );

    let stored = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT document_kind, title, source_fingerprint, extracted_text
        FROM documents
        WHERE document_id = $1
        "#,
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("stored document after rejected kind change");

    assert_eq!(stored.0, "markdown");
    assert_eq!(stored.1, "notes.md");
    assert_eq!(stored.2, first.source_fingerprint);
    assert_eq!(stored.3, "Notes\n\nInitial text.");
}

async fn live_document_context(
    _test_name: &str,
) -> Option<(sqlx::postgres::PgPool, DocumentImportStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((pool.clone(), DocumentImportStore::new(pool)))
}

async fn live_document_store(test_name: &str) -> Option<DocumentImportStore> {
    live_document_context(test_name)
        .await
        .map(|(_, store)| store)
}

fn disconnected_document_store() -> DocumentImportStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    DocumentImportStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/email_account_management_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_management_api.rs`
- Size bytes / Размер в байтах: `14379`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, NewRawCommunicationRecord, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::domains::signal_hub::SignalHubStore;
use makosh_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use sqlx::Row;
use testkit::context::TestContext;

const TOKEN: &str = "mail-account-management-test-token";

async fn app(ctx: &TestContext) -> axum::Router {
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(TOKEN, ctx.connection_string().as_str()),
        database,
    )
}

fn request(method: Method, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TOKEN);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json")
}

#[tokio::test]
async fn email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account() {
    let ctx = TestContext::new().await;
    let store = CommunicationIngestionStore::new(ctx.pool().clone());
    let secret_store = SecretReferenceStore::new(ctx.pool().clone());
    store
        .upsert_provider_account(
            &NewProviderAccount::new(
                "fastmail-primary",
                EmailProviderKind::Imap,
                "Fastmail",
                "alex@example.com",
            )
            .config(json!({
                "host": "imap.fastmail.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "alex@example.com",
                "smtp_host": "smtp.fastmail.com",
                "smtp_port": 587,
                "smtp_tls": true,
                "smtp_starttls": true,
                "provider_preset": "fastmail"
            })),
        )
        .await
        .expect("account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            "secret:fastmail-primary:imap-password",
            SecretKind::AppPassword,
            SecretStoreKind::TestDouble,
            "Fastmail app password",
        ))
        .await
        .expect("secret reference");
    store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            "fastmail-primary",
            ProviderAccountSecretPurpose::ImapPassword,
            "secret:fastmail-primary:imap-password",
        ))
        .await
        .expect("bind provider secret");
    SignalHubStore::new(ctx.pool().clone())
        .restore_system_sources()
        .await
        .expect("restore signal hub sources");

    let app = app(&ctx).await;

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/integrations/mail/accounts",
            None,
        ))
        .await
        .expect("list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(
        body["items"][0]["account"]["account_id"],
        "fastmail-primary"
    );
    assert_eq!(body["items"][0]["capabilities"]["send"], true);
    assert_eq!(body["items"][0]["capabilities"]["local_trash"], true);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/integrations/mail/accounts/fastmail-primary",
            None,
        ))
        .await
        .expect("get response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["external_account_id"], "alex@example.com");

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/integrations/mail/accounts/fastmail-primary/export",
            None,
        ))
        .await
        .expect("export response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["account_id"], "fastmail-primary");
    let serialized = body.to_string();
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("secret_ref"));
    assert!(!serialized.contains("token"));

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/integrations/mail/accounts/fastmail-primary/logout",
            None,
        ))
        .await
        .expect("logout response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["config"]["auth_state"], "logged_out");
    assert_eq!(body["sync_settings"]["sync_enabled"], false);
    let logged_out_signal_status: String = sqlx::query_scalar(
        r#"
        SELECT status
        FROM signal_connections
        WHERE source_code = 'mail'
          AND settings->>'account_id' = 'fastmail-primary'
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .fetch_one(ctx.pool())
    .await
    .expect("logged out mail signal status");
    assert_eq!(logged_out_signal_status, "disconnected");

    let logout_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code, link.relationship_kind, observation.payload
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'vault'
           AND link.entity_kind = 'communication_provider_account'
           AND link.entity_id = 'fastmail-primary'
           AND link.relationship_kind = 'config_update'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .fetch_one(ctx.pool())
    .await
    .expect("logout config observation");
    assert_eq!(
        logout_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        logout_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION"
    );
    let logout_payload: Value = logout_observation.try_get("payload").expect("payload");
    assert_eq!(logout_payload["action"], "logout");
    assert_eq!(logout_payload["account_id"], "fastmail-primary");

    let response = app
        .clone()
        .oneshot(request(
            Method::DELETE,
            "/api/v1/integrations/mail/accounts/fastmail-primary",
            None,
        ))
        .await
        .expect("delete response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["deleted"], true);
    assert_eq!(
        body["unbound_secret_refs"],
        json!(["secret:fastmail-primary:imap-password"])
    );
    let removed_signal_status: String = sqlx::query_scalar(
        r#"
        SELECT status
        FROM signal_connections
        WHERE source_code = 'mail'
          AND settings->>'account_id' = 'fastmail-primary'
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .fetch_one(ctx.pool())
    .await
    .expect("removed mail signal status");
    assert_eq!(removed_signal_status, "removed");

    let account_delete_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code, link.relationship_kind
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'vault'
           AND link.entity_kind = 'communication_provider_account'
           AND link.entity_id = 'fastmail-primary'
           AND link.relationship_kind = 'delete'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .fetch_one(ctx.pool())
    .await
    .expect("provider account delete observation");
    assert_eq!(
        account_delete_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        account_delete_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT_DELETED"
    );

    let binding_remove_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code, link.relationship_kind
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'vault'
           AND link.entity_kind = 'communication_provider_secret_binding'
           AND link.entity_id = 'fastmail-primary:imap_password'
           AND link.relationship_kind = 'remove'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .fetch_one(ctx.pool())
    .await
    .expect("provider secret binding removal observation");
    assert_eq!(
        binding_remove_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        binding_remove_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED"
    );

    let response = app
        .oneshot(request(
            Method::GET,
            "/api/v1/integrations/mail/accounts/fastmail-primary",
            None,
        ))
        .await
        .expect("get deleted response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn email_account_delete_rejects_accounts_with_retained_raw_records() {
    let ctx = TestContext::new().await;
    let store = CommunicationIngestionStore::new(ctx.pool().clone());
    store
        .upsert_provider_account(&NewProviderAccount::new(
            "imap-with-evidence",
            EmailProviderKind::Imap,
            "Evidence IMAP",
            "evidence@example.com",
        ))
        .await
        .expect("account");
    store
        .record_raw_source(&NewRawCommunicationRecord::new(
            "raw:mail-account-delete",
            "imap-with-evidence",
            "email",
            "provider:1",
            "sha256:test",
            "batch:test",
            json!({}),
        ))
        .await
        .expect("raw record");

    let app = app(&ctx).await;
    let response = app
        .oneshot(request(
            Method::DELETE,
            "/api/v1/integrations/mail/accounts/imap-with-evidence",
            None,
        ))
        .await
        .expect("delete response");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["er
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_account_setup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup.rs`
- Size bytes / Размер в байтах: `436`
- Included characters / Включено символов: `436`
- Truncated / Обрезано: `no`

```rust
#[path = "email_account_setup/config.rs"]
mod config;
#[path = "email_account_setup/gmail_api.rs"]
mod gmail_api;
#[path = "email_account_setup/gmail_service.rs"]
mod gmail_service;
#[path = "email_account_setup/imap_api.rs"]
mod imap_api;
#[path = "email_account_setup/send_api.rs"]
mod send_api;
#[path = "email_account_setup/support.rs"]
mod support;
#[path = "email_account_setup/vault_reconciliation.rs"]
mod vault_reconciliation;
```

### `backend/tests/email_account_setup/config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/config.rs`
- Size bytes / Размер в байтах: `2884`
- Included characters / Включено символов: `2884`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::integrations::mail::accounts::GmailOAuthSetupRequest;
use makosh_hub_backend::platform::config::{AppConfig, GoogleOAuthClientType};

#[test]
fn gmail_oauth_setup_defaults_to_mail_send_calendar_and_contacts_scopes() {
    let request = GmailOAuthSetupRequest::new(
        "acct_google_workspace",
        "Google Workspace",
        "",
        "desktop-client-id",
        "http://127.0.0.1:18088/oauth/callback",
    );

    assert_eq!(
        request.scopes,
        [
            "https://www.googleapis.com/auth/gmail.readonly",
            "https://www.googleapis.com/auth/gmail.send",
            "https://www.googleapis.com/auth/calendar.readonly",
            "https://www.googleapis.com/auth/contacts.readonly",
        ]
    );
}

#[test]
fn app_config_accepts_google_oauth_client_credentials() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_GOOGLE_OAUTH_CLIENT_ID", "google-client-id"),
        ("MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET", "google-client-secret"),
    ])
    .expect("config");

    assert_eq!(config.google_oauth_client_id(), Some("google-client-id"));
    assert_eq!(
        config
            .google_oauth_client_secret()
            .expect("google client secret")
            .expose_for_runtime(),
        "google-client-secret"
    );
}

#[test]
fn app_config_accepts_google_oauth_installed_client_json() {
    let config = AppConfig::from_pairs([(
        "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
        r#"{
            "installed": {
                "client_id": "desktop-client-id.apps.googleusercontent.com",
                "project_id": "makosh-local",
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token",
                "client_secret": "desktop-client-secret",
                "redirect_uris": ["http://localhost"]
            }
        }"#,
    )])
    .expect("config");

    let google_client = config
        .google_oauth_client()
        .expect("google oauth client config");
    assert_eq!(
        google_client.client_type(),
        GoogleOAuthClientType::Installed
    );
    assert_eq!(
        google_client.client_id(),
        "desktop-client-id.apps.googleusercontent.com"
    );
    assert_eq!(
        google_client
            .client_secret()
            .expect("desktop client secret")
            .expose_for_runtime(),
        "desktop-client-secret"
    );
    assert_eq!(
        google_client.authorization_endpoint(),
        "https://accounts.google.com/o/oauth2/auth"
    );
    assert_eq!(
        google_client.token_endpoint(),
        "https://oauth2.googleapis.com/token"
    );
    assert_eq!(google_client.redirect_uris(), ["http://localhost"]);
    assert_eq!(
        config.google_oauth_client_id(),
        Some("desktop-client-id.apps.googleusercontent.com")
    );
}
```

### `backend/tests/email_account_setup/gmail_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/gmail_api.rs`
- Size bytes / Размер в байтах: `23325`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::calendar::events::CalendarAccountStore;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::platform::secrets::{
    SecretReferenceStore, SecretResolver, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{HostVault, HostVaultConfig};
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, MockTokenServer, get_request, json_body, json_request_with_token_and_actor,
    text_body, unique_suffix, unlock_test_vault,
};

#[tokio::test]
async fn gmail_oauth_start_api_uses_configured_google_desktop_client_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let vault_dir = tempdir().expect("vault tempdir");
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
                (
                    "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                    r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
                ),
            ])
            .expect("config"),
        database.clone(),
    );
    unlock_test_vault(app.clone()).await;

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let authorization_url = body["authorization_url"]
        .as_str()
        .expect("authorization url");
    assert!(authorization_url.starts_with("https://accounts.google.com/o/oauth2/auth?"));
    assert!(authorization_url.contains("client_id=desktop-client-id.apps.googleusercontent.com"));
    assert!(authorization_url.contains("gmail.readonly"));
    assert!(authorization_url.contains("gmail.send"));
    assert!(authorization_url.contains("calendar.readonly"));
    assert!(authorization_url.contains("contacts.readonly"));

    drop(database);
}

#[tokio::test]
async fn gmail_oauth_start_api_requires_initialized_host_vault_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
                (
                    "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                    r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
                ),
            ])
            .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "host_vault_error");
    assert_eq!(body["message"], "host vault is not initialized");
}

#[tokio::test]
async fn gmail_oauth_start_api_reopens_initialized_host_vault_after_restart_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let initialized_app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );
    unlock_test_vault(initialized_app).await;

    let restarted_app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
                (
                    "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                    r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
                ),
            ])
            .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );

    let response = restarted_app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let authorization_url = body["authorization_url"]
        .as_str()
        .expect("authorization url");
    assert!(authorization_url.starts_with("https://accounts.google.com/o/oauth2/auth?"));
    assert!(authorization_url.contains("client_id=desktop-client-id.apps.googleusercontent.com"));
}

#[tokio::test]
async fn gmail_oauth_callback_completes_pending_grant_without_api_secret() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config"),
        database,
    );
    unlock_test_vault(app.clone()).await;

    let token_server = MockTokenServer::start();
    let suffix = unique_suffix();
    let account_id = format!("gmail-callback-{suffix}");
    let start_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Google Workspace",
                "client_id": "desktop-client-id",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback",
                "app_return_url": "http://127.0.0.1:5174/?makosh_oauth=gmail_connected",
                "authorization_endpoint": format!("{}/authorize", token_server.base_url()),
                "token_endpoint": format!("{}/token", token_server.base_url())
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    let state = start_body["state"].as_str().expect("state");

    let callback_response = app
        .oneshot(get_request(&format!(
            "/api/v1/integrations/mail/accounts/gmail/oauth/callback?code=authorization-code&state={state}"
        )))
        .await
        .expect("callback response");

    assert_eq!(callback_response.status(), StatusCode::OK);
    let callback_body = text_body(callback_response).await;
    assert!(callback_body.contains("Goog
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
