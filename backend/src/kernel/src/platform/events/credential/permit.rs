//! Turns approved Event Hub topology into a sealed-authority request.

use std::collections::BTreeSet;

use makosh_kernel_control_store::{ModuleRegistration, ModuleRegistrationState};
use makosh_runtime_protocol::v1::{
    EventsRuntimeConsumerGrantV1, IssueEventsRuntimeCredentialRequestV1,
};

use crate::platform::events::topology::EventTopologyPlanV1;

pub(crate) struct EventCredentialRequestInputV1<'a> {
    pub(crate) registration: &'a ModuleRegistration,
    pub(crate) runtime_instance_id: &'a str,
    pub(crate) runtime_generation: u64,
    pub(crate) credential_revision: u64,
    pub(crate) request_id: [u8; 16],
    pub(crate) recipient_public_key_x25519: [u8; 32],
    pub(crate) ttl_seconds: u32,
    pub(crate) topology: &'a EventTopologyPlanV1,
}

pub(crate) fn derive_credential_request(
    input: EventCredentialRequestInputV1<'_>,
) -> Result<IssueEventsRuntimeCredentialRequestV1, EventCredentialPermitErrorV1> {
    validate_input(
        input.registration,
        input.runtime_instance_id,
        input.runtime_generation,
        input.credential_revision,
        &input.request_id,
        input.ttl_seconds,
    )?;
    let publish_subjects = subjects_for_registration(
        input.topology.publishers(),
        input.registration.registration_id(),
        input.registration.grant_epoch(),
        |permit| permit.subject().as_str(),
    );
    let consumer_grants = consumer_grants_for_registration(
        input.topology.consumers(),
        input.registration.registration_id(),
        input.registration.grant_epoch(),
    );
    (!publish_subjects.is_empty() || !consumer_grants.is_empty())
        .then_some(IssueEventsRuntimeCredentialRequestV1 {
            logical_owner_id: input.registration.owner_id().to_owned(),
            registration_id: input.registration.registration_id().to_owned(),
            runtime_instance_id: input.runtime_instance_id.to_owned(),
            runtime_generation: input.runtime_generation,
            grant_epoch: input.registration.grant_epoch(),
            credential_revision: input.credential_revision,
            publish_subjects,
            subscribe_subjects: Vec::new(),
            ttl_seconds: input.ttl_seconds,
            request_id: input.request_id.to_vec(),
            recipient_public_key_x25519: input.recipient_public_key_x25519.to_vec(),
            consumer_grants,
        })
        .ok_or(EventCredentialPermitErrorV1::NoApprovedRoute)
}

fn consumer_grants_for_registration(
    permits: &[crate::platform::events::topology::EventConsumerPlanV1],
    registration_id: &str,
    grant_epoch: u64,
) -> Vec<EventsRuntimeConsumerGrantV1> {
    let mut values = std::collections::BTreeMap::new();
    for permit in permits.iter().filter(|permit| {
        permit.registration_id() == registration_id && permit.grant_epoch() == grant_epoch
    }) {
        let grant = EventsRuntimeConsumerGrantV1 {
            durable_name: permit.durable_name().to_owned(),
            filter_subject: permit.subject().as_str(),
        };
        values.insert(
            (grant.durable_name.clone(), grant.filter_subject.clone()),
            grant,
        );
    }
    values.into_values().collect()
}

fn validate_input(
    registration: &ModuleRegistration,
    runtime_instance_id: &str,
    runtime_generation: u64,
    credential_revision: u64,
    request_id: &[u8; 16],
    ttl_seconds: u32,
) -> Result<(), EventCredentialPermitErrorV1> {
    (registration.state() == ModuleRegistrationState::Approved
        && valid_id(registration.owner_id())
        && valid_id(runtime_instance_id)
        && runtime_generation > 0
        && credential_revision > 0
        && request_id.iter().any(|byte| *byte != 0)
        && (1..=600).contains(&ttl_seconds))
    .then_some(())
    .ok_or(EventCredentialPermitErrorV1::InvalidRuntimeFence)
}

fn subjects_for_registration<T>(
    permits: &[T],
    registration_id: &str,
    grant_epoch: u64,
    subject: impl Fn(&T) -> String,
) -> Vec<String>
where
    T: RegisteredEventPermit,
{
    permits
        .iter()
        .filter(|permit| {
            permit.registration_id() == registration_id && permit.grant_epoch() == grant_epoch
        })
        .map(subject)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

trait RegisteredEventPermit {
    fn registration_id(&self) -> &str;
    fn grant_epoch(&self) -> u64;
}

impl RegisteredEventPermit for crate::platform::events::topology::EventPublisherPermitPlanV1 {
    fn registration_id(&self) -> &str {
        self.registration_id()
    }

    fn grant_epoch(&self) -> u64 {
        self.grant_epoch()
    }
}

impl RegisteredEventPermit for crate::platform::events::topology::EventConsumerPlanV1 {
    fn registration_id(&self) -> &str {
        self.registration_id()
    }

    fn grant_epoch(&self) -> u64 {
        self.grant_epoch()
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EventCredentialPermitErrorV1 {
    InvalidRuntimeFence,
    NoApprovedRoute,
}
