//! Actual Relationships lifecycle, replay, restart, privacy and owner-RLS contour.

use super::*;

use std::time::{Duration, Instant};

use makosh_relationships_api::{
    RELATIONSHIPS_CLIENT_CAPABILITY_ID_V1, RELATIONSHIPS_MODULE_ID_V1, RELATIONSHIPS_OWNER_ID_V1,
    client_wire::{
        AddRelationshipEvidenceRequestV1, CreateRelationshipRequestV1, EndRelationshipRequestV1,
        GetRelationshipRequestV1, ListRelationshipEvidenceRequestV1,
        ListRelationshipEvidenceResultV1, ListRelationshipsForParticipantRequestV1,
        ListRelationshipsResultV1, ReactivateRelationshipRequestV1, RelationshipMutationResultV1,
        RelationshipParticipantKindV1, RelationshipParticipantV1, RelationshipStateV1,
        RelationshipTypeV1, RelationshipV1, RemoveRelationshipEvidenceRequestV1, TimestampV1,
        UpdateRelationshipValidityRequestV1,
    },
    relationships_client_add_evidence_contract_reference_v1,
    relationships_client_create_contract_reference_v1,
    relationships_client_end_contract_reference_v1, relationships_client_get_contract_reference_v1,
    relationships_client_list_evidence_contract_reference_v1,
    relationships_client_list_for_participant_contract_reference_v1,
    relationships_client_reactivate_contract_reference_v1,
    relationships_client_remove_evidence_contract_reference_v1,
    relationships_client_update_validity_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_EVIDENCE_RECORD_V1: &str = "relationships-private-evidence-record";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Relationships binaries"]
fn managed_relationships_lifecycle_replays_and_restarts_with_owner_rls() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-relationships");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_relationships_release_v1(&root);
    unsafe { std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel()) };
    let store = Arc::new(configured_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Relationships owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            RELATIONSHIPS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Relationships owner");
    let admitted = admit_relationships_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Relationships Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_relationships_runtime_v1(&supervisor, &store, admitted);
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let relationships =
        start_relationships_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Relationships clock")
        .as_secs() as i64;
    let at = |offset: i64| TimestampV1 {
        unix_seconds: now - 10 + offset,
        nanos: 0,
    };
    let person = RelationshipParticipantV1 {
        kind: RelationshipParticipantKindV1::RelationshipParticipantKindPerson as i32,
        public_id: vec![0x21; 16],
    };
    let organization = RelationshipParticipantV1 {
        kind: RelationshipParticipantKindV1::RelationshipParticipantKindOrganization as i32,
        public_id: vec![0x31; 16],
    };
    let create = CreateRelationshipRequestV1 {
        operation_id: vec![0x11; 16],
        logical_owner_id: String::new(),
        source: Some(person.clone()),
        target: Some(organization.clone()),
        relationship_type: RelationshipTypeV1::RelationshipTypeMemberOf as i32,
        valid_from: Some(at(0)),
        valid_until: None,
        evidence_source_owner_id: "organizations".to_owned(),
        evidence_source_record_id: PRIVATE_EVIDENCE_RECORD_V1.to_owned(),
        evidence_source_revision: 1,
        evidence_digest: vec![0x41; 32],
        evidence_observed_at: Some(at(0)),
        created_at: Some(at(1)),
    };
    let first: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        1,
        relationships_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    let replay: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        2,
        relationships_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    assert_eq!(first, replay);
    let created = first.relationship.expect("created relationship");
    assert_eq!(created.relationship_revision, 1);

    let mut changed = create.clone();
    changed.evidence_digest = vec![0x42; 32];
    assert_eq!(
        route_relationships_response_v1(
            &store,
            &supervisor,
            &relationships,
            3,
            relationships_client_create_contract_reference_v1(),
            changed.encode_to_vec(),
        )
        .error_code,
        "CONFLICT"
    );

    let added: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        4,
        relationships_client_add_evidence_contract_reference_v1(),
        AddRelationshipEvidenceRequestV1 {
            operation_id: vec![0x12; 16],
            relationship_id: created.relationship_id.clone(),
            logical_owner_id: String::new(),
            expected_relationship_revision: 1,
            source_owner_id: "persons".to_owned(),
            source_record_id: "public-person-source".to_owned(),
            source_revision: 2,
            evidence_digest: vec![0x51; 32],
            observed_at: Some(at(1)),
            changed_at: Some(at(2)),
        }
        .encode_to_vec(),
    );
    assert_eq!(added.relationship.expect("added").relationship_revision, 2);

    let validity: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        5,
        relationships_client_update_validity_contract_reference_v1(),
        UpdateRelationshipValidityRequestV1 {
            operation_id: vec![0x13; 16],
            relationship_id: created.relationship_id.clone(),
            logical_owner_id: String::new(),
            expected_relationship_revision: 2,
            valid_from: Some(at(0)),
            valid_until: Some(at(8)),
            changed_at: Some(at(3)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        validity
            .relationship
            .expect("validity")
            .relationship_revision,
        3
    );
    let ended: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        6,
        relationships_client_end_contract_reference_v1(),
        EndRelationshipRequestV1 {
            operation_id: vec![0x14; 16],
            relationship_id: created.relationship_id.clone(),
            logical_owner_id: String::new(),
            expected_relationship_revision: 3,
            valid_until: Some(at(4)),
            changed_at: Some(at(4)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        ended.relationship.expect("ended").state,
        RelationshipStateV1::RelationshipStateEnded as i32
    );
    let reactivated: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        7,
        relationships_client_reactivate_contract_reference_v1(),
        ReactivateRelationshipRequestV1 {
            operation_id: vec![0x15; 16],
            relationship_id: created.relationship_id.clone(),
            logical_owner_id: String::new(),
            expected_relationship_revision: 4,
            valid_from: Some(at(5)),
            valid_until: None,
            changed_at: Some(at(5)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        reactivated.relationship.expect("reactivated").state,
        RelationshipStateV1::RelationshipStateConfirmed as i32
    );
    let evidence: ListRelationshipEvidenceResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        8,
        relationships_client_list_evidence_contract_reference_v1(),
        ListRelationshipEvidenceRequestV1 {
            logical_owner_id: String::new(),
            relationship_id: created.relationship_id.clone(),
            after_evidence_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(evidence.evidence.len(), 2);
    let removed: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        9,
        relationships_client_remove_evidence_contract_reference_v1(),
        RemoveRelationshipEvidenceRequestV1 {
            operation_id: vec![0x16; 16],
            relationship_id: created.relationship_id.clone(),
            logical_owner_id: String::new(),
            expected_relationship_revision: 5,
            evidence_id: evidence.evidence[1].evidence_id.clone(),
            changed_at: Some(at(6)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        removed.relationship.expect("removed").relationship_revision,
        6
    );

    let second: RelationshipMutationResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        10,
        relationships_client_create_contract_reference_v1(),
        CreateRelationshipRequestV1 {
            operation_id: vec![0x17; 16],
            logical_owner_id: String::new(),
            source: Some(person.clone()),
            target: Some(RelationshipParticipantV1 {
                kind: RelationshipParticipantKindV1::RelationshipParticipantKindPerson as i32,
                public_id: vec![0x61; 16],
            }),
            relationship_type: RelationshipTypeV1::RelationshipTypeFriend as i32,
            valid_from: Some(at(0)),
            valid_until: None,
            evidence_source_owner_id: "persons".to_owned(),
            evidence_source_record_id: "public-friend-source".to_owned(),
            evidence_source_revision: 1,
            evidence_digest: vec![0x62; 32],
            evidence_observed_at: Some(at(0)),
            created_at: Some(at(1)),
        }
        .encode_to_vec(),
    );
    let second_id = second.relationship.expect("second").relationship_id;
    let page1: ListRelationshipsResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        11,
        relationships_client_list_for_participant_contract_reference_v1(),
        ListRelationshipsForParticipantRequestV1 {
            logical_owner_id: String::new(),
            participant: Some(person.clone()),
            after_relationship_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let page2: ListRelationshipsResultV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        12,
        relationships_client_list_for_participant_contract_reference_v1(),
        ListRelationshipsForParticipantRequestV1 {
            logical_owner_id: String::new(),
            participant: Some(person),
            after_relationship_id: page1.next_after_relationship_id.clone(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let mut ids = vec![
        page1.relationships[0].relationship_id.clone(),
        page2.relationships[0].relationship_id.clone(),
    ];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&created.relationship_id));
    assert!(ids.contains(&second_id));

    wait_for_relationships_relay_v1();
    let before_restart = durable_relationships_snapshot_v1();
    assert_eq!(before_restart, (2, 3, 7, 7, 0));
    assert_public_relationships_outbox_is_private_free_v1();
    let relationships =
        restart_relationships_runtime_v1(&supervisor, &store, &root.join("runtime"), relationships);
    let restarted: RelationshipV1 = route_relationships_v1(
        &store,
        &supervisor,
        &relationships,
        13,
        relationships_client_get_contract_reference_v1(),
        GetRelationshipRequestV1 {
            logical_owner_id: String::new(),
            relationship_id: created.relationship_id,
        }
        .encode_to_vec(),
    );
    assert_eq!(restarted.relationship_revision, 6);
    assert_eq!(durable_relationships_snapshot_v1(), before_restart);
    assert!(
        supervisor
            .is_active(&relationships.registration_id)
            .expect("active")
    );
    assert_eq!(
        supervisor.last_failure(&relationships.registration_id),
        Ok(None)
    );

    tokio::runtime::Runtime::new()
        .expect("Relationships RLS runtime")
        .block_on(assert_review_owner_rls_v1(
            "makosh_relationships_rls_test",
            &[
                "relationships_records",
                "relationships_evidence",
                "relationships_client_operations",
                "relationships_outbox",
            ],
        ));
    supervisor.shutdown().expect("stop Relationships contour");
    unsafe { std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE") };
    std::fs::remove_dir_all(root).expect("remove Relationships fixture");
}

fn route_relationships_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    relationships: &StartedRelationshipsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let response = route_relationships_response_v1(
        store,
        supervisor,
        relationships,
        request_id,
        contract,
        payload,
    );
    assert!(
        response.error_code.is_empty(),
        "Relationships request {request_id} failed: {}",
        response.error_code
    );
    T::decode(response.response_payload.as_slice()).expect("decode Relationships response")
}

fn route_relationships_response_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    relationships: &StartedRelationshipsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> ModuleClientResponseV1 {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: RELATIONSHIPS_MODULE_ID_V1.to_owned(),
        owner_id: RELATIONSHIPS_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: RELATIONSHIPS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &relationships.registration_id,
        &relationships.runtime_instance_id,
        relationships.runtime_generation,
        relationships.grant_epoch,
        RELATIONSHIPS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Relationships request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("Relationships response")
}

fn wait_for_relationships_relay_v1() {
    let deadline = Instant::now() + Duration::from_secs(15);
    while durable_relationships_snapshot_v1().4 != 0 {
        assert!(
            Instant::now() < deadline,
            "Relationships relay did not drain"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn durable_relationships_snapshot_v1() -> (i64, i64, i64, i64, i64) {
    tokio::runtime::Runtime::new()
        .expect("Relationships SQL runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            sqlx::query_as(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.relationships_records WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.relationships_evidence WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.relationships_client_operations WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.relationships_outbox WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.relationships_outbox WHERE logical_owner_id='owner-1' AND published_at_unix_millis IS NULL)",
            )
            .fetch_one(&pool)
            .await
            .expect("Relationships durable snapshot")
        })
}

fn assert_public_relationships_outbox_is_private_free_v1() {
    tokio::runtime::Runtime::new()
        .expect("Relationships privacy runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            let rows: Vec<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_bytes FROM makosh_data.relationships_outbox \
                 WHERE logical_owner_id='owner-1' ORDER BY outbox_sequence",
            )
            .fetch_all(&pool)
            .await
            .expect("Relationships outbox bytes");
            assert!(!rows.is_empty());
            for row in rows {
                assert!(
                    !row.windows(PRIVATE_EVIDENCE_RECORD_V1.len())
                        .any(|window| window == PRIVATE_EVIDENCE_RECORD_V1.as_bytes()),
                    "private Relationships evidence escaped public outbox"
                );
            }
        });
}
