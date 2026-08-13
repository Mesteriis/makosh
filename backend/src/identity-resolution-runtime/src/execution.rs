use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
};
use makosh_identity_resolution_api::{
    IdentityResolutionEnvelopeContextV1,
    build_identity_resolution_person_match_candidate_outbox_record_v1,
};
use makosh_identity_resolution_core::{
    IdentityMatchEvidenceV1, IdentityResolutionMatchKindV1, IdentityResolutionSourceV1,
    propose_person_link_merge_candidate_v1,
};
use makosh_identity_resolution_persistence::{
    ApplyIdentityEvidenceOperationV1, IdentityResolutionEnvelopeRecordV1,
    IdentityResolutionPersistenceErrorV1, IdentityResolutionPersistenceV1,
    IdentityResolutionReplayOutcomeV1,
};
use makosh_persons_api::{
    PERSONS_MODULE_ID_V1, persons_owner_partition_id_v1,
    persons_review_candidate_contract_reference_v1,
    wire::{
        IdentityMatchKindV1 as WireMatchKind, PersonReviewCandidateRaisedEventV1,
        ProviderSourceIdentityV1,
    },
};
use prost::Message;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityResolutionExecutionContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionExecutionErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    InvalidPayload,
    EventUnavailable,
    Persistence(IdentityResolutionPersistenceErrorV1),
}

pub async fn process_persons_identity_evidence_v1(
    persistence: &IdentityResolutionPersistenceV1,
    record: &OutboxRecordV1,
    context: &IdentityResolutionExecutionContextV1,
) -> Result<IdentityResolutionReplayOutcomeV1, IdentityResolutionExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = decode_exact(
        record.exact_bytes(),
        IdentityResolutionExecutionErrorV1::InvalidEnvelope,
    )?;
    let payload: PersonReviewCandidateRaisedEventV1 = decode_exact(
        &envelope.payload,
        IdentityResolutionExecutionErrorV1::InvalidPayload,
    )?;
    validate_envelope(record, &envelope, &payload, context)?;
    let evidence = IdentityMatchEvidenceV1 {
        evidence_event_id: id16(&payload.event_id)?,
        candidate_id: id16(&payload.candidate_id)?,
        logical_owner_id: payload.logical_owner_id,
        first_person_id: id16(&payload.first_person_id)?,
        second_person_id: id16(&payload.second_person_id)?,
        first_source: source(payload.first_source.as_ref())?,
        second_source: source(payload.second_source.as_ref())?,
        match_kind: match WireMatchKind::try_from(payload.match_kind) {
            Ok(WireMatchKind::IdentityMatchKindNormalizedEmail) => {
                IdentityResolutionMatchKindV1::NormalizedEmail
            }
            Ok(WireMatchKind::IdentityMatchKindNormalizedPhone) => {
                IdentityResolutionMatchKindV1::NormalizedPhone
            }
            _ => return Err(IdentityResolutionExecutionErrorV1::InvalidPayload),
        },
        observed_at_unix_millis: millis(
            payload
                .observed_at
                .as_ref()
                .ok_or(IdentityResolutionExecutionErrorV1::InvalidPayload)?
                .unix_seconds,
            payload.observed_at.as_ref().expect("checked").nanos,
        )?,
        resulting_owner_revision: payload.resulting_owner_revision,
    };
    if let Some(replay) = persistence
        .replay_if_completed(&context.logical_owner_id, &stored(record))
        .await
        .map_err(IdentityResolutionExecutionErrorV1::Persistence)?
    {
        return Ok(replay);
    }
    let proposal = propose_person_link_merge_candidate_v1(&evidence)
        .map_err(|_| IdentityResolutionExecutionErrorV1::InvalidPayload)?;
    let output = build_identity_resolution_person_match_candidate_outbox_record_v1(
        proposal,
        &IdentityResolutionEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_millis: context.now_unix_millis,
        },
    )
    .map_err(|_| IdentityResolutionExecutionErrorV1::InvalidPayload)?;
    persistence
        .apply_once(&ApplyIdentityEvidenceOperationV1 {
            input: stored(record),
            evidence,
            proposal: stored(&output),
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(IdentityResolutionExecutionErrorV1::Persistence)
}

fn validate_envelope(
    record: &OutboxRecordV1,
    envelope: &DurableEnvelopeV1,
    payload: &PersonReviewCandidateRaisedEventV1,
    context: &IdentityResolutionExecutionContextV1,
) -> Result<(), IdentityResolutionExecutionErrorV1> {
    let expected = persons_review_candidate_contract_reference_v1();
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    let source_ref = envelope
        .source
        .as_ref()
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    let observed = payload
        .observed_at
        .as_ref()
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidPayload)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(v)) => v.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)?;
    let partition = persons_owner_partition_id_v1(&payload.logical_owner_id)
        .map_err(|_| IdentityResolutionExecutionErrorV1::InvalidPayload)?;
    let observed_ms = millis(observed.unix_seconds, observed.nanos)?;
    let recorded_ms = millis(recorded.seconds, recorded.nanos)?;
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || source_ref.module_id != PERSONS_MODULE_ID_V1
        || source_ref.runtime_instance_id.len() != 16
        || source_ref.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != PERSONS_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != PERSONS_MODULE_ID_V1.as_bytes()
        || fence.epoch != source_ref.runtime_generation
        || envelope.message_id != payload.event_id
        || *record.message_id() != id16(&payload.event_id)?
        || envelope.partition_key != partition
        || envelope.correlation_id != partition
        || envelope.causation_message_id.len() != 16
        || occurred.seconds != observed.unix_seconds
        || occurred.nanos != observed.nanos
        || observed_ms > recorded_ms
        || recorded_ms > context.now_unix_millis
        || payload.logical_owner_id != context.logical_owner_id
    {
        return Err(IdentityResolutionExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}
fn source(
    v: Option<&ProviderSourceIdentityV1>,
) -> Result<IdentityResolutionSourceV1, IdentityResolutionExecutionErrorV1> {
    let v = v.ok_or(IdentityResolutionExecutionErrorV1::InvalidPayload)?;
    Ok(IdentityResolutionSourceV1 {
        integration_public_id: id16(&v.integration_public_id)?,
        account_public_id: id16(&v.account_public_id)?,
        provider_source_contact_public_id: id16(&v.provider_source_contact_public_id)?,
    })
}
fn stored(v: &OutboxRecordV1) -> IdentityResolutionEnvelopeRecordV1 {
    IdentityResolutionEnvelopeRecordV1 {
        message_id: *v.message_id(),
        envelope_sha256: *v.envelope_sha256(),
        envelope_bytes: v.exact_bytes().to_vec(),
    }
}
fn decode_exact<M: Message + Default>(
    bytes: &[u8],
    error: IdentityResolutionExecutionErrorV1,
) -> Result<M, IdentityResolutionExecutionErrorV1> {
    let value = M::decode(bytes).map_err(|_| error)?;
    if value.encode_to_vec() != bytes {
        return Err(error);
    }
    Ok(value)
}
fn id16(v: &[u8]) -> Result<[u8; 16], IdentityResolutionExecutionErrorV1> {
    v.try_into()
        .ok()
        .filter(|x: &[u8; 16]| x.iter().any(|b| *b != 0))
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidPayload)
}
fn millis(seconds: i64, nanos: i32) -> Result<i64, IdentityResolutionExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(IdentityResolutionExecutionErrorV1::InvalidEnvelope);
    }
    seconds
        .checked_mul(1000)
        .and_then(|v| v.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(IdentityResolutionExecutionErrorV1::InvalidEnvelope)
}
fn validate_context(
    v: &IdentityResolutionExecutionContextV1,
) -> Result<(), IdentityResolutionExecutionErrorV1> {
    if v.logical_owner_id.is_empty()
        || v.runtime_instance_id.is_empty()
        || v.runtime_generation == 0
        || v.now_unix_millis <= 0
    {
        Err(IdentityResolutionExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{delivery::OutboxRecordV1, v1::DurableEnvelopeV1};
    use makosh_persons_api::{
        PersonsActionDigestSourceV1, PersonsIdentityMatchKindV1,
        persons_identity_match_candidate_id_v1,
        wire::{IdentityMatchKindV1, PersonReviewCandidateRaisedEventV1, TimestampV1},
    };
    use makosh_persons_runtime::transport::{
        PersonsEnvelopeContextV1, build_persons_review_candidate_outbox_record_v1,
    };
    use prost::Message;

    use super::*;

    fn source(seed: u8) -> ProviderSourceIdentityV1 {
        ProviderSourceIdentityV1 {
            integration_public_id: vec![seed; 16],
            account_public_id: vec![seed + 1; 16],
            provider_source_contact_public_id: vec![seed + 2; 16],
        }
    }

    fn digest_source(value: &ProviderSourceIdentityV1) -> PersonsActionDigestSourceV1 {
        PersonsActionDigestSourceV1 {
            integration_public_id: value.integration_public_id.as_slice().try_into().unwrap(),
            account_public_id: value.account_public_id.as_slice().try_into().unwrap(),
            provider_source_contact_public_id: value
                .provider_source_contact_public_id
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }

    fn fixture() -> (OutboxRecordV1, IdentityResolutionExecutionContextV1) {
        let owner = "owner-identity-resolution";
        let first_source = source(11);
        let second_source = source(21);
        let candidate_id = persons_identity_match_candidate_id_v1(
            owner,
            digest_source(&first_source),
            digest_source(&second_source),
            PersonsIdentityMatchKindV1::NormalizedEmail,
        )
        .unwrap();
        let partition = persons_owner_partition_id_v1(owner).unwrap();
        let payload = PersonReviewCandidateRaisedEventV1 {
            event_id: vec![31; 16],
            candidate_id: candidate_id.to_vec(),
            logical_owner_id: owner.to_owned(),
            first_person_id: vec![41; 16],
            second_person_id: vec![51; 16],
            first_source: Some(first_source),
            second_source: Some(second_source),
            match_kind: IdentityMatchKindV1::IdentityMatchKindNormalizedEmail as i32,
            observed_at: Some(TimestampV1 {
                unix_seconds: 1,
                nanos: 0,
            }),
            resulting_owner_revision: 7,
        };
        let record = build_persons_review_candidate_outbox_record_v1(
            [61; 16],
            partition,
            [31; 16],
            partition,
            payload,
            &PersonsEnvelopeContextV1 {
                module_id: PERSONS_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "0123456789abcdef".to_owned(),
                runtime_generation: 3,
                recorded_at_unix_seconds: 2,
                recorded_at_nanos: 0,
            },
        )
        .unwrap();
        (
            record,
            IdentityResolutionExecutionContextV1 {
                logical_owner_id: owner.to_owned(),
                runtime_instance_id: "fedcba9876543210".to_owned(),
                runtime_generation: 4,
                now_unix_millis: 2_000,
            },
        )
    }

    fn validate(record: &OutboxRecordV1, context: &IdentityResolutionExecutionContextV1) {
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).unwrap();
        let payload =
            PersonReviewCandidateRaisedEventV1::decode(envelope.payload.as_slice()).unwrap();
        assert_eq!(
            validate_envelope(record, &envelope, &payload, context),
            Ok(())
        );
    }

    fn mutated(
        record: &OutboxRecordV1,
        mutate: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> OutboxRecordV1 {
        let mut envelope = DurableEnvelopeV1::decode(record.exact_bytes()).unwrap();
        mutate(&mut envelope);
        OutboxRecordV1::accept(envelope.encode_to_vec()).unwrap()
    }

    #[test]
    fn persons_candidate_authority_and_canonical_bytes_fail_closed() {
        let (record, context) = fixture();
        validate(&record, &context);

        for changed in [
            mutated(&record, |v| v.source.as_mut().unwrap().module_id.push('x')),
            mutated(&record, |v| {
                v.actor.as_mut().unwrap().kind = ActorKindV1::OwnerDevice as i32
            }),
            mutated(&record, |v| v.source_fence.as_mut().unwrap().epoch += 1),
            mutated(&record, |v| v.correlation_id = vec![99; 16]),
            mutated(&record, |v| v.partition_key = vec![98; 16]),
        ] {
            let envelope = DurableEnvelopeV1::decode(changed.exact_bytes()).unwrap();
            let payload =
                PersonReviewCandidateRaisedEventV1::decode(envelope.payload.as_slice()).unwrap();
            assert_eq!(
                validate_envelope(&changed, &envelope, &payload, &context),
                Err(IdentityResolutionExecutionErrorV1::InvalidEnvelope)
            );
        }

        let mut noncanonical = record.exact_bytes().to_vec();
        noncanonical.extend_from_slice(&[0xfa, 0x07, 0x03, b'r', b'a', b'w']);
        assert_eq!(
            decode_exact::<DurableEnvelopeV1>(
                &noncanonical,
                IdentityResolutionExecutionErrorV1::InvalidEnvelope,
            ),
            Err(IdentityResolutionExecutionErrorV1::InvalidEnvelope)
        );
    }
}
