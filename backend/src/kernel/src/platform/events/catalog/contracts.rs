//! Canonical Event Hub contracts assembled from approved route entries.

use std::collections::BTreeMap;

use makosh_kernel_control_store::{
    ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1,
    ModuleEventRouteRequestV1,
};

use super::entries::EventCatalogEntryV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCatalogParticipantV1 {
    registration_id: String,
    module_id: String,
    grant_epoch: u64,
    capability_id: String,
    max_in_flight: u16,
    delivery_policy: Option<ModuleEventDeliveryPolicyV1>,
}

impl EventCatalogParticipantV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn module_id(&self) -> &str {
        &self.module_id
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub const fn max_in_flight(&self) -> u16 {
        self.max_in_flight
    }

    #[must_use]
    pub const fn delivery_policy(&self) -> Option<ModuleEventDeliveryPolicyV1> {
        self.delivery_policy
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCatalogContractV1 {
    envelope_kind: ModuleEventEnvelopeKindV1,
    owner: String,
    name: String,
    major: u32,
    revision: u32,
    schema_sha256: [u8; 32],
    publishers: Vec<EventCatalogParticipantV1>,
    consumers: Vec<EventCatalogParticipantV1>,
}

impl EventCatalogContractV1 {
    #[must_use]
    pub const fn envelope_kind(&self) -> ModuleEventEnvelopeKindV1 {
        self.envelope_kind
    }

    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    #[must_use]
    pub const fn schema_sha256(&self) -> &[u8; 32] {
        &self.schema_sha256
    }

    #[must_use]
    pub fn publishers(&self) -> &[EventCatalogParticipantV1] {
        &self.publishers
    }

    #[must_use]
    pub fn consumers(&self) -> &[EventCatalogParticipantV1] {
        &self.consumers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCatalogContractErrorV1 {
    IncompatibleRevisionOrSchema,
}

pub(super) fn build(
    entries: Vec<EventCatalogEntryV1>,
) -> Result<Vec<EventCatalogContractV1>, EventCatalogContractErrorV1> {
    let mut contracts = BTreeMap::<ContractKey, EventCatalogContractV1>::new();
    for entry in entries {
        let route = entry.route();
        let key = ContractKey::from(route);
        let participant = participant(&entry);
        let contract = contracts
            .entry(key)
            .or_insert_with(|| EventCatalogContractV1 {
                envelope_kind: route.envelope_kind(),
                owner: route.contract_owner().to_owned(),
                name: route.contract_name().to_owned(),
                major: route.contract_major(),
                revision: route.contract_revision(),
                schema_sha256: *route.contract_schema_sha256(),
                publishers: Vec::new(),
                consumers: Vec::new(),
            });
        if contract.revision != route.contract_revision()
            || contract.schema_sha256 != *route.contract_schema_sha256()
        {
            return Err(EventCatalogContractErrorV1::IncompatibleRevisionOrSchema);
        }
        match route.direction() {
            ModuleEventRouteDirectionV1::Publish => contract.publishers.push(participant),
            ModuleEventRouteDirectionV1::Consume => contract.consumers.push(participant),
        }
    }
    for contract in contracts.values_mut() {
        sort_participants(&mut contract.publishers);
        sort_participants(&mut contract.consumers);
    }
    Ok(contracts.into_values().collect())
}

fn participant(entry: &EventCatalogEntryV1) -> EventCatalogParticipantV1 {
    EventCatalogParticipantV1 {
        registration_id: entry.registration_id().to_owned(),
        module_id: entry.module_id().to_owned(),
        grant_epoch: entry.grant_epoch(),
        capability_id: entry.capability_id().to_owned(),
        max_in_flight: entry.route().max_in_flight(),
        delivery_policy: entry.route().delivery_policy(),
    }
}

fn sort_participants(participants: &mut [EventCatalogParticipantV1]) {
    participants.sort_by(|left, right| {
        (
            left.registration_id.as_str(),
            left.capability_id.as_str(),
            left.module_id.as_str(),
        )
            .cmp(&(
                right.registration_id.as_str(),
                right.capability_id.as_str(),
                right.module_id.as_str(),
            ))
    });
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct ContractKey {
    envelope_kind: i64,
    owner: String,
    name: String,
    major: u32,
}

impl From<&ModuleEventRouteRequestV1> for ContractKey {
    fn from(route: &ModuleEventRouteRequestV1) -> Self {
        Self {
            envelope_kind: route.envelope_kind().as_i64(),
            owner: route.contract_owner().to_owned(),
            name: route.contract_name().to_owned(),
            major: route.contract_major(),
        }
    }
}
