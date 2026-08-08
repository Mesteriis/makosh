//! SQLite persistence for descriptor-declared owner ClientRpc routes.

use std::collections::BTreeSet;

use makosh_kernel_control_store::ModuleClientRpcRouteV1;
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

impl SqliteControlStore {
    pub fn approved_module_client_rpc_routes(
        &self,
    ) -> Result<Vec<ModuleClientRpcRouteV1>, StoreError> {
        self.with_connection(read_approved_client_rpc_routes)
    }
}

pub(crate) fn validate_client_rpc_routes(
    registration: &makosh_kernel_control_store::ModuleRegistration,
    capabilities: &[String],
    routes: &[ModuleClientRpcRouteV1],
) -> Result<(), StoreError> {
    let capabilities = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut paths = BTreeSet::new();
    routes
        .iter()
        .all(|route| {
            route.registration_id() == registration.registration_id()
                && route.owner() == registration.owner_id()
                && capabilities.contains(route.capability_id())
                && valid_capability_ids(&[route.capability_id().to_owned()])
                && valid_identity_token(route.owner())
                && valid_identity_token(route.contract_name())
                && route.contract_major() > 0
                && route.contract_revision() > 0
                && valid_connect_path(route.path())
                && paths.insert(route.path())
        })
        .then_some(())
        .ok_or(StoreError::InvalidModuleClientRpcRoute)
}

pub(crate) fn insert_client_rpc_routes(
    connection: &Connection,
    routes: &[ModuleClientRpcRouteV1],
) -> Result<(), StoreError> {
    for route in routes {
        connection.execute(
            "INSERT INTO makosh_kernel_module_client_rpc_route_request
             (registration_id, capability_id, contract_owner, contract_name, contract_major,
              contract_revision, contract_schema_sha256, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                route.registration_id(),
                route.capability_id(),
                route.owner(),
                route.contract_name(),
                i64::from(route.contract_major()),
                i64::from(route.contract_revision()),
                route.contract_schema_sha256().as_slice(),
                route.path()
            ],
        )?;
    }
    Ok(())
}

fn read_approved_client_rpc_routes(
    connection: &mut Connection,
) -> Result<Vec<ModuleClientRpcRouteV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT route.registration_id, route.capability_id, route.contract_owner, route.contract_name,
                route.contract_major, route.contract_revision, route.contract_schema_sha256, route.path
         FROM makosh_kernel_module_client_rpc_route_request route
         JOIN makosh_kernel_module_registration registration ON registration.registration_id = route.registration_id
         JOIN makosh_kernel_module_registration_capability capability
           ON capability.registration_id = route.registration_id AND capability.capability_id = route.capability_id
         WHERE registration.state = 'approved'
         ORDER BY route.path, route.registration_id",
    )?;
    statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Vec<u8>>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?
        .map(|row| {
            let (registration_id, capability_id, owner, name, major, revision, digest, path) = row?;
            Ok(ModuleClientRpcRouteV1::new(
                registration_id,
                capability_id,
                owner,
                name,
                makosh_kernel_control_store::ModuleClientRpcContractVersionV1 {
                    major: u32::try_from(major)
                        .map_err(|_| StoreError::InvalidModuleClientRpcRoute)?,
                    revision: u32::try_from(revision)
                        .map_err(|_| StoreError::InvalidModuleClientRpcRoute)?,
                },
                digest
                    .try_into()
                    .map_err(|_| StoreError::InvalidModuleClientRpcRoute)?,
                path,
            ))
        })
        .collect()
}

fn valid_connect_path(path: &str) -> bool {
    let mut segments = path.split('/');
    matches!(segments.next(), Some(""))
        && segments.next().is_some_and(valid_component)
        && segments.next().is_some_and(valid_component)
        && segments.next().is_none()
        && path.len() <= 512
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
}
