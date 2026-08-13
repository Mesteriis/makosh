#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-decisions-core";
pub const MAX_TITLE_CHARS_V1: usize = 240;
pub const MAX_QUESTION_CHARS_V1: usize = 4_000;
pub const MAX_RATIONALE_CHARS_V1: usize = 8_000;
pub const MAX_ALTERNATIVE_DESCRIPTION_CHARS_V1: usize = 8_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DecisionTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionStateV1 {
    Draft,
    Decided,
    Superseded,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionAlternativeStateV1 {
    Candidate,
    Selected,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionAlternativeV1 {
    pub alternative_id: [u8; 16],
    pub decision_id: [u8; 16],
    pub title: String,
    pub description: String,
    pub state: DecisionAlternativeStateV1,
    pub alternative_revision: u64,
    pub updated_at_decision_revision: u64,
    pub created_at: DecisionTimestampV1,
    pub updated_at: DecisionTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionEvidenceLinkV1 {
    pub evidence_link_id: [u8; 16],
    pub evidence_owner_id: String,
    pub evidence_record_id: [u8; 16],
    pub evidence_revision: u64,
    pub evidence_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRecordV1 {
    pub decision_id: [u8; 16],
    pub logical_owner_id: String,
    pub title: String,
    pub question: String,
    pub rationale: String,
    pub state: DecisionStateV1,
    pub selected_alternative_id: Option<[u8; 16]>,
    pub superseded_by_decision_id: Option<[u8; 16]>,
    pub decision_revision: u64,
    pub alternatives: Vec<DecisionAlternativeV1>,
    pub evidence: Vec<DecisionEvidenceLinkV1>,
    pub created_at: DecisionTimestampV1,
    pub updated_at: DecisionTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionLifecycleErrorV1 {
    InvalidInput,
    InvalidOwner,
    InvalidRevision,
    RevisionOverflow,
    InvalidStateTransition,
    AlternativeConflict,
    AlternativeNotFound,
    EvidenceConflict,
    EvidenceNotFound,
}

pub fn derive_decision_id_v1(
    owner: &str,
    operation_id: &[u8; 16],
) -> Result<[u8; 16], DecisionLifecycleErrorV1> {
    if !valid_owner(owner) || !nonzero(operation_id) {
        return Err(DecisionLifecycleErrorV1::InvalidOwner);
    }
    Ok(derive_id(
        b"makosh.decisions.decision-id.v1\0",
        &[owner.as_bytes(), operation_id],
    ))
}

pub fn derive_alternative_id_v1(
    decision_id: &[u8; 16],
    operation_id: &[u8; 16],
) -> Result<[u8; 16], DecisionLifecycleErrorV1> {
    if !nonzero(decision_id) || !nonzero(operation_id) {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    Ok(derive_id(
        b"makosh.decisions.alternative-id.v1\0",
        &[decision_id, operation_id],
    ))
}

pub fn create_decision_v1(
    owner: String,
    operation_id: [u8; 16],
    title: String,
    question: String,
    created_at: DecisionTimestampV1,
) -> Result<DecisionRecordV1, DecisionLifecycleErrorV1> {
    validate_text(&title, 1, MAX_TITLE_CHARS_V1)?;
    validate_text(&question, 1, MAX_QUESTION_CHARS_V1)?;
    validate_timestamp(created_at)?;
    Ok(DecisionRecordV1 {
        decision_id: derive_decision_id_v1(&owner, &operation_id)?,
        logical_owner_id: owner,
        title,
        question,
        rationale: String::new(),
        state: DecisionStateV1::Draft,
        selected_alternative_id: None,
        superseded_by_decision_id: None,
        decision_revision: 1,
        alternatives: Vec::new(),
        evidence: Vec::new(),
        created_at,
        updated_at: created_at,
    })
}

pub fn update_decision_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    title: Option<String>,
    question: Option<String>,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    if title.is_none() && question.is_none() {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    if let Some(value) = title {
        validate_text(&value, 1, MAX_TITLE_CHARS_V1)?;
        decision.title = value;
    }
    if let Some(value) = question {
        validate_text(&value, 1, MAX_QUESTION_CHARS_V1)?;
        decision.question = value;
    }
    advance(decision, changed_at)
}

pub fn add_alternative_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    operation_id: [u8; 16],
    title: String,
    description: String,
    changed_at: DecisionTimestampV1,
) -> Result<[u8; 16], DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    validate_text(&title, 1, MAX_TITLE_CHARS_V1)?;
    validate_text(&description, 0, MAX_ALTERNATIVE_DESCRIPTION_CHARS_V1)?;
    let alternative_id = derive_alternative_id_v1(&decision.decision_id, &operation_id)?;
    if decision
        .alternatives
        .iter()
        .any(|value| value.alternative_id == alternative_id)
    {
        return Err(DecisionLifecycleErrorV1::AlternativeConflict);
    }
    let next_revision = next_revision(decision.decision_revision)?;
    decision.alternatives.push(DecisionAlternativeV1 {
        alternative_id,
        decision_id: decision.decision_id,
        title,
        description,
        state: DecisionAlternativeStateV1::Candidate,
        alternative_revision: 1,
        updated_at_decision_revision: next_revision,
        created_at: changed_at,
        updated_at: changed_at,
    });
    decision.decision_revision = next_revision;
    decision.updated_at = changed_at;
    Ok(alternative_id)
}

#[allow(clippy::too_many_arguments)]
pub fn update_alternative_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    alternative_id: [u8; 16],
    expected_alternative_revision: u64,
    title: Option<String>,
    description: Option<String>,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    if title.is_none() && description.is_none() {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    let alternative = decision
        .alternatives
        .iter_mut()
        .find(|value| value.alternative_id == alternative_id)
        .ok_or(DecisionLifecycleErrorV1::AlternativeNotFound)?;
    if alternative.alternative_revision != expected_alternative_revision {
        return Err(DecisionLifecycleErrorV1::InvalidRevision);
    }
    if let Some(value) = title {
        validate_text(&value, 1, MAX_TITLE_CHARS_V1)?;
        alternative.title = value;
    }
    if let Some(value) = description {
        validate_text(&value, 0, MAX_ALTERNATIVE_DESCRIPTION_CHARS_V1)?;
        alternative.description = value;
    }
    let next = next_revision(decision.decision_revision)?;
    alternative.alternative_revision = next_revision(alternative.alternative_revision)?;
    alternative.updated_at_decision_revision = next;
    alternative.updated_at = changed_at;
    decision.decision_revision = next;
    decision.updated_at = changed_at;
    Ok(())
}

pub fn remove_alternative_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    alternative_id: [u8; 16],
    expected_alternative_revision: u64,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    let index = decision
        .alternatives
        .iter()
        .position(|value| value.alternative_id == alternative_id)
        .ok_or(DecisionLifecycleErrorV1::AlternativeNotFound)?;
    if decision.alternatives[index].alternative_revision != expected_alternative_revision {
        return Err(DecisionLifecycleErrorV1::InvalidRevision);
    }
    decision.alternatives.remove(index);
    advance(decision, changed_at)
}

pub fn add_evidence_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    evidence: DecisionEvidenceLinkV1,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    validate_evidence(&evidence)?;
    if decision.evidence.iter().any(|value| {
        value.evidence_link_id == evidence.evidence_link_id
            || (value.evidence_owner_id == evidence.evidence_owner_id
                && value.evidence_record_id == evidence.evidence_record_id
                && value.evidence_revision == evidence.evidence_revision)
    }) {
        return Err(DecisionLifecycleErrorV1::EvidenceConflict);
    }
    decision.evidence.push(evidence);
    advance(decision, changed_at)
}

pub fn remove_evidence_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    evidence_link_id: [u8; 16],
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    let index = decision
        .evidence
        .iter()
        .position(|value| value.evidence_link_id == evidence_link_id)
        .ok_or(DecisionLifecycleErrorV1::EvidenceNotFound)?;
    decision.evidence.remove(index);
    advance(decision, changed_at)
}

pub fn decide_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    selected_alternative_id: [u8; 16],
    rationale: String,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    validate_text(&rationale, 1, MAX_RATIONALE_CHARS_V1)?;
    if decision.alternatives.len() < 2
        || !decision
            .alternatives
            .iter()
            .any(|value| value.alternative_id == selected_alternative_id)
    {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    let next = next_revision(decision.decision_revision)?;
    for alternative in &mut decision.alternatives {
        alternative.state = if alternative.alternative_id == selected_alternative_id {
            DecisionAlternativeStateV1::Selected
        } else {
            DecisionAlternativeStateV1::Rejected
        };
        alternative.alternative_revision = next_revision(alternative.alternative_revision)?;
        alternative.updated_at_decision_revision = next;
        alternative.updated_at = changed_at;
    }
    decision.state = DecisionStateV1::Decided;
    decision.rationale = rationale;
    decision.selected_alternative_id = Some(selected_alternative_id);
    decision.decision_revision = next;
    decision.updated_at = changed_at;
    Ok(())
}

pub fn supersede_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    replacement_decision_id: [u8; 16],
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_revision_time(decision, expected_revision, changed_at)?;
    if decision.state != DecisionStateV1::Decided
        || !nonzero(&replacement_decision_id)
        || replacement_decision_id == decision.decision_id
    {
        return Err(DecisionLifecycleErrorV1::InvalidStateTransition);
    }
    decision.state = DecisionStateV1::Superseded;
    decision.superseded_by_decision_id = Some(replacement_decision_id);
    advance(decision, changed_at)
}

pub fn cancel_v1(
    decision: &mut DecisionRecordV1,
    expected_revision: u64,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_draft(decision, expected_revision, changed_at)?;
    decision.state = DecisionStateV1::Cancelled;
    advance(decision, changed_at)
}

fn require_draft(
    decision: &DecisionRecordV1,
    expected_revision: u64,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    require_revision_time(decision, expected_revision, changed_at)?;
    (decision.state == DecisionStateV1::Draft)
        .then_some(())
        .ok_or(DecisionLifecycleErrorV1::InvalidStateTransition)
}

fn require_revision_time(
    decision: &DecisionRecordV1,
    expected_revision: u64,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    if decision.decision_revision != expected_revision {
        return Err(DecisionLifecycleErrorV1::InvalidRevision);
    }
    validate_timestamp(changed_at)?;
    if changed_at < decision.updated_at {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn advance(
    decision: &mut DecisionRecordV1,
    changed_at: DecisionTimestampV1,
) -> Result<(), DecisionLifecycleErrorV1> {
    decision.decision_revision = next_revision(decision.decision_revision)?;
    decision.updated_at = changed_at;
    Ok(())
}

fn validate_evidence(value: &DecisionEvidenceLinkV1) -> Result<(), DecisionLifecycleErrorV1> {
    if !nonzero(&value.evidence_link_id)
        || !valid_owner(&value.evidence_owner_id)
        || !nonzero(&value.evidence_record_id)
        || value.evidence_revision == 0
        || !nonzero(&value.evidence_digest)
    {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_timestamp(value: DecisionTimestampV1) -> Result<(), DecisionLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_text(value: &str, min: usize, max: usize) -> Result<(), DecisionLifecycleErrorV1> {
    let count = value.chars().count();
    if count < min || count > max || value.contains('\0') {
        return Err(DecisionLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn next_revision(value: u64) -> Result<u64, DecisionLifecycleErrorV1> {
    value
        .checked_add(1)
        .ok_or(DecisionLifecycleErrorV1::RevisionOverflow)
}

fn derive_id(domain: &[u8], parts: &[&[u8]]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize()[..16].try_into().expect("fixed digest")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(seconds: i64) -> DecisionTimestampV1 {
        DecisionTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }

    fn draft() -> DecisionRecordV1 {
        create_decision_v1(
            "owner-1".to_owned(),
            [1; 16],
            "Choose a storage engine".to_owned(),
            "Which durable store should own the aggregate?".to_owned(),
            timestamp(1),
        )
        .expect("draft")
    }

    #[test]
    fn decide_requires_two_alternatives_and_one_selected() {
        let mut value = draft();
        let first = add_alternative_v1(
            &mut value,
            1,
            [2; 16],
            "PostgreSQL".to_owned(),
            String::new(),
            timestamp(2),
        )
        .expect("first");
        assert_eq!(
            decide_v1(&mut value, 2, first, "Durable".to_owned(), timestamp(3)),
            Err(DecisionLifecycleErrorV1::InvalidInput)
        );
        add_alternative_v1(
            &mut value,
            2,
            [3; 16],
            "SQLite".to_owned(),
            String::new(),
            timestamp(3),
        )
        .expect("second");
        decide_v1(
            &mut value,
            3,
            first,
            "Owner RLS is required".to_owned(),
            timestamp(4),
        )
        .expect("decide");
        assert_eq!(value.state, DecisionStateV1::Decided);
        assert_eq!(value.selected_alternative_id, Some(first));
        assert_eq!(
            value
                .alternatives
                .iter()
                .filter(|item| item.state == DecisionAlternativeStateV1::Selected)
                .count(),
            1
        );
    }

    #[test]
    fn terminal_lifecycle_is_bounded_and_revision_checked() {
        let mut cancelled = draft();
        cancel_v1(&mut cancelled, 1, timestamp(2)).expect("cancel");
        assert_eq!(
            update_decision_v1(&mut cancelled, 2, Some("No".to_owned()), None, timestamp(3)),
            Err(DecisionLifecycleErrorV1::InvalidStateTransition)
        );

        let mut decided = draft();
        let selected = add_alternative_v1(
            &mut decided,
            1,
            [2; 16],
            "A".to_owned(),
            String::new(),
            timestamp(2),
        )
        .expect("first");
        add_alternative_v1(
            &mut decided,
            2,
            [3; 16],
            "B".to_owned(),
            String::new(),
            timestamp(3),
        )
        .expect("second");
        decide_v1(
            &mut decided,
            3,
            selected,
            "Because".to_owned(),
            timestamp(4),
        )
        .expect("decide");
        supersede_v1(&mut decided, 4, [9; 16], timestamp(5)).expect("supersede");
        assert_eq!(decided.state, DecisionStateV1::Superseded);
    }

    #[test]
    fn evidence_is_public_identity_only_and_exactly_unique() {
        let mut value = draft();
        let evidence = DecisionEvidenceLinkV1 {
            evidence_link_id: [4; 16],
            evidence_owner_id: "documents".to_owned(),
            evidence_record_id: [5; 16],
            evidence_revision: 2,
            evidence_digest: [6; 32],
        };
        add_evidence_v1(&mut value, 1, evidence.clone(), timestamp(2)).expect("evidence");
        assert_eq!(
            add_evidence_v1(&mut value, 2, evidence, timestamp(3)),
            Err(DecisionLifecycleErrorV1::EvidenceConflict)
        );
    }

    #[test]
    fn draft_mutations_update_and_remove_exact_children() {
        let mut value = draft();
        update_decision_v1(
            &mut value,
            1,
            Some("Choose the durable owner store".to_owned()),
            None,
            timestamp(2),
        )
        .expect("update Decision");
        let first = add_alternative_v1(
            &mut value,
            2,
            [2; 16],
            "PostgreSQL".to_owned(),
            String::new(),
            timestamp(3),
        )
        .expect("first alternative");
        update_alternative_v1(
            &mut value,
            3,
            first,
            1,
            None,
            Some("FORCE RLS".to_owned()),
            timestamp(4),
        )
        .expect("update alternative");
        let second = add_alternative_v1(
            &mut value,
            4,
            [3; 16],
            "SQLite".to_owned(),
            String::new(),
            timestamp(5),
        )
        .expect("second alternative");
        remove_alternative_v1(&mut value, 5, second, 1, timestamp(6)).expect("remove alternative");
        let evidence = DecisionEvidenceLinkV1 {
            evidence_link_id: [4; 16],
            evidence_owner_id: "documents".to_owned(),
            evidence_record_id: [5; 16],
            evidence_revision: 2,
            evidence_digest: [6; 32],
        };
        add_evidence_v1(&mut value, 6, evidence, timestamp(7)).expect("add evidence");
        remove_evidence_v1(&mut value, 7, [4; 16], timestamp(8)).expect("remove evidence");
        assert_eq!(value.decision_revision, 8);
        assert_eq!(value.alternatives.len(), 1);
        assert_eq!(value.alternatives[0].alternative_revision, 2);
        assert!(value.evidence.is_empty());
    }
}
