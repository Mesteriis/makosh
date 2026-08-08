use makosh_communication_task_candidate_core::{
    COMMUNICATION_TASK_MAX_CANDIDATES_V1, COMMUNICATION_TASK_MAX_CONFIDENCE_BASIS_POINTS_V1,
    COMMUNICATION_TASK_MAX_HINT_CHARS_V1, COMMUNICATION_TASK_MAX_TITLE_CHARS_V1,
    CommunicationTaskCandidateDraftV1, CommunicationTaskCandidateRejectionCodeV1,
    CommunicationTaskCandidateStatusV1, CommunicationTaskCandidateTransitionV1,
    CommunicationTaskCandidateV1, CommunicationTaskSignalKindV1, CommunicationTaskSourceBasisV1,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_TASK_CANDIDATE_RECOVERY_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_TASK_CANDIDATE_REALTIME_LIMIT_V1: u16 = 1_024;
pub const COMMUNICATION_TASK_CANDIDATE_OUTBOX_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_TASK_CANDIDATE_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_TASK_CANDIDATE_MAX_SOURCE_READ_RECEIPT_BYTES_V1: usize = 16 * 1024;
pub const COMMUNICATION_TASK_CANDIDATE_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const COMMUNICATION_TASK_CANDIDATE_MAX_ENCODED_CANDIDATES_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTaskCandidateBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCommunicationTaskCandidateRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationTaskCandidateDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCommunicationTaskCandidateRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationTaskCandidateDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: CommunicationTaskCandidateStatusV1,
    pub source_read_receipt_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationTaskCandidateBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCommunicationTaskCandidateOutcomeV1 {
    Created(PersistedCommunicationTaskCandidateRunV1),
    Existing(PersistedCommunicationTaskCandidateRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTaskCandidateSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: CommunicationTaskCandidateTransitionV1,
    pub source_read_receipt_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationTaskCandidateBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateInboxResultV1 {
    Applied(PersistedCommunicationTaskCandidateRunV1),
    Duplicate(PersistedCommunicationTaskCandidateRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCommunicationTaskCandidateEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidatePersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &CommunicationTaskCandidateDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_task_candidate_extraction.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.finalize().into()
}

pub(crate) fn encode_candidates(
    candidates: &[CommunicationTaskCandidateV1],
) -> Result<Vec<u8>, CommunicationTaskCandidatePersistenceErrorV1> {
    if candidates.len() > COMMUNICATION_TASK_MAX_CANDIDATES_V1 {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidInput);
    }
    let mut bytes = Vec::new();
    bytes.push(1);
    bytes.push(
        u8::try_from(candidates.len())
            .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidInput)?,
    );
    for candidate in candidates {
        if !valid_candidate(candidate) {
            return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidInput);
        }
        bytes.extend_from_slice(&candidate.candidate_id);
        bytes.extend_from_slice(&candidate.candidate_digest);
        bytes.push(source_basis_code(candidate.source_basis));
        bytes.push(signal_kind_code(candidate.signal_kind));
        bytes.extend_from_slice(&candidate.confidence_basis_points.to_be_bytes());
        bytes.extend_from_slice(&candidate.source_evidence_id);
        bytes.extend_from_slice(&candidate.source_evidence_revision.to_be_bytes());
        encode_text(&mut bytes, &candidate.title)?;
        encode_optional_text(&mut bytes, candidate.due_text_hint.as_deref())?;
        encode_optional_text(&mut bytes, candidate.assignee_label_hint.as_deref())?;
    }
    if bytes.len() > COMMUNICATION_TASK_CANDIDATE_MAX_ENCODED_CANDIDATES_BYTES_V1 {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidInput);
    }
    Ok(bytes)
}

pub(crate) fn decode_candidates(
    bytes: &[u8],
) -> Result<Vec<CommunicationTaskCandidateV1>, CommunicationTaskCandidatePersistenceErrorV1> {
    if bytes.len() < 2 || bytes.len() > COMMUNICATION_TASK_CANDIDATE_MAX_ENCODED_CANDIDATES_BYTES_V1
    {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
    }
    let mut cursor = CandidateCursor::new(bytes);
    if cursor.byte()? != 1 {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
    }
    let count = usize::from(cursor.byte()?);
    if count > COMMUNICATION_TASK_MAX_CANDIDATES_V1 {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
    }
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        let candidate = CommunicationTaskCandidateV1 {
            candidate_id: cursor.array()?,
            candidate_digest: cursor.array()?,
            source_basis: source_basis_from_code(cursor.byte()?)?,
            signal_kind: signal_kind_from_code(cursor.byte()?)?,
            confidence_basis_points: cursor.u32()?,
            source_evidence_id: cursor.array()?,
            source_evidence_revision: cursor.u64()?,
            title: cursor.text()?,
            due_text_hint: cursor.optional_text()?,
            assignee_label_hint: cursor.optional_text()?,
        };
        if !valid_candidate(&candidate) {
            return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
        }
        candidates.push(candidate);
    }
    if !cursor.finished() {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
    }
    Ok(candidates)
}

fn encode_text(
    bytes: &mut Vec<u8>,
    value: &str,
) -> Result<(), CommunicationTaskCandidatePersistenceErrorV1> {
    let length = u16::try_from(value.len())
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidInput)?;
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_optional_text(
    bytes: &mut Vec<u8>,
    value: Option<&str>,
) -> Result<(), CommunicationTaskCandidatePersistenceErrorV1> {
    match value {
        Some(value) => {
            bytes.push(1);
            encode_text(bytes, value)
        }
        None => {
            bytes.push(0);
            Ok(())
        }
    }
}

struct CandidateCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> CandidateCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(
        &mut self,
        length: usize,
    ) -> Result<&'a [u8], CommunicationTaskCandidatePersistenceErrorV1> {
        let end = self
            .position
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
        let value = &self.bytes[self.position..end];
        self.position = end;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8, CommunicationTaskCandidatePersistenceErrorV1> {
        Ok(self.take(1)?[0])
    }

    fn array<const N: usize>(
        &mut self,
    ) -> Result<[u8; N], CommunicationTaskCandidatePersistenceErrorV1> {
        self.take(N)?
            .try_into()
            .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)
    }

    fn u32(&mut self) -> Result<u32, CommunicationTaskCandidatePersistenceErrorV1> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, CommunicationTaskCandidatePersistenceErrorV1> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn text(&mut self) -> Result<String, CommunicationTaskCandidatePersistenceErrorV1> {
        let length = usize::from(u16::from_be_bytes(self.array()?));
        std::str::from_utf8(self.take(length)?)
            .map(str::to_owned)
            .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)
    }

    fn optional_text(
        &mut self,
    ) -> Result<Option<String>, CommunicationTaskCandidatePersistenceErrorV1> {
        match self.byte()? {
            0 => Ok(None),
            1 => self.text().map(Some),
            _ => Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow),
        }
    }

    fn finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

const fn source_basis_code(value: CommunicationTaskSourceBasisV1) -> u8 {
    match value {
        CommunicationTaskSourceBasisV1::Subject => 1,
        CommunicationTaskSourceBasisV1::Body => 2,
        CommunicationTaskSourceBasisV1::Combined => 3,
    }
}

fn source_basis_from_code(
    value: u8,
) -> Result<CommunicationTaskSourceBasisV1, CommunicationTaskCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationTaskSourceBasisV1::Subject),
        2 => Ok(CommunicationTaskSourceBasisV1::Body),
        3 => Ok(CommunicationTaskSourceBasisV1::Combined),
        _ => Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow),
    }
}

const fn signal_kind_code(value: CommunicationTaskSignalKindV1) -> u8 {
    match value {
        CommunicationTaskSignalKindV1::ExplicitAction => 1,
        CommunicationTaskSignalKindV1::DirectRequest => 2,
        CommunicationTaskSignalKindV1::FollowUp => 3,
    }
}

fn signal_kind_from_code(
    value: u8,
) -> Result<CommunicationTaskSignalKindV1, CommunicationTaskCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationTaskSignalKindV1::ExplicitAction),
        2 => Ok(CommunicationTaskSignalKindV1::DirectRequest),
        3 => Ok(CommunicationTaskSignalKindV1::FollowUp),
        _ => Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn valid_candidate(candidate: &CommunicationTaskCandidateV1) -> bool {
    nonzero(&candidate.candidate_id)
        && nonzero(&candidate.candidate_digest)
        && !candidate.title.is_empty()
        && candidate.title.chars().count() <= COMMUNICATION_TASK_MAX_TITLE_CHARS_V1
        && candidate.due_text_hint.as_ref().is_none_or(|value| {
            !value.is_empty() && value.chars().count() <= COMMUNICATION_TASK_MAX_HINT_CHARS_V1
        })
        && candidate.assignee_label_hint.as_ref().is_none_or(|value| {
            !value.is_empty() && value.chars().count() <= COMMUNICATION_TASK_MAX_HINT_CHARS_V1
        })
        && (1..=COMMUNICATION_TASK_MAX_CONFIDENCE_BASIS_POINTS_V1)
            .contains(&candidate.confidence_basis_points)
        && nonzero(&candidate.source_evidence_id)
        && candidate.source_evidence_revision > 0
}

pub(crate) const fn rejection_code(value: CommunicationTaskCandidateRejectionCodeV1) -> i16 {
    match value {
        CommunicationTaskCandidateRejectionCodeV1::InvalidRequest => 1,
        CommunicationTaskCandidateRejectionCodeV1::SourceRejected => 2,
        CommunicationTaskCandidateRejectionCodeV1::ExtractionRejected => 3,
        CommunicationTaskCandidateRejectionCodeV1::Policy => 4,
    }
}

pub(crate) fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    value > 0
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_fingerprint_is_stable_and_excludes_run_identity() {
        let draft = CommunicationTaskCandidateDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 4,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.expected_source_revision = 5;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }

    #[test]
    fn candidate_codec_preserves_all_typed_fields_and_empty_result() {
        let candidates = vec![CommunicationTaskCandidateV1 {
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            title: "Action: send signed report".to_owned(),
            due_text_hint: Some("tomorrow".to_owned()),
            assignee_label_hint: Some("owner".to_owned()),
            source_basis: CommunicationTaskSourceBasisV1::Combined,
            signal_kind: CommunicationTaskSignalKindV1::ExplicitAction,
            confidence_basis_points: 9_000,
            source_evidence_id: [3; 16],
            source_evidence_revision: 7,
        }];
        let encoded = encode_candidates(&candidates).expect("encode");
        assert_eq!(decode_candidates(&encoded), Ok(candidates));
        assert_eq!(
            decode_candidates(&encode_candidates(&[]).expect("empty")),
            Ok(vec![])
        );
    }

    #[test]
    fn candidate_codec_rejects_trailing_or_truncated_bytes() {
        let mut encoded = encode_candidates(&[]).expect("empty");
        encoded.push(0);
        assert_eq!(
            decode_candidates(&encoded),
            Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)
        );
        assert_eq!(
            decode_candidates(&[1]),
            Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)
        );
    }
}
