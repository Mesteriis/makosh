//! Exact, still-unadmitted descriptor for the Mail integration runtime.
//!
//! This artifact describes the smallest Mail-owned capability set. It does
//! not register Mail in the production inventory or grant any capability.

use makosh_attachment_security_contract::admission::attachment_security_scan_candidate_observed_publish_request_v1;
use makosh_communications_attachment_contract::admission::{
    communication_attachment_anchor_recorded_contract_reference_v1,
    communication_attachment_blob_admission_observed_publish_request_v1,
    communication_attachment_safety_state_changed_contract_reference_v1,
};
use makosh_communications_ingress::admission::{
    COMMUNICATION_OBSERVED_MAX_IN_FLIGHT, communication_observed_publish_request_v1,
};
use makosh_mail_address_book_contract::{
    MAIL_PERSON_SOURCE_CAPABILITY_ID_V1, MailPersonSourceContractV1,
};
use makosh_mail_api::client_contract::{
    MAIL_CLIENT_CONTRACT_MAJOR, MAIL_CLIENT_CONTRACT_REVISION, MAIL_CLIENT_DESCRIPTOR_SET_V1,
    MAIL_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1,
    MAIL_OPERATIONAL_REALTIME_CAPABILITY_ID_V1, MailClientContractV1,
};
pub use makosh_mail_api::client_contract::{MAIL_MODULE_ID, MAIL_OWNER_ID};
pub use makosh_mail_delivery_intent_contract::MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1;
use makosh_mail_delivery_intent_contract::{
    mail_delivery_intent_execute_consume_request_v1,
    mail_delivery_intent_rejected_publish_request_v1,
    mail_delivery_intent_succeeded_publish_request_v1,
};
use makosh_mail_retained_evidence_replay_contract::{
    mail_replay_command_consume_request_v1, mail_replay_result_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, DurableEnvelopeKindV1,
    EventRouteDirectionV1, EventRouteRequestV1, EventSubscriptionRequirementV1, ModuleDescriptorV1,
    ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1, VaultActionV1,
    VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

use crate::settings::{
    MAIL_SETTINGS_SCHEMA_MAJOR_V2, MAIL_SETTINGS_SCHEMA_REVISION_V2, mail_settings_schema_bytes_v2,
};
use makosh_runtime_protocol::SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID;

pub const MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID: &str =
    "mail.attachment.scan-candidate.publish.v1";
pub const MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID: &str = "mail.attachment-anchor.consume.v1";
pub const MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID: &str =
    "mail.attachment-safety-state.consume.v1";
pub const MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID: &str =
    "mail.attachment-blob-admission.publish.v1";
pub const MAIL_BLOB_CAPABILITY_ID: &str = "mail.blob.v1";
pub const MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID: &str =
    "mail.communication-observed.publish.v1";
pub const MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID: &str = "mail.gmail.credentials.v1";
pub const MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID: &str =
    "mail.gmail.credential-lifecycle.v1";
pub const MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID: &str =
    "mail.gmail.refresh-credential-lifecycle.v1";
pub const MAIL_GMAIL_OAUTH_REFRESH_CREDENTIALS_CAPABILITY_ID: &str =
    "mail.gmail.oauth-refresh.credentials.v1";
pub const MAIL_GMAIL_OAUTH_SETUP_CREDENTIALS_CAPABILITY_ID: &str =
    "mail.gmail.oauth-setup.credentials.v1";
pub const MAIL_GMAIL_OAUTH_CLIENT_SECRET_PROVISIONING_CAPABILITY_ID: &str =
    "mail.gmail.oauth-client-secret.credential-provisioning.v1";
pub const MAIL_IMAP_CREDENTIALS_CAPABILITY_ID: &str = "mail.imap.credentials.v1";
pub const MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID: &str = "mail.imap.credential-lifecycle.v1";
pub const MAIL_IMAP_CREDENTIAL_PROVISIONING_CAPABILITY_ID: &str =
    "mail.imap.credential-provisioning.v1";
pub const MAIL_SMTP_CREDENTIALS_CAPABILITY_ID: &str = "mail.smtp.credentials.v1";
pub const MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID: &str = "mail.smtp.credential-lifecycle.v1";
pub const MAIL_SMTP_CREDENTIAL_PROVISIONING_CAPABILITY_ID: &str =
    "mail.smtp.credential-provisioning.v1";
pub const MAIL_STORAGE_CAPABILITY_ID: &str = "mail.storage.v1";
pub const MAIL_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID: &str = "mail.retained-evidence-replay.v1";
/// Cumulative durable budget for the complete Mail-owned Blob custody scope.
/// Per-message attachment limits are enforced independently by Mail contracts.
pub const MAIL_BLOB_CUSTODY_QUOTA_BYTES: u64 = 1 << 30;
pub const MAIL_ATTACHMENT_BLOB_CUSTODY_SCOPE_ID: &str = "mail.attachment.content.v1";
pub const MAIL_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const MAIL_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const MAIL_EVENT_MAX_DELIVER: u32 = 8;
pub const MAIL_EVENT_ACK_WAIT_MILLIS: u32 = 30_000;
pub const MAIL_CREDENTIAL_LEASE_TTL_SECONDS: u32 = 60;

#[must_use]
pub fn mail_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        mail_client_capability_v1(MailClientContractV1::AccountCatalog),
        mail_client_capability_v1(MailClientContractV1::AccountCredentialBind),
        mail_client_capability_v1(MailClientContractV1::AccountDelete),
        mail_client_capability_v1(MailClientContractV1::AccountLifecycleQuery),
        mail_client_capability_v1(MailClientContractV1::AccountLifecycleRetry),
        mail_client_capability_v1(MailClientContractV1::AccountQuery),
        mail_client_capability_v1(MailClientContractV1::AccountRetire),
        mail_attachment_anchor_consume_capability_v1(),
        mail_attachment_blob_admission_publish_capability_v1(),
        mail_attachment_safety_state_consume_capability_v1(),
        mail_attachment_scan_candidate_publish_capability_v1(),
        mail_blob_capability_v1(),
        mail_communication_observed_publish_capability_v1(),
        mail_client_capability_v1(MailClientContractV1::CompositionCommand),
        mail_client_capability_v1(MailClientContractV1::CompositionQuery),
        mail_delivery_intent_capability_v1(),
        mail_client_capability_v1(MailClientContractV1::DeliveryQuery),
        mail_client_capability_v1(MailClientContractV1::Delivery),
        mail_provider_credential_lifecycle_capability_v1(
            MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
            "mail_gmail_access_token",
            VaultSecretClassV1::ProviderCredential,
        ),
        mail_provider_credential_capability_v1(
            MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID,
            "mail_gmail_access_token",
        ),
        mail_provider_credential_provisioning_capability_v1(
            MAIL_GMAIL_OAUTH_CLIENT_SECRET_PROVISIONING_CAPABILITY_ID,
            "mail_gmail_oauth_client_secret",
            VaultSecretClassV1::ProviderCredential,
        ),
        mail_gmail_oauth_refresh_credential_capability_v1(),
        mail_gmail_oauth_setup_credential_capability_v1(),
        mail_provider_credential_lifecycle_capability_v1(
            MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
            "mail_gmail_refresh_credential",
            VaultSecretClassV1::OauthRefreshCredential,
        ),
        mail_provider_credential_lifecycle_capability_v1(
            MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
            "mail_imap_password",
            VaultSecretClassV1::ProviderCredential,
        ),
        mail_provider_credential_provisioning_capability_v1(
            MAIL_IMAP_CREDENTIAL_PROVISIONING_CAPABILITY_ID,
            "mail_imap_password",
            VaultSecretClassV1::ProviderCredential,
        ),
        mail_provider_credential_capability_v1(
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID,
            "mail_imap_password",
        ),
        mail_client_capability_v1(MailClientContractV1::MessageFlagCommand),
        mail_client_capability_v1(MailClientContractV1::MessageFlagQuery),
        mail_client_capability_v1(MailClientContractV1::MessageLocationCommand),
        mail_client_capability_v1(MailClientContractV1::MessageLocationQuery),
        mail_client_capability_v1(MailClientContractV1::MessagePermanentDeleteCommand),
        mail_client_capability_v1(MailClientContractV1::MessagePermanentDeleteQuery),
        mail_client_capability_v1(MailClientContractV1::GmailOAuthComplete),
        mail_client_capability_v1(MailClientContractV1::GmailOAuthQuery),
        mail_client_capability_v1(MailClientContractV1::GmailOAuthRefresh),
        mail_client_capability_v1(MailClientContractV1::GmailOAuthStart),
        mail_client_capability_v1(MailClientContractV1::OperationalQuery),
        mail_operational_realtime_capability_v1(),
        mail_person_source_provider_capability_v1(),
        mail_retained_evidence_replay_capability_v1(),
        mail_provider_credential_lifecycle_capability_v1(
            MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
            "mail_smtp_password",
            VaultSecretClassV1::ProviderCredential,
        ),
        mail_provider_credential_provisioning_capability_v1(
            MAIL_SMTP_CREDENTIAL_PROVISIONING_CAPABILITY_ID,
            "mail_smtp_password",
            VaultSecretClassV1::ProviderCredential,
        ),
        mail_provider_credential_capability_v1(
            MAIL_SMTP_CREDENTIALS_CAPABILITY_ID,
            "mail_smtp_password",
        ),
        mail_storage_capability_v1(),
        mail_client_capability_v1(MailClientContractV1::SyncHealthQuery),
        mail_client_capability_v1(MailClientContractV1::Sync),
        mail_settings_configuration_catalog_capability_v1(),
    ]
}

fn mail_person_source_provider_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_PERSON_SOURCE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![
            MailPersonSourceContractV1::AccountReady.publish_request(),
            MailPersonSourceContractV1::AccountRetired.publish_request(),
            MailPersonSourceContractV1::FetchPageCommand.consume_request(),
            MailPersonSourceContractV1::SourceObserved.publish_request(),
            MailPersonSourceContractV1::SourceUpdated.publish_request(),
            MailPersonSourceContractV1::SourceRemoved.publish_request(),
            MailPersonSourceContractV1::PageCompleted.publish_request(),
            MailPersonSourceContractV1::PageRejected.publish_request(),
            provider_credential_request_v1("mail_icloud_carddav_password"),
        ],
        ..Default::default()
    }
}

fn mail_operational_realtime_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_OPERATIONAL_REALTIME_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
            contract: Some(ContractReferenceV1 {
                owner: MAIL_OWNER_ID.to_owned(),
                name: MAIL_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1.to_owned(),
                major: MAIL_CLIENT_CONTRACT_MAJOR,
                revision: MAIL_CLIENT_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(MAIL_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
            }),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_retained_evidence_replay_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            mail_replay_command_consume_request_v1(),
            mail_replay_result_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn mail_delivery_intent_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![
            mail_delivery_intent_execute_consume_request_v1(),
            mail_delivery_intent_succeeded_publish_request_v1(),
            mail_delivery_intent_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn mail_settings_configuration_catalog_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        ..Default::default()
    }
}

#[must_use]
pub fn mail_attachment_scan_candidate_publish_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![attachment_security_scan_candidate_observed_publish_request_v1()],
        ..Default::default()
    }
}

fn mail_client_capability_v1(contract: MailClientContractV1) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: contract.capability_id().to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(mail_client_contract_reference_v1(contract)),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: contract.connect_path().to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn mail_client_contract_reference_v1(
    contract: MailClientContractV1,
) -> makosh_runtime_protocol::v1::ContractReferenceV1 {
    makosh_runtime_protocol::v1::ContractReferenceV1 {
        owner: MAIL_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: MAIL_CLIENT_CONTRACT_MAJOR,
        revision: MAIL_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(MAIL_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    }
}

#[must_use]
pub fn mail_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 2,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: MAIL_BLOB_CUSTODY_QUOTA_BYTES,
                custody_scope_id: MAIL_ATTACHMENT_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
fn mail_provider_credential_capability_v1(
    capability_id: &str,
    purpose_id: &str,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![provider_credential_request_v1(purpose_id)],
        ..Default::default()
    }
}

#[must_use]
fn mail_provider_credential_lifecycle_capability_v1(
    capability_id: &str,
    purpose_id: &str,
    secret_class: VaultSecretClassV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![vault_purpose_request_v1(
            purpose_id,
            &[secret_class],
            &[VaultActionV1::Retire, VaultActionV1::Delete],
        )],
        ..Default::default()
    }
}

#[must_use]
fn mail_provider_credential_provisioning_capability_v1(
    capability_id: &str,
    purpose_id: &str,
    secret_class: VaultSecretClassV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![vault_purpose_request_v1(
            purpose_id,
            &[secret_class],
            &[VaultActionV1::Create, VaultActionV1::ReplaceCas],
        )],
        ..Default::default()
    }
}

fn provider_credential_request_v1(purpose_id: &str) -> CapabilityRequestV1 {
    vault_purpose_request_v1(
        purpose_id,
        &[VaultSecretClassV1::ProviderCredential],
        &[VaultActionV1::Resolve],
    )
}

#[must_use]
fn mail_gmail_oauth_setup_credential_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_GMAIL_OAUTH_SETUP_CREDENTIALS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![
            vault_purpose_request_v1(
                "mail_gmail_oauth_client_secret",
                &[VaultSecretClassV1::ProviderCredential],
                &[VaultActionV1::Resolve],
            ),
            vault_purpose_request_v1(
                "mail_gmail_access_token",
                &[VaultSecretClassV1::ProviderCredential],
                &[VaultActionV1::Create, VaultActionV1::ReplaceCas],
            ),
            vault_purpose_request_v1(
                "mail_gmail_refresh_credential",
                &[VaultSecretClassV1::OauthRefreshCredential],
                &[VaultActionV1::Create, VaultActionV1::ReplaceCas],
            ),
        ],
        ..Default::default()
    }
}

#[must_use]
fn mail_gmail_oauth_refresh_credential_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_GMAIL_OAUTH_REFRESH_CREDENTIALS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![
            vault_purpose_request_v1(
                "mail_gmail_oauth_client_secret",
                &[VaultSecretClassV1::ProviderCredential],
                &[VaultActionV1::Resolve],
            ),
            vault_purpose_request_v1(
                "mail_gmail_access_token",
                &[VaultSecretClassV1::ProviderCredential],
                &[VaultActionV1::ReplaceCas],
            ),
            vault_purpose_request_v1(
                "mail_gmail_refresh_credential",
                &[VaultSecretClassV1::OauthRefreshCredential],
                &[VaultActionV1::Resolve, VaultActionV1::ReplaceCas],
            ),
        ],
        ..Default::default()
    }
}

fn vault_purpose_request_v1(
    purpose_id: &str,
    secret_classes: &[VaultSecretClassV1],
    actions: &[VaultActionV1],
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
            purpose_id: purpose_id.to_owned(),
            requested_lease_ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
            allowed_secret_classes: secret_classes
                .iter()
                .map(|secret_class| *secret_class as i32)
                .collect(),
            actions: actions.iter().map(|action| *action as i32).collect(),
            target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
            key_schema_revision: 0,
        })),
    }
}

#[must_use]
pub fn mail_communication_observed_publish_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
        capability_revision: 2,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![communication_observed_publish_request_v1()],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_attachment_blob_admission_publish_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![communication_attachment_blob_admission_observed_publish_request_v1()],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_attachment_anchor_consume_capability_v1() -> CapabilityDescriptorV1 {
    let anchor_recorded = communication_attachment_anchor_recorded_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: DurableEnvelopeKindV1::Event as i32,
                contract: Some(anchor_recorded),
                direction: EventRouteDirectionV1::Consume as i32,
                max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                max_deliver: MAIL_EVENT_MAX_DELIVER,
                ack_wait_millis: MAIL_EVENT_ACK_WAIT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_attachment_safety_state_consume_capability_v1() -> CapabilityDescriptorV1 {
    let safety_state_changed =
        communication_attachment_safety_state_changed_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: DurableEnvelopeKindV1::Event as i32,
                contract: Some(safety_state_changed),
                direction: EventRouteDirectionV1::Consume as i32,
                max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                max_deliver: MAIL_EVENT_MAX_DELIVER,
                ack_wait_millis: MAIL_EVENT_ACK_WAIT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: MAIL_OWNER_ID.to_owned(),
                connection_budget: MAIL_STORAGE_CONNECTION_BUDGET,
                timeout_millis: MAIL_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn mail_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = mail_settings_schema_bytes_v2();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 7,
        module_id: MAIL_MODULE_ID.to_owned(),
        owner_id: MAIL_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: mail_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: MAIL_SETTINGS_SCHEMA_MAJOR_V2,
            revision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: MAIL_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 256 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Mail".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::validate_descriptor_v1;

    use super::*;

    #[test]
    fn mail_descriptor_is_valid_and_requests_only_its_exact_boundary() {
        let descriptor = mail_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                MailClientContractV1::AccountCatalog.capability_id(),
                MailClientContractV1::AccountCredentialBind.capability_id(),
                MailClientContractV1::AccountDelete.capability_id(),
                MailClientContractV1::AccountLifecycleQuery.capability_id(),
                MailClientContractV1::AccountLifecycleRetry.capability_id(),
                MailClientContractV1::AccountQuery.capability_id(),
                MailClientContractV1::AccountRetire.capability_id(),
                MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID,
                MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID,
                MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID,
                MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID,
                MAIL_BLOB_CAPABILITY_ID,
                MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID,
                MailClientContractV1::CompositionCommand.capability_id(),
                MailClientContractV1::CompositionQuery.capability_id(),
                MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
                MailClientContractV1::DeliveryQuery.capability_id(),
                MailClientContractV1::Delivery.capability_id(),
                MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID,
                MAIL_GMAIL_OAUTH_CLIENT_SECRET_PROVISIONING_CAPABILITY_ID,
                MAIL_GMAIL_OAUTH_REFRESH_CREDENTIALS_CAPABILITY_ID,
                MAIL_GMAIL_OAUTH_SETUP_CREDENTIALS_CAPABILITY_ID,
                MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                MAIL_IMAP_CREDENTIAL_PROVISIONING_CAPABILITY_ID,
                MAIL_IMAP_CREDENTIALS_CAPABILITY_ID,
                MailClientContractV1::MessageFlagCommand.capability_id(),
                MailClientContractV1::MessageFlagQuery.capability_id(),
                MailClientContractV1::MessageLocationCommand.capability_id(),
                MailClientContractV1::MessageLocationQuery.capability_id(),
                MailClientContractV1::MessagePermanentDeleteCommand.capability_id(),
                MailClientContractV1::MessagePermanentDeleteQuery.capability_id(),
                MailClientContractV1::GmailOAuthComplete.capability_id(),
                MailClientContractV1::GmailOAuthQuery.capability_id(),
                MailClientContractV1::GmailOAuthRefresh.capability_id(),
                MailClientContractV1::GmailOAuthStart.capability_id(),
                MailClientContractV1::OperationalQuery.capability_id(),
                MAIL_OPERATIONAL_REALTIME_CAPABILITY_ID_V1,
                MAIL_PERSON_SOURCE_CAPABILITY_ID_V1,
                MAIL_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID,
                MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                MAIL_SMTP_CREDENTIAL_PROVISIONING_CAPABILITY_ID,
                MAIL_SMTP_CREDENTIALS_CAPABILITY_ID,
                MAIL_STORAGE_CAPABILITY_ID,
                MailClientContractV1::SyncHealthQuery.capability_id(),
                MailClientContractV1::Sync.capability_id(),
                SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID,
            ]
        );

        let address_book = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == MAIL_PERSON_SOURCE_CAPABILITY_ID_V1)
            .expect("Mail Person-source provider capability");
        assert_eq!(address_book.provides, []);
        assert_eq!(address_book.requests.len(), 9);
        assert_eq!(
            address_book
                .requests
                .iter()
                .filter(|request| matches!(request.request, Some(Request::EventRoute(_))))
                .count(),
            8,
        );
        assert!(address_book.requests.iter().any(|request| matches!(
            request.request.as_ref(),
            Some(Request::VaultPurpose(request))
                if request.purpose_id == "mail_icloud_carddav_password"
                        && request.actions == [VaultActionV1::Resolve as i32]
        )));

        let mail_blob = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == MAIL_BLOB_CAPABILITY_ID)
            .expect("Mail Blob capability");
        assert_eq!(mail_blob.capability_revision, 2);
        assert!(matches!(
            mail_blob.requests[0].request.as_ref(),
            Some(Request::BlobQuota(quota))
                if quota.max_bytes == MAIL_BLOB_CUSTODY_QUOTA_BYTES
                    && quota.custody_scope_id == MAIL_ATTACHMENT_BLOB_CUSTODY_SCOPE_ID
                    && quota.allowed_operations == [
                        BlobQuotaOperationV1::Write as i32,
                        BlobQuotaOperationV1::ReadRange as i32,
                    ]
        ));

        let candidate = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID
            })
            .expect("Mail Attachment Security candidate capability");
        assert_eq!(
            candidate.criticality,
            CapabilityCriticalityV1::Optional as i32
        );
        assert_eq!(candidate.provides, []);
        assert_eq!(candidate.requests.len(), 1);
        assert!(matches!(
            candidate.requests[0].request.as_ref(),
            Some(Request::EventRoute(route))
                if route.direction == EventRouteDirectionV1::Publish as i32
                    && route.envelope_kind == DurableEnvelopeKindV1::Observation as i32
        ));

        for capability_id in [
            MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID,
            MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID,
            MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID,
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID,
        ] {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == capability_id)
                .expect("split Mail event capability");
            assert_eq!(capability.provides, []);
            assert_eq!(capability.requests.len(), 1);
            assert!(matches!(
                capability.requests[0].request,
                Some(Request::EventRoute(_))
            ));
        }

        for contract in MailClientContractV1::ALL {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == contract.capability_id())
                .expect("Mail client capability");
            assert_eq!(capability.provides.len(), 1);
            assert_eq!(
                capability.provides[0]
                    .client_rpc_route
                    .as_ref()
                    .expect("Mail client route")
                    .path,
                contract.connect_path()
            );
            assert_eq!(
                capability.criticality,
                CapabilityCriticalityV1::Optional as i32
            );
        }

        for (capability_id, purpose_id) in [
            (
                MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID,
                "mail_gmail_access_token",
            ),
            (MAIL_IMAP_CREDENTIALS_CAPABILITY_ID, "mail_imap_password"),
            (MAIL_SMTP_CREDENTIALS_CAPABILITY_ID, "mail_smtp_password"),
        ] {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == capability_id)
                .expect("Mail credential capability");
            assert_eq!(
                capability.criticality,
                CapabilityCriticalityV1::Optional as i32
            );
            assert_eq!(capability.requests.len(), 1);
            assert!(matches!(
                capability.requests[0].request.as_ref(),
                Some(Request::VaultPurpose(request)) if request.purpose_id == purpose_id
            ));
        }

        for (capability_id, purpose_id) in [
            (
                MAIL_IMAP_CREDENTIAL_PROVISIONING_CAPABILITY_ID,
                "mail_imap_password",
            ),
            (
                MAIL_SMTP_CREDENTIAL_PROVISIONING_CAPABILITY_ID,
                "mail_smtp_password",
            ),
        ] {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == capability_id)
                .expect("Mail credential provisioning capability");
            assert!(matches!(
                capability.requests[0].request.as_ref(),
                Some(Request::VaultPurpose(request))
                    if request.purpose_id == purpose_id
                        && request.actions
                            == [
                                VaultActionV1::Create as i32,
                                VaultActionV1::ReplaceCas as i32,
                            ]
            ));
        }

        for (capability_id, purpose_id) in [
            (
                MAIL_GMAIL_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                "mail_gmail_access_token",
            ),
            (
                MAIL_GMAIL_REFRESH_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                "mail_gmail_refresh_credential",
            ),
            (
                MAIL_IMAP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                "mail_imap_password",
            ),
            (
                MAIL_SMTP_CREDENTIAL_LIFECYCLE_CAPABILITY_ID,
                "mail_smtp_password",
            ),
        ] {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == capability_id)
                .expect("Mail credential lifecycle capability");
            assert!(matches!(
                capability.requests[0].request.as_ref(),
                Some(Request::VaultPurpose(request))
                    if request.purpose_id == purpose_id
                        && request.actions
                            == [VaultActionV1::Retire as i32, VaultActionV1::Delete as i32]
            ));
        }

        let setup = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == MAIL_GMAIL_OAUTH_SETUP_CREDENTIALS_CAPABILITY_ID
            })
            .expect("Gmail OAuth setup credential capability");
        assert_eq!(setup.requests.len(), 3);
        assert!(setup.requests.iter().any(|request| matches!(
            request.request.as_ref(),
            Some(Request::VaultPurpose(request))
                if request.purpose_id == "mail_gmail_oauth_client_secret"
                    && request.actions == [VaultActionV1::Resolve as i32]
        )));
        assert!(
            setup
                .requests
                .iter()
                .filter(|request| matches!(
                    request.request.as_ref(),
                    Some(Request::VaultPurpose(request))
                        if request.actions
                            == [
                                VaultActionV1::Create as i32,
                                VaultActionV1::ReplaceCas as i32,
                            ]
                ))
                .count()
                == 2
        );

        let refresh = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == MAIL_GMAIL_OAUTH_REFRESH_CREDENTIALS_CAPABILITY_ID
            })
            .expect("Gmail OAuth refresh credential capability");
        assert_eq!(refresh.requests.len(), 3);
        assert!(refresh.requests.iter().any(|request| matches!(
            request.request.as_ref(),
            Some(Request::VaultPurpose(request))
                if request.purpose_id == "mail_gmail_oauth_client_secret"
                    && request.actions == [VaultActionV1::Resolve as i32]
        )));
        assert!(refresh.requests.iter().any(|request| matches!(
            request.request.as_ref(),
            Some(Request::VaultPurpose(request))
                if request.purpose_id == "mail_gmail_refresh_credential"
                    && request.actions
                        == [
                            VaultActionV1::Resolve as i32,
                            VaultActionV1::ReplaceCas as i32,
                        ]
        )));
    }
}
