use crate::support::*;
use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::domains::signal_hub::store::SignalHubStore;
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};

#[tokio::test]
async fn ai_answer_api_returns_source_backed_answer_and_persists_run() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let person_store = PersonaProjectionStore::new(pool.clone());
    let owner = person_store
        .upsert_email_persona(&format!("ai-owner-{suffix}@example.com"))
        .await
        .expect("owner persona candidate");
    let owner = person_store
        .set_owner_persona(&owner.persona_id)
        .await
        .expect("set owner persona");
    let retrieval_token = format!("V3AIAnswer{suffix}");
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("ai-answer-{suffix}@example.com"),
        &[format!("ai-recipient-{suffix}@example.com")],
        &format!("provider-ai-answer-{suffix}"),
        &format!("Макошь AI roadmap {retrieval_token}"),
        &format!("The V3 AI plan for {retrieval_token} uses Ollama and source-backed citations."),
    )
    .await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str()),
            ("MAKOSH_OLLAMA_CHAT_MODEL", "qwen3:4b"),
            ("MAKOSH_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/answers",
            json!({
                "command_id": format!("answer-{suffix}"),
                "query": format!("V3 AI plan for {retrieval_token}"),
                "agent_id": "MNEMOSYNE"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::ACCEPTED, "body={body}");
    assert_eq!(body["status"], json!("accepted"));

    let run_id = body["run_id"].as_str().expect("run id");
    let stored = wait_for_run_status(&pool, run_id, "completed").await;
    assert_eq!(stored.agent_id, "MNEMOSYNE");
    assert_eq!(
        stored.agent_persona_id.as_deref(),
        Some("persona:v1:ai_agent:MNEMOSYNE")
    );
    assert_eq!(
        stored.owner_persona_id.as_deref(),
        Some(owner.persona_id.as_str())
    );
    assert_eq!(stored.chat_model, "qwen3:4b");
    assert_eq!(stored.embedding_model, "qwen3-embedding:4b");
    assert_eq!(
        stored.answer.as_deref(),
        Some("Макошь V3 is source-backed.")
    );
    assert_eq!(stored.status, "completed");
    assert!(stored.duration_ms.expect("duration") >= 0);
    let citations = stored.citations.as_array().expect("citations");
    assert!(!citations.is_empty());
    assert!(citations.iter().any(|citation| {
        citation["source_kind"] == json!("message") && citation["source_id"] == json!(message_id)
    }));

    let run_observations: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'agent_run'
          AND link.entity_id = $1
          AND kind.code IN ('AI_AGENT_RUN', 'AI_AGENT_RUN_STATUS')
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("run observations");
    assert!(
        run_observations >= 2,
        "expected run requested + completed observations"
    );

    let run_event_count = wait_for_event_types(
        &pool,
        run_id,
        run_id,
        &["ai.run.requested", "ai.run.completed"],
    )
    .await;
    assert_eq!(run_event_count, 2);

    let hub_event_count = wait_for_event_types(
        &pool,
        run_id,
        run_id,
        &["ai.hub.requested", "ai.hub.completed"],
    )
    .await;
    assert_eq!(hub_event_count, 2);

    let run_attribution = sqlx::query(
        r#"
        SELECT agent_persona_id, owner_persona_id
        FROM ai_agent_runs
        WHERE run_id = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("run attribution");
    assert_eq!(
        run_attribution
            .try_get::<Option<String>, _>("agent_persona_id")
            .unwrap()
            .as_deref(),
        Some("persona:v1:ai_agent:MNEMOSYNE")
    );
    assert_eq!(
        run_attribution
            .try_get::<Option<String>, _>("owner_persona_id")
            .unwrap()
            .as_deref(),
        Some(owner.persona_id.as_str())
    );
}

#[tokio::test]
async fn ai_answer_api_is_blocked_when_ai_source_is_muted_by_signal_hub() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    SignalHubStore::new(pool.clone())
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("ai".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "mute AI while debugging signal controls".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create ai mute policy");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str()),
            ("MAKOSH_OLLAMA_CHAT_MODEL", "qwen3:4b"),
            ("MAKOSH_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/answers",
            json!({
                "command_id": format!("answer-blocked-{suffix}"),
                "query": format!("Blocked AI query {suffix}"),
                "agent_id": "MNEMOSYNE"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED, "body={body}");
    assert_eq!(body["error"], json!("failed_precondition"));
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|message| message.contains("AI runtime is disabled by Signal Hub policy"))
    );

    let run_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM ai_agent_runs")
        .fetch_one(&pool)
        .await
        .expect("run count");
    assert_eq!(run_count, 0);
}

#[tokio::test]
async fn ai_answer_missing_route_becomes_failed_run_instead_of_sync_api_error() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama setting");

    let suffix = unique_suffix();
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str())])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/answers",
            json!({
                "command_id": format!("answer-missing-route-{suffix}"),
                "query": format!("Missing route query {suffix}"),
                "agent_id": "MNEMOSYNE"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::ACCEPTED, "body={body}");
    let run_id = body["run_id"].as_str().expect("run id");

    let stored = wait_for_run_status(&pool, run_id, "failed").await;
    let error_summary = stored.error_summary.expect("error summary");
    assert!(
        error_summary.contains("route_not_configured"),
        "unexpected error summary: {error_summary}"
    );

    let hub_failed_count = wait_for_event_types(
        &pool,
        run_id,
        run_id,
        &["ai.hub.requested", "ai.hub.failed"],
    )
    .await;
    assert_eq!(hub_failed_count, 2);
}

#[tokio::test]
async fn ai_task_refresh_creates_suggested_candidates_without_active_tasks() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("ai-task-{suffix}@example.com"),
        &[format!("ai-task-recipient-{suffix}@example.com")],
        &format!("provider-ai-task-{suffix}"),
        "AI task source",
        &format!("Please review the V3 implementation checklist {suffix}."),
    )
    .await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str())])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/task-candidates/refresh",
            json!({
                "command_id": format!("task-refresh-{suffix}"),
                "query": format!("Please review the V3 implementation checklist {suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::ACCEPTED, "body={body}");
    assert_eq!(body["status"], json!("accepted"));
    let run_id = body["run_id"].as_str().expect("run id");
    let stored = wait_for_run_status(&pool, run_id, "completed").await;
    assert_eq!(
        stored.answer.as_deref(),
        Some("Created 1 suggested task candidate(s).")
    );

    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let candidate = sqlx::query(
        r#"
        SELECT task_candidate_id, review_state, agent_run_id, observation_id
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("candidate");
    assert_eq!(candidate.get::<String, _>("review_state"), "suggested");
    assert!(candidate.get::<Option<String>, _>("agent_run_id").is_some());
    assert_eq!(
        candidate
            .get::<Option<String>, _>("observation_id")
            .as_deref(),
        Some(message_observation_id.as_str())
    );

    let active_task_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE source_id = $1")
            .bind(&message_observation_id)
            .fetch_one(&pool)
            .await
            .expect("active task count");
    assert_eq!(active_task_count, 0);

    let review_item_row = sqlx::query(
        r#"
        SELECT item_kind, status, metadata
        FROM review_items
        WHERE review_item_id IN (
            SELECT review_item_id
            FROM review_item_evidence
            WHERE observation_id = $1
        )
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("review item row");
    assert_eq!(
        review_item_row.get::<String, _>("item_kind"),
        "potential_task"
    );
    assert_eq!(review_item_row.get::<String, _>("status"), "new");
    let metadata = review_item_row.get::<serde_json::Value, _>("metadata");
    assert_eq!(metadata["mirrored_from"], json!("task_candidates"));

    let run_event_count = wait_for_event_types(
        &pool,
        run_id,
        run_id,
        &[
            "ai.run.requested",
            "ai.run.completed",
            "ai.task_extraction.completed",
        ],
    )
    .await;
    assert_eq!(run_event_count, 3);
}
