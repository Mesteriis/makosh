use makosh_events_protocol::v1::{ContractRefV1, DurableEnvelopeV1};
use makosh_runtime_protocol::v1::ContractReferenceV1;

use crate::task_results::ReviewedTaskCandidatePromotionEventErrorV1;

pub(crate) fn validate_contract(
    envelope: &DurableEnvelopeV1,
    expected: &ContractReferenceV1,
) -> Result<(), ReviewedTaskCandidatePromotionEventErrorV1> {
    if envelope
        .contract
        .as_ref()
        .is_none_or(|actual| !same_contract(actual, expected))
    {
        return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn same_contract(actual: &ContractRefV1, expected: &ContractReferenceV1) -> bool {
    actual.owner == expected.owner
        && actual.name == expected.name
        && actual.major == expected.major
        && actual.revision == expected.revision
        && actual.schema_sha256 == expected.schema_sha256
}

pub(crate) fn id16(value: &[u8]) -> Result<[u8; 16], ReviewedTaskCandidatePromotionEventErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)
}

pub(crate) fn id32(value: &[u8]) -> Result<[u8; 32], ReviewedTaskCandidatePromotionEventErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}
