use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_CONDITION_CHARS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_STATEMENT_CHARS_V1,
    ObligationEvidenceLinkV1, STABLE_ID_BYTES_V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationStatusV1 {
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObligationTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationProvenanceV1 {
    pub approved_candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub review_id: [u8; STABLE_ID_BYTES_V1],
    pub decision_revision: u64,
    pub decided_by_owner_device_id: [u8; STABLE_ID_BYTES_V1],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedCandidateObligationDraftV1 {
    pub logical_owner_id: String,
    pub provenance: ObligationProvenanceV1,
    pub statement: String,
    pub condition: Option<String>,
    pub due_at: Option<ObligationTimestampV1>,
    pub obligated_party_id: [u8; STABLE_ID_BYTES_V1],
    pub beneficiary_party_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub evidence_links: Vec<ObligationEvidenceLinkV1>,
    pub created_at: ObligationTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationV1 {
    pub obligation_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub statement: String,
    pub condition: Option<String>,
    pub due_at: Option<ObligationTimestampV1>,
    pub obligated_party_id: [u8; STABLE_ID_BYTES_V1],
    pub beneficiary_party_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub evidence_links: Vec<ObligationEvidenceLinkV1>,
    pub status: ObligationStatusV1,
    pub obligation_revision: u64,
    pub provenance: ObligationProvenanceV1,
    pub created_at: ObligationTimestampV1,
    pub updated_at: ObligationTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationsValidationErrorV1 {
    InvalidOwner,
    InvalidObligationId,
    InvalidCandidateId,
    InvalidCandidateDigest,
    InvalidSourceEvidence,
    InvalidSourceRevision,
    InvalidReviewId,
    InvalidDecisionRevision,
    InvalidDecisionActor,
    InvalidStatement,
    InvalidCondition,
    InvalidTimestamp,
    InvalidParty,
    InvalidEvidence,
    InvalidRevision,
}

pub fn derive_obligation_id_v1(
    logical_owner_id: &str,
    approved_candidate_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], ObligationsValidationErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(ObligationsValidationErrorV1::InvalidOwner);
    }
    if !nonzero(approved_candidate_id) {
        return Err(ObligationsValidationErrorV1::InvalidCandidateId);
    }
    Ok(digest(
        b"makosh.obligations.reviewed-candidate.obligation-id.v1",
        logical_owner_id.as_bytes(),
        approved_candidate_id,
    ))
}

pub fn obligation_creation_fingerprint_v1(
    draft: &ReviewedCandidateObligationDraftV1,
) -> Result<[u8; DIGEST_BYTES_V1], ObligationsValidationErrorV1> {
    validate_draft(draft)?;
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.obligations.reviewed-candidate.creation.v2");
    update_part(&mut hasher, draft.logical_owner_id.as_bytes());
    update_part(&mut hasher, &draft.provenance.approved_candidate_id);
    update_part(&mut hasher, &draft.provenance.candidate_digest);
    update_part(&mut hasher, &draft.provenance.source_evidence_id);
    update_part(
        &mut hasher,
        &draft.provenance.source_evidence_revision.to_be_bytes(),
    );
    update_part(&mut hasher, &draft.provenance.review_id);
    update_part(
        &mut hasher,
        &draft.provenance.decision_revision.to_be_bytes(),
    );
    update_part(&mut hasher, &draft.provenance.decided_by_owner_device_id);
    update_part(&mut hasher, draft.statement.as_bytes());
    update_optional_text(&mut hasher, draft.condition.as_deref());
    update_optional_timestamp(&mut hasher, draft.due_at);
    update_part(&mut hasher, &draft.obligated_party_id);
    update_optional_id(&mut hasher, draft.beneficiary_party_id.as_ref());
    for evidence in &draft.evidence_links {
        update_part(&mut hasher, &evidence.evidence_link_id);
        update_part(&mut hasher, evidence.evidence_owner_id.as_bytes());
        update_part(&mut hasher, &evidence.evidence_record_id);
        update_part(&mut hasher, &evidence.evidence_revision.to_be_bytes());
        update_part(&mut hasher, &evidence.evidence_digest);
    }
    Ok(hasher.finalize().into())
}

pub fn validate_obligation_v1(
    obligation: &ObligationV1,
) -> Result<(), ObligationsValidationErrorV1> {
    let expected = derive_obligation_id_v1(
        &obligation.logical_owner_id,
        &obligation.provenance.approved_candidate_id,
    )?;
    if obligation.obligation_id != expected || !nonzero(&obligation.obligation_id) {
        return Err(ObligationsValidationErrorV1::InvalidObligationId);
    }
    validate_provenance(&obligation.provenance)?;
    validate_content(
        &obligation.statement,
        obligation.condition.as_deref(),
        obligation.due_at,
        &obligation.obligated_party_id,
        obligation.beneficiary_party_id.as_ref(),
        &obligation.evidence_links,
    )?;
    if obligation.obligation_revision == 0 {
        return Err(ObligationsValidationErrorV1::InvalidRevision);
    }
    if !valid_timestamp(obligation.created_at)
        || !valid_timestamp(obligation.updated_at)
        || (
            obligation.updated_at.unix_seconds,
            obligation.updated_at.nanos,
        ) < (
            obligation.created_at.unix_seconds,
            obligation.created_at.nanos,
        )
    {
        return Err(ObligationsValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

pub(crate) fn validate_draft(
    draft: &ReviewedCandidateObligationDraftV1,
) -> Result<(), ObligationsValidationErrorV1> {
    derive_obligation_id_v1(
        &draft.logical_owner_id,
        &draft.provenance.approved_candidate_id,
    )?;
    validate_provenance(&draft.provenance)?;
    validate_content(
        &draft.statement,
        draft.condition.as_deref(),
        draft.due_at,
        &draft.obligated_party_id,
        draft.beneficiary_party_id.as_ref(),
        &draft.evidence_links,
    )?;
    if !valid_timestamp(draft.created_at) {
        return Err(ObligationsValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_provenance(value: &ObligationProvenanceV1) -> Result<(), ObligationsValidationErrorV1> {
    if !nonzero(&value.approved_candidate_id) {
        return Err(ObligationsValidationErrorV1::InvalidCandidateId);
    }
    if !nonzero(&value.candidate_digest) {
        return Err(ObligationsValidationErrorV1::InvalidCandidateDigest);
    }
    if !nonzero(&value.source_evidence_id) {
        return Err(ObligationsValidationErrorV1::InvalidSourceEvidence);
    }
    if value.source_evidence_revision == 0 {
        return Err(ObligationsValidationErrorV1::InvalidSourceRevision);
    }
    if !nonzero(&value.review_id) {
        return Err(ObligationsValidationErrorV1::InvalidReviewId);
    }
    if value.decision_revision == 0 {
        return Err(ObligationsValidationErrorV1::InvalidDecisionRevision);
    }
    if !nonzero(&value.decided_by_owner_device_id) {
        return Err(ObligationsValidationErrorV1::InvalidDecisionActor);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_content(
    statement: &str,
    condition: Option<&str>,
    due_at: Option<ObligationTimestampV1>,
    obligated_party_id: &[u8; STABLE_ID_BYTES_V1],
    beneficiary_party_id: Option<&[u8; STABLE_ID_BYTES_V1]>,
    evidence_links: &[ObligationEvidenceLinkV1],
) -> Result<(), ObligationsValidationErrorV1> {
    if !valid_text(statement, MAX_STATEMENT_CHARS_V1) {
        return Err(ObligationsValidationErrorV1::InvalidStatement);
    }
    if condition.is_some_and(|value| !valid_text(value, MAX_CONDITION_CHARS_V1)) {
        return Err(ObligationsValidationErrorV1::InvalidCondition);
    }
    if due_at.is_some_and(|value| !valid_timestamp(value)) {
        return Err(ObligationsValidationErrorV1::InvalidTimestamp);
    }
    if !nonzero(obligated_party_id) || beneficiary_party_id.is_some_and(|value| !nonzero(value)) {
        return Err(ObligationsValidationErrorV1::InvalidParty);
    }
    if evidence_links.iter().any(|value| {
        !nonzero(&value.evidence_link_id)
            || !valid_owner(&value.evidence_owner_id)
            || !nonzero(&value.evidence_record_id)
            || value.evidence_revision == 0
            || !nonzero(&value.evidence_digest)
    }) || evidence_links
        .windows(2)
        .any(|pair| pair[0].evidence_link_id >= pair[1].evidence_link_id)
    {
        return Err(ObligationsValidationErrorV1::InvalidEvidence);
    }
    Ok(())
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LOGICAL_OWNER_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_timestamp(value: ObligationTimestampV1) -> bool {
    value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos)
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn digest(label: &[u8], first: &[u8], second: &[u8]) -> [u8; STABLE_ID_BYTES_V1] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    update_part(&mut hasher, first);
    update_part(&mut hasher, second);
    hasher.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .expect("fixed digest")
}

fn update_part(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn update_optional_text(hasher: &mut Sha256, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            update_part(hasher, value.as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn update_optional_timestamp(hasher: &mut Sha256, value: Option<ObligationTimestampV1>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hasher.update(value.unix_seconds.to_be_bytes());
            hasher.update(value.nanos.to_be_bytes());
        }
        None => hasher.update([0]),
    }
}

fn update_optional_id(hasher: &mut Sha256, value: Option<&[u8; STABLE_ID_BYTES_V1]>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            update_part(hasher, value);
        }
        None => hasher.update([0]),
    }
}
