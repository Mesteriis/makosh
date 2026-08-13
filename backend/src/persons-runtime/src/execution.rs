use std::collections::{BTreeMap, BTreeSet};

use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, CommandMetadataV1, ContractRefV1, FenceKindV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::decode_envelope_v1,
};
use makosh_persons_api::{
    PERSONS_COMMAND_CAPABILITY_ID_V1, persons_command_contract_reference_v1,
    persons_owner_partition_id_v1,
    wire::{
        self, IdentityMatchKindV1 as WireIdentityMatchKindV1,
        LineageChangeKindV1 as WireLineageChangeKindV1,
        PersonChangeKindV1 as WirePersonChangeKindV1, PersonChangedEventV1,
        PersonCommandRejectedV1, PersonCommandSucceededV1,
        PersonLifecycleV1 as WirePersonLifecycleV1, PersonLineageChangedEventV1,
        PersonProfileChangedEventV1, PersonReviewCandidateRaisedEventV1,
        PersonRevisionV1 as WirePersonRevisionV1, PersonSourceLinkChangedEventV1, PersonsCommandV1,
        PersonsOwnerEventV1, SourceLinkChangeKindV1 as WireSourceLinkChangeKindV1,
        TimestampV1 as WireTimestampV1, persons_owner_event_v1::Event,
    },
};
use makosh_persons_core::{
    ConfirmedActionOutcomeV1, DecisionProvenanceV1, IdentityMatchKindV1, LineageChangeKindV1,
    OwnerProfileV1, PersonIdV1, PersonLifecycleV1, PersonRevisionV1, PersonV1, PersonsStateV1,
    PersonsTransitionErrorV1, ReviewCandidateV1, SourceLinkKeyV1, SourceLinkV1,
    SourceObservationOutcomeV1, SourceProvenanceV1, TimestampV1, attach_source_v1,
    create_manual_person_v1, detach_source_v1, merge_persons_v1, observe_source_v1,
    remove_source_v1, split_person_v1, update_owner_profile_v1,
};
use makosh_persons_persistence::{
    ApplyPersonsCommandOutcomeV1, ApplyPersonsCommandV1, PersonsCommandCommitV1,
    PersonsEnvelopeRecordV1, PersonsPersistenceErrorV1, PersonsPersistenceV1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    command::{DecodedPersonsCommandV1, decode_typed_command_v1, persons_wire_command_identity_v1},
    transport::{
        PersonsEnvelopeContextV1, build_persons_command_rejected_outbox_record_v1,
        build_persons_command_succeeded_outbox_record_v1,
        build_persons_owner_event_outbox_record_v1,
        build_persons_review_candidate_outbox_record_v1, persons_command_fingerprint_v1,
        persons_deterministic_public_id_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsCommandRuntimeContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsCommandExecutionErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(PersonsPersistenceErrorV1),
}

struct ValidatedCommandV1 {
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_id: [u8; 16],
    fingerprint: [u8; 32],
    correlation_id: [u8; 16],
    command: DecodedPersonsCommandV1,
    expired: bool,
}

#[derive(Clone)]
struct AppliedFactsV1 {
    change_kind: WirePersonChangeKindV1,
    person_revisions: Vec<PersonRevisionV1>,
    review_candidates: Vec<ReviewCandidateV1>,
    source_provenance: Option<SourceProvenanceV1>,
    decision: Option<DecisionProvenanceV1>,
    emit_source_diff: bool,
}

pub async fn execute_persons_command_record_v1(
    persistence: &PersonsPersistenceV1,
    record: &OutboxRecordV1,
    runtime: &PersonsCommandRuntimeContextV1,
) -> Result<ApplyPersonsCommandOutcomeV1, PersonsCommandExecutionErrorV1> {
    let validated = validate_command_record_v1(record, runtime)?;
    let loaded = persistence
        .load_owner(&runtime.logical_owner_id)
        .await
        .map_err(PersonsCommandExecutionErrorV1::Persistence)?;
    let expected_aggregate_revision = loaded.aggregate_revision;
    let command = validated.command.clone();
    let context = output_context(runtime);
    persistence
        .apply_command_once(
            &ApplyPersonsCommandV1 {
                logical_owner_id: runtime.logical_owner_id.clone(),
                command_message_id: validated.message_id,
                command_envelope_sha256: validated.envelope_sha256,
                command_id: validated.command_id,
                command_fingerprint: validated.fingerprint,
                expected_aggregate_revision,
                received_at_unix_millis: runtime.now_unix_millis,
            },
            |state| {
                if validated.expired {
                    return build_expired_commit_v1(
                        validated.message_id,
                        validated.command_id,
                        validated.correlation_id,
                        &runtime.logical_owner_id,
                        expected_aggregate_revision + 1,
                        runtime.now_unix_millis,
                        &context,
                    );
                }
                build_commit_v1(
                    state,
                    command,
                    validated.message_id,
                    validated.command_id,
                    validated.correlation_id,
                    &runtime.logical_owner_id,
                    expected_aggregate_revision + 1,
                    runtime.now_unix_millis,
                    &context,
                )
            },
        )
        .await
        .map_err(PersonsCommandExecutionErrorV1::Persistence)
}

fn validate_command_record_v1(
    record: &OutboxRecordV1,
    runtime: &PersonsCommandRuntimeContextV1,
) -> Result<ValidatedCommandV1, PersonsCommandExecutionErrorV1> {
    if runtime.logical_owner_id.is_empty()
        || runtime.logical_owner_id.len() > 128
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.now_unix_millis <= 0
    {
        return Err(PersonsCommandExecutionErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| PersonsCommandExecutionErrorV1::InvalidEnvelope)?;
    exact_contract(
        envelope.contract.as_ref(),
        &persons_command_contract_reference_v1(),
    )?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(PersonsCommandExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(PersonsCommandExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(PersonsCommandExecutionErrorV1::InvalidEnvelope)?;
    if envelope.message_id.as_slice() != record.message_id()
        || envelope.partition_key.len() != 16
        || envelope.correlation_id != envelope.partition_key
        || !envelope.causation_message_id.is_empty()
        || source.module_id.is_empty()
        || source.module_id.len() > 128
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != source.module_id.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != source.module_id.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        return Err(PersonsCommandExecutionErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(CommandMetadataV1 {
        command_id,
        target_capability,
        idempotency_key,
        deadline,
        logical_attempt,
    })) = envelope.semantics
    else {
        return Err(PersonsCommandExecutionErrorV1::InvalidEnvelope);
    };
    let deadline = deadline.ok_or(PersonsCommandExecutionErrorV1::InvalidEnvelope)?;
    let expired = deadline.seconds < runtime.now_unix_millis / 1_000
        || (deadline.seconds == runtime.now_unix_millis / 1_000
            && i64::from(deadline.nanos) <= (runtime.now_unix_millis % 1_000) * 1_000_000);
    if command_id.as_slice() != record.message_id()
        || target_capability != PERSONS_COMMAND_CAPABILITY_ID_V1
        || logical_attempt == 0
        || deadline.seconds <= 0
        || !(0..1_000_000_000).contains(&deadline.nanos)
    {
        return Err(PersonsCommandExecutionErrorV1::InvalidEnvelope);
    }
    let payload = PersonsCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| PersonsCommandExecutionErrorV1::InvalidPayload)?;
    let (payload_command_id, owner) = persons_wire_command_identity_v1(&payload)
        .map_err(|_| PersonsCommandExecutionErrorV1::InvalidPayload)?;
    let fingerprint = persons_command_fingerprint_v1(&payload);
    if payload_command_id.as_slice() != record.message_id()
        || owner != runtime.logical_owner_id
        || idempotency_key != fingerprint
    {
        return Err(PersonsCommandExecutionErrorV1::InvalidPayload);
    }
    let expected_partition = persons_owner_partition_id_v1(&owner)
        .map_err(|_| PersonsCommandExecutionErrorV1::InvalidPayload)?;
    if envelope.partition_key != expected_partition {
        return Err(PersonsCommandExecutionErrorV1::InvalidEnvelope);
    }
    let command = decode_typed_command_v1(payload.clone(), &owner, payload_command_id)
        .map_err(|_| PersonsCommandExecutionErrorV1::InvalidPayload)?;
    Ok(ValidatedCommandV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        command_id: payload_command_id,
        fingerprint,
        correlation_id: expected_partition,
        command,
        expired,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_expired_commit_v1(
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    correlation_id: [u8; 16],
    logical_owner_id: &str,
    resulting_owner_revision: u64,
    completed_at_unix_millis: i64,
    context: &PersonsEnvelopeContextV1,
) -> Result<PersonsCommandCommitV1, PersonsPersistenceErrorV1> {
    let rejected = PersonCommandRejectedV1 {
        command_id: command_id.to_vec(),
        code: wire::PersonRejectCodeV1::PersonRejectCodeInvalidRequest as i32,
        logical_owner_id: logical_owner_id.to_owned(),
        resulting_owner_revision,
    };
    let terminal = build_persons_command_rejected_outbox_record_v1(
        command_message_id,
        correlation_id,
        rejected,
        context,
    )
    .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
    Ok(PersonsCommandCommitV1 {
        terminal_result: envelope_record(&terminal),
        owner_events: Vec::new(),
        owner_event_order_keys: Vec::new(),
        completed_at_unix_millis,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_commit_v1(
    state: &mut PersonsStateV1,
    command: DecodedPersonsCommandV1,
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    correlation_id: [u8; 16],
    logical_owner_id: &str,
    resulting_owner_revision: u64,
    completed_at_unix_millis: i64,
    context: &PersonsEnvelopeContextV1,
) -> Result<PersonsCommandCommitV1, PersonsPersistenceErrorV1> {
    let before = state.clone();
    let mut after = before.clone();
    let applied = apply_transition_v1(&mut after, command);
    let (terminal_result, owner_events, owner_event_order_keys) = match applied {
        Ok(facts) => {
            let events = build_events_v1(
                &before,
                &after,
                &facts,
                command_message_id,
                correlation_id,
                logical_owner_id,
                resulting_owner_revision,
                context,
            )?;
            let succeeded = PersonCommandSucceededV1 {
                command_id: command_id.to_vec(),
                change_kind: facts.change_kind as i32,
                affected_person_ids: facts
                    .person_revisions
                    .iter()
                    .map(|revision| revision.person_id.0.to_vec())
                    .collect(),
                resulting_person_revisions: facts
                    .person_revisions
                    .iter()
                    .map(|revision| WirePersonRevisionV1 {
                        person_id: revision.person_id.0.to_vec(),
                        person_revision: revision.revision,
                    })
                    .collect(),
                logical_owner_id: logical_owner_id.to_owned(),
                resulting_owner_revision,
            };
            let terminal = build_persons_command_succeeded_outbox_record_v1(
                command_message_id,
                correlation_id,
                succeeded,
                context,
            )
            .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
            *state = after;
            (
                envelope_record(&terminal),
                events.records,
                events.order_keys,
            )
        }
        Err(error) => {
            let rejected = PersonCommandRejectedV1 {
                command_id: command_id.to_vec(),
                code: reject_code(error) as i32,
                logical_owner_id: logical_owner_id.to_owned(),
                resulting_owner_revision,
            };
            let terminal = build_persons_command_rejected_outbox_record_v1(
                command_message_id,
                correlation_id,
                rejected,
                context,
            )
            .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
            (envelope_record(&terminal), Vec::new(), Vec::new())
        }
    };
    Ok(PersonsCommandCommitV1 {
        terminal_result,
        owner_events,
        owner_event_order_keys,
        completed_at_unix_millis,
    })
}

fn apply_transition_v1(
    state: &mut PersonsStateV1,
    command: DecodedPersonsCommandV1,
) -> Result<AppliedFactsV1, PersonsTransitionErrorV1> {
    match command {
        DecodedPersonsCommandV1::ManualCreate(draft) => {
            let timestamp = draft.created_at;
            let person_id = create_manual_person_v1(state, draft)?;
            Ok(facts_for_person(
                WirePersonChangeKindV1::PersonChangeKindCreated,
                timestamp,
                state,
                person_id,
            ))
        }
        DecodedPersonsCommandV1::OwnerProfileUpdate {
            logical_owner_id,
            person_id,
            expected_person_revision,
            owner_profile,
            updated_at,
        } => {
            update_owner_profile_v1(
                state,
                &logical_owner_id,
                person_id,
                expected_person_revision,
                owner_profile,
                updated_at,
            )?;
            Ok(facts_for_person(
                WirePersonChangeKindV1::PersonChangeKindProfileUpdated,
                updated_at,
                state,
                person_id,
            ))
        }
        DecodedPersonsCommandV1::SourceObserve(observation) => {
            let timestamp = observation.provenance.observed_at;
            let provenance = observation.provenance;
            let outcome = observe_source_v1(state, observation)?;
            facts_for_source_outcome(state, outcome, timestamp, provenance)
        }
        DecodedPersonsCommandV1::SourceUpdate(observation) => {
            if state.source_owner(observation.key).is_none() {
                return Err(PersonsTransitionErrorV1::SourceNotFound);
            }
            let timestamp = observation.provenance.observed_at;
            let provenance = observation.provenance;
            let outcome = observe_source_v1(state, observation)?;
            let mut facts = facts_for_source_outcome(state, outcome, timestamp, provenance)?;
            if facts.change_kind == WirePersonChangeKindV1::PersonChangeKindSourceObserved {
                facts.change_kind = WirePersonChangeKindV1::PersonChangeKindSourceUpdated;
            }
            Ok(facts)
        }
        DecodedPersonsCommandV1::SourceRemove {
            logical_owner_id,
            source,
            provenance,
        } => {
            let outcome = remove_source_v1(state, &logical_owner_id, source, provenance)?;
            let person_revisions = outcome
                .person_id
                .and_then(|person_id| revision(state, person_id))
                .into_iter()
                .collect();
            Ok(AppliedFactsV1 {
                change_kind: WirePersonChangeKindV1::PersonChangeKindSourceRemoved,
                person_revisions,
                review_candidates: Vec::new(),
                source_provenance: Some(provenance),
                decision: None,
                emit_source_diff: true,
            })
        }
        DecodedPersonsCommandV1::ConfirmedAttach(action, decision) => {
            let timestamp = decision.decided_at;
            let result = attach_source_v1(state, action, decision.clone())?;
            facts_for_confirmed(
                WirePersonChangeKindV1::PersonChangeKindSourceAttached,
                timestamp,
                result,
                decision,
                true,
            )
        }
        DecodedPersonsCommandV1::ConfirmedDetach(action, decision) => {
            let timestamp = decision.decided_at;
            let result = detach_source_v1(state, action, decision.clone())?;
            facts_for_confirmed(
                WirePersonChangeKindV1::PersonChangeKindSourceDetached,
                timestamp,
                result,
                decision,
                true,
            )
        }
        DecodedPersonsCommandV1::ConfirmedMerge(action, decision) => {
            let timestamp = decision.decided_at;
            let result = merge_persons_v1(state, action, decision.clone())?;
            facts_for_confirmed(
                WirePersonChangeKindV1::PersonChangeKindMerged,
                timestamp,
                result,
                decision,
                false,
            )
        }
        DecodedPersonsCommandV1::ConfirmedSplit(action, decision) => {
            let timestamp = decision.decided_at;
            let result = split_person_v1(state, action, decision.clone())?;
            facts_for_confirmed(
                WirePersonChangeKindV1::PersonChangeKindSplit,
                timestamp,
                result,
                decision,
                false,
            )
        }
    }
}

fn facts_for_person(
    kind: WirePersonChangeKindV1,
    _timestamp: TimestampV1,
    state: &PersonsStateV1,
    person_id: PersonIdV1,
) -> AppliedFactsV1 {
    AppliedFactsV1 {
        change_kind: kind,
        person_revisions: revision(state, person_id).into_iter().collect(),
        review_candidates: Vec::new(),
        source_provenance: None,
        decision: None,
        emit_source_diff: false,
    }
}

fn facts_for_source_outcome(
    state: &PersonsStateV1,
    outcome: SourceObservationOutcomeV1,
    _timestamp: TimestampV1,
    provenance: SourceProvenanceV1,
) -> Result<AppliedFactsV1, PersonsTransitionErrorV1> {
    let kind = match outcome {
        SourceObservationOutcomeV1::Created { .. } => {
            WirePersonChangeKindV1::PersonChangeKindSourceObserved
        }
        SourceObservationOutcomeV1::Updated { .. } => {
            WirePersonChangeKindV1::PersonChangeKindSourceUpdated
        }
        SourceObservationOutcomeV1::Unchanged { .. } => {
            WirePersonChangeKindV1::PersonChangeKindSourceObserved
        }
    };
    let person_id = outcome.person_id();
    let person_revision =
        revision(state, person_id).ok_or(PersonsTransitionErrorV1::PersonNotFound)?;
    Ok(AppliedFactsV1 {
        change_kind: kind,
        person_revisions: vec![person_revision],
        review_candidates: outcome.review_candidates().to_vec(),
        source_provenance: Some(provenance),
        decision: None,
        emit_source_diff: !matches!(outcome, SourceObservationOutcomeV1::Unchanged { .. }),
    })
}

fn facts_for_confirmed(
    kind: WirePersonChangeKindV1,
    _timestamp: TimestampV1,
    outcome: ConfirmedActionOutcomeV1,
    decision: DecisionProvenanceV1,
    emit_source_diff: bool,
) -> Result<AppliedFactsV1, PersonsTransitionErrorV1> {
    Ok(AppliedFactsV1 {
        change_kind: kind,
        person_revisions: outcome.person_revisions,
        review_candidates: Vec::new(),
        source_provenance: None,
        decision: Some(decision),
        emit_source_diff,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_events_v1(
    before: &PersonsStateV1,
    after: &PersonsStateV1,
    facts: &AppliedFactsV1,
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    logical_owner_id: &str,
    resulting_owner_revision: u64,
    context: &PersonsEnvelopeContextV1,
) -> Result<BuiltEventsV1, PersonsPersistenceErrorV1> {
    let mut pending = Vec::<PendingEventV1>::new();
    for revision in &facts.person_revisions {
        let Some(after_person) = after.person(revision.person_id) else {
            continue;
        };
        let before_person = before.person(revision.person_id);
        if before_person != Some(after_person) {
            let kind = if facts.change_kind == WirePersonChangeKindV1::PersonChangeKindSourceRemoved
                && after_person.lifecycle == PersonLifecycleV1::Archived
            {
                WirePersonChangeKindV1::PersonChangeKindArchived
            } else {
                facts.change_kind
            };
            pending.push(PendingEventV1::Person(person_event(
                command_message_id,
                after_person,
                kind,
            )));
            if before_person.and_then(|person| person.owner_profile.as_ref())
                != after_person.owner_profile.as_ref()
                && let Some(profile) = after_person.owner_profile.as_ref()
            {
                pending.push(PendingEventV1::Profile(profile_event(
                    command_message_id,
                    after_person,
                    profile,
                )));
            }
        }
    }
    if facts.emit_source_diff {
        source_diff_events(before, after, facts, command_message_id, &mut pending);
    }
    if let Some(lineage) = after.lineage().nth(before.lineage().count()) {
        pending.push(PendingEventV1::Lineage(lineage_event(
            command_message_id,
            lineage,
            logical_owner_id,
        )));
    }
    pending.sort_by_key(PendingEventV1::sort_key);
    let mut records = Vec::with_capacity(pending.len() + facts.review_candidates.len());
    let mut order_keys = Vec::with_capacity(records.capacity());
    for mut event in pending {
        event.set_resulting_owner_revision(resulting_owner_revision);
        order_keys.push(event.semantic_order_key());
        let (message_id, partition_key, occurred_at, payload) = event.into_wire();
        let record = build_persons_owner_event_outbox_record_v1(
            command_message_id,
            correlation_id,
            message_id,
            partition_key,
            timestamp(occurred_at),
            payload,
            context,
        )
        .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
        records.push(envelope_record(&record));
    }
    let mut candidates = facts.review_candidates.clone();
    candidates.sort_by_key(|candidate| candidate.candidate_id);
    for candidate in candidates {
        let event = review_event(
            command_message_id,
            logical_owner_id,
            candidate,
            resulting_owner_revision,
        );
        let mut order_key = vec![5];
        order_key.extend_from_slice(&event.candidate_id);
        order_keys.push(order_key);
        let message_id = id16_array(&event.event_id)?;
        let partition_key = persons_owner_partition_id_v1(logical_owner_id)
            .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
        let record = build_persons_review_candidate_outbox_record_v1(
            command_message_id,
            correlation_id,
            message_id,
            partition_key,
            event,
            context,
        )
        .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
        records.push(envelope_record(&record));
    }
    Ok(BuiltEventsV1 {
        records,
        order_keys,
    })
}

struct BuiltEventsV1 {
    records: Vec<PersonsEnvelopeRecordV1>,
    order_keys: Vec<Vec<u8>>,
}

enum PendingEventV1 {
    Person(PersonChangedEventV1),
    Profile(PersonProfileChangedEventV1),
    Source(PersonSourceLinkChangedEventV1),
    Lineage(PersonLineageChangedEventV1),
}

impl PendingEventV1 {
    fn sort_key(&self) -> (u8, Vec<u8>, Vec<u8>) {
        match self {
            Self::Person(value) => (1, value.person_id.clone(), Vec::new()),
            Self::Profile(value) => (2, value.person_id.clone(), Vec::new()),
            Self::Source(value) => (3, value.person_id.clone(), value.source_link_id.clone()),
            Self::Lineage(value) => (
                4,
                value.source_person_id.clone(),
                value.target_person_id.clone(),
            ),
        }
    }

    fn semantic_order_key(&self) -> Vec<u8> {
        let (kind, first, second) = self.sort_key();
        let mut key = Vec::with_capacity(1 + first.len() + second.len());
        key.push(kind);
        key.extend_from_slice(&first);
        key.extend_from_slice(&second);
        key
    }

    fn set_resulting_owner_revision(&mut self, revision: u64) {
        match self {
            Self::Person(value) => value.resulting_owner_revision = revision,
            Self::Profile(value) => value.resulting_owner_revision = revision,
            Self::Source(value) => value.resulting_owner_revision = revision,
            Self::Lineage(value) => value.resulting_owner_revision = revision,
        }
    }

    fn into_wire(self) -> ([u8; 16], [u8; 16], TimestampV1, PersonsOwnerEventV1) {
        match self {
            Self::Person(value) => {
                let message = id16_array(&value.event_id).expect("validated event id");
                let partition = id16_array(&value.person_id).expect("validated Person id");
                let occurred = core_timestamp(value.changed_at.as_ref().expect("timestamp"));
                (
                    message,
                    partition,
                    occurred,
                    PersonsOwnerEventV1 {
                        event: Some(Event::PersonChanged(value)),
                    },
                )
            }
            Self::Profile(value) => {
                let message = id16_array(&value.event_id).expect("validated event id");
                let partition = id16_array(&value.person_id).expect("validated Person id");
                let occurred = core_timestamp(value.changed_at.as_ref().expect("timestamp"));
                (
                    message,
                    partition,
                    occurred,
                    PersonsOwnerEventV1 {
                        event: Some(Event::ProfileChanged(value)),
                    },
                )
            }
            Self::Source(value) => {
                let message = id16_array(&value.event_id).expect("validated event id");
                let partition = id16_array(&value.person_id).expect("validated Person id");
                let occurred = core_timestamp(value.changed_at.as_ref().expect("timestamp"));
                (
                    message,
                    partition,
                    occurred,
                    PersonsOwnerEventV1 {
                        event: Some(Event::SourceLinkChanged(value)),
                    },
                )
            }
            Self::Lineage(value) => {
                let message = id16_array(&value.event_id).expect("validated event id");
                let partition = id16_array(&value.source_person_id).expect("validated Person id");
                let occurred = core_timestamp(value.changed_at.as_ref().expect("timestamp"));
                (
                    message,
                    partition,
                    occurred,
                    PersonsOwnerEventV1 {
                        event: Some(Event::LineageChanged(value)),
                    },
                )
            }
        }
    }
}

fn person_event(
    command_message_id: [u8; 16],
    person: &PersonV1,
    kind: WirePersonChangeKindV1,
) -> PersonChangedEventV1 {
    let event_id = persons_deterministic_public_id_v1(
        b"persons-person-changed-v1",
        &command_message_id,
        &person.person_id.0,
    );
    PersonChangedEventV1 {
        event_id: event_id.to_vec(),
        person_id: person.person_id.0.to_vec(),
        logical_owner_id: person.logical_owner_id.clone(),
        lifecycle: wire_lifecycle(person.lifecycle) as i32,
        person_revision: person.revision,
        change_kind: kind as i32,
        changed_at: Some(wire_timestamp(person.updated_at)),
        resulting_owner_revision: 0,
    }
}

fn profile_event(
    command_message_id: [u8; 16],
    person: &PersonV1,
    profile: &OwnerProfileV1,
) -> PersonProfileChangedEventV1 {
    let event_id = persons_deterministic_public_id_v1(
        b"persons-profile-changed-v1",
        &command_message_id,
        &person.person_id.0,
    );
    PersonProfileChangedEventV1 {
        event_id: event_id.to_vec(),
        person_id: person.person_id.0.to_vec(),
        logical_owner_id: person.logical_owner_id.clone(),
        person_revision: person.revision,
        profile_digest: profile_digest(profile).to_vec(),
        changed_at: Some(wire_timestamp(person.updated_at)),
        resulting_owner_revision: 0,
    }
}

fn source_diff_events(
    before: &PersonsStateV1,
    after: &PersonsStateV1,
    facts: &AppliedFactsV1,
    command_message_id: [u8; 16],
    pending: &mut Vec<PendingEventV1>,
) {
    let before_links = source_links(before);
    let after_links = source_links(after);
    let keys = before_links
        .keys()
        .chain(after_links.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for key in keys {
        let before_link = before_links.get(&key);
        let after_link = after_links.get(&key);
        if before_link == after_link {
            continue;
        }
        let (person_id, from_person_id, to_person_id, provenance, change_kind) =
            match (before_link, after_link) {
                (None, None) => continue,
                (None, Some((person_id, link))) => (
                    *person_id,
                    None,
                    Some(*person_id),
                    link.provenance,
                    WireSourceLinkChangeKindV1::SourceLinkChangeKindObserved,
                ),
                (Some((_, _)), None) => {
                    let Some(provenance) = facts.source_provenance else {
                        continue;
                    };
                    let Some((person_id, _)) = before_link else {
                        continue;
                    };
                    (
                        *person_id,
                        Some(*person_id),
                        None,
                        provenance,
                        WireSourceLinkChangeKindV1::SourceLinkChangeKindRemoved,
                    )
                }
                (Some((before_person, _)), Some((after_person, link)))
                    if before_person == after_person =>
                {
                    (
                        *after_person,
                        Some(*before_person),
                        Some(*after_person),
                        link.provenance,
                        WireSourceLinkChangeKindV1::SourceLinkChangeKindUpdated,
                    )
                }
                (Some((before_person, _)), Some((after_person, link))) => (
                    *after_person,
                    Some(*before_person),
                    Some(*after_person),
                    link.provenance,
                    match facts.change_kind {
                        WirePersonChangeKindV1::PersonChangeKindSourceDetached => {
                            WireSourceLinkChangeKindV1::SourceLinkChangeKindDetached
                        }
                        _ => WireSourceLinkChangeKindV1::SourceLinkChangeKindAttached,
                    },
                ),
            };
        let source_link_id = source_link_id(key);
        let decision = facts.decision.as_ref();
        let event_id = persons_deterministic_public_id_v1(
            b"persons-source-link-changed-v1",
            &command_message_id,
            &source_link_id,
        );
        let owner = after
            .person(person_id)
            .or_else(|| before.person(person_id))
            .map(|person| person.logical_owner_id.clone())
            .unwrap_or_default();
        pending.push(PendingEventV1::Source(PersonSourceLinkChangedEventV1 {
            event_id: event_id.to_vec(),
            person_id: person_id.0.to_vec(),
            logical_owner_id: owner,
            source_link_id: source_link_id.to_vec(),
            source: Some(wire_source(key)),
            change_kind: change_kind as i32,
            source_revision: provenance.revision,
            source_digest: provenance.digest.0.to_vec(),
            changed_at: Some(wire_timestamp(
                decision.map_or(provenance.observed_at, |value| value.decided_at),
            )),
            decision_id: decision
                .map(|value| value.decision_id.0.to_vec())
                .unwrap_or_default(),
            decision_revision: decision.map_or(0, |value| value.revision),
            from_person_id: from_person_id
                .map(|value| value.0.to_vec())
                .unwrap_or_default(),
            to_person_id: to_person_id
                .map(|value| value.0.to_vec())
                .unwrap_or_default(),
            resulting_owner_revision: 0,
        }));
    }
}

fn lineage_event(
    command_message_id: [u8; 16],
    lineage: &makosh_persons_core::LineageRecordV1,
    owner: &str,
) -> PersonLineageChangedEventV1 {
    let event_id = persons_deterministic_public_id_v1(
        b"persons-lineage-changed-v1",
        &command_message_id,
        &lineage.decision.decision_id.0,
    );
    PersonLineageChangedEventV1 {
        event_id: event_id.to_vec(),
        logical_owner_id: owner.to_owned(),
        source_person_id: lineage.source_person_id.0.to_vec(),
        target_person_id: lineage.target_person_id.0.to_vec(),
        change_kind: match lineage.change_kind {
            LineageChangeKindV1::Merge => WireLineageChangeKindV1::LineageChangeKindMerged,
            LineageChangeKindV1::Split => WireLineageChangeKindV1::LineageChangeKindSplit,
        } as i32,
        decision_id: lineage.decision.decision_id.0.to_vec(),
        decision_revision: lineage.decision.revision,
        changed_at: Some(wire_timestamp(lineage.decision.decided_at)),
        resulting_owner_revision: 0,
    }
}

fn review_event(
    command_message_id: [u8; 16],
    owner: &str,
    candidate: ReviewCandidateV1,
    resulting_owner_revision: u64,
) -> PersonReviewCandidateRaisedEventV1 {
    let event_id = persons_deterministic_public_id_v1(
        b"persons-review-candidate-v1",
        &command_message_id,
        &candidate.candidate_id.0,
    );
    PersonReviewCandidateRaisedEventV1 {
        event_id: event_id.to_vec(),
        candidate_id: candidate.candidate_id.0.to_vec(),
        logical_owner_id: owner.to_owned(),
        first_person_id: candidate.first_person_id.0.to_vec(),
        second_person_id: candidate.second_person_id.0.to_vec(),
        first_source: Some(wire_source(candidate.first_source)),
        second_source: Some(wire_source(candidate.second_source)),
        match_kind: match candidate.match_kind {
            IdentityMatchKindV1::NormalizedEmail => {
                WireIdentityMatchKindV1::IdentityMatchKindNormalizedEmail
            }
            IdentityMatchKindV1::NormalizedPhone => {
                WireIdentityMatchKindV1::IdentityMatchKindNormalizedPhone
            }
        } as i32,
        observed_at: Some(wire_timestamp(candidate.observed_at)),
        resulting_owner_revision,
    }
}

fn source_links(state: &PersonsStateV1) -> BTreeMap<SourceLinkKeyV1, (PersonIdV1, SourceLinkV1)> {
    state
        .persons()
        .flat_map(|person| {
            person
                .source_links
                .iter()
                .map(|(key, link)| (*key, (person.person_id, link.clone())))
        })
        .collect()
}

fn revision(state: &PersonsStateV1, person_id: PersonIdV1) -> Option<PersonRevisionV1> {
    state.person(person_id).map(|person| PersonRevisionV1 {
        person_id,
        revision: person.revision,
    })
}

fn exact_contract(
    actual: Option<&ContractRefV1>,
    expected: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Result<(), PersonsCommandExecutionErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        Err(PersonsCommandExecutionErrorV1::InvalidEnvelope)
    } else {
        Ok(())
    }
}

fn reject_code(error: PersonsTransitionErrorV1) -> wire::PersonRejectCodeV1 {
    use wire::PersonRejectCodeV1 as Reject;
    match error {
        PersonsTransitionErrorV1::PersonNotFound | PersonsTransitionErrorV1::SourceNotFound => {
            Reject::PersonRejectCodeNotFound
        }
        PersonsTransitionErrorV1::StaleSourceRevision
        | PersonsTransitionErrorV1::ExpectedRevisionConflict
        | PersonsTransitionErrorV1::ExpectedSourceRevisionConflict => {
            Reject::PersonRejectCodeStaleRevision
        }
        PersonsTransitionErrorV1::DecisionRequired => Reject::PersonRejectCodeDecisionRequired,
        PersonsTransitionErrorV1::SourceOwnerConflict => {
            Reject::PersonRejectCodeSourceAlreadyAttached
        }
        PersonsTransitionErrorV1::LineageConflict
        | PersonsTransitionErrorV1::PersonMerged
        | PersonsTransitionErrorV1::EmptySplitSelection
        | PersonsTransitionErrorV1::ProfileFactUnavailable => {
            Reject::PersonRejectCodeLineageConflict
        }
        PersonsTransitionErrorV1::OwnerMismatch => Reject::PersonRejectCodePolicy,
        PersonsTransitionErrorV1::PersonAlreadyExists
        | PersonsTransitionErrorV1::SourceRevisionConflict
        | PersonsTransitionErrorV1::ActionDigestMismatch
        | PersonsTransitionErrorV1::DecisionReuseConflict
        | PersonsTransitionErrorV1::DecisionTimestampRegression
        | PersonsTransitionErrorV1::DuplicateSplitSelection
        | PersonsTransitionErrorV1::SamePerson => Reject::PersonRejectCodeConflict,
        PersonsTransitionErrorV1::InvalidOwner
        | PersonsTransitionErrorV1::InvalidPublicId
        | PersonsTransitionErrorV1::InvalidPersonId
        | PersonsTransitionErrorV1::InvalidDigest
        | PersonsTransitionErrorV1::InvalidTimestamp
        | PersonsTransitionErrorV1::InvalidRevision
        | PersonsTransitionErrorV1::InvalidProfile
        | PersonsTransitionErrorV1::InvalidSourceClaims
        | PersonsTransitionErrorV1::InvalidEmail
        | PersonsTransitionErrorV1::InvalidPhone
        | PersonsTransitionErrorV1::ReviewCandidateLimitExceeded
        | PersonsTransitionErrorV1::InvalidSnapshot => Reject::PersonRejectCodeInvalidRequest,
    }
}

fn profile_digest(profile: &OwnerProfileV1) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"persons-owner-profile-v1");
    for value in [
        profile.display_name.as_deref(),
        profile.given_name.as_deref(),
        profile.family_name.as_deref(),
    ] {
        match value {
            Some(value) => {
                hasher.update([1]);
                hasher.update((value.len() as u64).to_be_bytes());
                hasher.update(value.as_bytes());
            }
            None => hasher.update([0]),
        }
    }
    for values in [&profile.emails, &profile.phones] {
        hasher.update((values.len() as u64).to_be_bytes());
        for value in values {
            hasher.update((value.len() as u64).to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    hasher.finalize().into()
}

fn source_link_id(key: SourceLinkKeyV1) -> [u8; 16] {
    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(&key.integration_public_id.0);
    bytes.extend_from_slice(&key.account_public_id.0);
    bytes.extend_from_slice(&key.provider_source_contact_public_id.0);
    persons_deterministic_public_id_v1(b"persons-source-link-v1", &bytes, b"public")
}

fn wire_source(key: SourceLinkKeyV1) -> wire::ProviderSourceIdentityV1 {
    wire::ProviderSourceIdentityV1 {
        integration_public_id: key.integration_public_id.0.to_vec(),
        account_public_id: key.account_public_id.0.to_vec(),
        provider_source_contact_public_id: key.provider_source_contact_public_id.0.to_vec(),
    }
}

fn wire_lifecycle(value: PersonLifecycleV1) -> WirePersonLifecycleV1 {
    match value {
        PersonLifecycleV1::Provisional => WirePersonLifecycleV1::PersonLifecycleProvisional,
        PersonLifecycleV1::Active => WirePersonLifecycleV1::PersonLifecycleActive,
        PersonLifecycleV1::Merged => WirePersonLifecycleV1::PersonLifecycleMerged,
        PersonLifecycleV1::Archived => WirePersonLifecycleV1::PersonLifecycleArchived,
    }
}

fn wire_timestamp(value: TimestampV1) -> WireTimestampV1 {
    WireTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn timestamp(value: TimestampV1) -> Timestamp {
    Timestamp {
        seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn core_timestamp(value: &WireTimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn output_context(runtime: &PersonsCommandRuntimeContextV1) -> PersonsEnvelopeContextV1 {
    PersonsEnvelopeContextV1 {
        module_id: "makosh-persons-runtime".to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.clone(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn envelope_record(record: &OutboxRecordV1) -> PersonsEnvelopeRecordV1 {
    PersonsEnvelopeRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn id16_array(value: &[u8]) -> Result<[u8; 16], PersonsPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(PersonsPersistenceErrorV1::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::build_persons_command_outbox_record_v1;
    use makosh_persons_api::wire::{
        ManualCreatePersonCommandV1, PersonCommandRejectedV1, PersonProfileV1, TimestampV1,
        persons_command_v1::Command,
    };
    use makosh_persons_core::{
        AttachSourceActionV1, DecisionProvenanceV1, DetachSourceActionV1, DigestV1,
        MergePersonsActionV1, PersonLifecycleV1, PublicIdV1, SourceClaimsV1, SourceLinkKeyV1,
        SourceObservationV1, SourceProvenanceV1, SplitPersonActionV1, SplitProfileFactKindV1,
        SplitSourceSelectionV1, attach_source_action_digest_v1, detach_source_action_digest_v1,
        merge_persons_action_digest_v1, split_person_action_digest_v1, update_owner_profile_v1,
    };

    #[test]
    fn structurally_exact_expired_command_reaches_replay_resolution() {
        let context = test_context();
        let record = build_persons_command_outbox_record_v1(
            PersonsCommandV1 {
                command: Some(Command::ManualCreate(ManualCreatePersonCommandV1 {
                    command_id: vec![1; 16],
                    person_id: vec![2; 16],
                    logical_owner_id: "owner-a".to_owned(),
                    owner_profile: Some(PersonProfileV1 {
                        display_name: Some("Expired replay".to_owned()),
                        ..Default::default()
                    }),
                    created_at: Some(TimestampV1 {
                        unix_seconds: 1_800_000_100,
                        nanos: 0,
                    }),
                })),
            },
            1_800_000_200,
            &context,
        )
        .expect("command envelope");
        let runtime = PersonsCommandRuntimeContextV1 {
            logical_owner_id: "owner-a".to_owned(),
            runtime_instance_id: context.runtime_instance_id,
            runtime_generation: context.runtime_generation,
            now_unix_millis: 1_800_000_201_000,
        };
        let validated = validate_command_record_v1(&record, &runtime)
            .expect("expired structurally exact command reaches persistence replay resolution");
        assert!(validated.expired);
    }

    #[test]
    fn manual_create_emits_sanitized_person_and_digest_events() {
        let mut state = PersonsStateV1::default();
        let context = PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_001,
            recorded_at_nanos: 0,
        };
        let command = decode_typed_command_v1(
            PersonsCommandV1 {
                command: Some(Command::ManualCreate(ManualCreatePersonCommandV1 {
                    command_id: vec![1; 16],
                    person_id: vec![2; 16],
                    logical_owner_id: "owner-a".to_owned(),
                    owner_profile: Some(PersonProfileV1 {
                        display_name: Some("Ada".to_owned()),
                        ..Default::default()
                    }),
                    created_at: Some(TimestampV1 {
                        unix_seconds: 1_800_000_000,
                        nanos: 0,
                    }),
                })),
            },
            "owner-a",
            [1; 16],
        )
        .expect("decode");
        let commit = build_commit_v1(
            &mut state,
            command,
            [1; 16],
            [1; 16],
            [3; 16],
            "owner-a",
            1,
            1_800_000_001_000,
            &context,
        )
        .expect("commit");
        assert_eq!(commit.owner_events.len(), 2);
        assert_event_owner_revision(&commit, 1);
        let bytes = commit
            .owner_events
            .iter()
            .flat_map(|event| event.envelope_bytes.iter().copied())
            .collect::<Vec<_>>();
        assert!(!bytes.windows(3).any(|window| window == b"Ada"));
    }

    #[test]
    fn core_rejection_emits_only_bounded_terminal_and_preserves_state() {
        let mut state = PersonsStateV1::default();
        let before = state.clone();
        let command = DecodedPersonsCommandV1::OwnerProfileUpdate {
            logical_owner_id: "owner-a".to_owned(),
            person_id: PersonIdV1([2; 16]),
            expected_person_revision: 1,
            owner_profile: OwnerProfileV1 {
                display_name: Some("secret-profile".to_owned()),
                given_name: None,
                family_name: None,
                emails: vec![],
                phones: vec![],
            },
            updated_at: makosh_persons_core::TimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 0,
            },
        };
        let context = PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".to_owned(),
            runtime_instance_id: "owner-a".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_001,
            recorded_at_nanos: 0,
        };
        let commit = build_commit_v1(
            &mut state,
            command,
            [1; 16],
            [1; 16],
            [3; 16],
            "owner-a",
            1,
            1_800_000_001_000,
            &context,
        )
        .expect("bounded rejection");
        assert_eq!(state, before);
        assert!(commit.owner_events.is_empty());
        assert!(
            !commit
                .terminal_result
                .envelope_bytes
                .windows(14)
                .any(|window| window == b"secret-profile")
        );
    }

    #[test]
    fn attach_and_detach_publish_exact_two_person_revisions_and_unambiguous_move() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, observation(10)).expect("first source");
        let second = observe_source_v1(&mut state, observation(20)).expect("second source");
        let first_id = first.person_id();
        let second_id = second.person_id();
        let source = observation(10).key;
        let attach = AttachSourceActionV1 {
            logical_owner_id: "owner-a".to_owned(),
            from_person_id: first_id,
            expected_from_person_revision: 1,
            to_person_id: second_id,
            expected_to_person_revision: 1,
            source,
            expected_source_revision: 1,
        };
        let attach_decision = decision(
            30,
            attach_source_action_digest_v1(&attach).expect("attach digest"),
            1_800_000_030,
        );
        let attach_commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::ConfirmedAttach(attach, attach_decision.clone()),
            [31; 16],
            [32; 16],
            [33; 16],
            "owner-a",
            1,
            1_800_000_031_000,
            &test_context(),
        )
        .expect("attach commit");
        assert_event_owner_revision(&attach_commit, 1);
        assert_terminal_revisions(&attach_commit, &[(first_id, 2), (second_id, 2)]);
        assert_source_move(
            &attach_commit,
            WireSourceLinkChangeKindV1::SourceLinkChangeKindAttached,
            first_id,
            second_id,
            &attach_decision,
        );

        let detach = DetachSourceActionV1 {
            logical_owner_id: "owner-a".to_owned(),
            person_id: second_id,
            expected_person_revision: 2,
            source,
            expected_source_revision: 1,
            expected_detached_person_revision: 2,
        };
        let detach_decision = decision(
            40,
            detach_source_action_digest_v1(&detach).expect("detach digest"),
            1_800_000_040,
        );
        let detach_commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::ConfirmedDetach(detach, detach_decision.clone()),
            [41; 16],
            [42; 16],
            [43; 16],
            "owner-a",
            2,
            1_800_000_041_000,
            &test_context(),
        )
        .expect("detach commit");
        assert_event_owner_revision(&detach_commit, 2);
        assert_terminal_revisions(&detach_commit, &[(first_id, 3), (second_id, 3)]);
        assert_source_move(
            &detach_commit,
            WireSourceLinkChangeKindV1::SourceLinkChangeKindDetached,
            second_id,
            first_id,
            &detach_decision,
        );
    }

    #[test]
    fn source_lifecycle_emits_public_events_candidates_and_idempotent_noop() {
        let mut state = PersonsStateV1::default();
        let mut first = observation(50);
        first.claims.emails = vec!["shared@example.test".to_owned()];
        let first_commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::SourceObserve(first.clone()),
            [50; 16],
            [51; 16],
            [52; 16],
            "owner-a",
            1,
            1_800_000_051_000,
            &test_context(),
        )
        .expect("source create");
        assert_eq!(source_events(&first_commit).len(), 1);
        assert_eq!(review_events(&first_commit), 0);
        let first_person = state.source_owner(first.key).expect("first owner");

        let replay = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::SourceObserve(first.clone()),
            [53; 16],
            [54; 16],
            [55; 16],
            "owner-a",
            2,
            1_800_000_052_000,
            &test_context(),
        )
        .expect("exact source replay");
        assert!(replay.owner_events.is_empty());

        let mut second = observation(60);
        second.claims.emails = vec!["shared@example.test".to_owned()];
        let candidate = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::SourceObserve(second.clone()),
            [60; 16],
            [61; 16],
            [62; 16],
            "owner-a",
            3,
            1_800_000_061_000,
            &test_context(),
        )
        .expect("matching source");
        assert_eq!(review_events(&candidate), 1);
        assert_event_owner_revision(&candidate, 3);
        let second_person = state.source_owner(second.key).expect("second owner");
        assert_ne!(
            first_person, second_person,
            "matching facts must not silently merge"
        );

        let mut updated = first.clone();
        updated.provenance.revision = 2;
        updated.provenance.digest = DigestV1([63; 32]);
        updated.provenance.observed_at.unix_seconds += 100;
        let updated_commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::SourceUpdate(updated.clone()),
            [64; 16],
            [65; 16],
            [66; 16],
            "owner-a",
            4,
            1_800_000_164_000,
            &test_context(),
        )
        .expect("source update");
        assert_eq!(source_events(&updated_commit).len(), 1);

        let person_revision = state.person(first_person).expect("first Person").revision;
        update_owner_profile_v1(
            &mut state,
            "owner-a",
            first_person,
            person_revision,
            OwnerProfileV1 {
                display_name: Some("Owner retained".to_owned()),
                given_name: None,
                family_name: None,
                emails: Vec::new(),
                phones: Vec::new(),
            },
            makosh_persons_core::TimestampV1 {
                unix_seconds: 1_800_000_200,
                nanos: 0,
            },
        )
        .expect("owner profile");
        let removed = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::SourceRemove {
                logical_owner_id: "owner-a".to_owned(),
                source: first.key,
                provenance: SourceProvenanceV1 {
                    revision: 3,
                    digest: DigestV1([67; 32]),
                    observed_at: makosh_persons_core::TimestampV1 {
                        unix_seconds: 1_800_000_300,
                        nanos: 0,
                    },
                },
            },
            [68; 16],
            [69; 16],
            [70; 16],
            "owner-a",
            5,
            1_800_000_301_000,
            &test_context(),
        )
        .expect("source remove");
        assert_eq!(source_events(&removed).len(), 1);
        let retained = state.person(first_person).expect("retained Person");
        assert_eq!(retained.lifecycle, PersonLifecycleV1::Active);
        assert!(retained.owner_profile.is_some());
    }

    #[test]
    fn merge_and_selective_split_emit_exact_lineage_and_person_revisions() {
        let mut state = PersonsStateV1::default();
        let source = observe_source_v1(&mut state, observation(70)).expect("source");
        let target = observe_source_v1(&mut state, observation(80)).expect("target");
        let source_id = source.person_id();
        let target_id = target.person_id();
        update_owner_profile_v1(
            &mut state,
            "owner-a",
            source_id,
            1,
            OwnerProfileV1 {
                display_name: Some("Selected".to_owned()),
                given_name: Some("Not selected".to_owned()),
                family_name: None,
                emails: Vec::new(),
                phones: Vec::new(),
            },
            makosh_persons_core::TimestampV1 {
                unix_seconds: 1_800_000_090,
                nanos: 0,
            },
        )
        .expect("source profile");
        let merge = MergePersonsActionV1 {
            logical_owner_id: "owner-a".to_owned(),
            source_person_id: source_id,
            expected_source_person_revision: 2,
            target_person_id: target_id,
            expected_target_person_revision: 1,
        };
        let merge_decision = decision(
            90,
            merge_persons_action_digest_v1(&merge).expect("merge digest"),
            1_800_000_100,
        );
        let merge_commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::ConfirmedMerge(merge, merge_decision.clone()),
            [91; 16],
            [92; 16],
            [93; 16],
            "owner-a",
            1,
            1_800_000_101_000,
            &test_context(),
        )
        .expect("merge");
        assert_event_owner_revision(&merge_commit, 1);
        assert_terminal_revisions(&merge_commit, &[(source_id, 3), (target_id, 2)]);
        assert_lineage_event(&merge_commit, &merge_decision);

        let split = SplitPersonActionV1 {
            logical_owner_id: "owner-a".to_owned(),
            merged_person_id: source_id,
            expected_merged_person_revision: 3,
            target_person_id: target_id,
            expected_target_person_revision: 2,
            source_selection: vec![SplitSourceSelectionV1 {
                source: observation(70).key,
                expected_source_revision: 1,
            }],
            profile_fact_selection: vec![SplitProfileFactKindV1::DisplayName],
        };
        let split_decision = decision(
            100,
            split_person_action_digest_v1(&split).expect("split digest"),
            1_800_000_110,
        );
        let split_commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::ConfirmedSplit(split, split_decision.clone()),
            [101; 16],
            [102; 16],
            [103; 16],
            "owner-a",
            2,
            1_800_000_111_000,
            &test_context(),
        )
        .expect("selective split");
        assert_event_owner_revision(&split_commit, 2);
        assert_terminal_revisions(&split_commit, &[(source_id, 4), (target_id, 3)]);
        assert_lineage_event(&split_commit, &split_decision);
        let restored = state.person(source_id).expect("restored Person");
        let profile = restored.owner_profile.as_ref().expect("selected profile");
        assert_eq!(profile.display_name.as_deref(), Some("Selected"));
        assert_eq!(profile.given_name, None);
    }

    #[test]
    fn candidate_overflow_is_a_bounded_terminal_rejection_without_events_or_mutation() {
        let mut state = PersonsStateV1::default();
        for seed in 1..=129_u8 {
            let mut source = observation(seed);
            source.claims.emails = vec!["shared@example.test".to_owned()];
            observe_source_v1(&mut state, source).expect("candidate fixture");
        }
        let before = state.clone();
        let mut overflow = observation(200);
        overflow.claims.emails = vec!["shared@example.test".to_owned()];
        let commit = build_commit_v1(
            &mut state,
            DecodedPersonsCommandV1::SourceObserve(overflow),
            [201; 16],
            [201; 16],
            [202; 16],
            "owner-a",
            130,
            1_800_000_400_000,
            &test_context(),
        )
        .expect("bounded rejection commit");
        assert_eq!(state, before);
        assert!(commit.owner_events.is_empty());
        assert!(commit.terminal_result.envelope_bytes.len() < 1_024);
        let envelope =
            decode_envelope_v1(&commit.terminal_result.envelope_bytes).expect("rejection envelope");
        let rejected = PersonCommandRejectedV1::decode(envelope.payload.as_slice())
            .expect("rejection payload");
        assert_eq!(
            rejected.code,
            wire::PersonRejectCodeV1::PersonRejectCodeInvalidRequest as i32
        );
    }

    fn assert_terminal_revisions(commit: &PersonsCommandCommitV1, expected: &[(PersonIdV1, u64)]) {
        let envelope =
            decode_envelope_v1(&commit.terminal_result.envelope_bytes).expect("terminal envelope");
        let payload = PersonCommandSucceededV1::decode(envelope.payload.as_slice())
            .expect("terminal payload");
        let actual = payload
            .resulting_person_revisions
            .into_iter()
            .map(|revision| {
                (
                    PersonIdV1(revision.person_id.try_into().expect("Person ID")),
                    revision.person_revision,
                )
            })
            .collect::<Vec<_>>();
        let mut expected = expected.to_vec();
        expected.sort_by_key(|(person_id, _)| person_id.0);
        assert_eq!(actual, expected);
        let person_event_count = commit
            .owner_events
            .iter()
            .filter(|record| {
                let envelope = decode_envelope_v1(&record.envelope_bytes).expect("owner envelope");
                PersonsOwnerEventV1::decode(envelope.payload.as_slice())
                    .ok()
                    .is_some_and(|payload| matches!(payload.event, Some(Event::PersonChanged(_))))
            })
            .count();
        assert_eq!(person_event_count, 2);
    }

    fn assert_event_owner_revision(commit: &PersonsCommandCommitV1, expected: u64) {
        assert_eq!(
            commit.owner_events.len(),
            commit.owner_event_order_keys.len()
        );
        assert!(
            commit
                .owner_event_order_keys
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
        for record in &commit.owner_events {
            let envelope = decode_envelope_v1(&record.envelope_bytes).expect("event envelope");
            let contract = envelope.contract.as_ref().expect("event contract");
            let actual = if contract.name == "persons_review_candidate_raised" {
                PersonReviewCandidateRaisedEventV1::decode(envelope.payload.as_slice())
                    .expect("Review event")
                    .resulting_owner_revision
            } else {
                let payload =
                    PersonsOwnerEventV1::decode(envelope.payload.as_slice()).expect("owner event");
                match payload.event.expect("typed owner event") {
                    Event::PersonChanged(value) => value.resulting_owner_revision,
                    Event::ProfileChanged(value) => value.resulting_owner_revision,
                    Event::SourceLinkChanged(value) => value.resulting_owner_revision,
                    Event::LineageChanged(value) => value.resulting_owner_revision,
                }
            };
            assert_eq!(actual, expected);
        }
    }

    fn assert_source_move(
        commit: &PersonsCommandCommitV1,
        expected_kind: WireSourceLinkChangeKindV1,
        expected_from: PersonIdV1,
        expected_to: PersonIdV1,
        decision: &DecisionProvenanceV1,
    ) {
        let source_events = commit
            .owner_events
            .iter()
            .filter_map(|record| {
                let envelope = decode_envelope_v1(&record.envelope_bytes).ok()?;
                let payload = PersonsOwnerEventV1::decode(envelope.payload.as_slice()).ok()?;
                match payload.event {
                    Some(Event::SourceLinkChanged(event)) => Some(event),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(source_events.len(), 1);
        let event = &source_events[0];
        assert_eq!(event.change_kind, expected_kind as i32);
        assert_eq!(event.from_person_id, expected_from.0);
        assert_eq!(event.to_person_id, expected_to.0);
        assert_eq!(event.person_id, expected_to.0);
        assert_eq!(event.decision_id, decision.decision_id.0);
        assert_eq!(event.decision_revision, decision.revision);
        let source = event.source.as_ref().expect("public source tuple");
        assert_eq!(source.integration_public_id.len(), 16);
        assert_eq!(source.account_public_id.len(), 16);
        assert_eq!(source.provider_source_contact_public_id.len(), 16);
    }

    fn source_events(commit: &PersonsCommandCommitV1) -> Vec<PersonSourceLinkChangedEventV1> {
        commit
            .owner_events
            .iter()
            .filter_map(|record| {
                let envelope = decode_envelope_v1(&record.envelope_bytes).ok()?;
                let payload = PersonsOwnerEventV1::decode(envelope.payload.as_slice()).ok()?;
                match payload.event {
                    Some(Event::SourceLinkChanged(event)) => Some(event),
                    _ => None,
                }
            })
            .collect()
    }

    fn review_events(commit: &PersonsCommandCommitV1) -> usize {
        commit
            .owner_events
            .iter()
            .filter(|record| {
                decode_envelope_v1(&record.envelope_bytes)
                    .ok()
                    .and_then(|envelope| envelope.contract)
                    .is_some_and(|contract| contract.name == "persons_review_candidate_raised")
            })
            .count()
    }

    fn assert_lineage_event(commit: &PersonsCommandCommitV1, decision: &DecisionProvenanceV1) {
        let events = commit
            .owner_events
            .iter()
            .filter_map(|record| {
                let envelope = decode_envelope_v1(&record.envelope_bytes).ok()?;
                let payload = PersonsOwnerEventV1::decode(envelope.payload.as_slice()).ok()?;
                match payload.event {
                    Some(Event::LineageChanged(event)) => Some(event),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].decision_id, decision.decision_id.0);
        assert_eq!(events[0].decision_revision, decision.revision);
    }

    fn observation(seed: u8) -> SourceObservationV1 {
        SourceObservationV1 {
            logical_owner_id: "owner-a".to_owned(),
            key: SourceLinkKeyV1 {
                integration_public_id: PublicIdV1([1; 16]),
                account_public_id: PublicIdV1([seed; 16]),
                provider_source_contact_public_id: PublicIdV1([seed.wrapping_add(1); 16]),
            },
            claims: SourceClaimsV1 {
                display_name: Some(format!("Person {seed}")),
                emails: Vec::new(),
                phones: Vec::new(),
            },
            provenance: SourceProvenanceV1 {
                revision: 1,
                digest: DigestV1([seed.wrapping_add(2); 32]),
                observed_at: makosh_persons_core::TimestampV1 {
                    unix_seconds: 1_800_000_000 + i64::from(seed),
                    nanos: 0,
                },
            },
        }
    }

    fn decision(seed: u8, digest: DigestV1, unix_seconds: i64) -> DecisionProvenanceV1 {
        DecisionProvenanceV1 {
            decision_id: PublicIdV1([seed; 16]),
            review_id: PublicIdV1([seed.wrapping_add(1); 16]),
            revision: 1,
            decided_by_owner_device_id: PublicIdV1([seed.wrapping_add(2); 16]),
            decided_at: makosh_persons_core::TimestampV1 {
                unix_seconds,
                nanos: 0,
            },
            approved_action_digest: digest,
        }
    }

    fn test_context() -> PersonsEnvelopeContextV1 {
        PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".to_owned(),
            runtime_instance_id: "persons-runtime1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_100,
            recorded_at_nanos: 0,
        }
    }
}
