use makosh_backend_testkit::context::TestContext;
use sqlx::Row;

use super::*;
use crate::integrations::telegram::client::models::messages::TelegramReactionRequest;

#[tokio::test]
async fn provider_state_sync_deactivates_absent_self_reactions() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "reaction-sync", "telegram:123").await;
    let message_id = "msg_reaction_sync";
    let provider_chat_id = "-100reaction-sync";
    let provider_message_id = "-100reaction-sync:77";
    let self_sender_id = "user:123";

    let add_request = TelegramReactionRequest {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        sender_id: self_sender_id.to_owned(),
        sender_display_name: Some("Owner".to_owned()),
        reaction_emoji: "👍".to_owned(),
        command_id: None,
    };
    add_reaction(&pool, &add_request, message_id)
        .await
        .expect("add chosen reaction");

    let remove_request = TelegramReactionRequest {
        reaction_emoji: "🔥".to_owned(),
        ..add_request.clone()
    };
    add_reaction(&pool, &remove_request, message_id)
        .await
        .expect("add stale reaction");

    sync_provider_reactions(
        &pool,
        TelegramReactionMessageRef {
            message_id,
            account_id: &account_id,
            provider_chat_id,
            provider_message_id,
        },
        &[],
        Some(self_sender_id),
        &["👍".to_owned()],
    )
    .await
    .expect("sync provider reactions");

    let rows = sqlx::query(
        r#"
        SELECT reaction_emoji, is_active
        FROM telegram_message_reactions
        WHERE message_id = $1 AND sender_id = $2
        ORDER BY reaction_emoji ASC
        "#,
    )
    .bind(message_id)
    .bind(self_sender_id)
    .fetch_all(&pool)
    .await
    .expect("reaction rows");

    let states = rows
        .into_iter()
        .map(|row| {
            (
                row.try_get::<String, _>("reaction_emoji")
                    .expect("reaction_emoji"),
                row.try_get::<bool, _>("is_active").expect("is_active"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        states,
        vec![("👍".to_owned(), true), ("🔥".to_owned(), false)]
    );
}

async fn create_telegram_account(
    pool: &sqlx::PgPool,
    suffix: &str,
    external_account_id: &str,
) -> String {
    let account_id = format!("telegram-reactions-{suffix}");
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        &account_id,
        &format!("Telegram Reactions {suffix}"),
        external_account_id,
    )
    .await;
    account_id
}
