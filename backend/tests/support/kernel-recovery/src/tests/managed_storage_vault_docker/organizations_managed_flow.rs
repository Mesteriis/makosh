//! Actual Organizations lifecycle, replay, restart, privacy and owner-RLS contour.

use super::*;

use std::time::{Duration, Instant};

use makosh_organizations_api::{
    ORGANIZATIONS_CLIENT_CAPABILITY_ID_V1, ORGANIZATIONS_MODULE_ID_V1, ORGANIZATIONS_OWNER_ID_V1,
    client_wire::{
        AddOrganizationSourceRequestV1, CreateOrganizationRequestV1, GetOrganizationRequestV1,
        ListOrganizationSourcesRequestV1, ListOrganizationSourcesResultV1,
        ListOrganizationsRequestV1, ListOrganizationsResultV1, OrganizationMutationResultV1,
        OrganizationSourceStateV1, OrganizationStateV1, OrganizationV1,
        RemoveOrganizationSourceRequestV1, SearchOrganizationsRequestV1,
        SetOrganizationStateRequestV1, TimestampV1, UpdateOrganizationRequestV1,
    },
    organizations_client_add_source_contract_reference_v1,
    organizations_client_create_contract_reference_v1,
    organizations_client_get_contract_reference_v1,
    organizations_client_list_contract_reference_v1,
    organizations_client_list_sources_contract_reference_v1,
    organizations_client_remove_source_contract_reference_v1,
    organizations_client_search_contract_reference_v1,
    organizations_client_set_state_contract_reference_v1,
    organizations_client_update_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_DISPLAY_V1: &str = "organizations-private-display-marker";
const PRIVATE_LEGAL_V1: &str = "organizations-private-legal-marker";
const PRIVATE_DESCRIPTION_V1: &str = "organizations-private-description-marker";
const PRIVATE_SOURCE_V1: &str = "organizations-private-source-record";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Organizations binaries"]
fn managed_organizations_lifecycle_replays_and_restarts_with_owner_rls() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-organizations");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_organizations_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Organizations owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ORGANIZATIONS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Organizations owner");
    let admitted = admit_organizations_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Organizations Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_organizations_runtime_v1(&supervisor, &store, admitted);
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let organizations =
        start_organizations_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Organizations clock")
        .as_millis() as i64
        - 1_000;
    let timestamp = TimestampV1 {
        unix_seconds: now / 1_000,
        nanos: ((now % 1_000) * 1_000_000) as i32,
    };
    let create = CreateOrganizationRequestV1 {
        operation_id: vec![0x11; 16],
        logical_owner_id: String::new(),
        display_name: PRIVATE_DISPLAY_V1.to_owned(),
        legal_name: PRIVATE_LEGAL_V1.to_owned(),
        description: PRIVATE_DESCRIPTION_V1.to_owned(),
        website: "https://example.invalid".to_owned(),
        industry: "Software".to_owned(),
        country_code: "ES".to_owned(),
        created_at: Some(timestamp),
    };
    let first: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        1,
        organizations_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    let replayed: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        2,
        organizations_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    assert_eq!(first, replayed, "exact create replay response");
    let first_organization = first.organization.expect("created Organization");
    assert_eq!(first_organization.organization_revision, 1);

    let mut altered = create.clone();
    altered.display_name = "altered replay".to_owned();
    assert_eq!(
        route_organizations_response_v1(
            &store,
            &supervisor,
            &organizations,
            3,
            organizations_client_create_contract_reference_v1(),
            altered.encode_to_vec(),
        )
        .error_code,
        "CONFLICT",
    );
    let updated: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        4,
        organizations_client_update_contract_reference_v1(),
        UpdateOrganizationRequestV1 {
            operation_id: vec![0x12; 16],
            organization_id: first_organization.organization_id.clone(),
            logical_owner_id: String::new(),
            expected_organization_revision: 1,
            display_name: Some("Updated organization".to_owned()),
            legal_name: None,
            description: None,
            website: None,
            industry: None,
            country_code: None,
            updated_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        updated
            .organization
            .as_ref()
            .expect("updated")
            .organization_revision,
        2
    );
    let with_source: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        5,
        organizations_client_add_source_contract_reference_v1(),
        AddOrganizationSourceRequestV1 {
            operation_id: vec![0x13; 16],
            organization_id: first_organization.organization_id.clone(),
            logical_owner_id: String::new(),
            expected_organization_revision: 2,
            source_owner_id: "knowledge".to_owned(),
            source_record_id: PRIVATE_SOURCE_V1.to_owned(),
            source_revision: 1,
            evidence_digest: vec![0x31; 32],
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        with_source
            .organization
            .as_ref()
            .expect("source")
            .organization_revision,
        3
    );
    let sources: ListOrganizationSourcesResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        6,
        organizations_client_list_sources_contract_reference_v1(),
        ListOrganizationSourcesRequestV1 {
            logical_owner_id: String::new(),
            organization_id: first_organization.organization_id.clone(),
            after_source_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(sources.sources.len(), 1);
    let source_id = sources.sources[0].source_id.clone();
    let removed: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        7,
        organizations_client_remove_source_contract_reference_v1(),
        RemoveOrganizationSourceRequestV1 {
            operation_id: vec![0x14; 16],
            organization_id: first_organization.organization_id.clone(),
            logical_owner_id: String::new(),
            expected_organization_revision: 3,
            source_id,
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        removed
            .organization
            .as_ref()
            .expect("removed")
            .organization_revision,
        4
    );
    let archived: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        8,
        organizations_client_set_state_contract_reference_v1(),
        SetOrganizationStateRequestV1 {
            operation_id: vec![0x15; 16],
            organization_id: first_organization.organization_id.clone(),
            logical_owner_id: String::new(),
            expected_organization_revision: 4,
            state: OrganizationStateV1::OrganizationStateArchived as i32,
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        archived
            .organization
            .as_ref()
            .expect("archived")
            .organization_revision,
        5
    );
    let restored: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        9,
        organizations_client_set_state_contract_reference_v1(),
        SetOrganizationStateRequestV1 {
            operation_id: vec![0x16; 16],
            organization_id: first_organization.organization_id.clone(),
            logical_owner_id: String::new(),
            expected_organization_revision: 5,
            state: OrganizationStateV1::OrganizationStateActive as i32,
            changed_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        restored
            .organization
            .as_ref()
            .expect("restored")
            .organization_revision,
        6
    );

    let second: OrganizationMutationResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        10,
        organizations_client_create_contract_reference_v1(),
        CreateOrganizationRequestV1 {
            operation_id: vec![0x17; 16],
            logical_owner_id: String::new(),
            display_name: "Second organization".to_owned(),
            legal_name: String::new(),
            description: String::new(),
            website: String::new(),
            industry: String::new(),
            country_code: String::new(),
            created_at: Some(timestamp),
        }
        .encode_to_vec(),
    );
    let second_id = second
        .organization
        .expect("second Organization")
        .organization_id;
    let first_page: ListOrganizationsResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        11,
        organizations_client_list_contract_reference_v1(),
        ListOrganizationsRequestV1 {
            logical_owner_id: String::new(),
            after_organization_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let second_page: ListOrganizationsResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        12,
        organizations_client_list_contract_reference_v1(),
        ListOrganizationsRequestV1 {
            logical_owner_id: String::new(),
            after_organization_id: first_page.next_after_organization_id.clone(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let mut ids = vec![
        first_page.organizations[0].organization_id.clone(),
        second_page.organizations[0].organization_id.clone(),
    ];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&first_organization.organization_id));
    assert!(ids.contains(&second_id));
    let searched: ListOrganizationsResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        13,
        organizations_client_search_contract_reference_v1(),
        SearchOrganizationsRequestV1 {
            logical_owner_id: String::new(),
            query: "private-legal".to_owned(),
            after_organization_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(searched.organizations.len(), 1);

    wait_for_organizations_relay_v1();
    let before_restart = durable_organizations_snapshot_v1();
    assert_eq!(before_restart, (2, 1, 7, 7, 0));
    assert_public_organizations_outbox_is_private_free_v1();
    let organizations =
        restart_organizations_runtime_v1(&supervisor, &store, &root.join("runtime"), organizations);
    let restarted: OrganizationV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        14,
        organizations_client_get_contract_reference_v1(),
        GetOrganizationRequestV1 {
            logical_owner_id: String::new(),
            organization_id: first_organization.organization_id,
        }
        .encode_to_vec(),
    );
    assert_eq!(restarted.organization_revision, 6);
    let restarted_sources: ListOrganizationSourcesResultV1 = route_organizations_v1(
        &store,
        &supervisor,
        &organizations,
        15,
        organizations_client_list_sources_contract_reference_v1(),
        ListOrganizationSourcesRequestV1 {
            logical_owner_id: String::new(),
            organization_id: restarted.organization_id.clone(),
            after_source_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(
        restarted_sources.sources[0].state,
        OrganizationSourceStateV1::OrganizationSourceStateRemoved as i32
    );
    assert_eq!(durable_organizations_snapshot_v1(), before_restart);
    assert!(
        supervisor
            .is_active(&organizations.registration_id)
            .expect("Organizations active")
    );
    assert_eq!(
        supervisor.last_failure(&organizations.registration_id),
        Ok(None)
    );

    tokio::runtime::Runtime::new()
        .expect("Organizations RLS runtime")
        .block_on(assert_review_owner_rls_v1(
            "makosh_organizations_rls_test",
            &[
                "organizations_records",
                "organizations_sources",
                "organizations_client_operations",
                "organizations_outbox",
            ],
        ));
    supervisor.shutdown().expect("stop Organizations contour");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Organizations fixture");
}

fn route_organizations_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    organizations: &StartedOrganizationsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let response = route_organizations_response_v1(
        store,
        supervisor,
        organizations,
        request_id,
        contract,
        payload,
    );
    assert!(
        response.error_code.is_empty(),
        "Organizations request {request_id} failed: {}",
        response.error_code
    );
    T::decode(response.response_payload.as_slice()).expect("decode Organizations response")
}

fn route_organizations_response_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    organizations: &StartedOrganizationsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> ModuleClientResponseV1 {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: ORGANIZATIONS_MODULE_ID_V1.to_owned(),
        owner_id: ORGANIZATIONS_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: ORGANIZATIONS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &organizations.registration_id,
        &organizations.runtime_instance_id,
        organizations.runtime_generation,
        organizations.grant_epoch,
        ORGANIZATIONS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Organizations request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("Organizations response")
}

fn wait_for_organizations_relay_v1() {
    let deadline = Instant::now() + Duration::from_secs(15);
    while durable_organizations_snapshot_v1().4 != 0 {
        assert!(
            Instant::now() < deadline,
            "Organizations relay did not drain"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn durable_organizations_snapshot_v1() -> (i64, i64, i64, i64, i64) {
    tokio::runtime::Runtime::new().expect("Organizations SQL runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        sqlx::query_as(
            "SELECT \
             (SELECT count(*) FROM makosh_data.organizations_records WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.organizations_sources WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.organizations_client_operations WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.organizations_outbox WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.organizations_outbox WHERE logical_owner_id='owner-1' AND published_at_unix_millis IS NULL)",
        ).fetch_one(&pool).await.expect("Organizations durable snapshot")
    })
}

fn assert_public_organizations_outbox_is_private_free_v1() {
    tokio::runtime::Runtime::new().expect("Organizations privacy runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        let rows: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT envelope_bytes FROM makosh_data.organizations_outbox WHERE logical_owner_id='owner-1' ORDER BY outbox_sequence",
        ).fetch_all(&pool).await.expect("Organizations outbox bytes");
        assert!(!rows.is_empty());
        for row in rows {
            for marker in [PRIVATE_DISPLAY_V1, PRIVATE_LEGAL_V1, PRIVATE_DESCRIPTION_V1, PRIVATE_SOURCE_V1] {
                assert!(!row.windows(marker.len()).any(|window| window == marker.as_bytes()), "private Organizations marker escaped public outbox");
            }
        }
    });
}
