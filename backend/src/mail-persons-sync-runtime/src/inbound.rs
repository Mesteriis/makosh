use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1},
    validation::envelope::validate_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

#[derive(Clone, Copy)]
pub(crate) struct ExactInboundIdentityV1<'a> {
    pub contract: &'a ContractReferenceV1,
    pub source_module_id: &'a str,
    pub actor_kind: ActorKindV1,
}

pub(crate) fn validate_exact_inbound_identity_v1(
    envelope: &DurableEnvelopeV1,
    record: &OutboxRecordV1,
    expected: ExactInboundIdentityV1<'_>,
) -> Result<(), ()> {
    validate_envelope_v1(envelope).map_err(|_| ())?;
    let contract = envelope.contract.as_ref().ok_or(())?;
    let source = envelope.source.as_ref().ok_or(())?;
    let actor = envelope.actor.as_ref().ok_or(())?;
    let fence = envelope.source_fence.as_ref().ok_or(())?;
    if contract.owner != expected.contract.owner
        || contract.name != expected.contract.name
        || contract.major != expected.contract.major
        || contract.revision != expected.contract.revision
        || contract.schema_sha256 != expected.contract.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || source.module_id != expected.source_module_id
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != expected.actor_kind as i32
        || actor.actor_id != expected.source_module_id.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != expected.source_module_id.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        Err(())
    } else {
        Ok(())
    }
}
