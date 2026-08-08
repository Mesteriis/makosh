use serde_json::json;

use makosh_hub_backend::domains::communications::messages::errors::MessageProjectionError;
use makosh_hub_backend::domains::communications::messages::models::MessageSearchMatchMode;
use makosh_hub_backend::domains::communications::messages::models::{
    MessageSearchQuery, NewProjectedMessage, ProjectedMessagePageQuery,
};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::states::{
    LocalMessageState, WorkflowState,
};

use super::support::{
    disconnected_message_store, live_projection_context, record_raw_email_message,
    store_provider_account, stored_raw_record_with_payload, unique_suffix,
};

#[tokio::test]
async fn message_projection_list_messages_filters_by_account_state_channel_and_query_against_postgres()
 {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message filtered listing").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_left = format!("acct_message_filter_left_{suffix}");
    let account_right = format!("acct_message_filter_right_{suffix}");

    store_provider_account(
        &communication_store,
        &account_left,
        "Filter Left",
        format!("filter-left-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &account_right,
        "Filter Right",
        format!("filter-right-{suffix}@example.com"),
    )
    .await;

    let left_raw = record_raw_email_message(
        &communication_store,
        &account_left,
        &format!("raw_message_filter_left_{suffix}"),
        &format!("provider-filter-left-{suffix}"),
        "Quarterly Alpha Contract",
        "The alpha renewal needs a legal review.",
    )
    .await;
    let right_raw = record_raw_email_message(
        &communication_store,
        &account_right,
        &format!("raw_message_filter_right_{suffix}"),
        &format!("provider-filter-right-{suffix}"),
        "Quarterly Beta Invoice",
        "The beta invoice is already paid.",
    )
    .await;

    let left = project_raw_email_message(&message_store, &left_raw)
        .await
        .expect("project left message");
    let right = project_raw_email_message(&message_store, &right_raw)
        .await
        .expect("project right message");
    message_store
        .transition_workflow_state(&left.message_id, WorkflowState::NeedsAction)
        .await
        .expect("set left state");
    message_store
        .transition_workflow_state(&right.message_id, WorkflowState::Reviewed)
        .await
        .expect("set right state");

    let filtered = message_store
        .list_messages(
            Some(&account_left),
            Some(WorkflowState::NeedsAction),
            Some("email"),
            Some("alpha legal"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list filtered messages");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message.message_id, left.message_id);
    assert_eq!(filtered[0].message.account_id, account_left);

    let no_match = message_store
        .list_messages(
            Some(&account_left),
            Some(WorkflowState::NeedsAction),
            Some("email"),
            Some("beta"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list non-matching messages");
    assert!(no_match.is_empty());
}

#[tokio::test]
async fn message_projection_channel_kind_telegram_alias_matches_user_and_bot_messages_against_postgres()
 {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message telegram channel alias").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_telegram_alias_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Telegram Alias",
        format!("telegram-alias-{suffix}@example.com"),
    )
    .await;

    let user_raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_telegram_alias_user_{suffix}"),
        &format!("provider-telegram-alias-user-{suffix}"),
        "Telegram User",
        "User channel body",
    )
    .await;
    let bot_raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_telegram_alias_bot_{suffix}"),
        &format!("provider-telegram-alias-bot-{suffix}"),
        "Telegram Bot",
        "Bot channel body",
    )
    .await;
    let whatsapp_raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_telegram_alias_whatsapp_{suffix}"),
        &format!("provider-telegram-alias-whatsapp-{suffix}"),
        "WhatsApp",
        "WhatsApp channel body",
    )
    .await;

    let user_message = NewProjectedMessage {
        message_id: format!("msg:telegram-alias:user:{suffix}"),
        raw_record_id: user_raw.raw_record_id.clone(),
        account_id: account_id.clone(),
        provider_record_id: user_raw.provider_record_id.clone(),
        subject: "Telegram User".to_owned(),
        sender: "telegram:user:42".to_owned(),
        recipients: vec![],
        body_text: "User channel body".to_owned(),
        occurred_at: user_raw.occurred_at,
        channel_kind: "telegram_user".to_owned(),
        conversation_id: Some(format!("conversation:telegram-alias:{suffix}")),
        sender_display_name: Some("Ada".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };
    let bot_message = NewProjectedMessage {
        message_id: format!("msg:telegram-alias:bot:{suffix}"),
        raw_record_id: bot_raw.raw_record_id.clone(),
        account_id: account_id.clone(),
        provider_record_id: bot_raw.provider_record_id.clone(),
        subject: "Telegram Bot".to_owned(),
        sender: "telegram:bot:7".to_owned(),
        recipients: vec![],
        body_text: "Bot channel body".to_owned(),
        occurred_at: bot_raw.occurred_at,
        channel_kind: "telegram_bot".to_owned(),
        conversation_id: Some(format!("conversation:telegram-alias:{suffix}")),
        sender_display_name: Some("Build Bot".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };
    let whatsapp_message = NewProjectedMessage {
        message_id: format!("msg:telegram-alias:whatsapp:{suffix}"),
        raw_record_id: whatsapp_raw.raw_record_id.clone(),
        account_id: account_id.clone(),
        provider_record_id: whatsapp_raw.provider_record_id.clone(),
        subject: "WhatsApp".to_owned(),
        sender: "whatsapp:user:9".to_owned(),
        recipients: vec![],
        body_text: "WhatsApp channel body".to_owned(),
        occurred_at: whatsapp_raw.occurred_at,
        channel_kind: "whatsapp_web".to_owned(),
        conversation_id: Some(format!("conversation:whatsapp-alias:{suffix}")),
        sender_display_name: Some("Grace".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    let user = message_store
        .upsert_channel_message(&user_message)
        .await
        .expect("upsert telegram user message");
    let bot = message_store
        .upsert_channel_message(&bot_message)
        .await
        .expect("upsert telegram bot message");
    message_store
        .upsert_channel_message(&whatsapp_message)
        .await
        .expect("upsert whatsapp control message");

    let page = message_store
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: Some(&account_id),
            workflow_state: None,
            is_read: None,
            channel_kind: Some("telegram"),
            conversation_id: None,
            query: None,
            match_mode: MessageSearchMatchMode::All,
            search: MessageSearchQuery::default(),
            local_state: LocalMessageState::Active,
            cursor: None,
            limit: 10,
        })
        .await
        .expect("list telegram channel alias messages");

    let ids = page
        .items
        .iter()
        .map(|summary| summary.message.message_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&user.message_id.as_str()));
    assert!(ids.contains(&bot.message_id.as_str()));
}

#[tokio::test]
async fn message_local_trash_hides_from_default_lists_and_survives_reprojection_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message local trash").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_local_trash_{suffix}");
    let raw_record_id = format!("raw_message_local_trash_{suffix}");
    let provider_record_id = format!("provider-local-trash-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Local Trash Gmail",
        format!("local-trash-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &provider_record_id,
        "Local trash subject",
        "Local trash body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project local trash message");
    assert_eq!(projected.local_state, LocalMessageState::Active);

    let trashed = message_store
        .move_to_local_trash(&projected.message_id, "user_deleted")
        .await
        .expect("move message to local trash");
    assert_eq!(trashed.local_state, LocalMessageState::Trash);
    assert_eq!(trashed.local_state_reason.as_deref(), Some("user_deleted"));
    assert!(trashed.local_state_changed_at.is_some());

    let default_messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            Some("Local trash"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list active messages");
    assert!(default_messages.is_empty());

    let trash_messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            Some("Local trash"),
            LocalMessageState::Trash,
            10,
        )
        .await
        .expect("list trash messages");
    assert_eq!(trash_messages.len(), 1);
    assert_eq!(trash_messages[0].message.message_id, projected.message_id);

    let reprojected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("reproject local trash message");
    assert_eq!(reprojected.local_state, LocalMessageState::Trash);

    let restored = message_store
        .restore_from_local_trash(&projected.message_id)
        .await
        .expect("restore local trash message");
    assert_eq!(restored.local_state, LocalMessageState::Active);

    let restored_messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            Some("Local trash"),
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list restored messages");
    assert_eq!(restored_messages.len(), 1);
}

#[tokio::test]
async fn message_search_supports_any_mode_and_field_rules_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message search rules").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_search_rules_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Search Rules Gmail",
        format!("search-rules-{suffix}@example.com"),
    )
    .await;

    let quarterly = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_search_rules_quarterly_{suffix}"),
        &format!("provider-message-search-rules-quarterly-{suffix}"),
        "Quarterly Report",
        "Payment follow-up for the finance team",
    )
    .await;
    let travel = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_search_rules_travel_{suffix}"),
        &format!("provider-message-search-rules-travel-{suffix}"),
        "Travel Plan",
        "Invoice approved for next week",
    )
    .await;

    let quarterly_projected = project_raw_email_message(&message_store, &quarterly)
        .await
        .expect("project quarterly message");
    let travel_projected = project_raw_email_message(&message_store, &travel)
        .await
        .expect("project travel message");
    message_store
        .set_read_state(&quarterly_projected.message_id, true, "local_user")
        .await
        .expect("mark quarterly message read");

    let read_only = message_store
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: Some(&account_id),
            workflow_state: None,
            is_read: Some(true),
            channel_kind: None,
            conversation_id: None,
            query: None,
            match_mode: MessageSearchMatchMode::All,
            search: MessageSearchQuery::default(),
            local_state: LocalMessageState::Active,
            cursor: None,
            limit: 10,
        })
        .await
        .expect("list read messages");
    assert_eq!(read_only.items.len(), 1);
    assert_eq!(
        read_only.items[0].message.message_id,
        quarterly_projected.message_id
    );

    let any_mode = message_store
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: Some(&account_id),
            workflow_state: None,
            is_read: None,
            channel_kind: None,
            conversation_id: None,
            query: None,
            match_mode: MessageSearchMatchMode::Any,
            search: MessageSearchQuery {
                subject_contains: vec!["quarterly".to_owned()],
                body_contains: vec!["invoice".to_owned()],
                match_mode: MessageSearchMatchMode::Any,
                ..MessageSearchQuery::default()
            },
            local_state: LocalMessageState::Active,
            cursor: None,
            limit: 10,
        })
        .await
        .expect("list any-mode messages");

    let any_ids = any_mode
        .items
        .iter()
        .map(|summary| summary.message.message_id.as_str())
        .collect::<Vec<_>>();
    assert!(any_ids.contains(&quarterly_projected.message_id.as_str()));
    assert!(any_ids.contains(&travel_projected.message_id.as_str()));

    let all_mode = message_store
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: Some(&account_id),
            workflow_state: None,
            is_read: None,
            channel_kind: None,
            conversation_id: None,
            query: None,
            match_mode: MessageSearchMatchMode::All,
            search: MessageSearchQuery {
                subject_contains: vec!["quarterly".to_owned()],
                body_contains: vec!["payment".to_owned()],
                match_mode: MessageSearchMatchMode::All,
                ..MessageSearchQuery::default()
            },
            local_state: LocalMessageState::Active,
            cursor: None,
            limit: 10,
        })
        .await
        .expect("list all-mode messages");

    assert_eq!(all_mode.items.len(), 1);
    assert_eq!(
        all_mode.items[0].message.message_id,
        quarterly_projected.message_id
    );
}

#[tokio::test]
async fn message_projection_reports_missing_or_wrong_payload_fields() {
    let store = disconnected_message_store();
    let cases = [
        (
            "subject",
            json!({"from":"alice@example.com","to":["bob@example.com"],"body_text":"Body"}),
        ),
        (
            "from",
            json!({"subject":"Subject","from":42,"to":["bob@example.com"],"body_text":"Body"}),
        ),
        (
            "to",
            json!({"subject":"Subject","from":"alice@example.com","to":"bob@example.com","body_text":"Body"}),
        ),
        (
            "to",
            json!({"subject":"Subject","from":"alice@example.com","to":["bob@example.com",42],"body_text":"Body"}),
        ),
        (
            "body_text",
            json!({"subject":"Subject","from":"alice@example.com","to":["bob@example.com"]}),
        ),
    ];

    for (field_name, payload) in cases {
        let raw = stored_raw_record_with_payload(payload);
        let error = project_raw_email_message(&store, &raw)
            .await
            .expect_err("projecting malformed payload must fail");

        assert!(
            matches!(
                error,
                MessageProjectionError::MissingPayloadField(actual) if actual == field_name
            ),
            "expected MissingPayloadField({field_name}), got {error:?}"
        );
    }
}

#[tokio::test]
async fn message_projection_rejects_direct_upsert_with_mismatched_raw_tuple_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("direct message upsert raw tuple mismatch").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let raw_account_id = format!("acct_message_raw_tuple_{suffix}");
    let mismatched_account_id = format!("acct_message_raw_tuple_other_{suffix}");
    let provider_record_id = format!("provider-message-raw-tuple-{suffix}");

    store_provider_account(
        &communication_store,
        &raw_account_id,
        "Projection raw tuple source",
        format!("projection-raw-tuple-source-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &mismatched_account_id,
        "Projection raw tuple mismatch",
        format!("projection-raw-tuple-mismatch-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &raw_account_id,
        &format!("raw_message_tuple_{suffix}"),
        &provider_record_id,
        "Raw tuple subject",
        "Raw tuple body",
    )
    .await;
    let message = NewProjectedMessage {
        message_id: format!("msg:mismatched:{suffix}"),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: mismatched_account_id.clone(),
        provider_record_id: provider_record_id.clone(),
        subject: "Raw tuple subject".to_owned(),
        sender: "alice@example.com".to_owned(),
        recipients: vec!["bob@example.com".to_owned()],
        body_text: "Raw tuple body".to_owned(),
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some("alice@example.com".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    let error = message_store
        .upsert_message(&message)
        .await
        .expect_err("direct upsert must reject mismatched raw tuple");

    assert!(
        matches!(
            error,
            MessageProjectionError::RawRecordTupleMismatch {
                ref raw_record_id,
                ref account_id,
                provider_record_id: ref actual_provider_record_id,
            } if raw_record_id.as_str() == raw.raw_record_id
                && account_id.as_str() == mismatched_account_id
                && actual_provider_record_id.as_str() == provider_record_id
        ),
        "expected RawRecordTupleMismatch, got {error:?}"
    );

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE raw_record_id = $1
          AND account_id = $2
          AND provider_record_id = $3
        "#,
    )
    .bind(&raw.raw_record_id)
    .bind(&mismatched_account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("mismatched projected message count");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn message_projection_rejects_empty_fields_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message validation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_validation_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Validation Gmail",
        format!("validation-{suffix}@example.com"),
    )
    .await;

    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_empty_subj_{suffix}"),
        &format!("provider-empty-subj-{suffix}"),
        "",
        "body",
    )
    .await;
    let result = project_raw_email_message(&message_store, &raw).await;
    assert!(result.is_err());
}
