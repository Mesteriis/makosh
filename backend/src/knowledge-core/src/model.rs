use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_EXCERPT_CHARS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1,
    MAX_TOPIC_HINTS_V1, STABLE_ID_BYTES_V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifiedKnowledgeNoteStatusV1 {
    Verified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeNoteSourceBasisV1 {
    Subject,
    Body,
    Combined,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum KnowledgeNoteTopicHintV1 {
    Financial,
    Legal,
    DecisionStatement,
    DeadlineStatement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KnowledgeNoteTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeNoteProvenanceV1 {
    pub approved_candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub review_id: [u8; STABLE_ID_BYTES_V1],
    pub decision_revision: u64,
    pub decided_by_owner_device_id: [u8; STABLE_ID_BYTES_V1],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedCandidateKnowledgeNoteDraftV1 {
    pub logical_owner_id: String,
    pub provenance: KnowledgeNoteProvenanceV1,
    pub title: String,
    pub excerpt: String,
    pub topic_hints: Vec<KnowledgeNoteTopicHintV1>,
    pub source_basis: KnowledgeNoteSourceBasisV1,
    pub confidence_basis_points: u32,
    pub created_at: KnowledgeNoteTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedKnowledgeNoteV1 {
    pub note_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub excerpt: String,
    pub topic_hints: Vec<KnowledgeNoteTopicHintV1>,
    pub source_basis: KnowledgeNoteSourceBasisV1,
    pub confidence_basis_points: u32,
    pub status: VerifiedKnowledgeNoteStatusV1,
    pub note_revision: u64,
    pub provenance: KnowledgeNoteProvenanceV1,
    pub created_at: KnowledgeNoteTimestampV1,
    pub updated_at: KnowledgeNoteTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeValidationErrorV1 {
    InvalidOwner,
    InvalidNoteId,
    InvalidCandidateId,
    InvalidCandidateDigest,
    InvalidSourceEvidence,
    InvalidSourceRevision,
    InvalidReviewId,
    InvalidDecisionRevision,
    InvalidDecisionActor,
    InvalidTitle,
    InvalidExcerpt,
    InvalidTopicHints,
    InvalidConfidence,
    InvalidTimestamp,
    InvalidRevision,
}

pub fn derive_verified_knowledge_note_id_v1(
    logical_owner_id: &str,
    approved_candidate_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], KnowledgeValidationErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(KnowledgeValidationErrorV1::InvalidOwner);
    }
    if !nonzero(approved_candidate_id) {
        return Err(KnowledgeValidationErrorV1::InvalidCandidateId);
    }
    Ok(digest(
        b"makosh.knowledge.reviewed-candidate.note-id.v1",
        logical_owner_id.as_bytes(),
        approved_candidate_id,
    ))
}

pub fn knowledge_note_creation_fingerprint_v1(
    draft: &ReviewedCandidateKnowledgeNoteDraftV1,
) -> Result<[u8; DIGEST_BYTES_V1], KnowledgeValidationErrorV1> {
    validate_draft(draft)?;
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.knowledge.reviewed-candidate.creation.v1");
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
    update_part(&mut hasher, draft.title.as_bytes());
    update_part(&mut hasher, draft.excerpt.as_bytes());
    for hint in &draft.topic_hints {
        hasher.update([topic_hint_byte(*hint)]);
    }
    hasher.update([source_basis_byte(draft.source_basis)]);
    hasher.update(draft.confidence_basis_points.to_be_bytes());
    Ok(hasher.finalize().into())
}

pub fn validate_verified_knowledge_note_v1(
    note: &VerifiedKnowledgeNoteV1,
) -> Result<(), KnowledgeValidationErrorV1> {
    let expected = derive_verified_knowledge_note_id_v1(
        &note.logical_owner_id,
        &note.provenance.approved_candidate_id,
    )?;
    if note.note_id != expected || !nonzero(&note.note_id) {
        return Err(KnowledgeValidationErrorV1::InvalidNoteId);
    }
    validate_provenance(&note.provenance)?;
    validate_content(
        &note.title,
        &note.excerpt,
        &note.topic_hints,
        note.confidence_basis_points,
    )?;
    if note.note_revision != 1 || note.status != VerifiedKnowledgeNoteStatusV1::Verified {
        return Err(KnowledgeValidationErrorV1::InvalidRevision);
    }
    if !valid_timestamp(note.created_at)
        || !valid_timestamp(note.updated_at)
        || note.updated_at.unix_seconds < note.created_at.unix_seconds
    {
        return Err(KnowledgeValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

pub(crate) fn validate_draft(
    draft: &ReviewedCandidateKnowledgeNoteDraftV1,
) -> Result<(), KnowledgeValidationErrorV1> {
    derive_verified_knowledge_note_id_v1(
        &draft.logical_owner_id,
        &draft.provenance.approved_candidate_id,
    )?;
    validate_provenance(&draft.provenance)?;
    validate_content(
        &draft.title,
        &draft.excerpt,
        &draft.topic_hints,
        draft.confidence_basis_points,
    )?;
    if !valid_timestamp(draft.created_at) {
        return Err(KnowledgeValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_provenance(
    value: &KnowledgeNoteProvenanceV1,
) -> Result<(), KnowledgeValidationErrorV1> {
    if !nonzero(&value.approved_candidate_id) {
        return Err(KnowledgeValidationErrorV1::InvalidCandidateId);
    }
    if !nonzero(&value.candidate_digest) {
        return Err(KnowledgeValidationErrorV1::InvalidCandidateDigest);
    }
    if !nonzero(&value.source_evidence_id) {
        return Err(KnowledgeValidationErrorV1::InvalidSourceEvidence);
    }
    if value.source_evidence_revision == 0 {
        return Err(KnowledgeValidationErrorV1::InvalidSourceRevision);
    }
    if !nonzero(&value.review_id) {
        return Err(KnowledgeValidationErrorV1::InvalidReviewId);
    }
    if value.decision_revision == 0 {
        return Err(KnowledgeValidationErrorV1::InvalidDecisionRevision);
    }
    if !nonzero(&value.decided_by_owner_device_id) {
        return Err(KnowledgeValidationErrorV1::InvalidDecisionActor);
    }
    Ok(())
}

fn validate_content(
    title: &str,
    excerpt: &str,
    topic_hints: &[KnowledgeNoteTopicHintV1],
    confidence_basis_points: u32,
) -> Result<(), KnowledgeValidationErrorV1> {
    if !valid_text(title, MAX_TITLE_CHARS_V1) {
        return Err(KnowledgeValidationErrorV1::InvalidTitle);
    }
    if !valid_excerpt(excerpt) {
        return Err(KnowledgeValidationErrorV1::InvalidExcerpt);
    }
    if topic_hints.is_empty()
        || topic_hints.len() > MAX_TOPIC_HINTS_V1
        || !topic_hints.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(KnowledgeValidationErrorV1::InvalidTopicHints);
    }
    if !(1..=10_000).contains(&confidence_basis_points) {
        return Err(KnowledgeValidationErrorV1::InvalidConfidence);
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

fn valid_excerpt(value: &str) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= MAX_EXCERPT_CHARS_V1
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n')
}

fn valid_timestamp(value: KnowledgeNoteTimestampV1) -> bool {
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

fn source_basis_byte(value: KnowledgeNoteSourceBasisV1) -> u8 {
    match value {
        KnowledgeNoteSourceBasisV1::Subject => 1,
        KnowledgeNoteSourceBasisV1::Body => 2,
        KnowledgeNoteSourceBasisV1::Combined => 3,
    }
}

fn topic_hint_byte(value: KnowledgeNoteTopicHintV1) -> u8 {
    match value {
        KnowledgeNoteTopicHintV1::Financial => 1,
        KnowledgeNoteTopicHintV1::Legal => 2,
        KnowledgeNoteTopicHintV1::DecisionStatement => 3,
        KnowledgeNoteTopicHintV1::DeadlineStatement => 4,
    }
}
