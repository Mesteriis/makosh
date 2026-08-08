use makosh_backend_testkit::context::TestContext;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::analytics::EmailAnalyticsStore;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

#[tokio::test]
async fn message_set_ai_analysis_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("ai analysis").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_ai_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "AI Gmail",
        format!("ai-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_ai_{suffix}"),
        &format!("provider-ai-{suffix}"),
        "AI test subject",
        "AI test body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let updated = message_store
        .set_ai_analysis(
            &projected.message_id,
            Some("work"),
            Some("This is a work-related email about project updates."),
            Some(85),
        )
        .await
        .expect("set ai analysis");

    assert_eq!(updated.ai_category.as_deref(), Some("work"));
    assert_eq!(
        updated.ai_summary.as_deref(),
        Some("This is a work-related email about project updates.")
    );
    assert_eq!(updated.importance_score, Some(85));
    assert!(updated.ai_summary_generated_at.is_some());

    let fetched = message_store
        .message(&projected.message_id)
        .await
        .expect("fetch message")
        .expect("message exists");
    assert_eq!(fetched.ai_category.as_deref(), Some("work"));
    assert_eq!(fetched.importance_score, Some(85));
}

#[tokio::test]
async fn message_analytics_decodes_averages_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_analytics_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Analytics Gmail",
        format!("analytics-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_analytics_{suffix}"),
        &format!("provider-analytics-{suffix}"),
        "Analytics subject",
        "Analytics body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");
    message_store
        .set_ai_analysis(
            &projected.message_id,
            Some("work"),
            Some("summary"),
            Some(80),
        )
        .await
        .expect("set ai analysis");

    let analytics = EmailAnalyticsStore::new(pool);
    let health = analytics
        .mailbox_health(Some(&account_id))
        .await
        .expect("mailbox health");
    let senders = analytics
        .top_senders(Some(&account_id), 10)
        .await
        .expect("top senders");

    assert_eq!(health.total_messages, 1);
    assert_eq!(health.average_importance, 80.0);
    assert_eq!(senders.len(), 1);
    assert_eq!(senders[0].avg_importance, 80.0);
}

#[tokio::test]
async fn message_set_ai_analysis_rejects_invalid_score() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("ai analysis invalid score").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_ai_score_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "AI Score Gmail",
        format!("ai-score-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_ai_score_{suffix}"),
        &format!("provider-ai-score-{suffix}"),
        "Score test",
        "Score body",
    )
    .await;
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let result = message_store
        .set_ai_analysis(&projected.message_id, None, None, Some(101))
        .await;

    assert!(result.is_err());
}
