//! Applies only Kernel-staged, fenced bindings to the private PgBouncer include.

use makosh_storage_pgbouncer::{
    PLATFORM_ADMIN_USERNAME, PgBouncerAuthEntryV1, PgBouncerAuthFileV1,
    PgBouncerDatabaseConfigFileV1, PgBouncerRuntimeConfigV1, PoolAliasV1,
    TokioPostgresPgBouncerAdminPortV1, database_is_configured, reload_configuration,
};
use makosh_storage_postgres::{StorageRoleSpecV1, read_runtime_role_scram_verifier};
use makosh_storage_protocol::v1::{
    StorageBindingV1, StorageRuntimeConfigurationV1, StorageRuntimeTopologyV1,
};
use makosh_storage_protocol::validation::validate_storage_runtime_configuration;
use std::time::Duration;
use zeroize::Zeroizing;

use super::{RuntimeRoleCredentialV1, admin_credential, admin_endpoint, connect_platform};

// PgBouncer may briefly reject RELOAD while its private include is being
// atomically replaced. Keep the bounded reconciliation window long enough for
// an independently supervised pooler to observe the new inode.
const RELOAD_ATTEMPTS: u8 = 30;
const RELOAD_RETRY_DELAY: Duration = Duration::from_millis(200);

pub(crate) fn apply_authorized_bindings(
    configuration: &StorageRuntimeConfigurationV1,
    pgbouncer_credential_bytes: &Zeroizing<Vec<u8>>,
    postgres_credential_bytes: &Zeroizing<Vec<u8>>,
    runtime_credentials: &[RuntimeRoleCredentialV1],
) -> Result<(), String> {
    validate_storage_runtime_configuration(configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    if configuration.desired_bindings.is_empty() {
        return Ok(());
    }
    let topology = configuration
        .topology
        .as_ref()
        .ok_or_else(|| "Storage runtime configuration is invalid".to_owned())?;
    let endpoint = admin_endpoint(topology)?;
    let credential = admin_credential(pgbouncer_credential_bytes)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage PgBouncer admin runtime is unavailable".to_owned())?;
    let entries = database_entries(topology, &configuration.desired_bindings)?;
    runtime.block_on(replace_auth_and_reload(AuthorizedPoolReplaceInputV1 {
        configuration,
        topology,
        postgres_credential: postgres_credential_bytes,
        pgbouncer_credential_bytes,
        runtime_credentials,
        endpoint: &endpoint,
        pgbouncer_credential: &credential,
        entries: &entries,
    }))
}

// Live storage composition tests stage the database include before the
// credential-bound authentication reconciliation runs.
#[allow(dead_code)]
pub(crate) fn apply_staged_pool_configuration(
    configuration: &StorageRuntimeConfigurationV1,
    pgbouncer_credential_bytes: &Zeroizing<Vec<u8>>,
) -> Result<(), String> {
    validate_storage_runtime_configuration(configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    if configuration.desired_bindings.is_empty() {
        return Ok(());
    }
    let topology = configuration
        .topology
        .as_ref()
        .ok_or_else(|| "Storage runtime configuration is invalid".to_owned())?;
    let entries = database_entries(topology, &configuration.desired_bindings)?;
    PgBouncerDatabaseConfigFileV1::replace(
        std::path::Path::new(&configuration.pgbouncer_database_config_path),
        &entries,
    )
    .map_err(|_| "Storage PgBouncer database configuration is unavailable".to_owned())?;
    let endpoint = admin_endpoint(topology)?;
    let credential = admin_credential(pgbouncer_credential_bytes)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage PgBouncer admin runtime is unavailable".to_owned())?;
    runtime.block_on(reload_and_verify(&endpoint, &credential, &entries))
}

struct AuthorizedPoolReplaceInputV1<'a> {
    configuration: &'a StorageRuntimeConfigurationV1,
    topology: &'a StorageRuntimeTopologyV1,
    postgres_credential: &'a Zeroizing<Vec<u8>>,
    pgbouncer_credential_bytes: &'a Zeroizing<Vec<u8>>,
    runtime_credentials: &'a [RuntimeRoleCredentialV1],
    endpoint: &'a makosh_storage_pgbouncer::PgBouncerAdminEndpointV1,
    pgbouncer_credential: &'a makosh_storage_pgbouncer::PgBouncerAdminCredentialV1,
    entries: &'a [PgBouncerRuntimeConfigV1],
}

async fn replace_auth_and_reload(input: AuthorizedPoolReplaceInputV1<'_>) -> Result<(), String> {
    let auth_entries = auth_entries(
        input.topology,
        input.postgres_credential,
        input.pgbouncer_credential_bytes,
        input.runtime_credentials,
    )
    .await?;
    PgBouncerAuthFileV1::replace(
        std::path::Path::new(&input.configuration.pgbouncer_auth_file_path),
        auth_entries,
    )
    .map_err(|_| "Storage PgBouncer authentication configuration is unavailable".to_owned())?;
    PgBouncerDatabaseConfigFileV1::replace(
        std::path::Path::new(&input.configuration.pgbouncer_database_config_path),
        input.entries,
    )
    .map_err(|_| "Storage PgBouncer database configuration is unavailable".to_owned())?;
    reload_and_verify(input.endpoint, input.pgbouncer_credential, input.entries).await
}

async fn auth_entries(
    topology: &StorageRuntimeTopologyV1,
    postgres_credential: &Zeroizing<Vec<u8>>,
    pgbouncer_credential: &Zeroizing<Vec<u8>>,
    runtime_credentials: &[RuntimeRoleCredentialV1],
) -> Result<Vec<PgBouncerAuthEntryV1>, String> {
    let connector = connect_platform(topology, postgres_credential).await?;
    let mut entries = Vec::with_capacity(runtime_credentials.len() + 1);
    entries.push(
        PgBouncerAuthEntryV1::pooler_admin(PLATFORM_ADMIN_USERNAME, pgbouncer_credential).map_err(
            |_| "Storage PgBouncer authentication configuration is unavailable".to_owned(),
        )?,
    );
    for credential in runtime_credentials {
        let spec = StorageRoleSpecV1::platform_binding(credential.binding().clone())
            .map_err(|_| "Storage role specification is invalid".to_owned())?;
        let verifier = read_runtime_role_scram_verifier(&connector, &spec)
            .await
            .map_err(|_| "Storage runtime SCRAM verifier is unavailable".to_owned())?;
        entries.push(
            PgBouncerAuthEntryV1::runtime_scram(spec.runtime_principal(), verifier)
                .map_err(|_| "Storage runtime SCRAM verifier is unavailable".to_owned())?,
        );
    }
    Ok(entries)
}

fn database_entries(
    topology: &StorageRuntimeTopologyV1,
    bindings: &[StorageBindingV1],
) -> Result<Vec<PgBouncerRuntimeConfigV1>, String> {
    bindings
        .iter()
        .map(|binding| database_entry(topology, binding))
        .collect()
}

fn database_entry(
    topology: &StorageRuntimeTopologyV1,
    binding: &StorageBindingV1,
) -> Result<PgBouncerRuntimeConfigV1, String> {
    let alias = PoolAliasV1::new(&binding.registration_id, binding.runtime_generation)
        .map_err(|_| "Storage binding pool alias is invalid".to_owned())?;
    let budget = binding
        .effective_budgets
        .as_ref()
        .ok_or_else(|| "Storage binding budget is invalid".to_owned())?;
    let port = u16::try_from(topology.pgbouncer_postgres_port)
        .map_err(|_| "Storage PostgreSQL endpoint is invalid".to_owned())?;
    PgBouncerRuntimeConfigV1::new(
        alias,
        topology.pgbouncer_postgres_host.clone(),
        port,
        topology.database_id.clone(),
        binding.runtime_principal.clone(),
        u16::try_from(budget.max_connections)
            .map_err(|_| "Storage binding budget is invalid".to_owned())?,
    )
    .map_err(|_| "Storage binding configuration is invalid".to_owned())
}

async fn reload_and_verify(
    endpoint: &makosh_storage_pgbouncer::PgBouncerAdminEndpointV1,
    credential: &makosh_storage_pgbouncer::PgBouncerAdminCredentialV1,
    entries: &[PgBouncerRuntimeConfigV1],
) -> Result<(), String> {
    for attempt in 1..=RELOAD_ATTEMPTS {
        match reload_and_verify_once(endpoint, credential, entries).await {
            Ok(()) => return Ok(()),
            Err(_) if attempt < RELOAD_ATTEMPTS => {
                tokio::time::sleep(RELOAD_RETRY_DELAY).await;
            }
            Err(error) => return Err(error),
        }
    }
    Err("Storage PgBouncer configuration reload is unavailable".to_owned())
}

async fn reload_and_verify_once(
    endpoint: &makosh_storage_pgbouncer::PgBouncerAdminEndpointV1,
    credential: &makosh_storage_pgbouncer::PgBouncerAdminCredentialV1,
    entries: &[PgBouncerRuntimeConfigV1],
) -> Result<(), String> {
    let mut admin = TokioPostgresPgBouncerAdminPortV1::connect(endpoint, credential)
        .await
        .map_err(|_| "Storage PgBouncer admin authentication is unavailable".to_owned())?;
    reload_configuration(&mut admin)
        .await
        .map_err(|_| "Storage PgBouncer configuration reload is unavailable".to_owned())?;
    for entry in entries {
        if !database_is_configured(endpoint, credential, entry.alias())
            .await
            .map_err(|_| "Storage PgBouncer catalog is unavailable".to_owned())?
        {
            return Err("Storage PgBouncer configuration is unavailable".to_owned());
        }
    }
    Ok(())
}
