//! Actual Decisions lifecycle, replay, restart, privacy and owner-RLS contour.

use super::*;

use std::time::{Duration, Instant};

use makosh_decisions_api::{
    DECISIONS_CLIENT_CAPABILITY_ID_V1, DECISIONS_MODULE_ID_V1, DECISIONS_OWNER_ID_V1,
    client_wire::{
        AddDecisionAlternativeRequestV1, AddDecisionEvidenceRequestV1, CancelDecisionRequestV1,
        CreateDecisionRequestV1, DecideRequestV1, DecisionEvidenceLinkV1, DecisionMutationResultV1,
        DecisionStateV1, DecisionV1, GetDecisionRequestV1, ListDecisionAlternativesRequestV1,
        ListDecisionAlternativesResultV1, ListDecisionEvidenceRequestV1,
        ListDecisionEvidenceResultV1, ListDecisionsRequestV1, ListDecisionsResultV1,
        SupersedeDecisionRequestV1, TimestampV1,
    },
    decisions_client_routes_v1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_TITLE_V1: &str = "decisions-private-title";
const PRIVATE_QUESTION_V1: &str = "decisions-private-question";
const PRIVATE_ALTERNATIVE_V1: &str = "decisions-private-alternative";
const PRIVATE_RATIONALE_V1: &str = "decisions-private-rationale";
const PRIVATE_EVIDENCE_OWNER_V1: &str = "decisions-private-evidence-owner";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Decisions binaries"]
fn managed_decisions_lifecycle_replays_and_restarts_with_owner_rls() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-decisions");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_decisions_release_v1(&root);
    unsafe { std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel()) };
    let store = Arc::new(configured_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Decisions owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            DECISIONS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Decisions owner");
    let admitted = admit_decisions_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Decisions Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_decisions_runtime_v1(&supervisor, &store, admitted);
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let decisions =
        start_decisions_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Decisions clock")
        .as_secs() as i64;
    let at = |offset: i64| TimestampV1 {
        unix_seconds: now - 20 + offset,
        nanos: 0,
    };
    let create = CreateDecisionRequestV1 {
        operation_id: vec![0x11; 16],
        logical_owner_id: String::new(),
        title: PRIVATE_TITLE_V1.to_owned(),
        question: PRIVATE_QUESTION_V1.to_owned(),
        created_at: Some(at(0)),
    };
    let first: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        1,
        contract_v1("decisions_client_create"),
        create.encode_to_vec(),
    );
    let replay: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        2,
        contract_v1("decisions_client_create"),
        create.encode_to_vec(),
    );
    assert_eq!(first, replay);
    let created = first.decision.expect("created Decision");
    assert_eq!(created.decision_revision, 1);
    assert_eq!(created.state, DecisionStateV1::DecisionStateDraft as i32);

    let mut changed = create.clone();
    changed.question.push_str(" changed");
    assert_eq!(
        route_decisions_response_v1(
            &store,
            &supervisor,
            &decisions,
            3,
            contract_v1("decisions_client_create"),
            changed.encode_to_vec(),
        )
        .error_code,
        "CONFLICT"
    );

    let first_alternative: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        4,
        contract_v1("decisions_client_add_alternative"),
        AddDecisionAlternativeRequestV1 {
            operation_id: vec![0x12; 16],
            decision_id: created.decision_id.clone(),
            logical_owner_id: String::new(),
            expected_decision_revision: 1,
            title: PRIVATE_ALTERNATIVE_V1.to_owned(),
            description: "first bounded alternative".to_owned(),
            changed_at: Some(at(1)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        first_alternative
            .decision
            .expect("first alternative")
            .decision_revision,
        2
    );
    let second_alternative: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        5,
        contract_v1("decisions_client_add_alternative"),
        AddDecisionAlternativeRequestV1 {
            operation_id: vec![0x13; 16],
            decision_id: created.decision_id.clone(),
            logical_owner_id: String::new(),
            expected_decision_revision: 2,
            title: "Second alternative".to_owned(),
            description: "second bounded alternative".to_owned(),
            changed_at: Some(at(2)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        second_alternative
            .decision
            .expect("second alternative")
            .decision_revision,
        3
    );
    let evidence: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        6,
        contract_v1("decisions_client_add_evidence"),
        AddDecisionEvidenceRequestV1 {
            operation_id: vec![0x14; 16],
            decision_id: created.decision_id.clone(),
            logical_owner_id: String::new(),
            expected_decision_revision: 3,
            evidence: Some(DecisionEvidenceLinkV1 {
                evidence_link_id: vec![0x31; 16],
                evidence_owner_id: PRIVATE_EVIDENCE_OWNER_V1.to_owned(),
                evidence_record_id: vec![0x32; 16],
                evidence_revision: 7,
                evidence_digest: vec![0x33; 32],
            }),
            changed_at: Some(at(3)),
        }
        .encode_to_vec(),
    );
    assert_eq!(evidence.decision.expect("evidence").decision_revision, 4);

    let alternatives: ListDecisionAlternativesResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        7,
        contract_v1("decisions_client_list_alternatives"),
        ListDecisionAlternativesRequestV1 {
            logical_owner_id: String::new(),
            decision_id: created.decision_id.clone(),
            after_alternative_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(alternatives.alternatives.len(), 2);
    let selected_alternative_id = alternatives.alternatives[0].alternative_id.clone();
    let evidence_links: ListDecisionEvidenceResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        8,
        contract_v1("decisions_client_list_evidence"),
        ListDecisionEvidenceRequestV1 {
            logical_owner_id: String::new(),
            decision_id: created.decision_id.clone(),
            after_evidence_link_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(evidence_links.evidence_links.len(), 1);

    let decided: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        9,
        contract_v1("decisions_client_decide"),
        DecideRequestV1 {
            operation_id: vec![0x15; 16],
            decision_id: created.decision_id.clone(),
            logical_owner_id: String::new(),
            expected_decision_revision: 4,
            selected_alternative_id,
            rationale: PRIVATE_RATIONALE_V1.to_owned(),
            decided_at: Some(at(4)),
        }
        .encode_to_vec(),
    );
    let decided = decided.decision.expect("decided");
    assert_eq!(decided.decision_revision, 5);
    assert_eq!(decided.state, DecisionStateV1::DecisionStateDecided as i32);

    let second: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        10,
        contract_v1("decisions_client_create"),
        CreateDecisionRequestV1 {
            operation_id: vec![0x21; 16],
            logical_owner_id: String::new(),
            title: "Second decision".to_owned(),
            question: "Second question".to_owned(),
            created_at: Some(at(5)),
        }
        .encode_to_vec(),
    );
    let second_id = second.decision.expect("second decision").decision_id;
    let superseded: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        11,
        contract_v1("decisions_client_supersede"),
        SupersedeDecisionRequestV1 {
            operation_id: vec![0x22; 16],
            decision_id: created.decision_id.clone(),
            logical_owner_id: String::new(),
            expected_decision_revision: 5,
            replacement_decision_id: second_id.clone(),
            changed_at: Some(at(6)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        superseded.decision.expect("superseded").state,
        DecisionStateV1::DecisionStateSuperseded as i32
    );
    let third: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        12,
        contract_v1("decisions_client_create"),
        CreateDecisionRequestV1 {
            operation_id: vec![0x23; 16],
            logical_owner_id: String::new(),
            title: "Cancelled decision".to_owned(),
            question: "Should this draft be cancelled?".to_owned(),
            created_at: Some(at(7)),
        }
        .encode_to_vec(),
    );
    let third = third.decision.expect("third decision");
    let cancelled: DecisionMutationResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        13,
        contract_v1("decisions_client_cancel"),
        CancelDecisionRequestV1 {
            operation_id: vec![0x24; 16],
            decision_id: third.decision_id.clone(),
            logical_owner_id: String::new(),
            expected_decision_revision: 1,
            changed_at: Some(at(8)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        cancelled.decision.expect("cancelled").state,
        DecisionStateV1::DecisionStateCancelled as i32
    );
    let page1: ListDecisionsResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        14,
        contract_v1("decisions_client_list"),
        ListDecisionsRequestV1 {
            logical_owner_id: String::new(),
            after_decision_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let page2: ListDecisionsResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        15,
        contract_v1("decisions_client_list"),
        ListDecisionsRequestV1 {
            logical_owner_id: String::new(),
            after_decision_id: page1.next_after_decision_id.clone(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let page3: ListDecisionsResultV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        16,
        contract_v1("decisions_client_list"),
        ListDecisionsRequestV1 {
            logical_owner_id: String::new(),
            after_decision_id: page2.next_after_decision_id.clone(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let mut ids = vec![
        page1.decisions[0].decision_id.clone(),
        page2.decisions[0].decision_id.clone(),
        page3.decisions[0].decision_id.clone(),
    ];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&created.decision_id));
    assert!(ids.contains(&second_id));
    assert!(ids.contains(&third.decision_id));

    wait_for_decisions_relay_v1();
    let before_restart = durable_decisions_snapshot_v1();
    assert_eq!(before_restart, (3, 2, 1, 9, 9, 0));
    assert_public_decisions_outbox_is_private_free_v1();
    let decisions =
        restart_decisions_runtime_v1(&supervisor, &store, &root.join("runtime"), decisions);
    let restarted: DecisionV1 = route_decisions_v1(
        &store,
        &supervisor,
        &decisions,
        17,
        contract_v1("decisions_client_get"),
        GetDecisionRequestV1 {
            logical_owner_id: String::new(),
            decision_id: created.decision_id,
        }
        .encode_to_vec(),
    );
    assert_eq!(restarted.decision_revision, 6);
    assert_eq!(
        restarted.state,
        DecisionStateV1::DecisionStateSuperseded as i32
    );
    assert_eq!(durable_decisions_snapshot_v1(), before_restart);
    assert!(
        supervisor
            .is_active(&decisions.registration_id)
            .expect("active")
    );
    assert_eq!(
        supervisor.last_failure(&decisions.registration_id),
        Ok(None)
    );

    tokio::runtime::Runtime::new()
        .expect("Decisions RLS runtime")
        .block_on(assert_review_owner_rls_v1(
            "makosh_decisions_rls_test",
            &[
                "decisions_records",
                "decisions_alternatives",
                "decisions_evidence_links",
                "decisions_client_operations",
                "decisions_outbox",
            ],
        ));
    supervisor.shutdown().expect("stop Decisions contour");
    unsafe { std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE") };
    std::fs::remove_dir_all(root).expect("remove Decisions fixture");
}

fn contract_v1(name: &str) -> ContractReferenceV1 {
    decisions_client_routes_v1()
        .into_iter()
        .find_map(|(contract, _)| (contract.name == name).then_some(contract))
        .expect("Decisions contract")
}

fn route_decisions_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    decisions: &StartedDecisionsRuntimeV1,
    request_id: u64,
    contract: ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let response =
        route_decisions_response_v1(store, supervisor, decisions, request_id, contract, payload);
    assert!(
        response.error_code.is_empty(),
        "Decisions request {request_id} failed: {}",
        response.error_code
    );
    T::decode(response.response_payload.as_slice()).expect("decode Decisions response")
}

fn route_decisions_response_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    decisions: &StartedDecisionsRuntimeV1,
    request_id: u64,
    contract: ContractReferenceV1,
    payload: Vec<u8>,
) -> ModuleClientResponseV1 {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: DECISIONS_MODULE_ID_V1.to_owned(),
        owner_id: DECISIONS_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: DECISIONS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &decisions.registration_id,
        &decisions.runtime_instance_id,
        decisions.runtime_generation,
        decisions.grant_epoch,
        DECISIONS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Decisions request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("Decisions response")
}

fn wait_for_decisions_relay_v1() {
    let deadline = Instant::now() + Duration::from_secs(15);
    while durable_decisions_snapshot_v1().5 != 0 {
        assert!(Instant::now() < deadline, "Decisions relay did not drain");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn durable_decisions_snapshot_v1() -> (i64, i64, i64, i64, i64, i64) {
    tokio::runtime::Runtime::new()
        .expect("Decisions SQL runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            sqlx::query_as(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.decisions_records WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.decisions_alternatives WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.decisions_evidence_links WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.decisions_client_operations WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.decisions_outbox WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.decisions_outbox WHERE logical_owner_id='owner-1' AND published_at_unix_millis IS NULL)",
            )
            .fetch_one(&pool)
            .await
            .expect("Decisions durable snapshot")
        })
}

fn assert_public_decisions_outbox_is_private_free_v1() {
    tokio::runtime::Runtime::new()
        .expect("Decisions privacy runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            let rows: Vec<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_bytes FROM makosh_data.decisions_outbox WHERE logical_owner_id='owner-1' ORDER BY outbox_sequence",
            )
            .fetch_all(&pool)
            .await
            .expect("Decisions outbox bytes");
            assert!(!rows.is_empty());
            for row in rows {
                for private in [
                    PRIVATE_TITLE_V1,
                    PRIVATE_QUESTION_V1,
                    PRIVATE_ALTERNATIVE_V1,
                    PRIVATE_RATIONALE_V1,
                    PRIVATE_EVIDENCE_OWNER_V1,
                ] {
                    assert!(
                        !row
                            .windows(private.len())
                            .any(|window| window == private.as_bytes()),
                        "private Decisions content escaped public outbox"
                    );
                }
            }
        });
}
