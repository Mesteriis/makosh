#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-relationships-core";
pub const MAX_PUBLIC_ID_BYTES_V1: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RelationshipParticipantKindV1 {
    Person,
    Organization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipTypeV1 {
    Family,
    Friend,
    Colleague,
    ReportsTo,
    MemberOf,
    Partner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipStateV1 {
    Confirmed,
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipEvidenceStateV1 {
    Active,
    Removed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RelationshipTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RelationshipParticipantV1 {
    pub kind: RelationshipParticipantKindV1,
    pub public_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipEvidenceV1 {
    pub evidence_id: [u8; 16],
    pub source_owner_id: String,
    pub source_record_id: String,
    pub source_revision: u64,
    pub evidence_digest: [u8; 32],
    pub observed_at: RelationshipTimestampV1,
    pub state: RelationshipEvidenceStateV1,
    pub updated_at_relationship_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipRecordV1 {
    pub relationship_id: [u8; 16],
    pub logical_owner_id: String,
    pub source: RelationshipParticipantV1,
    pub target: RelationshipParticipantV1,
    pub relationship_type: RelationshipTypeV1,
    pub state: RelationshipStateV1,
    pub valid_from: RelationshipTimestampV1,
    pub valid_until: Option<RelationshipTimestampV1>,
    pub relationship_revision: u64,
    pub created_at: RelationshipTimestampV1,
    pub updated_at: RelationshipTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipCoreErrorV1 {
    InvalidInput,
    RevisionConflict,
    StateConflict,
    EvidenceConflict,
    RevisionOverflow,
}

#[allow(clippy::too_many_arguments)]
pub fn create_relationship_v1(
    logical_owner_id: String,
    operation_id: [u8; 16],
    mut source: RelationshipParticipantV1,
    mut target: RelationshipParticipantV1,
    relationship_type: RelationshipTypeV1,
    valid_from: RelationshipTimestampV1,
    valid_until: Option<RelationshipTimestampV1>,
    created_at: RelationshipTimestampV1,
) -> Result<RelationshipRecordV1, RelationshipCoreErrorV1> {
    if !valid_owner(&logical_owner_id)
        || !nonzero(&operation_id)
        || !valid_participant(&source)
        || !valid_participant(&target)
        || source == target
        || !valid_interval(valid_from, valid_until)
        || !valid_timestamp(created_at)
        || created_at < valid_from
    {
        return Err(RelationshipCoreErrorV1::InvalidInput);
    }
    if is_symmetric(relationship_type) && target < source {
        std::mem::swap(&mut source, &mut target);
    }
    let source_kind = [participant_kind_code(source.kind)];
    let target_kind = [participant_kind_code(target.kind)];
    let relationship_type_code = [relationship_type_code(relationship_type)];
    Ok(RelationshipRecordV1 {
        relationship_id: derive_id(
            b"makosh.relationships.relationship.v1\0",
            &[
                logical_owner_id.as_bytes(),
                &source_kind,
                &source.public_id,
                &target_kind,
                &target.public_id,
                &relationship_type_code,
            ],
        ),
        logical_owner_id,
        source,
        target,
        relationship_type,
        state: RelationshipStateV1::Confirmed,
        valid_from,
        valid_until,
        relationship_revision: 1,
        created_at,
        updated_at: created_at,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_relationship_with_evidence_v1(
    logical_owner_id: String,
    operation_id: [u8; 16],
    source: RelationshipParticipantV1,
    target: RelationshipParticipantV1,
    relationship_type: RelationshipTypeV1,
    valid_from: RelationshipTimestampV1,
    valid_until: Option<RelationshipTimestampV1>,
    evidence_source_owner_id: String,
    evidence_source_record_id: String,
    evidence_source_revision: u64,
    evidence_digest: [u8; 32],
    evidence_observed_at: RelationshipTimestampV1,
    created_at: RelationshipTimestampV1,
) -> Result<(RelationshipRecordV1, RelationshipEvidenceV1), RelationshipCoreErrorV1> {
    let relationship = create_relationship_v1(
        logical_owner_id,
        operation_id,
        source,
        target,
        relationship_type,
        valid_from,
        valid_until,
        created_at,
    )?;
    if !valid_public_id(&evidence_source_owner_id)
        || !valid_public_id(&evidence_source_record_id)
        || evidence_source_revision == 0
        || !nonzero(&evidence_digest)
        || !valid_timestamp(evidence_observed_at)
        || created_at < evidence_observed_at
    {
        return Err(RelationshipCoreErrorV1::InvalidInput);
    }
    let evidence_id = derive_evidence_id_v1(
        &relationship.relationship_id,
        &evidence_source_owner_id,
        &evidence_source_record_id,
    )?;
    let evidence = RelationshipEvidenceV1 {
        evidence_id,
        source_owner_id: evidence_source_owner_id,
        source_record_id: evidence_source_record_id,
        source_revision: evidence_source_revision,
        evidence_digest,
        observed_at: evidence_observed_at,
        state: RelationshipEvidenceStateV1::Active,
        updated_at_relationship_revision: relationship.relationship_revision,
    };
    Ok((relationship, evidence))
}

pub fn update_validity_v1(
    value: &mut RelationshipRecordV1,
    expected_revision: u64,
    valid_from: RelationshipTimestampV1,
    valid_until: Option<RelationshipTimestampV1>,
    changed_at: RelationshipTimestampV1,
) -> Result<(), RelationshipCoreErrorV1> {
    require_revision_time(value, expected_revision, changed_at)?;
    if !valid_interval(valid_from, valid_until) || changed_at < valid_from {
        return Err(RelationshipCoreErrorV1::InvalidInput);
    }
    value.valid_from = valid_from;
    value.valid_until = valid_until;
    advance(value, changed_at)
}

pub fn end_relationship_v1(
    value: &mut RelationshipRecordV1,
    expected_revision: u64,
    valid_until: RelationshipTimestampV1,
    changed_at: RelationshipTimestampV1,
) -> Result<(), RelationshipCoreErrorV1> {
    require_revision_time(value, expected_revision, changed_at)?;
    if value.state != RelationshipStateV1::Confirmed
        || valid_until <= value.valid_from
        || changed_at < valid_until
    {
        return Err(RelationshipCoreErrorV1::StateConflict);
    }
    value.state = RelationshipStateV1::Ended;
    value.valid_until = Some(valid_until);
    advance(value, changed_at)
}

pub fn reactivate_relationship_v1(
    value: &mut RelationshipRecordV1,
    expected_revision: u64,
    valid_from: RelationshipTimestampV1,
    valid_until: Option<RelationshipTimestampV1>,
    changed_at: RelationshipTimestampV1,
) -> Result<(), RelationshipCoreErrorV1> {
    require_revision_time(value, expected_revision, changed_at)?;
    if value.state != RelationshipStateV1::Ended
        || !valid_interval(valid_from, valid_until)
        || changed_at < valid_from
    {
        return Err(RelationshipCoreErrorV1::StateConflict);
    }
    value.state = RelationshipStateV1::Confirmed;
    value.valid_from = valid_from;
    value.valid_until = valid_until;
    advance(value, changed_at)
}

#[allow(clippy::too_many_arguments)]
pub fn add_evidence_v1(
    value: &mut RelationshipRecordV1,
    existing: &[RelationshipEvidenceV1],
    expected_revision: u64,
    source_owner_id: String,
    source_record_id: String,
    source_revision: u64,
    evidence_digest: [u8; 32],
    observed_at: RelationshipTimestampV1,
    changed_at: RelationshipTimestampV1,
) -> Result<RelationshipEvidenceV1, RelationshipCoreErrorV1> {
    require_revision_time(value, expected_revision, changed_at)?;
    if !valid_public_id(&source_owner_id)
        || !valid_public_id(&source_record_id)
        || source_revision == 0
        || !nonzero(&evidence_digest)
        || !valid_timestamp(observed_at)
        || changed_at < observed_at
    {
        return Err(RelationshipCoreErrorV1::InvalidInput);
    }
    let evidence_id =
        derive_evidence_id_v1(&value.relationship_id, &source_owner_id, &source_record_id)?;
    if existing.iter().any(|item| item.evidence_id == evidence_id) {
        return Err(RelationshipCoreErrorV1::EvidenceConflict);
    }
    let next = value
        .relationship_revision
        .checked_add(1)
        .ok_or(RelationshipCoreErrorV1::RevisionOverflow)?;
    value.relationship_revision = next;
    value.updated_at = changed_at;
    Ok(RelationshipEvidenceV1 {
        evidence_id,
        source_owner_id,
        source_record_id,
        source_revision,
        evidence_digest,
        observed_at,
        state: RelationshipEvidenceStateV1::Active,
        updated_at_relationship_revision: next,
    })
}

pub fn remove_evidence_v1(
    value: &mut RelationshipRecordV1,
    evidence: &mut RelationshipEvidenceV1,
    expected_revision: u64,
    changed_at: RelationshipTimestampV1,
) -> Result<(), RelationshipCoreErrorV1> {
    require_revision_time(value, expected_revision, changed_at)?;
    if evidence.state != RelationshipEvidenceStateV1::Active {
        return Err(RelationshipCoreErrorV1::EvidenceConflict);
    }
    let next = value
        .relationship_revision
        .checked_add(1)
        .ok_or(RelationshipCoreErrorV1::RevisionOverflow)?;
    value.relationship_revision = next;
    value.updated_at = changed_at;
    evidence.state = RelationshipEvidenceStateV1::Removed;
    evidence.updated_at_relationship_revision = next;
    Ok(())
}

pub fn derive_evidence_id_v1(
    relationship_id: &[u8; 16],
    source_owner_id: &str,
    source_record_id: &str,
) -> Result<[u8; 16], RelationshipCoreErrorV1> {
    if !nonzero(relationship_id)
        || !valid_public_id(source_owner_id)
        || !valid_public_id(source_record_id)
    {
        return Err(RelationshipCoreErrorV1::InvalidInput);
    }
    Ok(derive_id(
        b"makosh.relationships.evidence.v1\0",
        &[
            relationship_id,
            source_owner_id.as_bytes(),
            source_record_id.as_bytes(),
        ],
    ))
}

fn require_revision_time(
    value: &RelationshipRecordV1,
    expected_revision: u64,
    changed_at: RelationshipTimestampV1,
) -> Result<(), RelationshipCoreErrorV1> {
    if value.relationship_revision != expected_revision || expected_revision == 0 {
        return Err(RelationshipCoreErrorV1::RevisionConflict);
    }
    if !valid_timestamp(changed_at) || changed_at < value.updated_at {
        return Err(RelationshipCoreErrorV1::InvalidInput);
    }
    Ok(())
}

fn advance(
    value: &mut RelationshipRecordV1,
    changed_at: RelationshipTimestampV1,
) -> Result<(), RelationshipCoreErrorV1> {
    value.relationship_revision = value
        .relationship_revision
        .checked_add(1)
        .ok_or(RelationshipCoreErrorV1::RevisionOverflow)?;
    value.updated_at = changed_at;
    Ok(())
}

fn is_symmetric(value: RelationshipTypeV1) -> bool {
    matches!(
        value,
        RelationshipTypeV1::Family
            | RelationshipTypeV1::Friend
            | RelationshipTypeV1::Colleague
            | RelationshipTypeV1::Partner
    )
}
fn participant_kind_code(value: RelationshipParticipantKindV1) -> u8 {
    match value {
        RelationshipParticipantKindV1::Person => 1,
        RelationshipParticipantKindV1::Organization => 2,
    }
}
fn relationship_type_code(value: RelationshipTypeV1) -> u8 {
    match value {
        RelationshipTypeV1::Family => 1,
        RelationshipTypeV1::Friend => 2,
        RelationshipTypeV1::Colleague => 3,
        RelationshipTypeV1::ReportsTo => 4,
        RelationshipTypeV1::MemberOf => 5,
        RelationshipTypeV1::Partner => 6,
    }
}
fn valid_interval(from: RelationshipTimestampV1, until: Option<RelationshipTimestampV1>) -> bool {
    valid_timestamp(from) && until.is_none_or(|until| valid_timestamp(until) && until > from)
}
fn valid_timestamp(value: RelationshipTimestampV1) -> bool {
    value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos)
}
fn valid_participant(value: &RelationshipParticipantV1) -> bool {
    nonzero(&value.public_id)
}
fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}
fn valid_public_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PUBLIC_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/')
        })
}
fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn derive_id(domain: &[u8], chunks: &[&[u8]]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for chunk in chunks {
        hash.update((chunk.len() as u64).to_be_bytes());
        hash.update(chunk);
    }
    hash.finalize()[..16].try_into().expect("digest")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(seconds: i64) -> RelationshipTimestampV1 {
        RelationshipTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }
    fn participant(id: u8) -> RelationshipParticipantV1 {
        RelationshipParticipantV1 {
            kind: RelationshipParticipantKindV1::Person,
            public_id: [id; 16],
        }
    }

    #[test]
    fn confirmed_temporal_lifecycle_is_checked_and_reactivatable() {
        let mut value = create_relationship_v1(
            "owner-1".to_owned(),
            [1; 16],
            participant(3),
            participant(2),
            RelationshipTypeV1::Friend,
            time(10),
            None,
            time(10),
        )
        .expect("create");
        assert_eq!(value.source.public_id, [2; 16]);
        assert_eq!(value.state, RelationshipStateV1::Confirmed);
        end_relationship_v1(&mut value, 1, time(20), time(20)).expect("end");
        assert_eq!(value.state, RelationshipStateV1::Ended);
        reactivate_relationship_v1(&mut value, 2, time(30), None, time(30)).expect("reactivate");
        assert_eq!(value.relationship_revision, 3);
        assert_eq!(value.state, RelationshipStateV1::Confirmed);
    }

    #[test]
    fn evidence_identity_is_stable_and_revision_bound() {
        let mut value = create_relationship_v1(
            "owner-1".to_owned(),
            [1; 16],
            participant(1),
            participant(2),
            RelationshipTypeV1::ReportsTo,
            time(10),
            None,
            time(10),
        )
        .expect("create");
        let evidence = add_evidence_v1(
            &mut value,
            &[],
            1,
            "communications".to_owned(),
            "record-1".to_owned(),
            1,
            [9; 32],
            time(9),
            time(11),
        )
        .expect("evidence");
        assert_eq!(evidence.updated_at_relationship_revision, 2);
        assert_eq!(
            evidence.evidence_id,
            derive_evidence_id_v1(&value.relationship_id, "communications", "record-1")
                .expect("id")
        );
        assert_eq!(
            add_evidence_v1(
                &mut value,
                std::slice::from_ref(&evidence),
                2,
                "communications".to_owned(),
                "record-1".to_owned(),
                2,
                [8; 32],
                time(10),
                time(12),
            ),
            Err(RelationshipCoreErrorV1::EvidenceConflict)
        );
    }

    #[test]
    fn invalid_temporal_and_self_relationships_fail_closed() {
        assert_eq!(
            create_relationship_v1(
                "owner-1".to_owned(),
                [1; 16],
                participant(1),
                participant(1),
                RelationshipTypeV1::Family,
                time(10),
                None,
                time(10),
            ),
            Err(RelationshipCoreErrorV1::InvalidInput)
        );
        assert_eq!(
            create_relationship_v1(
                "owner-1".to_owned(),
                [1; 16],
                participant(1),
                participant(2),
                RelationshipTypeV1::Family,
                time(10),
                Some(time(10)),
                time(10),
            ),
            Err(RelationshipCoreErrorV1::InvalidInput)
        );
    }

    #[test]
    fn aggregate_identity_and_initial_evidence_are_stable() {
        let (left, evidence) = create_relationship_with_evidence_v1(
            "owner-1".to_owned(),
            [1; 16],
            participant(2),
            participant(1),
            RelationshipTypeV1::Friend,
            time(10),
            None,
            "persons".to_owned(),
            "record-1".to_owned(),
            1,
            [8; 32],
            time(9),
            time(10),
        )
        .expect("create with evidence");
        let right = create_relationship_v1(
            "owner-1".to_owned(),
            [9; 16],
            participant(1),
            participant(2),
            RelationshipTypeV1::Friend,
            time(10),
            None,
            time(10),
        )
        .expect("same aggregate");
        assert_eq!(left.relationship_id, right.relationship_id);
        assert_eq!(left.relationship_revision, 1);
        assert_eq!(evidence.updated_at_relationship_revision, 1);
    }
}
