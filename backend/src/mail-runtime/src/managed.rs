//! Kernel-admitted Mail runtime bootstrap. No CLI, provider, or domain fallback exists here.

use std::collections::BTreeMap;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_attachment_security_contract::{
    AttachmentSecurityObservationContextV1, AttachmentSecurityScanCandidateFactV1,
    admission::{
        ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
        ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID,
        ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID,
    },
    build_attachment_security_scan_candidate_outbox_record_v1,
};
use hermes_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use hermes_communications_attachment_contract::{
    AttachmentBlobAdmissionFactV1, AttachmentBlobAdmissionTransitionV1,
    AttachmentBlobExpectedStateV1, AttachmentObservationEnvelopeContextV1,
    build_attachment_blob_admission_outbox_record_v1,
};
use hermes_communications_ingress::{
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID, COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID,
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID, ObservationEnvelopeContextV1,
    account_source_cursor_v1, build_observation_outbox_record_v1, conversation_source_cursor_v1,
    scoped_record_source_cursor_v1,
};
use hermes_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use hermes_managed_vault_client::{
    ManagedProviderCredentialClientV2, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1, ManagedProviderCredentialRequestV1,
};
use hermes_runtime_protocol::v1::{
    BlobDataOperationV1, ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use hermes_runtime_protocol::validation::module_client::{
    validate_module_client_request_v1, validate_module_client_response_v1,
};
use hermes_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
        RejectManagedControlRequestsV2,
    },
    validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
};
use hermes_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use hermes_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use hermes_vault_protocol::SecretClassV1;
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::MailRuntimeAdmission;
use crate::account_lifecycle::{
    MailAccountLifecycleCoordinatorV1, MailAccountLifecycleRuntimeErrorV1,
};
use crate::address_book_consumer::{
    MailAddressBookConsumeErrorV1, consume_next_mail_address_book_fetch_v1,
    consume_next_mail_address_book_upsert_v1,
};
use crate::address_book_outbox::{
    MailAddressBookOutboxRelayErrorV1, relay_mail_address_book_outbox_once_v1,
};
use crate::admission::{
    MAIL_BLOB_CAPABILITY_ID, MAIL_CREDENTIAL_LEASE_TTL_SECONDS, MAIL_MODULE_ID,
};
use crate::attachment_anchor_mapping::{
    MailAttachmentAnchorMappingErrorV1, consume_next_attachment_anchor_recorded_v1,
};
use crate::attachment_safety_projection::{
    MailAttachmentSafetyProjectionErrorV1, consume_next_attachment_safety_state_changed_v1,
};
use crate::attachment_security_outbox::{
    MailAttachmentSecurityOutboxRelayError, relay_attachment_security_outbox_once,
};
use crate::communications_outbox::{
    MailCommunicationsOutboxRelayError, relay_communications_outbox_once,
};
use crate::delivery_intent_consumer::{
    MailDeliveryIntentConsumeErrorV1, MailDeliveryIntentResultContextV1,
    consume_next_mail_delivery_intent_v1,
};
use crate::delivery_intent_outbox::{
    MailDeliveryIntentOutboxRelayErrorV1, relay_mail_delivery_intent_outbox_once_v1,
};
use hermes_communications_ingress::{
    AttachmentDispositionV1, BodyAdmissionFailureV1, BodyAvailabilityV1, BodyBlobReceiptV1,
    CommunicationObservationDraft, ProviderProvenanceV1, with_admitted_body_blob,
    with_body_admission_failure,
};
use hermes_mail_api::{
    MailCredentialPurpose, MailDeliveryOperationStatusV1, MailDeliveryOutcomeV1,
    MailGmailConfigurationV1, MailInboundTransportV1, MailSendMailRequestV1, OutgoingMailV1,
    account::{
        MailAccountCatalogV1, MailAccountReadinessV1, MailAccountStatusV1,
        MailBindCredentialRequestV1, MailConnectorProfileV1, MailCredentialBindingReceiptV1,
        MailCredentialBindingStateV1, MailCredentialBindingStatusV1, MailCredentialPurposeV1,
        MailProviderPathReadinessV1,
    },
    account_lifecycle::{
        MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleReceiptV1,
        MailAccountLifecycleRetryV1, MailAccountLifecycleStateV1,
        MailAccountLifecycleStatusRequestV1, MailCredentialLifecycleProgressV1,
        MailCredentialLifecycleStateV1,
    },
    composition::{
        MailCompositionCommandV1, MailCompositionMutationReceiptV1, MailCompositionQueryResponseV1,
        MailCompositionQueryV1, composition_command_connection_id, composition_query_connection_id,
    },
    composition_wire::encode_composition_command,
    message_flags::{
        MailMessageFlagAcceptedV1, MailMessageFlagCommandV1, MailMessageFlagKindV1,
        MailMessageFlagOperationOutcomeV1, MailMessageFlagOperationStatusV1,
        MailMessageFlagStatusRequestV1,
    },
    message_flags_wire::{decode_message_flag_command, encode_message_flag_command},
    message_location::{
        MailMessageLocationAcceptedV1, MailMessageLocationCommandV1,
        MailMessageLocationOperationOutcomeV1, MailMessageLocationOperationStatusV1,
        MailMessageLocationStatusRequestV1,
    },
    message_location_wire::{decode_message_location_command, encode_message_location_command},
    message_permanent_delete::{
        MailMessagePermanentDeleteAcceptedV1, MailMessagePermanentDeleteCommandV1,
        MailMessagePermanentDeleteOperationOutcomeV1, MailMessagePermanentDeleteOperationStatusV1,
        MailMessagePermanentDeleteStatusRequestV1,
    },
    message_permanent_delete_wire::{
        decode_message_permanent_delete_command, encode_message_permanent_delete_command,
    },
    operational::{
        MailFolderKindV1, MailMessageFlagV1, MailOperationalQueryResponseV1,
        MailOperationalQueryV1, operational_query_connection_id,
    },
    sync_health::{
        MailSyncFailureCodeV1, MailSyncHealthQueryResponseV1, MailSyncHealthQueryV1,
        MailSyncOutcomeV1, MailSyncProviderPathReadinessV1, MailSyncTriggerV1,
        sync_health_query_connection_id,
    },
    valid_account_configuration,
};
use hermes_mail_core::rfc822::{
    AttachmentDispositionV1 as Rfc822AttachmentDispositionV1, Rfc822BodyContentV1,
    attachment_metadata, extract_attachment_part, operational_preview, readable_body_content,
};
use hermes_mail_core::{
    MAX_OUTBOUND_ATTACHMENT_BYTES, OutboundAttachmentDispositionV1, OutboundAttachmentV1,
    bounded_window, compose_rfc822, compose_rfc822_with_attachments,
    draft_attachment_ingress_observation, draft_delivery_observation,
    draft_ingress_observation_with_sender_subject_body, validate_sync_request,
};
use hermes_mail_gmail::{
    GmailAdapterErrorV1, GmailApiClientV1, GmailMutableMessageFlagV1, GmailRawMessageV1,
    decode_raw_rfc822,
};
use hermes_mail_imap::{
    ImapMailboxKindV1, ImapMessageFlagAccessV1, ImapMessageLocationAccessV1, ImapMessageLocatorV1,
    ImapMutableMessageFlagV1,
};
use hermes_mail_persistence::{
    MailAttachmentBlobAdmissionCompletionV1,
    MailAttachmentDispositionV1 as PersistedAttachmentDispositionV1,
    MailAttachmentMaterializationV1, MailCredentialBindingV1, MailDeliveryAttemptOutcomeV1,
    MailDeliveryEnqueueRequestV1, MailDeliveryRouteLocatorV1, MailDurablePersistence,
    MailDurablePersistenceError, MailImapMessageLocatorV1, MailMessageLocationReconciliationV1,
    MailMessagePermanentDeletePersistenceErrorV1, MailOperationalFolderSnapshotV1,
    MailOperationalMaterializationV1, MailOperationalMessageSnapshotV1, MailQueuedDeliveryV1,
    MailQueuedMessageFlagCommandV1, MailQueuedMessageLocationCommandV1,
    MailQueuedMessagePermanentDeleteCommandV1, MailSyncRunStartOutcomeV1, initial_imap_message_id,
};
use hermes_mail_retained_evidence_replay_contract::mail_replay_command_contract_reference_v1;
use hermes_mail_retained_evidence_replay_persistence::{
    MailRetainedEvidenceReplayPersistenceV1, RetainedMailReplayErrorV1,
};
use hermes_mail_smtp::SmtpAdapterErrorV1;

use crate::gmail_sync_worker::{
    CompletedGmailSyncProviderOperationV1, GmailSyncProviderCursorV1, GmailSyncProviderFailureV1,
    GmailSyncProviderOutcomeV1, GmailSyncProviderPageDeliveryV1, GmailSyncProviderPageV1,
    PreparedGmailSyncProviderOperationV1,
};
use crate::retained_evidence_replay_consumer::{
    MailReplayCommandConsumeErrorV1, MailReplayConsumerContextV1,
    consume_next_mail_replay_command_v1,
};
use crate::retained_evidence_replay_result::{
    MailReplayResultRelayErrorV1, relay_mail_replay_result_once_v1,
};

const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);
const OUTBOX_RELAY_TIMEOUT: Duration = Duration::from_secs(2);
pub const MAIL_SYNC_OPERATION_DEADLINE_SECONDS: i64 = 300;

const fn sync_operation_deadline(started_at_unix_seconds: i64) -> Option<i64> {
    started_at_unix_seconds.checked_add(MAIL_SYNC_OPERATION_DEADLINE_SECONDS)
}

#[derive(Clone)]
struct PendingMailSyncOperationV1 {
    operation_id: String,
    deadline_at_unix_seconds: i64,
}

pub struct MailAdmittedRuntime {
    pub control_channel: ManagedControlChannelV2<UnixStream>,
    pub durable: MailDurablePersistence,
    imap_password: Option<Zeroizing<Vec<u8>>>,
    smtp_password: Option<Zeroizing<Vec<u8>>>,
    carddav_password: Option<Zeroizing<Vec<u8>>>,
    account_lifecycle: MailAccountLifecycleCoordinatorV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    attachment_anchor_subscribe_permit: Option<RuntimeSubscribePermitV1>,
    attachment_safety_subscribe_permit: Option<RuntimeSubscribePermitV1>,
    delivery_intent_subscribe_permit: RuntimeSubscribePermitV1,
    address_book_fetch_subscribe_permit: RuntimeSubscribePermitV1,
    address_book_upsert_subscribe_permit: RuntimeSubscribePermitV1,
    pub(crate) address_book_persistence:
        hermes_mail_address_book_persistence::MailAddressBookPersistenceV1,
    replay_command_subscribe_permit: RuntimeSubscribePermitV1,
    replay_persistence: MailRetainedEvidenceReplayPersistenceV1,
    attachment_blob_admission_publish_permitted: bool,
    attachment_security_scan_candidate_publish_permitted: bool,
    pub(crate) account: hermes_mail_api::MailAccountConfigurationV1,
    pub(crate) address_book: hermes_mail_api::MailAddressBookConfigurationV1,
    pub(crate) configuration_instance_id: String,
    pub(crate) gmail_oauth: Option<hermes_mail_api::GmailOAuthConfigurationV1>,
    pub(crate) gmail_oauth_operation_in_flight: Option<String>,
    pending_sync_operation: Option<PendingMailSyncOperationV1>,
    pub(crate) provider_credential_context: ManagedProviderCredentialContextV1,
    pub(crate) settings_revision: u64,
    parked_accounts: BTreeMap<String, MailRuntimeAccountSlotV1>,
    pub(crate) runtime_instance_id: String,
    pub(crate) runtime_generation: u64,
    logical_owner_id: String,
    logical_human_owner_id: String,
    module_registration_id: String,
    grant_epoch: u64,
}

struct MailRuntimeAccountSlotV1 {
    imap_password: Option<Zeroizing<Vec<u8>>>,
    smtp_password: Option<Zeroizing<Vec<u8>>>,
    carddav_password: Option<Zeroizing<Vec<u8>>>,
    account_lifecycle: MailAccountLifecycleCoordinatorV1,
    account: hermes_mail_api::MailAccountConfigurationV1,
    address_book: hermes_mail_api::MailAddressBookConfigurationV1,
    configuration_instance_id: String,
    gmail_oauth: Option<hermes_mail_api::GmailOAuthConfigurationV1>,
    gmail_oauth_operation_in_flight: Option<String>,
    pending_sync_operation: Option<PendingMailSyncOperationV1>,
    provider_credential_context: ManagedProviderCredentialContextV1,
    settings_revision: u64,
}

struct MailAttachmentBlobWriteV1 {
    reference_id: [u8; 16],
    receipt_sha256: [u8; 32],
    custody_transfer_source_proof: Vec<u8>,
    reference_binding_sha256: [u8; 32],
    declared_size: u64,
}

struct ImapInboxSyncRequestV1<'a> {
    connection_id: &'a str,
    sync: &'a hermes_mail_imap::ImapSyncResult,
}

pub struct PreparedImapSyncProviderOperationV1 {
    connection_id: String,
    operation_id: String,
    host: String,
    port: u16,
    username: String,
    password: Zeroizing<Vec<u8>>,
    window: u32,
    windows: u32,
    priority_uids: Vec<u32>,
    deadline_at_unix_seconds: i64,
}

pub struct CompletedImapSyncProviderOperationV1 {
    connection_id: String,
    operation_id: String,
    result: Result<usize, MailBootstrapError>,
}

impl CompletedImapSyncProviderOperationV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

impl PreparedImapSyncProviderOperationV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    #[must_use]
    pub const fn deadline_at_unix_seconds(&self) -> i64 {
        self.deadline_at_unix_seconds
    }
}

pub struct ImapSyncProviderPageDeliveryV1 {
    connection_id: String,
    operation_id: String,
    sync: hermes_mail_imap::ImapSyncResult,
    acknowledgment: std::sync::mpsc::Sender<bool>,
}

impl ImapSyncProviderPageDeliveryV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

struct MailProviderDeliveryRequestV1<'a> {
    message: &'a OutgoingMailV1,
    attachments: &'a [OutboundAttachmentV1],
    from_address: &'a str,
    provider: ProviderProvenanceV1,
    queued: &'a MailQueuedDeliveryV1,
    completed_at_unix_seconds: i64,
}

struct MailAttachmentBlobAdmissionRequestV1<'a> {
    source_observation_id: [u8; 16],
    bytes: &'a [u8],
    filename: Option<String>,
    media_type: String,
    disposition: PersistedAttachmentDispositionV1,
    observed_at_unix_seconds: i64,
    observed_at_nanos: i32,
}

struct OwnedAttachmentBlobAdmissionV1 {
    source_observation_id: [u8; 16],
    bytes: Vec<u8>,
    filename: Option<String>,
    media_type: String,
    disposition: PersistedAttachmentDispositionV1,
}

struct GmailMessageRecordsV1 {
    materializations: Vec<MailOperationalMaterializationV1>,
    attachment_admissions: Vec<OwnedAttachmentBlobAdmissionV1>,
    observed_history_id: Option<String>,
}

struct InboundBodyObservationSourceV1 {
    source_id: String,
    sender: Option<String>,
    subject: Option<String>,
    body: Option<Rfc822BodyContentV1>,
}

#[derive(Debug)]
pub enum MailBootstrapError {
    Admission,
    Control,
    Storage,
    Credential,
    Persistence,
    Provider,
    EventHub,
    AttachmentAnchorMapping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryDispatchErrorV1 {
    InvalidStoredCommand,
    AttachmentRejected,
    Persistence,
    ProviderRejected,
    ProviderOutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageFlagDispatchErrorV1 {
    InvalidStoredCommand,
    Persistence,
    ProviderRejected,
    ProviderOutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageLocationDispatchErrorV1 {
    InvalidStoredCommand,
    Persistence,
    ProviderRejected,
    ProviderUnsupported,
    ProviderOutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessagePermanentDeleteDispatchErrorV1 {
    InvalidStoredCommand,
    Persistence,
    ProviderRejected,
    ProviderUnsupported,
    ReauthorizationRequired,
    ProviderOutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MailProviderDeliveryErrorV1 {
    Rejected,
    OutcomeUnknown,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    control_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &MailRuntimeAdmission,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<MailAdmittedRuntime, MailBootstrapError> {
    open_admitted_runtime_catalog(
        control_channel,
        descriptor_bytes,
        settings_schema_bytes,
        std::slice::from_ref(admission),
        storage_configuration,
        event_hub_endpoint,
        event_credential_revision,
    )
    .await
}

pub async fn open_admitted_runtime_catalog(
    control_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admissions: &[MailRuntimeAdmission],
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<MailAdmittedRuntime, MailBootstrapError> {
    let admission = admissions.first().ok_or(MailBootstrapError::Admission)?;
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admissions.len() > hermes_mail_api::account::MAX_MAIL_ACCOUNT_CATALOG_ENTRIES
        || admission.runtime_instance_id.trim().is_empty()
        || admission.logical_human_owner_id.trim().is_empty()
        || event_hub_endpoint.trim().is_empty()
        || event_credential_revision == 0
    {
        developer_admission_diagnostic("initial_contract");
        return Err(MailBootstrapError::Admission);
    }
    let mut configuration_instance_ids = std::collections::BTreeSet::new();
    let mut connection_ids = std::collections::BTreeSet::new();
    if admissions.iter().any(|candidate| {
        !valid_account_configuration(&candidate.account)
            || candidate.logical_owner_id != admission.logical_owner_id
            || candidate.logical_human_owner_id != admission.logical_human_owner_id
            || candidate.module_registration_id != admission.module_registration_id
            || candidate.runtime_instance_id != admission.runtime_instance_id
            || candidate.runtime_generation != admission.runtime_generation
            || candidate.grant_epoch != admission.grant_epoch
            || candidate.vault_runtime_generation != admission.vault_runtime_generation
            || !configuration_instance_ids.insert(candidate.configuration_instance_id.as_str())
            || !connection_ids.insert(candidate.account.connection_id.as_str())
    }) {
        developer_admission_diagnostic("account_catalog");
        return Err(MailBootstrapError::Admission);
    }
    control_channel
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| control_channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| MailBootstrapError::Control)?;
    let mut control_channel = ManagedControlChannelV2::new(control_channel);
    let identity = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| MailBootstrapError::Control)?;
    let registration_id = identity.registration_id;
    let runtime_generation = identity.runtime_generation;
    let grant_epoch = identity.grant_epoch;
    if registration_id != admission.module_registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        developer_admission_diagnostic("runtime_identity");
        return Err(MailBootstrapError::Admission);
    }

    let binding = storage_binding(&storage_configuration, admission)?;
    let storage_context = StorageVaultRouteContextV1::new(
        storage_configuration.vault_instance_id.clone(),
        storage_configuration.vault_runtime_generation,
        storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| MailBootstrapError::Storage)?,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let mut storage_leases = StorageVaultLeaseAdapterV1::new(
        InheritedKernelVaultRouteV2::new(control_channel),
        storage_context,
    );
    let lease_id = storage_leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|error| {
            if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_mail_storage_credential_issue_error={error:?}");
            }
            MailBootstrapError::Credential
        })?;
    let password = storage_leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|error| {
            if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_mail_storage_credential_resolve_error={error:?}");
            }
            MailBootstrapError::Credential
        })?;
    let mut control_channel = storage_leases.into_route_port().into_channel();
    let password = std::str::from_utf8(&password).map_err(|_| MailBootstrapError::Credential)?;
    let durable = MailDurablePersistence::connect_runtime(
        &binding,
        &storage_configuration.database_id,
        &storage_configuration.pgbouncer_host,
        storage_configuration.pgbouncer_port,
        password,
    )
    .await
    .map_err(|_| MailBootstrapError::Persistence)?;
    durable
        .interrupt_stale_sync_runs(admission.runtime_generation, current_unix_seconds()?)
        .await
        .map_err(|_| MailBootstrapError::Persistence)?;
    let mut account_slots = Vec::with_capacity(admissions.len());
    for admission in admissions {
        let provider_context = provider_credential_context(admission, &storage_configuration)?;
        let lifecycle_quiesced = durable
            .latest_account_lifecycle(&admission.account.connection_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?
            .is_some();
        let imap_password = match (&admission.account.inbound, lifecycle_quiesced) {
            (_, true) => None,
            (MailInboundTransportV1::Imap(_), false) => {
                activate_bound_account_credential(
                    &mut control_channel,
                    &provider_context,
                    &durable,
                    admission,
                    MailCredentialPurposeV1::ImapPassword,
                )
                .await?
            }
            (MailInboundTransportV1::Gmail(_), false) => None,
        };
        let smtp_password = if admission.account.smtp_endpoint.is_some() && !lifecycle_quiesced {
            activate_bound_account_credential(
                &mut control_channel,
                &provider_context,
                &durable,
                admission,
                MailCredentialPurposeV1::SmtpPassword,
            )
            .await?
        } else {
            None
        };
        let carddav_password = if matches!(
            admission.address_book.provider,
            hermes_mail_api::MailAddressBookProviderV1::IcloudCardDav
        ) && !lifecycle_quiesced
        {
            activate_bound_account_credential(
                &mut control_channel,
                &provider_context,
                &durable,
                admission,
                MailCredentialPurposeV1::IcloudCardDavPassword,
            )
            .await?
        } else {
            None
        };
        account_slots.push(MailRuntimeAccountSlotV1 {
            imap_password,
            smtp_password,
            carddav_password,
            account_lifecycle: MailAccountLifecycleCoordinatorV1::new(lifecycle_quiesced),
            account: admission.account.clone(),
            address_book: admission.address_book.clone(),
            configuration_instance_id: admission.configuration_instance_id.clone(),
            gmail_oauth: admission.gmail_oauth.clone(),
            gmail_oauth_operation_in_flight: None,
            pending_sync_operation: None,
            provider_credential_context: provider_context,
            settings_revision: admission.settings_revision,
        });
    }
    let active_slot = account_slots.remove(0);
    let parked_accounts = account_slots
        .into_iter()
        .map(|slot| (slot.account.connection_id.clone(), slot))
        .collect();
    let event_access = request_managed_runtime_event_access_v2(
        &mut control_channel,
        &admission.logical_owner_id,
        &admission.module_registration_id,
        &admission.runtime_instance_id,
        admission.runtime_generation,
        admission.grant_epoch,
        event_credential_revision,
    )
    .map_err(|_| mail_event_hub_error("access"))?;
    let identity = RuntimeNatsIdentity::new(
        admission.runtime_instance_id.clone(),
        admission.runtime_generation,
        admission.grant_epoch,
    )
    .map_err(|_| mail_event_hub_error("identity"))?;
    let event_publish_permit = event_access
        .publish_permit(
            &admission.module_registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| mail_event_hub_error("publish_permit"))?;
    let subscribe_permits = bind_event_subscribe_permits(
        event_access
            .subscribe_permits(
                &admission.module_registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| mail_event_hub_error("subscribe_permits"))?,
    )?;
    let attachment_blob_admission_publish_permitted =
        attachment_blob_admission_publish_permitted(&event_publish_permit)?;
    let attachment_security_scan_candidate_publish_permitted =
        attachment_security_scan_candidate_publish_permitted(&event_publish_permit)?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|error| {
        if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
            eprintln!("developer_mail_event_hub_connect_error={error:?}");
        }
        mail_event_hub_error("connect")
    })?;
    let replay_persistence = MailRetainedEvidenceReplayPersistenceV1::from_owner_local_pool(
        durable.owner_local_pool_handle(),
    );
    replay_persistence
        .verify_storage_ready()
        .await
        .map_err(|_| MailBootstrapError::Persistence)?;
    let address_book_persistence =
        hermes_mail_address_book_persistence::MailAddressBookPersistenceV1::from_owner_local_pool(
            durable.owner_local_pool_handle(),
        );
    address_book_persistence
        .verify_storage_ready()
        .await
        .map_err(|_| MailBootstrapError::Persistence)?;
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id,
            runtime_generation,
            grant_epoch,
        })
        .map_err(|_| MailBootstrapError::Control)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .and_then(|_| control_channel.inner_mut().set_nonblocking(true))
        .map_err(|_| MailBootstrapError::Control)?;
    let MailRuntimeAccountSlotV1 {
        imap_password,
        smtp_password,
        carddav_password,
        account_lifecycle,
        account,
        address_book,
        configuration_instance_id,
        gmail_oauth,
        gmail_oauth_operation_in_flight,
        pending_sync_operation,
        provider_credential_context,
        settings_revision,
    } = active_slot;
    Ok(MailAdmittedRuntime {
        control_channel,
        durable,
        imap_password,
        smtp_password,
        carddav_password,
        account_lifecycle,
        event_connection,
        event_publish_permit,
        attachment_anchor_subscribe_permit: subscribe_permits.anchor,
        attachment_safety_subscribe_permit: subscribe_permits.safety,
        delivery_intent_subscribe_permit: subscribe_permits.delivery_intent,
        address_book_fetch_subscribe_permit: subscribe_permits.address_book_fetch,
        address_book_upsert_subscribe_permit: subscribe_permits.address_book_upsert,
        address_book_persistence,
        replay_command_subscribe_permit: subscribe_permits.replay_command,
        replay_persistence,
        attachment_blob_admission_publish_permitted,
        attachment_security_scan_candidate_publish_permitted,
        account,
        address_book,
        configuration_instance_id,
        gmail_oauth,
        gmail_oauth_operation_in_flight,
        pending_sync_operation,
        provider_credential_context,
        settings_revision,
        parked_accounts,
        runtime_instance_id: admission.runtime_instance_id.clone(),
        runtime_generation: admission.runtime_generation,
        logical_owner_id: admission.logical_owner_id.clone(),
        logical_human_owner_id: admission.logical_human_owner_id.clone(),
        module_registration_id: admission.module_registration_id.clone(),
        grant_epoch: admission.grant_epoch,
    })
}

impl MailAdmittedRuntime {
    pub(crate) fn carddav_credentials(&self) -> Option<(&str, &str)> {
        let username = self.address_book.carddav_username.as_deref()?;
        let password = std::str::from_utf8(self.carddav_password.as_deref()?).ok()?;
        Some((username, password))
    }

    #[must_use]
    pub fn connection_ids(&self) -> Vec<String> {
        let mut connection_ids = self.parked_accounts.keys().cloned().collect::<Vec<_>>();
        connection_ids.push(self.account.connection_id.clone());
        connection_ids.sort();
        connection_ids
    }

    pub fn select_account(&mut self, connection_id: &str) -> Result<(), MailBootstrapError> {
        if self.account.connection_id == connection_id {
            return Ok(());
        }
        let next = self
            .parked_accounts
            .remove(connection_id)
            .ok_or(MailBootstrapError::Admission)?;
        let MailRuntimeAccountSlotV1 {
            imap_password,
            smtp_password,
            carddav_password,
            account_lifecycle,
            account,
            address_book,
            configuration_instance_id,
            gmail_oauth,
            gmail_oauth_operation_in_flight,
            pending_sync_operation,
            provider_credential_context,
            settings_revision,
        } = next;
        let previous = MailRuntimeAccountSlotV1 {
            imap_password: std::mem::replace(&mut self.imap_password, imap_password),
            smtp_password: std::mem::replace(&mut self.smtp_password, smtp_password),
            carddav_password: std::mem::replace(&mut self.carddav_password, carddav_password),
            account_lifecycle: std::mem::replace(&mut self.account_lifecycle, account_lifecycle),
            account: std::mem::replace(&mut self.account, account),
            address_book: std::mem::replace(&mut self.address_book, address_book),
            configuration_instance_id: std::mem::replace(
                &mut self.configuration_instance_id,
                configuration_instance_id,
            ),
            gmail_oauth: std::mem::replace(&mut self.gmail_oauth, gmail_oauth),
            gmail_oauth_operation_in_flight: std::mem::replace(
                &mut self.gmail_oauth_operation_in_flight,
                gmail_oauth_operation_in_flight,
            ),
            pending_sync_operation: std::mem::replace(
                &mut self.pending_sync_operation,
                pending_sync_operation,
            ),
            provider_credential_context: std::mem::replace(
                &mut self.provider_credential_context,
                provider_credential_context,
            ),
            settings_revision: std::mem::replace(&mut self.settings_revision, settings_revision),
        };
        self.parked_accounts
            .insert(previous.account.connection_id.clone(), previous);
        Ok(())
    }

    pub async fn account_catalog(&mut self) -> Result<MailAccountCatalogV1, MailBootstrapError> {
        let original_connection_id = self.account.connection_id.clone();
        let connection_ids = self.connection_ids();
        let mut accounts = Vec::with_capacity(connection_ids.len());
        let mut result = Ok(());
        for connection_id in connection_ids {
            if let Err(error) = self.select_account(&connection_id) {
                result = Err(error);
                break;
            }
            match self.account_status(&connection_id).await {
                Ok(status) => accounts.push(status),
                Err(error) => {
                    result = Err(error);
                    break;
                }
            }
        }
        let restore = self.select_account(&original_connection_id);
        result?;
        restore?;
        Ok(MailAccountCatalogV1 { accounts })
    }

    #[must_use]
    pub(crate) fn provider_io_permitted(&self) -> bool {
        self.account_lifecycle.provider_io_permitted()
    }

    pub(crate) fn with_blocking_provider_credential_request<T>(
        &mut self,
        request: impl FnOnce(
            &mut ManagedControlChannelV2<UnixStream>,
        ) -> Result<T, ManagedProviderCredentialErrorV1>,
    ) -> Result<T, ManagedProviderCredentialErrorV1> {
        execute_blocking_provider_credential_request(&mut self.control_channel, request)
    }

    pub async fn bind_account_credential(
        &mut self,
        request: &MailBindCredentialRequestV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailCredentialBindingReceiptV1, MailBootstrapError> {
        if request.connection_id != self.account.connection_id || !self.provider_io_permitted() {
            return Err(MailBootstrapError::Admission);
        }
        match request.purpose {
            MailCredentialPurposeV1::ImapPassword
                if matches!(&self.account.inbound, MailInboundTransportV1::Imap(_)) => {}
            MailCredentialPurposeV1::SmtpPassword
                if self.account.smtp_endpoint.is_some()
                    && matches!(&self.account.inbound, MailInboundTransportV1::Imap(_)) => {}
            MailCredentialPurposeV1::IcloudCardDavPassword
                if matches!(
                    self.address_book.provider,
                    hermes_mail_api::MailAddressBookProviderV1::IcloudCardDav
                ) => {}
            MailCredentialPurposeV1::ImapPassword
            | MailCredentialPurposeV1::SmtpPassword
            | MailCredentialPurposeV1::GmailAccessToken
            | MailCredentialPurposeV1::GmailRefreshCredential
            | MailCredentialPurposeV1::IcloudCardDavPassword => {
                return Err(MailBootstrapError::Admission);
            }
        }
        let receipt = self
            .durable
            .bind_account_credential(
                request,
                &self.configuration_instance_id,
                requested_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        match request.purpose {
            MailCredentialPurposeV1::ImapPassword => self.imap_password = None,
            MailCredentialPurposeV1::SmtpPassword => self.smtp_password = None,
            MailCredentialPurposeV1::IcloudCardDavPassword => self.carddav_password = None,
            MailCredentialPurposeV1::GmailAccessToken
            | MailCredentialPurposeV1::GmailRefreshCredential => {
                return Err(MailBootstrapError::Admission);
            }
        }
        Ok(receipt)
    }

    pub async fn apply_account_lifecycle(
        &mut self,
        command: &MailAccountLifecycleCommandV1,
        action: MailAccountLifecycleActionV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailAccountLifecycleReceiptV1, MailBootstrapError> {
        if command.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.imap_password = None;
        self.smtp_password = None;
        self.gmail_oauth_operation_in_flight = None;
        self.account_lifecycle
            .begin(
                &mut self.control_channel,
                &self.provider_credential_context,
                &self.durable,
                command,
                action,
                &self.configuration_instance_id,
                requested_at_unix_seconds,
            )
            .await
            .map_err(map_account_lifecycle_error)
    }

    pub async fn retry_account_lifecycle(
        &mut self,
        retry: &MailAccountLifecycleRetryV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailAccountLifecycleReceiptV1, MailBootstrapError> {
        if retry.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.imap_password = None;
        self.smtp_password = None;
        self.gmail_oauth_operation_in_flight = None;
        self.account_lifecycle
            .retry(
                &mut self.control_channel,
                &self.provider_credential_context,
                &self.durable,
                retry,
                &self.configuration_instance_id,
                requested_at_unix_seconds,
            )
            .await
            .map_err(map_account_lifecycle_error)
    }

    pub async fn account_lifecycle_status(
        &self,
        request: &MailAccountLifecycleStatusRequestV1,
    ) -> Result<MailAccountLifecycleReceiptV1, MailBootstrapError> {
        if request.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.account_lifecycle
            .status(&self.durable, request)
            .await
            .map_err(map_account_lifecycle_error)
    }

    pub async fn account_status(
        &self,
        connection_id: &str,
    ) -> Result<MailAccountStatusV1, MailBootstrapError> {
        if connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        let (mut bindings, connector_profile, mut sync_readiness, mut delivery_readiness) =
            match &self.account.inbound {
                MailInboundTransportV1::Imap(_) => {
                    let persisted = self
                        .durable
                        .account_credential_bindings(connection_id)
                        .await
                        .map_err(|_| MailBootstrapError::Persistence)?;
                    if persisted.iter().any(|binding| {
                        binding.configuration_instance_id != self.configuration_instance_id
                    }) {
                        return Err(MailBootstrapError::Admission);
                    }
                    let imap_binding = basic_binding_status(
                        persisted.iter().find(|binding| {
                            binding.purpose == MailCredentialPurposeV1::ImapPassword
                        }),
                        MailCredentialPurposeV1::ImapPassword,
                    );
                    let sync_readiness = provider_path_readiness(
                        std::slice::from_ref(&imap_binding),
                        self.runtime_generation,
                    );
                    let mut bindings = vec![imap_binding];
                    if matches!(
                        self.address_book.provider,
                        hermes_mail_api::MailAddressBookProviderV1::IcloudCardDav
                    ) {
                        bindings.push(basic_binding_status(
                            persisted.iter().find(|binding| {
                                binding.purpose == MailCredentialPurposeV1::IcloudCardDavPassword
                            }),
                            MailCredentialPurposeV1::IcloudCardDavPassword,
                        ));
                    }
                    if self.account.smtp_endpoint.is_some() {
                        let smtp_binding = basic_binding_status(
                            persisted.iter().find(|binding| {
                                binding.purpose == MailCredentialPurposeV1::SmtpPassword
                            }),
                            MailCredentialPurposeV1::SmtpPassword,
                        );
                        let delivery_readiness = provider_path_readiness(
                            std::slice::from_ref(&smtp_binding),
                            self.runtime_generation,
                        );
                        bindings.push(smtp_binding);
                        (
                            bindings,
                            MailConnectorProfileV1::ImapSmtp,
                            sync_readiness,
                            delivery_readiness,
                        )
                    } else {
                        (
                            bindings,
                            MailConnectorProfileV1::Imap,
                            sync_readiness,
                            MailProviderPathReadinessV1::NotConfigured,
                        )
                    }
                }
                MailInboundTransportV1::Gmail(configuration) => {
                    let bindings = match self
                        .durable
                        .gmail_oauth_credential_binding(connection_id)
                        .await
                        .map_err(|_| MailBootstrapError::Persistence)?
                    {
                        Some(binding) => vec![
                            gmail_binding_status(
                                MailCredentialPurposeV1::GmailAccessToken,
                                binding.access_token_revision,
                                self.runtime_generation,
                            ),
                            gmail_binding_status(
                                MailCredentialPurposeV1::GmailRefreshCredential,
                                binding.refresh_credential_revision,
                                self.runtime_generation,
                            ),
                        ],
                        None => vec![
                            unconfigured_binding_status(MailCredentialPurposeV1::GmailAccessToken),
                            unconfigured_binding_status(
                                MailCredentialPurposeV1::GmailRefreshCredential,
                            ),
                        ],
                    };
                    let readiness = provider_path_readiness(&bindings, self.runtime_generation);
                    let delivery_readiness = if configuration.from_address.is_some() {
                        readiness
                    } else {
                        MailProviderPathReadinessV1::NotConfigured
                    };
                    (
                        bindings,
                        MailConnectorProfileV1::Gmail,
                        readiness,
                        delivery_readiness,
                    )
                }
            };
        let lifecycle = self
            .durable
            .latest_account_lifecycle(connection_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        let lifecycle_revision = lifecycle
            .as_ref()
            .map_or(0, |lifecycle| lifecycle.lifecycle_revision);
        let lifecycle_operation_id = lifecycle
            .as_ref()
            .map(|lifecycle| lifecycle.operation_id.clone());
        let readiness = if let Some(lifecycle) = lifecycle {
            bindings = lifecycle
                .credentials
                .iter()
                .map(|progress| lifecycle_binding_status(progress, lifecycle.action))
                .collect();
            match &self.account.inbound {
                MailInboundTransportV1::Imap(_) => {
                    sync_readiness = lifecycle_path_readiness(
                        lifecycle.credentials.iter().filter(|progress| {
                            progress.purpose == MailCredentialPurposeV1::ImapPassword
                        }),
                        lifecycle.action,
                    );
                    delivery_readiness = if self.account.smtp_endpoint.is_some() {
                        lifecycle_path_readiness(
                            lifecycle.credentials.iter().filter(|progress| {
                                progress.purpose == MailCredentialPurposeV1::SmtpPassword
                            }),
                            lifecycle.action,
                        )
                    } else {
                        MailProviderPathReadinessV1::NotConfigured
                    };
                }
                MailInboundTransportV1::Gmail(configuration) => {
                    let path_readiness = lifecycle_path_readiness(
                        lifecycle.credentials.iter().filter(|progress| {
                            matches!(
                                progress.purpose,
                                MailCredentialPurposeV1::GmailAccessToken
                                    | MailCredentialPurposeV1::GmailRefreshCredential
                            )
                        }),
                        lifecycle.action,
                    );
                    sync_readiness = path_readiness;
                    delivery_readiness = if configuration.from_address.is_some() {
                        path_readiness
                    } else {
                        MailProviderPathReadinessV1::NotConfigured
                    };
                }
            }
            lifecycle_account_readiness(lifecycle.state, lifecycle.action)
        } else {
            account_readiness(&bindings, self.runtime_generation)
        };
        bindings.sort_by_key(|binding| binding.purpose);
        Ok(MailAccountStatusV1 {
            connection_id: connection_id.to_owned(),
            configuration_instance_id: self.configuration_instance_id.clone(),
            settings_revision: self.settings_revision,
            runtime_generation: self.runtime_generation,
            readiness,
            connector_profile,
            sync_readiness,
            delivery_readiness,
            bindings,
            lifecycle_revision,
            lifecycle_operation_id,
        })
    }

    pub async fn operational_query(
        &self,
        query: &MailOperationalQueryV1,
    ) -> Result<MailOperationalQueryResponseV1, MailBootstrapError> {
        if operational_query_connection_id(query) != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.durable
            .execute_operational_query(query)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn submit_message_flag_command(
        &self,
        command: &MailMessageFlagCommandV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailMessageFlagAcceptedV1, MailBootstrapError> {
        if !self.provider_io_permitted() || command.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        match &self.account.inbound {
            MailInboundTransportV1::Imap(_) if self.imap_password.is_none() => {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Gmail(_)
                if self
                    .durable
                    .gmail_oauth_credential_binding(&self.account.connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?
                    .is_none() =>
            {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Imap(_) | MailInboundTransportV1::Gmail(_) => {}
        }
        let canonical_command_bytes =
            encode_message_flag_command(command).map_err(|_| MailBootstrapError::Admission)?;
        self.durable
            .enqueue_message_flag_command(
                command,
                &canonical_command_bytes,
                requested_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn message_flag_operation_status(
        &self,
        request: &MailMessageFlagStatusRequestV1,
    ) -> Result<Option<MailMessageFlagOperationStatusV1>, MailBootstrapError> {
        if request.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.durable
            .message_flag_operation_status(request)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn execute_next_message_flag_command(
        &mut self,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, MailMessageFlagDispatchErrorV1> {
        if !self.provider_io_permitted() {
            return Ok(false);
        }
        let Some(queued) = self
            .durable
            .next_message_flag_command(&self.account.connection_id)
            .await
            .map_err(|_| MailMessageFlagDispatchErrorV1::Persistence)?
        else {
            return Ok(false);
        };
        let command = decode_message_flag_command(&queued.exact_command_bytes)
            .map_err(|_| MailMessageFlagDispatchErrorV1::InvalidStoredCommand)?;
        if !queued_matches_command(&queued, &command) {
            return Err(MailMessageFlagDispatchErrorV1::InvalidStoredCommand);
        }
        let provider_result = match self.account.inbound.clone() {
            MailInboundTransportV1::Imap(configuration) => {
                async {
                    let locator = self
                        .durable
                        .imap_message_locator(&command.connection_id, &command.message_id)
                        .await
                        .map_err(|_| MailMessageFlagDispatchErrorV1::Persistence)?
                        .ok_or(MailMessageFlagDispatchErrorV1::ProviderRejected)?;
                    let password = self
                        .imap_password
                        .as_ref()
                        .ok_or(MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown)?;
                    let password = Zeroizing::new(password.to_vec());
                    let password = std::str::from_utf8(&password)
                        .map_err(|_| MailMessageFlagDispatchErrorV1::InvalidStoredCommand)?;
                    hermes_mail_imap::set_message_flag(
                        ImapMessageFlagAccessV1 {
                            host: &configuration.host,
                            port: configuration.port,
                            username: &configuration.username,
                            password,
                        },
                        ImapMessageLocatorV1 {
                            mailbox_id: &locator.mailbox_id,
                            uid_validity: locator.uid_validity,
                            uid: locator.uid,
                        },
                        imap_message_flag(command.kind),
                        command.target_value,
                    )
                    .map_err(|error| {
                        if error.is_definite_rejection() {
                            MailMessageFlagDispatchErrorV1::ProviderRejected
                        } else {
                            MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown
                        }
                    })
                }
                .await
            }
            MailInboundTransportV1::Gmail(configuration) => {
                async {
                    let token =
                        self.resolve_gmail_access_token()
                            .await
                            .map_err(|error| match error {
                                MailBootstrapError::Persistence => {
                                    MailMessageFlagDispatchErrorV1::Persistence
                                }
                                MailBootstrapError::Credential => {
                                    MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown
                                }
                                _ => MailMessageFlagDispatchErrorV1::InvalidStoredCommand,
                            })?;
                    let token = std::str::from_utf8(&token)
                        .map_err(|_| MailMessageFlagDispatchErrorV1::InvalidStoredCommand)?;
                    let client = gmail_api_client(&configuration)
                        .map_err(|_| MailMessageFlagDispatchErrorV1::ProviderRejected)?;
                    client
                        .set_message_flag(
                            token,
                            &command.message_id,
                            gmail_message_flag(command.kind),
                            command.target_value,
                        )
                        .await
                        .map_err(|error| match error {
                            GmailAdapterErrorV1::InvalidRequest
                            | GmailAdapterErrorV1::ProviderStatus(400..=499) => {
                                MailMessageFlagDispatchErrorV1::ProviderRejected
                            }
                            GmailAdapterErrorV1::Transport
                            | GmailAdapterErrorV1::ProviderStatus(_)
                            | GmailAdapterErrorV1::InvalidResponse => {
                                MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown
                            }
                        })
                }
                .await
            }
        };
        if let Err(error) = provider_result {
            let outcome = match error {
                MailMessageFlagDispatchErrorV1::ProviderRejected
                | MailMessageFlagDispatchErrorV1::InvalidStoredCommand => {
                    MailMessageFlagOperationOutcomeV1::Rejected
                }
                MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown => {
                    MailMessageFlagOperationOutcomeV1::OutcomeUnknown
                }
                MailMessageFlagDispatchErrorV1::Persistence => return Err(error),
            };
            self.durable
                .complete_message_flag_failure(
                    &queued.operation_id,
                    &queued.connection_id,
                    outcome,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(|_| MailMessageFlagDispatchErrorV1::Persistence)?;
            return Err(error);
        }
        self.durable
            .complete_message_flag_success(&queued, completed_at_unix_seconds)
            .await
            .map_err(|_| MailMessageFlagDispatchErrorV1::Persistence)?;
        Ok(true)
    }

    pub async fn submit_message_location_command(
        &self,
        command: &MailMessageLocationCommandV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailMessageLocationAcceptedV1, MailBootstrapError> {
        if !self.provider_io_permitted() || command.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        match &self.account.inbound {
            MailInboundTransportV1::Imap(_) if self.imap_password.is_none() => {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Gmail(_)
                if self
                    .durable
                    .gmail_oauth_credential_binding(&self.account.connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?
                    .is_none() =>
            {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Imap(_) | MailInboundTransportV1::Gmail(_) => {}
        }
        let canonical_command_bytes =
            encode_message_location_command(command).map_err(|_| MailBootstrapError::Admission)?;
        self.durable
            .enqueue_message_location_command(
                command,
                &canonical_command_bytes,
                requested_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn message_location_operation_status(
        &self,
        request: &MailMessageLocationStatusRequestV1,
    ) -> Result<Option<MailMessageLocationOperationStatusV1>, MailBootstrapError> {
        if request.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.durable
            .message_location_operation_status(request)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn execute_next_message_location_command(
        &mut self,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, MailMessageLocationDispatchErrorV1> {
        if !self.provider_io_permitted() {
            return Ok(false);
        }
        let Some(queued) = self
            .durable
            .next_message_location_command(&self.account.connection_id)
            .await
            .map_err(|_| MailMessageLocationDispatchErrorV1::Persistence)?
        else {
            return Ok(false);
        };
        let command = decode_message_location_command(&queued.exact_command_bytes)
            .map_err(|_| MailMessageLocationDispatchErrorV1::InvalidStoredCommand)?;
        if !queued_location_matches_command(&queued, &command) {
            return Err(MailMessageLocationDispatchErrorV1::InvalidStoredCommand);
        }
        let provider_result = match self.account.inbound.clone() {
            MailInboundTransportV1::Imap(configuration) => {
                async {
                    let locator = self
                        .durable
                        .imap_message_locator(&command.connection_id, &command.message_id)
                        .await
                        .map_err(|_| MailMessageLocationDispatchErrorV1::Persistence)?
                        .ok_or(MailMessageLocationDispatchErrorV1::ProviderRejected)?;
                    let target = self
                        .durable
                        .message_location_target_folder(&command)
                        .await
                        .map_err(|_| MailMessageLocationDispatchErrorV1::Persistence)?
                        .ok_or(MailMessageLocationDispatchErrorV1::ProviderUnsupported)?;
                    let password = self
                        .imap_password
                        .as_ref()
                        .ok_or(MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown)?;
                    let password = Zeroizing::new(password.to_vec());
                    let password = std::str::from_utf8(&password)
                        .map_err(|_| MailMessageLocationDispatchErrorV1::InvalidStoredCommand)?;
                    let moved = hermes_mail_imap::move_message(
                        ImapMessageLocationAccessV1 {
                            host: &configuration.host,
                            port: configuration.port,
                            username: &configuration.username,
                            password,
                        },
                        ImapMessageLocatorV1 {
                            mailbox_id: &locator.mailbox_id,
                            uid_validity: locator.uid_validity,
                            uid: locator.uid,
                        },
                        &target.folder_id,
                    )
                    .map_err(|error| {
                        if error.is_unsupported() {
                            MailMessageLocationDispatchErrorV1::ProviderUnsupported
                        } else if error.is_definite_rejection() {
                            MailMessageLocationDispatchErrorV1::ProviderRejected
                        } else {
                            MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown
                        }
                    })?;
                    Ok(MailMessageLocationReconciliationV1 {
                        folders: vec![target],
                        imap_locator: Some(MailImapMessageLocatorV1 {
                            mailbox_id: moved.mailbox_id,
                            uid_validity: moved.uid_validity,
                            uid: moved.uid,
                        }),
                    })
                }
                .await
            }
            MailInboundTransportV1::Gmail(configuration) => {
                async {
                    let token =
                        self.resolve_gmail_access_token()
                            .await
                            .map_err(|error| match error {
                                MailBootstrapError::Persistence => {
                                    MailMessageLocationDispatchErrorV1::Persistence
                                }
                                MailBootstrapError::Credential => {
                                    MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown
                                }
                                _ => MailMessageLocationDispatchErrorV1::InvalidStoredCommand,
                            })?;
                    let token = std::str::from_utf8(&token)
                        .map_err(|_| MailMessageLocationDispatchErrorV1::InvalidStoredCommand)?;
                    let client = gmail_api_client(&configuration)
                        .map_err(|_| MailMessageLocationDispatchErrorV1::ProviderRejected)?;
                    let location = match command.kind {
                        hermes_mail_api::message_location::MailMessageLocationKindV1::Archive => {
                            client.archive_message(token, &command.message_id).await
                        }
                        hermes_mail_api::message_location::MailMessageLocationKindV1::Trash => {
                            client.trash_message(token, &command.message_id).await
                        }
                        hermes_mail_api::message_location::MailMessageLocationKindV1::Restore => {
                            client.restore_message(token, &command.message_id).await
                        }
                        hermes_mail_api::message_location::MailMessageLocationKindV1::Move => {
                            let target = self
                                .durable
                                .message_location_target_folder(&command)
                                .await
                                .map_err(|_| MailMessageLocationDispatchErrorV1::Persistence)?
                                .ok_or(MailMessageLocationDispatchErrorV1::ProviderUnsupported)?;
                            let target_is_inbox = match target.kind {
                                MailFolderKindV1::Inbox => true,
                                MailFolderKindV1::ProviderLabel => false,
                                _ => {
                                    return Err(
                                        MailMessageLocationDispatchErrorV1::ProviderUnsupported,
                                    );
                                }
                            };
                            client
                                .move_message(
                                    token,
                                    &command.message_id,
                                    &target.folder_id,
                                    target_is_inbox,
                                )
                                .await
                        }
                    }
                    .map_err(map_gmail_location_error)?;
                    Ok(MailMessageLocationReconciliationV1 {
                        folders: gmail_operational_folders(&location.label_ids),
                        imap_locator: None,
                    })
                }
                .await
            }
        };
        let reconciliation = match provider_result {
            Ok(reconciliation) => reconciliation,
            Err(error) => {
                let outcome = match error {
                    MailMessageLocationDispatchErrorV1::ProviderRejected
                    | MailMessageLocationDispatchErrorV1::InvalidStoredCommand => {
                        MailMessageLocationOperationOutcomeV1::Rejected
                    }
                    MailMessageLocationDispatchErrorV1::ProviderUnsupported => {
                        MailMessageLocationOperationOutcomeV1::Unsupported
                    }
                    MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown => {
                        MailMessageLocationOperationOutcomeV1::OutcomeUnknown
                    }
                    MailMessageLocationDispatchErrorV1::Persistence => return Err(error),
                };
                self.durable
                    .complete_message_location_failure(
                        &queued.operation_id,
                        &queued.connection_id,
                        outcome,
                        completed_at_unix_seconds,
                    )
                    .await
                    .map_err(|_| MailMessageLocationDispatchErrorV1::Persistence)?;
                return Err(error);
            }
        };
        self.durable
            .complete_message_location_success(&queued, &reconciliation, completed_at_unix_seconds)
            .await
            .map_err(|_| MailMessageLocationDispatchErrorV1::Persistence)?;
        Ok(true)
    }

    pub async fn submit_message_permanent_delete_command(
        &self,
        command: &MailMessagePermanentDeleteCommandV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailMessagePermanentDeleteAcceptedV1, MailBootstrapError> {
        if !self.provider_io_permitted() || command.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        match &self.account.inbound {
            MailInboundTransportV1::Imap(_) if self.imap_password.is_none() => {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Gmail(_)
                if self
                    .durable
                    .gmail_oauth_credential_binding(&self.account.connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?
                    .is_none() =>
            {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Imap(_) | MailInboundTransportV1::Gmail(_) => {}
        }
        let canonical_command_bytes = encode_message_permanent_delete_command(command)
            .map_err(|_| MailBootstrapError::Admission)?;
        self.durable
            .enqueue_message_permanent_delete_command(
                command,
                &canonical_command_bytes,
                requested_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn message_permanent_delete_operation_status(
        &self,
        request: &MailMessagePermanentDeleteStatusRequestV1,
    ) -> Result<Option<MailMessagePermanentDeleteOperationStatusV1>, MailBootstrapError> {
        if request.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.durable
            .message_permanent_delete_operation_status(request)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn execute_next_message_permanent_delete_command(
        &mut self,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, MailMessagePermanentDeleteDispatchErrorV1> {
        if !self.provider_io_permitted() {
            return Ok(false);
        }
        let Some(queued) = self
            .durable
            .next_message_permanent_delete_command(&self.account.connection_id)
            .await
            .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::Persistence)?
        else {
            return Ok(false);
        };
        let command = decode_message_permanent_delete_command(&queued.exact_command_bytes)
            .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand)?;
        if !queued_permanent_delete_matches_command(&queued, &command) {
            return Err(MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand);
        }
        let target = self
            .durable
            .message_permanent_delete_target(&queued)
            .await
            .map_err(|error| match error {
                MailMessagePermanentDeletePersistenceErrorV1::Database => {
                    MailMessagePermanentDeleteDispatchErrorV1::Persistence
                }
                MailMessagePermanentDeletePersistenceErrorV1::InvalidInput
                | MailMessagePermanentDeletePersistenceErrorV1::ConflictingOperation
                | MailMessagePermanentDeletePersistenceErrorV1::MissingMessage
                | MailMessagePermanentDeletePersistenceErrorV1::StaleProjection
                | MailMessagePermanentDeletePersistenceErrorV1::NotInTrash
                | MailMessagePermanentDeletePersistenceErrorV1::InvalidRow => {
                    MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected
                }
            })?;
        let provider_result = match self.account.inbound.clone() {
            MailInboundTransportV1::Imap(configuration) => {
                let locator = target
                    .imap_locator
                    .ok_or(MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected)?;
                let password = self
                    .imap_password
                    .as_ref()
                    .ok_or(MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown)?;
                let password = Zeroizing::new(password.to_vec());
                let password = std::str::from_utf8(&password)
                    .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand)?;
                hermes_mail_imap::permanently_delete_message(
                    ImapMessageLocationAccessV1 {
                        host: &configuration.host,
                        port: configuration.port,
                        username: &configuration.username,
                        password,
                    },
                    ImapMessageLocatorV1 {
                        mailbox_id: &locator.mailbox_id,
                        uid_validity: locator.uid_validity,
                        uid: locator.uid,
                    },
                )
                .map_err(|error| {
                    if error.is_unsupported() {
                        MailMessagePermanentDeleteDispatchErrorV1::ProviderUnsupported
                    } else if error.is_definite_rejection() {
                        MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected
                    } else {
                        MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown
                    }
                })
            }
            MailInboundTransportV1::Gmail(configuration) => {
                let binding = self
                    .durable
                    .gmail_oauth_credential_binding(&self.account.connection_id)
                    .await
                    .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::Persistence)?
                    .ok_or(MailMessagePermanentDeleteDispatchErrorV1::ReauthorizationRequired)?;
                if !binding.permanent_delete_authorized {
                    Err(MailMessagePermanentDeleteDispatchErrorV1::ReauthorizationRequired)
                } else {
                    let token =
                        self.resolve_gmail_access_token()
                            .await
                            .map_err(|error| {
                                match error {
                            MailBootstrapError::Persistence => {
                                MailMessagePermanentDeleteDispatchErrorV1::Persistence
                            }
                            MailBootstrapError::Credential => {
                                MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown
                            }
                            _ => {
                                MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand
                            }
                        }
                            })?;
                    let token = std::str::from_utf8(&token).map_err(|_| {
                        MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand
                    })?;
                    let client = gmail_api_client(&configuration)
                        .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected)?;
                    client
                        .permanently_delete_message(token, &target.provider_message_id)
                        .await
                        .map_err(map_gmail_permanent_delete_error)
                }
            }
        };
        if let Err(error) = provider_result {
            let outcome = match error {
                MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected
                | MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand => {
                    MailMessagePermanentDeleteOperationOutcomeV1::Rejected
                }
                MailMessagePermanentDeleteDispatchErrorV1::ProviderUnsupported => {
                    MailMessagePermanentDeleteOperationOutcomeV1::Unsupported
                }
                MailMessagePermanentDeleteDispatchErrorV1::ReauthorizationRequired => {
                    MailMessagePermanentDeleteOperationOutcomeV1::ReauthorizationRequired
                }
                MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown => {
                    MailMessagePermanentDeleteOperationOutcomeV1::OutcomeUnknown
                }
                MailMessagePermanentDeleteDispatchErrorV1::Persistence => return Err(error),
            };
            self.durable
                .complete_message_permanent_delete_failure(
                    &queued.operation_id,
                    &queued.connection_id,
                    outcome,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::Persistence)?;
            return Err(error);
        }
        self.durable
            .complete_message_permanent_delete_success(&queued, completed_at_unix_seconds)
            .await
            .map_err(|_| MailMessagePermanentDeleteDispatchErrorV1::Persistence)?;
        Ok(true)
    }

    pub async fn composition_command(
        &self,
        command: &MailCompositionCommandV1,
        requested_at_unix_seconds: i64,
    ) -> Result<MailCompositionMutationReceiptV1, MailBootstrapError> {
        if composition_command_connection_id(command) != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        let command_bytes =
            encode_composition_command(command).map_err(|_| MailBootstrapError::Admission)?;
        self.durable
            .execute_composition_command(command, &command_bytes, requested_at_unix_seconds)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn composition_query(
        &self,
        query: &MailCompositionQueryV1,
    ) -> Result<MailCompositionQueryResponseV1, MailBootstrapError> {
        if composition_query_connection_id(query) != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.durable
            .execute_composition_query(query)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn sync_health_query(
        &self,
        query: &MailSyncHealthQueryV1,
    ) -> Result<MailSyncHealthQueryResponseV1, MailBootstrapError> {
        let connection_id = sync_health_query_connection_id(query);
        if connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        let account = self.account_status(connection_id).await?;
        let readiness = if account.sync_readiness == MailProviderPathReadinessV1::Ready {
            MailSyncProviderPathReadinessV1::Ready
        } else {
            MailSyncProviderPathReadinessV1::Unavailable
        };
        self.durable
            .execute_sync_health_query(query, readiness)
            .await
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn try_consume_attachment_anchor_handoff(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailBootstrapError> {
        let Some(permit) = &self.attachment_anchor_subscribe_permit else {
            return Ok(false);
        };
        match consume_next_attachment_anchor_recorded_v1(
            &self.durable,
            &self.event_connection,
            permit,
            consumed_at_unix_seconds,
        )
        .await
        {
            Ok(_) => Ok(true),
            Err(MailAttachmentAnchorMappingErrorV1::Unavailable) => Ok(false),
            Err(error) => Err(map_attachment_anchor_mapping_error(error)),
        }
    }

    pub async fn try_consume_attachment_safety_state(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailBootstrapError> {
        let Some(permit) = &self.attachment_safety_subscribe_permit else {
            return Ok(false);
        };
        match consume_next_attachment_safety_state_changed_v1(
            &self.durable,
            &self.event_connection,
            permit,
            consumed_at_unix_seconds,
        )
        .await
        {
            Ok(_) => Ok(true),
            Err(MailAttachmentSafetyProjectionErrorV1::Unavailable) => Ok(false),
            Err(error) => Err(map_attachment_safety_projection_error(error)),
        }
    }

    pub async fn try_handle_client_delivery(&mut self) -> Result<bool, MailBootstrapError> {
        let Some((correlation_id, control_request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| MailBootstrapError::Control)?
        else {
            return Ok(false);
        };
        let request = match control_request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => request,
                _ => {
                    write_control_error(
                        &mut self.control_channel,
                        correlation_id,
                        "managed_runtime_control_invalid_client_delivery",
                    )?;
                    return Ok(true);
                }
            },
            _ => {
                write_control_error(
                    &mut self.control_channel,
                    correlation_id,
                    "managed_runtime_control_unexpected_request",
                )?;
                return Ok(true);
            }
        };
        let response = match crate::client_port::handle_client_request(
            self,
            &request.encode_to_vec(),
            current_unix_seconds()?,
        )
        .await
        {
            Ok(payload) => ModuleClientResponseV1::decode(payload.as_slice())
                .map_err(|_| MailBootstrapError::Provider)?,
            Err(error) => {
                if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                    eprintln!(
                        "developer_mail_client_request_error contract={} kind={error:?}",
                        request
                            .contract
                            .as_ref()
                            .map_or("missing", |contract| contract.name.as_str())
                    );
                }
                ModuleClientResponseV1 {
                    protocol_major: 1,
                    request_id: request.request_id,
                    response_payload: Vec::new(),
                    error_code: match error {
                        crate::client_port::MailClientPortErrorV1::Protocol => "INVALID_ARGUMENT",
                        crate::client_port::MailClientPortErrorV1::Runtime => "REJECTED",
                    }
                    .to_owned(),
                }
            }
        };
        validate_module_client_response_v1(&response).map_err(|_| MailBootstrapError::Provider)?;
        write_client_delivery_response(&mut self.control_channel, correlation_id, response)?;
        Ok(true)
    }

    pub async fn submit_delivery(
        &self,
        request: &MailSendMailRequestV1,
        requested_at_unix_seconds: i64,
    ) -> Result<String, MailBootstrapError> {
        if !self.provider_io_permitted() {
            return Err(MailBootstrapError::Credential);
        }
        match &self.account.inbound {
            MailInboundTransportV1::Imap(_) if self.smtp_password.is_none() => {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Gmail(_)
                if self
                    .durable
                    .gmail_oauth_credential_binding(&self.account.connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?
                    .is_none() =>
            {
                return Err(MailBootstrapError::Credential);
            }
            MailInboundTransportV1::Imap(_) | MailInboundTransportV1::Gmail(_) => {}
        }
        let message = self.outgoing_message(request);
        let from_address = match &self.account.inbound {
            MailInboundTransportV1::Imap(_) => self
                .account
                .smtp_endpoint
                .as_ref()
                .map(|endpoint| endpoint.from_address.as_str())
                .ok_or(MailBootstrapError::Admission)?,
            MailInboundTransportV1::Gmail(configuration) => configuration
                .from_address
                .as_deref()
                .ok_or(MailBootstrapError::Admission)?,
        };
        let rfc822_message =
            compose_rfc822(from_address, &message).map_err(|_| MailBootstrapError::Admission)?;
        let _ = rfc822_message;
        let exact_command_bytes = hermes_mail_api::client_wire::encode_delivery_request(request);
        let request_sha256: [u8; 32] = Sha256::digest(&exact_command_bytes).into();
        self.durable
            .enqueue_delivery_command(MailDeliveryEnqueueRequestV1 {
                operation_id: &message.operation_id,
                connection_id: &message.connection_id,
                request_sha256: &request_sha256,
                exact_command_bytes: &exact_command_bytes,
                attachment_anchor_ids: &request.attachment_anchor_ids,
                max_attachment_bytes: u64::try_from(MAX_OUTBOUND_ATTACHMENT_BYTES)
                    .map_err(|_| MailBootstrapError::Admission)?,
                requested_at_unix_seconds,
            })
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        Ok(message.operation_id)
    }

    pub async fn delivery_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<MailDeliveryOperationStatusV1>, MailBootstrapError> {
        self.durable
            .delivery_attempt(operation_id)
            .await
            .map(|status| {
                status.map(|status| {
                    let (outcome, response_code) = match status.outcome {
                        MailDeliveryAttemptOutcomeV1::Pending => {
                            (MailDeliveryOutcomeV1::Pending, None)
                        }
                        MailDeliveryAttemptOutcomeV1::Accepted { response_code } => {
                            (MailDeliveryOutcomeV1::Accepted, Some(response_code))
                        }
                        MailDeliveryAttemptOutcomeV1::Rejected => {
                            (MailDeliveryOutcomeV1::Rejected, None)
                        }
                        MailDeliveryAttemptOutcomeV1::OutcomeUnknown => {
                            (MailDeliveryOutcomeV1::OutcomeUnknown, None)
                        }
                    };
                    MailDeliveryOperationStatusV1 {
                        operation_id: status.operation_id,
                        connection_id: status.connection_id,
                        outcome,
                        requested_at_unix_seconds: status.requested_at_unix_seconds,
                        completed_at_unix_seconds: status.completed_at_unix_seconds,
                        response_code,
                    }
                })
            })
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn execute_next_delivery(
        &mut self,
        dispatched_at_unix_seconds: i64,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, MailDeliveryDispatchErrorV1> {
        if !self.provider_io_permitted() {
            return Ok(false);
        }
        let provider_ready = match &self.account.inbound {
            MailInboundTransportV1::Imap(_) => self.smtp_password.is_some(),
            MailInboundTransportV1::Gmail(_) => self
                .durable
                .gmail_oauth_credential_binding(&self.account.connection_id)
                .await
                .map_err(|_| MailDeliveryDispatchErrorV1::Persistence)?
                .is_some(),
        };
        if !provider_ready {
            return Ok(false);
        }
        let Some(queued) = self
            .durable
            .claim_next_delivery(&self.account.connection_id, dispatched_at_unix_seconds)
            .await
            .map_err(|_| MailDeliveryDispatchErrorV1::Persistence)?
        else {
            return Ok(false);
        };
        let request =
            hermes_mail_api::client_wire::decode_delivery_request(&queued.exact_command_bytes)
                .map_err(|_| MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        let message = self.outgoing_message(&request);
        let request_sha256: [u8; 32] = Sha256::digest(&queued.exact_command_bytes).into();
        if queued.operation_id != message.operation_id
            || queued.connection_id != message.connection_id
            || queued.request_sha256 != request_sha256
            || request.attachment_anchor_ids
                != queued
                    .attachments
                    .iter()
                    .map(|attachment| attachment.attachment_anchor_id)
                    .collect::<Vec<_>>()
        {
            return Err(MailDeliveryDispatchErrorV1::InvalidStoredCommand);
        }
        let attachments = match self.materialize_delivery_attachments(&queued) {
            Ok(attachments) => attachments,
            Err(_) => {
                self.durable
                    .complete_delivery_rejected(&message.operation_id, completed_at_unix_seconds)
                    .await
                    .map_err(|_| MailDeliveryDispatchErrorV1::Persistence)?;
                return Err(MailDeliveryDispatchErrorV1::AttachmentRejected);
            }
        };
        let account = self.account.clone();
        match account.inbound {
            MailInboundTransportV1::Imap(_) => {
                self.send_mail_via_smtp(
                    account
                        .smtp_endpoint
                        .as_ref()
                        .ok_or(MailDeliveryDispatchErrorV1::InvalidStoredCommand)?,
                    &message,
                    &attachments,
                    &queued,
                    completed_at_unix_seconds,
                )
                .await?;
            }
            MailInboundTransportV1::Gmail(configuration) => {
                self.send_mail_via_gmail(
                    &configuration,
                    &message,
                    &attachments,
                    &queued,
                    completed_at_unix_seconds,
                )
                .await?;
            }
        }
        Ok(true)
    }

    fn outgoing_message(&self, request: &MailSendMailRequestV1) -> OutgoingMailV1 {
        OutgoingMailV1 {
            operation_id: request.operation_id.clone(),
            connection_id: self.account.connection_id.clone(),
            provider_conversation_id: request.provider_conversation_id.clone(),
            recipients: request.recipients.clone(),
            cc_recipients: request.cc_recipients.clone(),
            bcc_recipients: request.bcc_recipients.clone(),
            subject: request.subject.clone(),
            text_body: request.text_body.clone(),
        }
    }

    fn materialize_delivery_attachments(
        &mut self,
        queued: &MailQueuedDeliveryV1,
    ) -> Result<Vec<OutboundAttachmentV1>, MailDeliveryDispatchErrorV1> {
        let mut total_bytes = 0_u64;
        let mut attachments = Vec::with_capacity(queued.attachments.len());
        for manifest in &queued.attachments {
            total_bytes = total_bytes
                .checked_add(manifest.declared_size)
                .filter(|total| {
                    *total
                        <= u64::try_from(MAX_OUTBOUND_ATTACHMENT_BYTES)
                            .expect("bounded attachment limit")
                })
                .ok_or(MailDeliveryDispatchErrorV1::AttachmentRejected)?;
            self.control_channel
                .inner_mut()
                .set_nonblocking(false)
                .map_err(|_| MailDeliveryDispatchErrorV1::AttachmentRejected)?;
            let mut dispatcher = MailBusyControlDispatcher;
            let session = request_managed_blob_session_v2(
                &mut self.control_channel,
                &mut dispatcher,
                ManagedBlobSessionRequestV1 {
                    capability_id: MAIL_BLOB_CAPABILITY_ID,
                    operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                    reference_id: &manifest.blob_reference_id,
                    declared_size: manifest.declared_size,
                    backup_class: 1,
                    receipt_sha256: Some(&manifest.receipt_sha256),
                    custody_target: None,
                },
            );
            let restored = self.control_channel.inner_mut().set_nonblocking(true);
            let session = session.map_err(|_| MailDeliveryDispatchErrorV1::AttachmentRejected)?;
            restored.map_err(|_| MailDeliveryDispatchErrorV1::AttachmentRejected)?;
            let bytes = BlobDataClient::new(session.data_socket_path)
                .and_then(|client| {
                    client.read_range(
                        session.grant,
                        session.channel_binding,
                        0,
                        manifest.declared_size,
                    )
                })
                .map_err(|_| MailDeliveryDispatchErrorV1::AttachmentRejected)?;
            if u64::try_from(bytes.len()).ok() != Some(manifest.declared_size)
                || <[u8; 32]>::from(Sha256::digest(&bytes)) != manifest.receipt_sha256
            {
                return Err(MailDeliveryDispatchErrorV1::AttachmentRejected);
            }
            attachments.push(OutboundAttachmentV1 {
                anchor_id: manifest.attachment_anchor_id,
                filename: manifest.filename.clone(),
                media_type: manifest.media_type.clone(),
                disposition: match manifest.disposition {
                    PersistedAttachmentDispositionV1::Attachment => {
                        OutboundAttachmentDispositionV1::Attachment
                    }
                    PersistedAttachmentDispositionV1::Inline => {
                        OutboundAttachmentDispositionV1::Inline
                    }
                },
                bytes,
            });
        }
        Ok(attachments)
    }

    async fn send_mail_via_smtp(
        &mut self,
        endpoint: &hermes_mail_api::SmtpEndpointV1,
        message: &OutgoingMailV1,
        attachments: &[OutboundAttachmentV1],
        queued: &MailQueuedDeliveryV1,
        completed_at_unix_seconds: i64,
    ) -> Result<u16, MailDeliveryDispatchErrorV1> {
        let password = self
            .smtp_password
            .as_deref()
            .ok_or(MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        let password = std::str::from_utf8(password)
            .map_err(|_| MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        self.send_mail(
            MailProviderDeliveryRequestV1 {
                message,
                attachments,
                from_address: &endpoint.from_address,
                provider: ProviderProvenanceV1::MailSmtp,
                queued,
                completed_at_unix_seconds,
            },
            |rfc822_message| async move {
                hermes_mail_smtp::send_implicit_tls(endpoint, message, password, &rfc822_message)
                    .await
                    .map(|receipt| receipt.response_code)
                    .map_err(|error| match error {
                        SmtpAdapterErrorV1::InvalidRequest | SmtpAdapterErrorV1::Rejected => {
                            MailProviderDeliveryErrorV1::Rejected
                        }
                        SmtpAdapterErrorV1::Unavailable | SmtpAdapterErrorV1::Protocol => {
                            MailProviderDeliveryErrorV1::OutcomeUnknown
                        }
                    })
            },
        )
        .await
    }

    async fn send_mail_via_gmail(
        &mut self,
        configuration: &MailGmailConfigurationV1,
        message: &OutgoingMailV1,
        attachments: &[OutboundAttachmentV1],
        queued: &MailQueuedDeliveryV1,
        completed_at_unix_seconds: i64,
    ) -> Result<u16, MailDeliveryDispatchErrorV1> {
        let access_token =
            self.resolve_gmail_access_token()
                .await
                .map_err(|error| match error {
                    MailBootstrapError::Persistence => MailDeliveryDispatchErrorV1::Persistence,
                    MailBootstrapError::Credential => {
                        MailDeliveryDispatchErrorV1::ProviderOutcomeUnknown
                    }
                    _ => MailDeliveryDispatchErrorV1::InvalidStoredCommand,
                })?;
        let access_token = std::str::from_utf8(&access_token)
            .map_err(|_| MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        self.send_mail(
            MailProviderDeliveryRequestV1 {
                message,
                attachments,
                from_address: configuration
                    .from_address
                    .as_deref()
                    .ok_or(MailDeliveryDispatchErrorV1::InvalidStoredCommand)?,
                provider: ProviderProvenanceV1::MailGmail,
                queued,
                completed_at_unix_seconds,
            },
            |rfc822_message| async move {
                let client = gmail_api_client(configuration)
                    .map_err(|_| MailProviderDeliveryErrorV1::Rejected)?;
                client
                    .send_raw_message(
                        access_token,
                        rfc822_message.as_bytes(),
                        Some(&message.provider_conversation_id),
                    )
                    .await
                    .map(|_| 200)
                    .map_err(|error| match error {
                        GmailAdapterErrorV1::InvalidRequest => {
                            MailProviderDeliveryErrorV1::Rejected
                        }
                        GmailAdapterErrorV1::Transport
                        | GmailAdapterErrorV1::ProviderStatus(_)
                        | GmailAdapterErrorV1::InvalidResponse => {
                            MailProviderDeliveryErrorV1::OutcomeUnknown
                        }
                    })
            },
        )
        .await
    }

    async fn send_mail<F, Fut>(
        &self,
        request: MailProviderDeliveryRequestV1<'_>,
        execute: F,
    ) -> Result<u16, MailDeliveryDispatchErrorV1>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<u16, MailProviderDeliveryErrorV1>>,
    {
        let MailProviderDeliveryRequestV1 {
            message,
            attachments,
            from_address,
            provider,
            queued,
            completed_at_unix_seconds,
        } = request;
        let rfc822_message = compose_rfc822_with_attachments(from_address, message, attachments)
            .map_err(|_| MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        let rfc822_sha256: [u8; 32] = Sha256::digest(rfc822_message.as_bytes()).into();
        if queued
            .legacy_rfc822_sha256
            .is_some_and(|expected| expected != rfc822_sha256)
            || queued
                .rendered_rfc822_sha256
                .is_some_and(|expected| expected != rfc822_sha256)
        {
            return Err(MailDeliveryDispatchErrorV1::InvalidStoredCommand);
        }
        if queued.legacy_rfc822_sha256.is_none() {
            self.durable
                .record_delivery_rendered_rfc822(
                    &message.operation_id,
                    &queued.request_sha256,
                    &rfc822_sha256,
                )
                .await
                .map_err(|_| MailDeliveryDispatchErrorV1::Persistence)?;
        }
        let response_code = match execute(rfc822_message).await {
            Ok(response_code) => response_code,
            Err(MailProviderDeliveryErrorV1::Rejected) => {
                self.durable
                    .complete_delivery_rejected(&message.operation_id, completed_at_unix_seconds)
                    .await
                    .map_err(|_| MailDeliveryDispatchErrorV1::Persistence)?;
                return Err(MailDeliveryDispatchErrorV1::ProviderRejected);
            }
            Err(MailProviderDeliveryErrorV1::OutcomeUnknown) => {
                return Err(MailDeliveryDispatchErrorV1::ProviderOutcomeUnknown);
            }
        };
        let observation = draft_delivery_observation(provider, message)
            .map_err(|_| MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        let record = build_observation_outbox_record_v1(
            &observation,
            &observation_context(
                &self.runtime_instance_id,
                self.runtime_generation,
                completed_at_unix_seconds,
                0,
            ),
        )
        .map_err(|_| MailDeliveryDispatchErrorV1::InvalidStoredCommand)?;
        self.durable
            .complete_delivery_accepted(
                &message.operation_id,
                &rfc822_sha256,
                response_code,
                &record,
                completed_at_unix_seconds,
            )
            .await
            .map_err(|_| MailDeliveryDispatchErrorV1::Persistence)?;
        Ok(response_code)
    }

    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, MailCommunicationsOutboxRelayError> {
        tokio::time::timeout(
            OUTBOX_RELAY_TIMEOUT,
            relay_communications_outbox_once(
                &self.durable,
                &self.event_connection,
                &self.event_publish_permit,
                published_at_unix_seconds,
            ),
        )
        .await
        .map_err(|_| MailCommunicationsOutboxRelayError::Unavailable)?
    }

    pub async fn try_consume_delivery_intent(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailDeliveryIntentConsumeErrorV1> {
        let outcome = consume_next_mail_delivery_intent_v1(
            &self.durable.delivery_intent_store(),
            &self.event_connection,
            &self.delivery_intent_subscribe_permit,
            &self.logical_owner_id,
            &MailDeliveryIntentResultContextV1 {
                runtime_instance_id: self.runtime_instance_id.clone(),
                runtime_generation: self.runtime_generation,
                completed_at_unix_seconds: consumed_at_unix_seconds,
                completed_at_nanos: 0,
            },
        )
        .await?;
        Ok(matches!(
            outcome,
            hermes_mail_persistence::MailDeliveryIntentInboxOutcomeV1::Pending
                | hermes_mail_persistence::MailDeliveryIntentInboxOutcomeV1::RouteNotFound
        ))
    }

    pub async fn relay_delivery_intent_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, MailDeliveryIntentOutboxRelayErrorV1> {
        tokio::time::timeout(
            OUTBOX_RELAY_TIMEOUT,
            relay_mail_delivery_intent_outbox_once_v1(
                &self.durable.delivery_intent_store(),
                &self.event_connection,
                &self.event_publish_permit,
                published_at_unix_seconds,
            ),
        )
        .await
        .map_err(|_| MailDeliveryIntentOutboxRelayErrorV1::Unavailable)?
    }

    pub async fn try_consume_address_book_upsert(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailAddressBookConsumeErrorV1> {
        consume_next_mail_address_book_upsert_v1(
            &self.address_book_persistence,
            &self.event_connection,
            &self.address_book_upsert_subscribe_permit,
            &self.logical_human_owner_id,
            consumed_at_unix_seconds,
        )
        .await
        .map(|outcome| {
            matches!(
                outcome,
                hermes_mail_address_book_persistence::MailAddressBookCommandInboxOutcomeV1::Accepted
            )
        })
    }

    pub async fn try_consume_address_book_fetch(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailAddressBookConsumeErrorV1> {
        consume_next_mail_address_book_fetch_v1(
            &self.address_book_persistence,
            &self.event_connection,
            &self.address_book_fetch_subscribe_permit,
            &self.logical_human_owner_id,
            consumed_at_unix_seconds,
        )
        .await
        .map(|outcome| {
            matches!(
                outcome,
                hermes_mail_address_book_persistence::MailAddressBookFetchInboxOutcomeV1::Accepted
            )
        })
    }

    pub async fn relay_address_book_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, MailAddressBookOutboxRelayErrorV1> {
        tokio::time::timeout(
            OUTBOX_RELAY_TIMEOUT,
            relay_mail_address_book_outbox_once_v1(
                &self.address_book_persistence,
                &self.event_connection,
                &self.event_publish_permit,
                published_at_unix_seconds,
            ),
        )
        .await
        .map_err(|_| MailAddressBookOutboxRelayErrorV1::Unavailable)?
    }

    pub async fn try_consume_replay_command(
        &self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailReplayCommandConsumeErrorV1> {
        // Event Hub owns the bounded pull deadline. Cancelling its future from
        // this integration before the server-side pull expires can strand a
        // delivered command as unacknowledged until redelivery.
        consume_next_mail_replay_command_v1(
            &self.replay_persistence,
            &self.event_connection,
            &self.replay_command_subscribe_permit,
            &self.event_publish_permit,
            &MailReplayConsumerContextV1 {
                logical_owner_id: self.logical_human_owner_id.clone(),
                producer_registration_id: self.module_registration_id.clone(),
                runtime_instance_id: self.runtime_instance_id.clone(),
                runtime_generation: self.runtime_generation,
                grant_epoch: self.grant_epoch,
                execution_attempt: 1,
                completed_at_unix_seconds: consumed_at_unix_seconds,
                completed_at_nanos: 0,
            },
        )
        .await
        .map(|outcome| outcome.is_some())
    }

    pub async fn relay_replay_result(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<bool, MailReplayResultRelayErrorV1> {
        relay_mail_replay_result_once_v1(
            &self.replay_persistence,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }

    pub async fn index_retained_attachment_scan_candidates(
        &self,
        indexed_at_unix_seconds: i64,
    ) -> Result<usize, RetainedMailReplayErrorV1> {
        self.replay_persistence
            .index_existing_scan_candidates(256, indexed_at_unix_seconds)
            .await
    }

    pub async fn relay_attachment_security_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, MailAttachmentSecurityOutboxRelayError> {
        if !self.attachment_security_scan_candidate_publish_permitted {
            return Ok(0);
        }
        tokio::time::timeout(
            OUTBOX_RELAY_TIMEOUT,
            relay_attachment_security_outbox_once(
                &self.durable,
                &self.event_connection,
                &self.event_publish_permit,
                published_at_unix_seconds,
            ),
        )
        .await
        .map_err(|_| MailAttachmentSecurityOutboxRelayError::Unavailable)?
    }

    pub async fn accept_sync_operation(
        &mut self,
        operation_id: &str,
        started_at_unix_seconds: i64,
    ) -> Result<(), MailBootstrapError> {
        if !self.provider_io_permitted() {
            return Err(MailBootstrapError::Credential);
        }
        let begin = self
            .durable
            .begin_sync_run(
                operation_id,
                &self.account.connection_id,
                MailSyncTriggerV1::Manual,
                self.runtime_generation,
                started_at_unix_seconds,
            )
            .await
            .map_err(map_sync_persistence_error)?;
        if matches!(begin, MailSyncRunStartOutcomeV1::Started(_)) {
            let deadline_at_unix_seconds = sync_operation_deadline(started_at_unix_seconds)
                .ok_or(MailBootstrapError::Admission)?;
            self.pending_sync_operation = Some(PendingMailSyncOperationV1 {
                operation_id: operation_id.to_owned(),
                deadline_at_unix_seconds,
            });
        }
        Ok(())
    }

    pub async fn prepare_pending_imap_sync(
        &mut self,
    ) -> Result<Option<PreparedImapSyncProviderOperationV1>, MailBootstrapError> {
        let MailInboundTransportV1::Imap(configuration) = &self.account.inbound else {
            return Ok(None);
        };
        let Some(pending) = self.pending_sync_operation.take() else {
            return Ok(None);
        };
        validate_sync_request(&configuration.host, configuration.port, 0)
            .map_err(|_| MailBootstrapError::Admission)?;
        bounded_window(self.account.sync_window, self.account.sync_windows)
            .map_err(|_| MailBootstrapError::Admission)?;
        let password = self
            .imap_password
            .as_ref()
            .ok_or(MailBootstrapError::Credential)?
            .clone();
        let priority_uids = self
            .durable
            .recent_inbox_imap_uids(&self.account.connection_id, 1_000)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        Ok(Some(PreparedImapSyncProviderOperationV1 {
            connection_id: self.account.connection_id.clone(),
            operation_id: pending.operation_id,
            host: configuration.host.clone(),
            port: configuration.port,
            username: configuration.username.clone(),
            password,
            window: self.account.sync_window,
            windows: self.account.sync_windows,
            priority_uids,
            deadline_at_unix_seconds: pending.deadline_at_unix_seconds,
        }))
    }

    pub async fn prepare_pending_gmail_sync(
        &mut self,
    ) -> Result<Option<PreparedGmailSyncProviderOperationV1>, MailBootstrapError> {
        let MailInboundTransportV1::Gmail(configuration) = self.account.inbound.clone() else {
            return Ok(None);
        };
        let Some(pending) = self.pending_sync_operation.take() else {
            return Ok(None);
        };
        let plan = bounded_window(self.account.sync_window, self.account.sync_windows)
            .map_err(|_| MailBootstrapError::Admission)?;
        let access_token = self.resolve_gmail_access_token().await?;
        let client = gmail_api_client(&configuration).map_err(|_| MailBootstrapError::Admission)?;
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_unix_seconds =
            i64::try_from(observed_at.as_secs()).map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_nanos =
            i32::try_from(observed_at.subsec_nanos()).map_err(|_| MailBootstrapError::Provider)?;
        let connection_id = self.account.connection_id.clone();
        let cursor = if let Some((start_history_id, page_token)) = self
            .durable
            .gmail_history_checkpoint(&connection_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?
        {
            GmailSyncProviderCursorV1::History {
                start_history_id,
                page_token,
            }
        } else {
            GmailSyncProviderCursorV1::Full {
                page_token: self
                    .durable
                    .gmail_sync_progress(&connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?
                    .map(|(page_token, _)| page_token),
            }
        };
        Ok(Some(PreparedGmailSyncProviderOperationV1 {
            connection_id,
            operation_id: pending.operation_id,
            client,
            access_token,
            cursor,
            max_results: u16::try_from(plan.window.min(500))
                .map_err(|_| MailBootstrapError::Admission)?,
            windows: plan.windows,
            observed_at_unix_seconds,
            observed_at_nanos,
            deadline_at_unix_seconds: pending.deadline_at_unix_seconds,
        }))
    }

    pub async fn finalize_gmail_sync_provider_page(
        &mut self,
        delivery: GmailSyncProviderPageDeliveryV1,
    ) -> Result<usize, MailBootstrapError> {
        let GmailSyncProviderPageDeliveryV1 {
            connection_id,
            operation_id,
            page,
            observed_at_unix_seconds,
            observed_at_nanos,
            acknowledgment,
        } = delivery;
        let result = self
            .finalize_gmail_sync_provider_page_inner(
                &connection_id,
                &operation_id,
                page,
                observed_at_unix_seconds,
                observed_at_nanos,
            )
            .await;
        let _ = acknowledgment.send(result.is_ok());
        result
    }

    async fn finalize_gmail_sync_provider_page_inner(
        &mut self,
        connection_id: &str,
        operation_id: &str,
        page: GmailSyncProviderPageV1,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<usize, MailBootstrapError> {
        if connection_id != self.account.connection_id || operation_id.trim().is_empty() {
            return Err(MailBootstrapError::Admission);
        }
        match page {
            GmailSyncProviderPageV1::Full {
                messages,
                next_page_token,
            } => {
                let observed_messages = messages.len();
                let mut observed_history_id = self
                    .durable
                    .gmail_sync_progress(connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?
                    .and_then(|(_, history_id)| history_id);
                let records = self.gmail_message_records(
                    connection_id,
                    messages.into_iter(),
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )?;
                observed_history_id = newer_gmail_history_id(
                    observed_history_id.as_deref(),
                    records.observed_history_id.as_deref(),
                )
                .map(str::to_owned);
                self.durable
                    .record_operational_materializations_and_store_gmail_sync_progress(
                        &records.materializations,
                        connection_id,
                        next_page_token.as_deref(),
                        observed_history_id.as_deref(),
                        observed_at_unix_seconds,
                    )
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                self.admit_owned_attachments(
                    records.attachment_admissions,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .await?;
                Ok(observed_messages)
            }
            GmailSyncProviderPageV1::History {
                messages,
                start_history_id,
                checkpoint_history_id,
                next_page_token,
            } => {
                let observed_messages = messages.len();
                let records = self.gmail_message_records(
                    connection_id,
                    messages.into_iter(),
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )?;
                let next_checkpoint = if next_page_token.is_some() {
                    &start_history_id
                } else {
                    &checkpoint_history_id
                };
                self.durable
                    .record_operational_materializations_and_store_gmail_history_checkpoint(
                        &records.materializations,
                        connection_id,
                        next_checkpoint,
                        next_page_token.as_deref(),
                        observed_at_unix_seconds,
                    )
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                self.admit_owned_attachments(
                    records.attachment_admissions,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .await?;
                Ok(observed_messages)
            }
        }
    }

    pub async fn finalize_gmail_sync_provider_operation(
        &mut self,
        completed: CompletedGmailSyncProviderOperationV1,
        completed_at_fallback_unix_seconds: i64,
    ) -> Result<(), MailBootstrapError> {
        if completed.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        let result = match completed.outcome {
            GmailSyncProviderOutcomeV1::Complete => Ok(completed.observed_messages),
            GmailSyncProviderOutcomeV1::HistoryExpired => {
                self.durable
                    .clear_gmail_history_checkpoint(&completed.connection_id)
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                self.pending_sync_operation = Some(PendingMailSyncOperationV1 {
                    operation_id: completed.operation_id,
                    deadline_at_unix_seconds: completed.deadline_at_unix_seconds,
                });
                return Ok(());
            }
            GmailSyncProviderOutcomeV1::Failed(failure) => Err(match failure {
                GmailSyncProviderFailureV1::Credential => MailBootstrapError::Credential,
                GmailSyncProviderFailureV1::Provider => MailBootstrapError::Provider,
                GmailSyncProviderFailureV1::Finalization => MailBootstrapError::Persistence,
            }),
        };
        self.complete_sync_operation(
            &completed.operation_id,
            result,
            completed_at_fallback_unix_seconds,
        )
        .await
        .map(|_| ())
    }

    pub async fn finalize_imap_sync_provider_operation(
        &mut self,
        completed: CompletedImapSyncProviderOperationV1,
        completed_at_fallback_unix_seconds: i64,
    ) -> Result<(), MailBootstrapError> {
        if completed.connection_id != self.account.connection_id {
            return Err(MailBootstrapError::Admission);
        }
        self.complete_sync_operation(
            &completed.operation_id,
            completed.result,
            completed_at_fallback_unix_seconds,
        )
        .await
        .map(|_| ())
    }

    pub async fn expire_pending_sync_operation(
        &mut self,
        now_unix_seconds: i64,
    ) -> Result<bool, MailBootstrapError> {
        let Some(pending) = self.pending_sync_operation.as_ref() else {
            return Ok(false);
        };
        if now_unix_seconds < pending.deadline_at_unix_seconds {
            return Ok(false);
        }
        let operation_id = pending.operation_id.clone();
        self.pending_sync_operation = None;
        self.expire_sync_operation(&operation_id, now_unix_seconds)
            .await?;
        Ok(true)
    }

    pub async fn expire_sync_operation(
        &mut self,
        operation_id: &str,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailBootstrapError> {
        self.durable
            .complete_sync_run(
                operation_id,
                MailSyncOutcomeV1::Failed,
                0,
                Some(MailSyncFailureCodeV1::DeadlineExceeded),
                completed_at_unix_seconds,
            )
            .await
            .map(|_| ())
            .map_err(|_| MailBootstrapError::Persistence)
    }

    pub async fn finalize_imap_sync_provider_page(
        &mut self,
        delivery: ImapSyncProviderPageDeliveryV1,
    ) -> Result<usize, MailBootstrapError> {
        let ImapSyncProviderPageDeliveryV1 {
            connection_id,
            operation_id,
            sync,
            acknowledgment,
        } = delivery;
        let result =
            if connection_id == self.account.connection_id && !operation_id.trim().is_empty() {
                self.apply_imap_sync_result(ImapInboxSyncRequestV1 {
                    connection_id: &connection_id,
                    sync: &sync,
                })
                .await
            } else {
                Err(MailBootstrapError::Admission)
            };
        let _ = acknowledgment.send(result.is_ok());
        result
    }

    async fn complete_sync_operation(
        &mut self,
        operation_id: &str,
        result: Result<usize, MailBootstrapError>,
        completed_at_fallback_unix_seconds: i64,
    ) -> Result<usize, MailBootstrapError> {
        let completed_at_unix_seconds =
            current_unix_seconds().unwrap_or(completed_at_fallback_unix_seconds);
        match result {
            Ok(observed_messages) => {
                self.durable
                    .complete_sync_run(
                        operation_id,
                        MailSyncOutcomeV1::Succeeded,
                        u64::try_from(observed_messages)
                            .map_err(|_| MailBootstrapError::Persistence)?,
                        None,
                        completed_at_unix_seconds,
                    )
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                Ok(observed_messages)
            }
            Err(error) => {
                self.durable
                    .complete_sync_run(
                        operation_id,
                        MailSyncOutcomeV1::Failed,
                        0,
                        Some(bootstrap_error_to_sync_failure(&error)),
                        completed_at_unix_seconds,
                    )
                    .await
                    .map_err(|_| MailBootstrapError::Persistence)?;
                Err(error)
            }
        }
    }

    async fn apply_imap_sync_result(
        &mut self,
        request: ImapInboxSyncRequestV1<'_>,
    ) -> Result<usize, MailBootstrapError> {
        let ImapInboxSyncRequestV1 {
            connection_id,
            sync,
        } = request;
        if connection_id.trim().is_empty() {
            return Err(MailBootstrapError::Admission);
        }
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_unix_seconds =
            i64::try_from(observed_at.as_secs()).map_err(|_| MailBootstrapError::Provider)?;
        let observed_at_nanos =
            i32::try_from(observed_at.subsec_nanos()).map_err(|_| MailBootstrapError::Provider)?;
        let folders = sync
            .mailboxes
            .iter()
            .map(imap_operational_folder)
            .collect::<Vec<_>>();
        self.durable
            .record_operational_folders(connection_id, &folders, observed_at_unix_seconds)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        let selected_folder = sync
            .mailboxes
            .iter()
            .find(|mailbox| mailbox.mailbox_id == sync.selected_mailbox.mailbox_id)
            .map(imap_operational_folder)
            .ok_or(MailBootstrapError::Provider)?;
        let mut materializations = Vec::with_capacity(sync.messages.len());
        let mut attachment_admissions = Vec::new();
        for message in &sync.messages {
            let locator = MailImapMessageLocatorV1 {
                mailbox_id: sync.selected_mailbox.mailbox_id.clone(),
                uid_validity: sync.selected_mailbox.uid_validity,
                uid: message.uid,
            };
            let legacy_message_id = message.uid.to_string();
            let message_id = self
                .durable
                .resolve_imap_message_id(connection_id, &locator, &legacy_message_id)
                .await
                .map_err(|_| MailBootstrapError::Persistence)?
                .map_or_else(|| initial_imap_message_id(connection_id, &locator), Ok)
                .map_err(|_| MailBootstrapError::Persistence)?;
            let observation = self.draft_inbound_body_observation(
                ProviderProvenanceV1::MailImap,
                connection_id,
                InboundBodyObservationSourceV1 {
                    source_id: message_id.clone(),
                    sender: message.sender.clone(),
                    subject: Some(message.subject.clone()),
                    body: message.body_content.clone(),
                },
            )?;
            let record = build_observation_outbox_record_v1(
                &observation,
                &observation_context(
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                ),
            )
            .map_err(|_| MailBootstrapError::Admission)?;
            let observation_anchor_id = *record.message_id();
            let provider_thread_id = format!("imap-message:{message_id}");
            let delivery_route_locator = mail_delivery_route_locator(
                &observation,
                connection_id,
                &provider_thread_id,
                &message_id,
                message.sender.clone(),
                message.recipients.clone(),
                message.subject.clone(),
            )?;
            let mut records = vec![record];
            for attachment in &message.attachments {
                let source_id = message_id.clone();
                let media_id = format!("{message_id}:{}", attachment.part_id);
                let disposition = match attachment.disposition {
                    hermes_mail_imap::ImapAttachmentDisposition::Attachment => {
                        AttachmentDispositionV1::Attachment
                    }
                    hermes_mail_imap::ImapAttachmentDisposition::Inline => {
                        AttachmentDispositionV1::Inline
                    }
                };
                let observation = draft_attachment_ingress_observation(
                    &inbound_observation_id(
                        ProviderProvenanceV1::MailImap,
                        connection_id,
                        &message_id,
                        Some(attachment.part_id),
                    ),
                    hermes_mail_core::MailAttachmentIngressRequestV1 {
                        provider: ProviderProvenanceV1::MailImap,
                        account_id: connection_id.to_owned(),
                        message_source_id: source_id,
                        media_id,
                        filename: attachment.filename.clone(),
                        media_type: attachment.media_type.clone(),
                        declared_bytes: attachment.declared_bytes,
                        disposition,
                    },
                )
                .map_err(|_| MailBootstrapError::Provider)?;
                let record = build_observation_outbox_record_v1(
                    &observation,
                    &observation_context(
                        &self.runtime_instance_id,
                        self.runtime_generation,
                        observed_at_unix_seconds,
                        observed_at_nanos,
                    ),
                )
                .map_err(|_| MailBootstrapError::Admission)?;
                let source_observation_id = *record.message_id();
                attachment_admissions.push(OwnedAttachmentBlobAdmissionV1 {
                    source_observation_id,
                    bytes: attachment.bytes().to_vec(),
                    filename: attachment.filename.clone(),
                    media_type: attachment.media_type.clone(),
                    disposition: match attachment.disposition {
                        hermes_mail_imap::ImapAttachmentDisposition::Attachment => {
                            PersistedAttachmentDispositionV1::Attachment
                        }
                        hermes_mail_imap::ImapAttachmentDisposition::Inline => {
                            PersistedAttachmentDispositionV1::Inline
                        }
                    },
                });
                records.push(record);
            }
            materializations.push(MailOperationalMaterializationV1 {
                message: MailOperationalMessageSnapshotV1 {
                    connection_id: connection_id.to_owned(),
                    message_id: message_id.clone(),
                    imap_locator: Some(locator),
                    provider_thread_id,
                    folders: vec![selected_folder.clone()],
                    subject: Some(message.subject.clone()),
                    sender: message.sender.clone(),
                    recipients: message.recipients.clone(),
                    snippet: Some(message.snippet.clone()),
                    sent_at_unix_seconds: message.sent_at_unix_seconds,
                    flags: message
                        .flags
                        .iter()
                        .map(|flag| match flag {
                            hermes_mail_imap::ImapMessageFlag::Read => MailMessageFlagV1::Read,
                            hermes_mail_imap::ImapMessageFlag::Starred => {
                                MailMessageFlagV1::Starred
                            }
                            hermes_mail_imap::ImapMessageFlag::Draft => MailMessageFlagV1::Draft,
                            hermes_mail_imap::ImapMessageFlag::Trashed => {
                                MailMessageFlagV1::Trashed
                            }
                        })
                        .collect(),
                    has_plain_text: message.has_plain_text,
                    has_attachments: !message.attachments.is_empty(),
                    observation_anchor_id,
                },
                delivery_route_locator,
                communications_outbox: records,
            });
        }
        self.durable
            .record_operational_materializations(&materializations, observed_at_unix_seconds)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        self.admit_owned_attachments(
            attachment_admissions,
            observed_at_unix_seconds,
            observed_at_nanos,
        )
        .await?;
        Ok(sync.messages.len())
    }

    fn gmail_message_records(
        &mut self,
        connection_id: &str,
        messages: impl Iterator<Item = (String, GmailRawMessageV1)>,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<GmailMessageRecordsV1, MailBootstrapError> {
        let mut materializations = Vec::new();
        let mut attachment_admissions = Vec::new();
        let mut observed_history_id = None;
        for (message_id, raw) in messages {
            let bytes = raw
                .raw
                .as_deref()
                .ok_or(MailBootstrapError::Provider)
                .and_then(|value| {
                    decode_raw_rfc822(value).map_err(|_| MailBootstrapError::Provider)
                })?;
            observed_history_id =
                newer_gmail_history_id(observed_history_id.as_deref(), raw.history_id.as_deref())
                    .map(str::to_owned);
            let provider_record_id = raw.id.unwrap_or(message_id);
            let provider_thread_id = raw
                .thread_id
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| format!("gmail-message:{provider_record_id}"));
            let label_ids = raw.label_ids.unwrap_or_default();
            let sent_at_unix_seconds = raw
                .internal_date
                .as_deref()
                .and_then(gmail_internal_date_unix_seconds);
            let preview = operational_preview(&bytes);
            let subject = preview
                .as_ref()
                .and_then(|preview| preview.subject.clone())
                .filter(|value| !value.trim().is_empty());
            let observation = self.draft_inbound_body_observation(
                ProviderProvenanceV1::MailGmail,
                connection_id,
                InboundBodyObservationSourceV1 {
                    source_id: format!("{connection_id}:{provider_record_id}"),
                    sender: preview.as_ref().and_then(|preview| preview.sender.clone()),
                    subject: subject.clone(),
                    body: readable_body_content(&bytes),
                },
            )?;
            let primary_record = build_observation_outbox_record_v1(
                &observation,
                &observation_context(
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                ),
            )
            .map_err(|_| MailBootstrapError::Admission)?;
            let observation_anchor_id = *primary_record.message_id();
            let subject = subject.unwrap_or_default();
            let sender = preview.as_ref().and_then(|preview| preview.sender.clone());
            let recipients = preview
                .as_ref()
                .map(|preview| preview.recipients.clone())
                .unwrap_or_default();
            let delivery_route_locator = mail_delivery_route_locator(
                &observation,
                connection_id,
                &provider_thread_id,
                &provider_record_id,
                sender.clone(),
                recipients.clone(),
                subject.clone(),
            )?;
            let mut records = vec![primary_record];
            for attachment in attachment_metadata(&bytes) {
                let attachment_bytes = extract_attachment_part(&bytes, attachment.part_id)
                    .map_err(|_| MailBootstrapError::Provider)?;
                let source_id = format!("{connection_id}:{provider_record_id}");
                let media_id = format!("{}:{}", provider_record_id, attachment.part_id);
                let disposition = match attachment.disposition {
                    Rfc822AttachmentDispositionV1::Attachment => {
                        AttachmentDispositionV1::Attachment
                    }
                    Rfc822AttachmentDispositionV1::Inline => AttachmentDispositionV1::Inline,
                };
                let observation = draft_attachment_ingress_observation(
                    &inbound_observation_id(
                        ProviderProvenanceV1::MailGmail,
                        connection_id,
                        &provider_record_id,
                        Some(attachment.part_id),
                    ),
                    hermes_mail_core::MailAttachmentIngressRequestV1 {
                        provider: ProviderProvenanceV1::MailGmail,
                        account_id: connection_id.to_owned(),
                        message_source_id: source_id,
                        media_id,
                        filename: attachment.filename.clone(),
                        media_type: attachment.media_type.clone(),
                        declared_bytes: attachment.declared_bytes,
                        disposition,
                    },
                )
                .map_err(|_| MailBootstrapError::Provider)?;
                let record = build_observation_outbox_record_v1(
                    &observation,
                    &observation_context(
                        &self.runtime_instance_id,
                        self.runtime_generation,
                        observed_at_unix_seconds,
                        observed_at_nanos,
                    ),
                )
                .map_err(|_| MailBootstrapError::Admission)?;
                attachment_admissions.push(OwnedAttachmentBlobAdmissionV1 {
                    source_observation_id: *record.message_id(),
                    bytes: attachment_bytes,
                    filename: attachment.filename,
                    media_type: attachment.media_type,
                    disposition: match attachment.disposition {
                        Rfc822AttachmentDispositionV1::Attachment => {
                            PersistedAttachmentDispositionV1::Attachment
                        }
                        Rfc822AttachmentDispositionV1::Inline => {
                            PersistedAttachmentDispositionV1::Inline
                        }
                    },
                });
                records.push(record);
            }
            materializations.push(MailOperationalMaterializationV1 {
                message: MailOperationalMessageSnapshotV1 {
                    connection_id: connection_id.to_owned(),
                    message_id: provider_record_id,
                    imap_locator: None,
                    provider_thread_id,
                    folders: gmail_operational_folders(&label_ids),
                    subject: Some(subject),
                    sender,
                    recipients,
                    snippet: preview.as_ref().and_then(|preview| preview.snippet.clone()),
                    sent_at_unix_seconds,
                    flags: gmail_operational_flags(&label_ids),
                    has_plain_text: preview
                        .as_ref()
                        .is_some_and(|preview| preview.has_plain_text),
                    has_attachments: records.len() > 1,
                    observation_anchor_id,
                },
                delivery_route_locator,
                communications_outbox: records,
            });
        }
        Ok(GmailMessageRecordsV1 {
            materializations,
            attachment_admissions,
            observed_history_id,
        })
    }

    async fn admit_owned_attachments(
        &mut self,
        admissions: Vec<OwnedAttachmentBlobAdmissionV1>,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<(), MailBootstrapError> {
        for admission in admissions {
            self.try_admit_attachment_blob(MailAttachmentBlobAdmissionRequestV1 {
                source_observation_id: admission.source_observation_id,
                bytes: &admission.bytes,
                filename: admission.filename,
                media_type: admission.media_type,
                disposition: admission.disposition,
                observed_at_unix_seconds,
                observed_at_nanos,
            })
            .await?;
        }
        Ok(())
    }

    fn draft_inbound_body_observation(
        &mut self,
        provider: ProviderProvenanceV1,
        connection_id: &str,
        source: InboundBodyObservationSourceV1,
    ) -> Result<CommunicationObservationDraft, MailBootstrapError> {
        let InboundBodyObservationSourceV1 {
            source_id,
            sender,
            subject,
            body,
        } = source;
        let Some(body) = body else {
            let operation_id = inbound_body_observation_id(
                provider,
                connection_id,
                &source_id,
                BodyObservationRevisionV1::Unavailable(BodyAdmissionFailureV1::PolicyRejected),
            );
            return unavailable_body_observation(
                &operation_id,
                provider,
                connection_id,
                source_id,
                sender,
                subject,
                BodyAdmissionFailureV1::PolicyRejected,
            );
        };
        match self.admit_body_content(&body) {
            Ok(receipt) => {
                let operation_id = inbound_body_observation_id(
                    provider,
                    connection_id,
                    &source_id,
                    BodyObservationRevisionV1::Admitted {
                        sha256: receipt.sha256,
                        media_type: &receipt.media_type,
                        source_receipt_binding_sha256: Sha256::digest(
                            &receipt.custody_transfer_source_proof,
                        )
                        .into(),
                    },
                );
                with_admitted_body_blob(
                    draft_ingress_observation_with_sender_subject_body(
                        &operation_id,
                        provider,
                        connection_id,
                        source_id,
                        sender,
                        subject,
                        BodyAvailabilityV1::AdmittedBlob,
                    )
                    .map_err(|_| MailBootstrapError::Provider)?,
                    receipt,
                )
                .map_err(|_| MailBootstrapError::Provider)
            }
            Err(failure) => {
                let operation_id = inbound_body_observation_id(
                    provider,
                    connection_id,
                    &source_id,
                    BodyObservationRevisionV1::Unavailable(failure),
                );
                unavailable_body_observation(
                    &operation_id,
                    provider,
                    connection_id,
                    source_id,
                    sender,
                    subject,
                    failure,
                )
            }
        }
    }

    fn admit_body_content(
        &mut self,
        body: &Rfc822BodyContentV1,
    ) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
        let plaintext = body.bytes.as_slice();
        if plaintext.is_empty() || plaintext.len() > hermes_mail_api::MAX_PLAIN_TEXT_BYTES {
            return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
        }
        let mut reference_id = [0_u8; 16];
        getrandom::fill(&mut reference_id)
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        if reference_id.iter().all(|byte| *byte == 0) {
            return Err(BodyAdmissionFailureV1::SourceUnavailable);
        }
        let sha256: [u8; 32] = Sha256::digest(plaintext).into();
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        let mut dispatcher = MailBusyControlDispatcher;
        let session = request_managed_blob_session_v2(
            &mut self.control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: MAIL_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size: u64::try_from(plaintext.len())
                    .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
                backup_class: 1,
                receipt_sha256: Some(&sha256),
                custody_target: Some(ManagedBlobCustodyTargetV1 {
                    owner_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
                    module_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID,
                    capability_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
                }),
            },
        );
        let restored = self.control_channel.inner_mut().set_nonblocking(true);
        let session = session.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
        restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        let custody_transfer_source_proof = session.custody_transfer_source_proof;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.write(session.grant, session.channel_binding, plaintext.to_vec())
            })
            .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
        Ok(BodyBlobReceiptV1 {
            blob_ref: format!("blob-content:{}", hex_reference_id(&reference_id)),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len())
                .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            sha256,
            custody_transfer_source_proof,
            media_type: body.media_type.to_owned(),
        })
    }

    async fn try_admit_attachment_blob(
        &mut self,
        request: MailAttachmentBlobAdmissionRequestV1<'_>,
    ) -> Result<(), MailBootstrapError> {
        let MailAttachmentBlobAdmissionRequestV1 {
            source_observation_id,
            bytes,
            filename,
            media_type,
            disposition,
            observed_at_unix_seconds,
            observed_at_nanos,
        } = request;
        if !self.attachment_blob_admission_publish_permitted {
            return Ok(());
        }
        let Some(mapping) = self
            .durable
            .attachment_anchor_mapping(source_observation_id)
            .await
            .map_err(|_| MailBootstrapError::Persistence)?
        else {
            return Ok(());
        };
        let context = attachment_observation_context(
            &self.runtime_instance_id,
            self.runtime_generation,
            observed_at_unix_seconds,
            observed_at_nanos,
        );
        let requested = build_attachment_blob_admission_outbox_record_v1(
            &AttachmentBlobAdmissionFactV1 {
                attachment_anchor_id: mapping.attachment_anchor_id,
                source_observation_id,
                correlation_id: mapping.correlation_id,
                media_cursor_sha256: mapping.media_cursor_sha256,
                expected_state: AttachmentBlobExpectedStateV1::DescriptorOnly,
                transition: AttachmentBlobAdmissionTransitionV1::Requested,
                observed_at_unix_seconds,
                blob_reference_binding_sha256: None,
            },
            &context,
        )
        .map_err(|_| MailBootstrapError::Admission)?;
        let outcome = self
            .durable
            .begin_attachment_blob_admission(
                source_observation_id,
                mapping.attachment_anchor_id,
                &requested,
                observed_at_unix_seconds,
            )
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        if !matches!(
            outcome,
            hermes_mail_persistence::MailAttachmentBlobAdmissionStartOutcomeV1::Started
        ) {
            return Ok(());
        }
        let write = self.write_attachment_blob(bytes);
        let terminal = match &write {
            Ok(write) => (
                2,
                AttachmentBlobAdmissionTransitionV1::Admitted,
                Some(write.reference_binding_sha256),
            ),
            Err(_) => (3, AttachmentBlobAdmissionTransitionV1::Rejected, None),
        };
        let terminal_record = build_attachment_blob_admission_outbox_record_v1(
            &AttachmentBlobAdmissionFactV1 {
                attachment_anchor_id: mapping.attachment_anchor_id,
                source_observation_id,
                correlation_id: mapping.correlation_id,
                media_cursor_sha256: mapping.media_cursor_sha256,
                expected_state: AttachmentBlobExpectedStateV1::BlobPending,
                transition: terminal.1,
                observed_at_unix_seconds,
                blob_reference_binding_sha256: terminal.2,
            },
            &context,
        )
        .map_err(|_| MailBootstrapError::Admission)?;
        let attachment_security_record = write
            .as_ref()
            .ok()
            .map(|write| {
                build_attachment_security_scan_candidate_outbox_record_v1(
                    &AttachmentSecurityScanCandidateFactV1 {
                        attachment_anchor_id: mapping.attachment_anchor_id,
                        blob_reference_id: write.reference_id,
                        declared_size: write.declared_size,
                        blob_receipt_sha256: write.receipt_sha256,
                        custody_transfer_source_proof: write.custody_transfer_source_proof.clone(),
                        source_observation_id,
                        correlation_id: mapping.correlation_id,
                        observed_at_unix_seconds,
                    },
                    &AttachmentSecurityObservationContextV1 {
                        runtime_instance_id: self.runtime_instance_id.clone(),
                        runtime_generation: self.runtime_generation,
                        module_id: MAIL_MODULE_ID.to_owned(),
                        recorded_at_unix_seconds: observed_at_unix_seconds,
                        recorded_at_nanos: observed_at_nanos,
                    },
                )
                .map_err(|_| MailBootstrapError::Admission)
            })
            .transpose()?;
        let materialization = write
            .as_ref()
            .ok()
            .map(|write| MailAttachmentMaterializationV1 {
                source_observation_id,
                attachment_anchor_id: mapping.attachment_anchor_id,
                blob_reference_id: write.reference_id,
                receipt_sha256: write.receipt_sha256,
                declared_size: write.declared_size,
                filename,
                media_type,
                disposition,
            });
        self.durable
            .complete_attachment_blob_admission(MailAttachmentBlobAdmissionCompletionV1 {
                source_observation_id,
                attachment_anchor_id: mapping.attachment_anchor_id,
                terminal_state: terminal.0,
                terminal_record: &terminal_record,
                attachment_security_record: attachment_security_record.as_ref(),
                materialization: materialization.as_ref(),
                completed_at_unix_seconds: observed_at_unix_seconds,
            })
            .await
            .map_err(|_| MailBootstrapError::Persistence)?;
        Ok(())
    }

    fn write_attachment_blob(
        &mut self,
        bytes: &[u8],
    ) -> Result<MailAttachmentBlobWriteV1, MailBootstrapError> {
        if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
            return Err(MailBootstrapError::Admission);
        }
        let mut reference_id = [0_u8; 16];
        getrandom::fill(&mut reference_id).map_err(|_| MailBootstrapError::Control)?;
        if reference_id.iter().all(|byte| *byte == 0) {
            return Err(MailBootstrapError::Control);
        }
        let receipt_sha256: [u8; 32] = Sha256::digest(bytes).into();
        let declared_size =
            u64::try_from(bytes.len()).map_err(|_| MailBootstrapError::Admission)?;
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| MailBootstrapError::Control)?;
        let mut dispatcher = MailBusyControlDispatcher;
        let session = request_managed_blob_session_v2(
            &mut self.control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: MAIL_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size,
                backup_class: 1,
                receipt_sha256: Some(&receipt_sha256),
                custody_target: Some(ManagedBlobCustodyTargetV1 {
                    owner_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID,
                    module_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID,
                    capability_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
                }),
            },
        );
        let restored = self.control_channel.inner_mut().set_nonblocking(true);
        let session = session.map_err(|_| MailBootstrapError::Control)?;
        restored.map_err(|_| MailBootstrapError::Control)?;
        if session.custody_transfer_source_proof.is_empty() {
            return Err(MailBootstrapError::Control);
        }
        let custody_transfer_source_proof = session.custody_transfer_source_proof;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
            .map_err(|_| MailBootstrapError::Control)?;
        Ok(MailAttachmentBlobWriteV1 {
            reference_id,
            receipt_sha256,
            reference_binding_sha256: Sha256::digest(&custody_transfer_source_proof).into(),
            custody_transfer_source_proof,
            declared_size,
        })
    }
}

#[must_use]
pub fn execute_imap_sync_provider_operation(
    prepared: PreparedImapSyncProviderOperationV1,
    page_sender: std::sync::mpsc::SyncSender<ImapSyncProviderPageDeliveryV1>,
) -> CompletedImapSyncProviderOperationV1 {
    let PreparedImapSyncProviderOperationV1 {
        connection_id,
        operation_id,
        host,
        port,
        username,
        password,
        window,
        windows,
        priority_uids,
        deadline_at_unix_seconds: _,
    } = prepared;
    let result = match std::str::from_utf8(&password) {
        Err(_) => Err(MailBootstrapError::Credential),
        Ok(password) => {
            let mut finalization_rejected = false;
            let provider_result = hermes_mail_imap::sync_inbox_prioritized(
                &host,
                port,
                &username,
                Some(password),
                window,
                windows,
                &priority_uids,
                |sync| {
                    let (acknowledgment, committed) = std::sync::mpsc::channel();
                    page_sender
                        .send(ImapSyncProviderPageDeliveryV1 {
                            connection_id: connection_id.clone(),
                            operation_id: operation_id.clone(),
                            sync,
                            acknowledgment,
                        })
                        .map_err(|_| {
                            finalization_rejected = true;
                        })?;
                    if committed.recv().is_ok_and(|committed| committed) {
                        Ok(())
                    } else {
                        finalization_rejected = true;
                        Err(())
                    }
                },
            );
            if finalization_rejected {
                Err(MailBootstrapError::Persistence)
            } else {
                provider_result.map_err(|_| MailBootstrapError::Provider)
            }
        }
    };
    CompletedImapSyncProviderOperationV1 {
        connection_id,
        operation_id,
        result,
    }
}

fn gmail_internal_date_unix_seconds(value: &str) -> Option<i64> {
    value
        .parse::<i64>()
        .ok()
        .and_then(|milliseconds| milliseconds.checked_div(1_000))
        .filter(|seconds| *seconds > 0)
}

fn imap_operational_folder(
    mailbox: &hermes_mail_imap::ImapMailboxV1,
) -> MailOperationalFolderSnapshotV1 {
    let kind = match mailbox.kind {
        ImapMailboxKindV1::Inbox => MailFolderKindV1::Inbox,
        ImapMailboxKindV1::Archive => MailFolderKindV1::Archive,
        ImapMailboxKindV1::Trash => MailFolderKindV1::Trash,
        ImapMailboxKindV1::Sent => MailFolderKindV1::Sent,
        ImapMailboxKindV1::Drafts => MailFolderKindV1::Drafts,
        ImapMailboxKindV1::Spam => MailFolderKindV1::Spam,
        ImapMailboxKindV1::All | ImapMailboxKindV1::ProviderFolder => {
            MailFolderKindV1::ProviderLabel
        }
    };
    MailOperationalFolderSnapshotV1 {
        folder_id: mailbox.mailbox_id.clone(),
        display_name: mailbox.display_name.clone(),
        kind,
    }
}

fn gmail_operational_folders(label_ids: &[String]) -> Vec<MailOperationalFolderSnapshotV1> {
    let mut folders = vec![MailOperationalFolderSnapshotV1 {
        folder_id: "ALL_MAIL".to_owned(),
        display_name: "All Mail".to_owned(),
        kind: MailFolderKindV1::Archive,
    }];
    for label_id in label_ids {
        let (display_name, kind) = match label_id.as_str() {
            "INBOX" => ("Inbox", MailFolderKindV1::Inbox),
            "SENT" => ("Sent", MailFolderKindV1::Sent),
            "DRAFT" => ("Drafts", MailFolderKindV1::Drafts),
            "TRASH" => ("Trash", MailFolderKindV1::Trash),
            "SPAM" => ("Spam", MailFolderKindV1::Spam),
            "UNREAD" | "STARRED" | "IMPORTANT" => continue,
            _ if valid_gmail_operational_label(label_id) => {
                (label_id.as_str(), MailFolderKindV1::ProviderLabel)
            }
            _ => continue,
        };
        if folders.iter().all(|folder| folder.folder_id != *label_id) {
            folders.push(MailOperationalFolderSnapshotV1 {
                folder_id: label_id.clone(),
                display_name: display_name.to_owned(),
                kind,
            });
        }
    }
    folders
}

fn gmail_operational_flags(label_ids: &[String]) -> Vec<MailMessageFlagV1> {
    let contains = |label: &str| label_ids.iter().any(|value| value == label);
    let mut flags = Vec::with_capacity(6);
    if !contains("UNREAD") {
        flags.push(MailMessageFlagV1::Read);
    }
    for (label, flag) in [
        ("STARRED", MailMessageFlagV1::Starred),
        ("DRAFT", MailMessageFlagV1::Draft),
        ("SENT", MailMessageFlagV1::Sent),
        ("TRASH", MailMessageFlagV1::Trashed),
        ("SPAM", MailMessageFlagV1::Spam),
    ] {
        if contains(label) {
            flags.push(flag);
        }
    }
    flags
}

fn queued_matches_command(
    queued: &MailQueuedMessageFlagCommandV1,
    command: &MailMessageFlagCommandV1,
) -> bool {
    queued.operation_id == command.operation_id
        && queued.connection_id == command.connection_id
        && queued.message_id == command.message_id
        && queued.kind == command.kind
        && queued.target_value == command.target_value
}

fn queued_location_matches_command(
    queued: &MailQueuedMessageLocationCommandV1,
    command: &MailMessageLocationCommandV1,
) -> bool {
    queued.operation_id == command.operation_id
        && queued.connection_id == command.connection_id
        && queued.message_id == command.message_id
        && queued.kind == command.kind
        && queued.target_folder_id == command.target_folder_id
}

fn queued_permanent_delete_matches_command(
    queued: &MailQueuedMessagePermanentDeleteCommandV1,
    command: &MailMessagePermanentDeleteCommandV1,
) -> bool {
    queued.operation_id == command.operation_id
        && queued.connection_id == command.connection_id
        && queued.message_id == command.message_id
        && queued.expected_projection_revision == command.expected_projection_revision
}

fn map_gmail_location_error(error: GmailAdapterErrorV1) -> MailMessageLocationDispatchErrorV1 {
    match error {
        GmailAdapterErrorV1::InvalidRequest | GmailAdapterErrorV1::ProviderStatus(400..=499) => {
            MailMessageLocationDispatchErrorV1::ProviderRejected
        }
        GmailAdapterErrorV1::Transport
        | GmailAdapterErrorV1::ProviderStatus(_)
        | GmailAdapterErrorV1::InvalidResponse => {
            MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown
        }
    }
}

fn map_gmail_permanent_delete_error(
    error: GmailAdapterErrorV1,
) -> MailMessagePermanentDeleteDispatchErrorV1 {
    match error {
        GmailAdapterErrorV1::InvalidRequest | GmailAdapterErrorV1::ProviderStatus(400..=499) => {
            MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected
        }
        GmailAdapterErrorV1::Transport
        | GmailAdapterErrorV1::ProviderStatus(_)
        | GmailAdapterErrorV1::InvalidResponse => {
            MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown
        }
    }
}

const fn imap_message_flag(kind: MailMessageFlagKindV1) -> ImapMutableMessageFlagV1 {
    match kind {
        MailMessageFlagKindV1::Read => ImapMutableMessageFlagV1::Read,
        MailMessageFlagKindV1::Starred => ImapMutableMessageFlagV1::Starred,
    }
}

const fn gmail_message_flag(kind: MailMessageFlagKindV1) -> GmailMutableMessageFlagV1 {
    match kind {
        MailMessageFlagKindV1::Read => GmailMutableMessageFlagV1::Read,
        MailMessageFlagKindV1::Starred => GmailMutableMessageFlagV1::Starred,
    }
}

fn valid_gmail_operational_label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 512 && !value.contains(['\0', '\r', '\n'])
}

pub(crate) fn execute_blocking_provider_credential_request<T>(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    request: impl FnOnce(
        &mut ManagedControlChannelV2<UnixStream>,
    ) -> Result<T, ManagedProviderCredentialErrorV1>,
) -> Result<T, ManagedProviderCredentialErrorV1> {
    let configure = control_channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_read_timeout(Some(CONTROL_TIMEOUT))
        })
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(CONTROL_TIMEOUT))
        });
    if configure.is_err() {
        restore_nonblocking_control_stream(control_channel.inner_mut());
        return Err(ManagedProviderCredentialErrorV1::Unavailable);
    }
    let result = request(control_channel);
    if !restore_nonblocking_control_stream(control_channel.inner_mut()) {
        return Err(ManagedProviderCredentialErrorV1::Unavailable);
    }
    result
}

fn restore_nonblocking_control_stream(stream: &mut UnixStream) -> bool {
    let read_timeout_cleared = stream.set_read_timeout(None).is_ok();
    let write_timeout_cleared = stream.set_write_timeout(None).is_ok();
    let nonblocking_restored = stream.set_nonblocking(true).is_ok();
    read_timeout_cleared && write_timeout_cleared && nonblocking_restored
}

pub(crate) struct MailBusyControlDispatcher;

impl ManagedControlRequestDispatcherV2<UnixStream> for MailBusyControlDispatcher {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "RUNTIME_BUSY".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

fn write_client_delivery_response(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    response: ModuleClientResponseV1,
) -> Result<(), MailBootstrapError> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ClientDelivery(
                    ManagedRuntimeClientDeliveryResponseV1 {
                        response: Some(response),
                    },
                )),
                error_code: String::new(),
            },
        )
        .map_err(|error| {
            if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_mail_client_response_write_error={error:?}");
            }
            MailBootstrapError::Control
        })
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    error_code: &str,
) -> Result<(), MailBootstrapError> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| MailBootstrapError::Control)
}

fn attachment_blob_admission_publish_permitted(
    permit: &RuntimePublishPermitV1,
) -> Result<bool, MailBootstrapError> {
    let contract = hermes_communications_attachment_contract::admission::communication_attachment_blob_admission_observed_contract_reference_v1();
    let subject = DurableSubjectV1::new(
        StreamKindV1::Observation,
        contract.owner,
        contract.name,
        contract.major,
    )
    .map_err(|_| MailBootstrapError::EventHub)?;
    Ok(permit.permits_subject(&subject))
}

fn attachment_security_scan_candidate_publish_permitted(
    permit: &RuntimePublishPermitV1,
) -> Result<bool, MailBootstrapError> {
    let contract = hermes_attachment_security_contract::admission::
        attachment_security_scan_candidate_observed_contract_reference_v1();
    let subject = DurableSubjectV1::new(
        StreamKindV1::Observation,
        contract.owner,
        contract.name,
        contract.major,
    )
    .map_err(|_| MailBootstrapError::EventHub)?;
    Ok(permit.permits_subject(&subject))
}

struct MailEventSubscribePermitsV1 {
    anchor: Option<RuntimeSubscribePermitV1>,
    safety: Option<RuntimeSubscribePermitV1>,
    delivery_intent: RuntimeSubscribePermitV1,
    address_book_fetch: RuntimeSubscribePermitV1,
    address_book_upsert: RuntimeSubscribePermitV1,
    replay_command: RuntimeSubscribePermitV1,
}

fn bind_event_subscribe_permits(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<MailEventSubscribePermitsV1, MailBootstrapError> {
    let expected_anchor = hermes_communications_attachment_contract::admission::
        communication_attachment_anchor_recorded_contract_reference_v1();
    let expected_safety = hermes_communications_attachment_contract::admission::
        communication_attachment_safety_state_changed_contract_reference_v1();
    let expected_delivery_intent =
        hermes_mail_delivery_intent_contract::mail_delivery_intent_execute_contract_reference_v1();
    let expected_replay_command = mail_replay_command_contract_reference_v1();
    let expected_address_book_upsert =
        hermes_mail_address_book_contract::MailAddressBookContractV1::UpsertEntryCommand
            .reference();
    let expected_address_book_fetch =
        hermes_mail_address_book_contract::MailAddressBookContractV1::FetchPageCommand.reference();
    let mut anchor = None;
    let mut safety = None;
    let mut delivery_intent = None;
    let mut replay_command = None;
    let mut address_book_upsert = None;
    let mut address_book_fetch = None;
    for permit in permits {
        let Some(contract) = permit.contract() else {
            return Err(MailBootstrapError::EventHub);
        };
        if exact_runtime_contract(contract, &expected_anchor) {
            if anchor.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else if exact_runtime_contract(contract, &expected_safety) {
            if safety.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else if exact_runtime_contract(contract, &expected_delivery_intent) {
            if delivery_intent.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else if exact_runtime_contract(contract, &expected_replay_command) {
            if replay_command.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else if exact_runtime_contract(contract, &expected_address_book_upsert) {
            if address_book_upsert.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else if exact_runtime_contract(contract, &expected_address_book_fetch) {
            if address_book_fetch.replace(permit).is_some() {
                return Err(MailBootstrapError::EventHub);
            }
        } else {
            return Err(MailBootstrapError::EventHub);
        }
    }
    Ok(MailEventSubscribePermitsV1 {
        anchor,
        safety,
        delivery_intent: delivery_intent.ok_or(MailBootstrapError::EventHub)?,
        address_book_fetch: address_book_fetch.ok_or(MailBootstrapError::EventHub)?,
        address_book_upsert: address_book_upsert.ok_or(MailBootstrapError::EventHub)?,
        replay_command: replay_command.ok_or(MailBootstrapError::EventHub)?,
    })
}

fn exact_runtime_contract(
    actual: &hermes_runtime_protocol::v1::ContractReferenceV1,
    expected: &hermes_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    actual.owner == expected.owner
        && actual.name == expected.name
        && actual.major == expected.major
        && actual.revision == expected.revision
        && actual.schema_sha256 == expected.schema_sha256
}

fn map_attachment_anchor_mapping_error(
    error: MailAttachmentAnchorMappingErrorV1,
) -> MailBootstrapError {
    let _ = error;
    MailBootstrapError::AttachmentAnchorMapping
}

fn map_attachment_safety_projection_error(
    error: MailAttachmentSafetyProjectionErrorV1,
) -> MailBootstrapError {
    let _ = error;
    MailBootstrapError::AttachmentAnchorMapping
}

fn map_account_lifecycle_error(error: MailAccountLifecycleRuntimeErrorV1) -> MailBootstrapError {
    match error {
        MailAccountLifecycleRuntimeErrorV1::Admission => MailBootstrapError::Admission,
        MailAccountLifecycleRuntimeErrorV1::Persistence(_) => MailBootstrapError::Persistence,
    }
}

fn developer_admission_diagnostic(stage: &str) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_admission_stage={stage}");
    }
}

fn map_sync_persistence_error(error: MailDurablePersistenceError) -> MailBootstrapError {
    match error {
        MailDurablePersistenceError::ConflictingSyncOperation
        | MailDurablePersistenceError::SyncRunInProgress
        | MailDurablePersistenceError::InvalidSyncTransition
        | MailDurablePersistenceError::InvalidRow => MailBootstrapError::Admission,
        _ => MailBootstrapError::Persistence,
    }
}

const fn bootstrap_error_to_sync_failure(error: &MailBootstrapError) -> MailSyncFailureCodeV1 {
    match error {
        MailBootstrapError::Admission => MailSyncFailureCodeV1::AdmissionRejected,
        MailBootstrapError::Control => MailSyncFailureCodeV1::ControlUnavailable,
        MailBootstrapError::Storage => MailSyncFailureCodeV1::StorageUnavailable,
        MailBootstrapError::Credential => MailSyncFailureCodeV1::CredentialUnavailable,
        MailBootstrapError::Persistence => MailSyncFailureCodeV1::PersistenceUnavailable,
        MailBootstrapError::Provider => MailSyncFailureCodeV1::ProviderUnavailable,
        MailBootstrapError::EventHub => MailSyncFailureCodeV1::EventHubUnavailable,
        MailBootstrapError::AttachmentAnchorMapping => {
            MailSyncFailureCodeV1::AttachmentAnchorUnavailable
        }
    }
}

fn valid_gmail_history_id(value: Option<&str>) -> Option<&str> {
    value.filter(|history_id| {
        !history_id.is_empty() && history_id.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn newer_gmail_history_id<'a>(
    current: Option<&'a str>,
    candidate: Option<&'a str>,
) -> Option<&'a str> {
    match (
        valid_gmail_history_id(current),
        valid_gmail_history_id(candidate),
    ) {
        (None, value) | (value, None) => value,
        (Some(current), Some(candidate))
            if candidate.len() > current.len()
                || (candidate.len() == current.len() && candidate > current) =>
        {
            Some(candidate)
        }
        (Some(current), Some(_)) => Some(current),
    }
}

#[cfg(test)]
mod gmail_history_checkpoint_tests {
    use super::{newer_gmail_history_id, valid_gmail_history_id};

    #[test]
    fn checkpoint_accepts_only_numeric_ids_and_never_regresses() {
        assert_eq!(valid_gmail_history_id(Some("")), None);
        assert_eq!(valid_gmail_history_id(Some("history-12")), None);
        assert_eq!(valid_gmail_history_id(Some("12")), Some("12"));
        assert_eq!(newer_gmail_history_id(Some("12"), Some("9")), Some("12"));
        assert_eq!(newer_gmail_history_id(Some("12"), Some("100")), Some("100"));
        assert_eq!(newer_gmail_history_id(None, Some("100")), Some("100"));
    }
}

fn unavailable_body_observation(
    operation_id: &str,
    provider: ProviderProvenanceV1,
    connection_id: &str,
    source_id: String,
    sender: Option<String>,
    subject: Option<String>,
    failure: BodyAdmissionFailureV1,
) -> Result<CommunicationObservationDraft, MailBootstrapError> {
    with_body_admission_failure(
        draft_ingress_observation_with_sender_subject_body(
            operation_id,
            provider,
            connection_id,
            source_id,
            sender,
            subject,
            BodyAvailabilityV1::Unavailable,
        )
        .map_err(|_| MailBootstrapError::Provider)?,
        failure,
    )
    .map_err(|_| MailBootstrapError::Provider)
}

fn inbound_observation_id(
    provider: ProviderProvenanceV1,
    connection_id: &str,
    provider_record_id: &str,
    attachment_part_id: Option<u16>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.mail.inbound-observation.v1\0");
    hasher.update(provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(connection_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_record_id.as_bytes());
    hasher.update(b"\0");
    if let Some(part_id) = attachment_part_id {
        hasher.update(part_id.to_be_bytes());
    }
    format!("mail-inbound:{}", hex_digest(&hasher.finalize()))
}

#[derive(Clone, Copy)]
enum BodyObservationRevisionV1<'a> {
    Unavailable(BodyAdmissionFailureV1),
    Admitted {
        sha256: [u8; 32],
        media_type: &'a str,
        source_receipt_binding_sha256: [u8; 32],
    },
}

fn inbound_body_observation_id(
    provider: ProviderProvenanceV1,
    connection_id: &str,
    provider_record_id: &str,
    revision: BodyObservationRevisionV1<'_>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.mail.inbound-body-observation.v6\0");
    hasher.update(provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(connection_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_record_id.as_bytes());
    hasher.update(b"\0");
    match revision {
        BodyObservationRevisionV1::Unavailable(failure) => {
            hasher.update(b"unavailable\0");
            hasher.update([body_admission_failure_code(failure)]);
        }
        BodyObservationRevisionV1::Admitted {
            sha256,
            media_type,
            source_receipt_binding_sha256,
        } => {
            hasher.update(b"admitted\0");
            hasher.update(media_type.as_bytes());
            hasher.update(b"\0");
            hasher.update(sha256);
            hasher.update(source_receipt_binding_sha256);
        }
    }
    format!("mail-inbound-body:{}", hex_digest(&hasher.finalize()))
}

const fn body_admission_failure_code(failure: BodyAdmissionFailureV1) -> u8 {
    match failure {
        BodyAdmissionFailureV1::SourceUnavailable => 1,
        BodyAdmissionFailureV1::SizeLimitExceeded => 2,
        BodyAdmissionFailureV1::IntegrityMismatch => 3,
        BodyAdmissionFailureV1::PolicyRejected => 4,
    }
}

fn mail_delivery_route_locator(
    observation: &CommunicationObservationDraft,
    connection_id: &str,
    provider_thread_id: &str,
    provider_message_id: &str,
    sender: Option<String>,
    recipients: Vec<String>,
    subject: String,
) -> Result<MailDeliveryRouteLocatorV1, MailBootstrapError> {
    let scope = observation
        .source
        .scope
        .as_ref()
        .ok_or(MailBootstrapError::Admission)?;
    if scope.external_account_id != connection_id {
        return Err(MailBootstrapError::Admission);
    }
    let external_conversation_id = scope
        .external_conversation_id
        .as_deref()
        .ok_or(MailBootstrapError::Admission)?;
    Ok(MailDeliveryRouteLocatorV1 {
        account_cursor: account_source_cursor_v1(observation.source.provider, connection_id)
            .map_err(|_| MailBootstrapError::Admission)?,
        conversation_cursor: conversation_source_cursor_v1(
            observation.source.provider,
            connection_id,
            external_conversation_id,
        )
        .map_err(|_| MailBootstrapError::Admission)?,
        source_cursor: scoped_record_source_cursor_v1(
            observation.source.provider,
            connection_id,
            &observation.source.external_record_id,
        )
        .map_err(|_| MailBootstrapError::Admission)?,
        connection_id: connection_id.to_owned(),
        provider_thread_id: provider_thread_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        sender,
        recipients,
        subject,
    })
}

fn hex_digest(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn observation_context(
    runtime_instance_id: &str,
    runtime_generation: u64,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> ObservationEnvelopeContextV1 {
    ObservationEnvelopeContextV1 {
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        module_id: MAIL_MODULE_ID.to_owned(),
        recorded_at_unix_seconds,
        recorded_at_nanos,
    }
}

fn attachment_observation_context(
    runtime_instance_id: &str,
    runtime_generation: u64,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> AttachmentObservationEnvelopeContextV1 {
    AttachmentObservationEnvelopeContextV1 {
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        module_id: MAIL_MODULE_ID.to_owned(),
        recorded_at_unix_seconds,
        recorded_at_nanos,
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use hermes_runtime_protocol::v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlAckV1,
        ModuleClientRequestV1, managed_runtime_control_frame_v2::Frame,
    };
    use hermes_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;

    use super::*;

    #[test]
    fn sync_operation_deadline_is_absolute_and_fails_closed_on_overflow() {
        assert_eq!(sync_operation_deadline(1_000), Some(1_300));
        assert_eq!(sync_operation_deadline(i64::MAX), None);
    }

    #[test]
    fn message_flag_provider_mappings_are_exact_and_owner_local() {
        assert_eq!(
            imap_message_flag(MailMessageFlagKindV1::Read),
            ImapMutableMessageFlagV1::Read
        );
        assert_eq!(
            gmail_message_flag(MailMessageFlagKindV1::Starred),
            GmailMutableMessageFlagV1::Starred
        );
    }

    #[test]
    fn attachment_blob_admission_requires_its_exact_publish_subject() {
        let expected = DurableSubjectV1::new(
            StreamKindV1::Observation,
            "communications",
            "communication_attachment_blob_admission_observed",
            1,
        )
        .expect("subject");
        let permit =
            RuntimePublishPermitV1::new(MAIL_MODULE_ID, "mail-runtime-1", 1, 1, vec![expected])
                .expect("permit");
        assert!(attachment_blob_admission_publish_permitted(&permit).is_ok_and(|value| value));

        let observed_only = RuntimePublishPermitV1::new(
            MAIL_MODULE_ID,
            "mail-runtime-1",
            1,
            1,
            vec![
                DurableSubjectV1::new(
                    StreamKindV1::Observation,
                    "communications",
                    "communication_observed",
                    1,
                )
                .expect("subject"),
            ],
        )
        .expect("permit");
        assert!(
            attachment_blob_admission_publish_permitted(&observed_only).is_ok_and(|value| !value)
        );
    }

    #[test]
    fn observations_use_the_exact_admitted_mail_module_identity() {
        let context = observation_context("mail-runtime-1", 7, 10, 11);

        assert_eq!(context.module_id, MAIL_MODULE_ID);
        assert_eq!(context.runtime_instance_id, "mail-runtime-1");
        assert_eq!(context.runtime_generation, 7);
    }

    #[test]
    fn inbound_identity_is_stable_across_sync_operations_and_distinguishes_parts() {
        let message =
            inbound_observation_id(ProviderProvenanceV1::MailImap, "account-1", "uid-42", None);

        assert_eq!(
            message,
            inbound_observation_id(ProviderProvenanceV1::MailImap, "account-1", "uid-42", None,),
        );
        assert_ne!(
            message,
            inbound_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                Some(1),
            ),
        );
    }

    #[test]
    fn inbound_body_identity_revises_when_admission_or_content_changes() {
        let unavailable = inbound_body_observation_id(
            ProviderProvenanceV1::MailImap,
            "account-1",
            "uid-42",
            BodyObservationRevisionV1::Unavailable(BodyAdmissionFailureV1::PolicyRejected),
        );
        let admitted = inbound_body_observation_id(
            ProviderProvenanceV1::MailImap,
            "account-1",
            "uid-42",
            BodyObservationRevisionV1::Admitted {
                sha256: [7; 32],
                media_type: "text/html",
                source_receipt_binding_sha256: [9; 32],
            },
        );

        assert_ne!(unavailable, admitted);
        assert_eq!(
            admitted,
            inbound_body_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                BodyObservationRevisionV1::Admitted {
                    sha256: [7; 32],
                    media_type: "text/html",
                    source_receipt_binding_sha256: [9; 32],
                },
            )
        );
        assert_ne!(
            admitted,
            inbound_body_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                BodyObservationRevisionV1::Admitted {
                    sha256: [8; 32],
                    media_type: "text/html",
                    source_receipt_binding_sha256: [9; 32],
                },
            )
        );
        assert_ne!(
            admitted,
            inbound_body_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                BodyObservationRevisionV1::Admitted {
                    sha256: [7; 32],
                    media_type: "text/plain",
                    source_receipt_binding_sha256: [9; 32],
                },
            )
        );
        assert_ne!(
            admitted,
            inbound_body_observation_id(
                ProviderProvenanceV1::MailImap,
                "account-1",
                "uid-42",
                BodyObservationRevisionV1::Admitted {
                    sha256: [7; 32],
                    media_type: "text/html",
                    source_receipt_binding_sha256: [10; 32],
                },
            )
        );
    }

    #[test]
    fn nested_client_delivery_gets_a_correlated_busy_response_without_stealing_platform_reply() {
        let (runtime, kernel) = UnixStream::pair().expect("control pair");
        let kernel = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(kernel);
            let (platform_id, _) = channel.receive_request().expect("platform request");
            channel
                .write_request(
                    [7; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::ClientDelivery(
                            ManagedRuntimeClientDeliveryRequestV1 {
                                request: Some(ModuleClientRequestV1 {
                                    protocol_major: 1,
                                    module_id: MAIL_MODULE_ID.to_owned(),
                                    owner_id: MAIL_MODULE_ID.to_owned(),
                                    contract: Some(ContractReferenceV1 {
                                        owner: MAIL_MODULE_ID.to_owned(),
                                        name: "query".to_owned(),
                                        major: 1,
                                        revision: 1,
                                        schema_sha256: vec![1; 32],
                                    }),
                                    request_id: 41,
                                    request_payload: vec![1],
                                    logical_owner_id: String::new(),
                                    authenticated_device_id: String::new(),
                                    authenticated_client_session_id: String::new(),
                                }),
                            },
                        )),
                    },
                )
                .expect("client delivery");
            let nested = channel.read_frame().expect("busy response");
            assert_eq!(
                nested.correlation_id,
                vec![7; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            let Some(Frame::Response(response)) = nested.frame else {
                panic!("nested response");
            };
            let Some(ControlResult::ClientDelivery(delivery)) = response.result else {
                panic!("client delivery response");
            };
            assert_eq!(
                delivery.response.expect("module response").error_code,
                "RUNTIME_BUSY"
            );
            channel
                .write_response(
                    platform_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("platform response");
        });

        let mut channel = ManagedControlChannelV2::new(runtime);
        let mut dispatcher = MailBusyControlDispatcher;
        let response = channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1::default())),
                },
                &mut dispatcher,
            )
            .expect("correlated platform response");
        assert!(matches!(response.result, Some(ControlResult::Ack(_))));
        kernel.join().expect("kernel join");
    }
}

fn hex_reference_id(reference_id: &[u8; 16]) -> String {
    reference_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn gmail_api_client(
    configuration: &MailGmailConfigurationV1,
) -> Result<GmailApiClientV1, GmailAdapterErrorV1> {
    #[cfg(feature = "conformance-test-support")]
    {
        GmailApiClientV1::for_conformance_endpoint(
            &configuration.api_endpoint.host,
            configuration.api_endpoint.port,
            configuration.api_endpoint.ca_certificate_pem.clone(),
            &configuration.user_id,
        )
    }
    #[cfg(not(feature = "conformance-test-support"))]
    {
        if configuration.api_endpoint.host != hermes_mail_api::GMAIL_API_HOST
            || configuration.api_endpoint.port != hermes_mail_api::GMAIL_API_HTTPS_PORT
            || configuration.api_endpoint.ca_certificate_pem.is_some()
        {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        GmailApiClientV1::new(&configuration.user_id)
    }
}

fn basic_binding_status(
    binding: Option<&MailCredentialBindingV1>,
    purpose: MailCredentialPurposeV1,
) -> MailCredentialBindingStatusV1 {
    binding.map_or_else(
        || unconfigured_binding_status(purpose),
        |binding| MailCredentialBindingStatusV1 {
            purpose,
            state: binding.state,
            binding_revision: Some(binding.binding_revision),
            credential_revision: Some(binding.credential_revision),
            applied_runtime_generation: binding.applied_runtime_generation,
        },
    )
}

fn gmail_binding_status(
    purpose: MailCredentialPurposeV1,
    credential_revision: u64,
    runtime_generation: u64,
) -> MailCredentialBindingStatusV1 {
    MailCredentialBindingStatusV1 {
        purpose,
        state: MailCredentialBindingStateV1::Active,
        binding_revision: None,
        credential_revision: Some(credential_revision),
        applied_runtime_generation: Some(runtime_generation),
    }
}

fn unconfigured_binding_status(purpose: MailCredentialPurposeV1) -> MailCredentialBindingStatusV1 {
    MailCredentialBindingStatusV1 {
        purpose,
        state: MailCredentialBindingStateV1::Unconfigured,
        binding_revision: None,
        credential_revision: None,
        applied_runtime_generation: None,
    }
}

fn account_readiness(
    bindings: &[MailCredentialBindingStatusV1],
    runtime_generation: u64,
) -> MailAccountReadinessV1 {
    if bindings
        .iter()
        .any(|binding| binding.state == MailCredentialBindingStateV1::Deleted)
    {
        return MailAccountReadinessV1::Deleted;
    }
    if bindings
        .iter()
        .any(|binding| binding.state == MailCredentialBindingStateV1::Retired)
    {
        return MailAccountReadinessV1::Retired;
    }
    if bindings
        .iter()
        .any(|binding| binding.state == MailCredentialBindingStateV1::PendingRestart)
    {
        return MailAccountReadinessV1::PendingRestart;
    }
    if bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::Active
            && binding.applied_runtime_generation == Some(runtime_generation)
    }) {
        return MailAccountReadinessV1::Ready;
    }
    if bindings
        .iter()
        .all(|binding| binding.state == MailCredentialBindingStateV1::Unconfigured)
    {
        return MailAccountReadinessV1::ConfigurationOnly;
    }
    MailAccountReadinessV1::Degraded
}

fn lifecycle_binding_status(
    progress: &MailCredentialLifecycleProgressV1,
    action: MailAccountLifecycleActionV1,
) -> MailCredentialBindingStatusV1 {
    let state = match progress.state {
        MailCredentialLifecycleStateV1::Completed => match action {
            MailAccountLifecycleActionV1::Retire => MailCredentialBindingStateV1::Retired,
            MailAccountLifecycleActionV1::Delete => MailCredentialBindingStateV1::Deleted,
        },
        MailCredentialLifecycleStateV1::Pending
        | MailCredentialLifecycleStateV1::Rejected
        | MailCredentialLifecycleStateV1::OutcomeUnknown => {
            MailCredentialBindingStateV1::PendingRestart
        }
    };
    MailCredentialBindingStatusV1 {
        purpose: progress.purpose,
        state,
        binding_revision: progress.binding_revision,
        credential_revision: Some(progress.credential_revision),
        applied_runtime_generation: None,
    }
}

fn lifecycle_account_readiness(
    state: MailAccountLifecycleStateV1,
    action: MailAccountLifecycleActionV1,
) -> MailAccountReadinessV1 {
    match state {
        MailAccountLifecycleStateV1::Pending => MailAccountReadinessV1::PendingRestart,
        MailAccountLifecycleStateV1::Completed => match action {
            MailAccountLifecycleActionV1::Retire => MailAccountReadinessV1::Retired,
            MailAccountLifecycleActionV1::Delete => MailAccountReadinessV1::Deleted,
        },
        MailAccountLifecycleStateV1::Rejected | MailAccountLifecycleStateV1::OutcomeUnknown => {
            MailAccountReadinessV1::Degraded
        }
    }
}

fn lifecycle_path_readiness<'a>(
    progress: impl Iterator<Item = &'a MailCredentialLifecycleProgressV1>,
    action: MailAccountLifecycleActionV1,
) -> MailProviderPathReadinessV1 {
    let states = progress.map(|progress| progress.state).collect::<Vec<_>>();
    if states.is_empty() {
        return MailProviderPathReadinessV1::NotConfigured;
    }
    if states.contains(&MailCredentialLifecycleStateV1::Rejected)
        || states.contains(&MailCredentialLifecycleStateV1::OutcomeUnknown)
    {
        return MailProviderPathReadinessV1::Degraded;
    }
    if states.contains(&MailCredentialLifecycleStateV1::Pending) {
        return MailProviderPathReadinessV1::PendingRestart;
    }
    match action {
        MailAccountLifecycleActionV1::Retire => MailProviderPathReadinessV1::Retired,
        MailAccountLifecycleActionV1::Delete => MailProviderPathReadinessV1::Deleted,
    }
}

fn provider_path_readiness(
    bindings: &[MailCredentialBindingStatusV1],
    runtime_generation: u64,
) -> MailProviderPathReadinessV1 {
    match account_readiness(bindings, runtime_generation) {
        MailAccountReadinessV1::ConfigurationOnly => {
            MailProviderPathReadinessV1::CredentialRequired
        }
        MailAccountReadinessV1::PendingRestart => MailProviderPathReadinessV1::PendingRestart,
        MailAccountReadinessV1::Ready => MailProviderPathReadinessV1::Ready,
        MailAccountReadinessV1::Retired => MailProviderPathReadinessV1::Retired,
        MailAccountReadinessV1::Deleted => MailProviderPathReadinessV1::Deleted,
        MailAccountReadinessV1::Degraded => MailProviderPathReadinessV1::Degraded,
    }
}

#[cfg(test)]
mod account_status_tests {
    use super::*;

    #[test]
    fn readiness_distinguishes_configuration_pending_active_and_stale_bindings() {
        let unconfigured = unconfigured_binding_status(MailCredentialPurposeV1::ImapPassword);
        assert_eq!(
            account_readiness(std::slice::from_ref(&unconfigured), 2),
            MailAccountReadinessV1::ConfigurationOnly
        );

        let pending = MailCredentialBindingStatusV1 {
            purpose: MailCredentialPurposeV1::ImapPassword,
            state: MailCredentialBindingStateV1::PendingRestart,
            binding_revision: Some(1),
            credential_revision: Some(2),
            applied_runtime_generation: None,
        };
        assert_eq!(
            account_readiness(std::slice::from_ref(&pending), 2),
            MailAccountReadinessV1::PendingRestart
        );

        let active = MailCredentialBindingStatusV1 {
            state: MailCredentialBindingStateV1::Active,
            applied_runtime_generation: Some(2),
            ..pending
        };
        assert_eq!(
            account_readiness(std::slice::from_ref(&active), 2),
            MailAccountReadinessV1::Ready
        );
        assert_eq!(
            account_readiness(std::slice::from_ref(&active), 3),
            MailAccountReadinessV1::Degraded
        );
    }
}

async fn activate_bound_account_credential(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    provider_context: &ManagedProviderCredentialContextV1,
    durable: &MailDurablePersistence,
    admission: &MailRuntimeAdmission,
    purpose: MailCredentialPurposeV1,
) -> Result<Option<Zeroizing<Vec<u8>>>, MailBootstrapError> {
    let Some(binding) = durable
        .account_credential_binding(&admission.account.connection_id, purpose)
        .await
        .map_err(|_| MailBootstrapError::Persistence)?
    else {
        return Ok(None);
    };
    if binding.configuration_instance_id != admission.configuration_instance_id {
        return Err(MailBootstrapError::Admission);
    }
    if matches!(
        binding.state,
        MailCredentialBindingStateV1::Retired | MailCredentialBindingStateV1::Deleted
    ) {
        return Ok(None);
    }
    let provider_purpose = match purpose {
        MailCredentialPurposeV1::ImapPassword => MailCredentialPurpose::ImapPassword,
        MailCredentialPurposeV1::SmtpPassword => MailCredentialPurpose::SmtpPassword,
        MailCredentialPurposeV1::IcloudCardDavPassword => {
            MailCredentialPurpose::IcloudCardDavPassword
        }
        MailCredentialPurposeV1::GmailAccessToken
        | MailCredentialPurposeV1::GmailRefreshCredential => {
            return Err(MailBootstrapError::Admission);
        }
    };
    let credential = {
        let mut provider_credentials = ManagedProviderCredentialClientV2::new(control_channel);
        let mut dispatcher = RejectManagedControlRequestsV2;
        match provider_credentials.resolve(
            &mut dispatcher,
            provider_context,
            ManagedProviderCredentialRequestV1 {
                configuration_instance_id: &admission.configuration_instance_id,
                purpose_id: provider_purpose.as_str(),
                credential_revision: binding.credential_revision,
                ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                secret_class: SecretClassV1::ProviderCredential,
            },
        ) {
            Ok(credential) => credential,
            Err(ManagedProviderCredentialErrorV1::InvalidContext) => {
                return Err(MailBootstrapError::Admission);
            }
            Err(
                ManagedProviderCredentialErrorV1::Rejected
                | ManagedProviderCredentialErrorV1::Unavailable,
            ) => return Ok(None),
        }
    };
    durable
        .mark_account_credential_active(
            &binding.connection_id,
            &binding.configuration_instance_id,
            binding.purpose,
            binding.binding_revision,
            binding.credential_revision,
            admission.runtime_generation,
            current_unix_seconds()?,
        )
        .await
        .map_err(|_| MailBootstrapError::Persistence)?;
    Ok(Some(credential))
}

fn provider_credential_context(
    admission: &MailRuntimeAdmission,
    configuration: &ManagedStorageRuntimeConfigurationV1,
) -> Result<ManagedProviderCredentialContextV1, MailBootstrapError> {
    let vault_public_key_x25519 = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| MailBootstrapError::Admission)?;
    if configuration.vault_runtime_generation != admission.vault_runtime_generation {
        return Err(MailBootstrapError::Admission);
    }
    Ok(ManagedProviderCredentialContextV1 {
        vault_instance_id: configuration.vault_instance_id.clone(),
        vault_runtime_generation: configuration.vault_runtime_generation,
        vault_public_key_x25519,
        logical_owner_id: admission.logical_owner_id.clone(),
        registration_id: admission.module_registration_id.clone(),
        runtime_instance_id: admission.runtime_instance_id.clone(),
        runtime_generation: admission.runtime_generation,
        grant_epoch: admission.grant_epoch,
    })
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &MailRuntimeAdmission,
) -> Result<StorageBindingV1, MailBootstrapError> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(MailBootstrapError::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.module_registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections).map_err(|_| MailBootstrapError::Storage)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailBootstrapError::Storage)?,
    )
    .map_err(|_| MailBootstrapError::Storage)?;
    StorageBindingV1::new(identity, fences, access).map_err(|_| MailBootstrapError::Storage)
}

fn current_unix_seconds() -> Result<i64, MailBootstrapError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MailBootstrapError::Provider)
        .and_then(|elapsed| {
            i64::try_from(elapsed.as_secs()).map_err(|_| MailBootstrapError::Provider)
        })
}

fn mail_event_hub_error(stage: &str) -> MailBootstrapError {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_event_hub_error={stage}");
    }
    MailBootstrapError::EventHub
}
