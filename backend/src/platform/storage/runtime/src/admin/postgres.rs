//! Bounded PostgreSQL platform-schema reconciliation at Storage startup.

use makosh_storage_postgres::{
    PLATFORM_ADMIN_USERNAME, PostgresAdminConnectorV1, StorageRoleSpecV1, ensure_platform_schemas,
    ensure_storage_roles, read_readiness, set_runtime_role_password,
};
use makosh_storage_protocol::{StorageBindingV1, v1::StorageRuntimeTopologyV1};
use zeroize::Zeroizing;

pub(crate) struct RuntimeRoleCredentialV1 {
    binding: StorageBindingV1,
    password: Zeroizing<Vec<u8>>,
}

impl RuntimeRoleCredentialV1 {
    pub(crate) fn new(
        binding: StorageBindingV1,
        password: Zeroizing<Vec<u8>>,
    ) -> Result<Self, String> {
        (!password.is_empty())
            .then_some(Self { binding, password })
            .ok_or_else(|| "Storage runtime credential is unavailable".to_owned())
    }

    pub(crate) fn binding(&self) -> &StorageBindingV1 {
        &self.binding
    }
}

pub(crate) fn verify_platform_postgres(
    topology: &StorageRuntimeTopologyV1,
    credential: &Zeroizing<Vec<u8>>,
) -> Result<(), String> {
    let port = postgres_port(topology)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage PostgreSQL runtime is unavailable".to_owned())?;
    runtime.block_on(reconcile(topology, port, credential))
}

pub(crate) fn reconcile_authorized_roles(
    topology: &StorageRuntimeTopologyV1,
    credential: &Zeroizing<Vec<u8>>,
    runtime_credentials: &[RuntimeRoleCredentialV1],
) -> Result<(), String> {
    if runtime_credentials.is_empty() {
        return Ok(());
    }
    let port = postgres_port(topology)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage PostgreSQL runtime is unavailable".to_owned())?;
    runtime.block_on(reconcile_roles(
        topology,
        port,
        credential,
        runtime_credentials,
    ))
}

async fn reconcile(
    topology: &StorageRuntimeTopologyV1,
    port: u16,
    credential: &Zeroizing<Vec<u8>>,
) -> Result<(), String> {
    let connector = PostgresAdminConnectorV1::connect_with_password(
        &topology.postgres_host,
        port,
        &topology.database_id,
        PLATFORM_ADMIN_USERNAME,
        credential,
    )
    .await
    .map_err(|_| "Storage PostgreSQL admin authentication is unavailable".to_owned())?;
    ensure_platform_schemas(&connector)
        .await
        .map_err(|_| "Storage PostgreSQL bootstrap is unavailable".to_owned())?;
    let readiness = read_readiness(&connector)
        .await
        .map_err(|_| "Storage PostgreSQL readiness is unavailable".to_owned())?;
    (readiness.database_id() == topology.database_id)
        .then_some(())
        .ok_or_else(|| "Storage PostgreSQL identity is unavailable".to_owned())
}

async fn reconcile_roles(
    topology: &StorageRuntimeTopologyV1,
    port: u16,
    credential: &Zeroizing<Vec<u8>>,
    runtime_credentials: &[RuntimeRoleCredentialV1],
) -> Result<(), String> {
    let connector = connect(topology, port, credential).await?;
    ensure_platform_schemas(&connector)
        .await
        .map_err(|_| "Storage PostgreSQL bootstrap is unavailable".to_owned())?;
    for runtime_credential in runtime_credentials {
        let spec = StorageRoleSpecV1::platform_binding(runtime_credential.binding.clone())
            .map_err(|_| "Storage role specification is invalid".to_owned())?;
        ensure_storage_roles(&connector, &spec)
            .await
            .map_err(|error| {
                if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_storage_role_reconciliation_error={error:?}");
                }
                "Storage role reconciliation is unavailable".to_owned()
            })?;
        set_runtime_role_password(&connector, &spec, &runtime_credential.password)
            .await
            .map_err(|_| "Storage role credential is unavailable".to_owned())?;
    }
    Ok(())
}

pub(crate) async fn connect_platform(
    topology: &StorageRuntimeTopologyV1,
    credential: &Zeroizing<Vec<u8>>,
) -> Result<PostgresAdminConnectorV1, String> {
    connect(topology, postgres_port(topology)?, credential).await
}

async fn connect(
    topology: &StorageRuntimeTopologyV1,
    port: u16,
    credential: &Zeroizing<Vec<u8>>,
) -> Result<PostgresAdminConnectorV1, String> {
    PostgresAdminConnectorV1::connect_with_password(
        &topology.postgres_host,
        port,
        &topology.database_id,
        PLATFORM_ADMIN_USERNAME,
        credential,
    )
    .await
    .map_err(|_| "Storage PostgreSQL admin authentication is unavailable".to_owned())
}

fn postgres_port(topology: &StorageRuntimeTopologyV1) -> Result<u16, String> {
    u16::try_from(topology.postgres_port)
        .map_err(|_| "Storage PostgreSQL admin endpoint is invalid".to_owned())
}
