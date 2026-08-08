use chrono::{Duration, Utc};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::json;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::integrations::telegram::client::commands;
use makosh_hub_backend::integrations::telegram::client::commands::queries;

use makosh_backend_testkit::context::TestContext;

#[tokio::test]
async fn telegram_outbox_claims_due_command_and_unlocks_while_awaiting_provider() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "claim-await").await;
    let command_id = "tcmd_claim_awaiting_provider";

    insert_edit_command(&pool, &account_id, command_id).await;

    let claimed = commands::claim_due_commands_for_execution(&pool, &account_id, Utc::now(), 10)
        .await
        .expect("claim due commands");

    assert_eq!(claimed.len(), 1);
    let command = &claimed[0];
    assert_eq!(command.command_id, command_id);
    assert_eq!(command.status, "executing");
    assert_eq!(command.retry_count, 1);
    assert!(command.last_attempt_at.is_some());
    assert_eq!(command.locked_by.as_deref(), Some("telegram-outbox-worker"));
    assert_eq!(command.reconciliation_status, "awaiting_provider");

    commands::mark_command_awaiting_provider(
        &pool,
        command_id,
        Utc::now(),
        json!({"dispatch": "accepted"}),
    )
    .await
    .expect("mark awaiting provider");

    let stored = queries::list_commands(&pool, &account_id, 10)
        .await
        .expect("list commands")
        .into_iter()
        .find(|item| item.command_id == command_id)
        .expect("stored command");

    assert_eq!(stored.status, "executing");
    assert_eq!(stored.reconciliation_status, "awaiting_provider");
    assert!(stored.locked_at.is_none());
    assert!(stored.locked_by.is_none());
}

#[tokio::test]
async fn telegram_outbox_recovers_stale_locked_execution_for_retry() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "stale-retry").await;
    let command_id = "tcmd_stale_retry";

    insert_edit_command(&pool, &account_id, command_id).await;
    let claimed = commands::claim_due_commands_for_execution(&pool, &account_id, Utc::now(), 10)
        .await
        .expect("claim due commands");
    assert_eq!(claimed.len(), 1);

    let stale_locked_at = Utc::now() - Duration::minutes(10);
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET locked_at = $2
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(stale_locked_at)
    .execute(&pool)
    .await
    .expect("backdate lock");

    let recovered = commands::recover_stale_executing_commands(
        &pool,
        Utc::now(),
        Utc::now() - Duration::minutes(2),
    )
    .await
    .expect("recover stale commands");

    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].command_id, command_id);
    assert_eq!(recovered[0].status, "retrying");
    assert!(recovered[0].next_attempt_at.is_some());
    assert!(recovered[0].locked_at.is_none());
    assert!(recovered[0].locked_by.is_none());
}

#[tokio::test]
async fn telegram_outbox_dead_letter_can_be_manually_retried() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "manual-retry").await;
    let command_id = "tcmd_manual_retry";

    insert_edit_command(&pool, &account_id, command_id).await;
    commands::dead_letter_command(
        &pool,
        command_id,
        Utc::now(),
        "Unsupported provider write command",
    )
    .await
    .expect("dead letter command");

    let dead_lettered = queries::list_commands(&pool, &account_id, 10)
        .await
        .expect("list commands")
        .into_iter()
        .find(|item| item.command_id == command_id)
        .expect("stored command");
    assert_eq!(dead_lettered.status, "dead_letter");
    assert!(dead_lettered.dead_lettered_at.is_some());

    let retried = commands::manual_retry_command(&pool, command_id, Utc::now())
        .await
        .expect("manual retry")
        .expect("eligible command");

    assert_eq!(retried.status, "retrying");
    assert_eq!(retried.retry_count, 0);
    assert!(retried.next_attempt_at.is_some());
    assert!(retried.dead_lettered_at.is_none());
    assert_eq!(retried.reconciliation_status, "not_observed");
}

async fn create_telegram_account(pool: &sqlx::PgPool, suffix: &str) -> String {
    let account_id = format!("telegram-outbox-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Outbox {suffix}"),
                format!("tg-outbox-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    account_id
}

async fn insert_edit_command(pool: &sqlx::PgPool, account_id: &str, command_id: &str) {
    commands::insert_command(
        pool,
        command_id,
        account_id,
        "edit",
        command_id,
        "-100telegram-outbox",
        Some("-100telegram-outbox:42"),
        "available",
        "provider_write",
        "not_required",
        "makosh-frontend",
        json!({"new_text": "edited text"}),
        json!({"provider_chat_id": "-100telegram-outbox", "provider_message_id": "-100telegram-outbox:42"}),
        json!({"source": "test"}),
    )
    .await
    .expect("insert command");
}
