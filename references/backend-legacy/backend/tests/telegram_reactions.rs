use chrono::Utc;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::json;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::integrations::telegram::client::commands;
use makosh_hub_backend::integrations::telegram::client::commands::queries;
use makosh_hub_backend::integrations::telegram::client::reactions::reconcile_reaction_commands_from_provider_reactions;

use makosh_backend_testkit::context::TestContext;

#[tokio::test]
async fn telegram_provider_reactions_reconcile_react_and_unreact_commands() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "reaction-reconcile", "telegram:123").await;
    let provider_chat_id = "-100reaction-reconcile";
    let provider_message_id = "-100reaction-reconcile:42";

    insert_reaction_command(
        &pool,
        &account_id,
        "tcmd_react_observed",
        "react",
        provider_chat_id,
        provider_message_id,
        "👍",
    )
    .await;
    insert_reaction_command(
        &pool,
        &account_id,
        "tcmd_unreact_observed",
        "unreact",
        provider_chat_id,
        provider_message_id,
        "🔥",
    )
    .await;
    insert_reaction_command(
        &pool,
        &account_id,
        "tcmd_react_still_pending",
        "react",
        provider_chat_id,
        provider_message_id,
        "😎",
    )
    .await;

    let reconciled = reconcile_reaction_commands_from_provider_reactions(
        &pool,
        &account_id,
        provider_chat_id,
        provider_message_id,
        &["👍".to_owned()],
        Utc::now(),
        "tdlib.interaction_info.reactions",
    )
    .await
    .expect("reconcile reaction commands");

    assert_eq!(reconciled.len(), 3);
    assert!(
        reconciled
            .iter()
            .any(|command| command.command_id == "tcmd_react_observed"
                && command.status == "completed"
                && command.reconciliation_status == "observed")
    );
    assert!(
        reconciled
            .iter()
            .any(|command| command.command_id == "tcmd_unreact_observed"
                && command.status == "completed"
                && command.reconciliation_status == "observed")
    );
    assert!(
        reconciled
            .iter()
            .any(|command| command.command_id == "tcmd_react_still_pending"
                && command.status == "failed"
                && command.reconciliation_status == "mismatch")
    );

    let commands = queries::list_commands(&pool, &account_id, 10)
        .await
        .expect("list commands");
    let pending = commands
        .iter()
        .find(|command| command.command_id == "tcmd_react_still_pending")
        .expect("pending command");
    assert_eq!(pending.status, "failed");
    assert_eq!(pending.reconciliation_status, "mismatch");
    assert_eq!(
        pending.last_error.as_deref(),
        Some("Provider observed a different reaction state than requested")
    );
    assert_eq!(pending.provider_state["reaction_emoji"], json!("😎"));
    assert_eq!(pending.provider_state["expected_is_chosen"], json!(true));
    assert_eq!(pending.provider_state["observed_is_chosen"], json!(false));
}

async fn create_telegram_account(
    pool: &sqlx::PgPool,
    suffix: &str,
    external_account_id: &str,
) -> String {
    let account_id = format!("telegram-reactions-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Reactions {suffix}"),
                external_account_id.to_owned(),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    account_id
}

async fn insert_reaction_command(
    pool: &sqlx::PgPool,
    account_id: &str,
    command_id: &str,
    command_kind: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    reaction_emoji: &str,
) {
    commands::insert_command(
        pool,
        command_id,
        account_id,
        command_kind,
        command_id,
        provider_chat_id,
        Some(provider_message_id),
        "available",
        "provider_write",
        "not_required",
        "makosh-frontend",
        json!({"reaction_emoji": reaction_emoji}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert reaction command");
}
