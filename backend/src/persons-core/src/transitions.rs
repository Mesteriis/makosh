use std::collections::{BTreeMap, BTreeSet};

use makosh_persons_api::{
    PersonsActionDigestSourceV1, PersonsActionDigestSplitSourceV1,
    persons_attach_source_action_digest_v1, persons_merge_action_digest_v1,
    persons_split_action_digest_v1,
};
use sha2::{Digest, Sha256};

use crate::{
    AttachSourceActionV1, ConfirmedActionOutcomeV1, ConfirmedActionStatusV1, DecisionProvenanceV1,
    DecisionReceiptV1, DetachSourceActionV1, DigestV1, IdentityMatchKindV1, LineageChangeKindV1,
    LineageRecordV1, MAX_EMAILS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_PHONES_V1,
    MAX_PROFILE_TEXT_CHARS_V1, MAX_REVIEW_CANDIDATES_PER_COMMAND_V1, ManualPersonDraftV1,
    MergePersonsActionV1, OwnerProfileV1, PersonIdV1, PersonLifecycleV1, PersonRevisionV1,
    PersonV1, PersonsStateV1, PersonsTransitionErrorV1, PublicIdV1, RemovedSourceV1,
    ReviewCandidateV1, SourceClaimsV1, SourceLinkKeyV1, SourceLinkV1, SourceObservationOutcomeV1,
    SourceObservationV1, SourceProvenanceV1, SourceRemovalOutcomeV1, SplitPersonActionV1,
    SplitProfileFactKindV1, SplitSourceSelectionV1, TimestampV1, normalize_email_v1,
    normalize_phone_v1,
};

pub fn create_manual_person_v1(
    state: &mut PersonsStateV1,
    draft: ManualPersonDraftV1,
) -> Result<PersonIdV1, PersonsTransitionErrorV1> {
    validate_person_id(draft.person_id)?;
    validate_owner(&draft.logical_owner_id)?;
    validate_timestamp(draft.created_at)?;
    let owner_profile = normalize_owner_profile(draft.owner_profile)?;
    if owner_profile.is_empty() {
        return Err(PersonsTransitionErrorV1::InvalidProfile);
    }
    if state.persons.contains_key(&draft.person_id) {
        return Err(PersonsTransitionErrorV1::PersonAlreadyExists);
    }
    state.persons.insert(
        draft.person_id,
        PersonV1 {
            person_id: draft.person_id,
            logical_owner_id: draft.logical_owner_id,
            lifecycle: PersonLifecycleV1::Active,
            revision: 1,
            owner_profile: Some(owner_profile),
            source_links: BTreeMap::new(),
            merged_into: None,
            created_at: draft.created_at,
            updated_at: draft.created_at,
        },
    );
    Ok(draft.person_id)
}

pub fn update_owner_profile_v1(
    state: &mut PersonsStateV1,
    logical_owner_id: &str,
    person_id: PersonIdV1,
    expected_revision: u64,
    profile: OwnerProfileV1,
    updated_at: TimestampV1,
) -> Result<(), PersonsTransitionErrorV1> {
    validate_owner(logical_owner_id)?;
    validate_timestamp(updated_at)?;
    let profile = normalize_owner_profile(profile)?;
    if profile.is_empty() {
        return Err(PersonsTransitionErrorV1::InvalidProfile);
    }
    let person = person_for_owner_mut(state, person_id, logical_owner_id)?;
    if person.lifecycle == PersonLifecycleV1::Merged {
        return Err(PersonsTransitionErrorV1::PersonMerged);
    }
    require_person_revision(person, expected_revision)?;
    if timestamp_before(updated_at, person.updated_at) {
        return Err(PersonsTransitionErrorV1::InvalidTimestamp);
    }
    person.owner_profile = Some(profile);
    person.lifecycle = PersonLifecycleV1::Active;
    person.revision += 1;
    person.updated_at = updated_at;
    Ok(())
}

pub fn observe_source_v1(
    state: &mut PersonsStateV1,
    observation: SourceObservationV1,
) -> Result<SourceObservationOutcomeV1, PersonsTransitionErrorV1> {
    let observation = normalize_observation(observation)?;
    if let Some(person_id) = state.source_owners.get(&observation.key).copied() {
        let owner_person = person_for_owner(state, person_id, &observation.logical_owner_id)?;
        let current = owner_person
            .source_links
            .get(&observation.key)
            .ok_or(PersonsTransitionErrorV1::SourceOwnerConflict)?;
        compare_source_revision(current.provenance, observation.provenance)?;
        if current.provenance.revision == observation.provenance.revision {
            return Ok(SourceObservationOutcomeV1::Unchanged { person_id });
        }
        if timestamp_before(observation.provenance.observed_at, owner_person.updated_at) {
            return Err(PersonsTransitionErrorV1::InvalidTimestamp);
        }
        let review_candidates = review_candidates_for(
            state,
            &observation.logical_owner_id,
            person_id,
            observation.key,
            &observation.claims,
            observation.provenance.observed_at,
        )?;
        let person = state
            .persons
            .get_mut(&person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        let link = person
            .source_links
            .get_mut(&observation.key)
            .ok_or(PersonsTransitionErrorV1::SourceNotFound)?;
        link.claims = observation.claims;
        link.provenance = observation.provenance;
        person.revision += 1;
        person.updated_at = observation.provenance.observed_at;
        return Ok(SourceObservationOutcomeV1::Updated {
            person_id,
            review_candidates,
        });
    }

    if let Some(removed) = state.removed_sources.get(&observation.key) {
        if removed.logical_owner_id != observation.logical_owner_id {
            return Err(PersonsTransitionErrorV1::OwnerMismatch);
        }
        compare_source_revision(removed.provenance, observation.provenance)?;
        if removed.provenance.revision == observation.provenance.revision {
            return Err(PersonsTransitionErrorV1::SourceNotFound);
        }
    }

    let person_id = derive_source_person_id(&observation.logical_owner_id, observation.key)?;
    let review_candidates = review_candidates_for(
        state,
        &observation.logical_owner_id,
        person_id,
        observation.key,
        &observation.claims,
        observation.provenance.observed_at,
    )?;
    let link = SourceLinkV1 {
        key: observation.key,
        claims: observation.claims,
        provenance: observation.provenance,
        last_decision: None,
    };
    match state.persons.get_mut(&person_id) {
        Some(person)
            if person.logical_owner_id == observation.logical_owner_id
                && person.lifecycle == PersonLifecycleV1::Archived
                && person.source_links.is_empty() =>
        {
            if timestamp_before(observation.provenance.observed_at, person.updated_at) {
                return Err(PersonsTransitionErrorV1::InvalidTimestamp);
            }
            person.source_links.insert(observation.key, link);
            person.lifecycle = lifecycle_for_unconfirmed(person);
            person.revision += 1;
            person.updated_at = observation.provenance.observed_at;
        }
        Some(_) => return Err(PersonsTransitionErrorV1::PersonAlreadyExists),
        None => {
            state.persons.insert(
                person_id,
                PersonV1 {
                    person_id,
                    logical_owner_id: observation.logical_owner_id,
                    lifecycle: PersonLifecycleV1::Provisional,
                    revision: 1,
                    owner_profile: None,
                    source_links: BTreeMap::from([(observation.key, link)]),
                    merged_into: None,
                    created_at: observation.provenance.observed_at,
                    updated_at: observation.provenance.observed_at,
                },
            );
        }
    }
    state.source_owners.insert(observation.key, person_id);
    state.removed_sources.remove(&observation.key);
    Ok(SourceObservationOutcomeV1::Created {
        person_id,
        review_candidates,
    })
}

pub fn remove_source_v1(
    state: &mut PersonsStateV1,
    logical_owner_id: &str,
    key: SourceLinkKeyV1,
    provenance: SourceProvenanceV1,
) -> Result<SourceRemovalOutcomeV1, PersonsTransitionErrorV1> {
    validate_owner(logical_owner_id)?;
    validate_source_key(key)?;
    validate_source_provenance(provenance)?;
    let Some(person_id) = state.source_owners.get(&key).copied() else {
        return match state.removed_sources.get(&key).cloned() {
            Some(previous) => {
                if previous.logical_owner_id != logical_owner_id {
                    return Err(PersonsTransitionErrorV1::OwnerMismatch);
                }
                compare_source_revision(previous.provenance, provenance)?;
                if previous.provenance.revision < provenance.revision {
                    state.removed_sources.insert(
                        key,
                        RemovedSourceV1 {
                            logical_owner_id: logical_owner_id.to_owned(),
                            provenance,
                        },
                    );
                }
                Ok(SourceRemovalOutcomeV1 {
                    person_id: None,
                    archived: false,
                })
            }
            None => Err(PersonsTransitionErrorV1::SourceNotFound),
        };
    };
    let person = person_for_owner(state, person_id, logical_owner_id)?;
    let current = person
        .source_links
        .get(&key)
        .ok_or(PersonsTransitionErrorV1::SourceOwnerConflict)?;
    if provenance.revision <= current.provenance.revision {
        return if provenance.revision < current.provenance.revision {
            Err(PersonsTransitionErrorV1::StaleSourceRevision)
        } else {
            Err(PersonsTransitionErrorV1::SourceRevisionConflict)
        };
    }
    if timestamp_before(provenance.observed_at, person.updated_at) {
        return Err(PersonsTransitionErrorV1::InvalidTimestamp);
    }
    let person = state
        .persons
        .get_mut(&person_id)
        .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
    person.source_links.remove(&key);
    person.revision += 1;
    person.updated_at = provenance.observed_at;
    person.lifecycle = lifecycle_after_source_loss(person);
    let archived = person.lifecycle == PersonLifecycleV1::Archived;
    state.source_owners.remove(&key);
    state.removed_sources.insert(
        key,
        RemovedSourceV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            provenance,
        },
    );
    Ok(SourceRemovalOutcomeV1 {
        person_id: Some(person_id),
        archived,
    })
}

pub fn attach_source_action_digest_v1(
    action: &AttachSourceActionV1,
) -> Result<DigestV1, PersonsTransitionErrorV1> {
    validate_owner(&action.logical_owner_id)?;
    validate_person_id(action.from_person_id)?;
    validate_person_id(action.to_person_id)?;
    validate_source_key(action.source)?;
    require_revision_value(action.expected_from_person_revision)?;
    require_revision_value(action.expected_to_person_revision)?;
    require_revision_value(action.expected_source_revision)?;
    persons_attach_source_action_digest_v1(
        &action.logical_owner_id,
        action.from_person_id.0,
        action.expected_from_person_revision,
        action.to_person_id.0,
        action.expected_to_person_revision,
        api_source(action.source),
        action.expected_source_revision,
    )
    .map(DigestV1)
    .map_err(|_| PersonsTransitionErrorV1::InvalidRevision)
}

pub fn detach_source_action_digest_v1(
    action: &DetachSourceActionV1,
) -> Result<DigestV1, PersonsTransitionErrorV1> {
    validate_owner(&action.logical_owner_id)?;
    validate_person_id(action.person_id)?;
    validate_source_key(action.source)?;
    require_revision_value(action.expected_person_revision)?;
    require_revision_value(action.expected_source_revision)?;
    let mut hasher = action_hasher("detach-source", &action.logical_owner_id);
    update_person_id(&mut hasher, action.person_id);
    update_revision(&mut hasher, action.expected_person_revision);
    update_source_key(&mut hasher, action.source);
    update_revision(&mut hasher, action.expected_source_revision);
    update_revision(&mut hasher, action.expected_detached_person_revision);
    Ok(DigestV1(hasher.finalize().into()))
}

pub fn merge_persons_action_digest_v1(
    action: &MergePersonsActionV1,
) -> Result<DigestV1, PersonsTransitionErrorV1> {
    validate_owner(&action.logical_owner_id)?;
    validate_person_id(action.source_person_id)?;
    validate_person_id(action.target_person_id)?;
    require_revision_value(action.expected_source_person_revision)?;
    require_revision_value(action.expected_target_person_revision)?;
    persons_merge_action_digest_v1(
        &action.logical_owner_id,
        action.source_person_id.0,
        action.expected_source_person_revision,
        action.target_person_id.0,
        action.expected_target_person_revision,
    )
    .map(DigestV1)
    .map_err(|_| PersonsTransitionErrorV1::InvalidRevision)
}

pub fn split_person_action_digest_v1(
    action: &SplitPersonActionV1,
) -> Result<DigestV1, PersonsTransitionErrorV1> {
    validate_owner(&action.logical_owner_id)?;
    validate_person_id(action.merged_person_id)?;
    validate_person_id(action.target_person_id)?;
    require_revision_value(action.expected_merged_person_revision)?;
    require_revision_value(action.expected_target_person_revision)?;
    let (sources, profile_facts) = normalized_split_selection(action)?;
    let sources = sources
        .into_iter()
        .map(|value| PersonsActionDigestSplitSourceV1 {
            source: api_source(value.source),
            expected_source_revision: value.expected_source_revision,
        })
        .collect::<Vec<_>>();
    let facts = profile_facts
        .into_iter()
        .map(profile_fact_tag)
        .collect::<Vec<_>>();
    persons_split_action_digest_v1(
        &action.logical_owner_id,
        action.merged_person_id.0,
        action.expected_merged_person_revision,
        action.target_person_id.0,
        action.expected_target_person_revision,
        &sources,
        &facts,
    )
    .map(DigestV1)
    .map_err(|_| PersonsTransitionErrorV1::InvalidRevision)
}

pub fn attach_source_v1(
    state: &mut PersonsStateV1,
    action: AttachSourceActionV1,
    decision: DecisionProvenanceV1,
) -> Result<ConfirmedActionOutcomeV1, PersonsTransitionErrorV1> {
    let digest = attach_source_action_digest_v1(&action)?;
    person_for_owner(state, action.from_person_id, &action.logical_owner_id)?;
    person_for_owner(state, action.to_person_id, &action.logical_owner_id)?;
    if let Some(replay) = validate_decision_and_replay(state, &decision, digest)? {
        return Ok(replay);
    }
    if action.from_person_id == action.to_person_id {
        return Err(PersonsTransitionErrorV1::SamePerson);
    }
    if state.source_owners.get(&action.source).copied() != Some(action.from_person_id) {
        return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
    }
    let source_person = person_for_owner(state, action.from_person_id, &action.logical_owner_id)?;
    let target_person = person_for_owner(state, action.to_person_id, &action.logical_owner_id)?;
    ensure_not_merged(source_person)?;
    ensure_not_merged(target_person)?;
    require_person_revision(source_person, action.expected_from_person_revision)?;
    require_person_revision(target_person, action.expected_to_person_revision)?;
    require_decision_timestamp(&decision, [source_person, target_person])?;
    let source_link = source_person
        .source_links
        .get(&action.source)
        .ok_or(PersonsTransitionErrorV1::SourceNotFound)?;
    require_source_revision(source_link, action.expected_source_revision)?;
    if target_person.source_links.contains_key(&action.source) {
        return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
    }

    let mut link = state
        .persons
        .get_mut(&action.from_person_id)
        .and_then(|person| person.source_links.remove(&action.source))
        .ok_or(PersonsTransitionErrorV1::SourceNotFound)?;
    link.last_decision = Some(decision.clone());
    {
        let source = state
            .persons
            .get_mut(&action.from_person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        source.revision += 1;
        source.updated_at = decision.decided_at;
        source.lifecycle = lifecycle_after_source_loss(source);
    }
    {
        let target = state
            .persons
            .get_mut(&action.to_person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        target.source_links.insert(action.source, link);
        target.revision += 1;
        target.updated_at = decision.decided_at;
        target.lifecycle = PersonLifecycleV1::Active;
    }
    state
        .source_owners
        .insert(action.source, action.to_person_id);
    Ok(record_decision(
        state,
        &action.logical_owner_id,
        decision,
        digest,
        vec![
            PersonRevisionV1 {
                person_id: action.from_person_id,
                revision: action.expected_from_person_revision + 1,
            },
            PersonRevisionV1 {
                person_id: action.to_person_id,
                revision: action.expected_to_person_revision + 1,
            },
        ],
    ))
}

pub fn detach_source_v1(
    state: &mut PersonsStateV1,
    action: DetachSourceActionV1,
    decision: DecisionProvenanceV1,
) -> Result<ConfirmedActionOutcomeV1, PersonsTransitionErrorV1> {
    let digest = detach_source_action_digest_v1(&action)?;
    person_for_owner(state, action.person_id, &action.logical_owner_id)?;
    if let Some(replay) = validate_decision_and_replay(state, &decision, digest)? {
        return Ok(replay);
    }
    if state.source_owners.get(&action.source).copied() != Some(action.person_id) {
        return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
    }
    let source_person = person_for_owner(state, action.person_id, &action.logical_owner_id)?;
    ensure_not_merged(source_person)?;
    require_person_revision(source_person, action.expected_person_revision)?;
    let source_link = source_person
        .source_links
        .get(&action.source)
        .ok_or(PersonsTransitionErrorV1::SourceNotFound)?;
    require_source_revision(source_link, action.expected_source_revision)?;
    let detached_person_id = derive_source_person_id(&action.logical_owner_id, action.source)?;
    if detached_person_id == action.person_id {
        return Err(PersonsTransitionErrorV1::SamePerson);
    }
    let detached_person = state.persons.get(&detached_person_id);
    match detached_person {
        Some(person) => {
            if person.logical_owner_id != action.logical_owner_id {
                return Err(PersonsTransitionErrorV1::OwnerMismatch);
            }
            if person.lifecycle == PersonLifecycleV1::Merged
                || person.merged_into.is_some()
                || !person.source_links.is_empty()
            {
                return Err(PersonsTransitionErrorV1::LineageConflict);
            }
            require_person_revision(person, action.expected_detached_person_revision)?;
            require_decision_timestamp(&decision, [source_person, person])?;
        }
        None => {
            if action.expected_detached_person_revision != 0 {
                return Err(PersonsTransitionErrorV1::ExpectedRevisionConflict);
            }
            require_decision_timestamp(&decision, [source_person])?;
        }
    }

    let mut link = state
        .persons
        .get_mut(&action.person_id)
        .and_then(|person| person.source_links.remove(&action.source))
        .ok_or(PersonsTransitionErrorV1::SourceNotFound)?;
    link.last_decision = Some(decision.clone());
    {
        let source = state
            .persons
            .get_mut(&action.person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        source.revision += 1;
        source.updated_at = decision.decided_at;
        source.lifecycle = lifecycle_after_source_loss(source);
    }
    match state.persons.get_mut(&detached_person_id) {
        Some(detached) => {
            detached.source_links.insert(action.source, link);
            detached.lifecycle = lifecycle_for_unconfirmed(detached);
            detached.revision += 1;
            detached.updated_at = decision.decided_at;
        }
        None => {
            state.persons.insert(
                detached_person_id,
                PersonV1 {
                    person_id: detached_person_id,
                    logical_owner_id: action.logical_owner_id.clone(),
                    lifecycle: PersonLifecycleV1::Provisional,
                    revision: 1,
                    owner_profile: None,
                    source_links: BTreeMap::from([(action.source, link)]),
                    merged_into: None,
                    created_at: decision.decided_at,
                    updated_at: decision.decided_at,
                },
            );
        }
    }
    state
        .source_owners
        .insert(action.source, detached_person_id);
    Ok(record_decision(
        state,
        &action.logical_owner_id,
        decision,
        digest,
        vec![
            PersonRevisionV1 {
                person_id: action.person_id,
                revision: action.expected_person_revision + 1,
            },
            PersonRevisionV1 {
                person_id: detached_person_id,
                revision: action.expected_detached_person_revision + 1,
            },
        ],
    ))
}

pub fn merge_persons_v1(
    state: &mut PersonsStateV1,
    action: MergePersonsActionV1,
    decision: DecisionProvenanceV1,
) -> Result<ConfirmedActionOutcomeV1, PersonsTransitionErrorV1> {
    let digest = merge_persons_action_digest_v1(&action)?;
    person_for_owner(state, action.source_person_id, &action.logical_owner_id)?;
    person_for_owner(state, action.target_person_id, &action.logical_owner_id)?;
    if let Some(replay) = validate_decision_and_replay(state, &decision, digest)? {
        return Ok(replay);
    }
    if action.source_person_id == action.target_person_id {
        return Err(PersonsTransitionErrorV1::SamePerson);
    }
    let source = person_for_owner(state, action.source_person_id, &action.logical_owner_id)?;
    let target = person_for_owner(state, action.target_person_id, &action.logical_owner_id)?;
    ensure_not_merged(source)?;
    ensure_not_merged(target)?;
    require_person_revision(source, action.expected_source_person_revision)?;
    require_person_revision(target, action.expected_target_person_revision)?;
    require_decision_timestamp(&decision, [source, target])?;
    if source
        .source_links
        .keys()
        .any(|key| target.source_links.contains_key(key))
    {
        return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
    }
    let source_snapshot = source.clone();
    let moved_sources: Vec<_> = source_snapshot.source_links.keys().copied().collect();
    let moved_links = {
        let source = state
            .persons
            .get_mut(&action.source_person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        let links = std::mem::take(&mut source.source_links);
        source.lifecycle = PersonLifecycleV1::Merged;
        source.merged_into = Some(action.target_person_id);
        source.revision += 1;
        source.updated_at = decision.decided_at;
        links
    };
    {
        let target = state
            .persons
            .get_mut(&action.target_person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        target.source_links.extend(moved_links);
        target.lifecycle = PersonLifecycleV1::Active;
        target.revision += 1;
        target.updated_at = decision.decided_at;
    }
    for key in &moved_sources {
        state.source_owners.insert(*key, action.target_person_id);
    }
    state.lineage.push(LineageRecordV1 {
        change_kind: LineageChangeKindV1::Merge,
        source_person_id: action.source_person_id,
        target_person_id: action.target_person_id,
        moved_sources,
        preserved_source_profile: source_snapshot.owner_profile,
        profile_fact_selection: Vec::new(),
        decision: decision.clone(),
    });
    Ok(record_decision(
        state,
        &action.logical_owner_id,
        decision,
        digest,
        vec![
            PersonRevisionV1 {
                person_id: action.source_person_id,
                revision: action.expected_source_person_revision + 1,
            },
            PersonRevisionV1 {
                person_id: action.target_person_id,
                revision: action.expected_target_person_revision + 1,
            },
        ],
    ))
}

pub fn split_person_v1(
    state: &mut PersonsStateV1,
    action: SplitPersonActionV1,
    decision: DecisionProvenanceV1,
) -> Result<ConfirmedActionOutcomeV1, PersonsTransitionErrorV1> {
    let digest = split_person_action_digest_v1(&action)?;
    person_for_owner(state, action.merged_person_id, &action.logical_owner_id)?;
    person_for_owner(state, action.target_person_id, &action.logical_owner_id)?;
    if let Some(replay) = validate_decision_and_replay(state, &decision, digest)? {
        return Ok(replay);
    }
    let (source_selection, profile_fact_selection) = normalized_split_selection(&action)?;
    let merged = person_for_owner(state, action.merged_person_id, &action.logical_owner_id)?;
    let target = person_for_owner(state, action.target_person_id, &action.logical_owner_id)?;
    if merged.lifecycle != PersonLifecycleV1::Merged
        || merged.merged_into != Some(action.target_person_id)
    {
        return Err(PersonsTransitionErrorV1::LineageConflict);
    }
    ensure_not_merged(target)?;
    require_person_revision(merged, action.expected_merged_person_revision)?;
    require_person_revision(target, action.expected_target_person_revision)?;
    require_decision_timestamp(&decision, [merged, target])?;
    let merge = state
        .lineage
        .iter()
        .rev()
        .find(|record| {
            record.change_kind == LineageChangeKindV1::Merge
                && record.source_person_id == action.merged_person_id
                && record.target_person_id == action.target_person_id
        })
        .cloned()
        .ok_or(PersonsTransitionErrorV1::LineageConflict)?;
    for selection in &source_selection {
        if !merge.moved_sources.contains(&selection.source)
            || state.source_owners.get(&selection.source).copied() != Some(action.target_person_id)
        {
            return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
        }
        let link = target
            .source_links
            .get(&selection.source)
            .ok_or(PersonsTransitionErrorV1::SourceNotFound)?;
        require_source_revision(link, selection.expected_source_revision)?;
    }
    let selected_profile = select_profile_facts(
        merge.preserved_source_profile.as_ref(),
        &profile_fact_selection,
    )?;

    let restored_links = {
        let target = state
            .persons
            .get_mut(&action.target_person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        let mut links = BTreeMap::new();
        for selection in &source_selection {
            let link = target
                .source_links
                .remove(&selection.source)
                .expect("split preflight proved selected source exists");
            links.insert(selection.source, link);
        }
        target.revision += 1;
        target.updated_at = decision.decided_at;
        target.lifecycle = lifecycle_after_source_loss(target);
        links
    };
    {
        let source = state
            .persons
            .get_mut(&action.merged_person_id)
            .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
        source.source_links = restored_links;
        source.owner_profile = selected_profile.clone();
        source.merged_into = None;
        source.revision += 1;
        source.updated_at = decision.decided_at;
        source.lifecycle = lifecycle_for_unconfirmed(source);
    }
    for selection in &source_selection {
        state
            .source_owners
            .insert(selection.source, action.merged_person_id);
    }
    state.lineage.push(LineageRecordV1 {
        change_kind: LineageChangeKindV1::Split,
        source_person_id: action.merged_person_id,
        target_person_id: action.target_person_id,
        moved_sources: source_selection
            .iter()
            .map(|selection| selection.source)
            .collect(),
        preserved_source_profile: selected_profile,
        profile_fact_selection,
        decision: decision.clone(),
    });
    Ok(record_decision(
        state,
        &action.logical_owner_id,
        decision,
        digest,
        vec![
            PersonRevisionV1 {
                person_id: action.merged_person_id,
                revision: action.expected_merged_person_revision + 1,
            },
            PersonRevisionV1 {
                person_id: action.target_person_id,
                revision: action.expected_target_person_revision + 1,
            },
        ],
    ))
}

fn validate_decision_and_replay(
    state: &PersonsStateV1,
    decision: &DecisionProvenanceV1,
    action_digest: DigestV1,
) -> Result<Option<ConfirmedActionOutcomeV1>, PersonsTransitionErrorV1> {
    validate_decision(decision)?;
    if decision.approved_action_digest != action_digest {
        return Err(PersonsTransitionErrorV1::ActionDigestMismatch);
    }
    let Some(receipt) = state.decisions.get(&decision.decision_id) else {
        return Ok(None);
    };
    if receipt.action_digest != action_digest || receipt.decision != *decision {
        return Err(PersonsTransitionErrorV1::DecisionReuseConflict);
    }
    let mut replay = receipt.outcome.clone();
    replay.status = ConfirmedActionStatusV1::Replayed;
    Ok(Some(replay))
}

fn record_decision(
    state: &mut PersonsStateV1,
    logical_owner_id: &str,
    decision: DecisionProvenanceV1,
    action_digest: DigestV1,
    mut person_revisions: Vec<PersonRevisionV1>,
) -> ConfirmedActionOutcomeV1 {
    person_revisions.sort();
    let outcome = ConfirmedActionOutcomeV1 {
        status: ConfirmedActionStatusV1::Applied,
        person_revisions,
    };
    state.decisions.insert(
        decision.decision_id,
        DecisionReceiptV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            action_digest,
            decision,
            outcome: outcome.clone(),
        },
    );
    outcome
}

fn normalized_split_selection(
    action: &SplitPersonActionV1,
) -> Result<(Vec<SplitSourceSelectionV1>, Vec<SplitProfileFactKindV1>), PersonsTransitionErrorV1> {
    if action.source_selection.is_empty() && action.profile_fact_selection.is_empty() {
        return Err(PersonsTransitionErrorV1::EmptySplitSelection);
    }
    let mut sources = action.source_selection.clone();
    for source in &sources {
        validate_source_key(source.source)?;
        require_revision_value(source.expected_source_revision)?;
    }
    sources.sort();
    if sources
        .windows(2)
        .any(|pair| pair[0].source == pair[1].source)
    {
        return Err(PersonsTransitionErrorV1::DuplicateSplitSelection);
    }
    let mut profile_facts = action.profile_fact_selection.clone();
    profile_facts.sort();
    if profile_facts.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PersonsTransitionErrorV1::DuplicateSplitSelection);
    }
    Ok((sources, profile_facts))
}

fn select_profile_facts(
    preserved: Option<&OwnerProfileV1>,
    selection: &[SplitProfileFactKindV1],
) -> Result<Option<OwnerProfileV1>, PersonsTransitionErrorV1> {
    if selection.is_empty() {
        return Ok(None);
    }
    let preserved = preserved.ok_or(PersonsTransitionErrorV1::ProfileFactUnavailable)?;
    let mut selected = OwnerProfileV1 {
        display_name: None,
        given_name: None,
        family_name: None,
        emails: Vec::new(),
        phones: Vec::new(),
    };
    for fact in selection {
        match fact {
            SplitProfileFactKindV1::DisplayName => {
                selected.display_name = Some(
                    preserved
                        .display_name
                        .clone()
                        .ok_or(PersonsTransitionErrorV1::ProfileFactUnavailable)?,
                );
            }
            SplitProfileFactKindV1::GivenName => {
                selected.given_name = Some(
                    preserved
                        .given_name
                        .clone()
                        .ok_or(PersonsTransitionErrorV1::ProfileFactUnavailable)?,
                );
            }
            SplitProfileFactKindV1::FamilyName => {
                selected.family_name = Some(
                    preserved
                        .family_name
                        .clone()
                        .ok_or(PersonsTransitionErrorV1::ProfileFactUnavailable)?,
                );
            }
            SplitProfileFactKindV1::Emails => {
                if preserved.emails.is_empty() {
                    return Err(PersonsTransitionErrorV1::ProfileFactUnavailable);
                }
                selected.emails.clone_from(&preserved.emails);
            }
            SplitProfileFactKindV1::Phones => {
                if preserved.phones.is_empty() {
                    return Err(PersonsTransitionErrorV1::ProfileFactUnavailable);
                }
                selected.phones.clone_from(&preserved.phones);
            }
        }
    }
    Ok(Some(selected))
}

fn normalize_observation(
    observation: SourceObservationV1,
) -> Result<SourceObservationV1, PersonsTransitionErrorV1> {
    validate_owner(&observation.logical_owner_id)?;
    validate_source_key(observation.key)?;
    validate_source_provenance(observation.provenance)?;
    Ok(SourceObservationV1 {
        logical_owner_id: observation.logical_owner_id,
        key: observation.key,
        claims: normalize_source_claims(observation.claims)?,
        provenance: observation.provenance,
    })
}

fn normalize_source_claims(
    claims: SourceClaimsV1,
) -> Result<SourceClaimsV1, PersonsTransitionErrorV1> {
    let display_name = normalize_optional_text(claims.display_name)?;
    let emails = normalize_emails(claims.emails)?;
    let phones = normalize_phones(claims.phones)?;
    if display_name.is_none() && emails.is_empty() && phones.is_empty() {
        return Err(PersonsTransitionErrorV1::InvalidSourceClaims);
    }
    Ok(SourceClaimsV1 {
        display_name,
        emails,
        phones,
    })
}

fn normalize_owner_profile(
    profile: OwnerProfileV1,
) -> Result<OwnerProfileV1, PersonsTransitionErrorV1> {
    Ok(OwnerProfileV1 {
        display_name: normalize_optional_text(profile.display_name)?,
        given_name: normalize_optional_text(profile.given_name)?,
        family_name: normalize_optional_text(profile.family_name)?,
        emails: normalize_emails(profile.emails)?,
        phones: normalize_phones(profile.phones)?,
    })
}

fn normalize_optional_text(
    value: Option<String>,
) -> Result<Option<String>, PersonsTransitionErrorV1> {
    value
        .map(|value| {
            let normalized = value.trim();
            if normalized.is_empty()
                || normalized.chars().count() > MAX_PROFILE_TEXT_CHARS_V1
                || normalized.chars().any(char::is_control)
            {
                return Err(PersonsTransitionErrorV1::InvalidProfile);
            }
            Ok(normalized.to_owned())
        })
        .transpose()
}

fn normalize_emails(values: Vec<String>) -> Result<Vec<String>, PersonsTransitionErrorV1> {
    if values.len() > MAX_EMAILS_V1 {
        return Err(PersonsTransitionErrorV1::InvalidSourceClaims);
    }
    values
        .into_iter()
        .map(|value| normalize_email_v1(&value))
        .collect::<Result<BTreeSet<_>, _>>()
        .map(BTreeSet::into_iter)
        .map(Iterator::collect)
}

fn normalize_phones(values: Vec<String>) -> Result<Vec<String>, PersonsTransitionErrorV1> {
    if values.len() > MAX_PHONES_V1 {
        return Err(PersonsTransitionErrorV1::InvalidSourceClaims);
    }
    values
        .into_iter()
        .map(|value| normalize_phone_v1(&value))
        .collect::<Result<BTreeSet<_>, _>>()
        .map(BTreeSet::into_iter)
        .map(Iterator::collect)
}

fn review_candidates_for(
    state: &PersonsStateV1,
    logical_owner_id: &str,
    observed_person_id: PersonIdV1,
    observed_key: SourceLinkKeyV1,
    observed_claims: &SourceClaimsV1,
    observed_at: TimestampV1,
) -> Result<Vec<ReviewCandidateV1>, PersonsTransitionErrorV1> {
    let mut candidates = BTreeMap::new();
    for person in state.persons.values() {
        if person.person_id == observed_person_id
            || person.lifecycle == PersonLifecycleV1::Merged
            || logical_owner_id != person.logical_owner_id
        {
            continue;
        }
        for link in person.source_links.values() {
            for match_kind in matching_kinds(observed_claims, &link.claims) {
                let candidate_id =
                    derive_candidate_id(logical_owner_id, observed_key, link.key, match_kind);
                if !candidates.contains_key(&candidate_id)
                    && candidates.len() == MAX_REVIEW_CANDIDATES_PER_COMMAND_V1
                {
                    return Err(PersonsTransitionErrorV1::ReviewCandidateLimitExceeded);
                }
                candidates.entry(candidate_id).or_insert(ReviewCandidateV1 {
                    candidate_id,
                    first_person_id: observed_person_id,
                    second_person_id: person.person_id,
                    first_source: observed_key,
                    second_source: link.key,
                    match_kind,
                    observed_at,
                });
            }
        }
    }
    Ok(candidates.into_values().collect())
}

fn matching_kinds(first: &SourceClaimsV1, second: &SourceClaimsV1) -> Vec<IdentityMatchKindV1> {
    let mut matches = Vec::new();
    if first
        .emails
        .iter()
        .any(|value| second.emails.contains(value))
    {
        matches.push(IdentityMatchKindV1::NormalizedEmail);
    }
    if first
        .phones
        .iter()
        .any(|value| second.phones.contains(value))
    {
        matches.push(IdentityMatchKindV1::NormalizedPhone);
    }
    matches
}

fn derive_source_person_id(
    logical_owner_id: &str,
    key: SourceLinkKeyV1,
) -> Result<PersonIdV1, PersonsTransitionErrorV1> {
    validate_owner(logical_owner_id)?;
    validate_source_key(key)?;
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.persons.source-person-id.v1");
    update_part(&mut hasher, logical_owner_id.as_bytes());
    update_source_key(&mut hasher, key);
    Ok(PersonIdV1(
        hasher.finalize()[..16].try_into().expect("fixed digest"),
    ))
}

fn derive_candidate_id(
    logical_owner_id: &str,
    first: SourceLinkKeyV1,
    second: SourceLinkKeyV1,
    match_kind: IdentityMatchKindV1,
) -> PublicIdV1 {
    use makosh_persons_api::{PersonsIdentityMatchKindV1, persons_identity_match_candidate_id_v1};
    PublicIdV1(
        persons_identity_match_candidate_id_v1(
            logical_owner_id,
            api_source(first),
            api_source(second),
            match match_kind {
                IdentityMatchKindV1::NormalizedEmail => PersonsIdentityMatchKindV1::NormalizedEmail,
                IdentityMatchKindV1::NormalizedPhone => PersonsIdentityMatchKindV1::NormalizedPhone,
            },
        )
        .expect("validated Persons candidate inputs"),
    )
}

fn action_hasher(action_kind: &str, logical_owner_id: &str) -> Sha256 {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.persons.confirmed-action.v1");
    update_part(&mut hasher, action_kind.as_bytes());
    update_part(&mut hasher, logical_owner_id.as_bytes());
    hasher
}

fn update_person_id(hasher: &mut Sha256, person_id: PersonIdV1) {
    update_part(hasher, &person_id.0);
}

fn update_revision(hasher: &mut Sha256, revision: u64) {
    hasher.update(revision.to_be_bytes());
}

fn update_source_key(hasher: &mut Sha256, key: SourceLinkKeyV1) {
    update_part(hasher, &key.integration_public_id.0);
    update_part(hasher, &key.account_public_id.0);
    update_part(hasher, &key.provider_source_contact_public_id.0);
}

const fn api_source(key: SourceLinkKeyV1) -> PersonsActionDigestSourceV1 {
    PersonsActionDigestSourceV1 {
        integration_public_id: key.integration_public_id.0,
        account_public_id: key.account_public_id.0,
        provider_source_contact_public_id: key.provider_source_contact_public_id.0,
    }
}

fn update_part(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn profile_fact_tag(fact: SplitProfileFactKindV1) -> u8 {
    match fact {
        SplitProfileFactKindV1::DisplayName => 1,
        SplitProfileFactKindV1::GivenName => 2,
        SplitProfileFactKindV1::FamilyName => 3,
        SplitProfileFactKindV1::Emails => 4,
        SplitProfileFactKindV1::Phones => 5,
    }
}

fn compare_source_revision(
    current: SourceProvenanceV1,
    incoming: SourceProvenanceV1,
) -> Result<(), PersonsTransitionErrorV1> {
    if incoming.revision < current.revision {
        return Err(PersonsTransitionErrorV1::StaleSourceRevision);
    }
    if incoming.revision == current.revision && incoming.digest != current.digest {
        return Err(PersonsTransitionErrorV1::SourceRevisionConflict);
    }
    Ok(())
}

fn person_for_owner<'a>(
    state: &'a PersonsStateV1,
    person_id: PersonIdV1,
    logical_owner_id: &str,
) -> Result<&'a PersonV1, PersonsTransitionErrorV1> {
    let person = state
        .persons
        .get(&person_id)
        .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
    if person.logical_owner_id != logical_owner_id {
        return Err(PersonsTransitionErrorV1::OwnerMismatch);
    }
    Ok(person)
}

fn person_for_owner_mut<'a>(
    state: &'a mut PersonsStateV1,
    person_id: PersonIdV1,
    logical_owner_id: &str,
) -> Result<&'a mut PersonV1, PersonsTransitionErrorV1> {
    let person = state
        .persons
        .get_mut(&person_id)
        .ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
    if person.logical_owner_id != logical_owner_id {
        return Err(PersonsTransitionErrorV1::OwnerMismatch);
    }
    Ok(person)
}

fn ensure_not_merged(person: &PersonV1) -> Result<(), PersonsTransitionErrorV1> {
    if person.lifecycle == PersonLifecycleV1::Merged {
        Err(PersonsTransitionErrorV1::PersonMerged)
    } else {
        Ok(())
    }
}

fn require_person_revision(
    person: &PersonV1,
    expected_revision: u64,
) -> Result<(), PersonsTransitionErrorV1> {
    require_revision_value(expected_revision)?;
    if person.revision == expected_revision {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::ExpectedRevisionConflict)
    }
}

fn require_source_revision(
    source: &SourceLinkV1,
    expected_revision: u64,
) -> Result<(), PersonsTransitionErrorV1> {
    require_revision_value(expected_revision)?;
    if source.provenance.revision == expected_revision {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::ExpectedSourceRevisionConflict)
    }
}

fn require_revision_value(revision: u64) -> Result<(), PersonsTransitionErrorV1> {
    if revision == 0 {
        Err(PersonsTransitionErrorV1::InvalidRevision)
    } else {
        Ok(())
    }
}

fn require_decision_timestamp<const N: usize>(
    decision: &DecisionProvenanceV1,
    persons: [&PersonV1; N],
) -> Result<(), PersonsTransitionErrorV1> {
    if persons
        .into_iter()
        .any(|person| timestamp_before(decision.decided_at, person.updated_at))
    {
        Err(PersonsTransitionErrorV1::DecisionTimestampRegression)
    } else {
        Ok(())
    }
}

fn validate_owner(value: &str) -> Result<(), PersonsTransitionErrorV1> {
    if value.is_empty()
        || value.len() > MAX_LOGICAL_OWNER_ID_BYTES_V1
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(PersonsTransitionErrorV1::InvalidOwner);
    }
    Ok(())
}

fn validate_source_key(key: SourceLinkKeyV1) -> Result<(), PersonsTransitionErrorV1> {
    validate_public_id(key.integration_public_id)?;
    validate_public_id(key.account_public_id)?;
    validate_public_id(key.provider_source_contact_public_id)
}

fn validate_source_provenance(value: SourceProvenanceV1) -> Result<(), PersonsTransitionErrorV1> {
    require_revision_value(value.revision)?;
    if !nonzero(&value.digest.0) {
        return Err(PersonsTransitionErrorV1::InvalidDigest);
    }
    validate_timestamp(value.observed_at)
}

fn validate_decision(value: &DecisionProvenanceV1) -> Result<(), PersonsTransitionErrorV1> {
    validate_public_id(value.decision_id)
        .map_err(|_| PersonsTransitionErrorV1::DecisionRequired)?;
    validate_public_id(value.review_id).map_err(|_| PersonsTransitionErrorV1::DecisionRequired)?;
    validate_public_id(value.decided_by_owner_device_id)
        .map_err(|_| PersonsTransitionErrorV1::DecisionRequired)?;
    if value.revision == 0
        || validate_timestamp(value.decided_at).is_err()
        || !nonzero(&value.approved_action_digest.0)
    {
        return Err(PersonsTransitionErrorV1::DecisionRequired);
    }
    Ok(())
}

fn validate_public_id(value: PublicIdV1) -> Result<(), PersonsTransitionErrorV1> {
    if nonzero(&value.0) {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::InvalidPublicId)
    }
}

fn validate_person_id(value: PersonIdV1) -> Result<(), PersonsTransitionErrorV1> {
    if nonzero(&value.0) {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::InvalidPersonId)
    }
}

fn validate_timestamp(value: TimestampV1) -> Result<(), PersonsTransitionErrorV1> {
    if value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos) {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::InvalidTimestamp)
    }
}

fn lifecycle_after_source_loss(person: &PersonV1) -> PersonLifecycleV1 {
    if person.source_links.is_empty() && person.owner_profile.is_none() {
        PersonLifecycleV1::Archived
    } else {
        PersonLifecycleV1::Active
    }
}

fn lifecycle_for_unconfirmed(person: &PersonV1) -> PersonLifecycleV1 {
    if person.owner_profile.is_some() {
        PersonLifecycleV1::Active
    } else if person.source_links.is_empty() {
        PersonLifecycleV1::Archived
    } else {
        PersonLifecycleV1::Provisional
    }
}

fn timestamp_before(first: TimestampV1, second: TimestampV1) -> bool {
    (first.unix_seconds, first.nanos) < (second.unix_seconds, second.nanos)
}

fn nonzero(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| *byte != 0)
}
