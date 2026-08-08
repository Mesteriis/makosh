use std::collections::BTreeSet;

use makosh_kernel_control_store::ModuleRequestContractV1;
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

pub(crate) fn validate_module_request_contracts(
    registration: &makosh_kernel_control_store::ModuleRegistration,
    capabilities: &[String],
    contracts: &[ModuleRequestContractV1],
) -> Result<(), StoreError> {
    let capabilities = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    // Descriptor admission has already enforced ADR-0354: only an Integration
    // may implement a foreign-owned request contract. The private store keeps
    // the exact provider inventory; it must not rewrite contract ownership to
    // the registration owner.
    let mut exact_contracts = BTreeSet::new();
    contracts
        .iter()
        .all(|contract| {
            contract.registration_id() == registration.registration_id()
                && capabilities.contains(contract.capability_id())
                && valid_capability_ids(&[contract.capability_id().to_owned()])
                && valid_identity_token(contract.owner())
                && valid_identity_token(contract.name())
                && contract.major() > 0
                && contract.revision() > 0
                && exact_contracts.insert((
                    contract.capability_id(),
                    contract.owner(),
                    contract.name(),
                    contract.major(),
                    contract.revision(),
                    contract.schema_sha256(),
                ))
        })
        .then_some(())
        .ok_or(StoreError::InvalidModuleRequestContract)
}

pub(crate) fn insert_module_request_rpc_routes(
    connection: &Connection,
    routes: &[ModuleRequestContractV1],
) -> Result<(), StoreError> {
    for contract in routes {
        connection.execute(
            "INSERT INTO makosh_kernel_module_request_rpc_route_request
             (registration_id, capability_id, contract_owner, contract_name, contract_major,
              contract_revision, contract_schema_sha256)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                contract.registration_id(),
                contract.capability_id(),
                contract.owner(),
                contract.name(),
                i64::from(contract.major()),
                i64::from(contract.revision()),
                contract.schema_sha256().as_slice(),
            ],
        )?;
    }
    Ok(())
}

impl SqliteControlStore {
    pub fn approved_module_request_rpc_routes(
        &self,
    ) -> Result<Vec<ModuleRequestContractV1>, StoreError> {
        self.with_connection(|connection| read_contracts(connection))
    }
}

fn read_contracts(connection: &Connection) -> Result<Vec<ModuleRequestContractV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT route.registration_id, route.capability_id, route.contract_owner,
                route.contract_name, route.contract_major, route.contract_revision,
                route.contract_schema_sha256
         FROM makosh_kernel_module_request_rpc_route_request AS route
         JOIN makosh_kernel_module_registration AS registration
           ON registration.registration_id = route.registration_id
         JOIN makosh_kernel_module_registration_capability AS capability
           ON capability.registration_id = route.registration_id
          AND capability.capability_id = route.capability_id
         WHERE registration.state = 'approved' AND capability.approved = 1
         ORDER BY route.registration_id, route.capability_id, route.contract_owner,
                  route.contract_name, route.contract_major, route.contract_revision",
    )?;
    statement
        .query_map([], |row| {
            let major: i64 = row.get(4)?;
            let revision: i64 = row.get(5)?;
            let digest: Vec<u8> = row.get(6)?;
            Ok(ModuleRequestContractV1::new(
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                u32::try_from(major)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, major))?,
                u32::try_from(revision)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, revision))?,
                digest.try_into().map_err(|digest: Vec<u8>| {
                    rusqlite::Error::FromSqlConversionFailure(
                        digest.len(),
                        rusqlite::types::Type::Blob,
                        "request contract digest must contain 32 bytes".into(),
                    )
                })?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::from)
}
