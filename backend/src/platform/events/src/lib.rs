//! Canonical durable-envelope Protobuf contract.

pub mod delivery;
pub mod runtime;
pub mod validation;

pub use runtime::credential::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialDeliveryErrorV1, NatsRuntimeCredentialDeliveryV1,
    NatsRuntimeCredentialRecipientPublicKeyV1, NatsRuntimeCredentialRecipientV1,
    RuntimeNatsJwtCredentialV1,
};
pub use validation::envelope;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/makosh.events.v1.rs"));
}
