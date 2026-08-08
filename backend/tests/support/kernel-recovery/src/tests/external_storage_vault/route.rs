//! Kernel-authorized relay from one external runtime session to Vault.

use std::sync::Arc;

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteV1};

use crate::runtime::external::sessions::ExternalRuntimeSessions;
use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::route::execute_route;
use crate::transport::session::VaultTransportReplayGuard;
use crate::vault::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};

pub(super) struct ExternalStorageVaultRoute {
    data_dir: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    sessions: ExternalRuntimeSessions,
    session_id: String,
    service: VaultService,
    keys: VaultTransportKeyPair,
    replay_guard: VaultTransportReplayGuard,
    kernel_authorization_key: [u8; 65],
}

impl ExternalStorageVaultRoute {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        data_dir: std::path::PathBuf,
        store: Arc<SqliteControlStore>,
        sessions: ExternalRuntimeSessions,
        session_id: String,
        service: VaultService,
        keys: VaultTransportKeyPair,
        kernel_authorization_key: [u8; 65],
    ) -> Self {
        Self {
            data_dir,
            store,
            sessions,
            session_id,
            service,
            keys,
            replay_guard: super::fixture::replay_guard(),
            kernel_authorization_key,
        }
    }

    fn relay(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
        let generation = self.service.runtime_generation();
        let mut route = self
            .sessions
            .authorize_vault_route(&self.store, &self.session_id, generation, route)
            .map_err(|_| StorageVaultRouteFailureV1::Rejected)?
            .into_route();
        crate::platform::vault::ciphertext_route::sign_for_kernel(
            &self.data_dir,
            self.store.snapshot().instance_id(),
            &mut route,
        )
        .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?;
        execute_route(
            &mut self.service,
            &self.keys,
            &mut self.replay_guard,
            self.kernel_authorization_key,
            route,
            1,
        )
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)
    }
}

impl StorageVaultRoutePortV1 for ExternalStorageVaultRoute {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl std::future::Future<
        Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>,
    > + Send {
        let response = self.relay(route);
        async move { response }
    }
}
