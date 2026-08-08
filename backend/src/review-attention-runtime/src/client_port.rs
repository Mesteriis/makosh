use makosh_review_attention_api::wire::{
    ClearSnoozeV1, ReviewAttentionChangedV1, ReviewAttentionCommandRequestV1,
    ReviewAttentionCommandResponseV1, ReviewAttentionPageV1 as WirePage,
    ReviewAttentionQueryRequestV1, ReviewAttentionQueryResponseV1,
    ReviewAttentionSummaryV1 as WireSummary, ReviewDispositionV1 as WireDisposition,
    ReviewImportanceV1 as WireImportance,
    review_attention_command_request_v1::Operation as WireCommand,
    review_attention_query_request_v1::Operation as WireQuery,
    review_attention_query_response_v1::Result as WireQueryResult,
};
use makosh_review_attention_core::{
    ReviewAttentionCommandV1, ReviewAttentionV1, ReviewDispositionV1, ReviewImportanceV1,
    ReviewTimestampV1,
};
use makosh_review_attention_persistence::{
    ApplyReviewAttentionOperationV1, ReviewAttentionListFilterV1,
    ReviewAttentionPersistenceErrorV1, ReviewAttentionPersistenceV1,
    ReviewAttentionRealtimeTransitionV1,
};
use prost::Message;

pub async fn command_payload_v1(
    persistence: &ReviewAttentionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = ReviewAttentionCommandRequestV1::decode(payload) else {
        return command_error("invalid_request");
    };
    if request.protocol_major != 1 {
        return command_error("incompatible_protocol");
    }
    let Some(operation_id) = id16(&request.operation_id) else {
        return command_error("invalid_request");
    };
    let Some(source_evidence_id) = id16(&request.source_evidence_id) else {
        return command_error("invalid_request");
    };
    let Some(command) = request.operation.and_then(command) else {
        return command_error("invalid_request");
    };
    let applied_at = authoritative_timestamp();
    match persistence
        .apply_operation(ApplyReviewAttentionOperationV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            operation_id,
            source_evidence_id,
            expected_revision: request.expected_revision,
            command,
            applied_at,
        })
        .await
    {
        Ok(outcome) => ReviewAttentionCommandResponseV1 {
            attention: Some(summary(&outcome.attention)),
            replayed: outcome.replayed,
            error_code: String::new(),
        }
        .encode_to_vec(),
        Err(error) => command_error(persistence_error(error)),
    }
}

pub async fn query_payload_v1(
    persistence: &ReviewAttentionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = ReviewAttentionQueryRequestV1::decode(payload) else {
        return query_error("invalid_request");
    };
    if request.protocol_major != 1 {
        return query_error("incompatible_protocol");
    }
    match request.operation {
        Some(WireQuery::Get(get)) => {
            let Some(attention_id) = id16(&get.attention_id) else {
                return query_error("invalid_request");
            };
            match persistence
                .get_attention(logical_owner_id, &attention_id)
                .await
            {
                Ok(Some(attention)) => {
                    query_response(WireQueryResult::Attention(summary(&attention)))
                }
                Ok(None) => query_error("not_found"),
                Err(error) => query_error(persistence_error(error)),
            }
        }
        Some(WireQuery::List(list)) => {
            let cursor = if list.cursor.is_empty() {
                None
            } else {
                match id16(&list.cursor) {
                    Some(value) => Some(value),
                    None => return query_error("invalid_cursor"),
                }
            };
            let disposition = list.disposition.and_then(wire_disposition);
            let importance = list.importance.and_then(wire_importance);
            if (list.disposition.is_some() && disposition.is_none())
                || (list.importance.is_some() && importance.is_none())
            {
                return query_error("invalid_filter");
            }
            let limit = u16::try_from(list.limit).unwrap_or(0);
            match persistence
                .list_attention(
                    logical_owner_id,
                    ReviewAttentionListFilterV1 {
                        after_attention_id: cursor,
                        disposition,
                        pinned: list.pinned,
                        importance,
                        snoozed: list.snoozed,
                        limit,
                    },
                )
                .await
            {
                Ok(page) => query_response(WireQueryResult::Page(WirePage {
                    attention: page.attention.iter().map(summary).collect(),
                    next_cursor: page
                        .next_cursor
                        .map_or_else(Vec::new, |value| value.to_vec()),
                })),
                Err(error) => query_error(persistence_error(error)),
            }
        }
        None => query_error("invalid_request"),
    }
}

pub fn realtime_transition_payload_v1(transition: &ReviewAttentionRealtimeTransitionV1) -> Vec<u8> {
    ReviewAttentionChangedV1 {
        attention_id: transition.attention_id.to_vec(),
        revision: transition.revision,
        disposition: wire_disposition_code(transition.disposition),
        pinned: transition.pinned,
        importance: wire_importance_code(transition.importance),
        snoozed_until_unix_seconds: transition.snoozed_until.map(|value| value.unix_seconds),
        snoozed_until_nanos: transition.snoozed_until.map(|value| value.nanos),
    }
    .encode_to_vec()
}

fn command(value: WireCommand) -> Option<ReviewAttentionCommandV1> {
    match value {
        WireCommand::MarkPending(_) => Some(ReviewAttentionCommandV1::MarkPending),
        WireCommand::MarkReviewed(_) => Some(ReviewAttentionCommandV1::MarkReviewed),
        WireCommand::Dismiss(_) => Some(ReviewAttentionCommandV1::Dismiss),
        WireCommand::SetPinned(value) => Some(ReviewAttentionCommandV1::SetPinned(value.pinned)),
        WireCommand::SetImportance(value) => {
            wire_importance(value.importance).map(ReviewAttentionCommandV1::SetImportance)
        }
        WireCommand::Snooze(value) => {
            Some(ReviewAttentionCommandV1::SnoozeUntil(ReviewTimestampV1 {
                unix_seconds: value.until_unix_seconds,
                nanos: value.until_nanos,
            }))
        }
        WireCommand::ClearSnooze(ClearSnoozeV1 {}) => Some(ReviewAttentionCommandV1::ClearSnooze),
    }
}

fn summary(attention: &ReviewAttentionV1) -> WireSummary {
    WireSummary {
        attention_id: attention.attention_id.to_vec(),
        source_evidence_id: attention.source_evidence_id.to_vec(),
        revision: attention.revision,
        disposition: wire_disposition_code(attention.disposition),
        pinned: attention.pinned,
        importance: wire_importance_code(attention.importance),
        snoozed_until_unix_seconds: attention.snoozed_until.map(|value| value.unix_seconds),
        snoozed_until_nanos: attention.snoozed_until.map(|value| value.nanos),
        updated_at_unix_seconds: attention.updated_at.unix_seconds,
        updated_at_nanos: attention.updated_at.nanos,
    }
}

fn query_response(result: WireQueryResult) -> Vec<u8> {
    ReviewAttentionQueryResponseV1 {
        result: Some(result),
        error_code: String::new(),
    }
    .encode_to_vec()
}

fn command_error(code: &str) -> Vec<u8> {
    ReviewAttentionCommandResponseV1 {
        attention: None,
        replayed: false,
        error_code: code.to_owned(),
    }
    .encode_to_vec()
}

fn query_error(code: &str) -> Vec<u8> {
    ReviewAttentionQueryResponseV1 {
        result: None,
        error_code: code.to_owned(),
    }
    .encode_to_vec()
}

fn persistence_error(error: ReviewAttentionPersistenceErrorV1) -> &'static str {
    match error {
        ReviewAttentionPersistenceErrorV1::InvalidInput => "invalid_request",
        ReviewAttentionPersistenceErrorV1::InvalidRow
        | ReviewAttentionPersistenceErrorV1::StorageUnavailable => "unavailable",
        ReviewAttentionPersistenceErrorV1::OperationConflict => "operation_conflict",
        ReviewAttentionPersistenceErrorV1::Domain(
            makosh_review_attention_core::ReviewAttentionErrorV1::RevisionConflict,
        ) => "stale_revision",
        ReviewAttentionPersistenceErrorV1::Domain(
            makosh_review_attention_core::ReviewAttentionErrorV1::DismissedAttention,
        ) => "dismissed",
        ReviewAttentionPersistenceErrorV1::Domain(_) => "invalid_request",
    }
}

fn wire_disposition(value: i32) -> Option<ReviewDispositionV1> {
    match WireDisposition::try_from(value).ok() {
        Some(WireDisposition::ReviewDispositionPending) => Some(ReviewDispositionV1::Pending),
        Some(WireDisposition::ReviewDispositionReviewed) => Some(ReviewDispositionV1::Reviewed),
        Some(WireDisposition::ReviewDispositionDismissed) => Some(ReviewDispositionV1::Dismissed),
        _ => None,
    }
}

fn wire_importance(value: i32) -> Option<ReviewImportanceV1> {
    match WireImportance::try_from(value).ok() {
        Some(WireImportance::ReviewImportanceNormal) => Some(ReviewImportanceV1::Normal),
        Some(WireImportance::ReviewImportanceImportant) => Some(ReviewImportanceV1::Important),
        _ => None,
    }
}

const fn wire_disposition_code(value: ReviewDispositionV1) -> i32 {
    match value {
        ReviewDispositionV1::Pending => WireDisposition::ReviewDispositionPending as i32,
        ReviewDispositionV1::Reviewed => WireDisposition::ReviewDispositionReviewed as i32,
        ReviewDispositionV1::Dismissed => WireDisposition::ReviewDispositionDismissed as i32,
    }
}

const fn wire_importance_code(value: ReviewImportanceV1) -> i32 {
    match value {
        ReviewImportanceV1::Normal => WireImportance::ReviewImportanceNormal as i32,
        ReviewImportanceV1::Important => WireImportance::ReviewImportanceImportant as i32,
    }
}

fn id16(value: &[u8]) -> Option<[u8; 16]> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
}

fn authoritative_timestamp() -> ReviewTimestampV1 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    ReviewTimestampV1 {
        unix_seconds: i64::try_from(now.as_secs()).unwrap_or(i64::MAX),
        nanos: i32::try_from(now.subsec_nanos()).expect("nanos fit i32"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realtime_payload_exposes_only_review_state() {
        let payload = realtime_transition_payload_v1(&ReviewAttentionRealtimeTransitionV1 {
            sequence: 1,
            attention_id: [1; 16],
            revision: 3,
            disposition: ReviewDispositionV1::Pending,
            pinned: true,
            importance: ReviewImportanceV1::Important,
            snoozed_until: None,
            occurred_at: ReviewTimestampV1 {
                unix_seconds: 1,
                nanos: 0,
            },
        });
        let decoded = ReviewAttentionChangedV1::decode(payload.as_slice()).expect("decode");
        assert_eq!(decoded.attention_id, vec![1; 16]);
        assert_eq!(decoded.revision, 3);
    }
}
