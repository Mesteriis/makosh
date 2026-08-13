use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_persons_api::{
    PERSONS_COMMAND_CAPABILITY_ID_V1, PERSONS_MODULE_ID_V1, persons_command_contract_reference_v1,
    persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1, persons_owner_partition_id_v1,
    wire::{PersonCommandRejectedV1, PersonCommandSucceededV1, PersonsCommandV1},
};
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1,
    review_person_match_candidate_approved_contract_reference_v1,
    wire::PersonMatchCandidateApprovedForPromotionV1,
};
use makosh_review_person_match_candidate_promotion_api::{
    ReviewPersonMatchCandidatePromotionEnvelopeContextV1,
    ReviewPersonMatchCandidatePromotionResultShapeV1,
    build_review_person_match_candidate_promotion_result_outbox_record_v1,
    review_person_match_candidate_promotion_result_id_v1,
    wire::{
        ReviewPersonMatchCandidatePromotionFailureCodeV1,
        ReviewPersonMatchCandidatePromotionOutcomeV1, ReviewPersonMatchCandidatePromotionResultV1,
    },
};
use makosh_reviewed_person_match_candidate_promotion_core::{
    ReviewedPersonMatchCandidatePromotionCoreErrorV1,
    plan_reviewed_person_match_candidate_promotion_v1,
};
use makosh_reviewed_person_match_candidate_promotion_persistence::{
    PersistReviewedPersonMatchApprovalFailureV1, PersistReviewedPersonMatchApprovalV1,
    PersistReviewedPersonMatchTerminalV1, ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    ReviewedPersonMatchCandidatePromotionPersistenceV1,
    ReviewedPersonMatchCandidatePromotionReplayV1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedPersonMatchCandidatePromotionExecutionErrorV1 {
    InvalidContext,
    EventUnavailable,
    InvalidEnvelope,
    InvalidPayload,
    Action(ReviewedPersonMatchCandidatePromotionCoreErrorV1),
    Persistence(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1),
}

pub async fn process_person_match_candidate_approval_v1(
    persistence: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    record: &OutboxRecordV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<
    ReviewedPersonMatchCandidatePromotionReplayV1,
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1,
> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = decode_exact(
        record.exact_bytes(),
        ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope,
    )?;
    let approved: PersonMatchCandidateApprovedForPromotionV1 = decode_exact(
        &envelope.payload,
        ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload,
    )?;
    validate_approval_envelope(&envelope, &approved, context)?;
    let approval_record = stored(record);
    if let Some(replay) = persistence
        .replay_approval_if_completed(&context.logical_owner_id, &approval_record)
        .await
        .map_err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence)?
    {
        return Ok(replay);
    }
    validate_approval_freshness(&envelope, &approved, context)?;
    let plan = match plan_reviewed_person_match_candidate_promotion_v1(&approved) {
        Ok(plan) => plan,
        Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::ActionDigestMismatch) => {
            let review_id = id16(&approved.review_id)?;
            let candidate_id = id16(&approved.candidate_id)?;
            let candidate_digest = id32(&approved.candidate_digest)?;
            let decision_id = id16(&approved.decision_id)?;
            let approved_action_digest = id32(&approved.approved_action_digest)?;
            let result_id = review_person_match_candidate_promotion_result_id_v1(
                *record.message_id(),
                decision_id,
                ReviewPersonMatchCandidatePromotionResultShapeV1::ActionDigestMismatch,
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?;
            let result_record = build_review_person_match_candidate_promotion_result_outbox_record_v1(
                *record.message_id(),
                ReviewPersonMatchCandidatePromotionResultV1 {
                    result_id: result_id.to_vec(),
                    review_id: review_id.to_vec(),
                    candidate_id: candidate_id.to_vec(),
                    decision_id: decision_id.to_vec(),
                    expected_review_revision: approved.decision_revision,
                    outcome: ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed as i32,
                    persons_command_id: None,
                    failure_code: ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeActionDigestMismatch as i32,
                    logical_owner_id: context.logical_owner_id.clone(),
                    completed_at_unix_millis: context.now_unix_millis,
                },
                &result_context(context),
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?;
            return persistence
                .persist_approval_failure_once(&PersistReviewedPersonMatchApprovalFailureV1 {
                    logical_owner_id: context.logical_owner_id.clone(),
                    approval: approval_record,
                    review_id,
                    candidate_id,
                    candidate_digest,
                    decision_id,
                    decision_revision: approved.decision_revision,
                    approved_action_digest,
                    review_result: stored(&result_record),
                    completed_at_unix_millis: context.now_unix_millis,
                })
                .await
                .map_err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence);
        }
        Err(error) => {
            return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Action(error));
        }
    };
    let command = build_persons_command_outbox_record_v1(
        plan.persons_command.clone(),
        context.now_unix_millis + 30_000,
        context,
    )?;
    persistence
        .persist_approval_once(&PersistReviewedPersonMatchApprovalV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            approval: approval_record,
            review_id: plan.review_id,
            candidate_id: plan.candidate_id,
            candidate_digest: id32(&approved.candidate_digest)?,
            decision_id: plan.decision_id,
            decision_revision: plan.decision_revision,
            approved_action_digest: plan.approved_action_digest,
            persons_command_id: plan.persons_command_id,
            persons_command_fingerprint: plan.persons_command_fingerprint,
            persons_command: stored(&command),
            occurred_at_unix_millis: approved.decided_at_unix_millis,
        })
        .await
        .map_err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence)
}

pub async fn process_persons_terminal_v1(
    persistence: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    record: &OutboxRecordV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<
    ReviewedPersonMatchCandidatePromotionReplayV1,
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1,
> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = decode_exact(
        record.exact_bytes(),
        ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope,
    )?;
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let succeeded_contract = persons_command_succeeded_contract_reference_v1();
    let rejected_contract = persons_command_rejected_contract_reference_v1();
    let succeeded = contract.owner == succeeded_contract.owner
        && contract.name == succeeded_contract.name
        && contract.major == succeeded_contract.major
        && contract.revision == succeeded_contract.revision
        && contract.schema_sha256 == succeeded_contract.schema_sha256;
    let rejected = contract.owner == rejected_contract.owner
        && contract.name == rejected_contract.name
        && contract.major == rejected_contract.major
        && contract.revision == rejected_contract.revision
        && contract.schema_sha256 == rejected_contract.schema_sha256;
    if !succeeded && !rejected {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope);
    }
    let (persons_command_id, payload_owner) = if succeeded {
        let payload: PersonCommandSucceededV1 = decode_exact(
            &envelope.payload,
            ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload,
        )?;
        (payload.command_id, payload.logical_owner_id)
    } else {
        let payload: PersonCommandRejectedV1 = decode_exact(
            &envelope.payload,
            ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload,
        )?;
        (payload.command_id, payload.logical_owner_id)
    };
    if payload_owner != context.logical_owner_id {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope);
    }
    let command_id = id16(&persons_command_id)?;
    validate_persons_terminal_envelope(&envelope, command_id, succeeded, context)?;
    let terminal_record = stored(record);
    if let Some(replay) = persistence
        .replay_terminal_if_completed(&context.logical_owner_id, &terminal_record)
        .await
        .map_err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence)?
    {
        return Ok(replay);
    }
    validate_persons_terminal_freshness(&envelope, context)?;
    let correlation = match persistence
        .load_correlation(&context.logical_owner_id, command_id)
        .await
    {
        Ok(value) => value,
        Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::NotFound) => {
            return Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Replayed);
        }
        Err(error) => {
            return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence(error));
        }
    };
    let result_id = review_person_match_candidate_promotion_result_id_v1(
        *record.message_id(),
        correlation.decision_id,
        ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal,
    )
    .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?;
    let payload = ReviewPersonMatchCandidatePromotionResultV1 {
        result_id: result_id.to_vec(),
        review_id: correlation.review_id.to_vec(),
        candidate_id: correlation.candidate_id.to_vec(),
        decision_id: correlation.decision_id.to_vec(),
        expected_review_revision: correlation.decision_revision,
        outcome: if succeeded {
            ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeSucceeded as i32
        } else {
            ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed as i32
        },
        persons_command_id: Some(command_id.to_vec()),
        failure_code: if succeeded {
            ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeUnspecified as i32
        } else {
            ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodePersonsRejected as i32
        },
        logical_owner_id: context.logical_owner_id.clone(),
        completed_at_unix_millis: context.now_unix_millis,
    };
    let result_record = build_review_person_match_candidate_promotion_result_outbox_record_v1(
        *record.message_id(),
        payload,
        &result_context(context),
    )
    .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?;
    persistence
        .persist_terminal_once(&PersistReviewedPersonMatchTerminalV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            persons_result: terminal_record,
            persons_command_id: command_id,
            review_id: correlation.review_id,
            candidate_id: correlation.candidate_id,
            succeeded,
            failure_code: (!succeeded).then_some(3),
            review_result: stored(&result_record),
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence)
}

pub fn build_persons_command_outbox_record_v1(
    payload: PersonsCommandV1,
    deadline_unix_millis: i64,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<OutboxRecordV1, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    validate_context(context)?;
    let (command_id, owner) = persons_command_identity(&payload)?;
    if owner != context.logical_owner_id || deadline_unix_millis <= context.now_unix_millis {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload);
    }
    let payload_bytes = payload.encode_to_vec();
    let fingerprint: [u8; 32] = Sha256::digest(&payload_bytes).into();
    let partition = persons_owner_partition_id_v1(&owner)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?;
    let recorded = millis_timestamp(context.now_unix_millis)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: command_id.to_vec(),
        contract: Some(contract_ref(persons_command_contract_reference_v1())),
        source: Some(SourceRefV1 {
            module_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest16(
                b"reviewed-person-match-runtime-v1",
                context.runtime_instance_id.as_bytes(),
                REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1.as_bytes(),
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(recorded),
        partition_key: partition.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: partition.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1
                .as_bytes()
                .to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1
                .as_bytes()
                .to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: PERSONS_COMMAND_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: fingerprint.to_vec(),
            deadline: Some(millis_timestamp(deadline_unix_millis)?),
            logical_attempt: 1,
        })),
        payload: payload_bytes,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_approval_envelope(
    envelope: &DurableEnvelopeV1,
    approved: &PersonMatchCandidateApprovedForPromotionV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    let expected = review_person_match_candidate_approved_contract_reference_v1();
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let event = match envelope.semantics.as_ref() {
        Some(Semantics::Event(event)) => event,
        _ => return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope),
    };
    let occurred = event
        .occurred_at
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || source.module_id != REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::OwnerDevice as i32
        || actor.actor_id != approved.decided_by_owner_device_id
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.partition_key != approved.review_id
        || envelope.correlation_id != approved.review_id
        || envelope.causation_message_id != approved.decision_id
        || timestamp_millis(occurred)? != approved.decided_at_unix_millis
        || timestamp_millis(recorded)? < approved.decided_at_unix_millis
        || approved.logical_owner_id != context.logical_owner_id
    {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn validate_persons_terminal_envelope(
    envelope: &DurableEnvelopeV1,
    command_id: [u8; 16],
    succeeded: bool,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    let source = envelope
        .source
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let result = match envelope.semantics.as_ref() {
        Some(Semantics::Result(result)) => result,
        _ => return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope),
    };
    let completed = result
        .completed_at
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    let owner_partition = persons_owner_partition_id_v1(&context.logical_owner_id)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?;
    if source.module_id != PERSONS_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != PERSONS_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != PERSONS_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || result.command_id != command_id
        || result.command_message_id != command_id
        || result.outcome
            != if succeeded {
                ResultOutcomeV1::Succeeded as i32
            } else {
                ResultOutcomeV1::Rejected as i32
            }
        || result.execution_attempt != 1
        || envelope.causation_message_id != command_id
        || envelope.correlation_id != owner_partition
        || envelope.partition_key != owner_partition
        || timestamp_millis(recorded)? != timestamp_millis(completed)?
    {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn validate_approval_freshness(
    envelope: &DurableEnvelopeV1,
    approved: &PersonMatchCandidateApprovedForPromotionV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    if timestamp_millis(recorded)? > context.now_unix_millis
        || approved.decided_at_unix_millis > context.now_unix_millis
    {
        Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)
    } else {
        Ok(())
    }
}

fn validate_persons_terminal_freshness(
    envelope: &DurableEnvelopeV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    let completed = match envelope.semantics.as_ref() {
        Some(Semantics::Result(result)) => result.completed_at.as_ref(),
        _ => None,
    }
    .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)?;
    if timestamp_millis(completed)? > context.now_unix_millis {
        Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)
    } else {
        Ok(())
    }
}

fn persons_command_identity(
    payload: &PersonsCommandV1,
) -> Result<([u8; 16], String), ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    use makosh_persons_api::wire::persons_command_v1::Command;
    match payload
        .command
        .as_ref()
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?
    {
        Command::ConfirmedAttach(value) => {
            Ok((id16(&value.command_id)?, value.logical_owner_id.clone()))
        }
        Command::ConfirmedMerge(value) => {
            Ok((id16(&value.command_id)?, value.logical_owner_id.clone()))
        }
        Command::ConfirmedSplit(value) => {
            Ok((id16(&value.command_id)?, value.logical_owner_id.clone()))
        }
        _ => Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload),
    }
}

fn contract_ref(value: makosh_runtime_protocol::v1::ContractReferenceV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: value.owner,
        name: value.name,
        major: value.major,
        revision: value.revision,
        schema_sha256: value.schema_sha256,
    }
}
fn stored(record: &OutboxRecordV1) -> ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
    ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}
fn result_context(
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> ReviewPersonMatchCandidatePromotionEnvelopeContextV1 {
    ReviewPersonMatchCandidatePromotionEnvelopeContextV1 {
        module_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_millis: context.now_unix_millis,
    }
}
fn millis_timestamp(
    value: i64,
) -> Result<Timestamp, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    if value <= 0 {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload);
    }
    Ok(Timestamp {
        seconds: value / 1_000,
        nanos: i32::try_from((value % 1_000) * 1_000_000)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)?,
    })
}
fn timestamp_millis(
    value: &Timestamp,
) -> Result<i64, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    if value.seconds <= 0
        || !(0..1_000_000_000).contains(&value.nanos)
        || value.nanos % 1_000_000 != 0
    {
        return Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload);
    }
    value
        .seconds
        .checked_mul(1_000)
        .and_then(|v| v.checked_add(i64::from(value.nanos / 1_000_000)))
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)
}
fn id16(value: &[u8]) -> Result<[u8; 16], ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|v: &[u8; 16]| v.iter().any(|b| *b != 0))
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)
}
fn id32(value: &[u8]) -> Result<[u8; 32], ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|v: &[u8; 32]| v.iter().any(|b| *b != 0))
        .ok_or(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload)
}
fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut h = Sha256::new();
    for v in [label, first, second] {
        h.update((v.len() as u64).to_be_bytes());
        h.update(v);
    }
    h.finalize()[..16].try_into().expect("SHA-256 prefix")
}
fn validate_context(
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.logical_owner_id.len() > 128
        || !context.logical_owner_id.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
    {
        Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}
const fn outbox_error(
    _: OutboxRecordError,
) -> ReviewedPersonMatchCandidatePromotionExecutionErrorV1 {
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope
}

fn decode_exact<T: Message + Default>(
    bytes: &[u8],
    error: ReviewedPersonMatchCandidatePromotionExecutionErrorV1,
) -> Result<T, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    let value = T::decode(bytes).map_err(|_| error)?;
    if value.encode_to_vec() != bytes {
        return Err(error);
    }
    Ok(value)
}

#[cfg(test)]
mod exact_decode_tests {
    use super::*;
    use makosh_events_protocol::v1::{ResultMetadataV1, durable_envelope_v1::Semantics};
    use makosh_review_person_match_candidate_api::{
        ReviewPersonMatchCandidateEnvelopeContextV1,
        build_review_person_match_candidate_approved_outbox_record_v1,
        wire::{
            AttachPersonSourceReviewActionV1, PersonMatchCandidateApprovedActionV1,
            person_match_candidate_approved_action_v1::Action,
        },
    };

    #[test]
    fn canonical_decode_rejects_unknown_private_extension_bytes() {
        let canonical = DurableEnvelopeV1::default().encode_to_vec();
        let mut extended = canonical.clone();
        extended.extend_from_slice(&[0x98, 0x06, 0x01]);
        assert!(
            decode_exact::<DurableEnvelopeV1>(
                &canonical,
                ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope
            )
            .is_ok()
        );
        assert_eq!(
            decode_exact::<DurableEnvelopeV1>(
                &extended,
                ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope
            ),
            Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)
        );
    }

    #[test]
    fn approval_authority_mutations_fail_closed() {
        let context = execution_context();
        let payload = approval_payload();
        let envelope = DurableEnvelopeV1::decode(
            build_review_person_match_candidate_approved_outbox_record_v1(
                payload.clone(),
                &ReviewPersonMatchCandidateEnvelopeContextV1 {
                    module_id: REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: "review-runtime-1".to_owned(),
                    runtime_generation: 2,
                    recorded_at_unix_millis: payload.decided_at_unix_millis,
                },
            )
            .expect("approval record")
            .exact_bytes(),
        )
        .expect("approval envelope");
        assert!(
            validate_approval_envelope(&envelope, &payload, &context)
                .and_then(|()| validate_approval_freshness(&envelope, &payload, &context))
                .is_ok()
        );
        for mutated in [
            mutate(&envelope, |value| {
                value.source.as_mut().expect("source").module_id = "private-provider".into();
            }),
            mutate(&envelope, |value| {
                value.actor.as_mut().expect("actor").kind = ActorKindV1::Module as i32;
            }),
            mutate(&envelope, |value| {
                value.source_fence.as_mut().expect("fence").epoch += 1;
            }),
            mutate(&envelope, |value| value.partition_key = vec![9; 16]),
            mutate(&envelope, |value| value.causation_message_id = vec![9; 16]),
            mutate(&envelope, |value| {
                value.recorded_at.as_mut().expect("recorded").seconds += 2;
            }),
        ] {
            assert_eq!(
                validate_approval_envelope(&mutated, &payload, &context)
                    .and_then(|()| validate_approval_freshness(&mutated, &payload, &context)),
                Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)
            );
        }
    }

    #[test]
    fn persons_terminal_authority_mutations_fail_closed() {
        let context = execution_context();
        let command_id = [1; 16];
        let owner_partition =
            persons_owner_partition_id_v1(&context.logical_owner_id).expect("owner partition");
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: vec![3; 16],
            contract: None,
            source: Some(SourceRefV1 {
                module_id: PERSONS_MODULE_ID_V1.to_owned(),
                runtime_instance_id: vec![4; 16],
                runtime_generation: 2,
            }),
            recorded_at: Some(Timestamp {
                seconds: 1,
                nanos: 0,
            }),
            partition_key: owner_partition.to_vec(),
            causation_message_id: command_id.to_vec(),
            correlation_id: owner_partition.to_vec(),
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::Module as i32,
                actor_id: PERSONS_MODULE_ID_V1.as_bytes().to_vec(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: PERSONS_MODULE_ID_V1.as_bytes().to_vec(),
                epoch: 2,
            }),
            semantics: Some(Semantics::Result(ResultMetadataV1 {
                command_id: command_id.to_vec(),
                command_message_id: command_id.to_vec(),
                outcome: ResultOutcomeV1::Succeeded as i32,
                completed_at: Some(Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
                execution_attempt: 1,
            })),
            payload: Vec::new(),
        };
        assert!(validate_persons_terminal_envelope(&envelope, command_id, true, &context).is_ok());
        for mutated in [
            mutate(&envelope, |value| {
                value.source.as_mut().expect("source").module_id = "private-provider".into();
            }),
            mutate(&envelope, |value| {
                value.actor.as_mut().expect("actor").kind = ActorKindV1::OwnerDevice as i32;
            }),
            mutate(&envelope, |value| {
                value.source_fence.as_mut().expect("fence").epoch += 1;
            }),
            mutate(&envelope, |value| {
                let Some(Semantics::Result(result)) = value.semantics.as_mut() else {
                    panic!("result")
                };
                result.execution_attempt = 2;
            }),
            mutate(&envelope, |value| value.causation_message_id = vec![9; 16]),
            mutate(&envelope, |value| value.partition_key = vec![9; 16]),
            mutate(&envelope, |value| {
                value.recorded_at.as_mut().expect("recorded").seconds += 1;
            }),
        ] {
            assert_eq!(
                validate_persons_terminal_envelope(&mutated, command_id, true, &context),
                Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope)
            );
        }
    }

    fn execution_context() -> ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
        ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
            logical_owner_id: "owner-a".to_owned(),
            runtime_instance_id: "promotion-runtime-1".to_owned(),
            runtime_generation: 3,
            now_unix_millis: 2_000,
        }
    }

    fn approval_payload() -> PersonMatchCandidateApprovedForPromotionV1 {
        PersonMatchCandidateApprovedForPromotionV1 {
            review_id: vec![1; 16],
            candidate_id: vec![2; 16],
            candidate_digest: vec![3; 32],
            decision_id: vec![4; 16],
            decision_revision: 2,
            decided_by_owner_device_id: vec![5; 16],
            decided_at_unix_millis: 1_000,
            approved_action: Some(PersonMatchCandidateApprovedActionV1 {
                action: Some(Action::Attach(AttachPersonSourceReviewActionV1 {
                    from_person_id: vec![6; 16],
                    expected_from_person_revision: 1,
                    to_person_id: vec![7; 16],
                    expected_to_person_revision: 1,
                    source: None,
                    expected_source_revision: 1,
                })),
            }),
            approved_action_digest: vec![8; 32],
            logical_owner_id: "owner-a".to_owned(),
        }
    }

    fn mutate(
        value: &DurableEnvelopeV1,
        mutation: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> DurableEnvelopeV1 {
        let mut value = value.clone();
        mutation(&mut value);
        value
    }
}
