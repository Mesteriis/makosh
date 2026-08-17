use rusqlite::{Connection, OptionalExtension, Transaction};

use crate::StoreError;

mod v01_to_v02;
mod v02_to_v03;
mod v03_to_v04;
mod v04_to_v05;
mod v05_to_v06;
mod v06_to_v07;
mod v07_to_v08;
mod v08_to_v09;
mod v09_to_v10;
mod v10_to_v11;
mod v11_to_v12;
mod v12_to_v13;
mod v13_to_v14;
mod v14_to_v15;
mod v15_to_v16;
mod v16_to_v17;
mod v17_to_v18;
mod v18_to_v19;
mod v19_to_v20;
mod v20_to_v21;
mod v21_to_v22;
mod v22_to_v23;
mod v23_to_v24;
mod v24_to_v25;
mod v25_to_v26;
mod v26_to_v27;
mod v27_to_v28;
mod v28_to_v29;
mod v29_to_v30;
mod v30_to_v31;
mod v31_to_v32;
mod v32_to_v33;
mod v33_to_v34;
mod v34_to_v35;
mod v35_to_v36;
mod v36_to_v37;
mod v37_to_v38;
mod v38_to_v39;
mod v39_to_v40;
mod v40_to_v41;
mod v41_to_v42;
mod v42_to_v43;
mod v43_to_v44;
mod v44_to_v45;
mod v45_to_v46;
mod v46_to_v47;
mod v47_to_v48;
mod v48_to_v49;
mod v49_to_v50;

pub const SCHEMA_VERSION: i64 = 50;

pub fn migrate_schema(connection: &Connection) -> Result<(), StoreError> {
    loop {
        let version = schema_version(connection)?;
        if version == SCHEMA_VERSION {
            assert_schema_for_version(connection, version)?;
            return Ok(());
        }
        if !(1..SCHEMA_VERSION).contains(&version) {
            return Err(StoreError::UnsupportedSchema(version));
        }

        let transaction = connection.unchecked_transaction()?;
        apply_step(version, &transaction)?;
        let actual = schema_version(&transaction)?;
        let expected = version + 1;
        if actual != expected {
            return Err(StoreError::MigrationInvariant { expected, actual });
        }
        assert_schema_for_version(&transaction, actual)?;
        transaction.commit()?;
    }
}

fn assert_schema_for_version(connection: &Connection, version: i64) -> Result<(), StoreError> {
    if !(1..=SCHEMA_VERSION).contains(&version) || !base_schema_exists(connection)? {
        return Err(StoreError::MigrationSchemaAssertion { version });
    }
    for required_version in 2..=version {
        if version >= 36 && required_version == 34 {
            continue;
        }
        if !version_feature_exists(connection, required_version)? {
            return Err(StoreError::MigrationSchemaAssertion { version });
        }
    }
    Ok(())
}

fn base_schema_exists(connection: &Connection) -> Result<bool, StoreError> {
    for column in ["singleton", "schema_version", "instance_id", "generation"] {
        if !column_exists(connection, "makosh_kernel_control_store_metadata", column)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn version_feature_exists(connection: &Connection, version: i64) -> Result<bool, StoreError> {
    match version {
        2 => Ok(column_exists(
            connection,
            "makosh_kernel_control_store_metadata",
            "identity_epoch",
        )? && column_exists(
            connection,
            "makosh_kernel_control_store_metadata",
            "grant_epoch",
        )?),
        3 => table_exists(connection, "makosh_kernel_initial_owner_identity"),
        4 => table_exists(connection, "makosh_kernel_module_registration"),
        5 => table_exists(connection, "makosh_kernel_module_registration_capability"),
        6 => table_exists(connection, "makosh_kernel_external_runtime_attestation"),
        7 => table_exists(connection, "makosh_kernel_settings_schema_binding"),
        8 => table_exists(connection, "makosh_kernel_settings_desired_snapshot"),
        9 => Ok(column_exists(
            connection,
            "makosh_kernel_settings_schema_binding",
            "apply_state",
        )? && column_exists(
            connection,
            "makosh_kernel_settings_schema_binding",
            "sanitized_reason_code",
        )?),
        10 => table_exists(connection, "makosh_kernel_owner_pinned_artifact_binding"),
        11 => table_exists(connection, "makosh_kernel_settings_schema_artifact"),
        12 => table_exists(connection, "makosh_kernel_external_runtime_identity"),
        13 => table_exists(connection, "makosh_kernel_server_bootstrap_pairing"),
        14 => Ok(
            table_exists(connection, "makosh_kernel_bundled_managed_launch_binding")?
                && table_exists(connection, "makosh_kernel_managed_launch_record")?,
        ),
        15 => Ok(
            table_exists(connection, "makosh_kernel_platform_managed_process_binding")?
                && table_exists(connection, "makosh_kernel_platform_managed_process_launch")?,
        ),
        version @ 16..=33 => platform_storage_feature_exists(connection, version),
        34 => table_exists(connection, "makosh_kernel_operator_settings"),
        35 => table_exists(connection, "makosh_kernel_operation_journal"),
        36 => table_exists(connection, "makosh_kernel_operator_settings").map(|exists| !exists),
        37 => table_exists(connection, "makosh_kernel_module_vault_purpose_request"),
        38 => column_exists(
            connection,
            "makosh_kernel_module_vault_purpose_request",
            "key_schema_revision",
        ),
        39 => table_exists(connection, "makosh_kernel_module_client_rpc_route_request"),
        40 => Ok(columns_exist(
            connection,
            "makosh_kernel_module_blob_quota_request",
            &["custody_scope_id", "allowed_operations"],
        )?),
        41 => table_exists(connection, "makosh_kernel_bundled_artifact_proposal"),
        42 => table_exists(connection, "makosh_kernel_module_client_blob_route_request"),
        43 => Ok(table_definition_contains(
            connection,
            "makosh_kernel_module_client_blob_route_request",
            "max_response_bytes BETWEEN 1 AND 25165824",
        )? || table_definition_contains(
            connection,
            "makosh_kernel_module_client_blob_route_request",
            "max_response_bytes BETWEEN 1 AND 33554432",
        )? || table_definition_contains(
            connection,
            "makosh_kernel_module_client_blob_route_request",
            "max_response_bytes BETWEEN 1 AND 4294967296",
        )?),
        44 => Ok(
            table_exists(connection, "makosh_kernel_settings_configuration_target")?
                && columns_exist(
                    connection,
                    "makosh_kernel_settings_desired_snapshot",
                    &["registration_id", "configuration_instance_id"],
                )?,
        ),
        45 => Ok(
            table_exists(connection, "makosh_kernel_module_query_rpc_route_request")?
                && table_exists(connection, "makosh_kernel_module_contract_dependency")?,
        ),
        46 => table_exists(
            connection,
            "makosh_kernel_module_client_realtime_route_request",
        ),
        47 => table_exists(connection, "makosh_kernel_module_request_rpc_route_request"),
        48 => table_definition_contains(
            connection,
            "makosh_kernel_module_blob_quota_request",
            "allowed_operations BETWEEN 0 AND 15",
        ),
        49 => Ok(table_definition_contains(
            connection,
            "makosh_kernel_module_client_blob_route_request",
            "max_response_bytes BETWEEN 1 AND 33554432",
        )? || table_definition_contains(
            connection,
            "makosh_kernel_module_client_blob_route_request",
            "max_response_bytes BETWEEN 1 AND 4294967296",
        )?),
        50 => table_definition_contains(
            connection,
            "makosh_kernel_module_client_blob_route_request",
            "max_response_bytes BETWEEN 1 AND 4294967296",
        ),
        _ => Ok(false),
    }
}

fn platform_storage_feature_exists(
    connection: &Connection,
    version: i64,
) -> Result<bool, StoreError> {
    match version {
        16 => table_exists(connection, "makosh_kernel_platform_storage_topology"),
        17 => storage_endpoint_columns_exist(connection),
        18 => table_exists(connection, "makosh_kernel_module_storage_request"),
        19 => column_exists(
            connection,
            "makosh_kernel_managed_launch_record",
            "runtime_instance_id",
        ),
        20 => table_exists(connection, "makosh_kernel_platform_storage_binding"),
        21 => table_exists(connection, "makosh_kernel_platform_storage_bundle"),
        22 => column_exists(
            connection,
            "makosh_kernel_platform_storage_binding",
            "state",
        ),
        23 => table_exists(connection, "makosh_kernel_module_event_route_request"),
        24 => table_exists(connection, "makosh_kernel_module_blob_quota_request"),
        25 => table_exists(connection, "makosh_kernel_module_event_delivery_policy"),
        26 => table_exists(
            connection,
            "makosh_kernel_platform_events_authority_configuration",
        ),
        27 => Ok(
            table_exists(connection, "makosh_kernel_platform_event_hub_topology")?
                && table_exists(connection, "makosh_kernel_platform_event_stream_budget")?,
        ),
        28 => event_hub_connection_columns_exist(connection),
        29 => table_exists(connection, "makosh_kernel_browser_device_identity"),
        30 => table_exists(connection, "makosh_kernel_module_scheduler_job_request"),
        31 => pgbouncer_backend_endpoint_columns_exist(connection),
        32 => column_exists(
            connection,
            "makosh_kernel_browser_device_identity",
            "browser_key_public_key",
        ),
        33 => Ok(columns_exist(
            connection,
            "makosh_kernel_browser_device_identity",
            &["backup_eligible", "backup_state"],
        )?),
        _ => Ok(false),
    }
}

fn storage_endpoint_columns_exist(connection: &Connection) -> Result<bool, StoreError> {
    columns_exist(
        connection,
        "makosh_kernel_platform_storage_topology",
        &[
            "postgres_host",
            "postgres_port",
            "pgbouncer_host",
            "pgbouncer_port",
        ],
    )
}

fn pgbouncer_backend_endpoint_columns_exist(connection: &Connection) -> Result<bool, StoreError> {
    columns_exist(
        connection,
        "makosh_kernel_platform_storage_topology",
        &["pgbouncer_backend_host", "pgbouncer_backend_port"],
    )
}

fn event_hub_connection_columns_exist(connection: &Connection) -> Result<bool, StoreError> {
    columns_exist(
        connection,
        "makosh_kernel_platform_event_hub_topology",
        &["nats_endpoint", "nats_username", "credential_revision"],
    )
}

fn columns_exist(
    connection: &Connection,
    table: &str,
    columns: &[&str],
) -> Result<bool, StoreError> {
    columns.iter().try_fold(true, |exists, column| {
        Ok(exists && column_exists(connection, table, column)?)
    })
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool, StoreError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?1)",
            [table],
            |row| row.get(0),
        )
        .map_err(StoreError::from)
}

fn table_definition_contains(
    connection: &Connection,
    table: &str,
    fragment: &str,
) -> Result<bool, StoreError> {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM sqlite_schema
                WHERE type = 'table' AND name = ?1 AND instr(sql, ?2) > 0
             )",
            [table, fragment],
            |row| row.get(0),
        )
        .map_err(StoreError::from)
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> Result<bool, StoreError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info(?1) WHERE name=?2)",
            [table, column],
            |row| row.get(0),
        )
        .map_err(StoreError::from)
}

fn schema_version(connection: &Connection) -> Result<i64, StoreError> {
    connection
        .query_row(
            "SELECT schema_version FROM makosh_kernel_control_store_metadata WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or(StoreError::MissingMetadata)
}

fn apply_step(version: i64, transaction: &Transaction<'_>) -> Result<(), StoreError> {
    match version {
        1 => v01_to_v02::apply(transaction),
        2 => v02_to_v03::apply(transaction),
        3 => v03_to_v04::apply(transaction),
        4 => v04_to_v05::apply(transaction),
        5 => v05_to_v06::apply(transaction),
        6 => v06_to_v07::apply(transaction),
        7 => v07_to_v08::apply(transaction),
        8 => v08_to_v09::apply(transaction),
        9 => v09_to_v10::apply(transaction),
        10 => v10_to_v11::apply(transaction),
        11 => v11_to_v12::apply(transaction),
        12 => v12_to_v13::apply(transaction),
        13 => v13_to_v14::apply(transaction),
        14 => v14_to_v15::apply(transaction),
        15 => v15_to_v16::apply(transaction),
        16 => v16_to_v17::apply(transaction),
        17 => v17_to_v18::apply(transaction),
        18 => v18_to_v19::apply(transaction),
        19 => v19_to_v20::apply(transaction),
        20 => v20_to_v21::apply(transaction),
        21 => v21_to_v22::apply(transaction),
        22 => v22_to_v23::apply(transaction),
        23 => v23_to_v24::apply(transaction),
        24 => v24_to_v25::apply(transaction),
        25 => v25_to_v26::apply(transaction),
        26 => v26_to_v27::apply(transaction),
        27 => v27_to_v28::apply(transaction),
        28 => v28_to_v29::apply(transaction),
        29 => v29_to_v30::apply(transaction),
        30 => v30_to_v31::apply(transaction),
        31 => v31_to_v32::apply(transaction),
        32 => v32_to_v33::apply(transaction),
        33 => v33_to_v34::apply(transaction),
        34 => v34_to_v35::apply(transaction),
        35 => v35_to_v36::apply(transaction),
        36 => v36_to_v37::apply(transaction),
        37 => v37_to_v38::apply(transaction),
        38 => v38_to_v39::apply(transaction),
        39 => v39_to_v40::apply(transaction),
        40 => v40_to_v41::apply(transaction),
        41 => v41_to_v42::apply(transaction),
        42 => v42_to_v43::apply(transaction),
        43 => v43_to_v44::apply(transaction),
        44 => v44_to_v45::apply(transaction),
        45 => v45_to_v46::apply(transaction),
        46 => v46_to_v47::apply(transaction),
        47 => v47_to_v48::apply(transaction),
        48 => v48_to_v49::apply(transaction),
        49 => v49_to_v50::apply(transaction),
        unsupported => Err(StoreError::UnsupportedSchema(unsupported)),
    }
}
