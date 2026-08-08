use makosh_attachment_preview_evidence_replay_api::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CAPABILITY_ID_V1,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1, ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1,
};
use makosh_communications_retained_evidence_replay_contract::{
    communications_replay_command_contract_reference_v1,
    communications_replay_command_publish_request_v1,
    communications_replay_result_consume_request_v1,
    communications_replay_result_contract_reference_v1,
};
use makosh_mail_retained_evidence_replay_contract::{
    mail_replay_command_contract_reference_v1, mail_replay_command_publish_request_v1,
    mail_replay_result_consume_request_v1, mail_replay_result_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1,
    ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1,
    StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::client_command_contract_v1;

pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1: &str =
    "attachment_preview_evidence_replay.storage.v1";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_COMMUNICATIONS_COMMAND_CAPABILITY_ID_V1: &str =
    "attachment_preview_evidence_replay.communications-command.publish.v1";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_COMMUNICATIONS_RESULT_CAPABILITY_ID_V1: &str =
    "attachment_preview_evidence_replay.communications-result.consume.v1";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAIL_COMMAND_CAPABILITY_ID_V1: &str =
    "attachment_preview_evidence_replay.mail-command.publish.v1";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAIL_RESULT_CAPABILITY_ID_V1: &str =
    "attachment_preview_evidence_replay.mail-result.consume.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn attachment_preview_evidence_replay_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn attachment_preview_evidence_replay_settings_schema_bytes_v1() -> Vec<u8> {
    attachment_preview_evidence_replay_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn attachment_preview_evidence_replay_module_descriptor_v1(
    build_id: &str,
) -> ModuleDescriptorV1 {
    let settings = attachment_preview_evidence_replay_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1.to_owned(),
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
            event_capability(
                ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_COMMUNICATIONS_COMMAND_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                communications_replay_command_contract_reference_v1(),
                communications_replay_command_publish_request_v1(),
            ),
            event_capability(
                ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_COMMUNICATIONS_RESULT_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurableConsumer,
                communications_replay_result_contract_reference_v1(),
                communications_replay_result_consume_request_v1(),
            ),
            event_capability(
                ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAIL_COMMAND_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                mail_replay_command_contract_reference_v1(),
                mail_replay_command_publish_request_v1(),
            ),
            event_capability(
                ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAIL_RESULT_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurableConsumer,
                mail_replay_result_contract_reference_v1(),
                mail_replay_result_consume_request_v1(),
            ),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Attachment Preview Evidence Replay".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(client_command_contract_v1()),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1.to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn event_capability(
    capability_id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: kind as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![request],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
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
    fn descriptor_is_one_client_two_publish_two_consume_and_owner_local_storage() {
        let descriptor = attachment_preview_evidence_replay_module_descriptor_v1("build-1");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&attachment_preview_evidence_replay_settings_schema_v1())
            .expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.capabilities.len(), 6);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .flat_map(|value| &value.provides)
                .filter(|value| value.kind == ProvidedSurfaceKindV1::DurablePublisher as i32)
                .count(),
            2
        );
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .flat_map(|value| &value.provides)
                .filter(|value| value.kind == ProvidedSurfaceKindV1::DurableConsumer as i32)
                .count(),
            2
        );
    }
}
