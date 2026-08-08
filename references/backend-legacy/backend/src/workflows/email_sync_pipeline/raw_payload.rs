use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use serde_json::{Value, json};

use crate::domains::communications::storage::models::StoredCommunicationBlob;
use makosh_communications_api::accounts::CommunicationProviderKind;

use super::errors::EmailSyncRecordError;

pub(super) fn raw_message_bytes(
    provider_kind: CommunicationProviderKind,
    payload: &Value,
) -> Result<Vec<u8>, EmailSyncRecordError> {
    match provider_kind {
        CommunicationProviderKind::Gmail => {
            let raw = required_payload_string(payload, "raw_base64url")?;
            URL_SAFE_NO_PAD
                .decode(raw)
                .or_else(|_| URL_SAFE.decode(raw))
                .map_err(|source| EmailSyncRecordError::InvalidRawPayloadBase64 {
                    field: "raw_base64url",
                    source,
                })
        }
        CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap => {
            let raw = required_payload_string(payload, "raw_rfc822_base64")?;
            BASE64_STANDARD.decode(raw).map_err(|source| {
                EmailSyncRecordError::InvalidRawPayloadBase64 {
                    field: "raw_rfc822_base64",
                    source,
                }
            })
        }
        CommunicationProviderKind::TelegramUser
        | CommunicationProviderKind::TelegramBot
        | CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud
        | CommunicationProviderKind::ZulipBot
        | CommunicationProviderKind::ZoomUser
        | CommunicationProviderKind::ZoomServerToServer
        | CommunicationProviderKind::YandexTelemostUser => Err(
            EmailSyncRecordError::UnsupportedProviderKind(provider_kind.as_str().to_owned()),
        ),
    }
}

pub(super) fn payload_with_raw_blob_reference(
    payload: &Value,
    blob: &StoredCommunicationBlob,
) -> Result<Value, EmailSyncRecordError> {
    let Some(object) = payload.as_object() else {
        return Err(EmailSyncRecordError::InvalidRawPayloadObject);
    };
    let mut object = object.clone();
    object.remove("raw_base64url");
    object.remove("raw_rfc822_base64");
    object.insert("raw_blob_id".to_owned(), json!(blob.blob_id));
    object.insert("raw_blob_sha256".to_owned(), json!(blob.sha256));
    object.insert("raw_blob_storage_kind".to_owned(), json!(blob.storage_kind));
    object.insert("raw_blob_storage_path".to_owned(), json!(blob.storage_path));
    object.insert("raw_blob_size_bytes".to_owned(), json!(blob.size_bytes));

    Ok(Value::Object(object))
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, EmailSyncRecordError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .ok_or(EmailSyncRecordError::MissingRawPayloadField { field })
}
