//! SQLite persistence for descriptor-declared Blob quota requests.

use std::collections::BTreeSet;

use makosh_kernel_control_store::{
    ModuleBlobOperationV1, ModuleBlobQuotaRequestV1, ModuleRegistration,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids};

const MAX_BLOB_QUOTA_BYTES: u64 = 1 << 40;

impl SqliteControlStore {
    pub fn module_blob_quota_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleBlobQuotaRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_blob_quota_request(connection, &registration_id, &capability_id)
        })
    }
}

pub(crate) fn validate_blob_quota_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleBlobQuotaRequestV1],
) -> Result<(), StoreError> {
    let requested = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut quotas_by_scope = std::collections::BTreeMap::new();
    let valid = requests.iter().all(|request| {
        let operations = request
            .allowed_operations()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        request.registration_id() == registration.registration_id()
            && request.owner_id() == registration.owner_id()
            && requested.contains(request.capability_id())
            && valid_capability_ids(&[request.capability_id().to_owned()])
            && valid_capability_ids(&[request.custody_scope_id().to_owned()])
            && (1..=MAX_BLOB_QUOTA_BYTES).contains(&request.max_bytes())
            && !operations.is_empty()
            && operations.len() == request.allowed_operations().len()
            && quotas_by_scope
                .entry(request.custody_scope_id())
                .or_insert(request.max_bytes())
                == &request.max_bytes()
            && seen.insert(request.capability_id())
    });
    valid
        .then_some(())
        .ok_or(StoreError::InvalidModuleBlobQuotaRequest)
}

pub(crate) fn insert_blob_quota_requests(
    connection: &Connection,
    requests: &[ModuleBlobQuotaRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        let quota = i64::try_from(request.max_bytes())
            .map_err(|_| StoreError::InvalidModuleBlobQuotaRequest)?;
        connection.execute(
            "INSERT INTO makosh_kernel_module_blob_quota_request
             (registration_id, capability_id, owner_id, max_bytes, custody_scope_id, allowed_operations)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                request.registration_id(),
                request.capability_id(),
                request.owner_id(),
                quota,
                request.custody_scope_id(),
                operation_mask(request.allowed_operations()),
            ],
        )?;
    }
    Ok(())
}

fn read_blob_quota_request(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Option<ModuleBlobQuotaRequestV1>, StoreError> {
    connection
        .query_row(
            "SELECT owner_id, max_bytes, custody_scope_id, allowed_operations
             FROM makosh_kernel_module_blob_quota_request
             WHERE registration_id = ?1 AND capability_id = ?2",
            params![registration_id, capability_id],
            |row| {
                let max_bytes = u64::try_from(row.get::<_, i64>(1)?)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?;
                Ok(ModuleBlobQuotaRequestV1::new(
                    registration_id,
                    capability_id,
                    row.get::<_, String>(0)?,
                    max_bytes,
                    row.get::<_, String>(2)?,
                    decode_operation_mask(row.get::<_, i64>(3)?),
                ))
            },
        )
        .optional()
        .map_err(StoreError::from)
}

fn operation_mask(operations: &[ModuleBlobOperationV1]) -> i64 {
    operations.iter().fold(0, |mask, operation| {
        mask | match operation {
            ModuleBlobOperationV1::Write => 1,
            ModuleBlobOperationV1::ReadRange => 2,
            ModuleBlobOperationV1::CustodyTransfer => 4,
            ModuleBlobOperationV1::ReleaseCustody => 8,
        }
    })
}

fn decode_operation_mask(mask: i64) -> Vec<ModuleBlobOperationV1> {
    [
        (1, ModuleBlobOperationV1::Write),
        (2, ModuleBlobOperationV1::ReadRange),
        (4, ModuleBlobOperationV1::CustodyTransfer),
        (8, ModuleBlobOperationV1::ReleaseCustody),
    ]
    .into_iter()
    .filter_map(|(bit, operation)| (mask & bit != 0).then_some(operation))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custody_release_keeps_a_distinct_persisted_operation_bit() {
        let operations = [
            ModuleBlobOperationV1::Write,
            ModuleBlobOperationV1::ReadRange,
            ModuleBlobOperationV1::CustodyTransfer,
            ModuleBlobOperationV1::ReleaseCustody,
        ];
        assert_eq!(operation_mask(&operations), 15);
        assert_eq!(decode_operation_mask(15), operations);
    }
}
