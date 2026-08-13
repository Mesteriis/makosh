//! Actual managed Review -> promotion -> Obligations contour.

use super::*;

use std::time::Instant;

use crate::identity::device::signer::DeviceSigner;
use futures_util::StreamExt;
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_obligations_api::{
    OBLIGATIONS_CLIENT_CAPABILITY_ID_V1, OBLIGATIONS_CLIENT_CONTRACT_MAJOR_V1,
    OBLIGATIONS_CLIENT_CONTRACT_REVISION_V1, OBLIGATIONS_CLIENT_SCHEMA_SHA256_V1,
    OBLIGATIONS_CLIENT_SET_STATE_CONTRACT_NAME_V1, OBLIGATIONS_MODULE_ID_V1,
    OBLIGATIONS_OWNER_ID_V1,
    client_wire::{
        ObligationMutationResultV1, ObligationStateV1 as WireObligationStateV1,
        SetObligationStateRequestV1, TimestampV1 as WireObligationTimestampV1,
    },
    wire::ObligationCreatedFromReviewedCandidateV1,
};
use makosh_review_obligation_candidate_api::{
    REVIEW_OBLIGATION_CANDIDATE_CLIENT_CAPABILITY_ID_V1,
    REVIEW_OBLIGATION_CANDIDATE_COMMAND_CONTRACT_NAME_V1,
    REVIEW_OBLIGATION_CANDIDATE_CONTRACT_MAJOR_V1,
    REVIEW_OBLIGATION_CANDIDATE_CONTRACT_REVISION_V1, REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1,
    REVIEW_OBLIGATION_CANDIDATE_OWNER_V1, REVIEW_OBLIGATION_CANDIDATE_SCHEMA_SHA256_V1,
    ReviewObligationCandidateEnvelopeContextV1,
    build_submit_review_obligation_candidate_outbox_record_v1,
    wire::{
        DecideReviewObligationCandidateRequestV1, DecideReviewObligationCandidateResponseV1,
        GetReviewObligationCandidateRequestV1, GetReviewObligationCandidateResponseV1,
        ObligationCandidateReviewSubmittedV1, ReviewObligationCandidateContentV1,
        ReviewObligationCandidateDecisionV1, ReviewObligationCandidateErrorCodeV1,
        ReviewObligationCandidatePromotionStatusV1, ReviewObligationCandidateStateV1,
        ReviewTargetBoundCandidateReceiptV1, SubmitObligationCandidateForReviewCommandV1,
    },
};
use makosh_review_obligation_candidate_promotion_api::wire::{
    ReviewObligationCandidatePromotionOutcomeV1, ReviewObligationCandidatePromotionResultV1,
};
use makosh_reviewed_obligation_candidate_promotion_core::REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1;
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

const PRIVATE_STATEMENT_V1: &str = "Prepare the private obligation evidence";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Review, promotion and Obligations binaries"]
fn managed_obligation_candidate_promotes_to_actual_obligation_and_replays() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-obligation-candidate");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_obligation_candidate_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Obligation candidate owner");
    let blob_source = ObligationCandidateBlobSourceFixtureV1::admit(&store);
    let admitted = admit_obligation_candidate_ensemble_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_obligation_candidate_realtime_v1(
        &supervisor,
        &store,
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64)
            .expect("Obligation candidate realtime source"),
    );
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Obligation candidate Event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_obligation_candidate_ensemble_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let child_stdio = private_directory(root.join("obligation-candidate-child-stdio"));
    let mut started = start_obligation_candidate_ensemble_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted,
        &child_stdio,
    );
    assert_eq!(started.len(), 3);

    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event topology")
        .expect("Event topology")
        .nats_endpoint()
        .to_owned();
    let runtime = tokio::runtime::Runtime::new().expect("Obligation candidate Tokio runtime");
    let (submission, review_id, obligation_id) = runtime.block_on(async {
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect Obligation candidate observer");
        let mut observer = client
            .subscribe("makosh.>")
            .await
            .expect("subscribe Obligation candidate observer");
        let jetstream = async_nats::jetstream::new(client);
        let now = wall_seconds_v1();
        let content = ReviewObligationCandidateContentV1 {
            statement: PRIVATE_STATEMENT_V1.to_owned(),
            due_at: Some(makosh_review_obligation_candidate_api::wire::TimestampV1 {
                unix_seconds: now + 86_400,
                nanos: 0,
            }),
            condition: Some("owner confirms completion".to_owned()),
            obligated_party_id: vec![0x31; 16],
            beneficiary_party_id: Some(vec![0x32; 16]),
            evidence_links: vec![
                makosh_review_obligation_candidate_api::wire::ReviewObligationEvidenceLinkV1 {
                    evidence_link_id: vec![0x33; 16],
                    evidence_owner_id: "documents".to_owned(),
                    evidence_record_id: vec![0x34; 16],
                    evidence_revision: 1,
                    evidence_digest: vec![0x35; 32],
                },
            ],
        }
        .encode_to_vec();
        let blob = blob_source.write(&store, &supervisor, &data, [0x41; 16], &content);
        let submission = build_submit_review_obligation_candidate_outbox_record_v1(
            SubmitObligationCandidateForReviewCommandV1 {
                submission_id: vec![0x11; 16],
                candidate_id: vec![0x12; 16],
                candidate_digest: Sha256::digest(&content).to_vec(),
                source_evidence_id: blob.evidence_id.to_vec(),
                source_evidence_revision: 1,
                candidate_content: Some(ReviewTargetBoundCandidateReceiptV1 {
                    reference_id: blob.reference_id.to_vec(),
                    declared_bytes: blob.declared_size,
                    sha256: blob.receipt_sha256.to_vec(),
                    custody_transfer_source_proof: blob.custody_transfer_source_proof,
                }),
                logical_owner_id: OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
            now + 300,
            &ReviewObligationCandidateEnvelopeContextV1 {
                module_id: "makosh-obligation-candidate-fixture".to_owned(),
                runtime_instance_id: "obligation-candidate-fixture-runtime".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: now,
                recorded_at_nanos: 0,
            },
        )
        .expect("build exact Obligation candidate submission");
        publish_record_v1(&jetstream, &submission).await;
        let submitted = next_contract_v1(
            &mut observer,
            "review_obligation_candidate_submitted",
        )
        .await;
        let submitted = ObligationCandidateReviewSubmittedV1::decode(submitted.payload.as_slice())
            .expect("decode submitted Obligation candidate");
        let review_id: [u8; 16] = submitted
            .review_id
            .as_slice()
            .try_into()
            .expect("Review ID");

        let promotion_position = started
            .iter()
            .position(|runtime| {
                runtime.module_id == REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1
            })
            .expect("promotion runtime position");
        let promotion_predecessor = started.remove(promotion_position);
        assert!(
            supervisor
                .request_stop_if_active(&promotion_predecessor.registration_id)
                .expect("request promotion outage")
        );
        assert!(
            supervisor
                .stop_if_active(&promotion_predecessor.registration_id)
                .expect("join promotion outage")
        );

        let decision = decide_v1(
            &store,
            &supervisor,
            review_runtime_v1(&started),
            [0x21; 16],
            review_id,
            ReviewObligationCandidateDecisionV1::ReviewObligationCandidateDecisionApprove,
        );
        assert_eq!(
            decision.error,
            ReviewObligationCandidateErrorCodeV1::ReviewObligationCandidateErrorCodeUnspecified
                as i32
        );
        assert!(!decision.replayed);
        let _approval = next_contract_v1(
            &mut observer,
            "review_obligation_candidate_approved_for_promotion",
        )
        .await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(obligation_count_v1().await, 0, "promotion outage must not create owner truth");
        let promotion_successor = restart_obligation_candidate_runtime_v1(
            &supervisor,
            &store,
            &root.join("runtime"),
            promotion_predecessor,
            &child_stdio,
        );
        started.insert(promotion_position, promotion_successor);
        let command = next_contract_v1(
            &mut observer,
            "obligations_create_from_reviewed_candidate",
        )
        .await;
        assert_private_bytes_absent_v1(&command.encode_to_vec());
        let created = next_contract_v1(
            &mut observer,
            "obligations_created_from_reviewed_candidate",
        )
        .await;
        assert_private_bytes_absent_v1(&created.encode_to_vec());
        let created = ObligationCreatedFromReviewedCandidateV1::decode(created.payload.as_slice())
            .expect("decode created Obligation");
        let obligation_id: [u8; 16] = created
            .obligation_id
            .as_slice()
            .try_into()
            .expect("Obligation ID");
        let promotion_result = next_contract_v1(
            &mut observer,
            "review_obligation_candidate_promotion_result",
        )
        .await;
        assert_private_bytes_absent_v1(&promotion_result.encode_to_vec());
        let promotion_result = ReviewObligationCandidatePromotionResultV1::decode(
            promotion_result.payload.as_slice(),
        )
        .expect("decode Obligation promotion result");
        assert_eq!(
            promotion_result.outcome,
            ReviewObligationCandidatePromotionOutcomeV1::ReviewObligationCandidatePromotionOutcomeSucceeded
                as i32
        );
        assert_eq!(promotion_result.obligation_id.as_deref(), Some(obligation_id.as_slice()));

        let rejected_content = ReviewObligationCandidateContentV1 {
            statement: "Private rejected obligation candidate".to_owned(),
            due_at: None,
            condition: Some("never promoted".to_owned()),
            obligated_party_id: vec![0x51; 16],
            beneficiary_party_id: None,
            evidence_links: Vec::new(),
        }
        .encode_to_vec();
        let rejected_blob = blob_source.write(
            &store,
            &supervisor,
            &data,
            [0x42; 16],
            &rejected_content,
        );
        let rejected_submission = build_submit_review_obligation_candidate_outbox_record_v1(
            SubmitObligationCandidateForReviewCommandV1 {
                submission_id: vec![0x61; 16],
                candidate_id: vec![0x62; 16],
                candidate_digest: Sha256::digest(&rejected_content).to_vec(),
                source_evidence_id: rejected_blob.evidence_id.to_vec(),
                source_evidence_revision: 1,
                candidate_content: Some(ReviewTargetBoundCandidateReceiptV1 {
                    reference_id: rejected_blob.reference_id.to_vec(),
                    declared_bytes: rejected_blob.declared_size,
                    sha256: rejected_blob.receipt_sha256.to_vec(),
                    custody_transfer_source_proof: rejected_blob.custody_transfer_source_proof,
                }),
                logical_owner_id: OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
            now + 300,
            &ReviewObligationCandidateEnvelopeContextV1 {
                module_id: "makosh-obligation-candidate-fixture".to_owned(),
                runtime_instance_id: "obligation-candidate-fixture-runtime".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: now,
                recorded_at_nanos: 0,
            },
        )
        .expect("build rejected Obligation candidate submission");
        publish_record_v1(&jetstream, &rejected_submission).await;
        let rejected_submitted = next_contract_v1(
            &mut observer,
            "review_obligation_candidate_submitted",
        )
        .await;
        let rejected_review_id: [u8; 16] =
            ObligationCandidateReviewSubmittedV1::decode(rejected_submitted.payload.as_slice())
                .expect("decode rejected candidate submission")
                .review_id
                .try_into()
                .expect("rejected Review ID");
        let rejection = decide_v1(
            &store,
            &supervisor,
            review_runtime_v1(&started),
            [0x63; 16],
            rejected_review_id,
            ReviewObligationCandidateDecisionV1::ReviewObligationCandidateDecisionReject,
        );
        let rejected_review = rejection.review.expect("rejected review");
        assert_eq!(
            rejected_review.state,
            ReviewObligationCandidateStateV1::ReviewObligationCandidateStateRejected as i32
        );
        assert_eq!(
            rejected_review.promotion_status,
            ReviewObligationCandidatePromotionStatusV1::ReviewObligationCandidatePromotionStatusNotRequested
                as i32
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(obligation_count_v1().await, 1, "rejected candidate must not promote");
        (submission, review_id, obligation_id)
    });

    wait_for_review_promotion_v1(
        &store,
        &supervisor,
        review_runtime_v1(&started),
        review_id,
        obligation_id,
    );
    let terminal_obligation = set_obligation_state_v1(
        &store,
        &supervisor,
        obligations_runtime_v1(&started),
        obligation_id,
    );
    assert_eq!(
        terminal_obligation
            .obligation
            .expect("terminal Obligation")
            .obligation_revision,
        2
    );
    let review_position = started
        .iter()
        .position(|runtime| runtime.module_id == REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1)
        .expect("Review runtime position");
    let predecessor = started.remove(review_position);
    let successor = restart_obligation_candidate_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        predecessor,
        &child_stdio,
    );
    started.insert(review_position, successor);

    let durable_counts_before_replay = runtime.block_on(durable_counts_v1());
    let replay = decide_v1(
        &store,
        &supervisor,
        review_runtime_v1(&started),
        [0x21; 16],
        review_id,
        ReviewObligationCandidateDecisionV1::ReviewObligationCandidateDecisionApprove,
    );
    assert!(replay.replayed);
    runtime.block_on(async {
        let endpoint = store
            .platform_event_hub_topology()
            .expect("read replay Event topology")
            .expect("replay Event topology")
            .nats_endpoint()
            .to_owned();
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect replay publisher");
        publish_record_v1(&async_nats::jetstream::new(client), &submission).await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(durable_counts_v1().await, durable_counts_before_replay);
        assert_review_owner_rls_v1(
            "makosh_obligation_review_rls",
            &[
                "review_obligation_candidate_submissions",
                "review_obligation_candidate_state",
                "review_obligation_candidate_operations",
                "review_obligation_candidate_promotion_inbox",
                "review_obligation_candidate_outbox",
                "review_obligation_candidate_realtime",
                "review_obligation_candidate_evidence",
            ],
        )
        .await;
        assert_review_owner_rls_v1(
            "makosh_obligation_promotion_rls",
            &[
                "reviewed_obligation_candidate_promotion_requests",
                "reviewed_obligation_candidate_promotion_result_inbox",
                "reviewed_obligation_candidate_promotion_outbox",
            ],
        )
        .await;
        assert_review_owner_rls_v1(
            "makosh_obligations_rls",
            &[
                "obligations_reviewed_candidate_inbox",
                "obligations_state",
                "obligations_outbox",
                "obligations_evidence",
                "obligations_client_operations",
            ],
        )
        .await;
    });

    for entry in std::fs::read_dir(&child_stdio).expect("read Obligation candidate child output") {
        let bytes = std::fs::read(entry.expect("child output entry").path())
            .expect("read Obligation candidate child output file");
        assert_private_bytes_absent_v1(&bytes);
    }

    for runtime in &started {
        assert!(
            supervisor
                .request_stop_if_active(&runtime.registration_id)
                .expect("request managed stop")
        );
        assert!(
            supervisor
                .stop_if_active(&runtime.registration_id)
                .expect("join managed stop")
        );
        assert_eq!(
            supervisor
                .last_failure(&runtime.registration_id)
                .expect("managed last failure"),
            None
        );
    }
    supervisor
        .shutdown()
        .expect("shutdown managed dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Obligation candidate root");
    std::fs::remove_dir_all(data).expect("remove Obligation candidate data");
}

fn review_runtime_v1(
    started: &[StartedObligationCandidateRuntimeV1],
) -> &StartedObligationCandidateRuntimeV1 {
    started
        .iter()
        .find(|runtime| runtime.module_id == REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1)
        .expect("started Review Obligation candidate runtime")
}

fn obligations_runtime_v1(
    started: &[StartedObligationCandidateRuntimeV1],
) -> &StartedObligationCandidateRuntimeV1 {
    started
        .iter()
        .find(|runtime| runtime.module_id == OBLIGATIONS_MODULE_ID_V1)
        .expect("started Obligations runtime")
}

fn set_obligation_state_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    obligations: &StartedObligationCandidateRuntimeV1,
    obligation_id: [u8; 16],
) -> ObligationMutationResultV1 {
    let now_millis = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("wall clock")
            .as_millis(),
    )
    .expect("wall milliseconds");
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: OBLIGATIONS_MODULE_ID_V1.to_owned(),
        owner_id: OBLIGATIONS_OWNER_ID_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: OBLIGATIONS_OWNER_ID_V1.to_owned(),
            name: OBLIGATIONS_CLIENT_SET_STATE_CONTRACT_NAME_V1.to_owned(),
            major: OBLIGATIONS_CLIENT_CONTRACT_MAJOR_V1,
            revision: OBLIGATIONS_CLIENT_CONTRACT_REVISION_V1,
            schema_sha256: OBLIGATIONS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
        }),
        request_id: 3,
        request_payload: SetObligationStateRequestV1 {
            operation_id: vec![0x71; 16],
            obligation_id: obligation_id.to_vec(),
            logical_owner_id: String::new(),
            expected_obligation_revision: 1,
            state: WireObligationStateV1::ObligationStateFulfilled as i32,
            changed_at: Some(WireObligationTimestampV1 {
                unix_seconds: now_millis / 1_000,
                nanos: i32::try_from((now_millis % 1_000) * 1_000_000).expect("timestamp nanos"),
            }),
        }
        .encode_to_vec(),
        logical_owner_id: OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &obligations.registration_id,
        &obligations.runtime_instance_id,
        obligations.runtime_generation,
        obligations.grant_epoch,
        OBLIGATIONS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let response = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Obligations state mutation");
    let response = ModuleClientResponseV1::decode(response.as_slice())
        .expect("decode Obligations module response");
    assert!(
        response.error_code.is_empty(),
        "Obligations client failed: {}",
        response.error_code
    );
    ObligationMutationResultV1::decode(response.response_payload.as_slice())
        .expect("decode terminal Obligations result")
}

fn decide_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    review: &StartedObligationCandidateRuntimeV1,
    operation_id: [u8; 16],
    review_id: [u8; 16],
    decision: ReviewObligationCandidateDecisionV1,
) -> DecideReviewObligationCandidateResponseV1 {
    route_review_v1(
        store,
        supervisor,
        review,
        REVIEW_OBLIGATION_CANDIDATE_COMMAND_CONTRACT_NAME_V1,
        1,
        DecideReviewObligationCandidateRequestV1 {
            protocol_major: 1,
            operation_id: operation_id.to_vec(),
            review_id: review_id.to_vec(),
            expected_review_revision: 1,
            decision: decision as i32,
        }
        .encode_to_vec(),
    )
}

fn get_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    review: &StartedObligationCandidateRuntimeV1,
    review_id: [u8; 16],
) -> GetReviewObligationCandidateResponseV1 {
    route_review_v1(
        store,
        supervisor,
        review,
        "review.obligation-candidate.query",
        2,
        GetReviewObligationCandidateRequestV1 {
            protocol_major: 1,
            review_id: review_id.to_vec(),
        }
        .encode_to_vec(),
    )
}

fn route_review_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    review: &StartedObligationCandidateRuntimeV1,
    contract_name: &str,
    request_id: u64,
    request_payload: Vec<u8>,
) -> T {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1.to_owned(),
        owner_id: REVIEW_OBLIGATION_CANDIDATE_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: REVIEW_OBLIGATION_CANDIDATE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: REVIEW_OBLIGATION_CANDIDATE_CONTRACT_MAJOR_V1,
            revision: REVIEW_OBLIGATION_CANDIDATE_CONTRACT_REVISION_V1,
            schema_sha256: REVIEW_OBLIGATION_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
        }),
        request_id,
        request_payload,
        logical_owner_id: OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &review.registration_id,
        &review.runtime_instance_id,
        review.runtime_generation,
        review.grant_epoch,
        REVIEW_OBLIGATION_CANDIDATE_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response = crate::modules::capability::router::route_managed_client_request(
            store,
            &supervisor.relay_port(),
            &route,
        )
        .expect("route Review Obligation candidate request");
        let response = ModuleClientResponseV1::decode(response.as_slice())
            .expect("decode Review Obligation candidate module response");
        if response.error_code.is_empty() {
            return T::decode(response.response_payload.as_slice())
                .expect("decode typed Review response");
        }
        assert!(
            response.error_code == "RUNTIME_UNAVAILABLE" && Instant::now() < deadline,
            "Review client failed: {}; last_failure={:?}",
            response.error_code,
            supervisor.last_failure(&review.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_review_promotion_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    review: &StartedObligationCandidateRuntimeV1,
    review_id: [u8; 16],
    _obligation_id: [u8; 16],
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let response = get_v1(store, supervisor, review, review_id);
        if response
            .review
            .as_ref()
            .is_some_and(|value| {
                value.promotion_status
                    == ReviewObligationCandidatePromotionStatusV1::ReviewObligationCandidatePromotionStatusSucceeded
                        as i32
                    && value.review_revision == 3
            })
        {
            assert_eq!(
                response.error,
                ReviewObligationCandidateErrorCodeV1::ReviewObligationCandidateErrorCodeUnspecified
                    as i32
            );
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Review promotion did not converge to the expected Obligation"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

async fn publish_record_v1(context: &async_nats::jetstream::Context, record: &OutboxRecordV1) {
    let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode durable record");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive durable subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish durable record")
        .await
        .expect("ack durable record");
}

async fn obligation_count_v1() -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.obligations_state WHERE logical_owner_id='owner-1'",
    )
    .fetch_one(&authenticated_storage_admin_pool_v1().await)
    .await
    .expect("count durable Obligations")
}

async fn durable_counts_v1() -> [i64; 8] {
    let pool = authenticated_storage_admin_pool_v1().await;
    let mut counts = [0_i64; 8];
    for (index, table) in [
        "review_obligation_candidate_submissions",
        "review_obligation_candidate_state",
        "review_obligation_candidate_outbox",
        "reviewed_obligation_candidate_promotion_requests",
        "reviewed_obligation_candidate_promotion_outbox",
        "obligations_reviewed_candidate_inbox",
        "obligations_state",
        "obligations_outbox",
    ]
    .into_iter()
    .enumerate()
    {
        counts[index] = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table} WHERE logical_owner_id='owner-1'"
        )))
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|error| panic!("count durable replay table {table}: {error}"));
    }
    counts
}

async fn next_contract_v1(
    subscriber: &mut async_nats::Subscriber,
    contract_name: &str,
) -> DurableEnvelopeV1 {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let message = tokio::time::timeout_at(deadline, subscriber.next())
            .await
            .expect("durable contract timeout")
            .expect("durable subscriber ended");
        let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
            .expect("decode observed durable envelope");
        if envelope
            .contract
            .as_ref()
            .is_some_and(|contract| contract.name == contract_name)
        {
            return envelope;
        }
    }
}

fn assert_private_bytes_absent_v1(bytes: &[u8]) {
    for private in [
        PRIVATE_STATEMENT_V1.as_bytes(),
        b"before the release window".as_slice(),
        b"owner confirms completion".as_slice(),
        b"Private rejected obligation candidate".as_slice(),
        b"never promoted".as_slice(),
    ] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "private candidate presentation leaked into durable envelope"
        );
    }
}

fn wall_seconds_v1() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("wall clock")
            .as_secs(),
    )
    .expect("wall seconds")
}
