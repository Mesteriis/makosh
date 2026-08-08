//! Generated client query adapter for canonical Communications call evidence.

use makosh_communications_call_evidence_api::{
    CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1, CALL_EVIDENCE_QUERY_MAX_PAGE_SIZE_V1,
    wire::{
        CallEvidenceQueryRequestV1, CallEvidenceQueryResponseV1, CallEvidenceSummaryV1,
        GetCallEvidenceResponseV1, ListCallEvidenceResponseV1,
        call_evidence_query_request_v1::Operation,
        call_evidence_query_response_v1::Result as QueryResult,
    },
};
use makosh_communications_call_evidence_core::{
    CallDirectionV1, CallEvidenceProjectionV1, CallLifecycleStateV1, CallMediaKindV1,
    CallProviderProvenanceV1, CallTerminalDispositionV1,
};
use makosh_communications_call_evidence_persistence::{
    CallEvidenceListFilterV1, CallEvidencePersistenceErrorV1,
    CommunicationsCallEvidencePersistenceV1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceQueryPortErrorV1 {
    Protocol,
    Unavailable,
}

pub async fn handle_call_evidence_query_v1(
    persistence: &CommunicationsCallEvidencePersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, CallEvidenceQueryPortErrorV1> {
    let request = CallEvidenceQueryRequestV1::decode(bytes)
        .map_err(|_| CallEvidenceQueryPortErrorV1::Protocol)?;
    if request.protocol_major != CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1 {
        return Err(CallEvidenceQueryPortErrorV1::Protocol);
    }
    let result = match request
        .operation
        .ok_or(CallEvidenceQueryPortErrorV1::Protocol)?
    {
        Operation::Get(request) => {
            let call_evidence_id = id16(&request.call_evidence_id)?;
            let Some(evidence) = persistence
                .get(logical_owner_id, call_evidence_id)
                .await
                .map_err(map_persistence_error)?
            else {
                return Ok(error_response("NOT_FOUND"));
            };
            QueryResult::Get(GetCallEvidenceResponseV1 {
                evidence: Some(summary(&evidence)),
            })
        }
        Operation::List(request) => {
            let page = persistence
                .list(
                    logical_owner_id,
                    CallEvidenceListFilterV1 {
                        provider: request.provider.map(provider_filter).transpose()?,
                        direction: request.direction.map(direction_filter).transpose()?,
                        media_kind: request.media_kind.map(media_kind_filter).transpose()?,
                        state: request.state.map(state_filter).transpose()?,
                    },
                    page_limit(request.limit)?,
                    &request.cursor,
                )
                .await
                .map_err(map_persistence_error)?;
            QueryResult::List(ListCallEvidenceResponseV1 {
                evidence: page.items.iter().map(summary).collect(),
                next_cursor: page.next_cursor,
            })
        }
    };
    Ok(CallEvidenceQueryResponseV1 {
        result: Some(result),
        error_code: String::new(),
    }
    .encode_to_vec())
}

fn error_response(error_code: &str) -> Vec<u8> {
    CallEvidenceQueryResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
    .encode_to_vec()
}

fn summary(projection: &CallEvidenceProjectionV1) -> CallEvidenceSummaryV1 {
    let evidence = &projection.evidence;
    CallEvidenceSummaryV1 {
        call_evidence_id: evidence.call_evidence_id.to_vec(),
        canonical_revision: projection.canonical_revision,
        provider: provider_value(evidence.provider),
        direction: direction_value(evidence.direction),
        media_kind: media_kind_value(evidence.media_kind),
        state: state_value(evidence.state),
        terminal_disposition: evidence
            .terminal_disposition
            .map_or(0, terminal_disposition_value),
        started_at_unix_seconds: evidence.started_at_unix_seconds,
        connected_at_unix_seconds: evidence.connected_at_unix_seconds,
        ended_at_unix_seconds: evidence.ended_at_unix_seconds,
        duration_seconds: evidence.duration_seconds,
        participant_display_label: evidence.participant_display_label.clone(),
    }
}

fn page_limit(value: u32) -> Result<u16, CallEvidenceQueryPortErrorV1> {
    let value = u16::try_from(value).map_err(|_| CallEvidenceQueryPortErrorV1::Protocol)?;
    (value > 0 && value <= CALL_EVIDENCE_QUERY_MAX_PAGE_SIZE_V1)
        .then_some(value)
        .ok_or(CallEvidenceQueryPortErrorV1::Protocol)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallEvidenceQueryPortErrorV1> {
    let id: [u8; 16] = value
        .try_into()
        .map_err(|_| CallEvidenceQueryPortErrorV1::Protocol)?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or(CallEvidenceQueryPortErrorV1::Protocol)
}

fn provider_filter(value: i32) -> Result<CallProviderProvenanceV1, CallEvidenceQueryPortErrorV1> {
    match value {
        1 => Ok(CallProviderProvenanceV1::Telegram),
        2 => Ok(CallProviderProvenanceV1::WhatsAppWeb),
        3 => Ok(CallProviderProvenanceV1::Zoom),
        4 => Ok(CallProviderProvenanceV1::YandexTelemost),
        _ => Err(CallEvidenceQueryPortErrorV1::Protocol),
    }
}

fn direction_filter(value: i32) -> Result<CallDirectionV1, CallEvidenceQueryPortErrorV1> {
    match value {
        1 => Ok(CallDirectionV1::Incoming),
        2 => Ok(CallDirectionV1::Outgoing),
        3 => Ok(CallDirectionV1::Unknown),
        _ => Err(CallEvidenceQueryPortErrorV1::Protocol),
    }
}

fn media_kind_filter(value: i32) -> Result<CallMediaKindV1, CallEvidenceQueryPortErrorV1> {
    match value {
        1 => Ok(CallMediaKindV1::OneToOneAudio),
        2 => Ok(CallMediaKindV1::Meeting),
        _ => Err(CallEvidenceQueryPortErrorV1::Protocol),
    }
}

fn state_filter(value: i32) -> Result<CallLifecycleStateV1, CallEvidenceQueryPortErrorV1> {
    match value {
        1 => Ok(CallLifecycleStateV1::Observed),
        2 => Ok(CallLifecycleStateV1::Ringing),
        3 => Ok(CallLifecycleStateV1::Connecting),
        4 => Ok(CallLifecycleStateV1::Active),
        5 => Ok(CallLifecycleStateV1::Ended),
        _ => Err(CallEvidenceQueryPortErrorV1::Protocol),
    }
}

pub(crate) const fn provider_value(value: CallProviderProvenanceV1) -> i32 {
    match value {
        CallProviderProvenanceV1::Telegram => 1,
        CallProviderProvenanceV1::WhatsAppWeb => 2,
        CallProviderProvenanceV1::Zoom => 3,
        CallProviderProvenanceV1::YandexTelemost => 4,
    }
}

pub(crate) const fn direction_value(value: CallDirectionV1) -> i32 {
    match value {
        CallDirectionV1::Incoming => 1,
        CallDirectionV1::Outgoing => 2,
        CallDirectionV1::Unknown => 3,
    }
}

pub(crate) const fn media_kind_value(value: CallMediaKindV1) -> i32 {
    match value {
        CallMediaKindV1::OneToOneAudio => 1,
        CallMediaKindV1::Meeting => 2,
    }
}

pub(crate) const fn state_value(value: CallLifecycleStateV1) -> i32 {
    match value {
        CallLifecycleStateV1::Observed => 1,
        CallLifecycleStateV1::Ringing => 2,
        CallLifecycleStateV1::Connecting => 3,
        CallLifecycleStateV1::Active => 4,
        CallLifecycleStateV1::Ended => 5,
    }
}

pub(crate) const fn terminal_disposition_value(value: CallTerminalDispositionV1) -> i32 {
    match value {
        CallTerminalDispositionV1::Completed => 1,
        CallTerminalDispositionV1::Missed => 2,
        CallTerminalDispositionV1::Declined => 3,
        CallTerminalDispositionV1::Canceled => 4,
        CallTerminalDispositionV1::Failed => 5,
        CallTerminalDispositionV1::Disconnected => 6,
    }
}

const fn map_persistence_error(
    error: CallEvidencePersistenceErrorV1,
) -> CallEvidenceQueryPortErrorV1 {
    match error {
        CallEvidencePersistenceErrorV1::InvalidInput
        | CallEvidencePersistenceErrorV1::InvalidRow
        | CallEvidencePersistenceErrorV1::InboxHashConflict => {
            CallEvidenceQueryPortErrorV1::Protocol
        }
        CallEvidencePersistenceErrorV1::StorageUnavailable => {
            CallEvidenceQueryPortErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_and_page_bounds_are_closed() {
        assert_eq!(page_limit(1), Ok(1));
        assert_eq!(page_limit(100), Ok(100));
        assert_eq!(page_limit(0), Err(CallEvidenceQueryPortErrorV1::Protocol));
        assert_eq!(page_limit(101), Err(CallEvidenceQueryPortErrorV1::Protocol));
        assert_eq!(
            provider_filter(0),
            Err(CallEvidenceQueryPortErrorV1::Protocol)
        );
        assert_eq!(state_filter(9), Err(CallEvidenceQueryPortErrorV1::Protocol));
    }
}
