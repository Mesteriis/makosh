//! Typed Zulip operational contract. It contains no transport or domain dependency.

pub mod account;
pub mod account_wire;
pub mod client_contract;
pub mod client_wire;
pub mod operational;
pub mod operational_wire;
pub mod realtime;
pub mod realtime_wire;
#[allow(clippy::large_enum_variant)]
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.zulip.v1.rs"));
}
#[allow(clippy::large_enum_variant)]
pub mod operational_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.zulip.operational.v1.rs"));
}
#[allow(clippy::large_enum_variant)]
pub mod realtime_wire_generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.zulip.operational.realtime.v1.rs"
    ));
}
#[allow(clippy::large_enum_variant)]
pub mod account_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.zulip.account.v1.rs"));
}

pub const PACKAGE: &str = "makosh-zulip-api";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipAccountV1 {
    pub account_id: String,
    pub realm_url: String,
    pub account_email: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipEventQueueV1 {
    pub queue_id: String,
    pub last_event_id: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipPolledEventV1 {
    pub event_id: i64,
    pub observations: Vec<ZulipEventV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipHistoryPageV1 {
    pub messages: Vec<ZulipMessageSnapshotV1>,
    pub oldest_provider_message_id: Option<String>,
    pub found_oldest: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipMessageSnapshotV1 {
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_conversation_id: String,
    pub conversation_kind: operational::ZulipConversationKindV1,
    pub stream_id: Option<String>,
    pub stream_name: Option<String>,
    pub topic: Option<String>,
    pub direct_recipient_id: Option<String>,
    pub sender_id: String,
    pub is_outgoing: bool,
    pub content: Option<String>,
    pub sent_at_unix_seconds: Option<i64>,
    pub edited_at_unix_seconds: Option<i64>,
    pub attachments: Vec<ZulipAttachmentV1>,
    pub reactions: Vec<operational::ZulipReactionStateV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipAttachmentV1 {
    pub provider_attachment_id: String,
    pub filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipReactionV1 {
    pub emoji_name: String,
    pub emoji_code: Option<String>,
    pub reaction_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipBlobIntentV1 {
    pub blob_ref: String,
    pub reference_id: Vec<u8>,
    pub declared_size: u64,
    pub backup_class: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipReactionOperationV1 {
    Add,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipCommandV1 {
    SendStream {
        operation_id: String,
        account_id: String,
        stream: String,
        topic: String,
        content: String,
    },
    SendDirect {
        operation_id: String,
        account_id: String,
        recipients: Vec<String>,
        content: String,
    },
    UpdateMessage {
        operation_id: String,
        account_id: String,
        provider_message_id: String,
        content: Option<String>,
        topic: Option<String>,
    },
    DeleteMessage {
        operation_id: String,
        account_id: String,
        provider_message_id: String,
    },
    Reaction {
        operation_id: String,
        account_id: String,
        provider_message_id: String,
        reaction: ZulipReactionV1,
        operation: ZulipReactionOperationV1,
    },
    SendStreamWithUpload {
        operation_id: String,
        account_id: String,
        stream: String,
        topic: String,
        content: String,
        blob: ZulipBlobIntentV1,
        filename: String,
    },
    SendDirectWithUpload {
        operation_id: String,
        account_id: String,
        recipients: Vec<String>,
        content: String,
        blob: ZulipBlobIntentV1,
        filename: String,
    },
    DownloadAttachment {
        operation_id: String,
        account_id: String,
        upload_path: String,
        blob: ZulipBlobIntentV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipCommandOperationOutcomeV1 {
    OutcomeUnknown,
    Accepted {
        provider_message_id: Option<i64>,
        blob_ref: Option<String>,
    },
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipCommandOperationStatusV1 {
    pub operation_id: String,
    pub account_id: String,
    pub outcome: ZulipCommandOperationOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipCommandReceiptV1 {
    pub operation_id: String,
    pub account_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipClientRequestV1 {
    AccountLifecycle(account::ZulipAccountLifecycleCommandV1),
    Command(ZulipCommandV1),
    OperationStatus { operation_id: String },
    OperationalQuery(operational::ZulipOperationalQueryV1),
    OperationalReplay(realtime::ZulipOperationalReplayRequestV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipClientResponseV1 {
    AccountLifecycle(account::ZulipAccountLifecycleReceiptV1),
    CommandReceipt(ZulipCommandReceiptV1),
    OperationStatus(Option<ZulipCommandOperationStatusV1>),
    OperationalQuery(operational::ZulipOperationalQueryResponseV1),
    OperationalReplay(realtime::ZulipOperationalReplayResponseV1),
}

#[must_use]
pub fn command_operation_id(command: &ZulipCommandV1) -> &str {
    match command {
        ZulipCommandV1::SendStream { operation_id, .. }
        | ZulipCommandV1::SendDirect { operation_id, .. }
        | ZulipCommandV1::UpdateMessage { operation_id, .. }
        | ZulipCommandV1::DeleteMessage { operation_id, .. }
        | ZulipCommandV1::Reaction { operation_id, .. }
        | ZulipCommandV1::SendStreamWithUpload { operation_id, .. }
        | ZulipCommandV1::SendDirectWithUpload { operation_id, .. } => operation_id,
        ZulipCommandV1::DownloadAttachment { operation_id, .. } => operation_id,
    }
}

#[must_use]
pub fn command_account_id(command: &ZulipCommandV1) -> &str {
    match command {
        ZulipCommandV1::SendStream { account_id, .. }
        | ZulipCommandV1::SendDirect { account_id, .. }
        | ZulipCommandV1::UpdateMessage { account_id, .. }
        | ZulipCommandV1::DeleteMessage { account_id, .. }
        | ZulipCommandV1::Reaction { account_id, .. }
        | ZulipCommandV1::SendStreamWithUpload { account_id, .. }
        | ZulipCommandV1::SendDirectWithUpload { account_id, .. } => account_id,
        ZulipCommandV1::DownloadAttachment { account_id, .. } => account_id,
    }
}

/// Returns a versioned, unambiguous provider-command representation for an
/// idempotency fingerprint. It is not a wire protocol and must never be logged.
#[must_use]
pub fn command_fingerprint_bytes(command: &ZulipCommandV1) -> Vec<u8> {
    let mut bytes = vec![1];
    match command {
        ZulipCommandV1::SendStream {
            operation_id,
            account_id,
            stream,
            topic,
            content,
        } => {
            bytes.push(1);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            append_text(&mut bytes, stream);
            append_text(&mut bytes, topic);
            append_text(&mut bytes, content);
        }
        ZulipCommandV1::SendDirect {
            operation_id,
            account_id,
            recipients,
            content,
        } => {
            bytes.push(2);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            bytes.extend_from_slice(
                &(u32::try_from(recipients.len()).unwrap_or(u32::MAX)).to_be_bytes(),
            );
            for recipient in recipients {
                append_text(&mut bytes, recipient);
            }
            append_text(&mut bytes, content);
        }
        ZulipCommandV1::UpdateMessage {
            operation_id,
            account_id,
            provider_message_id,
            content,
            topic,
        } => {
            bytes.push(3);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            append_text(&mut bytes, provider_message_id);
            append_optional_text(&mut bytes, content.as_deref());
            append_optional_text(&mut bytes, topic.as_deref());
        }
        ZulipCommandV1::DeleteMessage {
            operation_id,
            account_id,
            provider_message_id,
        } => {
            bytes.push(4);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            append_text(&mut bytes, provider_message_id);
        }
        ZulipCommandV1::Reaction {
            operation_id,
            account_id,
            provider_message_id,
            reaction,
            operation,
        } => {
            bytes.push(5);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            append_text(&mut bytes, provider_message_id);
            append_text(&mut bytes, &reaction.emoji_name);
            append_optional_text(&mut bytes, reaction.emoji_code.as_deref());
            append_optional_text(&mut bytes, reaction.reaction_type.as_deref());
            bytes.push(match operation {
                ZulipReactionOperationV1::Add => 1,
                ZulipReactionOperationV1::Remove => 2,
            });
        }
        ZulipCommandV1::SendStreamWithUpload {
            operation_id,
            account_id,
            stream,
            topic,
            content,
            blob,
            filename,
        } => {
            bytes.push(6);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            append_text(&mut bytes, stream);
            append_text(&mut bytes, topic);
            append_text(&mut bytes, content);
            append_blob_intent(&mut bytes, blob);
            append_text(&mut bytes, filename);
        }
        ZulipCommandV1::SendDirectWithUpload {
            operation_id,
            account_id,
            recipients,
            content,
            blob,
            filename,
        } => {
            bytes.push(7);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            bytes.extend_from_slice(
                &(u32::try_from(recipients.len()).unwrap_or(u32::MAX)).to_be_bytes(),
            );
            for recipient in recipients {
                append_text(&mut bytes, recipient);
            }
            append_text(&mut bytes, content);
            append_blob_intent(&mut bytes, blob);
            append_text(&mut bytes, filename);
        }
        ZulipCommandV1::DownloadAttachment {
            operation_id,
            account_id,
            upload_path,
            blob,
        } => {
            bytes.push(8);
            append_text(&mut bytes, operation_id);
            append_text(&mut bytes, account_id);
            append_text(&mut bytes, upload_path);
            append_blob_intent(&mut bytes, blob);
        }
    }
    bytes
}

#[must_use]
pub fn command_blob_intent(command: &ZulipCommandV1) -> Option<&ZulipBlobIntentV1> {
    match command {
        ZulipCommandV1::SendStreamWithUpload { blob, .. }
        | ZulipCommandV1::SendDirectWithUpload { blob, .. }
        | ZulipCommandV1::DownloadAttachment { blob, .. } => Some(blob),
        _ => None,
    }
}

fn append_text(bytes: &mut Vec<u8>, value: &str) {
    let length = u32::try_from(value.len()).unwrap_or(u32::MAX);
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn append_optional_text(bytes: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            bytes.push(1);
            append_text(bytes, value);
        }
        None => bytes.push(0),
    }
}

fn append_blob_intent(bytes: &mut Vec<u8>, value: &ZulipBlobIntentV1) {
    append_text(bytes, &value.blob_ref);
    bytes.extend_from_slice(
        &(u32::try_from(value.reference_id.len()).unwrap_or(u32::MAX)).to_be_bytes(),
    );
    bytes.extend_from_slice(&value.reference_id);
    bytes.extend_from_slice(&value.declared_size.to_be_bytes());
    bytes.extend_from_slice(&value.backup_class.to_be_bytes());
}

#[cfg(test)]
mod generated_client_wire_tests {
    use super::{
        ZulipCommandOperationOutcomeV1, ZulipCommandOperationStatusV1, ZulipCommandReceiptV1,
        ZulipCommandV1,
        client_wire::{
            decode_command_request, decode_command_response, decode_operation_status_query,
            decode_operation_status_response, encode_command_request, encode_command_response,
            encode_operation_status_query, encode_operation_status_response,
        },
    };

    #[test]
    fn preserves_exact_route_specific_command_and_status_payloads() {
        let command = ZulipCommandV1::SendDirectWithUpload {
            operation_id: "operation".into(),
            account_id: "account".into(),
            recipients: vec!["41".into(), "42".into()],
            content: "private body".into(),
            blob: super::ZulipBlobIntentV1 {
                blob_ref: "blob-ref".into(),
                reference_id: vec![1; 16],
                declared_size: 3,
                backup_class: 1,
            },
            filename: "attachment.txt".into(),
        };
        assert_eq!(
            decode_command_request(&encode_command_request(&command)),
            Ok(command)
        );
        assert_eq!(
            decode_operation_status_query(&encode_operation_status_query("operation")),
            Ok("operation".to_owned())
        );
        let receipt = ZulipCommandReceiptV1 {
            operation_id: "operation".into(),
            account_id: "account".into(),
        };
        assert_eq!(
            decode_command_response(&encode_command_response(&receipt)),
            Ok(receipt)
        );
        let status = ZulipCommandOperationStatusV1 {
            operation_id: "operation".into(),
            account_id: "account".into(),
            outcome: ZulipCommandOperationOutcomeV1::Accepted {
                provider_message_id: Some(7),
                blob_ref: None,
            },
            requested_at_unix_seconds: 1,
            completed_at_unix_seconds: Some(2),
        };
        assert_eq!(
            decode_operation_status_response(&encode_operation_status_response(Some(&status))),
            Ok(Some(status))
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZulipEventV1 {
    Message {
        account_id: String,
        event_id: i64,
        provider_message_id: String,
        provider_conversation_id: String,
        conversation_kind: operational::ZulipConversationKindV1,
        stream_id: Option<String>,
        stream_name: Option<String>,
        topic: Option<String>,
        direct_recipient_id: Option<String>,
        sender_id: String,
        is_outgoing: bool,
        content: Option<String>,
        sent_at_unix_seconds: Option<i64>,
        attachments: Vec<ZulipAttachmentV1>,
        reactions: Vec<operational::ZulipReactionStateV1>,
    },
    MessageUpdated {
        account_id: String,
        event_id: i64,
        provider_message_id: String,
        content: Option<String>,
        topic: Option<String>,
        edited_at_unix_seconds: Option<i64>,
    },
    MessageDeleted {
        account_id: String,
        event_id: i64,
        provider_message_id: String,
    },
    ReactionChanged {
        account_id: String,
        event_id: i64,
        provider_message_id: String,
        actor_id: String,
        reaction: operational::ZulipReactionStateV1,
        operation: ZulipReactionOperationV1,
    },
}

#[must_use]
pub fn validate_account(account: &ZulipAccountV1) -> bool {
    !account.account_id.trim().is_empty()
        && account.realm_url.starts_with("https://")
        && account.account_email.contains('@')
}

#[must_use]
pub fn direct_recipient_user_ids(recipients: &[String]) -> Option<Vec<i64>> {
    (!recipients.is_empty()).then(|| {
        recipients
            .iter()
            .map(|value| value.parse::<i64>().ok().filter(|value| *value > 0))
            .collect()
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_the_direct_recipient_identifier_mode() {
        assert_eq!(
            direct_recipient_user_ids(&["1".into(), "2".into()]),
            Some(vec![1, 2])
        );
        assert_eq!(
            direct_recipient_user_ids(&["owner@example.test".into()]),
            None
        );
        assert_eq!(direct_recipient_user_ids(&["0".into()]), None);
    }

    #[test]
    fn command_fingerprint_is_unambiguous_and_payload_sensitive() {
        let first = ZulipCommandV1::SendStream {
            operation_id: "op".into(),
            account_id: "account".into(),
            stream: "a".into(),
            topic: "bc".into(),
            content: "body".into(),
        };
        let second = ZulipCommandV1::SendStream {
            operation_id: "op".into(),
            account_id: "account".into(),
            stream: "ab".into(),
            topic: "c".into(),
            content: "body".into(),
        };
        assert_ne!(
            command_fingerprint_bytes(&first),
            command_fingerprint_bytes(&second)
        );
        assert_eq!(command_operation_id(&first), "op");
    }
}
