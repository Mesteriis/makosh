//! Real managed Vault and Storage binaries over disposable PostgreSQL/PgBouncer.

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use futures_util::StreamExt;
use makosh_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialRecipientPublicKeyV1, RuntimeNatsJwtCredentialV1, v1::DurableEnvelopeV1,
};
use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleEventDeliveryPolicyV1,
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
    ModuleStorageRequestV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
    PlatformStorageBundleV1, PlatformStorageEndpointV1, PlatformStorageTopology,
    StorageDeploymentProfileV1,
};
use makosh_runtime_protocol::v1::{
    ManagedDomainRuntimeConfigurationV1, ManagedRuntimeEventCredentialDeliveryV1,
    ManagedRuntimeEventCredentialRequestV1, ManagedWorkflowRuntimeConfigurationV1,
    SchedulerRuntimeControlRequestV1, SchedulerRuntimeControlResponseV1,
    SchedulerScheduleUpsertOutcomeV1, SettingsSchemaRefV1, SettingsSchemaV1,
    UpsertSchedulerScheduleRequestV1,
    scheduler_runtime_control_request_v1::Operation as SchedulerOperation,
    scheduler_runtime_control_response_v1::Result as SchedulerResult,
};
use makosh_scheduler_protocol::{
    SCHEDULER_JOB_DESCRIPTOR_SET_V1, SCHEDULER_RUNTIME_MODULE_ID_V1, v1::ScheduledJobCommandV1,
};
use makosh_storage_protocol::v1::{
    GetStorageRuntimeStatusRequestV1, StorageRuntimeControlRequestV1,
    StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
    storage_runtime_control_request_v1::Operation,
    storage_runtime_control_response_v1::Result as StorageResult,
};
use nats_jwt::KeyPair;
use prost::Message;

use super::common::*;
use crate::identity::device::signer::FileDeviceSigner;
use crate::platform::managed::signed_bundle::{
    InstalledSignedBundle, SignedNativeDependency, SignedRuntimeArtifact,
};
use crate::platform::vault::managed_route::KernelManagedVaultRouteHandler;
use crate::platform::vault::owner_derived_key::OwnerDerivedKeyHandlerV1;
use crate::platform::vault::provider_credential::ProviderCredentialHandlerV1;
use crate::platform::vault::status as vault_status;
use crate::platform::vault::{binding as vault_binding, launch as vault_launch};
use crate::platform::{
    blob::{
        binding as blob_binding, launch as blob_launch, release::BlobCustodyReleaseHandlerV1,
        session::BlobSessionHandlerV1,
    },
    events::{catalog as event_catalog, topology as event_topology},
    macos::managed_launch,
    scheduler::{launch as scheduler_launch, lifecycle as scheduler_lifecycle},
    storage::issuance::{StorageBindingIssueV1, issue_managed},
    storage::successor as storage_successor,
};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeEventCredentialHandler,
    ManagedRuntimeExpectation, ManagedRuntimeVaultRouteHandler,
};

#[path = "managed_storage_vault_docker/shared_fixture.rs"]
mod shared_fixture;
use shared_fixture::*;
#[path = "managed_storage_vault_docker/nats_outage_fixture.rs"]
mod nats_outage_fixture;
use nats_outage_fixture::*;
#[path = "managed_storage_vault_docker/owner_control_fixture.rs"]
mod owner_control_fixture;
use owner_control_fixture::*;
#[path = "managed_storage_vault_docker/scheduler_setup.rs"]
mod scheduler_setup;
use scheduler_setup::*;
#[path = "managed_storage_vault_docker/scheduler_events.rs"]
mod scheduler_events;
use scheduler_events::*;
#[path = "managed_storage_vault_docker/calendar_managed_setup.rs"]
mod calendar_managed_setup;
use calendar_managed_setup::*;
#[path = "managed_storage_vault_docker/calendar_managed_flow.rs"]
mod calendar_managed_flow;
#[path = "managed_storage_vault_docker/decisions_managed_setup.rs"]
mod decisions_managed_setup;
#[path = "managed_storage_vault_docker/documents_blob_fixture.rs"]
mod documents_blob_fixture;
#[path = "managed_storage_vault_docker/documents_managed_setup.rs"]
mod documents_managed_setup;
#[path = "managed_storage_vault_docker/organizations_managed_setup.rs"]
mod organizations_managed_setup;
#[path = "managed_storage_vault_docker/projects_managed_setup.rs"]
mod projects_managed_setup;
#[path = "managed_storage_vault_docker/relationships_managed_setup.rs"]
mod relationships_managed_setup;
use decisions_managed_setup::*;
use documents_blob_fixture::*;
use documents_managed_setup::*;
use organizations_managed_setup::*;
use projects_managed_setup::*;
use relationships_managed_setup::*;
#[path = "managed_storage_vault_docker/communications_setup.rs"]
mod communications_setup;
#[path = "managed_storage_vault_docker/decisions_managed_flow.rs"]
mod decisions_managed_flow;
#[path = "managed_storage_vault_docker/documents_managed_flow.rs"]
mod documents_managed_flow;
#[path = "managed_storage_vault_docker/obligation_candidate_blob_fixture.rs"]
mod obligation_candidate_blob_fixture;
#[path = "managed_storage_vault_docker/obligation_candidate_managed_flow.rs"]
mod obligation_candidate_managed_flow;
#[path = "managed_storage_vault_docker/obligation_candidate_managed_setup.rs"]
mod obligation_candidate_managed_setup;
#[path = "managed_storage_vault_docker/organizations_managed_flow.rs"]
mod organizations_managed_flow;
#[path = "managed_storage_vault_docker/projects_managed_flow.rs"]
mod projects_managed_flow;
#[path = "managed_storage_vault_docker/relationships_managed_flow.rs"]
mod relationships_managed_flow;
use communications_setup::*;
use obligation_candidate_blob_fixture::*;
use obligation_candidate_managed_setup::*;
#[path = "managed_storage_vault_docker/communications_ai_source_managed_flow.rs"]
mod communications_ai_source_managed_flow;
#[path = "managed_storage_vault_docker/reply_suggestion_managed_setup.rs"]
mod reply_suggestion_managed_setup;
use reply_suggestion_managed_setup::*;
#[path = "managed_storage_vault_docker/communication_summary_managed_setup.rs"]
mod communication_summary_managed_setup;
use communication_summary_managed_setup::*;
#[path = "managed_storage_vault_docker/communication_summary_managed_flow.rs"]
mod communication_summary_managed_flow;
#[path = "managed_storage_vault_docker/communication_translation_managed_setup.rs"]
mod communication_translation_managed_setup;
use communication_translation_managed_setup::*;
#[path = "managed_storage_vault_docker/communication_explanation_managed_setup.rs"]
mod communication_explanation_managed_setup;
#[path = "managed_storage_vault_docker/communication_translation_managed_flow.rs"]
mod communication_translation_managed_flow;
use communication_explanation_managed_setup::*;
#[path = "managed_storage_vault_docker/communication_explanation_managed_flow.rs"]
mod communication_explanation_managed_flow;
#[path = "managed_storage_vault_docker/communication_recipient_suggestion_managed_setup.rs"]
mod communication_recipient_suggestion_managed_setup;
use communication_recipient_suggestion_managed_setup::*;
#[path = "managed_storage_vault_docker/communication_recipient_suggestion_managed_flow.rs"]
mod communication_recipient_suggestion_managed_flow;
#[path = "managed_storage_vault_docker/communications_export_race.rs"]
mod communications_export_race;
#[path = "managed_storage_vault_docker/reply_suggestion_managed_flow.rs"]
mod reply_suggestion_managed_flow;
use communications_export_race::*;
#[path = "managed_storage_vault_docker/communications_backup.rs"]
mod communications_backup;
use communications_backup::*;
#[path = "managed_storage_vault_docker/delivery_intent_managed_setup.rs"]
mod delivery_intent_managed_setup;
use delivery_intent_managed_setup::*;
#[path = "managed_storage_vault_docker/cross_channel_forward_managed_setup.rs"]
mod cross_channel_forward_managed_setup;
use cross_channel_forward_managed_setup::*;
#[path = "managed_storage_vault_docker/delayed_delivery_managed_setup.rs"]
mod delayed_delivery_managed_setup;
use delayed_delivery_managed_setup::*;
#[path = "managed_storage_vault_docker/bulk_action_managed_setup.rs"]
mod bulk_action_managed_setup;
#[path = "managed_storage_vault_docker/cross_channel_forward_managed_flow.rs"]
mod cross_channel_forward_managed_flow;
#[path = "managed_storage_vault_docker/delayed_delivery_managed_flow.rs"]
mod delayed_delivery_managed_flow;
#[path = "managed_storage_vault_docker/delivery_intent_module_request_flow.rs"]
mod delivery_intent_module_request_flow;
#[path = "managed_storage_vault_docker/delivery_intent_realtime_flow.rs"]
mod delivery_intent_realtime_flow;
use bulk_action_managed_setup::*;
#[path = "managed_storage_vault_docker/bulk_action_managed_flow.rs"]
mod bulk_action_managed_flow;
#[path = "managed_storage_vault_docker/telegram_event_flow.rs"]
mod telegram_event_flow;
use telegram_event_flow::*;
#[path = "managed_storage_vault_docker/telegram_managed_setup.rs"]
mod telegram_managed_setup;
use telegram_managed_setup::*;
#[path = "managed_storage_vault_docker/telegram_owner_rls_conformance.rs"]
mod telegram_owner_rls_conformance;
use telegram_owner_rls_conformance::*;
#[path = "managed_storage_vault_docker/ai_inference_blob_fixture.rs"]
mod ai_inference_blob_fixture;
#[path = "managed_storage_vault_docker/ai_inference_managed_flow.rs"]
mod ai_inference_managed_flow;
#[path = "managed_storage_vault_docker/ai_owner_rls_conformance.rs"]
mod ai_owner_rls_conformance;
use ai_owner_rls_conformance::*;
#[path = "managed_storage_vault_docker/ai_inference_managed_setup.rs"]
mod ai_inference_managed_setup;
#[path = "managed_storage_vault_docker/archive_inspection_managed_setup.rs"]
mod archive_inspection_managed_setup;
#[path = "managed_storage_vault_docker/attachment_preview_gateway_fixture.rs"]
mod attachment_preview_gateway_fixture;
#[path = "managed_storage_vault_docker/attachment_preview_managed_flow.rs"]
mod attachment_preview_managed_flow;
#[path = "managed_storage_vault_docker/attachment_preview_managed_formats.rs"]
mod attachment_preview_managed_formats;
#[path = "managed_storage_vault_docker/attachment_preview_persistence_fixture.rs"]
mod attachment_preview_persistence_fixture;
use attachment_preview_persistence_fixture::*;
#[path = "managed_storage_vault_docker/attachment_preview_evidence_replay_managed_flow.rs"]
mod attachment_preview_evidence_replay_managed_flow;
#[path = "managed_storage_vault_docker/attachment_preview_evidence_replay_managed_setup.rs"]
mod attachment_preview_evidence_replay_managed_setup;
#[path = "managed_storage_vault_docker/attachment_preview_evidence_replay_persistence_fixture.rs"]
mod attachment_preview_evidence_replay_persistence_fixture;
#[path = "managed_storage_vault_docker/attachment_preview_managed_setup.rs"]
mod attachment_preview_managed_setup;
#[path = "managed_storage_vault_docker/attachment_security_blob_fixture.rs"]
mod attachment_security_blob_fixture;
#[path = "managed_storage_vault_docker/attachment_security_clamav_fixture.rs"]
mod attachment_security_clamav_fixture;
#[path = "managed_storage_vault_docker/attachment_security_event_flow.rs"]
mod attachment_security_event_flow;
#[path = "managed_storage_vault_docker/attachment_security_managed_flow.rs"]
mod attachment_security_managed_flow;
#[path = "managed_storage_vault_docker/attachment_security_managed_setup.rs"]
mod attachment_security_managed_setup;
#[path = "managed_storage_vault_docker/attachment_text_extraction_gateway_fixture.rs"]
mod attachment_text_extraction_gateway_fixture;
#[path = "managed_storage_vault_docker/attachment_text_extraction_managed_flow.rs"]
mod attachment_text_extraction_managed_flow;
#[path = "managed_storage_vault_docker/attachment_text_extraction_managed_setup.rs"]
mod attachment_text_extraction_managed_setup;
#[path = "managed_storage_vault_docker/attachment_text_extraction_persistence_fixture.rs"]
mod attachment_text_extraction_persistence_fixture;
#[path = "managed_storage_vault_docker/attachment_text_extraction_source_fixtures.rs"]
mod attachment_text_extraction_source_fixtures;
#[path = "managed_storage_vault_docker/attachment_translation_gateway_fixture.rs"]
mod attachment_translation_gateway_fixture;
#[path = "managed_storage_vault_docker/attachment_translation_managed_flow.rs"]
mod attachment_translation_managed_flow;
#[path = "managed_storage_vault_docker/attachment_translation_managed_setup.rs"]
mod attachment_translation_managed_setup;
use archive_inspection_managed_setup::*;
use attachment_preview_evidence_replay_managed_setup::*;
use attachment_preview_managed_setup::*;
use attachment_text_extraction_managed_setup::*;
use attachment_translation_gateway_fixture::*;
use attachment_translation_managed_setup::*;
#[path = "managed_storage_vault_docker/archive_inspection_managed_flow.rs"]
mod archive_inspection_managed_flow;
#[path = "managed_storage_vault_docker/attachment_security_persistence_fixture.rs"]
mod attachment_security_persistence_fixture;
#[path = "managed_storage_vault_docker/persons_managed_flow.rs"]
mod persons_managed_flow;
#[path = "managed_storage_vault_docker/persons_managed_setup.rs"]
mod persons_managed_setup;
use persons_managed_setup::*;
#[path = "managed_storage_vault_docker/identity_resolution_managed_setup.rs"]
mod identity_resolution_managed_setup;
use identity_resolution_managed_setup::*;
#[path = "managed_storage_vault_docker/projection_managed_setup.rs"]
mod projection_managed_setup;
use projection_managed_setup::*;
#[path = "managed_storage_vault_docker/projection_managed_flow.rs"]
mod projection_managed_flow;
#[path = "managed_storage_vault_docker/review_person_match_candidate_managed_setup.rs"]
mod review_person_match_candidate_managed_setup;
use review_person_match_candidate_managed_setup::*;
#[path = "managed_storage_vault_docker/review_person_match_candidate_managed_flow.rs"]
mod review_person_match_candidate_managed_flow;
#[path = "managed_storage_vault_docker/reviewed_person_match_candidate_promotion_managed_setup.rs"]
mod reviewed_person_match_candidate_promotion_managed_setup;
use reviewed_person_match_candidate_promotion_managed_setup::*;
#[path = "managed_storage_vault_docker/mail_persons_sync_managed_setup.rs"]
mod mail_persons_sync_managed_setup;
use mail_persons_sync_managed_setup::*;
#[path = "managed_storage_vault_docker/mail_attachment_flow.rs"]
mod mail_attachment_flow;
#[path = "managed_storage_vault_docker/mail_carddav_fixture.rs"]
mod mail_carddav_fixture;
#[path = "managed_storage_vault_docker/mail_persons_sync_managed_flow.rs"]
mod mail_persons_sync_managed_flow;
#[path = "managed_storage_vault_docker/persons_admission_cutover_flow.rs"]
mod persons_admission_cutover_flow;
use mail_carddav_fixture::*;
#[path = "managed_storage_vault_docker/call_transcription_gateway_fixture.rs"]
mod call_transcription_gateway_fixture;
#[path = "managed_storage_vault_docker/call_transcription_managed_flow.rs"]
mod call_transcription_managed_flow;
#[path = "managed_storage_vault_docker/call_transcription_managed_setup.rs"]
mod call_transcription_managed_setup;
#[path = "managed_storage_vault_docker/desktop_call_recording_blob_target_fixture.rs"]
mod desktop_call_recording_blob_target_fixture;
#[path = "managed_storage_vault_docker/desktop_call_recording_host_fixture.rs"]
mod desktop_call_recording_host_fixture;
#[path = "managed_storage_vault_docker/desktop_call_recording_managed_flow.rs"]
mod desktop_call_recording_managed_flow;
#[path = "managed_storage_vault_docker/desktop_call_recording_managed_setup.rs"]
mod desktop_call_recording_managed_setup;
#[path = "managed_storage_vault_docker/mail_composition_flow.rs"]
mod mail_composition_flow;
#[path = "managed_storage_vault_docker/mail_delivery_test_support.rs"]
mod mail_delivery_test_support;
#[path = "managed_storage_vault_docker/mail_event_flow.rs"]
mod mail_event_flow;
#[path = "managed_storage_vault_docker/mail_gmail_fixture.rs"]
mod mail_gmail_fixture;
#[path = "managed_storage_vault_docker/mail_gmail_oauth_fixture.rs"]
mod mail_gmail_oauth_fixture;
#[path = "managed_storage_vault_docker/mail_imap_fixture.rs"]
mod mail_imap_fixture;
#[path = "managed_storage_vault_docker/mail_managed_setup.rs"]
mod mail_managed_setup;
#[path = "managed_storage_vault_docker/mail_operational_flow.rs"]
mod mail_operational_flow;
#[path = "managed_storage_vault_docker/mail_smtp_fixture.rs"]
mod mail_smtp_fixture;
#[path = "managed_storage_vault_docker/mail_sync_health_flow.rs"]
mod mail_sync_health_flow;
#[path = "managed_storage_vault_docker/note_candidate_blob_negative.rs"]
mod note_candidate_blob_negative;
#[path = "managed_storage_vault_docker/note_candidate_gateway_flow.rs"]
mod note_candidate_gateway_flow;
#[path = "managed_storage_vault_docker/note_candidate_managed_flow.rs"]
mod note_candidate_managed_flow;
#[path = "managed_storage_vault_docker/note_candidate_managed_setup.rs"]
mod note_candidate_managed_setup;
#[path = "managed_storage_vault_docker/note_candidate_persistence_flow.rs"]
mod note_candidate_persistence_flow;
#[path = "managed_storage_vault_docker/ollama_ai_managed_flow.rs"]
mod ollama_ai_managed_flow;
#[path = "managed_storage_vault_docker/ollama_ai_managed_setup.rs"]
mod ollama_ai_managed_setup;
#[path = "managed_storage_vault_docker/review_attention_managed_flow.rs"]
mod review_attention_managed_flow;
#[path = "managed_storage_vault_docker/review_owner_rls_conformance.rs"]
mod review_owner_rls_conformance;
use review_owner_rls_conformance::*;
#[path = "managed_storage_vault_docker/review_attention_managed_setup.rs"]
mod review_attention_managed_setup;
#[path = "managed_storage_vault_docker/speech_to_text_managed_setup.rs"]
mod speech_to_text_managed_setup;
#[path = "managed_storage_vault_docker/speech_to_text_owner_rls_conformance.rs"]
mod speech_to_text_owner_rls_conformance;
#[path = "managed_storage_vault_docker/task_candidate_blob_negative.rs"]
mod task_candidate_blob_negative;
#[path = "managed_storage_vault_docker/task_candidate_gateway_flow.rs"]
mod task_candidate_gateway_flow;
#[path = "managed_storage_vault_docker/task_candidate_managed_flow.rs"]
mod task_candidate_managed_flow;
#[path = "managed_storage_vault_docker/task_candidate_managed_setup.rs"]
mod task_candidate_managed_setup;
#[path = "managed_storage_vault_docker/task_candidate_persistence_flow.rs"]
mod task_candidate_persistence_flow;
#[path = "managed_storage_vault_docker/whisper_stt_blob_fixture.rs"]
mod whisper_stt_blob_fixture;
#[path = "managed_storage_vault_docker/whisper_stt_managed_flow.rs"]
mod whisper_stt_managed_flow;
#[path = "managed_storage_vault_docker/whisper_stt_managed_setup.rs"]
mod whisper_stt_managed_setup;
use ai_inference_blob_fixture::*;
use ai_inference_managed_setup::*;
use call_transcription_gateway_fixture::*;
use call_transcription_managed_setup::*;
use desktop_call_recording_blob_target_fixture::*;
use desktop_call_recording_host_fixture::*;
use desktop_call_recording_managed_setup::*;
use mail_attachment_flow::*;
use mail_delivery_test_support::*;
use mail_event_flow::*;
use mail_gmail_fixture::*;
use mail_gmail_oauth_fixture::*;
use mail_imap_fixture::*;
use mail_smtp_fixture::*;
use note_candidate_blob_negative::*;
use note_candidate_gateway_flow::*;
use note_candidate_managed_setup::*;
use note_candidate_persistence_flow::*;
use ollama_ai_managed_setup::*;
use review_attention_managed_setup::*;
use speech_to_text_managed_setup::*;
use speech_to_text_owner_rls_conformance::*;
use task_candidate_blob_negative::*;
use task_candidate_gateway_flow::*;
use task_candidate_managed_setup::*;
use task_candidate_persistence_flow::*;
use whisper_stt_blob_fixture::*;
use whisper_stt_managed_setup::*;
#[path = "managed_storage_vault_docker/call_evidence_managed_flow.rs"]
mod call_evidence_managed_flow;
#[path = "managed_storage_vault_docker/telegram_managed_flow.rs"]
mod telegram_managed_flow;
use attachment_security_blob_fixture::*;
use attachment_security_event_flow::*;
use attachment_security_managed_setup::*;
use mail_managed_setup::*;
use mail_operational_flow::*;
use mail_sync_health_flow::*;
#[path = "managed_storage_vault_docker/mail_account_credential_flow.rs"]
mod mail_account_credential_flow;
#[path = "managed_storage_vault_docker/mail_delivery_flow.rs"]
mod mail_delivery_flow;
#[path = "managed_storage_vault_docker/mail_gmail_delivery_flow.rs"]
mod mail_gmail_delivery_flow;
#[path = "managed_storage_vault_docker/mail_gmail_oauth_flow.rs"]
mod mail_gmail_oauth_flow;
#[path = "managed_storage_vault_docker/mail_managed_flow.rs"]
mod mail_managed_flow;
#[path = "managed_storage_vault_docker/mail_message_flag_flow.rs"]
mod mail_message_flag_flow;
#[path = "managed_storage_vault_docker/mail_message_location_flow.rs"]
mod mail_message_location_flow;
#[path = "managed_storage_vault_docker/mail_message_permanent_delete_flow.rs"]
mod mail_message_permanent_delete_flow;
#[path = "managed_storage_vault_docker/mail_outbound_attachment_flow.rs"]
mod mail_outbound_attachment_flow;
#[path = "managed_storage_vault_docker/zulip_https_fixture.rs"]
mod zulip_https_fixture;
use zulip_https_fixture::*;
#[path = "managed_storage_vault_docker/zulip_managed_setup.rs"]
mod zulip_managed_setup;
use zulip_managed_setup::*;
#[path = "managed_storage_vault_docker/zulip_managed_fixture.rs"]
mod zulip_managed_fixture;
use zulip_managed_fixture::*;
#[path = "managed_storage_vault_docker/whatsapp_managed_setup.rs"]
mod whatsapp_managed_setup;
#[path = "managed_storage_vault_docker/zulip_event_flow.rs"]
mod zulip_event_flow;
#[path = "managed_storage_vault_docker/zulip_managed_flow.rs"]
mod zulip_managed_flow;
use whatsapp_managed_setup::*;
#[path = "managed_storage_vault_docker/whatsapp_managed_fixture.rs"]
mod whatsapp_managed_fixture;
use whatsapp_managed_fixture::*;
#[path = "managed_storage_vault_docker/whatsapp_host_fixture.rs"]
mod whatsapp_host_fixture;
use whatsapp_host_fixture::*;
#[path = "managed_storage_vault_docker/whatsapp_event_flow.rs"]
mod whatsapp_event_flow;
#[path = "managed_storage_vault_docker/whatsapp_managed_flow.rs"]
mod whatsapp_managed_flow;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault and Storage binaries"]
fn managed_storage_binary_bootstraps_through_live_vault() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-storage-vault-docker");
    let data = private_directory(root.join("kernel"));
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let release = installed_release(&root);
    let store = Arc::new(configured_store(&root, release.kernel()));
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    assert_eq!(
        start_vault(&supervisor, &store, &data, release.kernel()),
        1,
        "Vault starts from the signed release binding"
    );
    let vault =
        vault_status::read_current(&store, &supervisor.relay_port()).expect("live Vault status");
    assert_eq!(vault.runtime_generation(), 1);
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory()
        ),
        1,
        "Storage starts from the signed release binding"
    );
    assert_reconciling_status(&supervisor, 1);
    supervisor.stop("storage").expect("stop Storage");
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory()
        ),
        2,
        "restarted Storage re-verifies the signed release binding"
    );
    assert_reconciling_status(&supervisor, 2);
    supervisor.shutdown().expect("stop managed processes");
    std::fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Scheduler and NATS binaries"]
fn managed_scheduler_crash_uses_storage_control_successor_provisioning() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let fixture = SchedulerRecoveryFixture::start();
    let binding = fixture.start_initial_scheduler();
    let due_at = fixture.persist_recovery_schedule();
    let worker = fixture.restart_after_crash(due_at);
    let successor = fixture.assert_successor(&binding, due_at);
    fixture.assert_revoked_binding_does_not_restart(successor);
    fixture.shutdown(worker);
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Communications binaries"]
fn managed_communications_domain_starts_with_owner_local_storage_and_events() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-communications-domain");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::browser_gateway_session::admit_browser_test_device(&store, "owner-1");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Communications Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime")
        )
        .expect("start signed Blob runtime"),
        1,
        "Blob starts as a separate managed platform process"
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    configure_communications_jetstream(&store);

    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1,
        "generic managed-domain launch admits Communications without a Kernel owner facade"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("read Communications process state")
    );
    assert_communications_ingress_delivery(&store, &supervisor);
    assert_communications_relationship_projection(&store, &supervisor);
    assert_communications_attachment_anchor_projection(&store, &supervisor);
    let _ = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        true,
    );
    assert_communications_query_delivery(&store, &supervisor);
    assert_communications_module_query_delivery(&supervisor);
    assert_communications_canonical_read_v2_pagination(&store, &supervisor);
    assert_communications_search_query_delivery(&store, &supervisor);
    assert_communications_gateway_query_delivery(&store, &supervisor, &root, &data);
    assert_telegram_outbox_delivery(&store, &supervisor);
    assert_fenced_communications_target_cannot_issue_blob_custody_grant(&store, &supervisor, &data);

    supervisor.shutdown().expect("stop managed processes");
    assert_communications_storage_backup_restore(&root);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Blob and Communications Export workflow binaries"]
fn managed_communications_export_workflow_starts_with_owner_local_storage_and_events() {
    use makosh_communications_evidence_export_source_api::wire::EvidenceExportRejectCodeV1;
    use makosh_communications_export_api::{
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1, COMMUNICATIONS_EXPORT_MODULE_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        wire::{
            CommunicationsExportErrorCodeV1, EvidenceExportArtifactReadRequestV1,
            EvidenceExportStatusV1, GetEvidenceExportStatusRequestV1,
            GetEvidenceExportStatusResponseV1, IssueEvidenceExportReadRequestV1,
            IssueEvidenceExportReadResponseV1, StartEvidenceExportRequestV1,
            StartEvidenceExportResponseV1,
        },
    };
    use makosh_communications_export_runtime::admission::{
        communications_export_command_contract_reference_v1,
        communications_export_query_contract_reference_v1,
        communications_export_ticket_contract_reference_v1,
    };
    use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-communications-export");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::browser_gateway_session::admit_browser_test_device(&store, "owner-1");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let export_realtime = makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64)
        .expect("Communications Export realtime source");
    let revision_race = Arc::new(CommunicationsExportRevisionRaceV1::new());
    let race_blob_session_handler = Arc::new(CommunicationsExportRaceBlobSessionHandlerV1::new(
        Arc::clone(&store),
        supervisor.relay_port(),
        data.clone(),
        Arc::clone(&revision_race),
    ));
    configure_route_handlers(&supervisor, &store, &data, race_blob_session_handler);
    supervisor
        .configure_client_realtime_handler(Arc::new(
            crate::platform::client_realtime::ClientRealtimePublishHandlerV1::new(
                Arc::clone(&store),
                export_realtime.clone(),
            ),
        ))
        .expect("configure Communications Export client realtime");
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Export Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    blob_launch::start_from_kernel(
        &supervisor,
        &store,
        release.kernel(),
        &data,
        &root.join("runtime"),
    )
    .expect("start signed Blob runtime");
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    configure_communications_jetstream(&store);
    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1,
        "Communications source owner starts independently before its export workflow"
    );
    let message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    issue_initial_communications_export_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_export_storage_binding(&store),
    )
    .expect("provision Communications Export Storage binding after the source-owner recovery");
    assert_eq!(
        start_communications_export_workflow(&supervisor, &store, &root.join("runtime")),
        1,
        "generic managed-workflow launch admits Communications Export without a Kernel owner facade"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_EXPORT_REGISTRATION)
            .expect("read Communications Export process state")
    );
    let gateway_configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("export-sse-gateway-cert.der"),
        root.join("export-sse-gateway-key.der"),
    )
    .expect("Communications Export Gateway configuration");
    let gateway = crate::platform::gateway::gateway_service(
        Arc::clone(&store),
        &data,
        supervisor.clone(),
        export_realtime,
        &gateway_configuration,
        None,
    )
    .expect("compose Communications Export Gateway SSE route");
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway SSE runtime");
    let gateway_cookie =
        super::browser_gateway_session::authenticate_gateway_router(&gateway, &gateway_runtime);
    let realtime_response = gateway_runtime.block_on(
        gateway.route(
            hyper::Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &gateway_cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::new()))
                .expect("pre-open Communications Export SSE request"),
        ),
    );
    assert_eq!(realtime_response.status(), hyper::StatusCode::OK);
    let mut realtime_body = realtime_response.into_body();
    let route_as = |request_id: u64,
                    logical_owner_id: &str,
                    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
                    request_payload: Vec<u8>| {
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: COMMUNICATIONS_EXPORT_MODULE_ID_V1.to_owned(),
            owner_id: COMMUNICATIONS_EXPORT_OWNER_V1.to_owned(),
            contract: Some(contract),
            request_id,
            request_payload,
            logical_owner_id: logical_owner_id.to_owned(),
            authenticated_device_id: "desktop-1".to_owned(),
            authenticated_client_session_id: "session-1".to_owned(),
        }
        .encode_to_vec();
        let launch = store
            .effective_managed_launch_record(COMMUNICATIONS_EXPORT_REGISTRATION)
            .expect("read Communications Export launch")
            .expect("Communications Export launch is active");
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            launch.runtime_instance_id(),
            launch.runtime_generation(),
            launch.grant_epoch(),
            COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1,
            &request,
        );
        let bytes = crate::modules::capability::router::route_managed_client_request(
            store.as_ref(),
            &supervisor.relay_port(),
            &route,
        )
        .expect("route exact Communications Export client request");
        let response = ModuleClientResponseV1::decode(bytes.as_slice())
            .expect("decode Communications Export module response");
        assert_eq!(response.request_id, request_id);
        assert!(
            response.error_code.is_empty(),
            "Communications Export request {request_id} failed: {}",
            response.error_code,
        );
        response.response_payload
    };
    let route = |request_id: u64,
                 contract: makosh_runtime_protocol::v1::ContractReferenceV1,
                 request_payload: Vec<u8>| {
        route_as(request_id, "owner-1", contract, request_payload)
    };
    let export_id = [11; 16];
    let start = StartEvidenceExportResponseV1::decode(
        route(
            1,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted Communications Export command");
    assert_eq!(start.export_id, export_id);
    let terminal_event = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &export_id,
    );
    assert_eq!(
        terminal_event.status,
        EvidenceExportStatusV1::EvidenceExportStatusReady as i32
    );
    assert_eq!(terminal_event.requested_items, 1);
    assert_eq!(terminal_event.completed_items, 1);
    assert!(terminal_event.artifact_bytes > 0);
    let status = GetEvidenceExportStatusResponseV1::decode(
        route(
            2,
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode terminal Communications Export status snapshot");
    assert_eq!(
        status.status,
        EvidenceExportStatusV1::EvidenceExportStatusReady as i32
    );
    assert_eq!(status.requested_items, 1);
    assert_eq!(status.completed_items, 1);
    assert!(status.artifact_bytes > 0);
    let wrong_owner_status = GetEvidenceExportStatusResponseV1::decode(
        route_as(
            20,
            "owner-2",
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode wrong-owner Communications Export status");
    assert_eq!(
        wrong_owner_status.status,
        EvidenceExportStatusV1::EvidenceExportStatusUnspecified as i32
    );
    assert_eq!(wrong_owner_status.artifact_bytes, 0);
    assert_eq!(
        wrong_owner_status.error,
        CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotFound as i32
    );
    let wrong_owner_ticket = IssueEvidenceExportReadResponseV1::decode(
        route_as(
            21,
            "owner-2",
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode wrong-owner Communications Export ticket response");
    assert!(wrong_owner_ticket.opaque_read_capability.is_empty());
    assert_eq!(
        wrong_owner_ticket.error,
        CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotFound as i32
    );
    let edited_body = publish_and_wait_for_communications_message_edit(
        &store,
        &supervisor,
        &data,
        &message_id,
        b"fixture edited source body for custody transfer".to_vec(),
        1_783_024_009,
        10,
    );
    let edited_export_id = [15; 16];
    let edited_start = StartEvidenceExportResponseV1::decode(
        route(
            12,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: edited_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted edited-message export command");
    assert_eq!(edited_start.export_id, edited_export_id);
    let edited_terminal = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &edited_export_id,
    );
    assert_eq!(
        edited_terminal.status,
        EvidenceExportStatusV1::EvidenceExportStatusReady as i32
    );
    assert_eq!(edited_terminal.requested_items, 1);
    assert_eq!(edited_terminal.completed_items, 1);
    assert!(edited_terminal.artifact_bytes > 0);
    let stale_runtime_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            15,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: edited_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("issue edited export ticket before workflow restart");
    assert_eq!(stale_runtime_ticket.opaque_read_capability.len(), 32);
    assert_eq!(
        restart_communications_export_workflow(&supervisor, &store, &root.join("runtime")),
        2,
        "Communications Export restart advances its independent runtime generation"
    );
    let restarted_status = GetEvidenceExportStatusResponseV1::decode(
        route(
            16,
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: edited_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode Communications Export status after successor restart");
    assert_eq!(
        restarted_status.status,
        EvidenceExportStatusV1::EvidenceExportStatusReady as i32
    );
    assert_eq!(restarted_status.requested_items, 1);
    assert_eq!(restarted_status.completed_items, 1);
    assert!(restarted_status.artifact_bytes > 0);
    set_authenticated_nats_container_running(false);
    let outage_export_id = [16; 16];
    let outage_start = StartEvidenceExportResponseV1::decode(
        route(
            17,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: outage_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode Communications Export command accepted during NATS outage");
    assert_eq!(outage_start.export_id, outage_export_id);
    std::thread::sleep(std::time::Duration::from_millis(1_500));
    let outage_pending = GetEvidenceExportStatusResponseV1::decode(
        route(
            18,
            communications_export_query_contract_reference_v1(),
            GetEvidenceExportStatusRequestV1 {
                protocol_major: 1,
                export_id: outage_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode Communications Export status during NATS outage");
    let export_runtime_active_during_outage = supervisor
        .is_active(COMMUNICATIONS_EXPORT_REGISTRATION)
        .expect("read Communications Export process state during NATS outage");
    set_authenticated_nats_container_running(true);
    assert_eq!(
        outage_pending.status,
        EvidenceExportStatusV1::EvidenceExportStatusPendingSource as i32,
        "NATS outage retains the exact export request before source preparation"
    );
    assert!(
        export_runtime_active_during_outage,
        "NATS outage is retryable and does not stop Communications Export"
    );
    let outage_terminal = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &outage_export_id,
    );
    assert_eq!(
        outage_terminal.status,
        EvidenceExportStatusV1::EvidenceExportStatusReady as i32
    );
    assert_eq!(outage_terminal.requested_items, 1);
    assert_eq!(outage_terminal.completed_items, 1);
    assert!(outage_terminal.artifact_bytes > 0);
    let stale_revision_export_id = [18; 16];
    let communications_database_id = crate::platform::storage::topology::current(&store)
        .expect("read Communications Storage topology")
        .database_id()
        .to_owned();
    revision_race.arm(&communications_database_id, &message_id);
    let stale_revision_start = StartEvidenceExportResponseV1::decode(
        route(
            24,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: stale_revision_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted stale-revision export command");
    assert_eq!(stale_revision_start.export_id, stale_revision_export_id);
    let stale_revision_terminal = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &stale_revision_export_id,
    );
    assert_eq!(
        stale_revision_terminal.status,
        EvidenceExportStatusV1::EvidenceExportStatusRejected as i32
    );
    assert_eq!(stale_revision_terminal.completed_items, 0);
    assert_eq!(stale_revision_terminal.artifact_bytes, 0);
    assert_eq!(
        stale_revision_terminal.error,
        CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodePolicyRejected as i32,
    );
    assert!(
        revision_race.fired_revision() > 1,
        "managed source preparation must cross the injected canonical revision fence"
    );
    assert_eq!(
        communications_export_rejection_code(
            &communications_database_id,
            &stale_revision_export_id,
        ),
        EvidenceExportRejectCodeV1::EvidenceExportRejectCodeStaleRevision as u16,
        "workflow terminal state must preserve the typed STALE_REVISION source result",
    );
    let invalid_utf8_body = vec![0xf0, 0x28, 0x8c, 0x28];
    assert_eq!(
        publish_and_wait_for_communications_message_edit(
            &store,
            &supervisor,
            &data,
            &message_id,
            invalid_utf8_body.clone(),
            1_783_024_010,
            11,
        ),
        invalid_utf8_body,
    );
    let invalid_utf8_export_id = [17; 16];
    let invalid_utf8_start = StartEvidenceExportResponseV1::decode(
        route(
            22,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: invalid_utf8_export_id.to_vec(),
                message_ids: vec![message_id.clone()],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted invalid-UTF8 export command");
    assert_eq!(invalid_utf8_start.export_id, invalid_utf8_export_id);
    let invalid_utf8_terminal = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &invalid_utf8_export_id,
    );
    assert_eq!(
        invalid_utf8_terminal.status,
        EvidenceExportStatusV1::EvidenceExportStatusRejected as i32
    );
    assert_eq!(invalid_utf8_terminal.completed_items, 0);
    assert_eq!(invalid_utf8_terminal.artifact_bytes, 0);
    publish_and_wait_for_communications_message_deletion(store.as_ref(), &supervisor, &message_id);
    let deleted_export_id = [13; 16];
    let deleted_start = StartEvidenceExportResponseV1::decode(
        route(
            8,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: deleted_export_id.to_vec(),
                message_ids: vec![message_id],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted deleted-message export command");
    assert_eq!(deleted_start.export_id, deleted_export_id);
    let deleted_terminal = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &deleted_export_id,
    );
    assert_eq!(
        deleted_terminal.status,
        EvidenceExportStatusV1::EvidenceExportStatusRejected as i32
    );
    assert_eq!(deleted_terminal.completed_items, 0);
    assert_eq!(deleted_terminal.artifact_bytes, 0);
    let rejected_export_id = [12; 16];
    let rejected_start = StartEvidenceExportResponseV1::decode(
        route(
            4,
            communications_export_command_contract_reference_v1(),
            StartEvidenceExportRequestV1 {
                protocol_major: 1,
                operation_id: rejected_export_id.to_vec(),
                message_ids: vec![vec![99; 16]],
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode accepted unknown-message export command");
    assert_eq!(rejected_start.export_id, rejected_export_id);
    let rejected_terminal = await_terminal_communications_export_event(
        &gateway_runtime,
        &mut realtime_body,
        &rejected_export_id,
    );
    assert_eq!(
        rejected_terminal.status,
        EvidenceExportStatusV1::EvidenceExportStatusRejected as i32
    );
    assert_eq!(rejected_terminal.completed_items, 0);
    assert_eq!(rejected_terminal.artifact_bytes, 0);
    let ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            3,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode one-use Communications Export read ticket");
    assert_eq!(ticket.opaque_read_capability.len(), 32);
    assert!(ticket.declared_bytes > 0);
    let edited_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            14,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: edited_export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("decode edited-message Communications Export read ticket");
    assert_eq!(edited_ticket.opaque_read_capability.len(), 32);
    assert!(edited_ticket.declared_bytes > 0);
    let blob_outage_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            6,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("issue read ticket before Blob outage");
    let export_gateway_fixture = CommunicationsExportGatewayFixtureV1 {
        router: &gateway,
        runtime: &gateway_runtime,
        cookie: &gateway_cookie,
        store: &store,
        supervisor: &supervisor,
        root: &root,
        kernel_data: &data,
        kernel_executable: release.kernel(),
    };
    assert_communications_export_gateway_delivery(
        &export_gateway_fixture,
        CommunicationsExportGatewayDeliveryInputsV1 {
            opaque_read_capability: ticket.opaque_read_capability,
            declared_bytes: ticket.declared_bytes,
            edited_body: &edited_body,
            edited_opaque_read_capability: edited_ticket.opaque_read_capability,
            edited_declared_bytes: edited_ticket.declared_bytes,
            stale_runtime_read_capability: stale_runtime_ticket.opaque_read_capability,
            blob_outage_read_capability: blob_outage_ticket.opaque_read_capability,
        },
    );
    let revoked_ticket = IssueEvidenceExportReadResponseV1::decode(
        route(
            7,
            communications_export_ticket_contract_reference_v1(),
            IssueEvidenceExportReadRequestV1 {
                protocol_major: 1,
                export_id: export_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("issue read ticket before export workflow revoke");
    store
        .transition_module_registration(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            ModuleRegistrationState::Revoked,
        )
        .expect("revoke Communications Export workflow registration");
    assert_communications_export_gateway_rejects_revoked_ticket(
        &export_gateway_fixture,
        revoked_ticket.opaque_read_capability,
    );
    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn await_terminal_communications_export_event<B>(
    runtime: &tokio::runtime::Runtime,
    body: &mut B,
    export_id: &[u8],
) -> makosh_communications_export_api::wire::EvidenceExportStatusChangedV1
where
    B: hyper::body::Body<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    runtime.block_on(async {
        tokio::time::timeout(
            std::time::Duration::from_secs(15),
            find_terminal_communications_export_event(body, export_id),
        )
        .await
        .expect("Communications Export SSE terminal timeout")
    })
}

async fn find_terminal_communications_export_event<B>(
    body: &mut B,
    export_id: &[u8],
) -> makosh_communications_export_api::wire::EvidenceExportStatusChangedV1
where
    B: hyper::body::Body<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use http_body_util::BodyExt as _;
    use makosh_communications_export_api::{
        COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1,
        COMMUNICATIONS_EXPORT_REALTIME_EVENT_KIND_V1,
        wire::{EvidenceExportStatusChangedV1, EvidenceExportStatusV1},
    };
    use makosh_gateway_protocol::v1::{
        ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
    };

    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Communications Export SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Communications Export SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Communications Export realtime frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Communications Export realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != COMMUNICATIONS_EXPORT_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = EvidenceExportStatusChangedV1::decode(event.payload.as_slice())
                .expect("Communications Export realtime payload");
            if payload.export_id == export_id
                && matches!(
                    EvidenceExportStatusV1::try_from(payload.status),
                    Ok(EvidenceExportStatusV1::EvidenceExportStatusReady)
                        | Ok(EvidenceExportStatusV1::EvidenceExportStatusRejected)
                )
            {
                return payload;
            }
        }
    }
    panic!("Gateway SSE closed before terminal Communications Export event");
}

struct CommunicationsExportGatewayDeliveryInputsV1<'a> {
    opaque_read_capability: Vec<u8>,
    declared_bytes: u64,
    edited_body: &'a [u8],
    edited_opaque_read_capability: Vec<u8>,
    edited_declared_bytes: u64,
    stale_runtime_read_capability: Vec<u8>,
    blob_outage_read_capability: Vec<u8>,
}

struct CommunicationsExportGatewayFixtureV1<'a> {
    router: &'a crate::platform::gateway::BrowserGatewayRouter,
    runtime: &'a tokio::runtime::Runtime,
    cookie: &'a str,
    store: &'a Arc<SqliteControlStore>,
    supervisor: &'a ManagedRuntimeSupervisor,
    root: &'a std::path::Path,
    kernel_data: &'a std::path::Path,
    kernel_executable: &'a std::path::Path,
}

fn assert_communications_export_gateway_delivery(
    fixture: &CommunicationsExportGatewayFixtureV1<'_>,
    inputs: CommunicationsExportGatewayDeliveryInputsV1<'_>,
) {
    use http_body_util::BodyExt as _;
    use makosh_communications_export_api::{
        COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1, wire::EvidenceExportArtifactReadRequestV1,
    };
    let router = fixture.router;
    let runtime = fixture.runtime;
    let cookie = fixture.cookie;
    let store = fixture.store;
    let supervisor = fixture.supervisor;
    let root = fixture.root;
    let kernel_data = fixture.kernel_data;
    let kernel_executable = fixture.kernel_executable;

    let stale_runtime_read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.stale_runtime_read_capability,
    }
    .encode_to_vec();
    let stale_runtime_response = runtime.block_on(
        router.route(
            hyper::Request::builder()
                .method("POST")
                .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                .header("content-type", "application/proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    stale_runtime_read_request,
                )))
                .expect("Gateway stale-runtime Communications Export artifact read request"),
        ),
    );
    assert_eq!(
        stale_runtime_response.status(),
        hyper::StatusCode::NOT_FOUND,
        "workflow restart invalidates predecessor runtime-local read tickets"
    );
    let read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.opaque_read_capability,
    }
    .encode_to_vec();
    let read = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        read_request.clone(),
                    )))
                    .expect("Gateway Communications Export artifact read request"),
            ),
        )
    };
    let response = read();
    assert_eq!(response.status(), hyper::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(response.headers().get("x-blob-reference").is_none());
    assert!(response.headers().get("digest").is_none());
    let artifact = runtime
        .block_on(response.into_body().collect())
        .expect("Gateway Communications Export artifact response")
        .to_bytes();
    assert_eq!(
        u64::try_from(artifact.len()).ok(),
        Some(inputs.declared_bytes)
    );
    assert!(artifact.starts_with(
        br#"{"record_type":"manifest","schema":"makosh.communications.evidence-export.v1"#
    ));
    assert!(
        artifact
            .windows(br#""logical_owner_id":"owner-1""#.len())
            .any(|window| window == br#""logical_owner_id":"owner-1""#),
        "artifact manifest carries the exact logical owner provenance"
    );
    assert!(
        artifact
            .windows(b"fixture source body for custody transfer".len())
            .any(|window| window == b"fixture source body for custody transfer")
    );
    assert!(
        !artifact
            .windows(inputs.edited_body.len())
            .any(|window| window == inputs.edited_body),
        "pre-edit export artifact remains bound to its original canonical snapshot"
    );
    assert_eq!(read().status(), hyper::StatusCode::NOT_FOUND);
    let edited_read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.edited_opaque_read_capability,
    }
    .encode_to_vec();
    let read_edited = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        edited_read_request.clone(),
                    )))
                    .expect("Gateway edited Communications Export artifact read request"),
            ),
        )
    };
    let edited_response = read_edited();
    assert_eq!(edited_response.status(), hyper::StatusCode::OK);
    let edited_artifact = runtime
        .block_on(edited_response.into_body().collect())
        .expect("Gateway edited Communications Export artifact response")
        .to_bytes();
    assert_eq!(
        u64::try_from(edited_artifact.len()).ok(),
        Some(inputs.edited_declared_bytes)
    );
    assert!(
        edited_artifact
            .windows(inputs.edited_body.len())
            .any(|window| window == inputs.edited_body),
        "post-edit export artifact contains the edited canonical snapshot"
    );
    assert_eq!(read_edited().status(), hyper::StatusCode::NOT_FOUND);
    let blob_outage_read_request = EvidenceExportArtifactReadRequestV1 {
        opaque_read_capability: inputs.blob_outage_read_capability,
    }
    .encode_to_vec();
    let read_during_blob_outage = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        blob_outage_read_request.clone(),
                    )))
                    .expect("Gateway Communications Export Blob-outage read request"),
            ),
        )
    };

    supervisor
        .stop("blob")
        .expect("stop Blob for Communications Export artifact outage");
    assert_eq!(
        read_during_blob_outage().status(),
        hyper::StatusCode::SERVICE_UNAVAILABLE,
        "Blob outage fails closed without disclosing Communications Export artifact bytes"
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            supervisor,
            store,
            kernel_executable,
            kernel_data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after Communications Export artifact outage"),
        2
    );
    assert_eq!(
        read_during_blob_outage().status(),
        hyper::StatusCode::NOT_FOUND,
        "artifact ticket is consumed atomically before the failed Blob read and cannot be replayed"
    );
}

fn assert_communications_export_gateway_rejects_revoked_ticket(
    fixture: &CommunicationsExportGatewayFixtureV1<'_>,
    opaque_read_capability: Vec<u8>,
) {
    use makosh_communications_export_api::{
        COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1, wire::EvidenceExportArtifactReadRequestV1,
    };
    let router = fixture.router;
    let runtime = fixture.runtime;
    let cookie = fixture.cookie;

    let response = runtime.block_on(
        router.route(
            hyper::Request::builder()
                .method("POST")
                .uri(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1)
                .header("content-type", "application/proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    EvidenceExportArtifactReadRequestV1 {
                        opaque_read_capability,
                    }
                    .encode_to_vec(),
                )))
                .expect("Gateway revoked Communications Export artifact read request"),
        ),
    );
    assert_eq!(
        response.status(),
        hyper::StatusCode::NOT_FOUND,
        "revoke removes the exact export client_blob route before any artifact read"
    );
}

fn short_communications_kernel_data_directory() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    PathBuf::from("/tmp").join(format!("makosh-comms-{}-{suffix}", std::process::id()))
}

fn assert_communications_gateway_query_delivery(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &std::path::Path,
    kernel_data: &std::path::Path,
) {
    use http_body_util::BodyExt as _;

    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("gateway-cert.der"),
        root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(store),
        kernel_data,
        supervisor.clone(),
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(1_024)
            .expect("test realtime source"),
        &configuration,
        None,
    )
    .expect("compose owner Gateway routes");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway test runtime");
    let cookie = super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);
    let route_query =
        |request: makosh_communications_api::query_wire::CommunicationsQueryRequestV1| {
            let response = runtime.block_on(
                router.route(
                    hyper::Request::builder()
                        .method("POST")
                        .uri("/makosh.communications.query.v1.CommunicationsQueryService/Query")
                        .header("content-type", "application/connect+proto")
                        .header("cookie", &cookie)
                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                            request.encode_to_vec(),
                        )))
                        .expect("Gateway owner query request"),
                ),
            );
            assert_eq!(response.status(), hyper::StatusCode::OK);
            assert_eq!(
                response
                    .headers()
                    .get("content-type")
                    .and_then(|value| value.to_str().ok()),
                Some("application/proto"),
            );
            assert_eq!(
                response
                    .headers()
                    .get("connect-protocol-version")
                    .and_then(|value| value.to_str().ok()),
                Some("1"),
            );
            let bytes = runtime
                .block_on(response.into_body().collect())
                .expect("Gateway owner query response")
                .to_bytes();
            makosh_communications_api::query_wire::CommunicationsQueryResponseV1::decode(
                bytes.as_ref(),
            )
            .expect("decode Gateway Communications query response")
        };
    let route_saved_search =
        |request: makosh_communications_saved_query_api::CommunicationsSavedSearchRequestV1| {
            let response = runtime.block_on(
                router.route(
                    hyper::Request::builder()
                        .method("POST")
                        .uri(makosh_communications_saved_query_api::SAVED_SEARCH_CONNECT_PATH_V1)
                        .header("content-type", "application/connect+proto")
                        .header("cookie", &cookie)
                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                            request.encode_to_vec(),
                        )))
                        .expect("Gateway Communications saved-search request"),
                ),
            );
            assert_eq!(response.status(), hyper::StatusCode::OK);
            let bytes = runtime
                .block_on(response.into_body().collect())
                .expect("Gateway Communications saved-search response")
                .to_bytes();
            makosh_communications_saved_query_api::CommunicationsSavedSearchResponseV1::decode(
                bytes.as_ref(),
            )
            .expect("decode Gateway Communications saved-search response")
        };
    let route_sender_insights =
        |request: makosh_communications_sender_insights_api::ListSenderInsightsRequestV1| {
            let response = runtime.block_on(
                router.route(
                    hyper::Request::builder()
                        .method("POST")
                        .uri(
                            makosh_communications_sender_insights_api::SENDER_INSIGHTS_CONNECT_PATH_V1,
                        )
                        .header("content-type", "application/connect+proto")
                        .header("cookie", &cookie)
                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                            request.encode_to_vec(),
                        )))
                        .expect("Gateway Communications sender-insights request"),
                ),
            );
            assert_eq!(response.status(), hyper::StatusCode::OK);
            let bytes = runtime
                .block_on(response.into_body().collect())
                .expect("Gateway Communications sender-insights response")
                .to_bytes();
            makosh_communications_sender_insights_api::ListSenderInsightsResponseV1::decode(
                bytes.as_ref(),
            )
            .expect("decode Gateway Communications sender-insights response")
        };
    let response = route_query(
        makosh_communications_api::query_wire::CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(
                makosh_communications_api::query_wire::communications_query_request_v1::Operation::ListAccounts(
                    makosh_communications_api::query_wire::ListAccountsRequestV1 {
                        limit: 16,
                        cursor: Vec::new(),
                    },
                ),
            ),
        },
    );
    assert!(matches!(
        response.result,
        Some(makosh_communications_api::query_wire::communications_query_response_v1::Result::ListAccounts(accounts))
            if !accounts.accounts.is_empty()
    ));

    let response = route_query(
        makosh_communications_api::query_wire::CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(
                makosh_communications_api::query_wire::communications_query_request_v1::Operation::SearchCommunications(
                    makosh_communications_api::query_wire::SearchCommunicationsRequestV1 {
                        query: "fixture".to_owned(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                ),
            ),
        },
    );
    assert!(response.error_code.is_empty());
    assert!(matches!(
        &response.result,
        Some(makosh_communications_api::query_wire::communications_query_response_v1::Result::SearchCommunications(hits))
            if !hits.hits.is_empty()
                && hits.hits.iter().all(|hit| {
                    hit.evidence_id.len() == 16
                        && hit.message_id.len() == 16
                        && hit.conversation_id.len() == 16
                        && hit.matched_token_count > 0
                })
    ));
    let message_id = match &response.result {
        Some(
            makosh_communications_api::query_wire::communications_query_response_v1::Result::SearchCommunications(
                hits,
            ),
        ) => hits
            .hits
            .iter()
            .find_map(|hit| {
                let detail = route_query(
                    makosh_communications_api::query_wire::CommunicationsQueryRequestV1 {
                        protocol_major: 1,
                        operation: Some(
                            makosh_communications_api::query_wire::communications_query_request_v1::Operation::GetMessage(
                                makosh_communications_api::query_wire::GetMessageRequestV1 {
                                    message_id: hit.message_id.clone(),
                                },
                            ),
                        ),
                    },
                );
                matches!(
                    detail.result,
                    Some(
                        makosh_communications_api::query_wire::communications_query_response_v1::Result::GetMessage(
                            makosh_communications_api::query_wire::GetMessageResponseV1 {
                                message: Some(ref message),
                            },
                        ),
                    ) if message.body_state == 4
                )
                .then(|| hit.message_id.clone())
            })
            .expect("search result includes the admitted canonical body"),
        _ => unreachable!("search result checked above"),
    };
    let public_payload = response.encode_to_vec();
    for private_value in [
        "fixture source body for custody transfer",
        "blob://fixture-source/admitted-body-1",
    ] {
        assert!(
            !public_payload
                .windows(private_value.len())
                .any(|window| window == private_value.as_bytes()),
            "external Communications search must not reveal private body or Blob locator",
        );
    }

    let sender_insights = route_sender_insights(
        makosh_communications_sender_insights_api::ListSenderInsightsRequestV1 {
            protocol_major: 1,
            account_id: None,
            limit: 20,
            cursor: Vec::new(),
        },
    );
    assert_eq!(
        sender_insights.error,
        makosh_communications_sender_insights_api::SenderInsightsErrorCodeV1::SenderInsightsErrorCodeUnspecified
            as i32
    );
    let sender_insight = sender_insights
        .items
        .iter()
        .find(|item| item.display_label.as_deref() == Some("Fixture Sender <sender@example.test>"))
        .expect("managed sender projection contains the admitted Mail fixture sender");
    assert_eq!(sender_insight.sender_id.len(), 16);
    assert_eq!(
        sender_insight.display_label.as_deref(),
        Some("Fixture Sender <sender@example.test>")
    );
    assert_eq!(sender_insight.message_count, 1);
    assert_eq!(sender_insight.conversation_count, 1);
    assert!(sender_insight.first_observed_at_unix_seconds > 0);
    assert!(
        sender_insight.last_observed_at_unix_seconds
            >= sender_insight.first_observed_at_unix_seconds
    );
    let sender_insights_payload = sender_insights.encode_to_vec();
    for private_value in [
        "integration-private-account-1",
        "integration-private-record-1",
        "fixture source body for custody transfer",
        "blob://fixture-source/admitted-body-1",
    ] {
        assert!(
            !sender_insights_payload
                .windows(private_value.len())
                .any(|window| window == private_value.as_bytes()),
            "sender-insights response must not reveal provider locators or message content",
        );
    }

    use makosh_communications_saved_query_api::{
        CommunicationsSavedSearchRequestV1, CreateSavedSearchRequestV1, DeleteSavedSearchRequestV1,
        ExecuteSavedSearchRequestV1, ListSavedSearchesRequestV1, ReplaceSavedSearchRequestV1,
        SavedSearchErrorCodeV1,
        communications_saved_search_request_v1::Operation as SavedSearchOperation,
        communications_saved_search_response_v1::Result as SavedSearchResult,
    };
    let saved_search_id = vec![0x31; 16];
    let create = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Create(CreateSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            name: "Fixture evidence".to_owned(),
            description: Some("Managed conformance definition".to_owned()),
            account_id: None,
            query: "fixture".to_owned(),
        })),
    });
    assert_eq!(
        create.error,
        SavedSearchErrorCodeV1::SavedSearchErrorCodeUnspecified as i32
    );
    assert!(matches!(
        create.result,
        Some(SavedSearchResult::Mutation(ref mutation))
            if matches!(mutation.item, Some(ref item) if item.revision == 1)
    ));

    let list = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::List(ListSavedSearchesRequestV1 {
            limit: 16,
            cursor: Vec::new(),
        })),
    });
    assert!(matches!(
        list.result,
        Some(SavedSearchResult::List(ref page))
            if page.items.iter().any(|item| item.saved_search_id == saved_search_id)
    ));

    let execute = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Execute(ExecuteSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            limit: 16,
            cursor: Vec::new(),
        })),
    });
    assert!(matches!(
        execute.result,
        Some(SavedSearchResult::Execute(ref page))
            if page.definition_revision == 1 && !page.hits.is_empty()
    ));
    let saved_search_payload = execute.encode_to_vec();
    assert!(
        !saved_search_payload
            .windows("fixture".len())
            .any(|window| window == b"fixture"),
        "saved-search responses must not reveal query plaintext"
    );

    let stale_replace = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Replace(ReplaceSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            expected_revision: 99,
            name: "Fixture evidence".to_owned(),
            description: None,
            account_id: None,
            query: "fixture".to_owned(),
        })),
    });
    assert_eq!(
        stale_replace.error,
        SavedSearchErrorCodeV1::SavedSearchErrorCodeRevisionConflict as i32
    );

    let deleted = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Delete(DeleteSavedSearchRequestV1 {
            saved_search_id: saved_search_id.clone(),
            expected_revision: 1,
        })),
    });
    assert!(matches!(
        deleted.result,
        Some(SavedSearchResult::Delete(ref result))
            if result.saved_search_id == saved_search_id && result.revision == 2
    ));
    let missing = route_saved_search(CommunicationsSavedSearchRequestV1 {
        protocol_major: 1,
        operation: Some(SavedSearchOperation::Execute(ExecuteSavedSearchRequestV1 {
            saved_search_id,
            limit: 16,
            cursor: Vec::new(),
        })),
    });
    assert_eq!(
        missing.error,
        SavedSearchErrorCodeV1::SavedSearchErrorCodeNotFound as i32
    );

    let ticket_response = runtime.block_on(
        router.route(
            hyper::Request::builder()
                .method("POST")
                .uri(makosh_communications_content_api::CONTENT_TICKET_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(hyper::body::Bytes::from(
                    makosh_communications_content_api::IssueMessageBodyReadRequestV1 {
                        protocol_major: 1,
                        message_id,
                    }
                    .encode_to_vec(),
                )))
                .expect("Gateway Communications content ticket request"),
        ),
    );
    assert_eq!(ticket_response.status(), hyper::StatusCode::OK);
    let ticket_bytes = runtime
        .block_on(ticket_response.into_body().collect())
        .expect("Gateway Communications content ticket response")
        .to_bytes();
    let ticket = makosh_communications_content_api::IssueMessageBodyReadResponseV1::decode(
        ticket_bytes.as_ref(),
    )
    .expect("decode Communications content ticket");
    assert!(ticket.error_code.is_empty());
    assert_eq!(ticket.opaque_read_capability.len(), 32);
    assert_eq!(
        ticket.declared_bytes,
        u64::try_from("fixture source body for custody transfer".len()).expect("fixture body size")
    );
    let read_request = makosh_communications_content_api::ReadMessageBodyRequestV1 {
        protocol_major: 1,
        opaque_read_capability: ticket.opaque_read_capability,
    }
    .encode_to_vec();
    let read = || {
        runtime.block_on(
            router.route(
                hyper::Request::builder()
                    .method("POST")
                    .uri(makosh_communications_content_api::CONTENT_READ_BLOB_PATH_V1)
                    .header("content-type", "application/proto")
                    .header("cookie", &cookie)
                    .body(http_body_util::Full::new(hyper::body::Bytes::from(
                        read_request.clone(),
                    )))
                    .expect("Gateway Communications content read request"),
            ),
        )
    };
    let content = read();
    assert_eq!(content.status(), hyper::StatusCode::OK);
    assert_eq!(
        content
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(content.headers().get("x-blob-reference").is_none());
    assert!(content.headers().get("digest").is_none());
    assert_eq!(
        runtime
            .block_on(content.into_body().collect())
            .expect("Gateway Communications content response")
            .to_bytes()
            .as_ref(),
        b"fixture source body for custody transfer"
    );
    assert_eq!(read().status(), hyper::StatusCode::NOT_FOUND);
}

struct SchedulerRecoveryFixture {
    root: PathBuf,
    release: InstalledSignedBundle,
    store: Arc<SqliteControlStore>,
    shutdown: Arc<AtomicBool>,
    supervisor: ManagedRuntimeSupervisor,
}

impl SchedulerRecoveryFixture {
    fn start() -> Self {
        let root = unique_target_root("makosh-managed-scheduler-lifecycle");
        let data = private_directory(root.join("kernel"));
        initialize_vault(
            &private_directory(data.join("vault")),
            &credential_directory(),
        );
        let release = installed_scheduler_release(&root);
        let store = Arc::new(configured_scheduler_store(&root, release.kernel()));
        let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
        let shutdown = Arc::new(AtomicBool::new(false));
        let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
        configure_route_handler(&supervisor, &store, &data);
        supervisor
            .configure_event_credential_handler(Arc::new(
                UnauthenticatedNatsCredentialHandler::new(Arc::clone(&store)),
            ))
            .expect("configure Scheduler Event credential handler");
        start_vault(&supervisor, &store, &data, release.kernel());
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory(),
        );
        issue_initial_scheduler_storage_binding(&store);
        crate::platform::storage::provisioning::apply_reserved_binding(
            &supervisor,
            &store,
            &scheduler_binding(&store),
        )
        .unwrap_or_else(|error| panic!("provision initial Scheduler Storage binding: {error:?}"));
        configure_scheduler_jetstream(&store);
        configure_scheduler_delivery_observer(&store);
        Self {
            root,
            release,
            store,
            shutdown,
            supervisor,
        }
    }

    fn start_initial_scheduler(&self) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
        let reservation =
            managed_launch::load(&self.supervisor, &self.store, SCHEDULER_REGISTRATION)
                .expect("load initial Scheduler reservation");
        let binding = scheduler_binding(&self.store);
        assert_eq!(
            scheduler_launch::start_from_reservation(
                &self.supervisor,
                &self.store,
                self.release.kernel(),
                &self.root.join("runtime"),
                reservation,
                &binding,
            )
            .expect("start initial Scheduler"),
            1
        );
        binding
    }

    fn persist_recovery_schedule(&self) -> i64 {
        let replaced_due_at = future_due_at_unix_millis();
        let due_at = replaced_due_at + 3_000;
        upsert_recovery_schedule(
            &self.supervisor,
            1,
            replaced_due_at,
            SchedulerScheduleUpsertOutcomeV1::Inserted,
        );
        upsert_recovery_schedule(
            &self.supervisor,
            2,
            due_at,
            SchedulerScheduleUpsertOutcomeV1::Updated,
        );
        due_at
    }

    fn restart_after_crash(&self, due_at: i64) -> std::thread::JoinHandle<Result<(), String>> {
        self.supervisor
            .stop(SCHEDULER_REGISTRATION)
            .expect("simulate Scheduler crash");
        wait_until_due(due_at);
        let store = Arc::clone(&self.store);
        let supervisor = self.supervisor.clone();
        let shutdown = Arc::clone(&self.shutdown);
        let runtime_dir = self.root.join("runtime");
        let kernel = self.release.kernel().to_path_buf();
        std::thread::spawn(move || {
            scheduler_lifecycle::serve(store, &kernel, &runtime_dir, shutdown, supervisor, None)
        })
    }

    fn assert_successor(
        &self,
        binding: &makosh_kernel_control_store::PlatformStorageBindingV1,
        due_at: i64,
    ) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
        wait_for_scheduler_generation(&self.supervisor, &self.store, 2);
        let successor = scheduler_binding(&self.store);
        assert_eq!(successor.runtime_generation(), 2);
        assert_ne!(
            successor.runtime_instance_id(),
            binding.runtime_instance_id()
        );
        assert_eq!(successor.role_epoch(), 2);
        assert_eq!(successor.credential_lease_revision(), 2);
        assert_recovered_scheduler_delivery(&self.store, due_at);
        successor
    }

    fn assert_revoked_binding_does_not_restart(
        &self,
        successor: makosh_kernel_control_store::PlatformStorageBindingV1,
    ) {
        let revoking = self
            .store
            .begin_platform_storage_binding_revocation(
                SCHEDULER_REGISTRATION,
                STORAGE_CAPABILITY,
                successor.binding_revision(),
            )
            .expect("reserve successor binding revocation");
        self.supervisor
            .stop(SCHEDULER_REGISTRATION)
            .expect("stop revoked Scheduler");
        std::thread::sleep(Duration::from_millis(600));
        assert!(
            !self
                .supervisor
                .is_active(SCHEDULER_REGISTRATION)
                .expect("read Scheduler state")
        );
        assert_eq!(revoking.runtime_generation(), 2);
    }

    fn shutdown(self, worker: std::thread::JoinHandle<Result<(), String>>) {
        self.shutdown.store(true, Ordering::Release);
        worker
            .join()
            .expect("join Scheduler lifecycle")
            .expect("lifecycle exits");
        self.supervisor.shutdown().expect("stop managed processes");
        std::fs::remove_dir_all(self.root).expect("remove fixture");
    }
}

fn assert_recovered_scheduler_delivery(store: &SqliteControlStore, due_at: i64) {
    let envelope = recovered_scheduler_delivery(store);
    assert!(
        matches!(envelope.contract, Some(contract) if contract.owner == "platform" && contract.name == "maintenance")
    );
    assert!(
        matches!(envelope.source, Some(source) if source.module_id == SCHEDULER_RUNTIME_MODULE_ID_V1 && source.runtime_generation == 2)
    );
    let command = ScheduledJobCommandV1::decode(envelope.payload.as_slice())
        .expect("decode recovered Scheduler command");
    assert_eq!(command.schedule_revision, 2);
    assert_eq!(command.scheduled_for_unix_millis, due_at);
}

fn configure_route_handler(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
) {
    let blob_session_handler = Arc::new(BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        data.to_path_buf(),
    ));
    configure_route_handlers(supervisor, store, data, blob_session_handler);
}

fn configure_route_handlers(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
    blob_session_handler: Arc<dyn ManagedRuntimeBlobSessionHandler>,
) {
    let vault_route = Arc::new(KernelManagedVaultRouteHandler::new(
        Arc::clone(store),
        data,
        Arc::new(supervisor.relay_port()),
    ));
    let vault_handler: Arc<
        dyn crate::runtime::lifecycle::control::ManagedRuntimeVaultRouteHandler,
    > = vault_route.clone();
    supervisor
        .configure_vault_route_handler(vault_handler)
        .expect("Vault route handler");
    supervisor
        .configure_provider_credential_handler(Arc::new(ProviderCredentialHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            Arc::clone(&vault_route),
        )))
        .expect("provider credential handler");
    supervisor
        .configure_owner_derived_key_handler(Arc::new(OwnerDerivedKeyHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            vault_route,
        )))
        .expect("owner-derived key handler");
    supervisor
        .configure_blob_session_handler(blob_session_handler)
        .expect("Blob session handler");
    supervisor
        .configure_blob_custody_release_handler(Arc::new(BlobCustodyReleaseHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            data.to_path_buf(),
        )))
        .expect("Blob custody release handler");
}

fn upsert_recovery_schedule(
    supervisor: &ManagedRuntimeSupervisor,
    schedule_revision: u64,
    due_at: i64,
    expected_outcome: SchedulerScheduleUpsertOutcomeV1,
) {
    let request = SchedulerRuntimeControlRequestV1 {
        operation: Some(SchedulerOperation::UpsertSchedule(
            UpsertSchedulerScheduleRequestV1 {
                schedule_id: vec![9; 16],
                schedule_revision,
                job_owner: "platform".to_owned(),
                job_name: "maintenance".to_owned(),
                job_major: 1,
                contract_name: "platform.maintenance".to_owned(),
                contract_revision: 1,
                contract_schema_sha256: vec![7; 32],
                scope_id: "recovery:opaque".to_owned(),
                concurrency_key: "recovery:opaque".to_owned(),
                enabled: true,
                policy_canonical_bytes: one_shot_recovery_policy(due_at),
                next_due_at_unix_millis: due_at,
                updated_at_unix_millis: due_at - 1_000,
            },
        )),
    };
    let response = supervisor
        .relay(SCHEDULER_REGISTRATION, request.encode_to_vec())
        .expect("persist recovery schedule through Scheduler control");
    let response = SchedulerRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode Scheduler schedule response");
    assert!(matches!(
        response.result,
        Some(SchedulerResult::UpsertSchedule(result))
            if result.schedule_revision == schedule_revision
                && result.outcome == expected_outcome as i32
    ));
    assert!(response.error_code.is_empty());
}

fn future_due_at_unix_millis() -> i64 {
    current_unix_millis() + 2_000
}

fn current_unix_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock after epoch")
        .as_millis()
        .try_into()
        .expect("Unix milliseconds fit i64")
}

fn wait_until_due(due_at: i64) {
    let now = current_unix_millis();
    if due_at > now {
        let delay = u64::try_from(due_at - now).expect("future due delay") + 100;
        std::thread::sleep(Duration::from_millis(delay));
    }
}

fn one_shot_recovery_policy(due_at: i64) -> Vec<u8> {
    let mut policy = Vec::with_capacity(32);
    policy.push(1); // encoding version
    policy.push(1); // trigger: at
    policy.extend_from_slice(&due_at.to_be_bytes());
    policy.push(1); // overlap: forbid
    policy.push(2); // misfire: fire once after successor recovery
    policy.extend_from_slice(&1_u16.to_be_bytes()); // retry attempts
    policy.extend_from_slice(&1_000_u64.to_be_bytes()); // retry backoff
    policy.extend_from_slice(&1_000_u64.to_be_bytes()); // command deadline
    policy.extend_from_slice(&0_u64.to_be_bytes()); // jitter
    policy
}

const SCHEDULER_REGISTRATION: &str = "scheduler_registration";
const STORAGE_CAPABILITY: &str = "storage.scheduler";
const DISPATCH_CAPABILITY: &str = "events.scheduler.dispatch";
const ACK_CAPABILITY: &str = "events.scheduler.ack";
const RESULT_CAPABILITY: &str = "events.scheduler.result";
const SCHEDULE_CONTROL_COMMAND_CAPABILITY: &str = "events.scheduler.schedule_control.command";
const SCHEDULE_CONTROL_RESULT_CAPABILITY: &str = "events.scheduler.schedule_control.result";

fn scheduler_binding(
    store: &SqliteControlStore,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(SCHEDULER_REGISTRATION, STORAGE_CAPABILITY)
        .expect("read Scheduler Storage binding")
        .expect("Scheduler Storage binding")
}

fn wait_for_scheduler_generation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    expected_generation: u64,
) {
    // A managed child is allowed 15 seconds to announce readiness; include the
    // lifecycle poll and Storage/Vault provisioning time before declaring the
    // recovery contour failed.
    let deadline = std::time::Instant::now() + Duration::from_secs(25);
    while std::time::Instant::now() < deadline {
        let active = supervisor
            .is_active(SCHEDULER_REGISTRATION)
            .expect("read Scheduler runtime state");
        let generation = store
            .effective_managed_launch_record(SCHEDULER_REGISTRATION)
            .expect("read Scheduler launch record")
            .map(|record| record.runtime_generation());
        if active && generation == Some(expected_generation) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "Scheduler successor did not reach generation {expected_generation}: {:?}",
        supervisor.last_failure(SCHEDULER_REGISTRATION)
    );
}

struct UnauthenticatedNatsCredentialHandler {
    store: Arc<SqliteControlStore>,
}

impl UnauthenticatedNatsCredentialHandler {
    fn new(store: Arc<SqliteControlStore>) -> Self {
        Self { store }
    }
}

impl ManagedRuntimeEventCredentialHandler for UnauthenticatedNatsCredentialHandler {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String> {
        let registration = self
            .store
            .module_registration(expectation.registration_id())
            .map_err(|_| "Event registration is unavailable".to_owned())?
            .ok_or_else(|| "Event registration is unavailable".to_owned())?;
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| "Scheduler Event request is invalid".to_owned())?;
        let recipient = NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(
            request
                .recipient_public_key_x25519
                .as_slice()
                .try_into()
                .map_err(|_| "Scheduler Event request is invalid".to_owned())?,
        )
        .map_err(|_| "Scheduler Event request is invalid".to_owned())?;
        let binding = NatsRuntimeCredentialDeliveryBindingV1::new(
            NatsRuntimeCredentialDeliveryBindingInputV1 {
                logical_owner_id: registration.owner_id().to_owned(),
                registration_id: expectation.registration_id().to_owned(),
                runtime_instance_id: expectation.runtime_instance_id().to_owned(),
                runtime_generation: expectation.runtime_generation(),
                grant_epoch: expectation.grant_epoch(),
                credential_revision: request.credential_revision,
                request_id,
                recipient_public_key: recipient,
            },
        )
        .map_err(|_| "Scheduler Event binding is invalid".to_owned())?;
        let key = KeyPair::new_user();
        let credential = RuntimeNatsJwtCredentialV1::new(
            "test-jwt".to_owned(),
            key.seed()
                .map_err(|_| "Scheduler Event key is unavailable".to_owned())?,
            key.public_key(),
            u64::MAX,
        )
        .map_err(|_| "Scheduler Event credential is invalid".to_owned())?;
        let delivery = credential
            .seal_for(&binding)
            .map_err(|_| "Scheduler Event delivery is unavailable".to_owned())?;
        let contracts = event_catalog::resolve_contracts(&*self.store)
            .map_err(|_| "test Event topology is unavailable".to_owned())?;
        let configuration = self
            .store
            .platform_event_hub_topology()
            .map_err(|_| "test Event topology is unavailable".to_owned())?
            .ok_or_else(|| "test Event topology is unavailable".to_owned())?;
        if request.credential_revision != configuration.credential_revision() {
            return Err("test Event credential fence is stale".to_owned());
        }
        let topology = event_topology::plan(&contracts, &configuration)
            .map_err(|_| "test Event topology is unavailable".to_owned())?;
        let consumer_bindings = event_topology::managed_runtime_consumer_bindings(
            &topology,
            expectation.registration_id(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "test Event consumer binding is unavailable".to_owned())?;
        let publish_subjects = event_topology::managed_runtime_publish_subjects(
            &topology,
            expectation.registration_id(),
            expectation.grant_epoch(),
        );
        Ok(ManagedRuntimeEventCredentialDeliveryV1 {
            encapped_key: delivery.encapped_key().to_vec(),
            ciphertext: delivery.ciphertext().to_vec(),
            tag: delivery.tag().to_vec(),
            consumer_bindings,
            publish_subjects,
        })
    }
}
fn private_directory(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).expect("private directory");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("private directory mode");
    path
}
