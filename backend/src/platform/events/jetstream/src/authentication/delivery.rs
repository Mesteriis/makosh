//! Event Hub-specific construction of the public runtime credential fence.

use makosh_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialDeliveryErrorV1, NatsRuntimeCredentialRecipientPublicKeyV1,
};

use crate::vault::NatsRuntimeCredentialFenceV1;

pub fn bind_runtime_credential_delivery(
    fence: &NatsRuntimeCredentialFenceV1,
    request_id: [u8; 16],
    recipient_public_key: NatsRuntimeCredentialRecipientPublicKeyV1,
) -> Result<NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialDeliveryErrorV1> {
    NatsRuntimeCredentialDeliveryBindingV1::new(NatsRuntimeCredentialDeliveryBindingInputV1 {
        logical_owner_id: fence.logical_owner_id().to_owned(),
        registration_id: fence.registration_id().to_owned(),
        runtime_instance_id: fence.runtime_instance_id().to_owned(),
        runtime_generation: fence.runtime_generation(),
        grant_epoch: fence.grant_epoch(),
        credential_revision: fence.credential_revision(),
        request_id,
        recipient_public_key,
    })
}
