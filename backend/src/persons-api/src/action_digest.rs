use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PersonsActionDigestSourceV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub provider_source_contact_public_id: [u8; 16],
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PersonsActionDigestSplitSourceV1 {
    pub source: PersonsActionDigestSourceV1,
    pub expected_source_revision: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsIdentityMatchKindV1 {
    NormalizedEmail,
    NormalizedPhone,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsActionDigestErrorV1 {
    InvalidOwner,
    InvalidId,
    InvalidRevision,
    InvalidSelection,
}

pub fn persons_owner_partition_id_v1(owner: &str) -> Result<[u8; 16], PersonsActionDigestErrorV1> {
    validate_owner(owner)?;
    let mut h = Sha256::new();
    h.update(b"persons-owner-partition-v1");
    part(&mut h, owner.as_bytes());
    part(&mut h, b"persons");
    Ok(h.finalize()[..16].try_into().expect("SHA-256 prefix"))
}

pub fn persons_identity_match_candidate_id_v1(
    owner: &str,
    first: PersonsActionDigestSourceV1,
    second: PersonsActionDigestSourceV1,
    match_kind: PersonsIdentityMatchKindV1,
) -> Result<[u8; 16], PersonsActionDigestErrorV1> {
    validate_owner(owner)?;
    validate_source(first)?;
    validate_source(second)?;
    let (first, second) = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    let mut hash = Sha256::new();
    hash.update(b"makosh.persons.review-candidate-id.v1");
    part(&mut hash, owner.as_bytes());
    update_source(&mut hash, first);
    update_source(&mut hash, second);
    hash.update([match match_kind {
        PersonsIdentityMatchKindV1::NormalizedEmail => 1,
        PersonsIdentityMatchKindV1::NormalizedPhone => 2,
    }]);
    Ok(hash.finalize()[..16].try_into().expect("SHA-256 prefix"))
}

pub fn persons_confirmed_action_command_id_v1(
    decision_id: [u8; 16],
    approved_action_digest: [u8; 32],
) -> Result<[u8; 16], PersonsActionDigestErrorV1> {
    validate_id(decision_id)?;
    if approved_action_digest.iter().all(|byte| *byte == 0) {
        return Err(PersonsActionDigestErrorV1::InvalidId);
    }
    let mut hash = Sha256::new();
    for value in [
        b"makosh.reviewed-person-match-candidate.persons-command.v1".as_slice(),
        decision_id.as_slice(),
        approved_action_digest.as_slice(),
    ] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
    }
    Ok(hash.finalize()[..16].try_into().expect("SHA-256 prefix"))
}

#[allow(clippy::too_many_arguments)]
pub fn persons_attach_source_action_digest_v1(
    owner: &str,
    from: [u8; 16],
    from_revision: u64,
    to: [u8; 16],
    to_revision: u64,
    source: PersonsActionDigestSourceV1,
    source_revision: u64,
) -> Result<[u8; 32], PersonsActionDigestErrorV1> {
    validate_owner(owner)?;
    validate_id(from)?;
    validate_id(to)?;
    validate_source(source)?;
    revision(from_revision)?;
    revision(to_revision)?;
    revision(source_revision)?;
    let mut h = action_hasher("attach-source", owner);
    part(&mut h, &from);
    h.update(from_revision.to_be_bytes());
    part(&mut h, &to);
    h.update(to_revision.to_be_bytes());
    update_source(&mut h, source);
    h.update(source_revision.to_be_bytes());
    Ok(h.finalize().into())
}
pub fn persons_merge_action_digest_v1(
    owner: &str,
    source: [u8; 16],
    source_revision: u64,
    target: [u8; 16],
    target_revision: u64,
) -> Result<[u8; 32], PersonsActionDigestErrorV1> {
    validate_owner(owner)?;
    validate_id(source)?;
    validate_id(target)?;
    revision(source_revision)?;
    revision(target_revision)?;
    let mut h = action_hasher("merge-persons", owner);
    part(&mut h, &source);
    h.update(source_revision.to_be_bytes());
    part(&mut h, &target);
    h.update(target_revision.to_be_bytes());
    Ok(h.finalize().into())
}
#[allow(clippy::too_many_arguments)]
pub fn persons_split_action_digest_v1(
    owner: &str,
    merged: [u8; 16],
    merged_revision: u64,
    target: [u8; 16],
    target_revision: u64,
    sources: &[PersonsActionDigestSplitSourceV1],
    profile_fact_tags: &[u8],
) -> Result<[u8; 32], PersonsActionDigestErrorV1> {
    validate_owner(owner)?;
    validate_id(merged)?;
    validate_id(target)?;
    revision(merged_revision)?;
    revision(target_revision)?;
    if sources.is_empty() && profile_fact_tags.is_empty() {
        return Err(PersonsActionDigestErrorV1::InvalidSelection);
    }
    let mut normalized_sources = sources.to_vec();
    for selected in &normalized_sources {
        validate_source(selected.source)?;
        revision(selected.expected_source_revision)?;
    }
    normalized_sources.sort();
    if normalized_sources
        .windows(2)
        .any(|p| p[0].source == p[1].source)
    {
        return Err(PersonsActionDigestErrorV1::InvalidSelection);
    }
    let mut facts = profile_fact_tags.to_vec();
    facts.sort();
    if facts.iter().any(|v| !(1..=5).contains(v)) || facts.windows(2).any(|p| p[0] == p[1]) {
        return Err(PersonsActionDigestErrorV1::InvalidSelection);
    }
    let mut h = action_hasher("split-person", owner);
    part(&mut h, &merged);
    h.update(merged_revision.to_be_bytes());
    part(&mut h, &target);
    h.update(target_revision.to_be_bytes());
    h.update((normalized_sources.len() as u64).to_be_bytes());
    for selected in normalized_sources {
        update_source(&mut h, selected.source);
        h.update(selected.expected_source_revision.to_be_bytes());
    }
    h.update((facts.len() as u64).to_be_bytes());
    for fact in facts {
        h.update([fact]);
    }
    Ok(h.finalize().into())
}
fn action_hasher(kind: &str, owner: &str) -> Sha256 {
    let mut h = Sha256::new();
    h.update(b"makosh.persons.confirmed-action.v1");
    part(&mut h, kind.as_bytes());
    part(&mut h, owner.as_bytes());
    h
}
fn part(h: &mut Sha256, v: &[u8]) {
    h.update((v.len() as u64).to_be_bytes());
    h.update(v)
}
fn update_source(h: &mut Sha256, v: PersonsActionDigestSourceV1) {
    part(h, &v.integration_public_id);
    part(h, &v.account_public_id);
    part(h, &v.provider_source_contact_public_id)
}
fn validate_owner(v: &str) -> Result<(), PersonsActionDigestErrorV1> {
    if v.is_empty()
        || v.len() > 128
        || !v.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
    {
        Err(PersonsActionDigestErrorV1::InvalidOwner)
    } else {
        Ok(())
    }
}
fn validate_id(v: [u8; 16]) -> Result<(), PersonsActionDigestErrorV1> {
    if v.iter().all(|b| *b == 0) {
        Err(PersonsActionDigestErrorV1::InvalidId)
    } else {
        Ok(())
    }
}
fn validate_source(v: PersonsActionDigestSourceV1) -> Result<(), PersonsActionDigestErrorV1> {
    validate_id(v.integration_public_id)?;
    validate_id(v.account_public_id)?;
    validate_id(v.provider_source_contact_public_id)
}
fn revision(v: u64) -> Result<(), PersonsActionDigestErrorV1> {
    if v == 0 {
        Err(PersonsActionDigestErrorV1::InvalidRevision)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_digest_rejects_empty_and_is_order_independent() {
        let source = PersonsActionDigestSourceV1 {
            integration_public_id: [1; 16],
            account_public_id: [2; 16],
            provider_source_contact_public_id: [3; 16],
        };
        assert!(
            persons_attach_source_action_digest_v1("owner-a", [4; 16], 1, [5; 16], 1, source, 1)
                .is_ok()
        );
        assert_eq!(
            persons_split_action_digest_v1("owner-a", [4; 16], 1, [5; 16], 1, &[], &[]),
            Err(PersonsActionDigestErrorV1::InvalidSelection)
        );
    }
    #[test]
    fn owner_partition_is_canonical_and_owner_bound() {
        assert_eq!(
            persons_owner_partition_id_v1("owner-a"),
            persons_owner_partition_id_v1("owner-a")
        );
        assert_ne!(
            persons_owner_partition_id_v1("owner-a"),
            persons_owner_partition_id_v1("owner-b")
        );
        assert_eq!(
            persons_owner_partition_id_v1("Owner"),
            Err(PersonsActionDigestErrorV1::InvalidOwner)
        );
    }
}
