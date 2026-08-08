use serde_json::json;

use super::super::models::chats::{
    NewTelegramChat, NewTelegramChatParticipant, TelegramChatKind, TelegramSyncState,
};
use super::super::store::TelegramStore;
use super::{
    TelegramChatMember, inactive_roster_membership_state,
    mark_absent_members_from_exhaustive_roster, tdlib_self_membership_lifecycle,
    telegram_self_provider_member_id, upsert_chat_participant,
};

#[test]
fn derives_self_provider_member_id_only_from_numeric_telegram_identity() {
    assert_eq!(
        telegram_self_provider_member_id("telegram:12345").as_deref(),
        Some("user:12345")
    );
    assert_eq!(
        telegram_self_provider_member_id("user:456").as_deref(),
        Some("user:456")
    );
    assert_eq!(telegram_self_provider_member_id("fixture-user"), None);
    assert_eq!(
        telegram_self_provider_member_id("telegram:not-numeric"),
        None
    );
}

#[test]
fn parses_self_leave_and_join_membership_evidence_from_tdlib_service_messages() {
    let leave = tdlib_self_membership_lifecycle(
        "telegram:42",
        &json!({
            "content": {
                "@type": "messageChatDeleteMember",
                "user_id": 42
            }
        }),
    )
    .expect("leave lifecycle");
    assert_eq!(leave.command_kind, "leave");
    assert_eq!(leave.provider_member_id, "user:42");
    assert_eq!(leave.observed_via, "tdlib.messageChatDeleteMember");
    assert_eq!(leave.membership_state, "left");

    let join = tdlib_self_membership_lifecycle(
        "telegram:42",
        &json!({
            "content": {
                "@type": "messageChatAddMembers",
                "member_user_ids": [1, 42, 9]
            }
        }),
    )
    .expect("join lifecycle");
    assert_eq!(join.command_kind, "join");
    assert_eq!(join.provider_member_id, "user:42");
    assert_eq!(join.observed_via, "tdlib.messageChatAddMembers");
    assert_eq!(join.membership_state, "present");
}

#[test]
fn ignores_service_messages_for_other_members_or_unsupported_content() {
    assert!(
        tdlib_self_membership_lifecycle(
            "telegram:42",
            &json!({
                "content": {
                    "@type": "messageChatDeleteMember",
                    "user_id": 7
                }
            }),
        )
        .is_none()
    );
    assert!(
        tdlib_self_membership_lifecycle(
            "telegram:42",
            &json!({
                "content": {
                    "@type": "messageChatJoinByLink"
                }
            }),
        )
        .is_none()
    );
}

#[test]
fn derives_inactive_roster_membership_state_from_status_or_role() {
    let member = TelegramChatMember {
        sender_id: "user:42".to_owned(),
        sender_display_name: None,
        message_count: 0,
        last_message_at: None,
        source: "tdlib".to_owned(),
        provider_member_id: "user:42".to_owned(),
        username: None,
        role: Some("member".to_owned()),
        status: Some("left".to_owned()),
        is_admin: false,
        is_owner: false,
        permissions: json!({}),
        observed_at: None,
    };
    assert_eq!(inactive_roster_membership_state(&member), Some("left"));

    let banned = TelegramChatMember {
        status: Some("administrator".to_owned()),
        role: Some("banned".to_owned()),
        ..member
    };
    assert_eq!(inactive_roster_membership_state(&banned), Some("banned"));
}

#[tokio::test]
async fn marks_stale_tdlib_participants_as_absent_from_exhaustive_roster() {
    use crate::platform::storage::database::Database;
    use makosh_backend_testkit::context::TestContext;

    let ctx = TestContext::new().await;
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();

    crate::test_support::upsert_telegram_runtime_account(
        &pool,
        "acct-1",
        "Telegram Test Account",
        "telegram:1",
    )
    .await;

    let store = crate::test_support::telegram_store(&pool);
    let chat = store
        .upsert_chat(&NewTelegramChat {
            account_id: "acct-1".to_owned(),
            provider_chat_id: "provider-chat-1".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Roster Room".to_owned(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({}),
        })
        .await
        .expect("insert telegram chat");

    for (participant_id, provider_member_id, display_name) in [
        ("participant-1", "user:1", "User One"),
        ("participant-2", "user:2", "User Two"),
    ] {
        upsert_chat_participant(
            &pool,
            &NewTelegramChatParticipant {
                participant_id: participant_id.to_owned(),
                telegram_chat_id: chat.telegram_chat_id.clone(),
                account_id: "acct-1".to_owned(),
                provider_chat_id: "provider-chat-1".to_owned(),
                provider_member_id: provider_member_id.to_owned(),
                display_name: Some(display_name.to_owned()),
                username: None,
                role: "member".to_owned(),
                status: "member".to_owned(),
                is_admin: false,
                is_owner: false,
                permissions: json!({}),
                raw_payload: json!({}),
                source: "tdlib".to_owned(),
            },
        )
        .await
        .expect("insert participant");
    }

    let updated = mark_absent_members_from_exhaustive_roster(
        &pool,
        &chat.telegram_chat_id,
        &[String::from("user:1")],
        "tdlib.getSupergroupMembers.exhaustive_absence",
    )
    .await
    .expect("mark absent");

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].provider_member_id, "user:2");
    assert_eq!(updated[0].status.as_deref(), Some("absent_exhaustive"));
    assert_eq!(
        updated[0].permissions["membership_state"],
        "absent_exhaustive"
    );
}
