//! SQLite persistence for descriptor-declared authenticated client Blob routes.

use std::collections::BTreeSet;

use makosh_kernel_control_store::{
    ModuleBlobOperationV1, ModuleBlobQuotaRequestV1, ModuleClientBlobRouteV1,
};
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

const MAX_CLIENT_BLOB_RESPONSE_BYTES: u64 = 32 * 1024 * 1024;

impl SqliteControlStore {
    pub fn approved_module_client_blob_routes(
        &self,
    ) -> Result<Vec<ModuleClientBlobRouteV1>, StoreError> {
        self.with_connection(read_approved_client_blob_routes)
    }
}

pub(crate) fn validate_client_blob_routes(
    registration: &makosh_kernel_control_store::ModuleRegistration,
    capabilities: &[String],
    blob_requests: &[ModuleBlobQuotaRequestV1],
    routes: &[ModuleClientBlobRouteV1],
) -> Result<(), StoreError> {
    let capabilities = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut paths = BTreeSet::new();
    routes
        .iter()
        .all(|route| {
            let blob_request = blob_requests.iter().find(|request| {
                request.registration_id() == route.registration_id()
                    && request.capability_id() == route.capability_id()
            });
            route.registration_id() == registration.registration_id()
                && route.owner() == registration.owner_id()
                && capabilities.contains(route.capability_id())
                && valid_capability_ids(&[route.capability_id().to_owned()])
                && valid_identity_token(route.owner())
                && valid_identity_token(route.contract_name())
                && route.contract_major() > 0
                && route.contract_revision() > 0
                && valid_client_blob_path(route.path())
                && (1..=MAX_CLIENT_BLOB_RESPONSE_BYTES).contains(&route.max_response_bytes())
                && paths.insert(route.path())
                && blob_request.is_some_and(|request| {
                    request.max_bytes() >= route.max_response_bytes()
                        && request.allows(ModuleBlobOperationV1::ReadRange)
                })
        })
        .then_some(())
        .ok_or(StoreError::InvalidModuleClientBlobRoute)
}

pub(crate) fn insert_client_blob_routes(
    connection: &Connection,
    routes: &[ModuleClientBlobRouteV1],
) -> Result<(), StoreError> {
    for route in routes {
        connection.execute(
            "INSERT INTO makosh_kernel_module_client_blob_route_request
             (registration_id, capability_id, contract_owner, contract_name, contract_major,
              contract_revision, contract_schema_sha256, path, max_response_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                route.registration_id(),
                route.capability_id(),
                route.owner(),
                route.contract_name(),
                i64::from(route.contract_major()),
                i64::from(route.contract_revision()),
                route.contract_schema_sha256().as_slice(),
                route.path(),
                i64::try_from(route.max_response_bytes())
                    .map_err(|_| StoreError::InvalidModuleClientBlobRoute)?,
            ],
        )?;
    }
    Ok(())
}

fn read_approved_client_blob_routes(
    connection: &mut Connection,
) -> Result<Vec<ModuleClientBlobRouteV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT route.registration_id, route.capability_id, route.contract_owner, route.contract_name,
                route.contract_major, route.contract_revision, route.contract_schema_sha256,
                route.path, route.max_response_bytes
         FROM makosh_kernel_module_client_blob_route_request route
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
                row.get::<_, i64>(8)?,
            ))
        })?
        .map(|row| {
            let (registration_id, capability_id, owner, name, major, revision, digest, path, max) =
                row?;
            Ok(ModuleClientBlobRouteV1::new(
                registration_id,
                capability_id,
                owner,
                name,
                makosh_kernel_control_store::ModuleClientBlobContractVersionV1 {
                    major: u32::try_from(major)
                        .map_err(|_| StoreError::InvalidModuleClientBlobRoute)?,
                    revision: u32::try_from(revision)
                        .map_err(|_| StoreError::InvalidModuleClientBlobRoute)?,
                },
                digest
                    .try_into()
                    .map_err(|_| StoreError::InvalidModuleClientBlobRoute)?,
                makosh_kernel_control_store::ModuleClientBlobTransportV1 {
                    path,
                    max_response_bytes: u64::try_from(max)
                        .map_err(|_| StoreError::InvalidModuleClientBlobRoute)?,
                },
            ))
        })
        .collect()
}

fn valid_client_blob_path(path: &str) -> bool {
    path.starts_with("/api/blobs/")
        && path.len() <= 512
        && path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
        && !path.contains("//")
        && !path.ends_with('/')
}

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::{
        ModuleBlobOperationV1, ModuleBlobQuotaRequestV1, ModuleClientBlobContractVersionV1,
        ModuleClientBlobRouteV1, ModuleRegistration, ModuleRegistrationState,
    };

    use super::{MAX_CLIENT_BLOB_RESPONSE_BYTES, validate_client_blob_routes};

    fn registration() -> ModuleRegistration {
        ModuleRegistration::new(
            "registration-1",
            "module-notes",
            "notes",
            [1; 32],
            ModuleRegistrationState::Pending,
            1,
        )
    }

    fn route(max_response_bytes: u64) -> ModuleClientBlobRouteV1 {
        ModuleClientBlobRouteV1::new(
            "registration-1",
            "notes.content.v1",
            "notes",
            "notes.content-read",
            ModuleClientBlobContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [2; 32],
            makosh_kernel_control_store::ModuleClientBlobTransportV1 {
                path: "/api/blobs/notes/v1/content".to_owned(),
                max_response_bytes,
            },
        )
    }

    #[test]
    fn route_requires_same_capability_read_range_quota() {
        let registration = registration();
        let capabilities = vec!["notes.content.v1".to_owned()];
        let read = ModuleBlobQuotaRequestV1::new(
            "registration-1",
            "notes.content.v1",
            "notes",
            256 * 1024,
            "notes.content.v1",
            vec![ModuleBlobOperationV1::ReadRange],
        );
        assert!(
            validate_client_blob_routes(
                &registration,
                &capabilities,
                std::slice::from_ref(&read),
                &[route(256 * 1024)],
            )
            .is_ok()
        );

        let write_only = ModuleBlobQuotaRequestV1::new(
            "registration-1",
            "notes.content.v1",
            "notes",
            256 * 1024,
            "notes.content.v1",
            vec![ModuleBlobOperationV1::Write],
        );
        assert!(
            validate_client_blob_routes(
                &registration,
                &capabilities,
                &[write_only],
                &[route(256 * 1024)],
            )
            .is_err()
        );
        assert!(
            validate_client_blob_routes(
                &registration,
                &capabilities,
                &[read],
                &[route(256 * 1024 + 1)],
            )
            .is_err()
        );

        let ceiling_read = ModuleBlobQuotaRequestV1::new(
            "registration-1",
            "notes.content.v1",
            "notes",
            MAX_CLIENT_BLOB_RESPONSE_BYTES + 1,
            "notes.content.v1",
            vec![ModuleBlobOperationV1::ReadRange],
        );
        assert!(
            validate_client_blob_routes(
                &registration,
                &capabilities,
                std::slice::from_ref(&ceiling_read),
                &[route(MAX_CLIENT_BLOB_RESPONSE_BYTES)],
            )
            .is_ok()
        );
        assert!(
            validate_client_blob_routes(
                &registration,
                &capabilities,
                &[ceiling_read],
                &[route(MAX_CLIENT_BLOB_RESPONSE_BYTES + 1)],
            )
            .is_err()
        );
    }
}
