//! Translation of a validated module descriptor into Control Store request records.

use makosh_kernel_control_store::{
    ModuleBlobOperationV1, ModuleBlobQuotaRequestV1, ModuleClientBlobRouteV1,
    ModuleClientRealtimeRouteV1, ModuleClientRpcRouteV1, ModuleEventDeliveryPolicyV1,
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1,
    ModuleEventRouteRequestV1, ModuleEventSubscriptionRequirementV1, ModuleQueryContractV1,
    ModuleRegistration, ModuleRequestContractV1, ModuleSchedulerJobRequestV1,
    ModuleStorageRequestV1, ModuleVaultPurposeRequestV1,
};
use makosh_runtime_protocol::{
    v1::{
        BlobQuotaOperationV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
        EventSubscriptionRequirementV1, ModuleKindV1, ProvidedSurfaceKindV1, VaultActionV1,
        VaultSecretClassV1, VaultTargetScopeV1,
        capability_request_v1::Request as CapabilityRequest,
    },
    validation::descriptor::decode_descriptor_v1,
};
use sha2::{Digest, Sha256};

pub(super) struct DescriptorRegistrationRequests {
    module_id: String,
    owner_id: String,
    descriptor_sha256: [u8; 32],
    capability_ids: Vec<String>,
    storage: Vec<DescriptorStorageRequest>,
    events: Vec<DescriptorEventRouteRequest>,
    blobs: Vec<DescriptorBlobQuotaRequest>,
    scheduler: Vec<DescriptorSchedulerJobRequest>,
    vault_purposes: Vec<DescriptorVaultPurposeRequest>,
    client_rpc_routes: Vec<DescriptorClientRpcRoute>,
    client_blob_routes: Vec<DescriptorClientBlobRoute>,
    client_realtime_routes: Vec<DescriptorClientRealtimeRoute>,
    query_rpc_routes: Vec<DescriptorModuleQueryContract>,
    request_rpc_routes: Vec<DescriptorModuleQueryContract>,
    contract_dependencies: Vec<DescriptorModuleQueryContract>,
}

pub(super) struct BoundRegistrationRequests {
    pub(super) storage: Vec<ModuleStorageRequestV1>,
    pub(super) events: Vec<ModuleEventRouteRequestV1>,
    pub(super) blobs: Vec<ModuleBlobQuotaRequestV1>,
    pub(super) scheduler: Vec<ModuleSchedulerJobRequestV1>,
    pub(super) vault_purposes: Vec<ModuleVaultPurposeRequestV1>,
    pub(super) client_rpc_routes: Vec<ModuleClientRpcRouteV1>,
    pub(super) client_blob_routes: Vec<ModuleClientBlobRouteV1>,
    pub(super) client_realtime_routes: Vec<ModuleClientRealtimeRouteV1>,
    pub(super) query_rpc_routes: Vec<ModuleQueryContractV1>,
    pub(super) request_rpc_routes: Vec<ModuleRequestContractV1>,
    pub(super) contract_dependencies: Vec<ModuleQueryContractV1>,
}

impl DescriptorRegistrationRequests {
    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
        let descriptor = decode_descriptor_v1(bytes)
            .map_err(|_| "module descriptor is invalid or exceeds protocol limits".to_owned())?;
        Ok(Self {
            module_id: descriptor.module_id.clone(),
            owner_id: descriptor.owner_id.clone(),
            descriptor_sha256: Sha256::digest(bytes).into(),
            capability_ids: descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.clone())
                .collect(),
            storage: storage_requests(&descriptor)?,
            events: event_route_requests(&descriptor)?,
            blobs: blob_quota_requests(&descriptor)?,
            scheduler: scheduler_job_requests(&descriptor)?,
            vault_purposes: vault_purpose_requests(&descriptor)?,
            client_rpc_routes: client_rpc_routes(&descriptor)?,
            client_blob_routes: client_blob_routes(&descriptor)?,
            client_realtime_routes: client_realtime_routes(&descriptor)?,
            query_rpc_routes: query_rpc_routes(&descriptor)?,
            request_rpc_routes: request_rpc_routes(&descriptor)?,
            contract_dependencies: contract_dependencies(&descriptor)?,
        })
    }

    pub(super) fn module_id(&self) -> String {
        self.module_id.clone()
    }

    pub(super) fn owner_id(&self) -> String {
        self.owner_id.clone()
    }

    pub(super) const fn descriptor_sha256(&self) -> [u8; 32] {
        self.descriptor_sha256
    }

    pub(super) fn capability_ids(&self) -> &[String] {
        &self.capability_ids
    }

    pub(super) fn bind(&self, registration: &ModuleRegistration) -> BoundRegistrationRequests {
        BoundRegistrationRequests {
            storage: bind_storage_requests(&self.storage, registration),
            events: bind_event_route_requests(&self.events, registration),
            blobs: bind_blob_quota_requests(&self.blobs, registration),
            scheduler: bind_scheduler_job_requests(&self.scheduler, registration),
            vault_purposes: bind_vault_purpose_requests(&self.vault_purposes, registration),
            client_rpc_routes: bind_client_rpc_routes(&self.client_rpc_routes, registration),
            client_blob_routes: bind_client_blob_routes(&self.client_blob_routes, registration),
            client_realtime_routes: bind_client_realtime_routes(
                &self.client_realtime_routes,
                registration,
            ),
            query_rpc_routes: bind_module_query_contracts(&self.query_rpc_routes, registration),
            request_rpc_routes: bind_module_request_contracts(
                &self.request_rpc_routes,
                registration,
            ),
            contract_dependencies: bind_module_query_contracts(
                &self.contract_dependencies,
                registration,
            ),
        }
    }
}

fn bind_storage_requests(
    requests: &[DescriptorStorageRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleStorageRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleStorageRequestV1::new(
                registration.registration_id(),
                &request.capability_id,
                &request.owner_id,
                request.connection_budget,
                request.statement_timeout_millis,
            )
        })
        .collect()
}

fn bind_event_route_requests(
    requests: &[DescriptorEventRouteRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleEventRouteRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
                registration_id: registration.registration_id().to_owned(),
                capability_id: request.capability_id.clone(),
                envelope_kind: request.envelope_kind,
                contract_owner: request.contract_owner.clone(),
                contract_name: request.contract_name.clone(),
                contract_major: request.contract_major,
                contract_revision: request.contract_revision,
                contract_schema_sha256: request.contract_schema_sha256,
                direction: request.direction,
                max_in_flight: request.max_in_flight,
                delivery_policy: request.delivery_policy,
            })
        })
        .collect()
}

fn bind_blob_quota_requests(
    requests: &[DescriptorBlobQuotaRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleBlobQuotaRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleBlobQuotaRequestV1::new(
                registration.registration_id(),
                &request.capability_id,
                registration.owner_id(),
                request.max_bytes,
                &request.custody_scope_id,
                request.allowed_operations.clone(),
            )
        })
        .collect()
}

fn bind_scheduler_job_requests(
    requests: &[DescriptorSchedulerJobRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleSchedulerJobRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleSchedulerJobRequestV1::new(
                registration.registration_id(),
                &request.capability_id,
                &request.owner,
                &request.name,
                request.major,
                request.revision,
                request.schema_sha256,
            )
        })
        .collect()
}

fn bind_vault_purpose_requests(
    requests: &[DescriptorVaultPurposeRequest],
    registration: &ModuleRegistration,
) -> Vec<ModuleVaultPurposeRequestV1> {
    requests
        .iter()
        .map(|request| {
            ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
                registration.registration_id(),
                &request.capability_id,
                &request.purpose_id,
                request.requested_lease_ttl_seconds,
                makosh_kernel_control_store::ModuleVaultPurposePolicyV1 {
                    secret_class: request.secret_class,
                    action: request.action,
                    target_scope: request.target_scope,
                    key_schema_revision: request.key_schema_revision,
                },
            )
        })
        .collect()
}

fn bind_client_rpc_routes(
    requests: &[DescriptorClientRpcRoute],
    registration: &ModuleRegistration,
) -> Vec<ModuleClientRpcRouteV1> {
    requests
        .iter()
        .map(|request| {
            ModuleClientRpcRouteV1::new(
                registration.registration_id(),
                &request.capability_id,
                registration.owner_id(),
                &request.contract_name,
                makosh_kernel_control_store::ModuleClientRpcContractVersionV1 {
                    major: request.contract_major,
                    revision: request.contract_revision,
                },
                request.contract_schema_sha256,
                &request.path,
            )
        })
        .collect()
}

fn bind_client_blob_routes(
    requests: &[DescriptorClientBlobRoute],
    registration: &ModuleRegistration,
) -> Vec<ModuleClientBlobRouteV1> {
    requests
        .iter()
        .map(|request| {
            ModuleClientBlobRouteV1::new(
                registration.registration_id(),
                &request.capability_id,
                registration.owner_id(),
                &request.contract_name,
                makosh_kernel_control_store::ModuleClientBlobContractVersionV1 {
                    major: request.contract_major,
                    revision: request.contract_revision,
                },
                request.contract_schema_sha256,
                makosh_kernel_control_store::ModuleClientBlobTransportV1 {
                    path: request.path.clone(),
                    max_response_bytes: request.max_response_bytes,
                },
            )
        })
        .collect()
}

fn bind_client_realtime_routes(
    requests: &[DescriptorClientRealtimeRoute],
    registration: &ModuleRegistration,
) -> Vec<ModuleClientRealtimeRouteV1> {
    requests
        .iter()
        .map(|request| {
            ModuleClientRealtimeRouteV1::new(
                registration.registration_id(),
                &request.capability_id,
                registration.owner_id(),
                &request.contract_name,
                makosh_kernel_control_store::ModuleClientRealtimeContractVersionV1 {
                    major: request.contract_major,
                    revision: request.contract_revision,
                },
                request.contract_schema_sha256,
            )
        })
        .collect()
}

fn bind_module_query_contracts(
    requests: &[DescriptorModuleQueryContract],
    registration: &ModuleRegistration,
) -> Vec<ModuleQueryContractV1> {
    requests
        .iter()
        .map(|request| {
            ModuleQueryContractV1::new(
                registration.registration_id(),
                &request.capability_id,
                &request.owner,
                &request.name,
                request.major,
                request.revision,
                request.schema_sha256,
            )
        })
        .collect()
}

fn bind_module_request_contracts(
    requests: &[DescriptorModuleQueryContract],
    registration: &ModuleRegistration,
) -> Vec<ModuleRequestContractV1> {
    requests
        .iter()
        .map(|request| {
            ModuleRequestContractV1::new(
                registration.registration_id(),
                &request.capability_id,
                &request.owner,
                &request.name,
                request.major,
                request.revision,
                request.schema_sha256,
            )
        })
        .collect()
}

struct DescriptorStorageRequest {
    capability_id: String,
    owner_id: String,
    connection_budget: u16,
    statement_timeout_millis: u32,
}

struct DescriptorEventRouteRequest {
    capability_id: String,
    envelope_kind: ModuleEventEnvelopeKindV1,
    contract_owner: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    direction: ModuleEventRouteDirectionV1,
    max_in_flight: u16,
    delivery_policy: Option<ModuleEventDeliveryPolicyV1>,
}

struct DescriptorBlobQuotaRequest {
    capability_id: String,
    max_bytes: u64,
    custody_scope_id: String,
    allowed_operations: Vec<ModuleBlobOperationV1>,
}

struct DescriptorSchedulerJobRequest {
    capability_id: String,
    owner: String,
    name: String,
    major: u32,
    revision: u32,
    schema_sha256: [u8; 32],
}

struct DescriptorVaultPurposeRequest {
    capability_id: String,
    purpose_id: String,
    requested_lease_ttl_seconds: u16,
    secret_class: u8,
    action: u8,
    target_scope: u8,
    key_schema_revision: u32,
}

struct DescriptorClientRpcRoute {
    capability_id: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    path: String,
}

struct DescriptorClientBlobRoute {
    capability_id: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    path: String,
    max_response_bytes: u64,
}

struct DescriptorClientRealtimeRoute {
    capability_id: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
}

struct DescriptorModuleQueryContract {
    capability_id: String,
    owner: String,
    name: String,
    major: u32,
    revision: u32,
    schema_sha256: [u8; 32],
}

fn query_rpc_routes(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorModuleQueryContract>, String> {
    let mut routes = Vec::new();
    for capability in &descriptor.capabilities {
        for surface in &capability.provides {
            if ProvidedSurfaceKindV1::try_from(surface.kind).ok()
                != Some(ProvidedSurfaceKindV1::QueryRpc)
            {
                continue;
            }
            let contract = surface
                .contract
                .as_ref()
                .ok_or_else(|| "module Query RPC contract is invalid".to_owned())?;
            if contract.owner != descriptor.owner_id {
                return Err("module Query RPC contract owner is invalid".to_owned());
            }
            routes.push(descriptor_query_contract(
                &capability.capability_id,
                contract,
            )?);
        }
    }
    Ok(routes)
}

fn request_rpc_routes(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorModuleQueryContract>, String> {
    let mut routes = Vec::new();
    for capability in &descriptor.capabilities {
        for surface in &capability.provides {
            if ProvidedSurfaceKindV1::try_from(surface.kind).ok()
                != Some(ProvidedSurfaceKindV1::RequestRpc)
            {
                continue;
            }
            let contract = surface
                .contract
                .as_ref()
                .ok_or_else(|| "module Request RPC contract is invalid".to_owned())?;
            if contract.owner != descriptor.owner_id
                && ModuleKindV1::try_from(descriptor.module_kind).ok()
                    != Some(ModuleKindV1::Integration)
            {
                return Err("module Request RPC contract owner is invalid".to_owned());
            }
            routes.push(descriptor_query_contract(
                &capability.capability_id,
                contract,
            )?);
        }
    }
    Ok(routes)
}

fn contract_dependencies(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorModuleQueryContract>, String> {
    let mut dependencies = Vec::new();
    for capability in &descriptor.capabilities {
        for contract in &capability.dependencies {
            dependencies.push(descriptor_query_contract(
                &capability.capability_id,
                contract,
            )?);
        }
    }
    Ok(dependencies)
}

fn descriptor_query_contract(
    capability_id: &str,
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Result<DescriptorModuleQueryContract, String> {
    Ok(DescriptorModuleQueryContract {
        capability_id: capability_id.to_owned(),
        owner: contract.owner.clone(),
        name: contract.name.clone(),
        major: contract.major,
        revision: contract.revision,
        schema_sha256: contract
            .schema_sha256
            .as_slice()
            .try_into()
            .map_err(|_| "module contract dependency is invalid".to_owned())?,
    })
}

fn client_rpc_routes(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorClientRpcRoute>, String> {
    let mut seen_paths = std::collections::BTreeSet::new();
    let mut routes = Vec::new();
    for capability in &descriptor.capabilities {
        for surface in &capability.provides {
            if ProvidedSurfaceKindV1::try_from(surface.kind).ok()
                != Some(ProvidedSurfaceKindV1::ClientRpc)
            {
                continue;
            }
            let contract = surface
                .contract
                .as_ref()
                .ok_or_else(|| "module Client RPC contract is invalid".to_owned())?;
            let route = surface
                .client_rpc_route
                .as_ref()
                .ok_or_else(|| "module Client RPC route is invalid".to_owned())?;
            let schema_sha256 = contract
                .schema_sha256
                .as_slice()
                .try_into()
                .map_err(|_| "module Client RPC contract is invalid".to_owned())?;
            if contract.owner != descriptor.owner_id || !seen_paths.insert(route.path.clone()) {
                return Err("module Client RPC route owner or path is invalid".to_owned());
            }
            routes.push(DescriptorClientRpcRoute {
                capability_id: capability.capability_id.clone(),
                contract_name: contract.name.clone(),
                contract_major: contract.major,
                contract_revision: contract.revision,
                contract_schema_sha256: schema_sha256,
                path: route.path.clone(),
            });
        }
    }
    Ok(routes)
}

fn client_blob_routes(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorClientBlobRoute>, String> {
    let mut seen_paths = std::collections::BTreeSet::new();
    let mut routes = Vec::new();
    for capability in &descriptor.capabilities {
        for surface in &capability.provides {
            if ProvidedSurfaceKindV1::try_from(surface.kind).ok()
                != Some(ProvidedSurfaceKindV1::ClientBlob)
            {
                continue;
            }
            let contract = surface
                .contract
                .as_ref()
                .ok_or_else(|| "module client Blob contract is invalid".to_owned())?;
            let route = surface
                .client_blob_route
                .as_ref()
                .ok_or_else(|| "module client Blob route is invalid".to_owned())?;
            let schema_sha256 = contract
                .schema_sha256
                .as_slice()
                .try_into()
                .map_err(|_| "module client Blob contract is invalid".to_owned())?;
            if contract.owner != descriptor.owner_id || !seen_paths.insert(route.path.clone()) {
                return Err("module client Blob route owner or path is invalid".to_owned());
            }
            routes.push(DescriptorClientBlobRoute {
                capability_id: capability.capability_id.clone(),
                contract_name: contract.name.clone(),
                contract_major: contract.major,
                contract_revision: contract.revision,
                contract_schema_sha256: schema_sha256,
                path: route.path.clone(),
                max_response_bytes: route.max_response_bytes,
            });
        }
    }
    Ok(routes)
}

fn client_realtime_routes(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorClientRealtimeRoute>, String> {
    let mut seen_contracts = std::collections::BTreeSet::new();
    let mut routes = Vec::new();
    for capability in &descriptor.capabilities {
        for surface in &capability.provides {
            if ProvidedSurfaceKindV1::try_from(surface.kind).ok()
                != Some(ProvidedSurfaceKindV1::ClientRealtime)
            {
                continue;
            }
            let contract = surface
                .contract
                .as_ref()
                .ok_or_else(|| "module ClientRealtime contract is invalid".to_owned())?;
            let schema_sha256 = contract
                .schema_sha256
                .as_slice()
                .try_into()
                .map_err(|_| "module ClientRealtime contract is invalid".to_owned())?;
            let identity = (
                contract.name.clone(),
                contract.major,
                contract.revision,
                schema_sha256,
            );
            if contract.owner != descriptor.owner_id || !seen_contracts.insert(identity) {
                return Err(
                    "module ClientRealtime contract owner or identity is invalid".to_owned(),
                );
            }
            routes.push(DescriptorClientRealtimeRoute {
                capability_id: capability.capability_id.clone(),
                contract_name: contract.name.clone(),
                contract_major: contract.major,
                contract_revision: contract.revision,
                contract_schema_sha256: schema_sha256,
            });
        }
    }
    Ok(routes)
}

fn event_route_requests(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorEventRouteRequest>, String> {
    descriptor
        .capabilities
        .iter()
        .flat_map(|capability| {
            capability
                .requests
                .iter()
                .map(move |request| (capability, request))
        })
        .filter_map(|(capability, request)| match request.request.as_ref() {
            Some(CapabilityRequest::EventRoute(route)) => Some((capability, route)),
            _ => None,
        })
        .map(|(capability, route)| descriptor_event_route(capability, route))
        .collect()
}

fn descriptor_event_route(
    capability: &makosh_runtime_protocol::v1::CapabilityDescriptorV1,
    route: &makosh_runtime_protocol::v1::EventRouteRequestV1,
) -> Result<DescriptorEventRouteRequest, String> {
    let contract = route
        .contract
        .as_ref()
        .ok_or_else(|| "module Event route request is invalid".to_owned())?;
    let contract_schema_sha256 = contract
        .schema_sha256
        .as_slice()
        .try_into()
        .map_err(|_| "module Event route request is invalid".to_owned())?;
    Ok(DescriptorEventRouteRequest {
        capability_id: capability.capability_id.clone(),
        envelope_kind: event_envelope_kind(route.envelope_kind)?,
        contract_owner: contract.owner.clone(),
        contract_name: contract.name.clone(),
        contract_major: contract.major,
        contract_revision: contract.revision,
        contract_schema_sha256,
        direction: event_route_direction(route.direction)?,
        max_in_flight: u16::try_from(route.max_in_flight)
            .map_err(|_| "module Event route request is invalid".to_owned())?,
        delivery_policy: event_delivery_policy(route)?,
    })
}

fn event_envelope_kind(value: i32) -> Result<ModuleEventEnvelopeKindV1, String> {
    match DurableEnvelopeKindV1::try_from(value).ok() {
        Some(DurableEnvelopeKindV1::Command) => Ok(ModuleEventEnvelopeKindV1::Command),
        Some(DurableEnvelopeKindV1::Event) => Ok(ModuleEventEnvelopeKindV1::Event),
        Some(DurableEnvelopeKindV1::Observation) => Ok(ModuleEventEnvelopeKindV1::Observation),
        Some(DurableEnvelopeKindV1::Result) => Ok(ModuleEventEnvelopeKindV1::Result),
        Some(DurableEnvelopeKindV1::Ack) => Ok(ModuleEventEnvelopeKindV1::Ack),
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn event_route_direction(value: i32) -> Result<ModuleEventRouteDirectionV1, String> {
    match EventRouteDirectionV1::try_from(value).ok() {
        Some(EventRouteDirectionV1::Publish) => Ok(ModuleEventRouteDirectionV1::Publish),
        Some(EventRouteDirectionV1::Consume) => Ok(ModuleEventRouteDirectionV1::Consume),
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn event_delivery_policy(
    route: &makosh_runtime_protocol::v1::EventRouteRequestV1,
) -> Result<Option<ModuleEventDeliveryPolicyV1>, String> {
    match EventRouteDirectionV1::try_from(route.direction).ok() {
        Some(EventRouteDirectionV1::Publish) => Ok(None),
        Some(EventRouteDirectionV1::Consume) => Ok(Some(ModuleEventDeliveryPolicyV1::new(
            event_subscription_requirement(route.subscription_requirement)?,
            u8::try_from(route.max_deliver)
                .map_err(|_| "module Event route request is invalid".to_owned())?,
            route.ack_wait_millis,
        ))),
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn event_subscription_requirement(
    value: i32,
) -> Result<ModuleEventSubscriptionRequirementV1, String> {
    match EventSubscriptionRequirementV1::try_from(value).ok() {
        Some(EventSubscriptionRequirementV1::Required) => {
            Ok(ModuleEventSubscriptionRequirementV1::Required)
        }
        Some(EventSubscriptionRequirementV1::Optional) => {
            Ok(ModuleEventSubscriptionRequirementV1::Optional)
        }
        _ => Err("module Event route request is invalid".to_owned()),
    }
}

fn blob_quota_requests(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorBlobQuotaRequest>, String> {
    let requests = descriptor
        .capabilities
        .iter()
        .map(blob_quota_request_for_capability)
        .collect::<Result<Vec<_>, _>>()
        .map(|requests| requests.into_iter().flatten().collect::<Vec<_>>())?;
    let mut quotas_by_scope = std::collections::BTreeMap::new();
    if !requests.iter().all(|request| {
        quotas_by_scope
            .entry(request.custody_scope_id.as_str())
            .or_insert(request.max_bytes)
            == &request.max_bytes
    }) {
        return Err("module Blob quota request is invalid".to_owned());
    }
    Ok(requests)
}

fn blob_quota_request_for_capability(
    capability: &makosh_runtime_protocol::v1::CapabilityDescriptorV1,
) -> Result<Option<DescriptorBlobQuotaRequest>, String> {
    let requests = capability
        .requests
        .iter()
        .filter_map(|request| match request.request.as_ref() {
            Some(CapabilityRequest::BlobQuota(blob)) => Some(blob),
            _ => None,
        })
        .collect::<Vec<_>>();
    match requests.as_slice() {
        [] => Ok(None),
        [request] => {
            let allowed_operations = request
                .allowed_operations
                .iter()
                .map(|value| match BlobQuotaOperationV1::try_from(*value).ok() {
                    Some(BlobQuotaOperationV1::Write) => Ok(ModuleBlobOperationV1::Write),
                    Some(BlobQuotaOperationV1::ReadRange) => Ok(ModuleBlobOperationV1::ReadRange),
                    Some(BlobQuotaOperationV1::CustodyTransfer) => {
                        Ok(ModuleBlobOperationV1::CustodyTransfer)
                    }
                    Some(BlobQuotaOperationV1::ReleaseCustody) => {
                        Ok(ModuleBlobOperationV1::ReleaseCustody)
                    }
                    Some(BlobQuotaOperationV1::Unspecified) | None => {
                        Err("module Blob quota request is invalid".to_owned())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(DescriptorBlobQuotaRequest {
                capability_id: capability.capability_id.clone(),
                max_bytes: request.max_bytes,
                custody_scope_id: request.custody_scope_id.clone(),
                allowed_operations,
            }))
        }
        _ => Err("module Blob quota request is invalid".to_owned()),
    }
}

fn storage_requests(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorStorageRequest>, String> {
    let mut requests = Vec::new();
    for capability in &descriptor.capabilities {
        let requested = capability
            .requests
            .iter()
            .filter_map(|request| match request.request.as_ref() {
                Some(CapabilityRequest::StorageNamespace(storage)) => Some(storage),
                _ => None,
            })
            .collect::<Vec<_>>();
        if requested.len() > 1 {
            return Err("module Storage request is invalid".to_owned());
        }
        if let Some(request) = requested.first() {
            if request.owner_id != descriptor.owner_id {
                return Err("module Storage request owner is invalid".to_owned());
            }
            requests.push(DescriptorStorageRequest {
                capability_id: capability.capability_id.clone(),
                owner_id: request.owner_id.clone(),
                connection_budget: u16::try_from(request.connection_budget)
                    .map_err(|_| "module Storage request is invalid".to_owned())?,
                statement_timeout_millis: request.timeout_millis,
            });
        }
    }
    Ok(requests)
}

fn scheduler_job_requests(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorSchedulerJobRequest>, String> {
    let mut requests = Vec::new();
    for capability in &descriptor.capabilities {
        for request in
            capability
                .requests
                .iter()
                .filter_map(|request| match request.request.as_ref() {
                    Some(CapabilityRequest::SchedulerJob(scheduler)) => Some(scheduler),
                    _ => None,
                })
        {
            let job_kind = request
                .job_kind
                .as_ref()
                .ok_or_else(|| "module Scheduler job request is invalid".to_owned())?;
            let schema_sha256 = job_kind
                .schema_sha256
                .as_slice()
                .try_into()
                .map_err(|_| "module Scheduler job request is invalid".to_owned())?;
            if job_kind.owner != descriptor.owner_id || job_kind.major > u32::from(u16::MAX) {
                return Err("module Scheduler job request owner is invalid".to_owned());
            }
            requests.push(DescriptorSchedulerJobRequest {
                capability_id: capability.capability_id.clone(),
                owner: job_kind.owner.clone(),
                name: job_kind.name.clone(),
                major: job_kind.major,
                revision: job_kind.revision,
                schema_sha256,
            });
        }
    }
    Ok(requests)
}

fn vault_purpose_requests(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
) -> Result<Vec<DescriptorVaultPurposeRequest>, String> {
    let mut result = Vec::new();
    for capability in &descriptor.capabilities {
        for purpose in
            capability
                .requests
                .iter()
                .filter_map(|request| match request.request.as_ref() {
                    Some(CapabilityRequest::VaultPurpose(purpose)) => Some(purpose),
                    _ => None,
                })
        {
            let target_scope = VaultTargetScopeV1::try_from(purpose.target_scope)
                .ok()
                .ok_or_else(|| "module Vault purpose target scope is invalid".to_owned())?;
            let owner_derived = target_scope == VaultTargetScopeV1::OwnerDerivedProjectionKey;
            if target_scope != VaultTargetScopeV1::ConfigurationInstance && !owner_derived {
                return Err("module Vault purpose target scope is invalid".to_owned());
            }
            if (owner_derived
                && (purpose.key_schema_revision == 0
                    || purpose.allowed_secret_classes
                        != [VaultSecretClassV1::OwnerDerivedKey as i32]
                    || purpose.actions != [VaultActionV1::IssueOwnerDerivedKey as i32]))
                || (!owner_derived && purpose.key_schema_revision != 0)
            {
                return Err("module Vault purpose request is invalid".to_owned());
            }
            let ttl = u16::try_from(purpose.requested_lease_ttl_seconds)
                .map_err(|_| "module Vault purpose request is invalid".to_owned())?;
            for secret_class in &purpose.allowed_secret_classes {
                let secret_class = VaultSecretClassV1::try_from(*secret_class)
                    .ok()
                    .filter(|value| *value != VaultSecretClassV1::Unspecified)
                    .ok_or_else(|| "module Vault purpose request is invalid".to_owned())?
                    as u8;
                for action in &purpose.actions {
                    let action = VaultActionV1::try_from(*action)
                        .ok()
                        .filter(|value| *value != VaultActionV1::Unspecified)
                        .ok_or_else(|| "module Vault purpose request is invalid".to_owned())?
                        as u8;
                    result.push(DescriptorVaultPurposeRequest {
                        capability_id: capability.capability_id.clone(),
                        purpose_id: purpose.purpose_id.clone(),
                        requested_lease_ttl_seconds: ttl,
                        secret_class,
                        action,
                        target_scope: target_scope as u8,
                        key_schema_revision: purpose.key_schema_revision,
                    });
                }
            }
        }
    }
    Ok(result)
}
