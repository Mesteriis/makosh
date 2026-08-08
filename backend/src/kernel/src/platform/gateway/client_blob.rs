//! Owner-neutral authenticated client Blob adapter for Core Gateway.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use makosh_blob_client::BlobDataClient;
use makosh_gateway_runtime::{
    ClientBlobContractVersionV1, ClientBlobRouteErrorV1, ClientBlobRouteHandler, ClientBlobRouteV1,
    ClientBlobRouter, ClientBlobTransportV1, SharedBrowserGatewaySessionService,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    BlobDataOperationV1, ContractReferenceV1, ManagedRuntimeBlobSessionRequestV1,
    ModuleClientBlobAuthorizationV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::identity::browser_gateway::ControlStoreBrowserAuthority;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};
use crate::platform::blob::session::BlobSessionHandlerV1;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

pub(crate) fn compose_client_blob_routers(
    store: Arc<SqliteControlStore>,
    data_dir: &Path,
    supervisor: ManagedRuntimeSupervisor,
    session: SharedBrowserGatewaySessionService<ControlStoreBrowserAuthority>,
    request_id_sequence: Arc<AtomicU64>,
) -> Result<Vec<ClientBlobRouter<ControlStoreBrowserAuthority>>, String> {
    let routes = store
        .approved_module_client_blob_routes()
        .map_err(|_| "owner client Blob route records are unavailable".to_owned())?;
    let adapter = Arc::new(KernelClientBlobAdapterV1::new(
        store,
        data_dir,
        supervisor,
        request_id_sequence,
    ));
    let handler: ClientBlobRouteHandler = Arc::new(
        move |route,
              logical_owner_id,
              authenticated_device_id,
              authenticated_client_session_id,
              payload| {
            adapter.authorize_and_read(
                route,
                logical_owner_id,
                authenticated_device_id,
                authenticated_client_session_id,
                payload,
            )
        },
    );

    routes
        .into_iter()
        .map(|route| {
            ClientBlobRouteV1::new(
                route.registration_id(),
                route.capability_id(),
                route.owner(),
                route.contract_name(),
                ClientBlobContractVersionV1 {
                    major: route.contract_major(),
                    revision: route.contract_revision(),
                },
                *route.contract_schema_sha256(),
                ClientBlobTransportV1 {
                    path: route.path().to_owned(),
                    max_response_bytes: route.max_response_bytes(),
                },
            )
            .map(|route| ClientBlobRouter::new(Arc::clone(&session), route, Arc::clone(&handler)))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(str::to_owned)
}

struct KernelClientBlobAdapterV1 {
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    blob_sessions: BlobSessionHandlerV1,
    request_id_sequence: Arc<AtomicU64>,
}

impl KernelClientBlobAdapterV1 {
    fn new(
        store: Arc<SqliteControlStore>,
        data_dir: &Path,
        supervisor: ManagedRuntimeSupervisor,
        request_id_sequence: Arc<AtomicU64>,
    ) -> Self {
        let blob_sessions = BlobSessionHandlerV1::new(
            Arc::clone(&store),
            supervisor.relay_port(),
            data_dir.to_path_buf(),
        );
        Self {
            store,
            supervisor,
            blob_sessions,
            request_id_sequence,
        }
    }

    fn authorize_and_read(
        &self,
        route: &ClientBlobRouteV1,
        logical_owner_id: &str,
        authenticated_device_id: &str,
        authenticated_client_session_id: &str,
        request_payload: &[u8],
    ) -> Result<Vec<u8>, ClientBlobRouteErrorV1> {
        let (runtime, authorization) = self.authorize_module_read(
            route,
            logical_owner_id,
            authenticated_device_id,
            authenticated_client_session_id,
            request_payload,
        )?;
        let content = self.read_blob(route, &runtime, authorization)?;
        if u64::try_from(content.len()).ok() != Some(runtime.declared_size) {
            return Err(ClientBlobRouteErrorV1::Internal);
        }
        Ok(content)
    }

    fn authorize_module_read(
        &self,
        route: &ClientBlobRouteV1,
        logical_owner_id: &str,
        authenticated_device_id: &str,
        authenticated_client_session_id: &str,
        request_payload: &[u8],
    ) -> Result<(AuthorizedRuntimeReadV1, ModuleClientBlobAuthorizationV1), ClientBlobRouteErrorV1>
    {
        let snapshot = self
            .store
            .module_grant_snapshot(route.registration_id())
            .map_err(|_| ClientBlobRouteErrorV1::Internal)?
            .ok_or(ClientBlobRouteErrorV1::NotFound)?;
        if snapshot.registration().owner_id() != route.owner() {
            return Err(ClientBlobRouteErrorV1::NotFound);
        }
        let grants = snapshot
            .effective_grants()
            .ok_or(ClientBlobRouteErrorV1::NotFound)?;
        if grants
            .capability_ids()
            .binary_search_by(|candidate| candidate.as_str().cmp(route.capability_id()))
            .is_err()
        {
            return Err(ClientBlobRouteErrorV1::NotFound);
        }
        let launch = self
            .store
            .effective_managed_launch_record(route.registration_id())
            .map_err(|_| ClientBlobRouteErrorV1::Internal)?
            .ok_or(ClientBlobRouteErrorV1::Unavailable)?;
        let binding = self
            .store
            .effective_bundled_managed_launch_binding(route.registration_id())
            .map_err(|_| ClientBlobRouteErrorV1::Internal)?
            .ok_or(ClientBlobRouteErrorV1::Unavailable)?;
        let expectation = ManagedRuntimeExpectation::from_fenced_launch(
            snapshot.registration(),
            &binding,
            &launch,
        )
        .map_err(|_| ClientBlobRouteErrorV1::Unavailable)?;
        let request_id = self
            .request_id_sequence
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        if request_id == 0 {
            return Err(ClientBlobRouteErrorV1::Unavailable);
        }
        let request = encode_module_request(
            snapshot.registration().module_id(),
            route,
            logical_owner_id,
            authenticated_device_id,
            authenticated_client_session_id,
            request_id,
            request_payload,
        )
        .map_err(|_| ClientBlobRouteErrorV1::InvalidArgument)?;
        let routed = ManagedCapabilityRouteRequest::new(
            snapshot.registration().registration_id(),
            launch.runtime_instance_id(),
            launch.runtime_generation(),
            grants.grant_epoch(),
            route.capability_id(),
            &request,
        );
        let response_bytes =
            route_managed_client_request(&*self.store, &self.supervisor.relay_port(), &routed)
                .map_err(map_managed_route_error)?;
        let response = ModuleClientResponseV1::decode(response_bytes.as_slice())
            .map_err(|_| ClientBlobRouteErrorV1::Internal)?;
        if !response.error_code.is_empty() {
            return Err(map_module_response_error(&response.error_code));
        }
        let authorization =
            ModuleClientBlobAuthorizationV1::decode(response.response_payload.as_slice())
                .map_err(|_| ClientBlobRouteErrorV1::Internal)?;
        validate_authorization(route, &authorization)?;
        Ok((
            AuthorizedRuntimeReadV1 {
                expectation,
                declared_size: authorization.declared_size,
            },
            authorization,
        ))
    }

    fn read_blob(
        &self,
        route: &ClientBlobRouteV1,
        runtime: &AuthorizedRuntimeReadV1,
        authorization: ModuleClientBlobAuthorizationV1,
    ) -> Result<Vec<u8>, ClientBlobRouteErrorV1> {
        let mut session_request_id = [0_u8; 16];
        let mut channel_binding = [0_u8; 32];
        getrandom::fill(&mut session_request_id)
            .map_err(|_| ClientBlobRouteErrorV1::Unavailable)?;
        getrandom::fill(&mut channel_binding).map_err(|_| ClientBlobRouteErrorV1::Unavailable)?;
        if session_request_id.iter().all(|byte| *byte == 0)
            || channel_binding.iter().all(|byte| *byte == 0)
        {
            return Err(ClientBlobRouteErrorV1::Unavailable);
        }
        let delivery = self
            .blob_sessions
            .issue_blob_session(
                &runtime.expectation,
                ManagedRuntimeBlobSessionRequestV1 {
                    request_id: session_request_id.to_vec(),
                    capability_id: route.capability_id().to_owned(),
                    operation: BlobDataOperationV1::BlobDataOperationReadRangeV1 as u32,
                    channel_binding_sha256: Sha256::digest(channel_binding).to_vec(),
                    reference_id: authorization.reference_id,
                    declared_size: authorization.declared_size,
                    backup_class: authorization.backup_class,
                    ttl_seconds: 30,
                    receipt_sha256: authorization.expected_plaintext_sha256,
                    custody_source_proof: Vec::new(),
                    evidence_id: Vec::new(),
                    evidence_envelope_sha256: Vec::new(),
                    custody_target_owner_id: String::new(),
                    custody_target_module_id: String::new(),
                    custody_target_capability_id: String::new(),
                },
            )
            .map_err(|_| ClientBlobRouteErrorV1::Unavailable)?;
        let grant = delivery.grant.ok_or(ClientBlobRouteErrorV1::Internal)?;
        BlobDataClient::new(delivery.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    grant,
                    channel_binding.to_vec(),
                    0,
                    authorization.declared_size,
                )
            })
            .map_err(|_| ClientBlobRouteErrorV1::Unavailable)
    }
}

struct AuthorizedRuntimeReadV1 {
    expectation: ManagedRuntimeExpectation,
    declared_size: u64,
}

fn encode_module_request(
    module_id: &str,
    route: &ClientBlobRouteV1,
    logical_owner_id: &str,
    authenticated_device_id: &str,
    authenticated_client_session_id: &str,
    request_id: u64,
    request_payload: &[u8],
) -> Result<Vec<u8>, ()> {
    if module_id.is_empty()
        || logical_owner_id.is_empty()
        || authenticated_device_id.is_empty()
        || authenticated_client_session_id.is_empty()
        || request_id == 0
        || request_payload.is_empty()
    {
        return Err(());
    }
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: module_id.to_owned(),
        owner_id: route.owner().to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: route.owner().to_owned(),
            name: route.contract_name().to_owned(),
            major: route.contract_major(),
            revision: route.contract_revision(),
            schema_sha256: route.contract_schema_sha256().to_vec(),
        }),
        request_id,
        request_payload: request_payload.to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: authenticated_device_id.to_owned(),
        authenticated_client_session_id: authenticated_client_session_id.to_owned(),
    }
    .encode_to_vec())
}

fn validate_authorization(
    route: &ClientBlobRouteV1,
    authorization: &ModuleClientBlobAuthorizationV1,
) -> Result<(), ClientBlobRouteErrorV1> {
    if authorization.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || authorization.reference_id.len() != 16
        || authorization.expected_plaintext_sha256.len() != 32
        || authorization.declared_size == 0
        || authorization.declared_size > route.max_response_bytes()
        || authorization.backup_class == 0
    {
        return Err(ClientBlobRouteErrorV1::Internal);
    }
    Ok(())
}

fn map_managed_route_error(error: String) -> ClientBlobRouteErrorV1 {
    match error.as_str() {
        "managed runtime is unavailable" | "managed runtime fence is stale" => {
            ClientBlobRouteErrorV1::Unavailable
        }
        "module registration is not approved"
        | "capability is not granted to this registration" => ClientBlobRouteErrorV1::NotFound,
        _ => ClientBlobRouteErrorV1::Internal,
    }
}

fn map_module_response_error(error_code: &str) -> ClientBlobRouteErrorV1 {
    match error_code {
        "REJECTED" | "NOT_FOUND" => ClientBlobRouteErrorV1::NotFound,
        "UNAVAILABLE" => ClientBlobRouteErrorV1::Unavailable,
        _ => ClientBlobRouteErrorV1::Internal,
    }
}
