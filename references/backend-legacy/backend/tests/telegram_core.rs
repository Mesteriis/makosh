mod telegram_support;

use makosh_backend_testkit::context::TestContext;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_capability_status, assert_ok, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};
#[tokio::test]
async fn telegram_fixture_message_ingestion_refreshes_decision_and_obligation_candidates_against_postgres()
 {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-candidate-{suffix}");
    let chat_id = format!("tg-candidate-chat-{suffix}");
    let decision_title = format!("Use Telegram evidence for shared memory {suffix}");
    let decision_rationale = "channel context must feed the same domain model";
    let obligation_statement = format!("send the Telegram alignment note {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Candidate Source",
                "external_account_id": format!("tg-candidate-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/candidate-{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("tg-candidate-message-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Telegram Candidate Review",
                "sender_id": format!("telegram-candidate-sender-{suffix}"),
                "sender_display_name": "Telegram Candidate Sender",
                "text": format!(
                    "Decision: {decision_title} because {decision_rationale}. I will {obligation_statement} by Friday 5pm."
                ),
                "import_batch_id": format!("telegram-candidate-fixture-{suffix}"),
                "occurred_at": "2026-06-06T12:30:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();
    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let raw_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.telegram.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw telegram signal count");
    let accepted_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.telegram.message'",
    )
    .fetch_one(&pool)
    .await
    .expect("accepted telegram signal count");
    let legacy_integration_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type LIKE 'integration.telegram.%'",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy telegram integration event count");
    assert_eq!(raw_signal_count, 1);
    assert_eq!(accepted_signal_count, 1);
    assert_eq!(legacy_integration_count, 0);
    let trace_rows = sqlx::query(
        r#"
        SELECT event_id, event_type, causation_id, correlation_id
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
              'observation.captured.v1',
              'signal.raw.telegram.message.observed',
              'signal.accepted.telegram.message',
              'communication.message.recorded'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&pool)
    .await
    .expect("telegram trace rows");
    assert_eq!(trace_rows.len(), 4);
    let observation_event_id = format!("event:v1:observation-captured:{observation_id}");
    let raw_event_id: String = trace_rows[1].try_get("event_id").expect("raw event id");
    let accepted_event_id: String = trace_rows[2]
        .try_get("event_id")
        .expect("accepted event id");
    assert_eq!(
        trace_rows[0]
            .try_get::<String, _>("event_id")
            .expect("observation event id"),
        observation_event_id
    );
    assert_eq!(
        trace_rows[1]
            .try_get::<Option<String>, _>("causation_id")
            .expect("raw causation")
            .as_deref(),
        Some(observation_event_id.as_str())
    );
    assert_eq!(
        trace_rows[2]
            .try_get::<Option<String>, _>("causation_id")
            .expect("accepted causation")
            .as_deref(),
        Some(raw_event_id.as_str())
    );
    assert_eq!(
        trace_rows[3]
            .try_get::<Option<String>, _>("causation_id")
            .expect("communication causation")
            .as_deref(),
        Some(accepted_event_id.as_str())
    );
    assert!(trace_rows.iter().all(|row| {
        row.try_get::<Option<String>, _>("correlation_id")
            .expect("trace correlation")
            .as_deref()
            == Some(observation_id.as_str())
    }));

    let decision_row: (String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT d.title, d.rationale, d.review_state, e.source_kind, e.source_id
        FROM decisions d
        JOIN decision_evidence e ON e.decision_id = d.decision_id
        WHERE e.source_kind = 'communication'
          AND e.source_id = $1
          AND d.title = $2
        "#,
    )
    .bind(&message_id)
    .bind(&decision_title)
    .fetch_one(&pool)
    .await
    .expect("Telegram message should create a suggested Decision candidate");
    assert_eq!(decision_row.1, decision_rationale);
    assert_eq!(decision_row.2, "suggested");
    assert_eq!(decision_row.3, "communication");
    assert_eq!(decision_row.4, message_id);

    let task_candidate_row: (String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND source_id = $1
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("Telegram message should create an obligation-derived task candidate");
    assert_eq!(task_candidate_row.0, obligation_statement);
    assert_eq!(task_candidate_row.1, "suggested");
    assert_eq!(task_candidate_row.2, "obligation_task");
    assert_eq!(task_candidate_row.3.as_deref(), Some("Friday 5pm"));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&obligation_statement)
            .fetch_one(&pool)
            .await
            .expect("accepted obligation count");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn telegram_api_exercises_policy_and_call_foundation() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let chat_id = format!("tg-chat-{suffix}");
    let policy_id = format!("policy-telegram-{suffix}");
    let template_id = format!("template-telegram-{suffix}");
    let call_id = format!("call-telegram-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(capabilities_body["runtime_mode"], json!("fixture"));
    assert_eq!(
        capabilities_body["telegram_app_credentials_configured"],
        json!(false)
    );
    assert_eq!(capabilities_body["qr_login_ready"], json!(false));
    assert_capability_status(
        &capabilities_body,
        "telegram_fixture_runtime",
        "available",
        true,
    );
    assert_capability_status(&capabilities_body, "automation_dry_run", "available", true);
    assert_capability_status(&capabilities_body, "tdlib_live_runtime", "blocked", true);
    assert_capability_status(&capabilities_body, "automation_live_send", "blocked", true);
    assert_capability_status(
        &capabilities_body,
        "whisper_rs_speech_to_text",
        "blocked",
        true,
    );
    assert_capability_status(&capabilities_body, "topics.list", "degraded", false);
    assert_capability_status(&capabilities_body, "topics.create", "blocked", true);
    assert_capability_status(&capabilities_body, "topics.close", "blocked", true);
    assert!(
        capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "hidden_recording")
    );
    assert!(
        capabilities_body["planned_features"]
            .as_array()
            .expect("planned features")
            .iter()
            .any(|feature| feature == "bot_runtime")
    );
    assert!(
        capabilities_body["planned_features"]
            .as_array()
            .expect("planned features")
            .iter()
            .any(|feature| feature == "ai_review_flows")
    );
    assert!(
        !capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "forum_topic_mutations")
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram User",
                "external_account_id": format!("tg-user-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(account_body["provider_kind"], json!("telegram_user"));
    assert_eq!(account_body["runtime"], json!("fixture"));

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("tg-message-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Telegram Planning",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Maria Petrova",
                "text": "Please follow up on the Telegram policy plan.",
                "import_batch_id": format!("telegram-fixture-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    assert!(
        message_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v4:telegram:")
    );

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    assert_eq!(chats_body["items"][0]["provider_chat_id"], json!(chat_id));

    assert_ok(
        app.clone(),
        "/api/v1/policies/templates",
        json!({
            "template_id": template_id,
            "name": "Follow up",
            "body_template": "Hi {{name}}, I will follow up on {{topic}}.",
            "required_variables": ["name", "topic"]
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/policies",
        json!({
            "policy_id": policy_id,
            "template_id": template_id,
            "name": "Allowed Telegram follow up",
            "enabled": true,
            "account_id": account_id,
            "allowed_chat_ids": [chat_id],
            "scopes": [{
                "scope_kind": "mail.account",
                "scope_value": "mail-account-scope"
            }],
            "trigger_kind": "ai_follow_up",
            "max_sends_per_hour": 3,
            "quiet_hours": {},
            "conditions": {"source": "test"}
        }),
    )
    .await;

    let policies_response = app
        .clone()
        .oneshot(get_request_with_token("/api/v1/policies", LOCAL_API_TOKEN))
        .await
        .expect("policies response");
    assert_eq!(policies_response.status(), StatusCode::OK);
    let policies_body = json_body(policies_response).await;
    assert_eq!(
        policies_body["items"][0]["scopes"],
        json!([
            { "scope_kind": "mail.account", "scope_value": "mail-account-scope" },
            { "scope_kind": "telegram.chat", "scope_value": chat_id }
        ])
    );

    let blocked_command_id = format!("dry-run-blocked-{suffix}");
    let blocked_chat_id = format!("other-chat-{suffix}");
    let blocked = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/policies/telegram-send/dry-run",
            json!({
                "command_id": blocked_command_id,
                "policy_id": policy_id,
                "provider_chat_id": blocked_chat_id,
                "variables": {"name": "Maria", "topic": "Telegram"},
                "source_context": {"source": "test"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("blocked dry-run");
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);

    let rejected_audit = sqlx::query(
        r#"
        SELECT target_kind, target_id, metadata
        FROM api_audit_log
        WHERE operation = 'automation.telegram_send.dry_run'
          AND actor_id = $1
          AND metadata->>'decision' = 'rejected'
          AND metadata->>'provider_chat_id' = $2
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("makosh-frontend")
    .bind(&blocked_chat_id)
    .fetch_one(&pool)
    .await
    .expect("rejected dry-run audit");
    let rejected_target_kind: String = rejected_audit.try_get("target_kind").expect("target kind");
    let rejected_target_id: Option<String> =
        rejected_audit.try_get("target_id").expect("target id");
    let rejected_metadata: Value = rejected_audit.try_get("metadata").expect("metadata");
    assert_eq!(rejected_target_kind, "telegram_send_request");
    assert_eq!(
        rejected_target_id.as_deref(),
        Some(blocked_command_id.as_str())
    );
    assert_eq!(rejected_metadata["action_class"], json!("automation"));
    assert_eq!(rejected_metadata["capability"], json!("telegram.send"));
    assert_eq!(rejected_metadata["decision"], json!("rejected"));
    assert_eq!(
        rejected_metadata["reason"],
        json!("provider_chat_not_allowed")
    );
    assert_eq!(rejected_metadata["confirmation_required"], json!(true));
    assert_eq!(rejected_metadata["scoped_automation_policy"], json!(false));
    assert_eq!(rejected_metadata["automation_policy_id"], json!(policy_id));
    assert!(rejected_metadata.get("variables").is_none());
    assert!(rejected_metadata.get("source_context").is_none());
    assert!(rejected_metadata.get("rendered_text").is_none());
    assert!(rejected_metadata.get("rendered_preview_hash").is_none());

    let dry_run = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/policies/telegram-send/dry-run",
            json!({
                "command_id": format!("dry-run-allowed-{suffix}"),
                "policy_id": policy_id,
                "provider_chat_id": chat_id,
                "variables": {"name": "Maria", "topic": "Telegram"},
                "source_context": {"source": "test"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dry-run response");
    assert_eq!(dry_run.status(), StatusCode::OK);
    let dry_run_body = json_body(dry_run).await;
    assert_eq!(dry_run_body["status"], json!("allowed"));
    assert!(
        dry_run_body["rendered_preview_hash"]
            .as_str()
            .expect("hash")
            .starts_with("sha256:")
    );
    let outbound_message_id = dry_run_body["outbound_message_id"]
        .as_str()
        .expect("outbound message id");

    let template_observations = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'automation'
          AND link.entity_kind = 'template'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&template_id)
    .fetch_all(&pool)
    .await
    .expect("template observations");
    assert!(
        template_observations.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AUTOMATION_TEMPLATE"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<Value, _>("payload")["template_id"] == json!(template_id)
        }),
        "automation template observation must exist"
    );

    let policy_observations = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'automation'
          AND link.entity_kind = 'policy'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&policy_id)
    .fetch_all(&pool)
    .await
    .expect("policy observations");
    assert!(
        policy_observations.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AUTOMATION_POLICY"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<Value, _>("payload")["policy_id"] == json!(policy_id)
        }),
        "automation policy observation must exist"
    );

    let outbound_observations = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'automation'
          AND link.entity_kind = 'telegram_outbound_message'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(outbound_message_id)
    .fetch_all(&pool)
    .await
    .expect("outbound message observations");
    assert!(
        outbound_observations.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_OUTBOUND_MESSAGE"
                && row.get::<String, _>("relationship_kind") == "dry_run_allowed"
                && row.get::<Value, _>("payload")["outbound_message_id"]
                    == json!(outbound_message_id)
        }),
        "telegram outbound dry-run observation must exist"
    );

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_audit_log WHERE operation = 'automation.telegram_send.dry_run' AND actor_id = $1",
    )
    .bind("makosh-frontend")
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert!(audit_count >= 2);

    let allowed_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'automation.telegram_send.dry_run'
          AND actor_id = $1
          AND target_id = $2
          AND metadata->>'decision' = 'allowed'
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("makosh-frontend")
    .bind(outbound_message_id)
    .fetch_one(&pool)
    .await
    .expect("allowed dry-run audit metadata");
    assert_eq!(allowed_metadata["action_class"], json!("automation"));
    assert_eq!(allowed_metadata["capability"], json!("telegram.send"));
    assert_eq!(allowed_metadata["decision"], json!("allowed"));
    assert_eq!(
        allowed_metadata["reason"],
        json!("scoped_automation_policy_authorized")
    );
    assert_eq!(allowed_metadata["confirmation_required"], json!(false));
    assert_eq!(allowed_metadata["scoped_automation_policy"], json!(true));
    assert_eq!(allowed_metadata["automation_policy_id"], json!(policy_id));
    assert_eq!(
        allowed_metadata["rendered_preview_hash"],
        dry_run_body["rendered_preview_hash"]
    );
    assert!(allowed_metadata.get("variables").is_none());
    assert!(allowed_metadata.get("source_context").is_none());
    assert!(allowed_metadata.get("rendered_text").is_none());

    assert_ok(
        app.clone(),
        "/api/v1/calls",
        json!({
            "call_id": call_id,
            "account_id": account_id,
            "provider_call_id": format!("provider-call-{suffix}"),
            "provider_chat_id": chat_id,
            "direction": "incoming",
            "call_state": "ended",
            "started_at": "2026-06-06T12:10:00Z",
            "ended_at": "2026-06-06T12:20:00Z",
            "transcription_policy_id": policy_id,
            "metadata": {"runtime": "fixture"}
        }),
    )
    .await;

    let transcript_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/calls/{call_id}/transcript"),
            json!({
                "transcript_id": format!("transcript-telegram-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "source_audio_ref": format!("local-audio-{suffix}.wav"),
                "language_code": "en",
                "always_on_policy": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("transcript response");
    assert_eq!(transcript_response.status(), StatusCode::OK);
    let transcript_body = json_body(transcript_response).await;
    assert_eq!(transcript_body["transcript_status"], json!("succeeded"));
    assert_eq!(transcript_body["stt_provider"], json!("fixture-stt"));
    assert!(
        transcript_body["transcript_text"]
            .as_str()
            .expect("transcript text")
            .contains("follow up")
    );
}
