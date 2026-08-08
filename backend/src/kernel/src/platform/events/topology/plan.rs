//! Deterministic desired topology with no JetStream implementation dependency.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use makosh_kernel_control_store::{
    ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1, ModuleEventSubscriptionRequirementV1,
    PlatformEventHubTopologyV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

use crate::platform::events::catalog::{EventCatalogContractV1, EventCatalogParticipantV1};

use super::subject::{EventStreamKindV1, EventSubjectV1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventStreamPlanV1 {
    kind: EventStreamKindV1,
    max_bytes: u64,
    max_age_millis: u64,
    replicas: u8,
}

impl EventStreamPlanV1 {
    #[must_use]
    pub const fn kind(&self) -> EventStreamKindV1 {
        self.kind
    }

    #[must_use]
    pub const fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    #[must_use]
    pub const fn max_age_millis(&self) -> u64 {
        self.max_age_millis
    }

    #[must_use]
    pub const fn replicas(&self) -> u8 {
        self.replicas
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventPublisherPermitPlanV1 {
    registration_id: String,
    capability_id: String,
    grant_epoch: u64,
    subject: EventSubjectV1,
}

impl EventPublisherPermitPlanV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub fn subject(&self) -> &EventSubjectV1 {
        &self.subject
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventConsumerPlanV1 {
    registration_id: String,
    capability_id: String,
    grant_epoch: u64,
    durable_name: String,
    subject: EventSubjectV1,
    max_in_flight: u16,
    delivery_policy: ModuleEventDeliveryPolicyV1,
    contract: ContractReferenceV1,
}

impl EventConsumerPlanV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub fn durable_name(&self) -> &str {
        &self.durable_name
    }

    #[must_use]
    pub fn subject(&self) -> &EventSubjectV1 {
        &self.subject
    }

    #[must_use]
    pub const fn max_in_flight(&self) -> u16 {
        self.max_in_flight
    }

    #[must_use]
    pub const fn delivery_policy(&self) -> ModuleEventDeliveryPolicyV1 {
        self.delivery_policy
    }

    #[must_use]
    pub fn contract(&self) -> &ContractReferenceV1 {
        &self.contract
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTopologyPlanV1 {
    streams: Vec<EventStreamPlanV1>,
    publishers: Vec<EventPublisherPermitPlanV1>,
    consumers: Vec<EventConsumerPlanV1>,
}

impl EventTopologyPlanV1 {
    pub fn from_contracts(
        contracts: &[EventCatalogContractV1],
        configuration: &PlatformEventHubTopologyV1,
    ) -> Result<Self, String> {
        let mut streams = BTreeMap::new();
        let mut publishers = Vec::new();
        let mut consumers = Vec::new();
        for contract in contracts {
            let subject = subject(contract)?;
            streams.entry(subject.kind()).or_insert(stream_plan(
                contract.envelope_kind(),
                subject.kind(),
                configuration,
            )?);
            publishers.extend(
                contract
                    .publishers()
                    .iter()
                    .map(|participant| publisher(participant, &subject)),
            );
            consumers.extend(
                contract
                    .consumers()
                    .iter()
                    .map(|participant| consumer(participant, &subject, contract))
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        sort_publishers(&mut publishers);
        sort_consumers(&mut consumers);
        Ok(Self {
            streams: streams.into_values().collect(),
            publishers,
            consumers,
        })
    }

    #[must_use]
    pub fn streams(&self) -> &[EventStreamPlanV1] {
        &self.streams
    }

    #[must_use]
    pub fn publishers(&self) -> &[EventPublisherPermitPlanV1] {
        &self.publishers
    }

    #[must_use]
    pub fn consumers(&self) -> &[EventConsumerPlanV1] {
        &self.consumers
    }
}

fn stream_plan(
    envelope_kind: ModuleEventEnvelopeKindV1,
    kind: EventStreamKindV1,
    configuration: &PlatformEventHubTopologyV1,
) -> Result<EventStreamPlanV1, String> {
    let budget = configuration
        .budget_for(envelope_kind)
        .ok_or_else(|| "Event Hub stream budget is unavailable".to_owned())?;
    Ok(EventStreamPlanV1 {
        kind,
        max_bytes: budget.max_bytes(),
        max_age_millis: budget.max_age_millis(),
        replicas: budget.replicas(),
    })
}

fn subject(contract: &EventCatalogContractV1) -> Result<EventSubjectV1, String> {
    EventSubjectV1::new(
        EventStreamKindV1::from_envelope_kind(contract.envelope_kind()),
        contract.owner(),
        contract.name(),
        contract.major(),
    )
}

fn publisher(
    participant: &EventCatalogParticipantV1,
    subject: &EventSubjectV1,
) -> EventPublisherPermitPlanV1 {
    EventPublisherPermitPlanV1 {
        registration_id: participant.registration_id().to_owned(),
        capability_id: participant.capability_id().to_owned(),
        grant_epoch: participant.grant_epoch(),
        subject: subject.clone(),
    }
}

fn consumer(
    participant: &EventCatalogParticipantV1,
    subject: &EventSubjectV1,
    contract: &EventCatalogContractV1,
) -> Result<EventConsumerPlanV1, String> {
    let delivery_policy = participant
        .delivery_policy()
        .ok_or_else(|| "Event consumer lacks declared delivery policy".to_owned())?;
    matches!(
        delivery_policy.requirement(),
        ModuleEventSubscriptionRequirementV1::Required
            | ModuleEventSubscriptionRequirementV1::Optional
    )
    .then_some(EventConsumerPlanV1 {
        registration_id: participant.registration_id().to_owned(),
        capability_id: participant.capability_id().to_owned(),
        grant_epoch: participant.grant_epoch(),
        durable_name: durable_name(participant, subject),
        subject: subject.clone(),
        max_in_flight: participant.max_in_flight(),
        delivery_policy,
        contract: ContractReferenceV1 {
            owner: contract.owner().to_owned(),
            name: contract.name().to_owned(),
            major: contract.major(),
            revision: contract.revision(),
            schema_sha256: contract.schema_sha256().to_vec(),
        },
    })
    .ok_or_else(|| "Event consumer has an invalid delivery policy".to_owned())
}

fn durable_name(participant: &EventCatalogParticipantV1, subject: &EventSubjectV1) -> String {
    let mut digest = Sha256::new();
    digest.update(b"makosh-event-consumer-v1\0");
    digest.update(participant.registration_id().as_bytes());
    digest.update(b"\0");
    digest.update(participant.capability_id().as_bytes());
    digest.update(b"\0");
    digest.update(subject.as_str().as_bytes());
    let hash = digest.finalize();
    let mut output = String::with_capacity(70);
    output.push_str("event-");
    for byte in hash {
        output.push(hex(byte >> 4));
        output.push(hex(byte & 0x0f));
    }
    output
}

fn hex(value: u8) -> char {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    char::from(DIGITS[usize::from(value)])
}

fn sort_publishers(plans: &mut [EventPublisherPermitPlanV1]) {
    plans.sort_by(|left, right| {
        (
            left.registration_id.as_str(),
            left.capability_id.as_str(),
            left.subject.as_str(),
        )
            .cmp(&(
                right.registration_id.as_str(),
                right.capability_id.as_str(),
                right.subject.as_str(),
            ))
    });
}

fn sort_consumers(plans: &mut [EventConsumerPlanV1]) {
    plans.sort_by(|left, right| {
        (left.subject.kind(), left.durable_name.as_str())
            .cmp(&(right.subject.kind(), right.durable_name.as_str()))
    });
}
