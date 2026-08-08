use chrono::{DateTime, Utc};
use makosh_provider_zulip::{
    event_mapper::{
        ZulipEventMappingContext, map_zulip_event_to_raw_record, zulip_raw_signal_event_type,
        zulip_raw_signal_event_types,
    },
    models::ZulipEvent,
};
use serde_json::json;

fn context() -> ZulipEventMappingContext {
    let received_at = DateTime::parse_from_rfc3339("2026-06-29T08:00:00Z")
        .expect("valid timestamp")
        .with_timezone(&Utc);
    ZulipEventMappingContext::new("lab-zulip-account", "http://localhost:8080", received_at)
}

#[test]
fn maps_message_event_to_raw_signal_ready_record() {
    let event: ZulipEvent = serde_json::from_value(json!({
        "id": 42, "type": "message", "message": {
            "id": 7001, "content": "Надо проверить backup retention до пятницы.",
            "sender_email": "bot@example.test", "sender_full_name": "Макошь Bot",
            "stream_id": 10, "display_recipient": "Макошь Lab", "subject": "Tasks"
        }
    }))
    .expect("valid Zulip event fixture");
    let context = context()
        .with_import_batch_id("zulip-lab-batch")
        .with_correlation_id("lab-zulip-task-001")
        .with_scenario_id("zulip-message-to-task-run");

    let raw_record =
        map_zulip_event_to_raw_record(&event, &context).expect("message event maps to raw record");

    assert_eq!(raw_record.account_id, "lab-zulip-account");
    assert_eq!(raw_record.record_kind, "zulip_message");
    assert_eq!(raw_record.provider_record_id, "7001");
    assert_eq!(raw_record.import_batch_id, "zulip-lab-batch");
    assert_eq!(raw_record.payload["provider_message_id"], json!("7001"));
    assert_eq!(raw_record.payload["stream_name"], json!("Макошь Lab"));
    assert_eq!(raw_record.payload["topic"], json!("Tasks"));
    assert_eq!(
        raw_record.payload["content"],
        json!("Надо проверить backup retention до пятницы.")
    );
    assert_eq!(raw_record.provenance["provider_kind"], json!("zulip_bot"));
    assert_eq!(
        raw_record.provenance["scenario_run_id"],
        json!("zulip-message-to-task-run")
    );
    assert!(raw_record.source_fingerprint.starts_with("sha256:"));
    assert!(raw_record.observation_id.starts_with("raw_zulip_"));
}

#[test]
fn maps_message_attachment_metadata_without_materializing_bytes() {
    let event: ZulipEvent = serde_json::from_value(json!({
        "id": 46, "type": "message", "message": {
            "id": 7002, "content": "См. вложение.", "sender_email": "bot@example.test",
            "sender_full_name": "Макошь Bot", "stream_id": 10, "display_recipient": "Макошь Lab",
            "topic": "Evidence", "attachments": [{
                "id": "zulip-file-1", "name": "evidence.pdf", "content_type": "application/pdf",
                "size": 2048, "url": "/user_uploads/1/evidence.pdf", "path_id": "1/evidence.pdf"
            }]
        }
    }))
    .expect("valid Zulip event fixture");
    let raw_record = map_zulip_event_to_raw_record(&event, &context())
        .expect("message event maps to raw record");

    assert_eq!(
        raw_record.payload["attachments"][0]["provider"],
        json!("zulip")
    );
    assert_eq!(
        raw_record.payload["attachments"][0]["provider_attachment_id"],
        json!("zulip-file-1")
    );
    assert_eq!(
        raw_record.payload["attachments"][0]["filename"],
        json!("evidence.pdf")
    );
    assert_eq!(
        raw_record.payload["attachments"][0]["bytes_state"],
        json!("not_transferred")
    );
    assert_eq!(
        raw_record.payload["attachments"][0]["scan_status"],
        json!("not_scanned")
    );
    assert_eq!(
        raw_record.payload["attachment_state"]["materialization_state"],
        json!("not_materialized")
    );
    assert_eq!(
        raw_record.payload["raw_event"]["message"]["attachments"][0]["url"],
        json!("/user_uploads/1/evidence.pdf")
    );
}

#[test]
fn maps_user_upload_links_from_message_content_as_attachment_evidence() {
    let event: ZulipEvent = serde_json::from_value(json!({
        "id": 47, "type": "message", "message": {
            "id": 7003, "content": "<p><a href=\"/user_uploads/2/ab/makosh-fact.txt\">makosh-fact.txt</a></p>",
            "sender_email": "bot@example.test", "sender_full_name": "Макошь Bot", "stream_id": 10,
            "display_recipient": "Макошь Lab", "topic": "Evidence"
        }
    }))
    .expect("valid Zulip event fixture");
    let raw_record = map_zulip_event_to_raw_record(&event, &context())
        .expect("message event maps to raw record");

    assert_eq!(
        raw_record.payload["attachments"][0]["provider_attachment_id"],
        json!("2/ab/makosh-fact.txt")
    );
    assert_eq!(
        raw_record.payload["attachments"][0]["filename"],
        json!("makosh-fact.txt")
    );
    assert_eq!(
        raw_record.payload["attachments"][0]["url"],
        json!("/user_uploads/2/ab/makosh-fact.txt")
    );
    assert_eq!(
        raw_record.payload["attachment_state"]["bytes_state"],
        json!("not_transferred")
    );
    assert_eq!(
        raw_record.payload["attachment_state"]["materialization_state"],
        json!("not_materialized")
    );
}

#[test]
fn raw_signal_event_types_match_signal_hub_contract() {
    assert_eq!(
        zulip_raw_signal_event_type("message"),
        zulip_raw_signal_event_types::MESSAGE
    );
    assert_eq!(
        zulip_raw_signal_event_type("reaction"),
        zulip_raw_signal_event_types::REACTION
    );
    assert_eq!(
        zulip_raw_signal_event_type("update_message"),
        zulip_raw_signal_event_types::MESSAGE_UPDATE
    );
    assert_eq!(
        zulip_raw_signal_event_type("delete_message"),
        zulip_raw_signal_event_types::MESSAGE_DELETE
    );
    assert_eq!(
        zulip_raw_signal_event_type("realm_emoji"),
        zulip_raw_signal_event_types::UNKNOWN
    );
}

#[test]
fn maps_non_message_events_to_target_message_payloads() {
    let reaction_event: ZulipEvent = serde_json::from_value(json!({
        "id": 43, "type": "reaction", "message_id": 7001, "emoji_name": "+1", "emoji_code": "1f44d",
        "reaction_type": "unicode_emoji", "op": "add", "user_id": 55,
        "user": {"full_name": "Zulip Reactor", "email": "reactor@example.test"}
    }))
    .expect("valid reaction event fixture");
    let reaction_raw = map_zulip_event_to_raw_record(&reaction_event, &context())
        .expect("reaction event maps to raw record");
    assert_eq!(reaction_raw.record_kind, "zulip_reaction");
    assert_eq!(reaction_raw.provider_record_id, "reaction:43");
    assert_eq!(reaction_raw.payload["provider_message_id"], json!("7001"));
    assert_eq!(reaction_raw.payload["emoji_name"], json!("+1"));
    assert_eq!(reaction_raw.payload["reaction_op"], json!("add"));
    assert_eq!(reaction_raw.payload["provider_actor_id"], json!("55"));
    assert_eq!(
        reaction_raw.payload["sender_display_name"],
        json!("Zulip Reactor")
    );

    let update_event: ZulipEvent = serde_json::from_value(json!({
        "id": 44, "type": "update_message", "message_id": 7001, "content": "Updated Zulip content",
        "prev_content": "Original Zulip content", "topic": "Tasks", "edit_timestamp": 1782720060
    }))
    .expect("valid message update event fixture");
    let update_raw = map_zulip_event_to_raw_record(&update_event, &context())
        .expect("message update event maps to raw record");
    assert_eq!(update_raw.record_kind, "zulip_message_update");
    assert_eq!(update_raw.provider_record_id, "update_message:44");
    assert_eq!(update_raw.payload["provider_message_id"], json!("7001"));
    assert_eq!(
        update_raw.payload["content"],
        json!("Updated Zulip content")
    );
    assert_eq!(
        update_raw.payload["prev_content"],
        json!("Original Zulip content")
    );
    assert_eq!(update_raw.payload["topic"], json!("Tasks"));
    assert_eq!(update_raw.payload["edit_timestamp"], json!("1782720060"));

    let delete_event: ZulipEvent = serde_json::from_value(json!({
        "id": 45, "type": "delete_message", "message_id": 7001, "message_type": "stream", "stream_id": 10, "topic": "Tasks"
    }))
    .expect("valid message delete event fixture");
    let delete_raw = map_zulip_event_to_raw_record(&delete_event, &context())
        .expect("message delete event maps to raw record");
    assert_eq!(delete_raw.record_kind, "zulip_message_delete");
    assert_eq!(delete_raw.provider_record_id, "delete_message:45");
    assert_eq!(delete_raw.payload["provider_message_id"], json!("7001"));
    assert_eq!(delete_raw.payload["message_type"], json!("stream"));
    assert_eq!(delete_raw.payload["stream_id"], json!("10"));
    assert_eq!(delete_raw.payload["topic"], json!("Tasks"));
}
