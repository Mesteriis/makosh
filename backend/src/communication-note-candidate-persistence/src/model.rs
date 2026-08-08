use makosh_communication_note_candidate_core::{
    COMMUNICATION_NOTE_MAX_CANDIDATES_V1, COMMUNICATION_NOTE_MAX_CONFIDENCE_BASIS_POINTS_V1,
    COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1, COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1,
    CommunicationNoteCandidateDraftV1, CommunicationNoteCandidateRejectionCodeV1,
    CommunicationNoteCandidateStatusV1, CommunicationNoteCandidateTransitionV1,
    CommunicationNoteCandidateV1, CommunicationNoteSourceBasisV1, CommunicationNoteTopicHintV1,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_NOTE_CANDIDATE_RECOVERY_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_NOTE_CANDIDATE_REALTIME_LIMIT_V1: u16 = 1_024;
pub const COMMUNICATION_NOTE_CANDIDATE_OUTBOX_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_NOTE_CANDIDATE_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_NOTE_CANDIDATE_MAX_SOURCE_READ_RECEIPT_BYTES_V1: usize = 16 * 1024;
pub const COMMUNICATION_NOTE_CANDIDATE_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const COMMUNICATION_NOTE_CANDIDATE_MAX_ENCODED_CANDIDATES_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteCandidateBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCommunicationNoteCandidateRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationNoteCandidateDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCommunicationNoteCandidateRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationNoteCandidateDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: CommunicationNoteCandidateStatusV1,
    pub source_read_receipt_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationNoteCandidateBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCommunicationNoteCandidateOutcomeV1 {
    Created(PersistedCommunicationNoteCandidateRunV1),
    Existing(PersistedCommunicationNoteCandidateRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteCandidateSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: CommunicationNoteCandidateTransitionV1,
    pub source_read_receipt_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationNoteCandidateBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateInboxResultV1 {
    Applied(PersistedCommunicationNoteCandidateRunV1),
    Duplicate(PersistedCommunicationNoteCandidateRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCommunicationNoteCandidateEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidatePersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &CommunicationNoteCandidateDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_note_candidate_extraction.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.finalize().into()
}

pub(crate) fn encode_candidates(
    candidates: &[CommunicationNoteCandidateV1],
) -> Result<Vec<u8>, CommunicationNoteCandidatePersistenceErrorV1> {
    if candidates.len() > COMMUNICATION_NOTE_MAX_CANDIDATES_V1 {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidInput);
    }
    let mut bytes = Vec::new();
    bytes.push(1);
    bytes.push(
        u8::try_from(candidates.len())
            .map_err(|_| CommunicationNoteCandidatePersistenceErrorV1::InvalidInput)?,
    );
    for candidate in candidates {
        if !valid_candidate(candidate) {
            return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        bytes.extend_from_slice(&candidate.candidate_id);
        bytes.extend_from_slice(&candidate.candidate_digest);
        bytes.push(source_basis_code(candidate.source_basis));
        bytes.extend_from_slice(&candidate.confidence_basis_points.to_be_bytes());
        bytes.extend_from_slice(&candidate.source_evidence_id);
        bytes.extend_from_slice(&candidate.source_evidence_revision.to_be_bytes());
        encode_text(&mut bytes, &candidate.title)?;
        encode_text(&mut bytes, &candidate.excerpt)?;
        bytes.push(
            u8::try_from(candidate.topic_hints.len())
                .map_err(|_| CommunicationNoteCandidatePersistenceErrorV1::InvalidInput)?,
        );
        for hint in &candidate.topic_hints {
            bytes.push(topic_hint_code(*hint));
        }
    }
    if bytes.len() > COMMUNICATION_NOTE_CANDIDATE_MAX_ENCODED_CANDIDATES_BYTES_V1 {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidInput);
    }
    Ok(bytes)
}

pub(crate) fn decode_candidates(
    bytes: &[u8],
) -> Result<Vec<CommunicationNoteCandidateV1>, CommunicationNoteCandidatePersistenceErrorV1> {
    if bytes.len() < 2 || bytes.len() > COMMUNICATION_NOTE_CANDIDATE_MAX_ENCODED_CANDIDATES_BYTES_V1
    {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    let mut cursor = CandidateCursor::new(bytes);
    if cursor.byte()? != 1 {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    let count = usize::from(cursor.byte()?);
    if count > COMMUNICATION_NOTE_MAX_CANDIDATES_V1 {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        let candidate = CommunicationNoteCandidateV1 {
            candidate_id: cursor.array()?,
            candidate_digest: cursor.array()?,
            source_basis: source_basis_from_code(cursor.byte()?)?,
            confidence_basis_points: cursor.u32()?,
            source_evidence_id: cursor.array()?,
            source_evidence_revision: cursor.u64()?,
            title: cursor.text()?,
            excerpt: cursor.text()?,
            topic_hints: decode_topic_hints(&mut cursor)?,
        };
        if !valid_candidate(&candidate) {
            return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
        }
        candidates.push(candidate);
    }
    if !cursor.finished() {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    Ok(candidates)
}

fn encode_text(
    bytes: &mut Vec<u8>,
    value: &str,
) -> Result<(), CommunicationNoteCandidatePersistenceErrorV1> {
    let length = u16::try_from(value.len())
        .map_err(|_| CommunicationNoteCandidatePersistenceErrorV1::InvalidInput)?;
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
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
    ) -> Result<&'a [u8], CommunicationNoteCandidatePersistenceErrorV1> {
        let end = self
            .position
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow)?;
        let value = &self.bytes[self.position..end];
        self.position = end;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8, CommunicationNoteCandidatePersistenceErrorV1> {
        Ok(self.take(1)?[0])
    }

    fn array<const N: usize>(
        &mut self,
    ) -> Result<[u8; N], CommunicationNoteCandidatePersistenceErrorV1> {
        self.take(N)?
            .try_into()
            .map_err(|_| CommunicationNoteCandidatePersistenceErrorV1::InvalidRow)
    }

    fn u32(&mut self) -> Result<u32, CommunicationNoteCandidatePersistenceErrorV1> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, CommunicationNoteCandidatePersistenceErrorV1> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn text(&mut self) -> Result<String, CommunicationNoteCandidatePersistenceErrorV1> {
        let length = usize::from(u16::from_be_bytes(self.array()?));
        std::str::from_utf8(self.take(length)?)
            .map(str::to_owned)
            .map_err(|_| CommunicationNoteCandidatePersistenceErrorV1::InvalidRow)
    }

    fn finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

const fn source_basis_code(value: CommunicationNoteSourceBasisV1) -> u8 {
    match value {
        CommunicationNoteSourceBasisV1::Subject => 1,
        CommunicationNoteSourceBasisV1::Body => 2,
        CommunicationNoteSourceBasisV1::Combined => 3,
    }
}

fn source_basis_from_code(
    value: u8,
) -> Result<CommunicationNoteSourceBasisV1, CommunicationNoteCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationNoteSourceBasisV1::Subject),
        2 => Ok(CommunicationNoteSourceBasisV1::Body),
        3 => Ok(CommunicationNoteSourceBasisV1::Combined),
        _ => Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow),
    }
}

const fn topic_hint_code(value: CommunicationNoteTopicHintV1) -> u8 {
    match value {
        CommunicationNoteTopicHintV1::Financial => 1,
        CommunicationNoteTopicHintV1::Legal => 2,
        CommunicationNoteTopicHintV1::DecisionStatement => 3,
        CommunicationNoteTopicHintV1::DeadlineStatement => 4,
    }
}

fn topic_hint_from_code(
    value: u8,
) -> Result<CommunicationNoteTopicHintV1, CommunicationNoteCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationNoteTopicHintV1::Financial),
        2 => Ok(CommunicationNoteTopicHintV1::Legal),
        3 => Ok(CommunicationNoteTopicHintV1::DecisionStatement),
        4 => Ok(CommunicationNoteTopicHintV1::DeadlineStatement),
        _ => Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn decode_topic_hints(
    cursor: &mut CandidateCursor<'_>,
) -> Result<Vec<CommunicationNoteTopicHintV1>, CommunicationNoteCandidatePersistenceErrorV1> {
    let count = usize::from(cursor.byte()?);
    if !(1..=4).contains(&count) {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    let mut hints = Vec::with_capacity(count);
    for _ in 0..count {
        hints.push(topic_hint_from_code(cursor.byte()?)?);
    }
    if hints.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    Ok(hints)
}

fn valid_candidate(candidate: &CommunicationNoteCandidateV1) -> bool {
    nonzero(&candidate.candidate_id)
        && nonzero(&candidate.candidate_digest)
        && !candidate.title.is_empty()
        && candidate.title.chars().count() <= COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1
        && candidate.excerpt.chars().count() <= COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1
        && (1..=4).contains(&candidate.topic_hints.len())
        && !candidate
            .topic_hints
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        && (1..=COMMUNICATION_NOTE_MAX_CONFIDENCE_BASIS_POINTS_V1)
            .contains(&candidate.confidence_basis_points)
        && nonzero(&candidate.source_evidence_id)
        && candidate.source_evidence_revision > 0
}

pub(crate) const fn rejection_code(value: CommunicationNoteCandidateRejectionCodeV1) -> i16 {
    match value {
        CommunicationNoteCandidateRejectionCodeV1::InvalidRequest => 1,
        CommunicationNoteCandidateRejectionCodeV1::SourceRejected => 2,
        CommunicationNoteCandidateRejectionCodeV1::ExtractionRejected => 3,
        CommunicationNoteCandidateRejectionCodeV1::Policy => 4,
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
        let draft = CommunicationNoteCandidateDraftV1 {
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
        let candidates = vec![CommunicationNoteCandidateV1 {
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            title: "Contract approved".to_owned(),
            excerpt: "The agreement amount is confirmed.".to_owned(),
            topic_hints: vec![
                CommunicationNoteTopicHintV1::Financial,
                CommunicationNoteTopicHintV1::Legal,
                CommunicationNoteTopicHintV1::DecisionStatement,
            ],
            source_basis: CommunicationNoteSourceBasisV1::Combined,
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
            Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow)
        );
        assert_eq!(
            decode_candidates(&[1]),
            Err(CommunicationNoteCandidatePersistenceErrorV1::InvalidRow)
        );
    }
}
