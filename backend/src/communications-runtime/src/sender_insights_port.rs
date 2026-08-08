//! Generated client port for provider-neutral Communications sender insights.

use makosh_communications_persistence::{
    CommunicationsDurablePersistence, CommunicationsSenderInsightAfterV1,
    CommunicationsSenderInsightV1, CommunicationsSenderInsightsErrorV1,
};
use makosh_communications_sender_insights_api::{
    COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256, ListSenderInsightsRequestV1,
    ListSenderInsightsResponseV1, SENDER_INSIGHTS_CONTRACT_MAJOR_V1,
    SENDER_INSIGHTS_CONTRACT_NAME_V1, SENDER_INSIGHTS_CONTRACT_REVISION_V1, SenderInsightV1,
    SenderInsightsErrorCodeV1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::admission::{COMMUNICATIONS_MODULE_ID, COMMUNICATIONS_OWNER_ID};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const CURSOR_PREFIX: &[u8; 4] = b"HSI1";
const CURSOR_SCOPE_BYTES: usize = 16;
const CURSOR_CHECKSUM_BYTES: usize = 16;
const CURSOR_BYTES: usize = 4 + CURSOR_SCOPE_BYTES + 8 + 8 + 16 + CURSOR_CHECKSUM_BYTES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSenderInsightsClientPortErrorV1 {
    Protocol,
    Unavailable,
}

pub fn encode_module_sender_insights_request_v1(
    request_id: u64,
    payload: &[u8],
) -> Result<Vec<u8>, CommunicationsSenderInsightsClientPortErrorV1> {
    if request_id == 0 || payload.is_empty() {
        return Err(CommunicationsSenderInsightsClientPortErrorV1::Protocol);
    }
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        owner_id: COMMUNICATIONS_OWNER_ID.to_owned(),
        contract: Some(sender_insights_contract()),
        request_id,
        request_payload: payload.to_vec(),
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec())
}

pub async fn handle_module_sender_insights_request_v1(
    persistence: &CommunicationsDurablePersistence,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsSenderInsightsClientPortErrorV1> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|_| CommunicationsSenderInsightsClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != COMMUNICATIONS_MODULE_ID
        || envelope.owner_id != COMMUNICATIONS_OWNER_ID
        || envelope.contract.as_ref() != Some(&sender_insights_contract())
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
    {
        return Err(CommunicationsSenderInsightsClientPortErrorV1::Protocol);
    }
    let request = ListSenderInsightsRequestV1::decode(envelope.request_payload.as_slice())
        .map_err(|_| CommunicationsSenderInsightsClientPortErrorV1::Protocol)?;
    let response = list_sender_insights(persistence, request).await;
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id: envelope.request_id,
        response_payload: response.encode_to_vec(),
        error_code: String::new(),
    }
    .encode_to_vec())
}

async fn list_sender_insights(
    persistence: &CommunicationsDurablePersistence,
    request: ListSenderInsightsRequestV1,
) -> ListSenderInsightsResponseV1 {
    if request.protocol_major != SENDER_INSIGHTS_CONTRACT_MAJOR_V1 {
        return error_response(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest);
    }
    let account_id = match request.account_id.as_deref().map(id16).transpose() {
        Ok(value) => value,
        Err(error) => return error_response(error),
    };
    let limit = match page_limit(request.limit) {
        Ok(limit) => limit,
        Err(error) => return error_response(error),
    };
    let after = match decode_cursor(&request.cursor, account_id) {
        Ok(value) => value,
        Err(error) => return error_response(error),
    };
    match persistence
        .list_sender_insights(account_id, after, limit)
        .await
    {
        Ok(page) => {
            let next_cursor = if page.has_more {
                page.items
                    .last()
                    .map(|item| encode_cursor(item, account_id))
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            ListSenderInsightsResponseV1 {
                items: page.items.iter().map(insight_to_wire).collect(),
                next_cursor,
                error: SenderInsightsErrorCodeV1::SenderInsightsErrorCodeUnspecified as i32,
            }
        }
        Err(error) => error_response(map_persistence_error(error)),
    }
}

fn encode_cursor(item: &CommunicationsSenderInsightV1, account_id: Option<[u8; 16]>) -> Vec<u8> {
    let mut cursor = Vec::with_capacity(CURSOR_BYTES);
    cursor.extend_from_slice(CURSOR_PREFIX);
    cursor.extend_from_slice(&cursor_scope(account_id));
    cursor.extend_from_slice(&item.message_count.to_be_bytes());
    cursor.extend_from_slice(&item.last_observed_at_unix_seconds.to_be_bytes());
    cursor.extend_from_slice(&item.sender_id);
    let checksum = cursor_checksum(&cursor);
    cursor.extend_from_slice(&checksum);
    cursor
}

fn decode_cursor(
    cursor: &[u8],
    account_id: Option<[u8; 16]>,
) -> Result<Option<CommunicationsSenderInsightAfterV1>, SenderInsightsErrorCodeV1> {
    if cursor.is_empty() {
        return Ok(None);
    }
    if cursor.len() != CURSOR_BYTES
        || &cursor[..4] != CURSOR_PREFIX
        || cursor[4..20] != cursor_scope(account_id)
        || cursor[CURSOR_BYTES - CURSOR_CHECKSUM_BYTES..]
            != cursor_checksum(&cursor[..CURSOR_BYTES - CURSOR_CHECKSUM_BYTES])
    {
        return Err(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest);
    }
    let message_count = u64::from_be_bytes(
        cursor[20..28]
            .try_into()
            .map_err(|_| SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)?,
    );
    let last_observed_at_unix_seconds = i64::from_be_bytes(
        cursor[28..36]
            .try_into()
            .map_err(|_| SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)?,
    );
    let sender_id: [u8; 16] = cursor[36..52]
        .try_into()
        .map_err(|_| SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)?;
    if message_count == 0 || sender_id.iter().all(|byte| *byte == 0) {
        return Err(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest);
    }
    Ok(Some(CommunicationsSenderInsightAfterV1 {
        message_count,
        last_observed_at_unix_seconds,
        sender_id,
    }))
}

fn cursor_scope(account_id: Option<[u8; 16]>) -> [u8; CURSOR_SCOPE_BYTES] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communications.sender-insights.cursor-scope.v1");
    digest.update(COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256);
    match account_id {
        Some(account_id) => {
            digest.update([1]);
            digest.update(account_id);
        }
        None => digest.update([0]),
    }
    digest.finalize()[..CURSOR_SCOPE_BYTES]
        .try_into()
        .expect("fixed SHA-256 prefix")
}

fn cursor_checksum(cursor: &[u8]) -> [u8; CURSOR_CHECKSUM_BYTES] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communications.sender-insights.cursor-checksum.v1");
    digest.update(COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256);
    digest.update(cursor);
    digest.finalize()[..CURSOR_CHECKSUM_BYTES]
        .try_into()
        .expect("fixed SHA-256 prefix")
}

fn insight_to_wire(item: &CommunicationsSenderInsightV1) -> SenderInsightV1 {
    SenderInsightV1 {
        sender_id: item.sender_id.to_vec(),
        display_label: item.display_label.clone(),
        message_count: item.message_count,
        conversation_count: item.conversation_count,
        first_observed_at_unix_seconds: item.first_observed_at_unix_seconds,
        last_observed_at_unix_seconds: item.last_observed_at_unix_seconds,
    }
}

fn error_response(error: SenderInsightsErrorCodeV1) -> ListSenderInsightsResponseV1 {
    ListSenderInsightsResponseV1 {
        items: Vec::new(),
        next_cursor: Vec::new(),
        error: error as i32,
    }
}

const fn page_limit(value: u32) -> Result<u16, SenderInsightsErrorCodeV1> {
    if value == 0 || value > 100 {
        Err(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)
    } else {
        Ok(value as u16)
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], SenderInsightsErrorCodeV1> {
    value
        .try_into()
        .ok()
        .filter(|candidate: &[u8; 16]| candidate.iter().any(|byte| *byte != 0))
        .ok_or(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)
}

const fn map_persistence_error(
    error: CommunicationsSenderInsightsErrorV1,
) -> SenderInsightsErrorCodeV1 {
    match error {
        CommunicationsSenderInsightsErrorV1::Invalid => {
            SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest
        }
        CommunicationsSenderInsightsErrorV1::AccountNotFound => {
            SenderInsightsErrorCodeV1::SenderInsightsErrorCodeNotFound
        }
        CommunicationsSenderInsightsErrorV1::StorageUnavailable => {
            SenderInsightsErrorCodeV1::SenderInsightsErrorCodeUnavailable
        }
    }
}

fn sender_insights_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: SENDER_INSIGHTS_CONTRACT_NAME_V1.to_owned(),
        major: SENDER_INSIGHTS_CONTRACT_MAJOR_V1,
        revision: SENDER_INSIGHTS_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_is_scope_and_checksum_bound() {
        let item = CommunicationsSenderInsightV1 {
            sender_id: [7; 16],
            display_label: Some("Sender".to_owned()),
            message_count: 3,
            conversation_count: 2,
            first_observed_at_unix_seconds: 1,
            last_observed_at_unix_seconds: 2,
        };
        let cursor = encode_cursor(&item, Some([9; 16]));
        assert_eq!(
            decode_cursor(&cursor, Some([9; 16])),
            Ok(Some(CommunicationsSenderInsightAfterV1 {
                message_count: 3,
                last_observed_at_unix_seconds: 2,
                sender_id: [7; 16],
            }))
        );
        assert_eq!(
            decode_cursor(&cursor, None),
            Err(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)
        );
        let mut tampered = cursor;
        tampered[24] ^= 1;
        assert_eq!(
            decode_cursor(&tampered, Some([9; 16])),
            Err(SenderInsightsErrorCodeV1::SenderInsightsErrorCodeInvalidRequest)
        );
    }
}
