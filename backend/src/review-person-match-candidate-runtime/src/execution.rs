use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
};
use makosh_identity_resolution_api::{
    IDENTITY_RESOLUTION_MODULE_ID_V1, identity_resolution_owner_partition_id_v1,
    identity_resolution_person_match_candidate_contract_reference_v1,
    wire::{
        IdentityMatchKindV1 as IdentityWireMatchKind, PersonLinkMergeCandidateProposedEventV1,
        PublicPersonSourceIdentityV1 as IdentityWireSource,
    },
};
use makosh_persons_api::{
    PersonsActionDigestSourceV1, PersonsActionDigestSplitSourceV1,
    persons_attach_source_action_digest_v1, persons_merge_action_digest_v1,
    persons_split_action_digest_v1,
};
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1, ReviewPersonMatchCandidateEnvelopeContextV1,
    build_review_person_match_candidate_approved_outbox_record_v1,
    build_review_person_match_candidate_submitted_outbox_record_v1,
    review_person_match_candidate_decision_contract_reference_v1,
    wire::{
        AttachPersonSourceReviewActionV1, DecidePersonMatchCandidateRequestV1,
        MergePersonsReviewActionV1, PersonMatchCandidateApprovedActionV1 as WireApprovedAction,
        PersonMatchCandidateApprovedForPromotionV1, PersonMatchCandidateDecisionV1 as WireDecision,
        PersonMatchCandidateReviewSubmittedV1, PublicPersonSourceIdentityV1 as WirePublicSource,
        SplitPersonReviewActionV1, SplitPersonSourceSelectionV1,
        person_match_candidate_approved_action_v1::Action as WireAction,
    },
};
use makosh_review_person_match_candidate_core::{
    PersonMatchCandidateApprovedActionV1, PersonMatchCandidateDecisionV1,
    PersonMatchCandidateEvidenceV1, PersonMatchKindV1, PublicPersonSourceIdentityV1,
    SplitProfileFactKindV1, SplitSourceSelectionV1, create_person_match_candidate_review_v1,
    person_match_candidate_evidence_digest_v1,
};
use makosh_review_person_match_candidate_persistence::{
    DecidePersonMatchCandidateOperationV1, ReviewPersonMatchCandidateEnvelopeRecordV1,
    ReviewPersonMatchCandidatePersistenceErrorV1, ReviewPersonMatchCandidatePersistenceV1,
    ReviewPersonMatchCandidateReplayOutcomeV1, SubmitPersonMatchCandidateOperationV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub(crate) const REVIEW_DECISION_GATEWAY_MODULE_ID_V1: &str =
    "makosh-review-person-match-candidate-command-gateway";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidateExecutionContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidateExecutionErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    InvalidContext,
    EventUnavailable,
    Persistence(ReviewPersonMatchCandidatePersistenceErrorV1),
}

pub async fn process_persons_review_candidate_v1(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    record: &OutboxRecordV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<ReviewPersonMatchCandidateReplayOutcomeV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    validate_context(context)?;
    let command_record = stored(record);
    if let Some(replay) = persistence
        .replay_submission_if_completed(&context.logical_owner_id, &command_record)
        .await
        .map_err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence)?
    {
        return Ok(replay);
    }
    let envelope: DurableEnvelopeV1 = decode_exact(
        record.exact_bytes(),
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope,
    )?;
    let payload: PersonLinkMergeCandidateProposedEventV1 = decode_exact(
        &envelope.payload,
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload,
    )?;
    let evidence = decode_identity_resolution_candidate(&envelope, payload, context)?;
    let pending = create_person_match_candidate_review_v1(evidence.clone())
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    let expected_existing_revision = match persistence
        .load_review(&context.logical_owner_id, pending.review_id)
        .await
    {
        Ok(current) => Some(current.review_revision),
        Err(ReviewPersonMatchCandidatePersistenceErrorV1::NotFound) => None,
        Err(error) => {
            return Err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                error,
            ));
        }
    };
    let resulting_review_revision = expected_existing_revision
        .map_or(Some(1), |revision| revision.checked_add(1))
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    let submitted = build_review_person_match_candidate_submitted_outbox_record_v1(
        *record.message_id(),
        PersonMatchCandidateReviewSubmittedV1 {
            submission_id: record.message_id().to_vec(),
            review_id: pending.review_id.to_vec(),
            candidate_id: evidence.candidate_id.to_vec(),
            candidate_digest: evidence.candidate_digest.to_vec(),
            review_revision: resulting_review_revision,
            logical_owner_id: evidence.logical_owner_id.clone(),
        },
        &envelope_context(context),
    )
    .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    persistence
        .submit_once(&SubmitPersonMatchCandidateOperationV1 {
            command: command_record,
            evidence,
            submitted_result: stored(&submitted),
            expected_existing_revision,
            received_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence)
}

pub async fn process_person_match_candidate_decision_v1(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    record: &OutboxRecordV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<ReviewPersonMatchCandidateReplayOutcomeV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = decode_exact(
        record.exact_bytes(),
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope,
    )?;
    let payload: DecidePersonMatchCandidateRequestV1 = decode_exact(
        &envelope.payload,
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload,
    )?;
    let decoded = decode_decision(&envelope, &payload, context)?;
    let decision_revision = next_review_revision(decoded.expected_review_revision)?;
    let current = persistence
        .load_review(&context.logical_owner_id, decoded.review_id)
        .await
        .map_err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence)?;
    let approved_event = match &decoded.decision {
        PersonMatchCandidateDecisionV1::Approve {
            action,
            approved_action_digest,
        } => {
            let event = build_review_person_match_candidate_approved_outbox_record_v1(
                PersonMatchCandidateApprovedForPromotionV1 {
                    review_id: decoded.review_id.to_vec(),
                    candidate_id: current.evidence.candidate_id.to_vec(),
                    candidate_digest: current.evidence.candidate_digest.to_vec(),
                    decision_id: record.message_id().to_vec(),
                    decision_revision,
                    decided_by_owner_device_id: decoded.device_id.to_vec(),
                    decided_at_unix_millis: decoded.decided_at_unix_millis,
                    approved_action: Some(wire_action(action)),
                    approved_action_digest: approved_action_digest.to_vec(),
                    logical_owner_id: context.logical_owner_id.clone(),
                },
                &envelope_context(context),
            )
            .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
            Some(stored(&event))
        }
        PersonMatchCandidateDecisionV1::Reject => None,
    };
    persistence
        .decide_once(&DecidePersonMatchCandidateOperationV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            command: stored(record),
            review_id: decoded.review_id,
            expected_review_revision: decoded.expected_review_revision,
            decision: decoded.decision,
            decided_by_owner_device_id: decoded.device_id,
            decided_at_unix_millis: decoded.decided_at_unix_millis,
            approved_event,
            received_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence)
}

struct DecodedDecisionV1 {
    review_id: [u8; 16],
    expected_review_revision: u64,
    decision: PersonMatchCandidateDecisionV1,
    device_id: [u8; 16],
    decided_at_unix_millis: i64,
}

fn decode_identity_resolution_candidate(
    envelope: &DurableEnvelopeV1,
    payload: PersonLinkMergeCandidateProposedEventV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<PersonMatchCandidateEvidenceV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let expected = identity_resolution_person_match_candidate_contract_reference_v1();
    let source = envelope
        .source
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(value)) => value.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let observed_at_unix_millis = payload.observed_at_unix_millis;
    let recorded_at_unix_millis = timestamp_millis(recorded.seconds, recorded.nanos)?;
    let occurred_at_unix_millis = timestamp_millis(occurred.seconds, occurred.nanos)?;
    let partition = identity_resolution_owner_partition_id_v1(&payload.logical_owner_id)
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || source.module_id != IDENTITY_RESOLUTION_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != IDENTITY_RESOLUTION_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != IDENTITY_RESOLUTION_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.message_id != payload.event_id
        || envelope.partition_key != partition
        || envelope.correlation_id != partition
        || envelope.causation_message_id != payload.evidence_event_id
        || occurred_at_unix_millis != observed_at_unix_millis
        || observed_at_unix_millis <= 0
        || observed_at_unix_millis > recorded_at_unix_millis
        || recorded_at_unix_millis > context.now_unix_millis
        || payload.logical_owner_id != context.logical_owner_id
    {
        return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope);
    }
    let mut evidence = PersonMatchCandidateEvidenceV1 {
        evidence_event_id: id16(&payload.evidence_event_id)?,
        candidate_id: id16(&payload.candidate_id)?,
        logical_owner_id: payload.logical_owner_id,
        first_person_id: id16(&payload.first_person_id)?,
        second_person_id: id16(&payload.second_person_id)?,
        first_source: core_identity_source(payload.first_source.as_ref())?,
        second_source: core_identity_source(payload.second_source.as_ref())?,
        match_kind: match IdentityWireMatchKind::try_from(payload.match_kind) {
            Ok(IdentityWireMatchKind::IdentityMatchKindNormalizedEmail) => {
                PersonMatchKindV1::NormalizedEmail
            }
            Ok(IdentityWireMatchKind::IdentityMatchKindNormalizedPhone) => {
                PersonMatchKindV1::NormalizedPhone
            }
            _ => return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload),
        },
        observed_at_unix_millis,
        resulting_owner_revision: payload.resulting_owner_revision,
        candidate_digest: [0; 32],
    };
    evidence.candidate_digest = person_match_candidate_evidence_digest_v1(&evidence)
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    Ok(evidence)
}

fn decode_decision(
    envelope: &DurableEnvelopeV1,
    payload: &DecidePersonMatchCandidateRequestV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<DecodedDecisionV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let expected = review_person_match_candidate_decision_contract_reference_v1();
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let command = match envelope.semantics.as_ref() {
        Some(Semantics::Command(value)) => value,
        _ => return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope),
    };
    let review_id = id16(&payload.review_id)?;
    let operation_id = id16(&payload.operation_id)?;
    let device_id = id16(&payload.decided_by_owner_device_id)?;
    let recorded_at = timestamp_millis(recorded.seconds, recorded.nanos)?;
    let deadline = command
        .deadline
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let deadline_at = timestamp_millis(deadline.seconds, deadline.nanos)?;
    let payload_digest: [u8; 32] = Sha256::digest(payload.encode_to_vec()).into();
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || operation_id != id16(&envelope.message_id)?
        || command.command_id != payload.operation_id
        || command.target_capability != REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1
        || command.logical_attempt == 0
        || command.idempotency_key != payload_digest
        || source.module_id != REVIEW_DECISION_GATEWAY_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != REVIEW_DECISION_GATEWAY_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.partition_key != payload.review_id
        || envelope.correlation_id != payload.review_id
        || !envelope.causation_message_id.is_empty()
        || actor.kind != ActorKindV1::OwnerDevice as i32
        || actor.actor_id != payload.decided_by_owner_device_id
        || payload.protocol_major != 1
        || payload.expected_review_revision == 0
        || payload.decided_at_unix_millis <= 0
        || payload.decided_at_unix_millis != recorded_at
        || recorded_at > context.now_unix_millis
        || deadline_at <= context.now_unix_millis
    {
        return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope);
    }
    let decision = match WireDecision::try_from(payload.decision) {
        Ok(WireDecision::PersonMatchCandidateDecisionApprove) => {
            PersonMatchCandidateDecisionV1::Approve {
                action: core_action(payload.approved_action.as_ref())?,
                approved_action_digest: id32(&payload.approved_action_digest)?,
            }
        }
        Ok(WireDecision::PersonMatchCandidateDecisionReject)
            if payload.approved_action.is_none() && payload.approved_action_digest.is_empty() =>
        {
            PersonMatchCandidateDecisionV1::Reject
        }
        _ => return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload),
    };
    if let PersonMatchCandidateDecisionV1::Approve {
        action,
        approved_action_digest,
    } = &decision
        && canonical_action_digest(action, &context.logical_owner_id)? != *approved_action_digest
    {
        return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload);
    }
    Ok(DecodedDecisionV1 {
        review_id,
        expected_review_revision: payload.expected_review_revision,
        decision,
        device_id,
        decided_at_unix_millis: payload.decided_at_unix_millis,
    })
}

fn canonical_action_digest(
    action: &PersonMatchCandidateApprovedActionV1,
    owner: &str,
) -> Result<[u8; 32], ReviewPersonMatchCandidateExecutionErrorV1> {
    match action {
        PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id,
            expected_from_person_revision,
            to_person_id,
            expected_to_person_revision,
            source,
            expected_source_revision,
        } => persons_attach_source_action_digest_v1(
            owner,
            *from_person_id,
            *expected_from_person_revision,
            *to_person_id,
            *expected_to_person_revision,
            digest_source(*source),
            *expected_source_revision,
        ),
        PersonMatchCandidateApprovedActionV1::Merge {
            source_person_id,
            expected_source_person_revision,
            target_person_id,
            expected_target_person_revision,
        } => persons_merge_action_digest_v1(
            owner,
            *source_person_id,
            *expected_source_person_revision,
            *target_person_id,
            *expected_target_person_revision,
        ),
        PersonMatchCandidateApprovedActionV1::Split {
            merged_person_id,
            expected_merged_person_revision,
            target_person_id,
            expected_target_person_revision,
            source_selection,
            profile_fact_selection,
        } => {
            let sources = source_selection
                .iter()
                .map(|selected| PersonsActionDigestSplitSourceV1 {
                    source: digest_source(selected.source),
                    expected_source_revision: selected.expected_source_revision,
                })
                .collect::<Vec<_>>();
            let facts = profile_fact_selection
                .iter()
                .map(|fact| match fact {
                    SplitProfileFactKindV1::DisplayName => 1,
                    SplitProfileFactKindV1::GivenName => 2,
                    SplitProfileFactKindV1::FamilyName => 3,
                    SplitProfileFactKindV1::Emails => 4,
                    SplitProfileFactKindV1::Phones => 5,
                })
                .collect::<Vec<_>>();
            persons_split_action_digest_v1(
                owner,
                *merged_person_id,
                *expected_merged_person_revision,
                *target_person_id,
                *expected_target_person_revision,
                &sources,
                &facts,
            )
        }
    }
    .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

fn next_review_revision(current: u64) -> Result<u64, ReviewPersonMatchCandidateExecutionErrorV1> {
    current
        .checked_add(1)
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

const fn digest_source(source: PublicPersonSourceIdentityV1) -> PersonsActionDigestSourceV1 {
    PersonsActionDigestSourceV1 {
        integration_public_id: source.integration_public_id,
        account_public_id: source.account_public_id,
        provider_source_contact_public_id: source.provider_source_contact_public_id,
    }
}

fn core_action(
    value: Option<&WireApprovedAction>,
) -> Result<PersonMatchCandidateApprovedActionV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    match value.and_then(|value| value.action.as_ref()) {
        Some(WireAction::Attach(value)) => Ok(PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id: id16(&value.from_person_id)?,
            expected_from_person_revision: value.expected_from_person_revision,
            to_person_id: id16(&value.to_person_id)?,
            expected_to_person_revision: value.expected_to_person_revision,
            source: core_public_source(value.source.as_ref())?,
            expected_source_revision: value.expected_source_revision,
        }),
        Some(WireAction::Merge(value)) => Ok(PersonMatchCandidateApprovedActionV1::Merge {
            source_person_id: id16(&value.source_person_id)?,
            expected_source_person_revision: value.expected_source_person_revision,
            target_person_id: id16(&value.target_person_id)?,
            expected_target_person_revision: value.expected_target_person_revision,
        }),
        Some(WireAction::Split(value)) => Ok(PersonMatchCandidateApprovedActionV1::Split {
            merged_person_id: id16(&value.merged_person_id)?,
            expected_merged_person_revision: value.expected_merged_person_revision,
            target_person_id: id16(&value.target_person_id)?,
            expected_target_person_revision: value.expected_target_person_revision,
            source_selection: value
                .source_selection
                .iter()
                .map(|selected| {
                    Ok(SplitSourceSelectionV1 {
                        source: core_public_source(selected.source.as_ref())?,
                        expected_source_revision: selected.expected_source_revision,
                    })
                })
                .collect::<Result<_, ReviewPersonMatchCandidateExecutionErrorV1>>()?,
            profile_fact_selection: value
                .profile_fact_selection
                .iter()
                .map(|value| match *value {
                    1 => Ok(SplitProfileFactKindV1::DisplayName),
                    2 => Ok(SplitProfileFactKindV1::GivenName),
                    3 => Ok(SplitProfileFactKindV1::FamilyName),
                    4 => Ok(SplitProfileFactKindV1::Emails),
                    5 => Ok(SplitProfileFactKindV1::Phones),
                    _ => Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload),
                })
                .collect::<Result<_, _>>()?,
        }),
        None => Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload),
    }
}

fn wire_action(value: &PersonMatchCandidateApprovedActionV1) -> WireApprovedAction {
    let action = match value {
        PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id,
            expected_from_person_revision,
            to_person_id,
            expected_to_person_revision,
            source,
            expected_source_revision,
        } => WireAction::Attach(AttachPersonSourceReviewActionV1 {
            from_person_id: from_person_id.to_vec(),
            expected_from_person_revision: *expected_from_person_revision,
            to_person_id: to_person_id.to_vec(),
            expected_to_person_revision: *expected_to_person_revision,
            source: Some(wire_public_source(*source)),
            expected_source_revision: *expected_source_revision,
        }),
        PersonMatchCandidateApprovedActionV1::Merge {
            source_person_id,
            expected_source_person_revision,
            target_person_id,
            expected_target_person_revision,
        } => WireAction::Merge(MergePersonsReviewActionV1 {
            source_person_id: source_person_id.to_vec(),
            expected_source_person_revision: *expected_source_person_revision,
            target_person_id: target_person_id.to_vec(),
            expected_target_person_revision: *expected_target_person_revision,
        }),
        PersonMatchCandidateApprovedActionV1::Split {
            merged_person_id,
            expected_merged_person_revision,
            target_person_id,
            expected_target_person_revision,
            source_selection,
            profile_fact_selection,
        } => WireAction::Split(SplitPersonReviewActionV1 {
            merged_person_id: merged_person_id.to_vec(),
            expected_merged_person_revision: *expected_merged_person_revision,
            target_person_id: target_person_id.to_vec(),
            expected_target_person_revision: *expected_target_person_revision,
            source_selection: source_selection
                .iter()
                .map(|selected| SplitPersonSourceSelectionV1 {
                    source: Some(wire_public_source(selected.source)),
                    expected_source_revision: selected.expected_source_revision,
                })
                .collect(),
            profile_fact_selection: profile_fact_selection
                .iter()
                .map(|fact| match fact {
                    SplitProfileFactKindV1::DisplayName => 1,
                    SplitProfileFactKindV1::GivenName => 2,
                    SplitProfileFactKindV1::FamilyName => 3,
                    SplitProfileFactKindV1::Emails => 4,
                    SplitProfileFactKindV1::Phones => 5,
                })
                .collect(),
        }),
    };
    WireApprovedAction {
        action: Some(action),
    }
}

fn core_public_source(
    value: Option<&WirePublicSource>,
) -> Result<PublicPersonSourceIdentityV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    let value = value.ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    Ok(PublicPersonSourceIdentityV1 {
        integration_public_id: id16(&value.integration_public_id)?,
        account_public_id: id16(&value.account_public_id)?,
        provider_source_contact_public_id: id16(&value.provider_source_contact_public_id)?,
    })
}

fn core_identity_source(
    value: Option<&IdentityWireSource>,
) -> Result<PublicPersonSourceIdentityV1, ReviewPersonMatchCandidateExecutionErrorV1> {
    let value = value.ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    Ok(PublicPersonSourceIdentityV1 {
        integration_public_id: id16(&value.integration_public_id)?,
        account_public_id: id16(&value.account_public_id)?,
        provider_source_contact_public_id: id16(&value.provider_source_contact_public_id)?,
    })
}

fn wire_public_source(value: PublicPersonSourceIdentityV1) -> WirePublicSource {
    WirePublicSource {
        integration_public_id: value.integration_public_id.to_vec(),
        account_public_id: value.account_public_id.to_vec(),
        provider_source_contact_public_id: value.provider_source_contact_public_id.to_vec(),
    }
}

fn envelope_context(
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> ReviewPersonMatchCandidateEnvelopeContextV1 {
    ReviewPersonMatchCandidateEnvelopeContextV1 {
        module_id: REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1.to_owned(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_millis: context.now_unix_millis,
    }
}

fn stored(record: &OutboxRecordV1) -> ReviewPersonMatchCandidateEnvelopeRecordV1 {
    ReviewPersonMatchCandidateEnvelopeRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn validate_context(
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<(), ReviewPersonMatchCandidateExecutionErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.logical_owner_id.len() > 128
        || !context.logical_owner_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
    {
        Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}

fn timestamp_millis(
    seconds: i64,
    nanos: i32,
) -> Result<i64, ReviewPersonMatchCandidateExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload);
    }
    seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewPersonMatchCandidateExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ReviewPersonMatchCandidateExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

fn decode_exact<T: Message + Default>(
    bytes: &[u8],
    error: ReviewPersonMatchCandidateExecutionErrorV1,
) -> Result<T, ReviewPersonMatchCandidateExecutionErrorV1> {
    let value = T::decode(bytes).map_err(|_| error)?;
    if value.encode_to_vec() != bytes {
        return Err(error);
    }
    Ok(value)
}

#[cfg(test)]
mod exact_decode_tests {
    use super::*;
    use makosh_events_protocol::v1::{
        ActorRefV1, CommandMetadataV1, ContractRefV1, SourceFenceV1, SourceRefV1,
    };
    use makosh_identity_resolution_api::{
        IdentityResolutionEnvelopeContextV1,
        build_identity_resolution_person_match_candidate_outbox_record_v1,
        identity_resolution_proposal_event_id_v1,
    };
    use prost_types::Timestamp;

    #[test]
    fn canonical_decode_rejects_unknown_private_extension_bytes() {
        let canonical = DurableEnvelopeV1::default().encode_to_vec();
        let mut extended = canonical.clone();
        extended.extend_from_slice(&[0x98, 0x06, 0x01]);
        assert!(
            decode_exact::<DurableEnvelopeV1>(
                &canonical,
                ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope
            )
            .is_ok()
        );
        assert_eq!(
            decode_exact::<DurableEnvelopeV1>(
                &extended,
                ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope
            ),
            Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)
        );
    }

    #[test]
    fn candidate_authority_mutations_fail_closed() {
        let evidence_event_id = [1; 16];
        let candidate_id = [2; 16];
        let payload = PersonLinkMergeCandidateProposedEventV1 {
            event_id: identity_resolution_proposal_event_id_v1(evidence_event_id, candidate_id)
                .to_vec(),
            evidence_event_id: evidence_event_id.to_vec(),
            candidate_id: vec![2; 16],
            logical_owner_id: "owner-a".into(),
            first_person_id: vec![3; 16],
            second_person_id: vec![4; 16],
            first_source: Some(identity_source(5)),
            second_source: Some(identity_source(8)),
            match_kind: IdentityWireMatchKind::IdentityMatchKindNormalizedEmail as i32,
            observed_at_unix_millis: 1_000,
            resulting_owner_revision: 1,
        };
        let record = build_identity_resolution_person_match_candidate_outbox_record_v1(
            payload.clone(),
            &IdentityResolutionEnvelopeContextV1 {
                runtime_instance_id: "identity-resolution-1".into(),
                runtime_generation: 2,
                recorded_at_unix_millis: 2_000,
            },
        )
        .expect("candidate record");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let context = ReviewPersonMatchCandidateExecutionContextV1 {
            logical_owner_id: "owner-a".into(),
            runtime_instance_id: "review-1".into(),
            runtime_generation: 3,
            now_unix_millis: 2_000,
        };
        assert!(decode_identity_resolution_candidate(&envelope, payload.clone(), &context).is_ok());
        for mutated in [
            {
                let mut v = envelope.clone();
                v.source.as_mut().expect("source").module_id = "private-provider".into();
                v
            },
            {
                let mut v = envelope.clone();
                v.actor.as_mut().expect("actor").kind = ActorKindV1::OwnerDevice as i32;
                v
            },
            {
                let mut v = envelope.clone();
                v.source_fence.as_mut().expect("fence").epoch += 1;
                v
            },
            {
                let mut v = envelope.clone();
                v.correlation_id = vec![10; 16];
                v
            },
            {
                let mut v = envelope.clone();
                v.semantics = None;
                v
            },
        ] {
            assert_eq!(
                decode_identity_resolution_candidate(&mutated, payload.clone(), &context),
                Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)
            );
        }
    }

    #[test]
    fn decision_authority_mutations_fail_closed() {
        let payload = DecidePersonMatchCandidateRequestV1 {
            protocol_major: 1,
            operation_id: vec![1; 16],
            review_id: vec![2; 16],
            expected_review_revision: 1,
            decision: WireDecision::PersonMatchCandidateDecisionReject as i32,
            approved_action: None,
            approved_action_digest: Vec::new(),
            decided_by_owner_device_id: vec![3; 16],
            decided_at_unix_millis: 1_000,
        };
        let expected = review_person_match_candidate_decision_contract_reference_v1();
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: payload.operation_id.clone(),
            contract: Some(ContractRefV1 {
                owner: expected.owner,
                name: expected.name,
                major: expected.major,
                revision: expected.revision,
                schema_sha256: expected.schema_sha256,
            }),
            source: Some(SourceRefV1 {
                module_id: REVIEW_DECISION_GATEWAY_MODULE_ID_V1.into(),
                runtime_instance_id: vec![4; 16],
                runtime_generation: 2,
            }),
            recorded_at: Some(Timestamp {
                seconds: 1,
                nanos: 0,
            }),
            partition_key: payload.review_id.clone(),
            causation_message_id: Vec::new(),
            correlation_id: payload.review_id.clone(),
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::OwnerDevice as i32,
                actor_id: payload.decided_by_owner_device_id.clone(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: REVIEW_DECISION_GATEWAY_MODULE_ID_V1.as_bytes().to_vec(),
                epoch: 2,
            }),
            semantics: Some(Semantics::Command(CommandMetadataV1 {
                command_id: payload.operation_id.clone(),
                target_capability: REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1.into(),
                idempotency_key: Sha256::digest(payload.encode_to_vec()).to_vec(),
                deadline: Some(Timestamp {
                    seconds: 3,
                    nanos: 0,
                }),
                logical_attempt: 1,
            })),
            payload: payload.encode_to_vec(),
        };
        let context = ReviewPersonMatchCandidateExecutionContextV1 {
            logical_owner_id: "owner-a".into(),
            runtime_instance_id: "review-1".into(),
            runtime_generation: 3,
            now_unix_millis: 2_000,
        };
        assert!(decode_decision(&envelope, &payload, &context).is_ok());
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
            mutate(&envelope, |value| {
                value.recorded_at.as_mut().expect("recorded").seconds += 1;
            }),
            mutate(&envelope, |value| {
                let Some(Semantics::Command(command)) = value.semantics.as_mut() else {
                    panic!("command")
                };
                command.idempotency_key = vec![9; 32];
            }),
        ] {
            assert!(matches!(
                decode_decision(&mutated, &payload, &context),
                Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)
            ));
        }
    }

    #[test]
    fn decision_revision_overflow_is_a_bounded_invalid_payload() {
        assert_eq!(
            next_review_revision(u64::MAX),
            Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
        );
    }

    fn mutate(
        value: &DurableEnvelopeV1,
        mutation: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> DurableEnvelopeV1 {
        let mut value = value.clone();
        mutation(&mut value);
        value
    }

    fn identity_source(seed: u8) -> IdentityWireSource {
        IdentityWireSource {
            integration_public_id: vec![seed; 16],
            account_public_id: vec![seed + 1; 16],
            provider_source_contact_public_id: vec![seed + 2; 16],
        }
    }
}
