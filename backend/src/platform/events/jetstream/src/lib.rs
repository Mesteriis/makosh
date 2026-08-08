//! JetStream transport adapter for opaque Макошь durable-envelope bytes.
//!
//! The adapter owns NATS protocol details only. Kernel Event Hub owns catalog
//! authority and modules retain their PostgreSQL outbox/inbox state.

pub mod authentication;
pub mod connection;
pub mod resolver;
pub mod subjects;
pub mod topology;
pub mod vault;

pub use authentication::{
    NatsJwtConsumerGrantV1, NatsJwtIssueErrorV1, NatsJwtPermissionSetV1,
    NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialDeliveryErrorV1,
    NatsRuntimeCredentialDeliveryV1, NatsRuntimeCredentialRecipientPublicKeyV1,
    NatsRuntimeCredentialRecipientV1, RuntimeNatsJwtCredentialV1, RuntimeNatsJwtIssuerV1,
    bind_runtime_credential_delivery,
};
pub use connection::{
    EventHubJetStreamConnection, JetStreamClient, ManagedRuntimeEventAccessErrorV1,
    ManagedRuntimeEventAccessV1, NatsPasswordCredentialV1, PublishReceipt,
    RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimeOutboxPublisherV1,
    RuntimePublishPermitV1, RuntimePullDeliveryErrorV1, RuntimePullDeliveryV1,
    RuntimeSchedulerReceiptDeliveryV1, RuntimeSchedulerReceiptPortV1, RuntimeSubscribePermitV1,
    canonical_message_id, receive_runtime_pull_delivery, request_managed_runtime_event_access,
    request_managed_runtime_event_access_v2, try_receive_runtime_pull_delivery,
};
pub use resolver::{
    NatsAccountJwtUpdateV1, NatsResolverAccountJwtPublisherV1, NatsResolverSystemCredentialsV1,
    ResolverUpdateErrorV1,
};
pub use subjects::{DurableSubjectV1, SubjectError};
pub use topology::{
    ConsumerBudgetV1, ConsumerSpecV1, EventHubTopologyPlanV1, EventHubTopologyPlanViolationV1,
    StreamBudgetV1, StreamKindV1, StreamSpecV1,
};
pub use vault::{
    EventHubCredentialFenceV1, EventHubCredentialLeaseAdapterV1, EventHubCredentialLeaseErrorV1,
    NatsAccountSignerFenceV1, NatsAccountSignerLeaseAdapterV1, NatsAccountSignerLeaseErrorV1,
    NatsCredentialLeaseAdapterV1, NatsCredentialLeaseErrorV1, NatsResolverCredentialFenceV1,
    NatsResolverCredentialLeaseAdapterV1, NatsResolverCredentialLeaseErrorV1,
    NatsRuntimeCredentialFenceV1, NatsVaultRouteContextV1, NatsVaultRouteFailureV1,
    NatsVaultRoutePortV1,
};
