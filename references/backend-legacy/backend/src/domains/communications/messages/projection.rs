use serde_json::{Map, Value};

use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::platform::communications::email_sync::imap_mailbox_stream_id;
use crate::platform::communications::rfc822::models::ParsedCommunicationSourceMessage;
use crate::platform::communications::rfc822::parser::parse_rfc822_message;
use makosh_communications_api::evidence::StoredRawCommunicationRecord;

use super::errors::MessageProjectionError;
use super::ids::message_id;
use super::models::{NewProjectedMessage, ProjectedMessage};
use super::payload::{required_payload_string, required_payload_string_array};
use super::store::MessageProjectionStore;

pub async fn project_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let subject = required_payload_string(&raw.payload, "subject")?;
    let sender = required_payload_string(&raw.payload, "from")?;
    let recipients = required_payload_string_array(&raw.payload, "to")?;
    let body_text = required_payload_string(&raw.payload, "body_text")?;
    let provider_record_id = canonical_email_provider_record_id(raw);
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id,
        subject,
        sender: sender.clone(),
        recipients,
        body_text,
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some(sender.clone()),
        delivery_state: raw_email_delivery_state(raw),
        message_metadata: raw_email_message_metadata(raw),
    };

    store.upsert_email_message(&message).await
}

pub async fn project_raw_email_message_from_blob(
    store: &MessageProjectionStore,
    blob_store: &LocalCommunicationBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let parsed = parse_raw_email_message_from_blob(blob_store, raw).await?;
    project_parsed_raw_email_message(store, raw, &parsed).await
}

pub async fn parse_raw_email_message_from_blob(
    blob_store: &LocalCommunicationBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ParsedCommunicationSourceMessage, MessageProjectionError> {
    let storage_kind = required_payload_string(&raw.payload, "raw_blob_storage_kind")?;
    if storage_kind != "local_fs" {
        return Err(MessageProjectionError::UnsupportedRawBlobStorageKind(
            storage_kind,
        ));
    }
    let storage_path = required_payload_string(&raw.payload, "raw_blob_storage_path")?;
    let bytes = blob_store.read_blob(&storage_path).await?;
    Ok(parse_rfc822_message(&bytes)?)
}

pub async fn project_parsed_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
    parsed: &ParsedCommunicationSourceMessage,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let provider_record_id = canonical_email_provider_record_id(raw);
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id,
        subject: parsed.subject.clone(),
        sender: parsed.from.clone(),
        recipients: parsed.to.clone(),
        body_text: parsed.body_text.clone(),
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some(parsed.from.clone()),
        delivery_state: raw_email_delivery_state(raw),
        message_metadata: raw_email_message_metadata_from_parsed(raw, parsed),
    };

    store.upsert_email_message(&message).await
}

fn raw_email_message_metadata(raw: &StoredRawCommunicationRecord) -> Value {
    let mut metadata = Map::new();
    copy_raw_email_metadata_field(raw, &mut metadata, "provider");
    copy_raw_email_metadata_field(raw, &mut metadata, "transport");
    copy_raw_email_metadata_field(raw, &mut metadata, "mailbox");
    copy_raw_email_metadata_field(raw, &mut metadata, "uid");
    copy_raw_email_metadata_field(raw, &mut metadata, "uid_validity");
    copy_raw_email_metadata_field(raw, &mut metadata, "rfc822_message_id");
    copy_raw_email_metadata_field(raw, &mut metadata, "label_ids");
    copy_raw_email_metadata_field(raw, &mut metadata, "is_read");
    if let Some(starred) = provider_starred_state(raw) {
        metadata.insert("starred".to_owned(), Value::Bool(starred));
        metadata.insert(
            "starred_origin".to_owned(),
            Value::String("provider_observed".to_owned()),
        );
    }
    Value::Object(metadata)
}

fn provider_starred_state(raw: &StoredRawCommunicationRecord) -> Option<bool> {
    if raw.payload.get("provider").and_then(Value::as_str) == Some("gmail") {
        let labels = raw.payload.get("label_ids").and_then(Value::as_array)?;
        return Some(labels.iter().any(|label| label.as_str() == Some("STARRED")));
    }

    if raw.payload.get("transport").and_then(Value::as_str) == Some("imap") {
        return raw.payload.get("is_starred").and_then(Value::as_bool);
    }

    None
}

fn raw_email_message_metadata_from_parsed(
    raw: &StoredRawCommunicationRecord,
    parsed: &ParsedCommunicationSourceMessage,
) -> Value {
    let mut metadata = raw_email_message_metadata(raw);
    let Some(message_id) = normalized_rfc822_message_id(&parsed.headers) else {
        return metadata;
    };
    if let Some(object) = metadata.as_object_mut() {
        object.insert("rfc822_message_id".to_owned(), Value::String(message_id));
    }
    metadata
}

fn normalized_rfc822_message_id(headers: &[(String, String)]) -> Option<String> {
    let value = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("message-id"))?
        .1
        .trim();
    let value = value.strip_prefix('<')?.strip_suffix('>')?;
    if value.is_empty()
        || value.len() > 996
        || value
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return None;
    }
    Some(format!("<{value}>"))
}

fn canonical_email_provider_record_id(raw: &StoredRawCommunicationRecord) -> String {
    let Some(mailbox) = raw.payload.get("mailbox").and_then(Value::as_str) else {
        return raw.provider_record_id.clone();
    };
    let Some(uid_validity) = raw
        .payload
        .get("uid_validity")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
    else {
        return raw.provider_record_id.clone();
    };
    let Some(uid) = raw
        .payload
        .get("uid")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
    else {
        return raw.provider_record_id.clone();
    };
    if raw.payload.get("transport").and_then(Value::as_str) != Some("imap")
        || mailbox.trim().is_empty()
    {
        return raw.provider_record_id.clone();
    }

    format!(
        "imap:v2:{}:{uid_validity}:{uid}",
        imap_mailbox_stream_id(mailbox)
    )
}

fn raw_email_delivery_state(raw: &StoredRawCommunicationRecord) -> String {
    if raw_email_has_gmail_sent_label(raw) {
        return "sent".to_owned();
    }
    "received".to_owned()
}

fn raw_email_has_gmail_sent_label(raw: &StoredRawCommunicationRecord) -> bool {
    if raw.payload.get("provider").and_then(Value::as_str) != Some("gmail") {
        return false;
    }

    raw.payload
        .get("label_ids")
        .and_then(Value::as_array)
        .is_some_and(|label_ids| {
            label_ids
                .iter()
                .filter_map(Value::as_str)
                .any(|label_id| label_id == "SENT")
        })
}

fn copy_raw_email_metadata_field(
    raw: &StoredRawCommunicationRecord,
    metadata: &mut Map<String, Value>,
    field: &'static str,
) {
    if let Some(value) = raw.payload.get(field) {
        metadata.insert(field.to_owned(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{
        ParsedCommunicationSourceMessage, StoredRawCommunicationRecord,
        normalized_rfc822_message_id, provider_starred_state, raw_email_delivery_state,
        raw_email_message_metadata, raw_email_message_metadata_from_parsed,
    };

    #[test]
    fn raw_email_message_metadata_preserves_provider_mailbox_identity() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_mailbox_metadata".to_owned(),
            observation_id: "observation:v1:raw-mailbox-metadata".to_owned(),
            account_id: "acct_mailbox_metadata".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "imap:Junk:42".to_owned(),
            source_fingerprint: "sha256:raw-mailbox-metadata".to_owned(),
            import_batch_id: "batch_mailbox_metadata".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "provider": "icloud",
                "transport": "imap",
                "mailbox": "Junk",
                "uid": 42,
                "uid_validity": 7,
                "is_read": true,
                "is_starred": true,
                "label_ids": ["INBOX"],
                "raw_blob_storage_path": "/private/mail/blob"
            }),
            provenance: json!({"source": "unit_test"}),
        };

        let metadata = raw_email_message_metadata(&raw);

        assert_eq!(metadata["provider"], json!("icloud"));
        assert_eq!(metadata["transport"], json!("imap"));
        assert_eq!(metadata["mailbox"], json!("Junk"));
        assert_eq!(metadata["uid"], json!(42));
        assert_eq!(metadata["uid_validity"], json!(7));
        assert_eq!(metadata["is_read"], json!(true));
        assert_eq!(metadata["label_ids"], json!(["INBOX"]));
        assert_eq!(metadata["starred"], json!(true));
        assert_eq!(metadata["starred_origin"], json!("provider_observed"));
        assert!(metadata.get("raw_blob_storage_path").is_none());
    }

    #[test]
    fn provider_starred_state_uses_gmail_starred_label() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_gmail_starred".to_owned(),
            observation_id: "observation:v1:raw-gmail-starred".to_owned(),
            account_id: "acct_gmail_starred".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "gmail-starred".to_owned(),
            source_fingerprint: "sha256:raw-gmail-starred".to_owned(),
            import_batch_id: "batch_gmail_starred".to_owned(),
            occurred_at: None,
            captured_at: Utc::now(),
            payload: json!({
                "provider": "gmail",
                "label_ids": ["INBOX", "STARRED"]
            }),
            provenance: json!({"source": "unit_test"}),
        };

        assert_eq!(provider_starred_state(&raw), Some(true));
    }

    #[test]
    fn raw_email_delivery_state_does_not_infer_sent_from_mailbox_name() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_sent_mailbox".to_owned(),
            observation_id: "observation:v1:raw-sent-mailbox".to_owned(),
            account_id: "acct_sent_mailbox".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "imap:Sent Messages:42".to_owned(),
            source_fingerprint: "sha256:raw-sent-mailbox".to_owned(),
            import_batch_id: "batch_sent_mailbox".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "mailbox": "Sent Messages"
            }),
            provenance: json!({"source": "unit_test"}),
        };

        assert_eq!(raw_email_delivery_state(&raw), "received");
    }

    #[test]
    fn normalized_rfc822_message_id_accepts_a_single_bracketed_header_value() {
        let headers = vec![
            ("Subject".to_owned(), "Example".to_owned()),
            (
                "Message-ID".to_owned(),
                "<stable-id@example.test>".to_owned(),
            ),
        ];

        assert_eq!(
            normalized_rfc822_message_id(&headers),
            Some("<stable-id@example.test>".to_owned())
        );
    }

    #[test]
    fn normalized_rfc822_message_id_rejects_whitespace_and_unbracketed_values() {
        assert_eq!(
            normalized_rfc822_message_id(&[(
                "Message-ID".to_owned(),
                "stable id@example.test".to_owned(),
            )]),
            None
        );
        assert_eq!(
            normalized_rfc822_message_id(&[(
                "Message-ID".to_owned(),
                "stable-id@example.test".to_owned(),
            )]),
            None
        );
    }

    #[test]
    fn parsed_projection_metadata_preserves_valid_rfc822_message_id() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_rfc822_message_id".to_owned(),
            observation_id: "observation:v1:raw-rfc822-message-id".to_owned(),
            account_id: "acct_rfc822_message_id".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "imap:v2:imap:INBOX:7:42".to_owned(),
            source_fingerprint: "sha256:rfc822-message-id".to_owned(),
            import_batch_id: "batch_rfc822_message_id".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "provider": "imap",
                "transport": "imap",
                "mailbox": "INBOX",
                "uid": 42,
                "uid_validity": 7
            }),
            provenance: json!({"source": "unit_test"}),
        };
        let parsed = ParsedCommunicationSourceMessage {
            subject: "Subject".to_owned(),
            from: "alice@example.test".to_owned(),
            to: vec!["bob@example.test".to_owned()],
            headers: vec![(
                "Message-ID".to_owned(),
                "<stable-id@example.test>".to_owned(),
            )],
            body_text: "Body".to_owned(),
            body_html: None,
            attachments: Vec::new(),
        };

        let metadata = raw_email_message_metadata_from_parsed(&raw, &parsed);

        assert_eq!(
            metadata["rfc822_message_id"],
            json!("<stable-id@example.test>")
        );
    }

    #[test]
    fn raw_email_delivery_state_marks_gmail_sent_label_as_sent() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_gmail_sent_label".to_owned(),
            observation_id: "observation:v1:raw-gmail-sent-label".to_owned(),
            account_id: "acct_gmail_sent_label".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "gmail-sent-message-42".to_owned(),
            source_fingerprint: "sha256:raw-gmail-sent-label".to_owned(),
            import_batch_id: "batch_gmail_sent_label".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "provider": "gmail",
                "label_ids": ["SENT", "CATEGORY_PERSONAL"]
            }),
            provenance: json!({"source": "unit_test"}),
        };

        assert_eq!(raw_email_delivery_state(&raw), "sent");
    }

    #[test]
    fn raw_email_delivery_state_does_not_apply_gmail_labels_to_other_providers() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_imap_sent_label".to_owned(),
            observation_id: "observation:v1:raw-imap-sent-label".to_owned(),
            account_id: "acct_imap_sent_label".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "imap:INBOX:42".to_owned(),
            source_fingerprint: "sha256:raw-imap-sent-label".to_owned(),
            import_batch_id: "batch_imap_sent_label".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "provider": "imap",
                "label_ids": ["SENT"]
            }),
            provenance: json!({"source": "unit_test"}),
        };

        assert_eq!(raw_email_delivery_state(&raw), "received");
    }
}
