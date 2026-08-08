use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_HINT_CHARS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1,
    STABLE_ID_BYTES_V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskStatusV1 {
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskProvenanceV1 {
    pub approved_candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub review_id: [u8; STABLE_ID_BYTES_V1],
    pub decision_revision: u64,
    pub decided_by_owner_device_id: [u8; STABLE_ID_BYTES_V1],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedCandidateTaskDraftV1 {
    pub logical_owner_id: String,
    pub provenance: TaskProvenanceV1,
    pub title: String,
    pub due_text_hint: Option<String>,
    pub assignee_label_hint: Option<String>,
    pub created_at: TaskTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskV1 {
    pub task_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub due_text_hint: Option<String>,
    pub assignee_label_hint: Option<String>,
    pub status: TaskStatusV1,
    pub task_revision: u64,
    pub provenance: TaskProvenanceV1,
    pub created_at: TaskTimestampV1,
    pub updated_at: TaskTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TasksValidationErrorV1 {
    InvalidOwner,
    InvalidTaskId,
    InvalidCandidateId,
    InvalidCandidateDigest,
    InvalidSourceEvidence,
    InvalidSourceRevision,
    InvalidReviewId,
    InvalidDecisionRevision,
    InvalidDecisionActor,
    InvalidTitle,
    InvalidDueTextHint,
    InvalidAssigneeLabelHint,
    InvalidTimestamp,
    InvalidRevision,
}

pub fn derive_task_id_v1(
    logical_owner_id: &str,
    approved_candidate_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], TasksValidationErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(TasksValidationErrorV1::InvalidOwner);
    }
    if !nonzero(approved_candidate_id) {
        return Err(TasksValidationErrorV1::InvalidCandidateId);
    }
    Ok(digest(
        b"makosh.tasks.reviewed-candidate.task-id.v1",
        logical_owner_id.as_bytes(),
        approved_candidate_id,
    ))
}

pub fn task_creation_fingerprint_v1(
    draft: &ReviewedCandidateTaskDraftV1,
) -> Result<[u8; DIGEST_BYTES_V1], TasksValidationErrorV1> {
    validate_draft(draft)?;
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.tasks.reviewed-candidate.creation.v1");
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
    update_optional(&mut hasher, draft.due_text_hint.as_deref());
    update_optional(&mut hasher, draft.assignee_label_hint.as_deref());
    Ok(hasher.finalize().into())
}

pub fn validate_task_v1(task: &TaskV1) -> Result<(), TasksValidationErrorV1> {
    let expected = derive_task_id_v1(
        &task.logical_owner_id,
        &task.provenance.approved_candidate_id,
    )?;
    if task.task_id != expected || !nonzero(&task.task_id) {
        return Err(TasksValidationErrorV1::InvalidTaskId);
    }
    validate_provenance(&task.provenance)?;
    validate_content(
        &task.title,
        task.due_text_hint.as_deref(),
        task.assignee_label_hint.as_deref(),
    )?;
    if task.task_revision == 0 {
        return Err(TasksValidationErrorV1::InvalidRevision);
    }
    if !valid_timestamp(task.created_at)
        || !valid_timestamp(task.updated_at)
        || task.updated_at.unix_seconds < task.created_at.unix_seconds
    {
        return Err(TasksValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

pub(crate) fn validate_draft(
    draft: &ReviewedCandidateTaskDraftV1,
) -> Result<(), TasksValidationErrorV1> {
    derive_task_id_v1(
        &draft.logical_owner_id,
        &draft.provenance.approved_candidate_id,
    )?;
    validate_provenance(&draft.provenance)?;
    validate_content(
        &draft.title,
        draft.due_text_hint.as_deref(),
        draft.assignee_label_hint.as_deref(),
    )?;
    if !valid_timestamp(draft.created_at) {
        return Err(TasksValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_provenance(value: &TaskProvenanceV1) -> Result<(), TasksValidationErrorV1> {
    if !nonzero(&value.approved_candidate_id) {
        return Err(TasksValidationErrorV1::InvalidCandidateId);
    }
    if !nonzero(&value.candidate_digest) {
        return Err(TasksValidationErrorV1::InvalidCandidateDigest);
    }
    if !nonzero(&value.source_evidence_id) {
        return Err(TasksValidationErrorV1::InvalidSourceEvidence);
    }
    if value.source_evidence_revision == 0 {
        return Err(TasksValidationErrorV1::InvalidSourceRevision);
    }
    if !nonzero(&value.review_id) {
        return Err(TasksValidationErrorV1::InvalidReviewId);
    }
    if value.decision_revision == 0 {
        return Err(TasksValidationErrorV1::InvalidDecisionRevision);
    }
    if !nonzero(&value.decided_by_owner_device_id) {
        return Err(TasksValidationErrorV1::InvalidDecisionActor);
    }
    Ok(())
}

fn validate_content(
    title: &str,
    due_text_hint: Option<&str>,
    assignee_label_hint: Option<&str>,
) -> Result<(), TasksValidationErrorV1> {
    if !valid_text(title, MAX_TITLE_CHARS_V1) {
        return Err(TasksValidationErrorV1::InvalidTitle);
    }
    if due_text_hint.is_some_and(|value| !valid_text(value, MAX_HINT_CHARS_V1)) {
        return Err(TasksValidationErrorV1::InvalidDueTextHint);
    }
    if assignee_label_hint.is_some_and(|value| !valid_text(value, MAX_HINT_CHARS_V1)) {
        return Err(TasksValidationErrorV1::InvalidAssigneeLabelHint);
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

fn valid_timestamp(value: TaskTimestampV1) -> bool {
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

fn update_optional(hasher: &mut Sha256, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            update_part(hasher, value.as_bytes());
        }
        None => hasher.update([0]),
    }
}
