//! Bounded PgBouncer admin readiness using a Vault-resolved platform credential.

use makosh_storage_pgbouncer::{
    PLATFORM_ADMIN_USERNAME, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    verify_admin_connection,
};
use makosh_storage_protocol::v1::StorageRuntimeTopologyV1;
use zeroize::Zeroizing;

pub(crate) fn verify_platform_admin(
    topology: &StorageRuntimeTopologyV1,
    credential_bytes: &Zeroizing<Vec<u8>>,
) -> Result<(), String> {
    let endpoint = admin_endpoint(topology)?;
    let credential = admin_credential(credential_bytes)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage PgBouncer admin runtime is unavailable".to_owned())?;
    runtime
        .block_on(verify_admin_connection(&endpoint, &credential))
        .map_err(|_| "Storage PgBouncer admin authentication is unavailable".to_owned())
}

pub(crate) fn admin_endpoint(
    topology: &StorageRuntimeTopologyV1,
) -> Result<PgBouncerAdminEndpointV1, String> {
    let port = u16::try_from(topology.pgbouncer_port)
        .map_err(|_| "Storage PgBouncer admin endpoint is invalid".to_owned())?;
    PgBouncerAdminEndpointV1::new(topology.pgbouncer_host.clone(), port)
        .map_err(|_| "Storage PgBouncer admin endpoint is invalid".to_owned())
}

pub(crate) fn admin_credential(
    credential_bytes: &Zeroizing<Vec<u8>>,
) -> Result<PgBouncerAdminCredentialV1, String> {
    let password = String::from_utf8(credential_bytes.to_vec())
        .map_err(|_| "Storage PgBouncer admin credential is invalid".to_owned())?;
    if password.is_empty() || password.len() > 4 * 1024 || password.contains(['\0', '\r', '\n']) {
        return Err("Storage PgBouncer admin credential is invalid".to_owned());
    }
    PgBouncerAdminCredentialV1::new(PLATFORM_ADMIN_USERNAME.to_owned(), Zeroizing::new(password))
        .map_err(|_| "Storage PgBouncer admin credential is invalid".to_owned())
}
