use crate::{
    MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_STATEMENT_CHARS_V1, ObligationTimestampV1,
    STABLE_ID_BYTES_V1,
};

pub const MAX_CONDITION_CHARS_V1: usize = 4_000;
pub const MAX_EVIDENCE_OWNER_ID_BYTES_V1: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationLifecycleStateV1 {
    Open,
    Fulfilled,
    Waived,
    Breached,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationEvidenceLinkV1 {
    pub evidence_link_id: [u8; STABLE_ID_BYTES_V1],
    pub evidence_owner_id: String,
    pub evidence_record_id: [u8; STABLE_ID_BYTES_V1],
    pub evidence_revision: u64,
    pub evidence_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationRecordV1 {
    pub obligation_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub statement: String,
    pub condition: Option<String>,
    pub due_at: Option<ObligationTimestampV1>,
    pub state: ObligationLifecycleStateV1,
    pub obligation_revision: u64,
    pub obligated_party_id: [u8; STABLE_ID_BYTES_V1],
    pub beneficiary_party_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub evidence_links: Vec<ObligationEvidenceLinkV1>,
    pub created_at: ObligationTimestampV1,
    pub updated_at: ObligationTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationLifecycleErrorV1 {
    InvalidOwner,
    InvalidObligationId,
    InvalidStatement,
    InvalidCondition,
    InvalidTimestamp,
    InvalidParty,
    InvalidEvidence,
    EvidenceExists,
    EvidenceNotFound,
    RevisionConflict,
    RevisionOverflow,
    InvalidStateTransition,
}

#[allow(clippy::too_many_arguments)]
pub fn update_obligation_content_v1(
    obligation: &mut ObligationRecordV1,
    expected_revision: u64,
    statement: Option<String>,
    condition: Option<Option<String>>,
    due_at: Option<Option<ObligationTimestampV1>>,
    obligated_party_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    beneficiary_party_id: Option<Option<[u8; STABLE_ID_BYTES_V1]>>,
    changed_at: ObligationTimestampV1,
) -> Result<(), ObligationLifecycleErrorV1> {
    ensure_open(obligation)?;
    let next_statement = statement.as_deref().unwrap_or(&obligation.statement);
    let next_condition = condition
        .as_ref()
        .map_or(obligation.condition.as_deref(), |value| value.as_deref());
    validate_content(next_statement, next_condition)?;
    if let Some(Some(value)) = due_at {
        validate_timestamp(value)?;
    }
    let next_obligated = obligated_party_id.unwrap_or(obligation.obligated_party_id);
    let next_beneficiary = beneficiary_party_id.unwrap_or(obligation.beneficiary_party_id);
    validate_parties(&next_obligated, next_beneficiary.as_ref())?;
    let revision = next_revision(obligation, expected_revision, changed_at)?;
    if let Some(value) = statement {
        obligation.statement = value;
    }
    if let Some(value) = condition {
        obligation.condition = value;
    }
    if let Some(value) = due_at {
        obligation.due_at = value;
    }
    obligation.obligated_party_id = next_obligated;
    obligation.beneficiary_party_id = next_beneficiary;
    apply_revision(obligation, revision, changed_at);
    Ok(())
}

pub fn set_obligation_state_v1(
    obligation: &mut ObligationRecordV1,
    expected_revision: u64,
    state: ObligationLifecycleStateV1,
    changed_at: ObligationTimestampV1,
) -> Result<(), ObligationLifecycleErrorV1> {
    if obligation.state == state || obligation.state != ObligationLifecycleStateV1::Open {
        return Err(ObligationLifecycleErrorV1::InvalidStateTransition);
    }
    let revision = next_revision(obligation, expected_revision, changed_at)?;
    obligation.state = state;
    apply_revision(obligation, revision, changed_at);
    Ok(())
}

pub fn add_obligation_evidence_v1(
    obligation: &mut ObligationRecordV1,
    expected_revision: u64,
    evidence: ObligationEvidenceLinkV1,
    changed_at: ObligationTimestampV1,
) -> Result<(), ObligationLifecycleErrorV1> {
    ensure_open(obligation)?;
    validate_evidence(&evidence)?;
    if obligation
        .evidence_links
        .iter()
        .any(|current| current.evidence_link_id == evidence.evidence_link_id)
    {
        return Err(ObligationLifecycleErrorV1::EvidenceExists);
    }
    let revision = next_revision(obligation, expected_revision, changed_at)?;
    obligation.evidence_links.push(evidence);
    obligation
        .evidence_links
        .sort_by_key(|value| value.evidence_link_id);
    apply_revision(obligation, revision, changed_at);
    Ok(())
}

pub fn remove_obligation_evidence_v1(
    obligation: &mut ObligationRecordV1,
    expected_revision: u64,
    evidence_link_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: ObligationTimestampV1,
) -> Result<(), ObligationLifecycleErrorV1> {
    ensure_open(obligation)?;
    let Some(index) = obligation
        .evidence_links
        .iter()
        .position(|value| value.evidence_link_id == evidence_link_id)
    else {
        return Err(ObligationLifecycleErrorV1::EvidenceNotFound);
    };
    let revision = next_revision(obligation, expected_revision, changed_at)?;
    obligation.evidence_links.remove(index);
    apply_revision(obligation, revision, changed_at);
    Ok(())
}

pub fn validate_obligation_record_v1(
    obligation: &ObligationRecordV1,
) -> Result<(), ObligationLifecycleErrorV1> {
    if !valid_owner(&obligation.logical_owner_id) {
        return Err(ObligationLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(&obligation.obligation_id) || obligation.obligation_revision == 0 {
        return Err(ObligationLifecycleErrorV1::InvalidObligationId);
    }
    validate_content(&obligation.statement, obligation.condition.as_deref())?;
    if let Some(due_at) = obligation.due_at {
        validate_timestamp(due_at)?;
    }
    validate_parties(
        &obligation.obligated_party_id,
        obligation.beneficiary_party_id.as_ref(),
    )?;
    validate_timestamp(obligation.created_at)?;
    validate_timestamp(obligation.updated_at)?;
    if timestamp_key(obligation.updated_at) < timestamp_key(obligation.created_at) {
        return Err(ObligationLifecycleErrorV1::InvalidTimestamp);
    }
    if obligation
        .evidence_links
        .iter()
        .any(|value| validate_evidence(value).is_err())
        || obligation
            .evidence_links
            .windows(2)
            .any(|pair| pair[0].evidence_link_id >= pair[1].evidence_link_id)
    {
        return Err(ObligationLifecycleErrorV1::InvalidEvidence);
    }
    Ok(())
}

fn next_revision(
    obligation: &ObligationRecordV1,
    expected_revision: u64,
    changed_at: ObligationTimestampV1,
) -> Result<u64, ObligationLifecycleErrorV1> {
    if expected_revision == 0 || obligation.obligation_revision != expected_revision {
        return Err(ObligationLifecycleErrorV1::RevisionConflict);
    }
    validate_timestamp(changed_at)?;
    if timestamp_key(changed_at) < timestamp_key(obligation.updated_at) {
        return Err(ObligationLifecycleErrorV1::InvalidTimestamp);
    }
    obligation
        .obligation_revision
        .checked_add(1)
        .ok_or(ObligationLifecycleErrorV1::RevisionOverflow)
}

fn apply_revision(
    obligation: &mut ObligationRecordV1,
    revision: u64,
    changed_at: ObligationTimestampV1,
) {
    obligation.obligation_revision = revision;
    obligation.updated_at = changed_at;
}

fn ensure_open(obligation: &ObligationRecordV1) -> Result<(), ObligationLifecycleErrorV1> {
    if obligation.state == ObligationLifecycleStateV1::Open {
        Ok(())
    } else {
        Err(ObligationLifecycleErrorV1::InvalidStateTransition)
    }
}

fn validate_content(
    statement: &str,
    condition: Option<&str>,
) -> Result<(), ObligationLifecycleErrorV1> {
    if !valid_text(statement, MAX_STATEMENT_CHARS_V1) {
        return Err(ObligationLifecycleErrorV1::InvalidStatement);
    }
    if condition.is_some_and(|value| !valid_text(value, MAX_CONDITION_CHARS_V1)) {
        return Err(ObligationLifecycleErrorV1::InvalidCondition);
    }
    Ok(())
}

fn validate_parties(
    obligated_party_id: &[u8; STABLE_ID_BYTES_V1],
    beneficiary_party_id: Option<&[u8; STABLE_ID_BYTES_V1]>,
) -> Result<(), ObligationLifecycleErrorV1> {
    if !nonzero(obligated_party_id) || beneficiary_party_id.is_some_and(|value| !nonzero(value)) {
        return Err(ObligationLifecycleErrorV1::InvalidParty);
    }
    Ok(())
}

fn validate_evidence(value: &ObligationEvidenceLinkV1) -> Result<(), ObligationLifecycleErrorV1> {
    if !nonzero(&value.evidence_link_id)
        || !valid_owner(&value.evidence_owner_id)
        || !nonzero(&value.evidence_record_id)
        || value.evidence_revision == 0
        || !nonzero(&value.evidence_digest)
    {
        return Err(ObligationLifecycleErrorV1::InvalidEvidence);
    }
    Ok(())
}

fn validate_timestamp(value: ObligationTimestampV1) -> Result<(), ObligationLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(ObligationLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn timestamp_key(value: ObligationTimestampV1) -> (i64, i32) {
    (value.unix_seconds, value.nanos)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LOGICAL_OWNER_ID_BYTES_V1.min(MAX_EVIDENCE_OWNER_ID_BYTES_V1)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(seconds: i64) -> ObligationTimestampV1 {
        ObligationTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }

    fn obligation() -> ObligationRecordV1 {
        ObligationRecordV1 {
            obligation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            statement: "Confirm the reviewed obligation".to_owned(),
            condition: None,
            due_at: None,
            state: ObligationLifecycleStateV1::Open,
            obligation_revision: 1,
            obligated_party_id: [2; 16],
            beneficiary_party_id: Some([3; 16]),
            evidence_links: Vec::new(),
            created_at: timestamp(10),
            updated_at: timestamp(10),
        }
    }

    #[test]
    fn evidence_is_revisioned_and_terminal_records_do_not_reopen() {
        let mut value = obligation();
        add_obligation_evidence_v1(
            &mut value,
            1,
            ObligationEvidenceLinkV1 {
                evidence_link_id: [4; 16],
                evidence_owner_id: "communications".to_owned(),
                evidence_record_id: [5; 16],
                evidence_revision: 2,
                evidence_digest: [6; 32],
            },
            timestamp(11),
        )
        .expect("evidence");
        assert_eq!(value.obligation_revision, 2);
        set_obligation_state_v1(
            &mut value,
            2,
            ObligationLifecycleStateV1::Fulfilled,
            timestamp(12),
        )
        .expect("terminal");
        assert_eq!(
            set_obligation_state_v1(
                &mut value,
                3,
                ObligationLifecycleStateV1::Open,
                timestamp(13),
            ),
            Err(ObligationLifecycleErrorV1::InvalidStateTransition)
        );
    }
}
