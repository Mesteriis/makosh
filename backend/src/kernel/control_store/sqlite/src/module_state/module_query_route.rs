use std::collections::BTreeSet;

use makosh_kernel_control_store::ModuleQueryContractV1;
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

pub(crate) fn validate_module_query_contracts(
    registration: &makosh_kernel_control_store::ModuleRegistration,
    capabilities: &[String],
    contracts: &[ModuleQueryContractV1],
    provider_routes: bool,
) -> Result<(), StoreError> {
    let capabilities = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
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
                && (!provider_routes || contract.owner() == registration.owner_id())
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
        .ok_or(StoreError::InvalidModuleQueryContract)
}

pub(crate) fn insert_module_query_rpc_routes(
    connection: &Connection,
    routes: &[ModuleQueryContractV1],
) -> Result<(), StoreError> {
    insert_contracts(
        connection,
        "makosh_kernel_module_query_rpc_route_request",
        routes,
    )
}

pub(crate) fn insert_module_contract_dependencies(
    connection: &Connection,
    dependencies: &[ModuleQueryContractV1],
) -> Result<(), StoreError> {
    insert_contracts(
        connection,
        "makosh_kernel_module_contract_dependency",
        dependencies,
    )
}

fn insert_contracts(
    connection: &Connection,
    table: &str,
    contracts: &[ModuleQueryContractV1],
) -> Result<(), StoreError> {
    let sql = format!(
        "INSERT INTO {table}
         (registration_id, capability_id, contract_owner, contract_name, contract_major,
          contract_revision, contract_schema_sha256)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    );
    for contract in contracts {
        connection.execute(
            &sql,
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
    pub fn approved_module_query_rpc_routes(
        &self,
    ) -> Result<Vec<ModuleQueryContractV1>, StoreError> {
        self.with_connection(|connection| {
            read_contracts(
                connection,
                "makosh_kernel_module_query_rpc_route_request",
                None,
            )
        })
    }

    pub fn module_contract_dependencies(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleQueryContractV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_contracts(
                connection,
                "makosh_kernel_module_contract_dependency",
                Some((&registration_id, &capability_id)),
            )
        })
    }
}

fn read_contracts(
    connection: &Connection,
    table: &str,
    filter: Option<(&str, &str)>,
) -> Result<Vec<ModuleQueryContractV1>, StoreError> {
    let where_clause = if filter.is_some() {
        "WHERE route.registration_id = ?1 AND route.capability_id = ?2"
    } else {
        "WHERE registration.state = 'approved' AND capability.approved = 1"
    };
    let sql = format!(
        "SELECT route.registration_id, route.capability_id, route.contract_owner,
                route.contract_name, route.contract_major, route.contract_revision,
                route.contract_schema_sha256
         FROM {table} AS route
         JOIN makosh_kernel_module_registration AS registration
           ON registration.registration_id = route.registration_id
         JOIN makosh_kernel_module_registration_capability AS capability
           ON capability.registration_id = route.registration_id
          AND capability.capability_id = route.capability_id
         {where_clause}
         ORDER BY route.registration_id, route.capability_id, route.contract_owner,
                  route.contract_name, route.contract_major, route.contract_revision"
    );
    let mut statement = connection.prepare(&sql)?;
    let decode = |row: &rusqlite::Row<'_>| -> Result<ModuleQueryContractV1, rusqlite::Error> {
        let major: i64 = row.get(4)?;
        let revision: i64 = row.get(5)?;
        let digest: Vec<u8> = row.get(6)?;
        Ok(ModuleQueryContractV1::new(
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            u32::try_from(major).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, major))?,
            u32::try_from(revision)
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, revision))?,
            digest.try_into().map_err(|digest: Vec<u8>| {
                rusqlite::Error::FromSqlConversionFailure(
                    digest.len(),
                    rusqlite::types::Type::Blob,
                    "query contract digest must contain 32 bytes".into(),
                )
            })?,
        ))
    };
    let rows = match filter {
        Some((registration_id, capability_id)) => {
            statement.query_map(params![registration_id, capability_id], decode)?
        }
        None => statement.query_map([], decode)?,
    };
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::from)
}
