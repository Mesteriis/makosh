use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, KnowledgeNoteProvenanceV1, KnowledgeNoteTimestampV1,
    MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1, STABLE_ID_BYTES_V1,
};

pub const MAX_KNOWLEDGE_BODY_CHARS_V1: usize = 16_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeLifecycleStateV1 {
    Active,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeNoteOriginV1 {
    ReviewedCandidate,
    OwnerAuthored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeSourceStateV1 {
    Active,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeSourceV1 {
    pub source_id: [u8; STABLE_ID_BYTES_V1],
    pub source_owner_id: String,
    pub source_record_id: [u8; STABLE_ID_BYTES_V1],
    pub source_revision: u64,
    pub evidence_digest: [u8; DIGEST_BYTES_V1],
    pub state: KnowledgeSourceStateV1,
    pub updated_at_note_revision: u64,
    pub created_at: KnowledgeNoteTimestampV1,
    pub updated_at: KnowledgeNoteTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeNoteRecordV1 {
    pub note_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub body: String,
    pub state: KnowledgeLifecycleStateV1,
    pub origin: KnowledgeNoteOriginV1,
    pub note_revision: u64,
    pub reviewed_provenance: Option<KnowledgeNoteProvenanceV1>,
    pub sources: Vec<KnowledgeSourceV1>,
    pub created_at: KnowledgeNoteTimestampV1,
    pub updated_at: KnowledgeNoteTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualKnowledgeNoteDraftV1 {
    pub operation_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub body: String,
    pub created_at: KnowledgeNoteTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeLifecycleErrorV1 {
    InvalidOwner,
    InvalidOperationId,
    InvalidNoteId,
    InvalidTitle,
    InvalidBody,
    InvalidTimestamp,
    InvalidRevision,
    RevisionOverflow,
    InvalidStateTransition,
    InvalidSource,
    SourceExists,
    SourceNotFound,
}

pub fn derive_manual_knowledge_note_id_v1(
    logical_owner_id: &str,
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], KnowledgeLifecycleErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(KnowledgeLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(operation_id) {
        return Err(KnowledgeLifecycleErrorV1::InvalidOperationId);
    }
    Ok(derive_id(
        b"makosh.knowledge.owner-authored.note-id.v1\0",
        &[logical_owner_id.as_bytes(), operation_id],
    ))
}

pub fn derive_knowledge_source_id_v1(
    note_id: &[u8; STABLE_ID_BYTES_V1],
    source_owner_id: &str,
    source_record_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], KnowledgeLifecycleErrorV1> {
    if !nonzero(note_id) || !valid_owner(source_owner_id) || !nonzero(source_record_id) {
        return Err(KnowledgeLifecycleErrorV1::InvalidSource);
    }
    Ok(derive_id(
        b"makosh.knowledge.public-source-id.v1\0",
        &[note_id, source_owner_id.as_bytes(), source_record_id],
    ))
}

pub fn create_manual_knowledge_note_v1(
    draft: ManualKnowledgeNoteDraftV1,
) -> Result<KnowledgeNoteRecordV1, KnowledgeLifecycleErrorV1> {
    validate_content(&draft.title, &draft.body)?;
    validate_timestamp(draft.created_at)?;
    let note = KnowledgeNoteRecordV1 {
        note_id: derive_manual_knowledge_note_id_v1(&draft.logical_owner_id, &draft.operation_id)?,
        logical_owner_id: draft.logical_owner_id,
        title: draft.title,
        body: draft.body,
        state: KnowledgeLifecycleStateV1::Active,
        origin: KnowledgeNoteOriginV1::OwnerAuthored,
        note_revision: 1,
        reviewed_provenance: None,
        sources: Vec::new(),
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_knowledge_note_record_v1(&note)?;
    Ok(note)
}

pub fn update_knowledge_note_content_v1(
    note: &mut KnowledgeNoteRecordV1,
    expected_revision: u64,
    title: Option<String>,
    body: Option<String>,
    updated_at: KnowledgeNoteTimestampV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    require_mutable(note, expected_revision, updated_at)?;
    if title.is_none() && body.is_none() {
        return Err(KnowledgeLifecycleErrorV1::InvalidBody);
    }
    let next_title = title.unwrap_or_else(|| note.title.clone());
    let next_body = body.unwrap_or_else(|| note.body.clone());
    validate_content(&next_title, &next_body)?;
    note.title = next_title;
    note.body = next_body;
    advance(note, updated_at)
}

pub fn set_knowledge_note_state_v1(
    note: &mut KnowledgeNoteRecordV1,
    expected_revision: u64,
    state: KnowledgeLifecycleStateV1,
    changed_at: KnowledgeNoteTimestampV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    require_revision_and_time(note, expected_revision, changed_at)?;
    if note.state == state {
        return Err(KnowledgeLifecycleErrorV1::InvalidStateTransition);
    }
    note.state = state;
    advance(note, changed_at)
}

pub fn add_knowledge_source_v1(
    note: &mut KnowledgeNoteRecordV1,
    expected_revision: u64,
    source_owner_id: String,
    source_record_id: [u8; STABLE_ID_BYTES_V1],
    source_revision: u64,
    evidence_digest: [u8; DIGEST_BYTES_V1],
    changed_at: KnowledgeNoteTimestampV1,
) -> Result<[u8; STABLE_ID_BYTES_V1], KnowledgeLifecycleErrorV1> {
    require_mutable(note, expected_revision, changed_at)?;
    let source_id =
        derive_knowledge_source_id_v1(&note.note_id, &source_owner_id, &source_record_id)?;
    if source_revision == 0 || !nonzero(&evidence_digest) {
        return Err(KnowledgeLifecycleErrorV1::InvalidSource);
    }
    if let Some(existing) = note
        .sources
        .iter_mut()
        .find(|value| value.source_id == source_id)
    {
        if existing.state == KnowledgeSourceStateV1::Active
            || existing.source_revision != source_revision
            || existing.evidence_digest != evidence_digest
        {
            return Err(KnowledgeLifecycleErrorV1::SourceExists);
        }
        let next_revision = note
            .note_revision
            .checked_add(1)
            .ok_or(KnowledgeLifecycleErrorV1::RevisionOverflow)?;
        existing.state = KnowledgeSourceStateV1::Active;
        existing.updated_at_note_revision = next_revision;
        existing.updated_at = changed_at;
    } else {
        let next_revision = note
            .note_revision
            .checked_add(1)
            .ok_or(KnowledgeLifecycleErrorV1::RevisionOverflow)?;
        note.sources.push(KnowledgeSourceV1 {
            source_id,
            source_owner_id,
            source_record_id,
            source_revision,
            evidence_digest,
            state: KnowledgeSourceStateV1::Active,
            updated_at_note_revision: next_revision,
            created_at: changed_at,
            updated_at: changed_at,
        });
        note.sources.sort_by_key(|value| value.source_id);
    }
    advance(note, changed_at)?;
    Ok(source_id)
}

pub fn remove_knowledge_source_v1(
    note: &mut KnowledgeNoteRecordV1,
    expected_revision: u64,
    source_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: KnowledgeNoteTimestampV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    require_mutable(note, expected_revision, changed_at)?;
    let next_revision = note
        .note_revision
        .checked_add(1)
        .ok_or(KnowledgeLifecycleErrorV1::RevisionOverflow)?;
    let source = note
        .sources
        .iter_mut()
        .find(|value| value.source_id == source_id && value.state == KnowledgeSourceStateV1::Active)
        .ok_or(KnowledgeLifecycleErrorV1::SourceNotFound)?;
    source.state = KnowledgeSourceStateV1::Removed;
    source.updated_at_note_revision = next_revision;
    source.updated_at = changed_at;
    advance(note, changed_at)
}

pub fn validate_knowledge_note_record_v1(
    note: &KnowledgeNoteRecordV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    if !valid_owner(&note.logical_owner_id) {
        return Err(KnowledgeLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(&note.note_id) || note.note_revision == 0 {
        return Err(KnowledgeLifecycleErrorV1::InvalidNoteId);
    }
    validate_content(&note.title, &note.body)?;
    validate_timestamp(note.created_at)?;
    validate_timestamp(note.updated_at)?;
    if timestamp_millis(note.updated_at)? < timestamp_millis(note.created_at)? {
        return Err(KnowledgeLifecycleErrorV1::InvalidTimestamp);
    }
    if (note.origin == KnowledgeNoteOriginV1::ReviewedCandidate)
        != note.reviewed_provenance.is_some()
    {
        return Err(KnowledgeLifecycleErrorV1::InvalidNoteId);
    }
    if note
        .sources
        .windows(2)
        .any(|pair| pair[0].source_id >= pair[1].source_id)
    {
        return Err(KnowledgeLifecycleErrorV1::InvalidSource);
    }
    for source in &note.sources {
        let expected = derive_knowledge_source_id_v1(
            &note.note_id,
            &source.source_owner_id,
            &source.source_record_id,
        )?;
        if source.source_id != expected
            || source.source_revision == 0
            || !nonzero(&source.evidence_digest)
            || source.updated_at_note_revision == 0
            || source.updated_at_note_revision > note.note_revision
        {
            return Err(KnowledgeLifecycleErrorV1::InvalidSource);
        }
        validate_timestamp(source.created_at)?;
        validate_timestamp(source.updated_at)?;
    }
    Ok(())
}

fn require_mutable(
    note: &KnowledgeNoteRecordV1,
    expected_revision: u64,
    changed_at: KnowledgeNoteTimestampV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    require_revision_and_time(note, expected_revision, changed_at)?;
    if note.state != KnowledgeLifecycleStateV1::Active {
        return Err(KnowledgeLifecycleErrorV1::InvalidStateTransition);
    }
    Ok(())
}

fn require_revision_and_time(
    note: &KnowledgeNoteRecordV1,
    expected_revision: u64,
    changed_at: KnowledgeNoteTimestampV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    validate_knowledge_note_record_v1(note)?;
    if expected_revision == 0 || note.note_revision != expected_revision {
        return Err(KnowledgeLifecycleErrorV1::InvalidRevision);
    }
    validate_timestamp(changed_at)?;
    if timestamp_millis(changed_at)? < timestamp_millis(note.updated_at)? {
        return Err(KnowledgeLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn advance(
    note: &mut KnowledgeNoteRecordV1,
    changed_at: KnowledgeNoteTimestampV1,
) -> Result<(), KnowledgeLifecycleErrorV1> {
    note.note_revision = note
        .note_revision
        .checked_add(1)
        .ok_or(KnowledgeLifecycleErrorV1::RevisionOverflow)?;
    note.updated_at = changed_at;
    validate_knowledge_note_record_v1(note)
}

fn validate_content(title: &str, body: &str) -> Result<(), KnowledgeLifecycleErrorV1> {
    if !valid_text(title, MAX_TITLE_CHARS_V1) {
        return Err(KnowledgeLifecycleErrorV1::InvalidTitle);
    }
    if !valid_text(body, MAX_KNOWLEDGE_BODY_CHARS_V1) {
        return Err(KnowledgeLifecycleErrorV1::InvalidBody);
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
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
}

fn validate_timestamp(value: KnowledgeNoteTimestampV1) -> Result<(), KnowledgeLifecycleErrorV1> {
    timestamp_millis(value).map(|_| ())
}

fn timestamp_millis(value: KnowledgeNoteTimestampV1) -> Result<i128, KnowledgeLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..=999_999_999).contains(&value.nanos) {
        return Err(KnowledgeLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(i128::from(value.unix_seconds) * 1_000 + i128::from(value.nanos) / 1_000_000)
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn derive_id(domain: &[u8], parts: &[&[u8]]) -> [u8; STABLE_ID_BYTES_V1] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .expect("fixed digest")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: i64) -> KnowledgeNoteTimestampV1 {
        KnowledgeNoteTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }

    fn note() -> KnowledgeNoteRecordV1 {
        create_manual_knowledge_note_v1(ManualKnowledgeNoteDraftV1 {
            operation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            title: "Runbook".to_owned(),
            body: "Restart from the last durable checkpoint.".to_owned(),
            created_at: at(100),
        })
        .expect("note")
    }

    #[test]
    fn manual_note_id_and_content_lifecycle_are_deterministic() {
        let first = note();
        let second = note();
        assert_eq!(first, second);
        let mut updated = first;
        update_knowledge_note_content_v1(
            &mut updated,
            1,
            Some("Recovery runbook".to_owned()),
            None,
            at(101),
        )
        .expect("update");
        assert_eq!(updated.note_revision, 2);
        assert_eq!(updated.body, "Restart from the last durable checkpoint.");
    }

    #[test]
    fn archived_notes_reject_content_and_source_mutations_but_can_restore() {
        let mut value = note();
        set_knowledge_note_state_v1(&mut value, 1, KnowledgeLifecycleStateV1::Archived, at(101))
            .expect("archive");
        assert_eq!(
            update_knowledge_note_content_v1(
                &mut value,
                2,
                None,
                Some("changed".to_owned()),
                at(102),
            ),
            Err(KnowledgeLifecycleErrorV1::InvalidStateTransition)
        );
        set_knowledge_note_state_v1(&mut value, 2, KnowledgeLifecycleStateV1::Active, at(102))
            .expect("restore");
    }

    #[test]
    fn source_identity_is_public_deterministic_and_revisioned() {
        let mut value = note();
        let source_id = add_knowledge_source_v1(
            &mut value,
            1,
            "communications".to_owned(),
            [2; 16],
            3,
            [4; 32],
            at(101),
        )
        .expect("add");
        assert_eq!(value.note_revision, 2);
        remove_knowledge_source_v1(&mut value, 2, source_id, at(102)).expect("remove");
        assert_eq!(value.note_revision, 3);
        assert_eq!(value.sources[0].state, KnowledgeSourceStateV1::Removed);
        add_knowledge_source_v1(
            &mut value,
            3,
            "communications".to_owned(),
            [2; 16],
            3,
            [4; 32],
            at(103),
        )
        .expect("reactivate");
        assert_eq!(value.sources[0].source_id, source_id);
        assert_eq!(value.note_revision, 4);
    }

    #[test]
    fn revisions_and_private_shaped_source_data_fail_closed() {
        let mut value = note();
        assert_eq!(
            update_knowledge_note_content_v1(
                &mut value,
                u64::MAX,
                None,
                Some("changed".to_owned()),
                at(101),
            ),
            Err(KnowledgeLifecycleErrorV1::InvalidRevision)
        );
        assert_eq!(
            add_knowledge_source_v1(
                &mut value,
                1,
                "provider/private".to_owned(),
                [2; 16],
                3,
                [4; 32],
                at(101),
            ),
            Err(KnowledgeLifecycleErrorV1::InvalidSource)
        );
    }
}
