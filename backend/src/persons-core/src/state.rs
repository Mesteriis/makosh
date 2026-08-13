use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ConfirmedActionStatusV1, DecisionProvenanceV1, DecisionReceiptV1, DigestV1, LineageRecordV1,
    MAX_EMAILS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_PHONES_V1, MAX_PROFILE_TEXT_CHARS_V1,
    OwnerProfileV1, PersonIdV1, PersonLifecycleV1, PersonV1, PersonsOwnerSnapshotV1,
    PersonsTransitionErrorV1, PublicIdV1, RemovedSourceV1, SourceClaimsV1, SourceLinkKeyV1,
    SourceLinkV1, SourceProvenanceV1, TimestampV1, normalize_email_v1, normalize_phone_v1,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PersonsStateV1 {
    pub(crate) persons: BTreeMap<PersonIdV1, PersonV1>,
    pub(crate) source_owners: BTreeMap<SourceLinkKeyV1, PersonIdV1>,
    pub(crate) removed_sources: BTreeMap<SourceLinkKeyV1, RemovedSourceV1>,
    pub(crate) lineage: Vec<LineageRecordV1>,
    pub(crate) decisions: BTreeMap<PublicIdV1, DecisionReceiptV1>,
}

impl PersonsStateV1 {
    #[must_use]
    pub fn person(&self, person_id: PersonIdV1) -> Option<&PersonV1> {
        self.persons.get(&person_id)
    }

    pub fn persons(&self) -> impl Iterator<Item = &PersonV1> {
        self.persons.values()
    }

    pub fn lineage(&self) -> impl Iterator<Item = &LineageRecordV1> {
        self.lineage.iter()
    }

    #[must_use]
    pub fn source_owner(&self, source: SourceLinkKeyV1) -> Option<PersonIdV1> {
        self.source_owners.get(&source).copied()
    }

    pub fn snapshot_for_owner_v1(
        &self,
        logical_owner_id: &str,
    ) -> Result<PersonsOwnerSnapshotV1, PersonsTransitionErrorV1> {
        validate_owner(logical_owner_id)?;
        let persons = self
            .persons
            .values()
            .filter(|person| person.logical_owner_id == logical_owner_id)
            .cloned()
            .collect::<Vec<_>>();
        let owner_person_ids = persons
            .iter()
            .map(|person| person.person_id)
            .collect::<BTreeSet<_>>();
        let removed_sources = self
            .removed_sources
            .iter()
            .filter(|(_, removed)| removed.logical_owner_id == logical_owner_id)
            .map(|(key, removed)| (*key, removed.clone()))
            .collect();
        let lineage = self
            .lineage
            .iter()
            .filter(|record| owner_person_ids.contains(&record.source_person_id))
            .cloned()
            .collect();
        let decision_receipts = self
            .decisions
            .values()
            .filter(|receipt| receipt.logical_owner_id == logical_owner_id)
            .cloned()
            .collect();
        Ok(PersonsOwnerSnapshotV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            persons,
            removed_sources,
            lineage,
            decision_receipts,
        })
    }

    pub fn reconstitute_owner_v1(
        snapshot: PersonsOwnerSnapshotV1,
    ) -> Result<Self, PersonsTransitionErrorV1> {
        validate_owner(&snapshot.logical_owner_id)?;
        let mut state = Self::default();
        for person in snapshot.persons {
            validate_person(&person, &snapshot.logical_owner_id)?;
            let person_id = person.person_id;
            for (key, source) in &person.source_links {
                validate_source(*key, source)?;
                if state.source_owners.insert(*key, person_id).is_some() {
                    return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
                }
            }
            if state.persons.insert(person_id, person).is_some() {
                return Err(PersonsTransitionErrorV1::InvalidSnapshot);
            }
        }
        for (key, removed) in snapshot.removed_sources {
            validate_source_key(key)?;
            if removed.logical_owner_id != snapshot.logical_owner_id
                || state.source_owners.contains_key(&key)
                || state.removed_sources.contains_key(&key)
            {
                return Err(PersonsTransitionErrorV1::SourceOwnerConflict);
            }
            validate_provenance(removed.provenance)?;
            state.removed_sources.insert(key, removed);
        }
        for record in snapshot.lineage {
            validate_lineage(&record, &state, &snapshot.logical_owner_id)?;
            state.lineage.push(record);
        }
        for receipt in snapshot.decision_receipts {
            validate_receipt(&receipt, &state, &snapshot.logical_owner_id)?;
            if state
                .decisions
                .insert(receipt.decision.decision_id, receipt)
                .is_some()
            {
                return Err(PersonsTransitionErrorV1::DecisionReuseConflict);
            }
        }
        validate_owner_graph(&state, &snapshot.logical_owner_id)?;
        Ok(state)
    }
}

fn validate_owner_graph(
    state: &PersonsStateV1,
    logical_owner_id: &str,
) -> Result<(), PersonsTransitionErrorV1> {
    for person in state.persons.values() {
        if let Some(target_id) = person.merged_into {
            if target_id == person.person_id {
                return Err(PersonsTransitionErrorV1::LineageConflict);
            }
            let target = state
                .person(target_id)
                .ok_or(PersonsTransitionErrorV1::LineageConflict)?;
            if target.logical_owner_id != logical_owner_id {
                return Err(PersonsTransitionErrorV1::OwnerMismatch);
            }
            let mut visited = BTreeSet::new();
            let mut next = Some(person.person_id);
            while let Some(person_id) = next {
                if !visited.insert(person_id) {
                    return Err(PersonsTransitionErrorV1::LineageConflict);
                }
                next = state
                    .person(person_id)
                    .ok_or(PersonsTransitionErrorV1::LineageConflict)?
                    .merged_into;
            }
        }
    }

    let mut lineage_decisions = BTreeSet::new();
    for record in &state.lineage {
        if record.source_person_id == record.target_person_id
            || !lineage_decisions.insert(record.decision.decision_id)
        {
            return Err(PersonsTransitionErrorV1::InvalidSnapshot);
        }
        let receipt = state
            .decisions
            .get(&record.decision.decision_id)
            .ok_or(PersonsTransitionErrorV1::InvalidSnapshot)?;
        if receipt.decision != record.decision
            || receipt.action_digest != record.decision.approved_action_digest
            || receipt.outcome.status != ConfirmedActionStatusV1::Applied
        {
            return Err(PersonsTransitionErrorV1::InvalidSnapshot);
        }
        let outcome_ids = receipt
            .outcome
            .person_revisions
            .iter()
            .map(|revision| revision.person_id)
            .collect::<BTreeSet<_>>();
        let expected_ids = BTreeSet::from([record.source_person_id, record.target_person_id]);
        if receipt.outcome.person_revisions.len() != 2 || outcome_ids != expected_ids {
            return Err(PersonsTransitionErrorV1::InvalidSnapshot);
        }
    }
    Ok(())
}

fn validate_person(
    person: &PersonV1,
    logical_owner_id: &str,
) -> Result<(), PersonsTransitionErrorV1> {
    validate_person_id(person.person_id)?;
    if person.logical_owner_id != logical_owner_id || person.revision == 0 {
        return Err(PersonsTransitionErrorV1::OwnerMismatch);
    }
    validate_timestamp(person.created_at)?;
    validate_timestamp(person.updated_at)?;
    if timestamp_before(person.updated_at, person.created_at) {
        return Err(PersonsTransitionErrorV1::InvalidTimestamp);
    }
    if let Some(profile) = &person.owner_profile {
        validate_profile(profile)?;
    }
    match person.lifecycle {
        PersonLifecycleV1::Merged => {
            let merged_into = person
                .merged_into
                .ok_or(PersonsTransitionErrorV1::LineageConflict)?;
            validate_person_id(merged_into)?;
            if !person.source_links.is_empty() {
                return Err(PersonsTransitionErrorV1::LineageConflict);
            }
        }
        PersonLifecycleV1::Archived => {
            if person.merged_into.is_some()
                || person.owner_profile.is_some()
                || !person.source_links.is_empty()
            {
                return Err(PersonsTransitionErrorV1::InvalidSnapshot);
            }
        }
        PersonLifecycleV1::Active => {
            if person.merged_into.is_some()
                || (person.owner_profile.is_none() && person.source_links.is_empty())
            {
                return Err(PersonsTransitionErrorV1::InvalidSnapshot);
            }
        }
        PersonLifecycleV1::Provisional => {
            if person.merged_into.is_some()
                || person.owner_profile.is_some()
                || person.source_links.is_empty()
            {
                return Err(PersonsTransitionErrorV1::InvalidSnapshot);
            }
        }
    }
    Ok(())
}

fn validate_source(
    key: SourceLinkKeyV1,
    source: &SourceLinkV1,
) -> Result<(), PersonsTransitionErrorV1> {
    validate_source_key(key)?;
    if source.key != key {
        return Err(PersonsTransitionErrorV1::InvalidSnapshot);
    }
    validate_claims(&source.claims)?;
    validate_provenance(source.provenance)?;
    if let Some(decision) = &source.last_decision {
        validate_decision(decision)?;
    }
    Ok(())
}

fn validate_lineage(
    record: &LineageRecordV1,
    state: &PersonsStateV1,
    logical_owner_id: &str,
) -> Result<(), PersonsTransitionErrorV1> {
    let source = state
        .person(record.source_person_id)
        .ok_or(PersonsTransitionErrorV1::LineageConflict)?;
    let target = state
        .person(record.target_person_id)
        .ok_or(PersonsTransitionErrorV1::LineageConflict)?;
    if source.logical_owner_id != logical_owner_id || target.logical_owner_id != logical_owner_id {
        return Err(PersonsTransitionErrorV1::OwnerMismatch);
    }
    validate_decision(&record.decision)?;
    if let Some(profile) = &record.preserved_source_profile {
        validate_profile(profile)?;
    }
    let mut sources = BTreeSet::new();
    for key in &record.moved_sources {
        validate_source_key(*key)?;
        if !sources.insert(*key) {
            return Err(PersonsTransitionErrorV1::InvalidSnapshot);
        }
    }
    let mut facts = BTreeSet::new();
    if record
        .profile_fact_selection
        .iter()
        .any(|fact| !facts.insert(*fact))
    {
        return Err(PersonsTransitionErrorV1::InvalidSnapshot);
    }
    Ok(())
}

fn validate_receipt(
    receipt: &DecisionReceiptV1,
    state: &PersonsStateV1,
    logical_owner_id: &str,
) -> Result<(), PersonsTransitionErrorV1> {
    if receipt.logical_owner_id != logical_owner_id
        || !valid_digest(receipt.action_digest)
        || receipt.decision.approved_action_digest != receipt.action_digest
        || receipt.outcome.status != ConfirmedActionStatusV1::Applied
        || receipt.outcome.person_revisions.is_empty()
    {
        return Err(PersonsTransitionErrorV1::InvalidSnapshot);
    }
    validate_decision(&receipt.decision)?;
    let mut persons = BTreeSet::new();
    for result in &receipt.outcome.person_revisions {
        let person = state
            .person(result.person_id)
            .ok_or(PersonsTransitionErrorV1::InvalidSnapshot)?;
        if person.logical_owner_id != logical_owner_id
            || result.revision == 0
            || result.revision > person.revision
            || !persons.insert(result.person_id)
        {
            return Err(PersonsTransitionErrorV1::InvalidSnapshot);
        }
    }
    Ok(())
}

fn validate_profile(profile: &OwnerProfileV1) -> Result<(), PersonsTransitionErrorV1> {
    if profile.is_empty()
        || !valid_optional_text(&profile.display_name)
        || !valid_optional_text(&profile.given_name)
        || !valid_optional_text(&profile.family_name)
        || !valid_emails(&profile.emails)
        || !valid_phones(&profile.phones)
    {
        return Err(PersonsTransitionErrorV1::InvalidProfile);
    }
    Ok(())
}

fn validate_claims(claims: &SourceClaimsV1) -> Result<(), PersonsTransitionErrorV1> {
    if (claims.display_name.is_none() && claims.emails.is_empty() && claims.phones.is_empty())
        || !valid_optional_text(&claims.display_name)
        || !valid_emails(&claims.emails)
        || !valid_phones(&claims.phones)
    {
        return Err(PersonsTransitionErrorV1::InvalidSourceClaims);
    }
    Ok(())
}

fn valid_optional_text(value: &Option<String>) -> bool {
    value.as_ref().is_none_or(|value| {
        !value.is_empty()
            && value.chars().count() <= MAX_PROFILE_TEXT_CHARS_V1
            && !value.chars().any(char::is_control)
            && value.trim() == value
    })
}

fn valid_emails(values: &[String]) -> bool {
    values.len() <= MAX_EMAILS_V1
        && values.windows(2).all(|pair| pair[0] < pair[1])
        && values
            .iter()
            .all(|value| normalize_email_v1(value).is_ok_and(|normalized| normalized == *value))
}

fn valid_phones(values: &[String]) -> bool {
    values.len() <= MAX_PHONES_V1
        && values.windows(2).all(|pair| pair[0] < pair[1])
        && values
            .iter()
            .all(|value| normalize_phone_v1(value).is_ok_and(|normalized| normalized == *value))
}

fn validate_source_key(key: SourceLinkKeyV1) -> Result<(), PersonsTransitionErrorV1> {
    validate_public_id(key.integration_public_id)?;
    validate_public_id(key.account_public_id)?;
    validate_public_id(key.provider_source_contact_public_id)
}

fn validate_provenance(value: SourceProvenanceV1) -> Result<(), PersonsTransitionErrorV1> {
    if value.revision == 0 || !valid_digest(value.digest) {
        return Err(PersonsTransitionErrorV1::InvalidSnapshot);
    }
    validate_timestamp(value.observed_at)
}

fn validate_decision(value: &DecisionProvenanceV1) -> Result<(), PersonsTransitionErrorV1> {
    validate_public_id(value.decision_id)?;
    validate_public_id(value.review_id)?;
    validate_public_id(value.decided_by_owner_device_id)?;
    if value.revision == 0 || !valid_digest(value.approved_action_digest) {
        return Err(PersonsTransitionErrorV1::InvalidSnapshot);
    }
    validate_timestamp(value.decided_at)
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

fn validate_person_id(value: PersonIdV1) -> Result<(), PersonsTransitionErrorV1> {
    if value.0.iter().any(|byte| *byte != 0) {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::InvalidPersonId)
    }
}

fn validate_public_id(value: PublicIdV1) -> Result<(), PersonsTransitionErrorV1> {
    if value.0.iter().any(|byte| *byte != 0) {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::InvalidPublicId)
    }
}

fn valid_digest(value: DigestV1) -> bool {
    value.0.iter().any(|byte| *byte != 0)
}

fn validate_timestamp(value: TimestampV1) -> Result<(), PersonsTransitionErrorV1> {
    if value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos) {
        Ok(())
    } else {
        Err(PersonsTransitionErrorV1::InvalidTimestamp)
    }
}

fn timestamp_before(first: TimestampV1, second: TimestampV1) -> bool {
    (first.unix_seconds, first.nanos) < (second.unix_seconds, second.nanos)
}
