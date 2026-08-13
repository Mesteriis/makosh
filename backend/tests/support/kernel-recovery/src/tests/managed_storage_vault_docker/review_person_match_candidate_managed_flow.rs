use super::*;
use crate::identity::device::signer::DeviceSigner;
use futures_util::StreamExt;
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::{
    delivery::{OutboxRecordV1, OutboxRecordV1 as DurableRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_persons_api::{
    PERSONS_MODULE_ID_V1, PersonsActionDigestSourceV1, PersonsIdentityMatchKindV1,
    persons_identity_match_candidate_id_v1, persons_merge_action_digest_v1,
    persons_owner_partition_id_v1,
    wire::{
        IdentityMatchKindV1, PersonCommandRejectedV1, PersonCommandSucceededV1,
        PersonReviewCandidateRaisedEventV1, ProviderSourceIdentityV1, TimestampV1,
    },
};
use makosh_persons_runtime::transport::{
    PersonsEnvelopeContextV1, build_persons_review_candidate_outbox_record_v1,
};
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_SHA256_V1,
    review_person_match_candidate_approved_contract_reference_v1,
    review_person_match_candidate_client_list_contract_reference_v1,
    review_person_match_candidate_decision_contract_reference_v1,
    wire::{
        DecidePersonMatchCandidateRequestV1, ListPersonMatchCandidatesRequestV1,
        ListPersonMatchCandidatesResultV1, MergePersonsReviewActionV1,
        PersonMatchCandidateApprovedActionV1, PersonMatchCandidateDecisionV1,
        PersonMatchCandidateReviewSubmittedV1, person_match_candidate_approved_action_v1::Action,
    },
};
use makosh_review_person_match_candidate_promotion_api::wire::{
    ReviewPersonMatchCandidatePromotionOutcomeV1, ReviewPersonMatchCandidatePromotionResultV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Review binary"]
fn managed_review_person_match_candidate_bootstraps_and_stops_promptly() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-review-person-match-candidate");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_review_pm_e2e_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            REVIEW_PM_HUMAN_OWNER,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim owner");
    let admitted = admit_review_pm(&store);
    let promotion = admit_reviewed_person_match_promotion_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        store.as_ref(),
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_review_pm(&supervisor, &store, admitted);
    let promotion = prepare_reviewed_person_match_promotion_v1(&supervisor, &store, promotion);
    configure_communications_jetstream(&store);
    let started = start_review_pm(&supervisor, &store, &root.join("runtime"), admitted);
    let promotion = start_reviewed_person_match_promotion_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        promotion,
    );
    assert!(
        supervisor
            .relay_port()
            .is_ready(&started.registration_id)
            .expect("ready")
    );
    let launch = store
        .effective_managed_launch_record(&started.registration_id)
        .expect("read Review client launch")
        .expect("Review client launch");
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id:
            makosh_review_person_match_candidate_api::REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1
                .to_owned(),
        owner_id: makosh_review_person_match_candidate_api::REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1
            .to_owned(),
        contract: Some(review_person_match_candidate_client_list_contract_reference_v1()),
        request_id: 1,
        request_payload: ListPersonMatchCandidatesRequestV1 {
            logical_owner_id: String::new(),
            after_review_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
        logical_owner_id: REVIEW_PM_HUMAN_OWNER.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &started.registration_id,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let response = crate::modules::capability::router::route_managed_client_request(
        store.as_ref(),
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Review list");
    let response = ModuleClientResponseV1::decode(response.as_slice()).expect("Review response");
    assert!(response.error_code.is_empty(), "{}", response.error_code);
    assert!(
        ListPersonMatchCandidatesResultV1::decode(response.response_payload.as_slice())
            .expect("Review list")
            .candidates
            .is_empty()
    );
    assert!(
        supervisor
            .relay_port()
            .is_ready(&promotion.registration_id)
            .expect("promotion ready")
    );
    let before = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&started.registration_id)
            .expect("request stop")
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .expect("join stop")
    );
    assert!(before.elapsed() < Duration::from_secs(2));
    assert_eq!(
        supervisor
            .last_failure(&started.registration_id)
            .expect("failure"),
        None
    );
    assert!(
        supervisor
            .request_stop_if_active(&promotion.registration_id)
            .expect("request promotion stop")
    );
    assert!(
        supervisor
            .stop_if_active(&promotion.registration_id)
            .expect("join promotion stop")
    );
    assert_eq!(
        supervisor
            .last_failure(&promotion.registration_id)
            .expect("promotion failure"),
        None
    );
    supervisor.shutdown().expect("shutdown dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove root");
    std::fs::remove_dir_all(data).expect("remove data");
}

#[derive(Clone, Copy)]
enum PromotionBootstrapNegativeV1 {
    MissingConsumer,
    StaleStorageFence,
    NatsControlClose,
}

#[test]
#[ignore = "requires disposable Docker plus actual reviewed Person-match promotion binary"]
fn managed_review_person_match_candidate_promotion_rejects_missing_consumer() {
    run_promotion_bootstrap_negative_v1(PromotionBootstrapNegativeV1::MissingConsumer);
}

#[test]
#[ignore = "requires disposable Docker plus actual reviewed Person-match promotion binary"]
fn managed_review_person_match_candidate_promotion_rejects_stale_storage_fence() {
    run_promotion_bootstrap_negative_v1(PromotionBootstrapNegativeV1::StaleStorageFence);
}

#[test]
#[ignore = "requires disposable Docker plus actual reviewed Person-match promotion binary"]
fn managed_review_person_match_candidate_promotion_nats_bootstrap_control_close_is_prompt() {
    run_promotion_bootstrap_negative_v1(PromotionBootstrapNegativeV1::NatsControlClose);
}

fn run_promotion_bootstrap_negative_v1(negative: PromotionBootstrapNegativeV1) {
    let root = unique_target_root("makosh-managed-review-person-match-promotion-negative");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_reviewed_person_match_promotion_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            REVIEW_PM_HUMAN_OWNER,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim owner");
    let promotion = admit_reviewed_person_match_promotion_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let promotion = prepare_reviewed_person_match_promotion_v1(&supervisor, &store, promotion);
    configure_communications_jetstream(&store);
    let runtime = tokio::runtime::Runtime::new().expect("promotion negative runtime");
    if matches!(negative, PromotionBootstrapNegativeV1::MissingConsumer) {
        runtime.block_on(delete_promotion_approval_consumer_v1(&store));
    }
    let stalled_listener = matches!(negative, PromotionBootstrapNegativeV1::NatsControlClose)
        .then(|| std::net::TcpListener::bind("127.0.0.1:0").expect("bind stalled NATS"));
    let bootstrap_override = match negative {
        PromotionBootstrapNegativeV1::StaleStorageFence => {
            ReviewedPersonMatchPromotionBootstrapOverrideV1::StaleCredentialFence
        }
        PromotionBootstrapNegativeV1::NatsControlClose => {
            let endpoint = format!(
                "nats://{}",
                stalled_listener
                    .as_ref()
                    .expect("stalled listener")
                    .local_addr()
                    .expect("stalled address"),
            );
            ReviewedPersonMatchPromotionBootstrapOverrideV1::UnavailableEventEndpoint(endpoint)
        }
        PromotionBootstrapNegativeV1::MissingConsumer => {
            ReviewedPersonMatchPromotionBootstrapOverrideV1::None
        }
    };
    let started = launch_reviewed_person_match_promotion_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        promotion,
        bootstrap_override,
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        assert!(
            !supervisor
                .relay_port()
                .is_ready(&started.registration_id)
                .expect("negative readiness")
        );
        if !supervisor
            .is_active(&started.registration_id)
            .expect("negative activity")
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    if matches!(negative, PromotionBootstrapNegativeV1::NatsControlClose) {
        assert!(
            supervisor
                .is_active(&started.registration_id)
                .expect("NATS activity")
        );
    }
    let stop_started = std::time::Instant::now();
    if supervisor
        .request_stop_if_active(&started.registration_id)
        .expect("request negative stop")
    {
        assert!(
            supervisor
                .stop_if_active(&started.registration_id)
                .expect("join negative stop")
        );
        assert!(stop_started.elapsed() < Duration::from_secs(2));
    }
    supervisor
        .shutdown()
        .expect("shutdown negative dependencies");
    shutdown.store(true, Ordering::SeqCst);
    drop(stalled_listener);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove negative root");
    std::fs::remove_dir_all(data).expect("remove negative data");
}

async fn delete_promotion_approval_consumer_v1(store: &SqliteControlStore) {
    let configuration = store
        .platform_event_hub_topology()
        .expect("read promotion topology")
        .expect("promotion topology");
    let contracts = event_catalog::resolve_contracts(store).expect("promotion contracts");
    let plan = event_topology::plan(&contracts, &configuration).expect("promotion plan");
    let expected = review_person_match_candidate_approved_contract_reference_v1();
    let durable = plan
        .consumers()
        .iter()
        .find(|consumer| {
            consumer.contract().owner == expected.owner
                && consumer.contract().name == expected.name
                && consumer.contract().major == expected.major
        })
        .expect("promotion approval consumer")
        .durable_name()
        .to_owned();
    async_nats::jetstream::new(
        async_nats::connect(configuration.nats_endpoint())
            .await
            .expect("connect promotion topology"),
    )
    .delete_consumer_from_stream(durable, "MAKOSH_EVENT_V1")
    .await
    .expect("delete promotion approval consumer");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Persons, Review and promotion binaries"]
fn managed_review_person_match_candidate_approval_reaches_actual_persons_and_review_result() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-review-person-match-e2e");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_review_pm_e2e_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            REVIEW_PM_HUMAN_OWNER,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim owner");
    let persons = admit_persons_runtime_v1(&store);
    let identity_resolution = admit_identity_resolution_v1(&store);
    let review = admit_review_pm(&store);
    let promotion = admit_reviewed_person_match_promotion_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let persons = prepare_persons_runtime_v1(&supervisor, &store, persons);
    let identity_resolution =
        prepare_identity_resolution_v1(&supervisor, &store, identity_resolution);
    let review = prepare_review_pm(&supervisor, &store, review);
    let promotion = prepare_reviewed_person_match_promotion_v1(&supervisor, &store, promotion);
    configure_communications_jetstream(&store);
    let persons = start_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), persons);
    let identity_resolution = start_identity_resolution_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        identity_resolution,
    );
    let review = start_review_pm(&supervisor, &store, &root.join("runtime"), review);
    let promotion = start_reviewed_person_match_promotion_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        promotion,
    );

    let endpoint = store
        .platform_event_hub_topology()
        .expect("event topology")
        .expect("event topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new().expect("Review E2E Tokio runtime").block_on(async {
        let client = async_nats::connect(endpoint).await.expect("connect Review E2E observer");
        let mut observer = client.subscribe("makosh.>").await.expect("subscribe Review E2E");
        let jetstream = async_nats::jetstream::new(client);
        let now = wall_seconds_v1();
        for (command, person) in [([0x11; 16], [0x21; 16]), ([0x12; 16], [0x22; 16])] {
            super::persons_managed_flow::publish(
                &jetstream,
                &super::persons_managed_flow::manual_create_command(command, person, now),
            ).await;
            let terminal = next_contract_v1(&mut observer, "persons_command_succeeded").await;
            let payload = PersonCommandSucceededV1::decode(terminal.payload.as_slice())
                .expect("decode Person create terminal");
            assert_eq!(payload.command_id, command);
        }

        let candidate = candidate_record_v1(now);
        publish_record_v1(&jetstream, &candidate).await;
        let proposal = next_contract_optional_v1(
            &mut observer,
            "identity_resolution_person_match_candidate_proposed",
        )
        .await;
        if proposal.is_none() {
            panic!(
                "identity proposal timeout identity={:?} review={:?}",
                supervisor.last_failure(&identity_resolution.registration_id),
                supervisor.last_failure(&review.registration_id),
            );
        }
        let submitted = next_contract_optional_v1(
            &mut observer,
            "review_person_match_candidate_submitted",
        )
        .await
        .unwrap_or_else(|| {
            panic!(
                "review submission timeout identity={:?} review={:?}",
                supervisor.last_failure(&identity_resolution.registration_id),
                supervisor.last_failure(&review.registration_id),
            )
        });
        let submitted = PersonMatchCandidateReviewSubmittedV1::decode(submitted.payload.as_slice())
            .expect("decode submitted Review candidate");
        assert_eq!(submitted.review_revision, 1);
        let review_id: [u8; 16] = submitted.review_id.as_slice().try_into()
            .expect("Review public ID");

        let decision = decision_record_v1(review_id, wall_seconds_v1());
        publish_record_v1(&jetstream, &decision).await;
        let observed_decision = next_contract_v1(
            &mut observer,
            "review_person_match_candidate_decision",
        ).await;
        assert_eq!(observed_decision.message_id, decision.message_id());
        let approval = next_contract_optional_v1(
            &mut observer,
            "review_person_match_candidate_approved_for_promotion",
        ).await;
        if approval.is_none() {
            panic!(
                "approval timeout queue={:?} promotion={:?} consumer={:?}",
                supervisor.last_failure(&review.registration_id),
                supervisor.last_failure(&promotion.registration_id),
                decision_consumer_diagnostic_v1(&store).await,
            );
        }
        let _persons_command = next_contract_v1(&mut observer, "persons_command").await;
        let persons_terminal = next_contract_names_v1(
            &mut observer,
            &["persons_command_succeeded", "persons_command_rejected"],
        ).await;
        if persons_terminal.contract.as_ref().is_some_and(|value| value.name == "persons_command_rejected") {
            let rejected = PersonCommandRejectedV1::decode(persons_terminal.payload.as_slice())
                .expect("decode rejected Persons promotion command");
            panic!("promotion Persons command rejected code={}", rejected.code);
        }
        let persons_terminal = PersonCommandSucceededV1::decode(persons_terminal.payload.as_slice())
            .expect("decode merged Persons terminal");
        assert_eq!(persons_terminal.affected_person_ids.len(), 2);
        let result = next_contract_v1(
            &mut observer,
            "review_person_match_candidate_promotion_result",
        ).await;
        let result_bytes = result.encode_to_vec();
        let result = ReviewPersonMatchCandidatePromotionResultV1::decode(result.payload.as_slice())
            .expect("decode Review promotion result");
        assert_eq!(result.review_id, review_id);
        assert_eq!(
            result.outcome,
            ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeSucceeded as i32,
        );
        for forbidden in [b"normalized_email".as_slice(), b"provider_entry_id".as_slice(), b"private_locator".as_slice()] {
            assert!(!result_bytes.windows(forbidden.len()).any(|window| window == forbidden));
        }
    });

    for registration in [
        &review.registration_id,
        &promotion.registration_id,
        &persons.registration_id,
        &identity_resolution.registration_id,
    ] {
        assert!(
            supervisor
                .request_stop_if_active(registration)
                .expect("request stop")
        );
        assert!(supervisor.stop_if_active(registration).expect("join stop"));
        assert_eq!(
            supervisor.last_failure(registration).expect("last failure"),
            None
        );
    }
    supervisor.shutdown().expect("shutdown dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove E2E root");
    std::fs::remove_dir_all(data).expect("remove E2E data");
}

fn candidate_record_v1(now: i64) -> OutboxRecordV1 {
    let first_source = source_v1(0x41);
    let second_source = source_v1(0x51);
    let candidate_id = persons_identity_match_candidate_id_v1(
        REVIEW_PM_HUMAN_OWNER,
        action_source_v1(&first_source),
        action_source_v1(&second_source),
        PersonsIdentityMatchKindV1::NormalizedEmail,
    )
    .expect("canonical candidate ID");
    let event_id = [0x32; 16];
    let owner_partition = persons_owner_partition_id_v1(REVIEW_PM_HUMAN_OWNER)
        .expect("canonical Persons owner partition");
    build_persons_review_candidate_outbox_record_v1(
        [0x33; 16],
        owner_partition,
        event_id,
        owner_partition,
        PersonReviewCandidateRaisedEventV1 {
            event_id: event_id.to_vec(),
            candidate_id: candidate_id.to_vec(),
            logical_owner_id: REVIEW_PM_HUMAN_OWNER.to_owned(),
            first_person_id: vec![0x21; 16],
            second_person_id: vec![0x22; 16],
            first_source: Some(first_source),
            second_source: Some(second_source),
            match_kind: IdentityMatchKindV1::IdentityMatchKindNormalizedEmail as i32,
            observed_at: Some(TimestampV1 {
                unix_seconds: now,
                nanos: 0,
            }),
            resulting_owner_revision: 2,
        },
        &PersonsEnvelopeContextV1 {
            module_id: PERSONS_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "review-person-match-e2e-persons".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .expect("build sanitized Persons candidate")
}

fn action_source_v1(source: &ProviderSourceIdentityV1) -> PersonsActionDigestSourceV1 {
    PersonsActionDigestSourceV1 {
        integration_public_id: source
            .integration_public_id
            .as_slice()
            .try_into()
            .expect("integration"),
        account_public_id: source
            .account_public_id
            .as_slice()
            .try_into()
            .expect("account"),
        provider_source_contact_public_id: source
            .provider_source_contact_public_id
            .as_slice()
            .try_into()
            .expect("source"),
    }
}

fn source_v1(seed: u8) -> ProviderSourceIdentityV1 {
    ProviderSourceIdentityV1 {
        integration_public_id: vec![seed; 16],
        account_public_id: vec![seed.wrapping_add(1); 16],
        provider_source_contact_public_id: vec![seed.wrapping_add(2); 16],
    }
}

fn decision_record_v1(review_id: [u8; 16], now: i64) -> OutboxRecordV1 {
    let operation_id = [0x61; 16];
    let owner_device_id = [0x62; 16];
    let digest =
        persons_merge_action_digest_v1(REVIEW_PM_HUMAN_OWNER, [0x21; 16], 1, [0x22; 16], 1)
            .expect("canonical merge action digest");
    let payload = DecidePersonMatchCandidateRequestV1 {
        protocol_major: 1,
        operation_id: operation_id.to_vec(),
        review_id: review_id.to_vec(),
        expected_review_revision: 1,
        decision: PersonMatchCandidateDecisionV1::PersonMatchCandidateDecisionApprove as i32,
        approved_action: Some(PersonMatchCandidateApprovedActionV1 {
            action: Some(Action::Merge(MergePersonsReviewActionV1 {
                source_person_id: vec![0x21; 16],
                expected_source_person_revision: 1,
                target_person_id: vec![0x22; 16],
                expected_target_person_revision: 1,
            })),
        }),
        approved_action_digest: digest.to_vec(),
        decided_by_owner_device_id: owner_device_id.to_vec(),
        decided_at_unix_millis: now * 1_000,
    };
    let contract = review_person_match_candidate_decision_contract_reference_v1();
    let recorded = Timestamp {
        seconds: now,
        nanos: 0,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: operation_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: "makosh-review-person-match-candidate-command-gateway".to_owned(),
            runtime_instance_id: vec![0x63; 16],
            runtime_generation: 1,
        }),
        recorded_at: Some(recorded),
        partition_key: review_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: review_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::OwnerDevice as i32,
            actor_id: owner_device_id.to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"makosh-review-person-match-candidate-command-gateway".to_vec(),
            epoch: 1,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(payload.encode_to_vec()).to_vec(),
            deadline: Some(Timestamp {
                seconds: now + 30,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    DurableRecordV1::accept(envelope.encode_to_vec()).expect("accept exact Review decision")
}

async fn publish_record_v1(context: &async_nats::jetstream::Context, record: &OutboxRecordV1) {
    let envelope =
        DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode published record");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish Review E2E record")
        .await
        .expect("ack Review E2E record");
}

async fn next_contract_v1(
    subscriber: &mut async_nats::Subscriber,
    expected_name: &str,
) -> DurableEnvelopeV1 {
    next_contract_optional_v1(subscriber, expected_name)
        .await
        .unwrap_or_else(|| panic!("timed out waiting for {expected_name}"))
}

async fn next_contract_optional_v1(
    subscriber: &mut async_nats::Subscriber,
    expected_name: &str,
) -> Option<DurableEnvelopeV1> {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let message = subscriber.next().await.expect("Review E2E observer stream");
            let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
                .expect("decode Review E2E envelope");
            if envelope
                .contract
                .as_ref()
                .is_some_and(|contract| contract.name == expected_name)
            {
                return envelope;
            }
        }
    })
    .await
    .ok()
}

async fn next_contract_names_v1(
    subscriber: &mut async_nats::Subscriber,
    expected_names: &[&str],
) -> DurableEnvelopeV1 {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let message = subscriber.next().await.expect("Review E2E observer stream");
            let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
                .expect("decode Review E2E envelope");
            if envelope
                .contract
                .as_ref()
                .is_some_and(|contract| expected_names.contains(&contract.name.as_str()))
            {
                return envelope;
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {expected_names:?}"))
}

fn wall_seconds_v1() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_secs()
        .try_into()
        .expect("wall seconds")
}

async fn decision_consumer_diagnostic_v1(store: &SqliteControlStore) -> String {
    let configuration = store
        .platform_event_hub_topology()
        .expect("read diagnostic topology")
        .expect("diagnostic topology");
    let contracts = event_catalog::resolve_contracts(store).expect("diagnostic contracts");
    let plan = event_topology::plan(&contracts, &configuration).expect("diagnostic plan");
    let expected = review_person_match_candidate_decision_contract_reference_v1();
    let durable = plan
        .consumers()
        .iter()
        .find(|consumer| {
            consumer.contract().owner == expected.owner
                && consumer.contract().name == expected.name
                && consumer.contract().major == expected.major
        })
        .expect("decision consumer plan")
        .durable_name()
        .to_owned();
    let context = async_nats::jetstream::new(
        async_nats::connect(configuration.nats_endpoint())
            .await
            .expect("diagnostic NATS"),
    );
    let consumer = context
        .get_consumer_from_stream::<async_nats::jetstream::consumer::pull::Config, _, _>(
            durable,
            "MAKOSH_COMMAND_V1",
        )
        .await
        .expect("diagnostic consumer");
    format!("{:?}", consumer.cached_info())
}
