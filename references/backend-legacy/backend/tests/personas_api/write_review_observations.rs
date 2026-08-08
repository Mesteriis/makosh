use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::enrichment_engine::EnrichmentResultStore;
use makosh_hub_backend::domains::personas::identity::store::PersonaIdentityReviewStore;
use makosh_hub_backend::platform::storage::database::Database;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app_with_database, delete_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn persona_enrichment_review_entrypoints_capture_observations_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person enrichment review observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&format!("manual-enrichment-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let enrichment = EnrichmentResultStore::new(pool.clone())
        .upsert(
            &person.persona_id,
            "linkedin",
            json!({
                "extracted_claim": "Works on canonical evidence architecture"
            }),
            0.88,
        )
        .await
        .expect("create enrichment result");
    let app = build_personas_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.persona_id);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{encoded_person_id}/enrichment/{}/apply",
                urlencoding_percent_encode(&enrichment.id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("enrichment apply response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let apply_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'enrichment_result'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&enrichment.id)
    .fetch_one(&pool)
    .await
    .expect("enrichment apply observation link count");
    assert_eq!(apply_link_count, 1);

    let enrichment_rejected = EnrichmentResultStore::new(pool.clone())
        .upsert(
            &person.persona_id,
            "telegram",
            json!({
                "extracted_claim": "Prefers async communication"
            }),
            0.74,
        )
        .await
        .expect("create enrichment reject result");

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{encoded_person_id}/enrichment/{}/reject",
                urlencoding_percent_encode(&enrichment_rejected.id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("enrichment reject response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let reject_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'enrichment_result'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&enrichment_rejected.id)
    .fetch_one(&pool)
    .await
    .expect("enrichment reject observation link count");
    assert_eq!(reject_link_count, 1);
}

#[tokio::test]
async fn persona_entrypoints_capture_observations_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("persona entrypoint observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&format!("manual-compat-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let app = build_personas_app_with_database(&database_url, database);
    let encoded_persona_id = urlencoding_percent_encode(&person.persona_id);

    let role_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_persona_id}/roles"),
            json!({"role": "colleague"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("role response");
    assert_eq!(role_response.status(), axum::http::StatusCode::OK);
    let role_id = json_body(role_response).await["id"]
        .as_str()
        .expect("role id")
        .to_owned();
    let role_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'role'
           AND entity_id = $1
           AND metadata ->> 'action' = 'assign'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&role_id)
    .fetch_one(&pool)
    .await
    .expect("role observation link");

    let persona_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_persona_id}/interaction-contexts"),
            json!({
                "persona_id": format!("pers:v1:manual:{suffix}"),
                "name": "Work Persona",
                "context": "Professional context",
                "preferred_channel": "email"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona response");
    assert_eq!(persona_response.status(), axum::http::StatusCode::OK);
    let interaction_context_id = json_body(persona_response).await["interaction_context_id"]
        .as_str()
        .expect("interaction context id")
        .to_owned();
    let persona_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'persona'
           AND entity_id = $1
           AND metadata ->> 'action' = 'upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&interaction_context_id)
    .fetch_one(&pool)
    .await
    .expect("persona observation link");
    let persona_pref_source: String = sqlx::query_scalar(
        "SELECT source
         FROM persona_preferences
         WHERE persona_id = $1
           AND preference_type = $2",
    )
    .bind(&person.persona_id)
    .bind(format!(
        "interaction_context:{interaction_context_id}:preferred_channel"
    ))
    .fetch_one(&pool)
    .await
    .expect("persona preference source");
    assert!(persona_pref_source.starts_with("observation:"));

    let delete_role_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/personas/{}/roles/colleague",
                urlencoding_percent_encode(&person.persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete role response");
    assert_eq!(delete_role_response.status(), axum::http::StatusCode::OK);
    let delete_role_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'role'
           AND entity_id = $1
           AND metadata ->> 'action' = 'delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!("{}:{}", person.persona_id, "colleague"))
    .fetch_one(&pool)
    .await
    .expect("role delete observation link");

    let delete_persona_response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/personas/{}/interaction-contexts/{}",
                urlencoding_percent_encode(&person.persona_id),
                urlencoding_percent_encode(&interaction_context_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete persona response");
    assert_eq!(delete_persona_response.status(), axum::http::StatusCode::OK);
    let delete_persona_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'persona'
           AND entity_id = $1
           AND metadata ->> 'action' = 'delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&interaction_context_id)
    .fetch_one(&pool)
    .await
    .expect("persona delete observation link");

    for observation_id in [
        role_observation_id,
        persona_observation_id,
        persona_pref_source
            .strip_prefix("observation:")
            .expect("persona pref observation prefix")
            .to_owned(),
        delete_role_observation_id,
        delete_persona_observation_id,
    ] {
        let row = sqlx::query(
            "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(&observation_id)
        .fetch_one(&pool)
        .await
        .expect("stored observation");
        assert_eq!(
            row.try_get::<String, _>("origin_kind")
                .expect("origin kind"),
            "manual"
        );
        assert_eq!(
            row.try_get::<String, _>("kind_code").expect("kind code"),
            "PERSONA_RECORD_MUTATION"
        );
    }
}

#[tokio::test]
async fn identity_candidate_review_captures_observation_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("identity candidate review observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person_store = PersonaProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Review Observation {suffix}");
    let left = person_store
        .upsert_email_persona(&format!("identity-review-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_persona(&format!("identity-review-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    sqlx::query(
        r#"
        UPDATE personas
        SET display_name = $1
        WHERE persona_id = $2 OR persona_id = $3
        "#,
    )
    .bind(&shared_name)
    .bind(&left.persona_id)
    .bind(&right.persona_id)
    .execute(&pool)
    .await
    .expect("seed display names");

    let _ = PersonaIdentityReviewStore::new(pool.clone())
        .refresh_candidates(100)
        .await
        .expect("refresh identity candidates");
    let (left_persona_id, right_persona_id) = if left.persona_id <= right.persona_id {
        (left.persona_id.clone(), right.persona_id.clone())
    } else {
        (right.persona_id.clone(), left.persona_id.clone())
    };
    let identity_candidate_id =
        format!("identity_candidate:v1:merge_personas:{left_persona_id}:{right_persona_id}");

    let app = build_personas_app_with_database(&database_url, database);
    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/identity-candidates/{}/review",
                urlencoding_percent_encode(&identity_candidate_id)
            ),
            json!({
                "command_id": format!("identity-review-command-{suffix}"),
                "review_state": "user_rejected"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["identity_candidate_id"], json!(identity_candidate_id));
    assert_eq!(body["review_state"], "user_rejected");

    let review_state: String = sqlx::query_scalar(
        "SELECT review_state FROM persona_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("candidate review state");
    assert_eq!(review_state, "user_rejected");

    let row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'identity_candidate'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate observation link");
    let observation_id: String = row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = row.try_get("metadata").expect("observation metadata");
    assert_eq!(metadata["review_state"], "user_rejected");

    let stored =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("stored observation");
    assert_eq!(
        stored
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    let payload: serde_json::Value = stored.try_get("payload").expect("observation payload");
    assert_eq!(
        payload["identity_candidate_id"],
        json!(identity_candidate_id)
    );
    assert_eq!(payload["review_state"], "user_rejected");
}
