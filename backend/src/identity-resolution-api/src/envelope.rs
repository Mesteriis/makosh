use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    IDENTITY_RESOLUTION_CONTRACT_MAJOR_V1, IDENTITY_RESOLUTION_CONTRACT_REVISION_V1,
    IDENTITY_RESOLUTION_MODULE_ID_V1, IDENTITY_RESOLUTION_OWNER_ID_V1,
    IDENTITY_RESOLUTION_PERSON_MATCH_CANDIDATE_CONTRACT_NAME_V1,
    IDENTITY_RESOLUTION_SCHEMA_SHA256_V1, identity_resolution_owner_partition_id_v1,
    identity_resolution_proposal_event_id_v1, wire::PersonLinkMergeCandidateProposedEventV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityResolutionEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
}

pub fn build_identity_resolution_person_match_candidate_outbox_record_v1(
    payload: PersonLinkMergeCandidateProposedEventV1,
    context: &IdentityResolutionEnvelopeContextV1,
) -> Result<OutboxRecordV1, IdentityResolutionEnvelopeBuildErrorV1> {
    let id = id16(&payload.event_id)?;
    let evidence = id16(&payload.evidence_event_id)?;
    let candidate = id16(&payload.candidate_id)?;
    if id != identity_resolution_proposal_event_id_v1(evidence, candidate)
        || !owner(&payload.logical_owner_id)
        || payload.first_person_id.len() != 16
        || payload.second_person_id.len() != 16
        || payload.first_source.is_none()
        || payload.second_source.is_none()
        || !(1..=2).contains(&payload.match_kind)
        || payload.observed_at_unix_millis <= 0
        || payload.resulting_owner_revision == 0
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.recorded_at_unix_millis < payload.observed_at_unix_millis
    {
        return Err(IdentityResolutionEnvelopeBuildErrorV1::InvalidPayload);
    }
    let partition = identity_resolution_owner_partition_id_v1(&payload.logical_owner_id)
        .map_err(|_| IdentityResolutionEnvelopeBuildErrorV1::InvalidPayload)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: IDENTITY_RESOLUTION_OWNER_ID_V1.to_owned(),
            name: IDENTITY_RESOLUTION_PERSON_MATCH_CANDIDATE_CONTRACT_NAME_V1.to_owned(),
            major: IDENTITY_RESOLUTION_CONTRACT_MAJOR_V1,
            revision: IDENTITY_RESOLUTION_CONTRACT_REVISION_V1,
            schema_sha256: IDENTITY_RESOLUTION_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: IDENTITY_RESOLUTION_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest16(
                b"identity-resolution-runtime.v1",
                context.runtime_instance_id.as_bytes(),
                IDENTITY_RESOLUTION_MODULE_ID_V1.as_bytes(),
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(ts(context.recorded_at_unix_millis)?),
        partition_key: partition.to_vec(),
        causation_message_id: evidence.to_vec(),
        correlation_id: partition.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: IDENTITY_RESOLUTION_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: IDENTITY_RESOLUTION_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(ts(payload.observed_at_unix_millis)?),
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| IdentityResolutionEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec())
        .map_err(|_| IdentityResolutionEnvelopeBuildErrorV1::InvalidEnvelope)
}

fn id16(v: &[u8]) -> Result<[u8; 16], IdentityResolutionEnvelopeBuildErrorV1> {
    v.try_into()
        .ok()
        .filter(|x: &[u8; 16]| x.iter().any(|b| *b != 0))
        .ok_or(IdentityResolutionEnvelopeBuildErrorV1::InvalidPayload)
}
fn owner(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 128
        && v.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
}
fn ts(v: i64) -> Result<Timestamp, IdentityResolutionEnvelopeBuildErrorV1> {
    if v <= 0 {
        return Err(IdentityResolutionEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(Timestamp {
        seconds: v / 1000,
        nanos: ((v % 1000) * 1_000_000)
            .try_into()
            .map_err(|_| IdentityResolutionEnvelopeBuildErrorV1::InvalidPayload)?,
    })
}
fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut h = Sha256::new();
    for v in [label, first, second] {
        h.update((v.len() as u64).to_be_bytes());
        h.update(v);
    }
    h.finalize()[..16].try_into().expect("sha")
}
