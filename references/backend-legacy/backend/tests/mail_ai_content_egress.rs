use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use serde_json::json;
use std::sync::Arc;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::ai_state::{
    CommunicationAiState, CommunicationAiStateStore,
};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::sensitive_forwarding::SensitiveForwardingPgStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::workflows::email_intelligence::pipeline::MailAiPipelineService;

#[tokio::test]
async fn external_mail_ai_requires_explicit_body_egress_permission() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-ai-egress-denied";
    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());

    ingestion
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Gmail,
                "External AI Egress Test",
                "egress@example.test",
            )
            .config(json!({})),
        )
        .await
        .expect("store account");
    let raw = ingestion
        .record_raw_source(&NewRawCommunicationRecord::new(
            "raw-mail-ai-egress-denied",
            account_id,
            "email_message",
            "provider-mail-ai-egress-denied",
            "sha256:mail-ai-egress-denied",
            "batch-mail-ai-egress-denied",
            json!({
                "subject": "Private message",
                "from": "sender@example.test",
                "to": ["owner@example.test"],
                "body_text": "This private message must not reach an external AI provider."
            }),
        ))
        .await
        .expect("record raw message");
    let message = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let report = MailAiPipelineService::new(
        pool.clone(),
        None,
        "ru",
        Arc::new(SensitiveForwardingPgStore::new(pool.clone())),
    )
    .requiring_external_body_egress(true)
    .process_next_batch(10)
    .await
    .expect("process mail AI batch");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.processed, 1);
    assert_eq!(report.suppressed, 1);
    assert_eq!(report.failed, 0);

    let ai_state = CommunicationAiStateStore::new(pool.clone())
        .current(&message.message_id)
        .await
        .expect("read AI state")
        .expect("message AI state");
    assert_eq!(ai_state.ai_state, CommunicationAiState::ReviewRequired);
    assert_eq!(
        ai_state.review_reason.as_deref(),
        Some("body_egress_denied")
    );
    assert!(ai_state.last_error.is_none());

    let projected = message_store
        .message(&message.message_id)
        .await
        .expect("read projected message")
        .expect("projected message");
    assert_eq!(
        projected.message_metadata["mail_ai_pipeline"]["status"],
        "review_required"
    );
    assert_eq!(
        projected.message_metadata["mail_ai_pipeline"]["body_included"],
        false
    );
}
