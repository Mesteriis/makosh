use std::collections::BTreeSet;

use makosh_kernel_control_store::{ModuleRegistration, ModuleVaultPurposeRequestV1};
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

impl SqliteControlStore {
    pub fn module_vault_purpose_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleVaultPurposeRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_vault_purpose_requests(connection, &registration_id, &capability_id)
        })
    }
}

pub(crate) fn validate_vault_purpose_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleVaultPurposeRequestV1],
) -> Result<(), StoreError> {
    let capabilities = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    requests
        .iter()
        .all(|request| {
            request.registration_id() == registration.registration_id()
                && capabilities.contains(request.capability_id())
                && valid_capability_ids(&[request.capability_id().to_owned()])
                && valid_identity_token(request.purpose_id())
                && request.requested_lease_ttl_seconds() > 0
                && valid_purpose_shape(request)
                && seen.insert((
                    request.capability_id(),
                    request.purpose_id(),
                    request.secret_class(),
                    request.action(),
                    request.target_scope(),
                    request.key_schema_revision(),
                ))
        })
        .then_some(())
        .ok_or(StoreError::InvalidModuleVaultPurposeRequest)
}

pub(crate) fn insert_vault_purpose_requests(
    connection: &Connection,
    requests: &[ModuleVaultPurposeRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        connection.execute(
            "INSERT INTO makosh_kernel_module_vault_purpose_request
             (registration_id, capability_id, purpose_id, requested_lease_ttl_seconds, secret_class, action, target_scope, key_schema_revision)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![request.registration_id(), request.capability_id(), request.purpose_id(),
                i64::from(request.requested_lease_ttl_seconds()), i64::from(request.secret_class()),
                i64::from(request.action()), i64::from(request.target_scope()), i64::from(request.key_schema_revision())],
        )?;
    }
    Ok(())
}

fn read_vault_purpose_requests(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Vec<ModuleVaultPurposeRequestV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT purpose_id, requested_lease_ttl_seconds, secret_class, action, target_scope, key_schema_revision
         FROM makosh_kernel_module_vault_purpose_request
         WHERE registration_id = ?1 AND capability_id = ?2
         ORDER BY purpose_id, secret_class, action",
    )?;
    let rows = statement.query_map(params![registration_id, capability_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
        ))
    })?;
    rows.map(|row| {
        let (purpose_id, ttl, secret_class, action, target_scope, key_schema_revision) = row?;
        Ok(ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
            registration_id,
            capability_id,
            purpose_id,
            u16::try_from(ttl).map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
            makosh_kernel_control_store::ModuleVaultPurposePolicyV1 {
                secret_class: u8::try_from(secret_class)
                    .map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
                action: u8::try_from(action)
                    .map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
                target_scope: u8::try_from(target_scope)
                    .map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
                key_schema_revision: u32::try_from(key_schema_revision)
                    .map_err(|_| StoreError::InvalidModuleVaultPurposeRequest)?,
            },
        ))
    })
    .collect()
}

fn valid_purpose_shape(request: &ModuleVaultPurposeRequestV1) -> bool {
    match request.target_scope() {
        1 => {
            (1..=5).contains(&request.secret_class())
                && (1..=6).contains(&request.action())
                && request.key_schema_revision() == 0
        }
        2 => {
            request.secret_class() == 6
                && request.action() == 7
                && request.key_schema_revision() != 0
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::ModuleVaultPurposeRequestV1;

    use super::valid_purpose_shape;

    #[test]
    fn owner_derived_key_requires_its_exact_scope_and_revision() {
        let request = ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
            "registration",
            "search",
            "communications.search.index",
            60,
            makosh_kernel_control_store::ModuleVaultPurposePolicyV1 {
                secret_class: 6,
                action: 7,
                target_scope: 2,
                key_schema_revision: 1,
            },
        );
        assert!(valid_purpose_shape(&request));
        let wrong_scope = ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
            "registration",
            "search",
            "communications.search.index",
            60,
            makosh_kernel_control_store::ModuleVaultPurposePolicyV1 {
                secret_class: 6,
                action: 7,
                target_scope: 1,
                key_schema_revision: 1,
            },
        );
        assert!(!valid_purpose_shape(&wrong_scope));
        let no_revision = ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
            "registration",
            "search",
            "communications.search.index",
            60,
            makosh_kernel_control_store::ModuleVaultPurposePolicyV1 {
                secret_class: 6,
                action: 7,
                target_scope: 2,
                key_schema_revision: 0,
            },
        );
        assert!(!valid_purpose_shape(&no_revision));
    }
}
