//! Live Kernel-routed managed-module request into the delivery-intent workflow.

use super::*;

use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256,
    wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusV1, SubmitDeliveryIntentRequestV1,
        SubmitDeliveryIntentResponseV1,
    },
};
use makosh_kernel_control_store::{BundledManagedLaunchBinding, ManagedLaunchRecord};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, ContractReferenceV1,
    ManagedRuntimeModuleRequestRequestV1, ModuleDescriptorV1, ModuleKindV1,
};

use crate::{
    modules::{capability::module_request::ModuleRequestRouteHandlerV1, registration::registry},
    runtime::lifecycle::control::{ManagedRuntimeExpectation, ManagedRuntimeModuleRequestHandler},
};

const CALLER_CAPABILITY_ID: &str = "communication_bulk_action.delivery_intent.v1";

pub(super) fn assert_live_delivery_intent_module_request(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    conversation_id: Vec<u8>,
) {
    let contract = delivery_intent_command_contract();
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "makosh-communication-bulk-action-conformance".to_owned(),
        owner_id: "communication_bulk_action".to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: "managed-request-live".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: CALLER_CAPABILITY_ID.to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            dependencies: vec![contract.clone()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = registry::register(store.as_ref(), &descriptor_bytes)
        .expect("register live request caller");
    let grants = registry::approve_after_owner_authorization(
        store.as_ref(),
        registration.registration_id(),
        &[CALLER_CAPABILITY_ID.to_owned()],
    )
    .expect("approve live request caller");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-request-conformance",
            "workflow.communication_bulk_action.conformance",
            [8; 32],
            *registration.descriptor_sha256(),
            None,
        ))
        .expect("record live request caller binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            registration.registration_id(),
            "bulk-action-request-runtime-1",
            1,
            1,
            1,
            grants.grant_epoch(),
        ))
        .expect("record live request caller launch");
    let expectation = ManagedRuntimeExpectation::new(
        registration.registration_id(),
        "bulk-action-request-runtime-1",
        descriptor.module_id,
        1,
        grants.grant_epoch(),
        *registration.descriptor_sha256(),
        None,
    );
    let operation_id = vec![0x62; 16];
    let response = ModuleRequestRouteHandlerV1::new(Arc::clone(store), supervisor.relay_port())
        .route_module_request(
            &expectation,
            ManagedRuntimeModuleRequestRequestV1 {
                request_id: vec![0x61; 16],
                contract: Some(contract),
                request_payload: SubmitDeliveryIntentRequestV1 {
                    protocol_major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
                    operation_id: operation_id.clone(),
                    conversation_id,
                    reply_to_message_id: None,
                    body_utf8: b"managed module request body".to_vec(),
                }
                .encode_to_vec(),
                deadline_millis: 5_000,
                response_blob_capability_id: String::new(),
            },
        )
        .expect("route live delivery-intent module request");
    assert!(response.error_code.is_empty());
    let receipt = SubmitDeliveryIntentResponseV1::decode(response.response_payload.as_slice())
        .expect("decode live delivery-intent module receipt");
    assert_eq!(receipt.intent_id, operation_id);
    assert_eq!(
        receipt.status,
        DeliveryIntentStatusV1::DeliveryIntentStatusAccepted as i32
    );
    assert_eq!(
        receipt.error,
        DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32
    );
}

fn delivery_intent_command_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        name: COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}
