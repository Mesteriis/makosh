use crate::support::*;
use makosh_backend_testkit::context::TestContext;

#[tokio::test]
async fn ai_meeting_prep_returns_briefing_without_calendar_dependency() {
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
    let project_id = format!("project:v1:ai-meeting:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("AI Meeting Project {suffix}"),
                "Product Development",
                "Meeting prep project",
                "Alex Morgan",
                vec![format!("MeetingPrep{suffix}")],
            )
            .progress(42),
        )
        .await
        .expect("project");
    seed_document(
        &pool,
        &format!("ai_meeting_doc_{suffix}"),
        &format!("MeetingPrep{suffix} notes"),
        "Discuss V3 AI risks and validation.",
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
            "/api/v1/ai/meeting-prep",
            json!({
                "command_id": format!("meeting-prep-{suffix}"),
                "topic": "V3 AI implementation review",
                "project_id": project_id
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
    assert_eq!(stored.agent_id, "HESTIA");
    assert_eq!(
        stored.answer.as_deref(),
        Some("Discuss V3 risks and validation evidence.")
    );
    assert!(!stored.citations.as_array().expect("citations").is_empty());

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
}

#[tokio::test]
async fn ai_status_and_agents_are_protected() {
    let app = build_router(config_with_api_token());

    let missing_token = app
        .clone()
        .oneshot(get_request("/api/v1/ai/status"))
        .await
        .expect("response");
    assert_eq!(missing_token.status(), StatusCode::FORBIDDEN);

    let agents = app
        .oneshot(get_request_with_token("/api/v1/ai/agents", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(agents.status(), StatusCode::OK);
    let body = json_body(agents).await;
    let items = body["items"].as_array().expect("agents");
    assert_eq!(items.len(), 5);
    assert_eq!(items[0]["agent_id"], json!("HESTIA"));
    assert_eq!(items[0]["display_name"], json!("hestia@sh-inc.ru"));
    assert!(
        items
            .iter()
            .any(|item| item["agent_id"] == json!("HEPHAESTUS")),
        "HEPHAESTUS must be part of the initial AI agent registry"
    );
}

#[tokio::test]
async fn ai_agents_api_materializes_agent_personas_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token("/api/v1/ai/agents", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("agents");
    let hestia = items
        .iter()
        .find(|item| item["agent_id"] == "HESTIA")
        .expect("HESTIA descriptor");
    assert_eq!(hestia["persona_id"], "persona:v1:ai_agent:HESTIA");
    assert_eq!(hestia["persona_type"], "ai_agent");
    assert_eq!(hestia["persona_email"], "hestia@sh-inc.ru");
    let hephaestus = items
        .iter()
        .find(|item| item["agent_id"] == "HEPHAESTUS")
        .expect("HEPHAESTUS descriptor");
    assert_eq!(hephaestus["persona_id"], "persona:v1:ai_agent:HEPHAESTUS");
    assert_eq!(hephaestus["persona_type"], "ai_agent");
    assert_eq!(hephaestus["persona_email"], "hephaestus@sh-inc.ru");

    let row = sqlx::query(
        r#"
        SELECT display_name, person_type, email_address, is_self
        FROM personas
        WHERE persona_id = 'persona:v1:ai_agent:HESTIA'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HESTIA Persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "hestia@sh-inc.ru"
    );
    assert_eq!(row.try_get::<String, _>("person_type").unwrap(), "ai_agent");
    assert_eq!(
        row.try_get::<String, _>("email_address").unwrap(),
        "hestia@sh-inc.ru"
    );
    assert!(!row.try_get::<bool, _>("is_self").unwrap());

    let identity_value: String = sqlx::query_scalar(
        r#"
        SELECT identity_value
        FROM persona_identities
        WHERE persona_id = 'persona:v1:ai_agent:HESTIA'
          AND identity_type = 'email'
          AND source = 'ai_agent_registry'
          AND status = 'active'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HESTIA email identity");
    assert_eq!(identity_value, "hestia@sh-inc.ru");

    let graph_row = sqlx::query(
        r#"
        SELECT label, properties
        FROM graph_nodes
        WHERE node_kind = 'persona'
          AND stable_key = 'persona:v1:ai_agent:HESTIA'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HESTIA graph node");
    assert_eq!(
        graph_row.try_get::<String, _>("label").unwrap(),
        "hestia@sh-inc.ru"
    );
    let properties = graph_row
        .try_get::<serde_json::Value, _>("properties")
        .unwrap();
    assert_eq!(properties["persona_type"], "ai_agent");
    assert_eq!(properties["agent_id"], "HESTIA");

    let row = sqlx::query(
        r#"
        SELECT display_name, person_type, email_address, is_self
        FROM personas
        WHERE persona_id = 'persona:v1:ai_agent:HEPHAESTUS'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HEPHAESTUS Persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "hephaestus@sh-inc.ru"
    );
    assert_eq!(row.try_get::<String, _>("person_type").unwrap(), "ai_agent");
    assert_eq!(
        row.try_get::<String, _>("email_address").unwrap(),
        "hephaestus@sh-inc.ru"
    );
    assert!(!row.try_get::<bool, _>("is_self").unwrap());

    let identity_value: String = sqlx::query_scalar(
        r#"
        SELECT identity_value
        FROM persona_identities
        WHERE persona_id = 'persona:v1:ai_agent:HEPHAESTUS'
          AND identity_type = 'email'
          AND source = 'ai_agent_registry'
          AND status = 'active'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HEPHAESTUS email identity");
    assert_eq!(identity_value, "hephaestus@sh-inc.ru");

    let graph_row = sqlx::query(
        r#"
        SELECT label, properties
        FROM graph_nodes
        WHERE node_kind = 'persona'
          AND stable_key = 'persona:v1:ai_agent:HEPHAESTUS'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HEPHAESTUS graph node");
    assert_eq!(
        graph_row.try_get::<String, _>("label").unwrap(),
        "hephaestus@sh-inc.ru"
    );
    let properties = graph_row
        .try_get::<serde_json::Value, _>("properties")
        .unwrap();
    assert_eq!(properties["persona_type"], "ai_agent");
    assert_eq!(properties["agent_id"], "HEPHAESTUS");
}
