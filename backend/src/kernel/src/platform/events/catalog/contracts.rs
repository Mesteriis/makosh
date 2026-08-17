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
        let schema_sha256 = canonical_schema_sha256(route);
        let contract = contracts
            .entry(key)
            .or_insert_with(|| EventCatalogContractV1 {
                envelope_kind: route.envelope_kind(),
                owner: route.contract_owner().to_owned(),
                name: route.contract_name().to_owned(),
                major: route.contract_major(),
                revision: route.contract_revision(),
                schema_sha256,
                publishers: Vec::new(),
                consumers: Vec::new(),
            });
        if contract.revision != route.contract_revision() || contract.schema_sha256 != schema_sha256
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

fn canonical_schema_sha256(route: &ModuleEventRouteRequestV1) -> [u8; 32] {
    let schema_sha256 = *route.contract_schema_sha256();
    let alias = match (
        route.contract_owner(),
        route.contract_name(),
        route.contract_major(),
        route.contract_revision(),
    ) {
        ("communications", "call_evidence_observed", 1, 2) => Some((
            "f3c216395f5c5956660e2d25a5548d740b253d9891e433d65a95421163eaaf93",
            "a9de49dbe36295ab2d959d604756e58af97b304b0090d205c22738a6a6fce5e7",
        )),
        ("communications", "communication_attachment_anchor_recorded", 1, 1) => Some((
            "187587925677ca88afa545037bd80af4b2c9c63fd028ff9d2d33e6be0b4efeb4",
            "0b413fae8f2b692d515f66f6283ec8bdec5d0ee0861188d5826cbf57acdf42e2",
        )),
        ("communications", "communication_attachment_blob_admission_observed", 1, 1) => Some((
            "98c16eae9314da0eb15ade794b6e0ebcca1ad7fbd7bd741c65f4ce69f530388a",
            "fbe09c9e84488c030727e08711deb4e5ea3106a0982971ac191b3d35542b0479",
        )),
        ("communications", "communication_attachment_safety_state_changed", 1, 1) => Some((
            "905ae9d12053462d2abf89ae734d1fd0b12e6b5f94df9f148d543f289044b21c",
            "691a2117d0cb3de11b85e3c994cda9eaca6d1e109de19bb7b4a672e4fc4243cf",
        )),
        ("communications", "communication_attachment_safety_verdict_observed", 1, 1) => Some((
            "9ea1faaa3f1980b8723cd431b51f505b9f28ed95f4678804d9fb5df28c3467f6",
            "a0d25cb180aaa9f51a3c22acac62fe5413ff34af731b1110b74e91a6ccaadbd0",
        )),
        ("communications", "communication_observed", 1, 3) => Some((
            "c1367fc89b5e44933e29c2d9da829e4b713362cdc2ea8ddddfc8a44fcfe61fc6",
            "f72cc96c042e1e7f75196f2c989357564ab4e50fc0f52d1b2ca7b0e62b1fd3d2",
        )),
        (
            "communications",
            "communications_retained_evidence_replay_command"
            | "communications_retained_evidence_replay_result",
            1,
            2,
        ) => Some((
            "12059e4ed50fd4e7a24b2c243f844457b67061f48cfb094e7e7641e385dd0ace",
            "9252059812e3c82a2c85db1ed25323003fff40d1bff75414f3c58626087582de",
        )),
        (
            "communications",
            "evidence_export_prepare" | "evidence_export_prepared" | "evidence_export_rejected",
            1,
            1,
        ) => Some((
            "982070745aac8e3e1b132a9fa1985af41712fd371928fafd88ad1f03ad373c85",
            "67deb85692da7dc395c253f977ba361b85de756a666490babe66be14fe12c28a",
        )),
        ("scheduler", "job_receipt" | "schedule_control", 1, 1) => Some((
            "3f9bb7b2dea5e0a78d4c9fd68cf5867d366b2b70f21f81afe69d31b864767415",
            "c5600521888d9f7689b3c65e61ab726c9ddf165fd6eab33fe88c5ca720953710",
        )),
        _ => None,
    };
    let Some((legacy, current)) = alias else {
        return schema_sha256;
    };
    if schema_matches_hex(&schema_sha256, legacy) || schema_matches_hex(&schema_sha256, current) {
        decode_schema_hex(current)
    } else {
        schema_sha256
    }
}

fn schema_matches_hex(schema_sha256: &[u8; 32], hex: &str) -> bool {
    hex.len() == 64
        && hex
            .as_bytes()
            .chunks_exact(2)
            .zip(schema_sha256)
            .all(|(encoded, actual)| decode_hex_byte(encoded) == Some(*actual))
}

fn decode_schema_hex(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64, "hard-coded schema digest must be 32 bytes");
    let mut decoded = [0; 32];
    for (target, encoded) in decoded.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        *target = decode_hex_byte(encoded).expect("hard-coded schema digest must be hexadecimal");
    }
    decoded
}

fn decode_hex_byte(encoded: &[u8]) -> Option<u8> {
    let [high, low] = encoded else {
        return None;
    };
    Some(hex_nibble(*high)? << 4 | hex_nibble(*low)?)
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::{canonical_schema_sha256, decode_schema_hex};
    use makosh_kernel_control_store::{
        ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1,
        ModuleEventRouteRequestV1,
    };

    fn route(name: &str, revision: u32, schema_sha256: [u8; 32]) -> ModuleEventRouteRequestV1 {
        ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
            registration_id: "registration".to_owned(),
            capability_id: "capability".to_owned(),
            envelope_kind: ModuleEventEnvelopeKindV1::Event,
            contract_owner: "communications".to_owned(),
            contract_name: name.to_owned(),
            contract_major: 1,
            contract_revision: revision,
            contract_schema_sha256: schema_sha256,
            direction: ModuleEventRouteDirectionV1::Publish,
            max_in_flight: 1,
            delivery_policy: None,
        })
    }

    #[test]
    fn canonicalizes_only_the_exact_clean_room_schema_alias() {
        let legacy =
            decode_schema_hex("c1367fc89b5e44933e29c2d9da829e4b713362cdc2ea8ddddfc8a44fcfe61fc6");
        let current =
            decode_schema_hex("f72cc96c042e1e7f75196f2c989357564ab4e50fc0f52d1b2ca7b0e62b1fd3d2");
        assert_eq!(
            canonical_schema_sha256(&route("communication_observed", 3, legacy)),
            current,
        );
        assert_eq!(
            canonical_schema_sha256(&route("communication_observed", 3, current)),
            current,
        );
        assert_eq!(
            canonical_schema_sha256(&route("other_contract", 3, legacy)),
            legacy,
        );
        assert_eq!(
            canonical_schema_sha256(&route("communication_observed", 2, legacy)),
            legacy,
        );
    }
}
