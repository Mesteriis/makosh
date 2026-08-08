use std::sync::{Arc, Mutex};

use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, InitialOwnerIdentity, ManagedLaunchRecord,
    ModuleDescriptorRegistrationRequestsV1, ModuleQueryContractV1, ModuleRegistration,
    ModuleRegistrationState, ModuleRequestContractV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, ContractReferenceV1,
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestRequestV1, ManagedRuntimeModuleRequestResponseV1,
    ModuleDescriptorV1, ModuleKindV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    managed_runtime_control_request_v1, managed_runtime_control_response_v1,
};
use prost::Message;

use crate::modules::capability::module_request::ModuleRequestRouteHandlerV1;
use crate::modules::registration::registry;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeExpectation, ManagedRuntimeModuleRequestHandler,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

use super::common::unique_target_root;

const OWNER: &str = "owner_notes";
const PROVIDER_CAPABILITY: &str = "notes.query";
const CALLER_CAPABILITY: &str = "notes.compose";

#[test]
fn request_providers_are_approval_gated_and_dependencies_remain_capability_scoped() {
    let root = unique_target_root("makosh-module-request-route");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(OWNER, "device_notes", [4; 65]))
        .expect("claim initial owner");

    let registration = registry::register(&store, &descriptor(OWNER).encode_to_vec())
        .expect("register request contracts");
    assert!(
        store
            .approved_module_request_rpc_routes()
            .expect("read pending routes")
            .is_empty()
    );
    assert_eq!(
        store
            .module_contract_dependencies(registration.registration_id(), CALLER_CAPABILITY)
            .expect("read caller dependency"),
        vec![dependency_record(
            registration.registration_id(),
            CALLER_CAPABILITY
        )]
    );

    store
        .approve_module_registration(
            registration.registration_id(),
            &[CALLER_CAPABILITY.to_owned(), PROVIDER_CAPABILITY.to_owned()],
        )
        .expect("approve request capabilities");
    assert_eq!(
        store
            .approved_module_request_rpc_routes()
            .expect("read approved route"),
        vec![contract_record(
            registration.registration_id(),
            PROVIDER_CAPABILITY
        )]
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn only_an_integration_may_implement_a_foreign_owned_request_contract() {
    let root = unique_target_root("makosh-module-request-foreign-owner");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(OWNER, "device_notes", [4; 65]))
        .expect("claim initial owner");

    assert!(
        registry::register(&store, &descriptor("owner_other").encode_to_vec()).is_err(),
        "a domain cannot claim another owner request contract",
    );
    let mut integration = descriptor("owner_other");
    integration.module_kind = ModuleKindV1::Integration as i32;
    let registration = registry::register(&store, &integration.encode_to_vec())
        .expect("register integration implementation of exact request port");
    assert!(
        store
            .approved_module_request_rpc_routes()
            .expect("read pending routes")
            .is_empty()
    );
    store
        .approve_module_registration(
            registration.registration_id(),
            &[CALLER_CAPABILITY.to_owned(), PROVIDER_CAPABILITY.to_owned()],
        )
        .expect("approve exact integration implementation capability");
    assert_eq!(
        store
            .approved_module_request_rpc_routes()
            .expect("read approved integration route"),
        vec![ModuleRequestContractV1::new(
            registration.registration_id(),
            PROVIDER_CAPABILITY,
            "owner_other",
            "notes.command.submit",
            1,
            1,
            [7; 32],
        )]
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn kernel_routes_an_exact_dependency_without_exposing_module_coordinates() {
    let fixture = RequestRouteFixture::new("makosh-module-request-success", 1);
    let relay = RequestRelay::success(&fixture.provider_id);
    let handler = ModuleRequestRouteHandlerV1::new(Arc::clone(&fixture.store), relay);

    let response = handler
        .route_module_request(&fixture.caller_expectation, request_request())
        .expect("route exact request dependency");
    assert_eq!(response.response_payload, vec![9]);
    assert_eq!(response.error_code, "");
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

#[test]
fn kernel_rejects_zero_or_ambiguous_request_providers_before_relay() {
    let zero = RequestRouteFixture::new("makosh-module-request-zero", 0);
    let zero_error =
        ModuleRequestRouteHandlerV1::new(Arc::clone(&zero.store), RequestRelay::unreachable())
            .route_module_request(&zero.caller_expectation, request_request())
            .expect_err("missing provider");
    assert_eq!(zero_error, "managed module request provider is unavailable");
    std::fs::remove_dir_all(zero.root).expect("remove zero fixture");

    let ambiguous = RequestRouteFixture::new("makosh-module-request-ambiguous", 2);
    let ambiguous_error =
        ModuleRequestRouteHandlerV1::new(Arc::clone(&ambiguous.store), RequestRelay::unreachable())
            .route_module_request(&ambiguous.caller_expectation, request_request())
            .expect_err("ambiguous provider");
    assert_eq!(
        ambiguous_error,
        "managed module request provider is ambiguous"
    );
    std::fs::remove_dir_all(ambiguous.root).expect("remove ambiguous fixture");
}

#[test]
fn kernel_does_not_route_a_request_to_a_query_only_provider() {
    let fixture = RequestRouteFixture::new("makosh-module-request-query-only", 0);
    let provider = registration(
        "communications-query-only",
        "communications_query_module",
        "communications",
        [2; 32],
    );
    let query_route = ModuleQueryContractV1::new(
        provider.registration_id(),
        PROVIDER_CAPABILITY,
        "communications",
        "communications.delivery-intent.submit",
        1,
        1,
        [7; 32],
    );
    fixture
        .store
        .create_pending_registration_with_all_descriptor_requests(
            &provider,
            &[PROVIDER_CAPABILITY.to_owned()],
            ModuleDescriptorRegistrationRequestsV1 {
                storage: &[],
                events: &[],
                blobs: &[],
                scheduler: &[],
                vault_purposes: &[],
                client_rpc_routes: &[],
                client_blob_routes: &[],
                client_realtime_routes: &[],
                query_rpc_routes: std::slice::from_ref(&query_route),
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("register query-only provider");
    let grants = fixture
        .store
        .approve_module_registration(
            provider.registration_id(),
            &[PROVIDER_CAPABILITY.to_owned()],
        )
        .expect("approve query-only provider");
    record_launch(
        &fixture.store,
        &provider,
        "query-only-runtime",
        grants.grant_epoch(),
    );

    let error =
        ModuleRequestRouteHandlerV1::new(Arc::clone(&fixture.store), RequestRelay::unreachable())
            .route_module_request(&fixture.caller_expectation, request_request())
            .expect_err("query-only provider must not serve request RPC");
    assert_eq!(error, "managed module request provider is unavailable");
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

#[test]
fn kernel_rejects_stale_caller_and_provider_fences_before_relay() {
    let stale_caller = RequestRouteFixture::new("makosh-module-request-stale-caller", 1);
    let stale_expectation = ManagedRuntimeExpectation::new(
        stale_caller.caller_id.clone(),
        "caller-runtime",
        "workflow_module",
        2,
        stale_caller.caller_expectation.grant_epoch(),
        [1; 32],
        None,
    );
    let caller_error = ModuleRequestRouteHandlerV1::new(
        Arc::clone(&stale_caller.store),
        RequestRelay::unreachable(),
    )
    .route_module_request(&stale_expectation, request_request())
    .expect_err("stale caller");
    assert_eq!(caller_error, "managed module request caller fence is stale");
    std::fs::remove_dir_all(stale_caller.root).expect("remove caller fixture");

    let stale_provider = RequestRouteFixture::new("makosh-module-request-stale-provider", 1);
    stale_provider
        .store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            &stale_provider.provider_id,
            2,
            "distribution-communications-v2",
            "communications-runtime-v2",
            [8; 32],
            [2; 32],
            None,
        ))
        .expect("replace provider binding");
    let provider_error = ModuleRequestRouteHandlerV1::new(
        Arc::clone(&stale_provider.store),
        RequestRelay::unreachable(),
    )
    .route_module_request(&stale_provider.caller_expectation, request_request())
    .expect_err("stale provider");
    assert_eq!(
        provider_error,
        "managed module request provider fence is stale"
    );
    std::fs::remove_dir_all(stale_provider.root).expect("remove provider fixture");
}

#[test]
fn kernel_rejects_provider_response_mismatch() {
    let fixture = RequestRouteFixture::new("makosh-module-request-response-mismatch", 1);
    let error = ModuleRequestRouteHandlerV1::new(
        Arc::clone(&fixture.store),
        RequestRelay::mismatch(&fixture.provider_id),
    )
    .route_module_request(&fixture.caller_expectation, request_request())
    .expect_err("response mismatch");
    assert_eq!(
        error,
        "managed module request provider response does not match request"
    );
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

#[test]
fn kernel_rejects_a_revoked_request_provider_before_relay() {
    let fixture = RequestRouteFixture::new("makosh-module-request-revoked-provider", 1);
    fixture
        .store
        .transition_module_registration(&fixture.provider_id, ModuleRegistrationState::Revoked)
        .expect("revoke provider");
    let error =
        ModuleRequestRouteHandlerV1::new(Arc::clone(&fixture.store), RequestRelay::unreachable())
            .route_module_request(&fixture.caller_expectation, request_request())
            .expect_err("revoked provider");
    assert_eq!(error, "managed module request provider is unavailable");
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

fn descriptor(provider_owner: &str) -> ModuleDescriptorV1 {
    let contract = ContractReferenceV1 {
        owner: provider_owner.to_owned(),
        name: "notes.command.submit".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: vec![7; 32],
    };
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: OWNER.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![
            CapabilityDescriptorV1 {
                capability_id: CALLER_CAPABILITY.to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                dependencies: vec![contract.clone()],
                ..Default::default()
            },
            CapabilityDescriptorV1 {
                capability_id: PROVIDER_CAPABILITY.to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                provides: vec![ProvidedSurfaceV1 {
                    kind: ProvidedSurfaceKindV1::RequestRpc as i32,
                    contract: Some(contract.clone()),
                    client_rpc_route: None,
                    client_blob_route: None,
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn contract_record(registration_id: &str, capability_id: &str) -> ModuleRequestContractV1 {
    ModuleRequestContractV1::new(
        registration_id,
        capability_id,
        OWNER,
        "notes.command.submit",
        1,
        1,
        [7; 32],
    )
}

fn dependency_record(registration_id: &str, capability_id: &str) -> ModuleQueryContractV1 {
    ModuleQueryContractV1::new(
        registration_id,
        capability_id,
        OWNER,
        "notes.command.submit",
        1,
        1,
        [7; 32],
    )
}

struct RequestRouteFixture {
    root: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    caller_id: String,
    provider_id: String,
    caller_expectation: ManagedRuntimeExpectation,
}

impl RequestRouteFixture {
    fn new(prefix: &str, provider_count: usize) -> Self {
        let root = unique_target_root(prefix);
        std::fs::create_dir_all(&root).expect("create fixture directory");
        let store = Arc::new(
            SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
                .expect("create Control Store"),
        );
        store
            .claim_initial_owner(&InitialOwnerIdentity::new(
                "owner_local",
                "device_local",
                [4; 65],
            ))
            .expect("claim initial owner");

        let caller = registration(
            "delivery-intent",
            "workflow_module",
            "communication_delivery_intent",
            [1; 32],
        );
        let dependency = ModuleQueryContractV1::new(
            caller.registration_id(),
            CALLER_CAPABILITY,
            "communications",
            "communications.delivery-intent.submit",
            1,
            1,
            [7; 32],
        );
        register_with_contracts(
            &store,
            &caller,
            CALLER_CAPABILITY,
            &[],
            std::slice::from_ref(&dependency),
        );
        let caller_grants = store
            .approve_module_registration(caller.registration_id(), &[CALLER_CAPABILITY.to_owned()])
            .expect("approve caller");
        record_launch(
            &store,
            &caller,
            "caller-runtime",
            caller_grants.grant_epoch(),
        );
        let caller_expectation = ManagedRuntimeExpectation::new(
            caller.registration_id(),
            "caller-runtime",
            caller.module_id(),
            1,
            caller_grants.grant_epoch(),
            *caller.descriptor_sha256(),
            None,
        );

        let mut provider_id = String::new();
        for index in 0..provider_count {
            let registration_id = format!("communications-{index}");
            let provider = registration(
                &registration_id,
                &format!("communications_module_{index}"),
                "communications",
                [2; 32],
            );
            let route = ModuleRequestContractV1::new(
                provider.registration_id(),
                PROVIDER_CAPABILITY,
                "communications",
                "communications.delivery-intent.submit",
                1,
                1,
                [7; 32],
            );
            register_with_contracts(
                &store,
                &provider,
                PROVIDER_CAPABILITY,
                std::slice::from_ref(&route),
                &[],
            );
            let grants = store
                .approve_module_registration(
                    provider.registration_id(),
                    &[PROVIDER_CAPABILITY.to_owned()],
                )
                .expect("approve provider");
            record_launch(
                &store,
                &provider,
                &format!("provider-runtime-{index}"),
                grants.grant_epoch(),
            );
            if index == 0 {
                provider_id = registration_id;
            }
        }

        Self {
            root,
            store,
            caller_id: caller.registration_id().to_owned(),
            provider_id,
            caller_expectation,
        }
    }
}

fn registration(
    registration_id: &str,
    module_id: &str,
    owner_id: &str,
    descriptor_sha256: [u8; 32],
) -> ModuleRegistration {
    ModuleRegistration::new(
        registration_id,
        module_id,
        owner_id,
        descriptor_sha256,
        ModuleRegistrationState::Pending,
        1,
    )
}

fn register_with_contracts(
    store: &SqliteControlStore,
    registration: &ModuleRegistration,
    capability_id: &str,
    request_routes: &[ModuleRequestContractV1],
    dependencies: &[ModuleQueryContractV1],
) {
    store
        .create_pending_registration_with_all_descriptor_requests(
            registration,
            &[capability_id.to_owned()],
            ModuleDescriptorRegistrationRequestsV1 {
                storage: &[],
                events: &[],
                blobs: &[],
                scheduler: &[],
                vault_purposes: &[],
                client_rpc_routes: &[],
                client_blob_routes: &[],
                client_realtime_routes: &[],
                query_rpc_routes: &[],
                request_rpc_routes: request_routes,
                contract_dependencies: dependencies,
            },
        )
        .expect("register request participant");
}

fn record_launch(
    store: &SqliteControlStore,
    registration: &ModuleRegistration,
    runtime_instance_id: &str,
    grant_epoch: u64,
) {
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            format!("distribution-{}", registration.registration_id()),
            format!("runtime-{}", registration.registration_id()),
            [8; 32],
            *registration.descriptor_sha256(),
            None,
        ))
        .expect("record launch binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            registration.registration_id(),
            runtime_instance_id,
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record launch");
}

fn request_request() -> ManagedRuntimeModuleRequestRequestV1 {
    ManagedRuntimeModuleRequestRequestV1 {
        request_id: vec![1; 16],
        contract: Some(ContractReferenceV1 {
            owner: "communications".to_owned(),
            name: "communications.delivery-intent.submit".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }),
        request_payload: vec![3],
        deadline_millis: 1_000,
        response_blob_capability_id: String::new(),
    }
}

struct RequestRelay {
    expected_registration_id: Option<String>,
    mismatch: bool,
    calls: Mutex<usize>,
}

impl RequestRelay {
    fn success(registration_id: &str) -> Self {
        Self {
            expected_registration_id: Some(registration_id.to_owned()),
            mismatch: false,
            calls: Mutex::new(0),
        }
    }

    fn mismatch(registration_id: &str) -> Self {
        Self {
            expected_registration_id: Some(registration_id.to_owned()),
            mismatch: true,
            calls: Mutex::new(0),
        }
    }

    fn unreachable() -> Self {
        Self {
            expected_registration_id: None,
            mismatch: false,
            calls: Mutex::new(0),
        }
    }
}

impl ManagedRuntimeRelay for RequestRelay {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        let expected = self
            .expected_registration_id
            .as_deref()
            .ok_or_else(|| "managed runtime relay was reached".to_owned())?;
        assert_eq!(registration_id, expected);
        let request =
            ManagedRuntimeControlRequestV1::decode(payload.as_slice()).expect("decode delivery");
        let delivery = match request.operation.expect("delivery operation") {
            managed_runtime_control_request_v1::Operation::DeliverModuleRequest(delivery) => {
                delivery
            }
            _ => panic!("unexpected relay operation"),
        };
        assert_eq!(delivery.logical_owner_id, "owner_local");
        *self.calls.lock().expect("relay calls") += 1;
        let request_id = if self.mismatch {
            vec![2; 16]
        } else {
            delivery.request_id
        };
        Ok(ManagedRuntimeControlResponseV1 {
            result: Some(
                managed_runtime_control_response_v1::Result::ModuleRequestDelivery(
                    ManagedRuntimeModuleRequestResponseV1 {
                        request_id,
                        response_payload: vec![9],
                        error_code: String::new(),
                    },
                ),
            ),
            error_code: String::new(),
        }
        .encode_to_vec())
    }
}
