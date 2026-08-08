use makosh_attachment_security_contract::admission::{
    ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
    attachment_security_scan_candidate_observed_contract_reference_v1,
};
use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
    ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
    ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1, ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OWNER_V1, ATTACHMENT_TEXT_EXTRACTION_QUERY_CONNECT_PATH_V1,
};
use makosh_attachment_text_extraction_ingress::{
    ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_CAPABILITY_ID_V1,
    attachment_text_custody_delegated_consume_request_v1,
    attachment_text_custody_delegated_contract_reference_v1,
    attachment_text_custody_delegation_rejected_consume_request_v1,
    attachment_text_custody_delegation_rejected_contract_reference_v1,
    attachment_text_custody_delegation_requested_contract_reference_v1,
    attachment_text_custody_delegation_requested_publish_request_v1,
};
use makosh_attachment_translation_ingress::{
    ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1,
    attachment_translation_source_prepared_contract_reference_v1,
    attachment_translation_source_prepared_publish_request_v1,
    attachment_translation_source_rejected_contract_reference_v1,
    attachment_translation_source_rejected_publish_request_v1,
    attachment_translation_source_requested_consume_request_v1,
    attachment_translation_source_requested_contract_reference_v1,
};
use makosh_communications_attachment_contract::admission::{
    COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
    communication_attachment_safety_state_changed_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, DurableEnvelopeKindV1,
    EventRouteDirectionV1, EventRouteRequestV1, EventSubscriptionRequirementV1, ModuleDescriptorV1,
    ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeArtifactRequestV1, RuntimeArtifactUseV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    SettingsSchemaV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::{
    command_contract_v1, content_contract_v1, query_contract_v1, realtime_contract_v1,
};
use crate::ocr_resources::{
    ATTACHMENT_TEXT_EXTRACTION_OCR_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
};

pub const ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1: &str =
    ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_CAPABILITY_ID_V1;
pub const ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1: &str =
    "attachment_text_extraction.storage.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_CANDIDATE_CAPABILITY_ID_V1: &str =
    "attachment_text_extraction.candidate.observe.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_SAFETY_CAPABILITY_ID_V1: &str =
    "attachment_text_extraction.safety.observe.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_CUSTODY_REQUEST_CAPABILITY_ID_V1: &str =
    "attachment_text_extraction.custody-request.publish.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_CUSTODY_RESULT_CAPABILITY_ID_V1: &str =
    "attachment_text_extraction.custody-result.consume.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn attachment_text_extraction_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn attachment_text_extraction_settings_schema_bytes_v1() -> Vec<u8> {
    attachment_text_extraction_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn attachment_text_extraction_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = attachment_text_extraction_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            client_capability(),
            blob_capability(),
            consumer_capability(
                ATTACHMENT_TEXT_EXTRACTION_CANDIDATE_CAPABILITY_ID_V1,
                attachment_security_scan_candidate_observed_contract_reference_v1(),
                DurableEnvelopeKindV1::Observation,
                ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
            ),
            custody_request_capability(),
            custody_result_capability(),
            ocr_runtime_capability(),
            consumer_capability(
                ATTACHMENT_TEXT_EXTRACTION_SAFETY_CAPABILITY_ID_V1,
                communication_attachment_safety_state_changed_contract_reference_v1(),
                DurableEnvelopeKindV1::Event,
                COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
            ),
            storage_capability(),
            translation_source_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 2,
            max_connections: STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_millis: 2_000,
        }),
        display_name: "Attachment Text Extraction".to_owned(),
    }
}

fn ocr_runtime_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_OCR_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            runtime_artifact_request(
                ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
                RuntimeArtifactUseV1::ReadOnlyData,
            ),
            runtime_artifact_request(
                ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
                RuntimeArtifactUseV1::NativeExecutable,
            ),
            runtime_artifact_request(
                ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
                RuntimeArtifactUseV1::ReadOnlyData,
            ),
        ],
        ..Default::default()
    }
}

fn runtime_artifact_request(
    artifact_id: &str,
    use_kind: RuntimeArtifactUseV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::RuntimeArtifact(RuntimeArtifactRequestV1 {
            artifact_id: artifact_id.to_owned(),
            r#use: use_kind as i32,
        })),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_rpc(
                command_contract_v1(),
                ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1,
            ),
            client_rpc(
                query_contract_v1(),
                ATTACHMENT_TEXT_EXTRACTION_QUERY_CONNECT_PATH_V1,
            ),
            client_rpc(
                content_contract_v1(),
                ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_rpc(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: makosh_attachment_text_extraction_core::ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1
                    + (ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1 as u64 * 2),
                custody_scope_id: "attachment_text_extraction.content.v1".to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn consumer_capability(
    capability_id: &str,
    contract: ContractReferenceV1,
    kind: DurableEnvelopeKindV1,
    max_in_flight: u32,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(contract.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![event_route(kind, contract, max_in_flight)],
        ..Default::default()
    }
}

fn event_route(
    kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    max_in_flight: u32,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: EventRouteDirectionV1::Consume as i32,
            max_in_flight,
            subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
            max_deliver: 10,
            ack_wait_millis: 30_000,
        })),
    }
}

fn custody_request_capability() -> CapabilityDescriptorV1 {
    let contract = attachment_text_custody_delegation_requested_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_CUSTODY_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![attachment_text_custody_delegation_requested_publish_request_v1()],
        ..Default::default()
    }
}

fn custody_result_capability() -> CapabilityDescriptorV1 {
    let contracts = [
        (
            attachment_text_custody_delegated_contract_reference_v1(),
            attachment_text_custody_delegated_consume_request_v1(),
        ),
        (
            attachment_text_custody_delegation_rejected_contract_reference_v1(),
            attachment_text_custody_delegation_rejected_consume_request_v1(),
        ),
    ];
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_CUSTODY_RESULT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: contracts
            .iter()
            .map(|(contract, _)| ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
                contract: Some(contract.clone()),
                client_rpc_route: None,
                client_blob_route: None,
            })
            .collect(),
        requests: contracts.into_iter().map(|(_, request)| request).collect(),
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn translation_source_capability() -> CapabilityDescriptorV1 {
    let contracts = [
        (
            ProvidedSurfaceKindV1::DurableConsumer,
            attachment_translation_source_requested_contract_reference_v1(),
            attachment_translation_source_requested_consume_request_v1(),
        ),
        (
            ProvidedSurfaceKindV1::DurablePublisher,
            attachment_translation_source_prepared_contract_reference_v1(),
            attachment_translation_source_prepared_publish_request_v1(),
        ),
        (
            ProvidedSurfaceKindV1::DurablePublisher,
            attachment_translation_source_rejected_contract_reference_v1(),
            attachment_translation_source_rejected_publish_request_v1(),
        ),
    ];
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: contracts
            .iter()
            .map(|(kind, contract, _)| ProvidedSurfaceV1 {
                kind: *kind as i32,
                contract: Some(contract.clone()),
                client_rpc_route: None,
                client_blob_route: None,
            })
            .collect(),
        requests: contracts
            .into_iter()
            .map(|(_, _, request)| request)
            .collect(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_exact_nine_capability_workflow() {
        let descriptor = attachment_text_extraction_module_descriptor_v1("build-1");
        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.capabilities.len(), 9);
        assert_eq!(
            validate_settings_schema_v1(&attachment_text_extraction_settings_schema_v1()),
            Ok(())
        );
    }
}
