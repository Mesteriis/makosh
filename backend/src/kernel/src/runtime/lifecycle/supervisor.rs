//! Owns active managed-child workers for one Kernel process.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, Weak};

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::distribution::staged_contracts::StagedRuntimeContracts;
pub use crate::modules::capability::router::ManagedRuntimeRelay;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobCustodyReleaseHandler, ManagedRuntimeBlobSessionHandler,
    ManagedRuntimeClientRealtimeHandler, ManagedRuntimeEventCredentialHandler,
    ManagedRuntimeExpectation, ManagedRuntimeModuleQueryHandler,
    ManagedRuntimeModuleRequestHandler, ManagedRuntimeOwnerDerivedKeyHandler,
    ManagedRuntimeProviderCredentialHandler, ManagedRuntimeRelayRequest,
    ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;
use makosh_runtime_protocol::managed_control::ManagedControlTransportMajorV1;

#[path = "supervisor/worker.rs"]
mod worker;

use worker::{ActiveWorker, ActiveWorkerInput, new_active_worker, remove_staged_launch};

const MANAGED_RUNTIME_RELAY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const MANAGED_RUNTIME_READY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

type ConfiguredRequestHandlers = (
    Option<Arc<dyn ManagedRuntimeVaultRouteHandler>>,
    Option<Arc<dyn ManagedRuntimeEventCredentialHandler>>,
    Option<Arc<dyn ManagedRuntimeProviderCredentialHandler>>,
    Option<Arc<dyn ManagedRuntimeOwnerDerivedKeyHandler>>,
    Option<Arc<dyn ManagedRuntimeBlobSessionHandler>>,
    Option<Arc<dyn ManagedRuntimeBlobCustodyReleaseHandler>>,
    Option<Arc<dyn ManagedRuntimeModuleQueryHandler>>,
    Option<Arc<dyn ManagedRuntimeModuleRequestHandler>>,
    Option<Arc<dyn ManagedRuntimeClientRealtimeHandler>>,
);

#[derive(Clone)]
pub struct ManagedRuntimeSupervisor {
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct ManagedRuntimeRelayPort {
    inner: Weak<Inner>,
}

struct Inner {
    shutdown_requested: Arc<AtomicBool>,
    workers: Mutex<HashMap<String, ActiveWorker>>,
    failures: Mutex<HashMap<String, String>>,
    event_credential_handler: Mutex<Option<Arc<dyn ManagedRuntimeEventCredentialHandler>>>,
    provider_credential_handler: Mutex<Option<Arc<dyn ManagedRuntimeProviderCredentialHandler>>>,
    owner_derived_key_handler: Mutex<Option<Arc<dyn ManagedRuntimeOwnerDerivedKeyHandler>>>,
    blob_session_handler: Mutex<Option<Arc<dyn ManagedRuntimeBlobSessionHandler>>>,
    blob_custody_release_handler: Mutex<Option<Arc<dyn ManagedRuntimeBlobCustodyReleaseHandler>>>,
    module_query_handler: Mutex<Option<Arc<dyn ManagedRuntimeModuleQueryHandler>>>,
    module_request_handler: Mutex<Option<Arc<dyn ManagedRuntimeModuleRequestHandler>>>,
    client_realtime_handler: Mutex<Option<Arc<dyn ManagedRuntimeClientRealtimeHandler>>>,
    vault_route_handler: Mutex<Option<Arc<dyn ManagedRuntimeVaultRouteHandler>>>,
}

pub(crate) struct ManagedRuntimeLaunchRequest {
    pub registration_id: String,
    pub staged_executable: StagedNativeArtifact,
    pub arguments: Vec<String>,
    pub expectation: ManagedRuntimeExpectation,
    pub policy: ManagedChildExecutionPolicy,
    pub control_transport: ManagedControlTransportMajorV1,
    pub contracts: Option<StagedRuntimeContracts>,
    pub cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl ManagedRuntimeSupervisor {
    #[must_use]
    pub fn new(shutdown_requested: Arc<AtomicBool>) -> Self {
        Self {
            inner: Arc::new(Inner {
                shutdown_requested,
                workers: Mutex::new(HashMap::new()),
                failures: Mutex::new(HashMap::new()),
                event_credential_handler: Mutex::new(None),
                provider_credential_handler: Mutex::new(None),
                owner_derived_key_handler: Mutex::new(None),
                blob_session_handler: Mutex::new(None),
                blob_custody_release_handler: Mutex::new(None),
                module_query_handler: Mutex::new(None),
                module_request_handler: Mutex::new(None),
                client_realtime_handler: Mutex::new(None),
                vault_route_handler: Mutex::new(None),
            }),
        }
    }

    pub fn configure_vault_route_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeVaultRouteHandler>,
    ) -> Result<(), String> {
        if !self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .is_empty()
        {
            return Err(
                "managed runtime Vault route handler must be configured before launch".to_owned(),
            );
        }
        let mut current = self
            .inner
            .vault_route_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?;
        if current.is_some() {
            return Err("managed runtime Vault route handler is already configured".to_owned());
        }
        *current = Some(handler);
        Ok(())
    }

    pub fn configure_event_credential_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeEventCredentialHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.event_credential_handler,
            handler,
            "managed runtime Event credential handler",
        )
    }

    pub fn configure_provider_credential_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeProviderCredentialHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.provider_credential_handler,
            handler,
            "managed runtime provider credential handler",
        )
    }

    pub fn configure_owner_derived_key_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeOwnerDerivedKeyHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.owner_derived_key_handler,
            handler,
            "managed runtime owner-derived key handler",
        )
    }

    pub fn configure_blob_session_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeBlobSessionHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.blob_session_handler,
            handler,
            "managed runtime Blob session handler",
        )
    }

    pub fn configure_blob_custody_release_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeBlobCustodyReleaseHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.blob_custody_release_handler,
            handler,
            "managed runtime Blob custody release handler",
        )
    }

    pub fn configure_module_query_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeModuleQueryHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.module_query_handler,
            handler,
            "managed runtime module query handler",
        )
    }

    pub fn configure_module_request_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeModuleRequestHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.module_request_handler,
            handler,
            "managed runtime module request handler",
        )
    }

    pub fn configure_client_realtime_handler(
        &self,
        handler: Arc<dyn ManagedRuntimeClientRealtimeHandler>,
    ) -> Result<(), String> {
        self.configure_before_launch(
            &self.inner.client_realtime_handler,
            handler,
            "managed runtime ClientRealtime handler",
        )
    }

    #[must_use]
    pub fn relay_port(&self) -> ManagedRuntimeRelayPort {
        ManagedRuntimeRelayPort {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn start(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
    ) -> Result<(), String> {
        self.start_with_arguments(
            registration_id,
            staged_executable,
            Vec::new(),
            expectation,
            policy,
        )
    }

    pub fn start_with_arguments(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        arguments: Vec<String>,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
    ) -> Result<(), String> {
        self.start_with_optional_contracts(ManagedRuntimeLaunchRequest {
            registration_id,
            staged_executable,
            arguments,
            expectation,
            policy,
            control_transport: ManagedControlTransportMajorV1::LegacyV1,
            contracts: None,
            cleanup: None,
        })
    }

    pub fn start_with_arguments_and_contracts(
        &self,
        registration_id: String,
        staged_executable: StagedNativeArtifact,
        arguments: Vec<String>,
        expectation: ManagedRuntimeExpectation,
        policy: ManagedChildExecutionPolicy,
        contracts: StagedRuntimeContracts,
    ) -> Result<(), String> {
        self.start_with_optional_contracts(ManagedRuntimeLaunchRequest {
            registration_id,
            staged_executable,
            arguments,
            expectation,
            policy,
            control_transport: ManagedControlTransportMajorV1::LegacyV1,
            contracts: Some(contracts),
            cleanup: None,
        })
    }

    pub(crate) fn start_with_arguments_contracts_and_cleanup(
        &self,
        request: ManagedRuntimeLaunchRequest,
    ) -> Result<(), String> {
        self.start_with_optional_contracts(request)
    }

    fn start_with_optional_contracts(
        &self,
        request: ManagedRuntimeLaunchRequest,
    ) -> Result<(), String> {
        let ManagedRuntimeLaunchRequest {
            registration_id,
            staged_executable,
            arguments,
            expectation,
            policy,
            control_transport,
            contracts,
            cleanup,
        } = request;
        self.reap_finished();
        if self.inner.shutdown_requested.load(Ordering::Acquire) {
            remove_staged_launch(staged_executable, contracts, cleanup);
            return Err("managed runtime supervisor is shutting down".to_owned());
        }
        let mut workers = match self.inner.workers.lock() {
            Ok(workers) => workers,
            Err(_) => {
                remove_staged_launch(staged_executable, contracts, cleanup);
                return Err("managed runtime supervisor state is unavailable".to_owned());
            }
        };
        if workers.contains_key(&registration_id) {
            drop(workers);
            remove_staged_launch(staged_executable, contracts, cleanup);
            return Err("managed runtime is already active for this registration".to_owned());
        }
        if let Err(error) = self.clear_failure(&registration_id) {
            drop(workers);
            remove_staged_launch(staged_executable, contracts, cleanup);
            return Err(error);
        }
        let (
            vault_route_handler,
            event_credential_handler,
            provider_credential_handler,
            owner_derived_key_handler,
            blob_session_handler,
            blob_custody_release_handler,
            module_query_handler,
            module_request_handler,
            client_realtime_handler,
        ) = match self.configured_request_handlers() {
            Ok(handlers) => handlers,
            Err(error) => {
                drop(workers);
                remove_staged_launch(staged_executable, contracts, cleanup);
                return Err(error);
            }
        };
        let worker = new_active_worker(ActiveWorkerInput {
            inner: Arc::clone(&self.inner),
            registration_id: registration_id.clone(),
            staged_executable,
            arguments,
            expectation,
            policy,
            control_transport,
            contracts,
            cleanup,
            vault_route_handler,
            event_credential_handler,
            provider_credential_handler,
            owner_derived_key_handler,
            blob_session_handler,
            blob_custody_release_handler,
            module_query_handler,
            module_request_handler,
            client_realtime_handler,
        })?;
        workers.insert(registration_id, worker);
        Ok(())
    }

    fn configured_request_handlers(&self) -> Result<ConfiguredRequestHandlers, String> {
        Ok((
            self.vault_route_handler()?,
            self.event_credential_handler()?,
            self.provider_credential_handler()?,
            self.owner_derived_key_handler()?,
            self.blob_session_handler()?,
            self.blob_custody_release_handler()?,
            self.module_query_handler()?,
            self.module_request_handler()?,
            self.client_realtime_handler()?,
        ))
    }

    fn vault_route_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeVaultRouteHandler>>, String> {
        self.inner
            .vault_route_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn event_credential_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeEventCredentialHandler>>, String> {
        self.inner
            .event_credential_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn provider_credential_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeProviderCredentialHandler>>, String> {
        self.inner
            .provider_credential_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn owner_derived_key_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeOwnerDerivedKeyHandler>>, String> {
        self.inner
            .owner_derived_key_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn blob_session_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeBlobSessionHandler>>, String> {
        self.inner
            .blob_session_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn blob_custody_release_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeBlobCustodyReleaseHandler>>, String> {
        self.inner
            .blob_custody_release_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn module_query_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeModuleQueryHandler>>, String> {
        self.inner
            .module_query_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn module_request_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeModuleRequestHandler>>, String> {
        self.inner
            .module_request_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn client_realtime_handler(
        &self,
    ) -> Result<Option<Arc<dyn ManagedRuntimeClientRealtimeHandler>>, String> {
        self.inner
            .client_realtime_handler
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
            .map(|handler| handler.clone())
    }

    fn configure_before_launch<T>(
        &self,
        slot: &Mutex<Option<Arc<T>>>,
        handler: Arc<T>,
        label: &str,
    ) -> Result<(), String>
    where
        T: ?Sized + Send + Sync,
    {
        if !self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .is_empty()
        {
            return Err(format!("{label} must be configured before launch"));
        }
        let mut current = slot
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?;
        if current.is_some() {
            return Err(format!("{label} is already configured"));
        }
        *current = Some(handler);
        Ok(())
    }

    fn clear_failure(&self, registration_id: &str) -> Result<(), String> {
        self.inner
            .failures
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .remove(registration_id);
        Ok(())
    }

    pub fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        self.relay_port().relay(registration_id, payload)
    }

    pub(crate) fn relay_with_timeout(
        &self,
        registration_id: &str,
        payload: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>, String> {
        self.relay_port()
            .relay_with_timeout(registration_id, payload, timeout)
    }

    pub fn is_active(&self, registration_id: &str) -> Result<bool, String> {
        self.reap_finished();
        self.inner
            .workers
            .lock()
            .map(|workers| workers.contains_key(registration_id))
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
    }

    pub fn wait_until_ready(&self, registration_id: &str) -> Result<(), String> {
        self.wait_until_ready_with_timeout(registration_id, MANAGED_RUNTIME_READY_TIMEOUT)
    }

    pub(crate) fn wait_until_ready_with_timeout(
        &self,
        registration_id: &str,
        timeout: std::time::Duration,
    ) -> Result<(), String> {
        let receiver = self.take_ready_receiver(registration_id)?;
        match receiver.recv_timeout(timeout) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(error),
            Err(RecvTimeoutError::Timeout) => {
                let _ = self.stop(registration_id);
                Err("managed runtime did not become ready before its deadline".to_owned())
            }
            Err(RecvTimeoutError::Disconnected) => Err(self
                .last_failure(registration_id)?
                .unwrap_or_else(|| "managed runtime stopped before readiness".to_owned())),
        }
    }

    pub fn last_failure(&self, registration_id: &str) -> Result<Option<String>, String> {
        self.inner
            .failures
            .lock()
            .map(|failures| failures.get(registration_id).cloned())
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())
    }

    pub(crate) fn record_failure(
        &self,
        registration_id: &str,
        error: String,
    ) -> Result<(), String> {
        self.inner
            .failures
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .insert(registration_id.to_owned(), error);
        Ok(())
    }

    fn take_ready_receiver(
        &self,
        registration_id: &str,
    ) -> Result<Receiver<Result<(), String>>, String> {
        let workers = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?;
        let worker = workers
            .get(registration_id)
            .ok_or_else(|| "managed runtime is unavailable".to_owned())?;
        worker
            .ready
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .take()
            .ok_or_else(|| "managed runtime readiness was already consumed".to_owned())
    }

    pub fn stop(&self, registration_id: &str) -> Result<(), String> {
        self.stop_if_active(registration_id)?
            .then_some(())
            .ok_or_else(|| "managed runtime is unavailable".to_owned())
    }

    /// Prevents an active worker from restarting its child while an external
    /// authority fence is completed. The worker remains registered until
    /// `stop_if_active` joins or reaps it.
    pub(crate) fn request_stop_if_active(&self, registration_id: &str) -> Result<bool, String> {
        let workers = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?;
        let Some(worker) = workers.get(registration_id) else {
            return Ok(false);
        };
        worker.stop_requested.store(true, Ordering::Release);
        let _ = worker.relay.wake();
        Ok(true)
    }

    /// Stops the exact managed registration when present and treats an already
    /// inactive registration as a successful no-op.
    pub fn stop_if_active(&self, registration_id: &str) -> Result<bool, String> {
        self.reap_finished();
        let worker = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .remove(registration_id);
        let Some(worker) = worker else {
            return Ok(false);
        };
        worker.stop_requested.store(true, Ordering::Release);
        let _ = worker.relay.wake();
        worker
            .join
            .join()
            .map_err(|_| "managed runtime supervisor worker panicked".to_owned())?;
        Ok(true)
    }

    pub fn reap_finished(&self) {
        let finished = match self.inner.workers.lock() {
            Ok(mut workers) => {
                let ids = workers
                    .iter()
                    .filter(|(_, worker)| worker.join.is_finished())
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                ids.into_iter()
                    .filter_map(|id| workers.remove(&id))
                    .collect::<Vec<_>>()
            }
            Err(_) => return,
        };
        for worker in finished {
            let _ = worker.join.join();
        }
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.inner.shutdown_requested.store(true, Ordering::Release);
        let workers = self
            .inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .drain()
            .map(|(_, worker)| {
                let _ = worker.relay.wake();
                worker.join
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker
                .join()
                .map_err(|_| "managed runtime supervisor worker panicked".to_owned())?;
        }
        Ok(())
    }
}

impl ManagedRuntimeRelayPort {
    pub(crate) fn is_ready(&self, registration_id: &str) -> Result<bool, String> {
        let inner = self
            .inner
            .upgrade()
            .ok_or_else(|| "managed runtime supervisor is unavailable".to_owned())?;
        inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .get(registration_id)
            .map(|worker| worker.ready_state.load(Ordering::Acquire))
            .ok_or_else(|| "managed runtime is unavailable".to_owned())
    }

    pub fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        self.relay_with_timeout(registration_id, payload, MANAGED_RUNTIME_RELAY_TIMEOUT)
    }

    pub(crate) fn relay_with_timeout(
        &self,
        registration_id: &str,
        payload: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>, String> {
        let inner = self
            .inner
            .upgrade()
            .ok_or_else(|| "managed runtime supervisor is unavailable".to_owned())?;
        let sender = inner
            .workers
            .lock()
            .map_err(|_| "managed runtime supervisor state is unavailable".to_owned())?
            .get(registration_id)
            .map(|worker| worker.relay.clone())
            .ok_or_else(|| "managed runtime is unavailable".to_owned())?;
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        sender.try_send(ManagedRuntimeRelayRequest::new(payload, response_sender))?;
        response_receiver
            .recv_timeout(timeout)
            .map_err(|_| "managed runtime relay timed out".to_owned())?
    }
}

impl ManagedRuntimeRelay for ManagedRuntimeRelayPort {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        Self::relay(self, registration_id, payload)
    }

    fn relay_with_timeout(
        &self,
        registration_id: &str,
        payload: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>, String> {
        Self::relay_with_timeout(self, registration_id, payload, timeout)
    }
}
