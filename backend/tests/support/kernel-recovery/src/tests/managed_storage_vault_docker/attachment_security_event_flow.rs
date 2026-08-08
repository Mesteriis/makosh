//! Typed event-only Attachment Security to Communications verdict conformance.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use makosh_attachment_security_contract::{
    AttachmentSecurityObservationContextV1, AttachmentSecurityScanCandidateFactV1,
    build_attachment_security_scan_candidate_outbox_record_v1,
};
use makosh_attachment_security_runtime::admission::ATTACHMENT_SECURITY_MODULE_ID;
use makosh_communications_attachment_contract::{
    AttachmentBlobAdmissionFactV1, AttachmentBlobAdmissionTransitionV1,
    AttachmentBlobExpectedStateV1, AttachmentObservationEnvelopeContextV1,
    AttachmentSafetyVerdictFactV1, build_attachment_blob_admission_outbox_record_v1,
    build_attachment_safety_verdict_outbox_record_v1,
    lifecycle_v1::{
        AttachmentSafetyStateChangedV1, AttachmentSafetyStateV1 as AttachmentSafetyStateWireV1,
    },
    safety_verdict_v1::{
        AttachmentSafetyExpectedStateV1, AttachmentSafetyVerdictObservationV1,
        AttachmentSafetyVerdictV1,
    },
};
use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use prost::Message;
use zeroize::Zeroizing;

use super::attachment_security_clamav_fixture::{
    AttachmentSecurityClamAvFixture, ClamAvFixtureOutcomeV1,
};
use super::attachment_security_persistence_fixture::{
    AttachmentSecurityPersistenceConformanceV1, AttachmentSecurityPersistenceDiagnosticsV1,
    AttachmentSecurityScanJobDiagnosticsV1,
};
use super::*;

const COMMUNICATIONS_OBSERVATION_SUBJECT: &str =
    "makosh.observation.v1.communications.communication_observed.v1";
const ATTACHMENT_ANCHOR_SUBJECT: &str =
    "makosh.event.v1.communications.communication_attachment_anchor_recorded.v1";
const ATTACHMENT_ADMISSION_SUBJECT: &str = "makosh.observation.v1.communications.\
    communication_attachment_blob_admission_observed.v1";
const ATTACHMENT_STATE_SUBJECT: &str =
    "makosh.event.v1.communications.communication_attachment_safety_state_changed.v1";
const ATTACHMENT_SCAN_CANDIDATE_SUBJECT: &str = "makosh.observation.v1.attachment_security.\
    attachment_security_scan_candidate_observed.v1";
const ATTACHMENT_VERDICT_SUBJECT: &str = "makosh.observation.v1.communications.\
    communication_attachment_safety_verdict_observed.v1";

pub(super) struct CommunicationsAttachmentFixtureV1 {
    pub(super) attachment_anchor_id: [u8; 16],
    pub(super) source_observation_id: [u8; 16],
    pub(super) correlation_id: [u8; 16],
    pub(super) blob_admitted_state_message_id: [u8; 16],
}

struct AttachmentSecurityVerdictExpectationV1 {
    scanner_outcome: ClamAvFixtureOutcomeV1,
    verdict: AttachmentSafetyVerdictV1,
    state: AttachmentSafetyStateWireV1,
}

pub(super) fn prepare_communications_attachment_for_scan(
    store: &SqliteControlStore,
    scenario_id: &str,
    declared_size: u64,
    receipt_sha256: [u8; 32],
) -> CommunicationsAttachmentFixtureV1 {
    let base_time = current_unix_seconds();
    let external_account_id = format!("attachment-security-account-{scenario_id}");
    let external_record_id = format!("attachment-security-record-{scenario_id}");
    let external_conversation_id = format!("attachment-security-conversation-{scenario_id}");
    let provider_media_locator = format!("attachment-security-private-media-{scenario_id}");
    let base = makosh_communications_ingress::new_scoped_communication_observation_draft(
        format!("attachment-security-base-{scenario_id}"),
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::MailImap,
            external_record_id: external_record_id.clone(),
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id: external_account_id.clone(),
                external_conversation_id: Some(external_conversation_id.clone()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
        makosh_communications_ingress::BodyAvailabilityV1::MetadataOnly,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(base_time),
    )
    .expect("build Attachment Security base observation");
    let base_record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &base,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: format!("attachment-security-source-{scenario_id}"),
            runtime_generation: 1,
            module_id: "attachment-security-fixture-source".to_owned(),
            recorded_at_unix_seconds: base_time,
            recorded_at_nanos: 0,
        },
    )
    .expect("build Attachment Security base envelope");
    let attachment = makosh_communications_ingress::new_scoped_communication_observation_draft(
        format!("attachment-security-media-{scenario_id}"),
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::MailImap,
            external_record_id,
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id,
                external_conversation_id: Some(external_conversation_id),
                external_participant_id: None,
                external_media_id: Some(provider_media_locator.clone()),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::MediaChanged,
        makosh_communications_ingress::BodyAvailabilityV1::MetadataOnly,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(base_time + 1),
    )
    .expect("build Attachment Security media observation");
    let attachment = makosh_communications_ingress::with_attachment_descriptor(
        attachment,
        makosh_communications_ingress::AttachmentDescriptorV1 {
            filename: Some(format!("{scenario_id}.bin")),
            media_type: "application/octet-stream".to_owned(),
            declared_bytes: declared_size,
            sha256: Some(receipt_sha256),
            disposition: makosh_communications_ingress::AttachmentDispositionV1::Attachment,
        },
    )
    .expect("attach Attachment Security descriptor");
    let attachment_record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &attachment,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: format!("attachment-security-source-{scenario_id}"),
            runtime_generation: 1,
            module_id: "attachment-security-fixture-source".to_owned(),
            recorded_at_unix_seconds: base_time + 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("build Attachment Security media envelope");
    let endpoint = event_endpoint(store);
    tokio::runtime::Runtime::new()
        .expect("Attachment Security event runtime")
        .block_on(async move {
            let client = async_nats::connect(endpoint)
                .await
                .expect("connect Attachment Security event fixture");
            let mut anchors = client
                .subscribe(ATTACHMENT_ANCHOR_SUBJECT)
                .await
                .expect("subscribe attachment anchors");
            let mut states = client
                .subscribe(ATTACHMENT_STATE_SUBJECT)
                .await
                .expect("subscribe attachment states");
            client
                .flush()
                .await
                .expect("activate attachment fixture observers");
            let context = async_nats::jetstream::new(client);
            publish_exact(&context, COMMUNICATIONS_OBSERVATION_SUBJECT, &base_record).await;
            publish_exact(
                &context,
                COMMUNICATIONS_OBSERVATION_SUBJECT,
                &attachment_record,
            )
            .await;
            let anchor_event = tokio::time::timeout(Duration::from_secs(10), anchors.next())
                .await
                .expect("attachment anchor timeout")
                .expect("attachment anchor event");
            let anchor_envelope = decode_envelope_v1(anchor_event.payload.as_ref())
                .expect("attachment anchor envelope");
            assert_eq!(
                anchor_envelope.causation_message_id,
                attachment_record.message_id().to_vec()
            );
            let anchor =
                makosh_communications_attachment_contract::anchor_recorded_v1::AttachmentAnchorRecordedV1::decode(
                    anchor_envelope.payload.as_slice(),
                )
                .expect("attachment anchor payload");
            let attachment_anchor_id: [u8; 16] = anchor
                .attachment_anchor_id
                .as_slice()
                .try_into()
                .expect("attachment anchor identifier");
            let media_cursor_sha256: [u8; 32] = anchor
                .media_cursor_sha256
                .as_slice()
                .try_into()
                .expect("attachment media cursor");
            let ingress = decode_envelope_v1(attachment_record.exact_bytes())
                .expect("attachment ingress envelope");
            let correlation_id: [u8; 16] = ingress
                .correlation_id
                .as_slice()
                .try_into()
                .expect("attachment correlation identifier");
            let requested = build_attachment_blob_admission_outbox_record_v1(
                &AttachmentBlobAdmissionFactV1 {
                    attachment_anchor_id,
                    source_observation_id: *attachment_record.message_id(),
                    correlation_id,
                    media_cursor_sha256,
                    expected_state: AttachmentBlobExpectedStateV1::DescriptorOnly,
                    transition: AttachmentBlobAdmissionTransitionV1::Requested,
                    observed_at_unix_seconds: base_time + 2,
                    blob_reference_binding_sha256: None,
                },
                &AttachmentObservationEnvelopeContextV1 {
                    runtime_instance_id: format!("attachment-security-source-{scenario_id}"),
                    runtime_generation: 1,
                    module_id: "attachment-security-fixture-source".to_owned(),
                    recorded_at_unix_seconds: base_time + 2,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build requested Blob admission");
            let admitted = build_attachment_blob_admission_outbox_record_v1(
                &AttachmentBlobAdmissionFactV1 {
                    attachment_anchor_id,
                    source_observation_id: *attachment_record.message_id(),
                    correlation_id,
                    media_cursor_sha256,
                    expected_state: AttachmentBlobExpectedStateV1::BlobPending,
                    transition: AttachmentBlobAdmissionTransitionV1::Admitted,
                    observed_at_unix_seconds: base_time + 3,
                    blob_reference_binding_sha256: Some(
                        Sha256::digest(receipt_sha256).into(),
                    ),
                },
                &AttachmentObservationEnvelopeContextV1 {
                    runtime_instance_id: format!("attachment-security-source-{scenario_id}"),
                    runtime_generation: 1,
                    module_id: "attachment-security-fixture-source".to_owned(),
                    recorded_at_unix_seconds: base_time + 3,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build admitted Blob observation");
            publish_exact(&context, ATTACHMENT_ADMISSION_SUBJECT, &requested).await;
            publish_exact(&context, ATTACHMENT_ADMISSION_SUBJECT, &admitted).await;
            let mut blob_admitted_state_message_id = None;
            for expected_causation in [requested.message_id(), admitted.message_id()] {
                let state_event = tokio::time::timeout(Duration::from_secs(10), states.next())
                    .await
                    .expect("attachment state timeout")
                    .expect("attachment state event");
                let state_envelope = decode_envelope_v1(state_event.payload.as_ref())
                    .expect("attachment state envelope");
                assert_eq!(
                    state_envelope.causation_message_id,
                    expected_causation.to_vec()
                );
                let state =
                    AttachmentSafetyStateChangedV1::decode(state_envelope.payload.as_slice())
                        .expect("attachment state payload");
                if state.next_state == AttachmentSafetyStateWireV1::BlobAdmitted as i32 {
                    blob_admitted_state_message_id = Some(
                        state_envelope
                            .message_id
                            .as_slice()
                            .try_into()
                            .expect("Blob-admitted state message identifier"),
                    );
                }
            }
            assert!(
                !anchor_event
                    .payload
                    .windows(provider_media_locator.len())
                    .any(|window| window == provider_media_locator.as_bytes()),
                "canonical attachment event must not expose provider media locator"
            );
            CommunicationsAttachmentFixtureV1 {
                attachment_anchor_id,
                source_observation_id: *attachment_record.message_id(),
                correlation_id,
                blob_admitted_state_message_id: blob_admitted_state_message_id
                    .expect("Blob-admitted canonical state"),
            }
        })
}

pub(super) fn assert_clean_attachment_security_verdict_flow(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
    clamav: &AttachmentSecurityClamAvFixture,
    forbidden_plaintext: &[u8],
) {
    assert_attachment_security_verdict_flow(
        store,
        attachment,
        blob,
        clamav,
        forbidden_plaintext,
        AttachmentSecurityVerdictExpectationV1 {
            scanner_outcome: ClamAvFixtureOutcomeV1::Clean,
            verdict: AttachmentSafetyVerdictV1::SafeForDelivery,
            state: AttachmentSafetyStateWireV1::SafeForDelivery,
        },
    );
}

pub(super) fn assert_threat_attachment_security_verdict_flow(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
    clamav: &AttachmentSecurityClamAvFixture,
    forbidden_plaintext: &[u8],
) {
    assert_attachment_security_verdict_flow(
        store,
        attachment,
        blob,
        clamav,
        forbidden_plaintext,
        AttachmentSecurityVerdictExpectationV1 {
            scanner_outcome: ClamAvFixtureOutcomeV1::Threat,
            verdict: AttachmentSafetyVerdictV1::Quarantined,
            state: AttachmentSafetyStateWireV1::Quarantined,
        },
    );
}

pub(super) fn assert_stale_attachment_security_verdict_cas_is_rejected(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
) {
    let observed_at = current_unix_seconds();
    let stale = build_attachment_safety_verdict_outbox_record_v1(
        &AttachmentSafetyVerdictFactV1 {
            attachment_anchor_id: attachment.attachment_anchor_id,
            evidence_id: [91; 16],
            causation_message_id: attachment.blob_admitted_state_message_id,
            correlation_id: attachment.correlation_id,
            expected_state:
                makosh_communications_attachment_contract::AttachmentSafetyExpectedStateV1::BlobAdmitted,
            verdict:
                makosh_communications_attachment_contract::AttachmentSafetyVerdictV1::Quarantined,
            observed_at_unix_seconds: observed_at,
        },
        &AttachmentObservationEnvelopeContextV1 {
            runtime_instance_id: "attachment-security-stale-cas-fixture".to_owned(),
            runtime_generation: 1,
            module_id: ATTACHMENT_SECURITY_MODULE_ID.to_owned(),
            recorded_at_unix_seconds: observed_at,
            recorded_at_nanos: 0,
        },
    )
    .expect("build stale Attachment Security verdict");
    let endpoint = event_endpoint(store);
    tokio::runtime::Runtime::new()
        .expect("Attachment Security stale CAS runtime")
        .block_on(async move {
            let client = async_nats::connect(endpoint)
                .await
                .expect("connect Attachment Security stale CAS observer");
            let mut states = client
                .subscribe(ATTACHMENT_STATE_SUBJECT)
                .await
                .expect("subscribe Communications stale CAS states");
            client
                .flush()
                .await
                .expect("activate Attachment Security stale CAS observer");
            let context = async_nats::jetstream::new(client);
            publish_exact(&context, ATTACHMENT_VERDICT_SUBJECT, stale.record()).await;
            assert!(
                tokio::time::timeout(Duration::from_secs(2), states.next())
                    .await
                    .is_err(),
                "stale Attachment Security verdict must not mutate Communications state"
            );
        });
}

fn assert_attachment_security_verdict_flow(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
    clamav: &AttachmentSecurityClamAvFixture,
    forbidden_plaintext: &[u8],
    expectation: AttachmentSecurityVerdictExpectationV1,
) {
    let candidate = build_attachment_security_candidate(attachment, blob);
    let endpoint = event_endpoint(store);
    tokio::runtime::Runtime::new()
        .expect("Attachment Security verdict runtime")
        .block_on(async move {
            let before = attachment_security_persistence_diagnostics().await;
            let scanner_count_before = clamav.outcome_count(expectation.scanner_outcome);
            let client = async_nats::connect(endpoint)
                .await
                .expect("connect Attachment Security verdict observer");
            let mut verdicts = client
                .subscribe(ATTACHMENT_VERDICT_SUBJECT)
                .await
                .expect("subscribe Attachment Security verdicts");
            let mut states = client
                .subscribe(ATTACHMENT_STATE_SUBJECT)
                .await
                .expect("subscribe Communications attachment states");
            client
                .flush()
                .await
                .expect("activate Attachment Security verdict observers");
            let context = async_nats::jetstream::new(client);
            publish_exact(&context, ATTACHMENT_SCAN_CANDIDATE_SUBJECT, &candidate).await;
            let verdict_event =
                match tokio::time::timeout(Duration::from_secs(30), verdicts.next()).await {
                    Ok(Some(verdict)) => verdict,
                    outcome => {
                        let diagnostics = attachment_security_persistence_diagnostics().await;
                        panic!(
                            "Attachment Security verdict unavailable: outcome={outcome:?} \
                         candidates={} canonical_states={} jobs={} attempts={} \
                         target_blob_receipts={} clamav_scans={} outbox={}",
                            diagnostics.candidates,
                            diagnostics.canonical_states,
                            diagnostics.jobs,
                            diagnostics.attempts,
                            diagnostics.target_blob_receipts,
                            clamav.scan_count(),
                            diagnostics.outbox,
                        );
                    }
                };
            let verdict_exact_bytes = verdict_event.payload.to_vec();
            let verdict_envelope = decode_envelope_v1(&verdict_exact_bytes)
                .expect("Attachment Security verdict envelope");
            assert_eq!(
                verdict_envelope.causation_message_id,
                attachment.blob_admitted_state_message_id
            );
            assert_eq!(verdict_envelope.correlation_id, attachment.correlation_id);
            let verdict =
                AttachmentSafetyVerdictObservationV1::decode(verdict_envelope.payload.as_slice())
                    .expect("Attachment Security verdict payload");
            assert_eq!(
                verdict.attachment_anchor_id,
                attachment.attachment_anchor_id
            );
            assert_eq!(
                verdict.expected_state,
                AttachmentSafetyExpectedStateV1::BlobAdmitted as i32
            );
            assert_eq!(verdict.verdict, expectation.verdict as i32);
            assert_eq!(verdict.evidence_id.len(), 16);
            let state_event = tokio::time::timeout(Duration::from_secs(30), states.next())
                .await
                .expect("Communications attachment state timeout")
                .expect("Communications attachment state");
            let state_envelope = decode_envelope_v1(state_event.payload.as_ref())
                .expect("Communications safe state envelope");
            assert_eq!(
                state_envelope.causation_message_id,
                verdict_envelope.message_id
            );
            let state = AttachmentSafetyStateChangedV1::decode(state_envelope.payload.as_slice())
                .expect("Communications safe state payload");
            assert_eq!(state.next_state, expectation.state as i32);
            for forbidden in [
                forbidden_plaintext,
                blob.reference_id.as_slice(),
                blob.receipt_sha256.as_slice(),
                blob.custody_transfer_source_proof.as_slice(),
                b"stream:".as_slice(),
                b"127.0.0.1".as_slice(),
            ] {
                assert!(
                    !verdict_exact_bytes
                        .windows(forbidden.len())
                        .any(|window| window == forbidden),
                    "verdict must not expose Blob, scanner or endpoint data"
                );
            }
            publish_exact(&context, ATTACHMENT_SCAN_CANDIDATE_SUBJECT, &candidate).await;
            assert!(
                tokio::time::timeout(Duration::from_millis(500), verdicts.next())
                    .await
                    .is_err(),
                "candidate replay must not create a second verdict"
            );
            let diagnostics = attachment_security_persistence_diagnostics().await;
            assert_eq!(diagnostics.candidates, before.candidates + 1);
            assert_eq!(
                diagnostics.canonical_states, diagnostics.candidates,
                "each scan candidate must retain one exact canonical attachment state regardless of consumer timing"
            );
            assert_eq!(diagnostics.jobs, before.jobs + 1);
            assert_eq!(diagnostics.attempts, before.attempts + 1);
            assert_eq!(
                diagnostics.target_blob_receipts,
                before.target_blob_receipts + 1
            );
            assert_eq!(diagnostics.outbox, before.outbox + 1);
            assert_eq!(
                clamav.outcome_count(expectation.scanner_outcome),
                scanner_count_before + 1
            );
        });
}

pub(super) fn assert_attachment_security_outbox_replays_after_nats_outage_and_restart<Started>(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
    clamav: &AttachmentSecurityClamAvFixture,
    stop_runtime: impl FnOnce(),
    restart_communications: impl FnOnce(),
    restart_runtime: impl FnOnce() -> Started,
) -> Started {
    let candidate = build_attachment_security_candidate(attachment, blob);
    let endpoint = event_endpoint(store);
    let runtime = tokio::runtime::Runtime::new().expect("Attachment Security outage runtime");
    let _runtime_context = runtime.enter();
    let client = runtime.block_on(async {
        let client = async_nats::connect(endpoint.clone())
            .await
            .expect("connect Attachment Security outage observer");
        client
            .flush()
            .await
            .expect("activate Attachment Security outage observer");
        client
    });
    let context = async_nats::jetstream::new(client.clone());
    runtime.block_on(publish_exact(
        &context,
        ATTACHMENT_SCAN_CANDIDATE_SUBJECT,
        &candidate,
    ));
    clamav.wait_until_held_scan_started();
    set_authenticated_nats_container_running(false);
    clamav.release_held_scan();
    let pending_exact_bytes = runtime.block_on(wait_for_pending_outage_verdict(
        attachment.attachment_anchor_id,
        clamav,
    ));
    stop_runtime();
    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&runtime, &client, "Attachment Security observer");
    let (replay_client, mut verdicts, mut states) = runtime.block_on(async {
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect fresh Attachment Security replay observer");
        let verdicts = client
            .subscribe(ATTACHMENT_VERDICT_SUBJECT)
            .await
            .expect("subscribe fresh Attachment Security replay verdicts");
        let states = client
            .subscribe(ATTACHMENT_STATE_SUBJECT)
            .await
            .expect("subscribe fresh Communications replay states");
        client
            .flush()
            .await
            .expect("activate fresh Attachment Security replay observers");
        (client, verdicts, states)
    });
    let replay_context = async_nats::jetstream::new(replay_client);
    restart_communications();
    let restarted = restart_runtime();
    runtime.block_on(async {
        let verdict = tokio::time::timeout(Duration::from_secs(15), verdicts.next())
            .await
            .expect("replayed Attachment Security verdict timeout")
            .expect("replayed Attachment Security verdict");
        assert_eq!(
            verdict.payload.as_ref(),
            pending_exact_bytes,
            "restarted relay must publish the exact persisted verdict bytes"
        );
        let verdict_envelope = decode_envelope_v1(verdict.payload.as_ref())
            .expect("replayed Attachment Security verdict envelope");
        assert_eq!(
            verdict_envelope.causation_message_id,
            attachment.blob_admitted_state_message_id
        );
        let state = tokio::time::timeout(Duration::from_secs(15), states.next())
            .await
            .expect("replayed Communications state timeout")
            .expect("replayed Communications state");
        let state = decode_envelope_v1(state.payload.as_ref())
            .expect("replayed Communications state envelope");
        assert_eq!(state.causation_message_id, verdict_envelope.message_id);
        let state = AttachmentSafetyStateChangedV1::decode(state.payload.as_slice())
            .expect("replayed Communications state payload");
        assert_eq!(
            state.next_state,
            AttachmentSafetyStateWireV1::SafeForDelivery as i32
        );
        publish_exact(
            &replay_context,
            ATTACHMENT_SCAN_CANDIDATE_SUBJECT,
            &candidate,
        )
        .await;
        assert!(
            tokio::time::timeout(Duration::from_millis(500), verdicts.next())
                .await
                .is_err(),
            "outage candidate replay must not create another verdict"
        );
        assert!(
            attachment_security_pending_verdict_outbox()
                .await
                .is_empty(),
            "restarted relay must mark the persisted verdict published"
        );
    });
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::HeldClean), 1);
    restarted
}

pub(super) fn assert_attachment_security_scanner_failure_is_fail_closed(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
    clamav: &AttachmentSecurityClamAvFixture,
    scanner_outcome: ClamAvFixtureOutcomeV1,
) {
    assert!(matches!(
        scanner_outcome,
        ClamAvFixtureOutcomeV1::Malformed
            | ClamAvFixtureOutcomeV1::Disconnect
            | ClamAvFixtureOutcomeV1::Timeout
    ));
    let candidate = build_attachment_security_candidate(attachment, blob);
    let endpoint = event_endpoint(store);
    tokio::runtime::Runtime::new()
        .expect("Attachment Security scanner failure runtime")
        .block_on(async move {
            let before = attachment_security_persistence_diagnostics().await;
            let scanner_count_before = clamav.outcome_count(scanner_outcome);
            let client = async_nats::connect(endpoint)
                .await
                .expect("connect Attachment Security failure observer");
            let mut verdicts = client
                .subscribe(ATTACHMENT_VERDICT_SUBJECT)
                .await
                .expect("subscribe Attachment Security failure verdicts");
            client
                .flush()
                .await
                .expect("activate Attachment Security failure observer");
            let context = async_nats::jetstream::new(client);
            publish_exact(&context, ATTACHMENT_SCAN_CANDIDATE_SUBJECT, &candidate).await;
            let first_job = wait_for_failed_scan_attempt(
                attachment.attachment_anchor_id,
                clamav,
                scanner_outcome,
                scanner_count_before,
            )
            .await;
            assert_eq!(first_job.state, 1);
            assert_eq!(first_job.attempt_count, 1);
            assert!(first_job.target_blob_receipt_present);
            assert!(!first_job.outbox_message_id_present);
            assert!(!first_job.claimed);
            assert!(
                tokio::time::timeout(Duration::from_millis(250), verdicts.next())
                    .await
                    .is_err(),
                "scanner failure must not create a verdict"
            );

            publish_exact(&context, ATTACHMENT_SCAN_CANDIDATE_SUBJECT, &candidate).await;
            assert!(
                tokio::time::timeout(Duration::from_millis(250), verdicts.next())
                    .await
                    .is_err(),
                "failed candidate replay must not create a verdict"
            );
            let replayed_job =
                attachment_security_scan_job_diagnostics(attachment.attachment_anchor_id)
                    .await
                    .expect("failed Attachment Security scan job");
            assert_eq!(replayed_job, first_job);
            assert_eq!(
                clamav.outcome_count(scanner_outcome),
                scanner_count_before + 1
            );
            let expected_candidates = before.candidates + 1;
            let expected_canonical_states = before.canonical_states + 1;
            let deadline = Instant::now() + Duration::from_secs(3);
            let after = loop {
                let diagnostics = attachment_security_persistence_diagnostics().await;
                if diagnostics.candidates == expected_candidates
                    && diagnostics.canonical_states == expected_canonical_states
                {
                    break diagnostics;
                }
                assert!(
                    Instant::now() < deadline,
                    "Attachment Security did not durably consume both join facts: {diagnostics:?}"
                );
                tokio::time::sleep(Duration::from_millis(25)).await;
            };
            assert_eq!(after.candidates, expected_candidates);
            assert_eq!(after.canonical_states, expected_canonical_states);
            assert_eq!(after.jobs, before.jobs + 1);
            assert_eq!(after.target_blob_receipts, before.target_blob_receipts + 1);
            assert_eq!(after.outbox, before.outbox);
        });
}

pub(super) fn assert_attachment_security_custody_failure_is_fail_closed(
    store: &SqliteControlStore,
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
    clamav: &AttachmentSecurityClamAvFixture,
    scanner_probe: ClamAvFixtureOutcomeV1,
) {
    let candidate = build_attachment_security_candidate(attachment, blob);
    let endpoint = event_endpoint(store);
    tokio::runtime::Runtime::new()
        .expect("Attachment Security custody failure runtime")
        .block_on(async move {
            let scanner_count_before = clamav.outcome_count(scanner_probe);
            let client = async_nats::connect(endpoint)
                .await
                .expect("connect Attachment Security custody failure observer");
            let mut verdicts = client
                .subscribe(ATTACHMENT_VERDICT_SUBJECT)
                .await
                .expect("subscribe Attachment Security custody failure verdicts");
            client
                .flush()
                .await
                .expect("activate Attachment Security custody failure observer");
            let context = async_nats::jetstream::new(client);
            publish_exact(&context, ATTACHMENT_SCAN_CANDIDATE_SUBJECT, &candidate).await;
            let first_job = wait_for_custody_failure(
                attachment.attachment_anchor_id,
                clamav,
                scanner_probe,
                scanner_count_before,
            )
            .await;
            assert_eq!(first_job.state, 1);
            assert!(first_job.attempt_count >= 1);
            assert!(!first_job.target_blob_receipt_present);
            assert!(!first_job.outbox_message_id_present);
            assert!(!first_job.claimed);
            assert!(
                tokio::time::timeout(Duration::from_millis(500), verdicts.next())
                    .await
                    .is_err(),
                "custody failure must not create a verdict"
            );

            publish_exact(&context, ATTACHMENT_SCAN_CANDIDATE_SUBJECT, &candidate).await;
            assert!(
                tokio::time::timeout(Duration::from_millis(500), verdicts.next())
                    .await
                    .is_err(),
                "custody failure replay must not create a verdict"
            );
            let replayed_job =
                attachment_security_scan_job_diagnostics(attachment.attachment_anchor_id)
                    .await
                    .expect("failed Attachment Security custody job");
            assert_eq!(replayed_job.state, 1);
            assert!(replayed_job.attempt_count >= first_job.attempt_count);
            assert!(!replayed_job.target_blob_receipt_present);
            assert!(!replayed_job.outbox_message_id_present);
            assert_eq!(clamav.outcome_count(scanner_probe), scanner_count_before);
        });
}

fn build_attachment_security_candidate(
    attachment: &CommunicationsAttachmentFixtureV1,
    blob: &AttachmentSecurityFixtureBlobV1,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    let observed_at = current_unix_seconds();
    build_attachment_security_scan_candidate_outbox_record_v1(
        &AttachmentSecurityScanCandidateFactV1 {
            attachment_anchor_id: attachment.attachment_anchor_id,
            blob_reference_id: blob.reference_id,
            declared_size: blob.declared_size,
            blob_receipt_sha256: blob.receipt_sha256,
            custody_transfer_source_proof: blob.custody_transfer_source_proof.clone(),
            source_observation_id: attachment.source_observation_id,
            correlation_id: attachment.correlation_id,
            observed_at_unix_seconds: observed_at,
        },
        &AttachmentSecurityObservationContextV1 {
            runtime_instance_id: "attachment-security-fixture-source-runtime".to_owned(),
            runtime_generation: 1,
            module_id: "attachment-security-fixture-source".to_owned(),
            recorded_at_unix_seconds: observed_at,
            recorded_at_nanos: 0,
        },
    )
    .expect("build Attachment Security candidate")
}

async fn wait_for_failed_scan_attempt(
    attachment_anchor_id: [u8; 16],
    clamav: &AttachmentSecurityClamAvFixture,
    scanner_outcome: ClamAvFixtureOutcomeV1,
    scanner_count_before: usize,
) -> AttachmentSecurityScanJobDiagnosticsV1 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let job = attachment_security_scan_job_diagnostics(attachment_anchor_id).await;
        if clamav.outcome_count(scanner_outcome) == scanner_count_before + 1
            && job
                .as_ref()
                .is_some_and(|job| job.attempt_count >= 1 && !job.claimed)
        {
            return job.expect("failed Attachment Security scan job");
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Security scanner failure was not persisted"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_custody_failure(
    attachment_anchor_id: [u8; 16],
    clamav: &AttachmentSecurityClamAvFixture,
    scanner_probe: ClamAvFixtureOutcomeV1,
    scanner_count_before: usize,
) -> AttachmentSecurityScanJobDiagnosticsV1 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let job = attachment_security_scan_job_diagnostics(attachment_anchor_id).await;
        if clamav.outcome_count(scanner_probe) == scanner_count_before
            && job.as_ref().is_some_and(|job| {
                job.state == 1
                    && job.attempt_count >= 1
                    && !job.target_blob_receipt_present
                    && !job.outbox_message_id_present
                    && !job.claimed
            })
        {
            return job.expect("failed Attachment Security custody job");
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Security custody failure was not persisted"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_pending_outage_verdict(
    attachment_anchor_id: [u8; 16],
    clamav: &AttachmentSecurityClamAvFixture,
) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let job = attachment_security_scan_job_diagnostics(attachment_anchor_id).await;
        let pending = attachment_security_pending_verdict_outbox().await;
        if clamav.outcome_count(ClamAvFixtureOutcomeV1::HeldClean) == 1
            && job.as_ref().is_some_and(|job| {
                job.state == 2
                    && job.attempt_count == 1
                    && job.target_blob_receipt_present
                    && job.outbox_message_id_present
                    && !job.claimed
            })
            && pending.len() == 1
        {
            return pending[0].exact_bytes().to_vec();
        }
        assert!(
            Instant::now() < deadline,
            "Attachment Security verdict was not retained during NATS outage"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn publish_exact(
    context: &async_nats::jetstream::Context,
    subject: &str,
    record: &makosh_events_protocol::delivery::OutboxRecordV1,
) {
    context
        .publish(subject.to_owned(), record.exact_bytes().to_vec().into())
        .await
        .expect("publish exact attachment event")
        .await
        .expect("acknowledge exact attachment event");
}

fn event_endpoint(store: &SqliteControlStore) -> String {
    store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned()
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("runtime clock")
        .as_secs()
        .try_into()
        .expect("runtime clock seconds")
}

async fn attachment_security_persistence_diagnostics() -> AttachmentSecurityPersistenceDiagnosticsV1
{
    let persistence = attachment_security_conformance_persistence().await;
    persistence
        .diagnostics()
        .await
        .expect("read Attachment Security persistence diagnostics")
}

async fn attachment_security_scan_job_diagnostics(
    attachment_anchor_id: [u8; 16],
) -> Option<AttachmentSecurityScanJobDiagnosticsV1> {
    let persistence = attachment_security_conformance_persistence().await;
    persistence
        .scan_job_diagnostics(attachment_anchor_id)
        .await
        .expect("read Attachment Security scan job diagnostics")
}

async fn attachment_security_pending_verdict_outbox()
-> Vec<makosh_events_protocol::delivery::OutboxRecordV1> {
    let persistence = attachment_security_conformance_persistence().await;
    persistence
        .pending_verdict_outbox(64)
        .await
        .expect("read pending Attachment Security verdict outbox")
}

async fn attachment_security_conformance_persistence() -> AttachmentSecurityPersistenceConformanceV1
{
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let port = required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
        .parse::<u16>()
        .expect("valid PostgreSQL port");
    AttachmentSecurityPersistenceConformanceV1::connect(
        &required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"),
        port,
        "makosh_postgres_admin",
        password.as_str(),
        "makosh_storage_authenticated",
    )
    .await
    .expect("connect Attachment Security conformance persistence")
}
