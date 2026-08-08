#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    AttachmentPreviewCustodyEnvelopeBuildErrorV1, AttachmentPreviewCustodyEnvelopeContextV1,
    build_attachment_preview_custody_delegated_outbox_record_v1,
    build_attachment_preview_custody_delegation_rejected_outbox_record_v1,
    build_request_attachment_preview_custody_delegation_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-attachment-preview-ingress";
pub const ATTACHMENT_PREVIEW_INGRESS_OWNER_V1: &str = "attachment_preview";
pub const ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1: &str =
    "attachment_preview_custody_delegation_requested";
pub const ATTACHMENT_PREVIEW_CUSTODY_DELEGATED_CONTRACT_NAME_V1: &str =
    "attachment_preview_custody_delegated";
pub const ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1: &str =
    "attachment_preview_custody_delegation_rejected";
pub const ATTACHMENT_PREVIEW_INGRESS_CONTRACT_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_INGRESS_CONTRACT_REVISION_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_INGRESS_MAX_IN_FLIGHT_V1: u32 = 32;
pub const ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1: u64 = 100 * 1024 * 1024;
pub const ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1: &str =
    "attachment_security.preview-delegation.v1";
pub const ATTACHMENT_PREVIEW_BLOB_TARGET_OWNER_ID_V1: &str = "attachment_preview";
pub const ATTACHMENT_PREVIEW_BLOB_TARGET_MODULE_ID_V1: &str = "makosh-attachment-preview-runtime";
pub const ATTACHMENT_PREVIEW_BLOB_TARGET_CAPABILITY_ID_V1: &str = "attachment_preview.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_preview.ingress.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_preview_ingress_schema.rs"
));

pub const ATTACHMENT_PREVIEW_INGRESS_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-preview-ingress-v1.bin"
));

#[must_use]
pub fn attachment_preview_custody_delegation_request_id_v1(
    preview_run_id: [u8; 16],
    candidate_message_id: [u8; 16],
    safety_message_id: [u8; 16],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-preview.custody-delegation-request.v1\0");
    hasher.update(preview_run_id);
    hasher.update(candidate_message_id);
    hasher.update(safety_message_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

#[must_use]
pub fn attachment_preview_custody_delegated_message_id_v1(request_id: [u8; 16]) -> [u8; 16] {
    custody_result_message_id_v1(b"delegated", request_id)
}

#[must_use]
pub fn attachment_preview_custody_delegation_rejected_message_id_v1(
    request_id: [u8; 16],
) -> [u8; 16] {
    custody_result_message_id_v1(b"rejected", request_id)
}

#[must_use]
pub fn attachment_preview_custody_delegation_requested_contract_reference_v1() -> ContractReferenceV1
{
    contract_reference(ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn attachment_preview_custody_delegated_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(ATTACHMENT_PREVIEW_CUSTODY_DELEGATED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn attachment_preview_custody_delegation_rejected_contract_reference_v1() -> ContractReferenceV1
{
    contract_reference(ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn attachment_preview_custody_delegation_requested_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        attachment_preview_custody_delegation_requested_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn attachment_preview_custody_delegation_requested_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        attachment_preview_custody_delegation_requested_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn attachment_preview_custody_delegated_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        attachment_preview_custody_delegated_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn attachment_preview_custody_delegated_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        attachment_preview_custody_delegated_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn attachment_preview_custody_delegation_rejected_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        attachment_preview_custody_delegation_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn attachment_preview_custody_delegation_rejected_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        attachment_preview_custody_delegation_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_PREVIEW_INGRESS_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: ATTACHMENT_PREVIEW_INGRESS_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_PREVIEW_INGRESS_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_PREVIEW_INGRESS_SCHEMA_SHA256.to_vec(),
    }
}

fn result_route(
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        contract,
        direction,
        requirement,
    )
}

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: ATTACHMENT_PREVIEW_INGRESS_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

fn custody_result_message_id_v1(label: &[u8], request_id: [u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-preview.custody-result.v1\0");
    hasher.update(label);
    hasher.update(request_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_and_event_directions_are_exact() {
        assert_eq!(
            ATTACHMENT_PREVIEW_BLOB_TARGET_OWNER_ID_V1,
            "attachment_preview"
        );
        assert_eq!(
            ATTACHMENT_PREVIEW_BLOB_TARGET_MODULE_ID_V1,
            "makosh-attachment-preview-runtime"
        );
        let Some(Request::EventRoute(route)) =
            attachment_preview_custody_delegation_requested_consume_request_v1().request
        else {
            panic!("event route");
        };
        assert_eq!(route.envelope_kind, DurableEnvelopeKindV1::Command as i32);
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
    }
}
