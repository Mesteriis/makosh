//! Owns one active managed-child worker and its staged launch cleanup.

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobCustodyReleaseHandler, ManagedRuntimeBlobSessionHandler,
    ManagedRuntimeClientRealtimeHandler, ManagedRuntimeEventCredentialHandler,
    ManagedRuntimeExpectation, ManagedRuntimeModuleQueryHandler,
    ManagedRuntimeModuleRequestHandler, ManagedRuntimeOwnerDerivedKeyHandler,
    ManagedRuntimeProviderCredentialHandler, ManagedRuntimeRelayRequest,
    ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;
use crate::runtime::managed::supervisor as managed_child_supervisor;
use makosh_runtime_protocol::managed_control::ManagedControlTransportMajorV1;

use super::Inner;

pub(super) struct ActiveWorker {
    pub(super) join: JoinHandle<()>,
    pub(super) relay: SyncSender<ManagedRuntimeRelayRequest>,
    pub(super) ready: Mutex<Option<Receiver<Result<(), String>>>>,
    pub(super) ready_state: Arc<AtomicBool>,
    pub(super) stop_requested: Arc<AtomicBool>,
}

pub(super) struct ActiveWorkerInput {
    pub(super) inner: Arc<Inner>,
    pub(super) registration_id: String,
    pub(super) staged_executable: StagedNativeArtifact,
    pub(super) arguments: Vec<String>,
    pub(super) expectation: ManagedRuntimeExpectation,
    pub(super) policy: ManagedChildExecutionPolicy,
    pub(super) control_transport: ManagedControlTransportMajorV1,
    pub(super) contracts: Option<StagedRuntimeContracts>,
    pub(super) cleanup: Option<Box<dyn FnOnce() + Send>>,
    pub(super) vault_route_handler: Option<Arc<dyn ManagedRuntimeVaultRouteHandler>>,
    pub(super) event_credential_handler: Option<Arc<dyn ManagedRuntimeEventCredentialHandler>>,
    pub(super) provider_credential_handler:
        Option<Arc<dyn ManagedRuntimeProviderCredentialHandler>>,
    pub(super) owner_derived_key_handler: Option<Arc<dyn ManagedRuntimeOwnerDerivedKeyHandler>>,
    pub(super) blob_session_handler: Option<Arc<dyn ManagedRuntimeBlobSessionHandler>>,
    pub(super) blob_custody_release_handler:
        Option<Arc<dyn ManagedRuntimeBlobCustodyReleaseHandler>>,
    pub(super) module_query_handler: Option<Arc<dyn ManagedRuntimeModuleQueryHandler>>,
    pub(super) module_request_handler: Option<Arc<dyn ManagedRuntimeModuleRequestHandler>>,
    pub(super) client_realtime_handler: Option<Arc<dyn ManagedRuntimeClientRealtimeHandler>>,
}

pub(super) fn new_active_worker(input: ActiveWorkerInput) -> ActiveWorker {
    let ActiveWorkerInput {
        inner,
        registration_id,
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
    } = input;
    let shutdown_requested = Arc::clone(&inner.shutdown_requested);
    let stop_requested = Arc::new(AtomicBool::new(false));
    let worker_stop_requested = Arc::clone(&stop_requested);
    let ready_state = Arc::new(AtomicBool::new(false));
    let worker_ready_state = Arc::clone(&ready_state);
    let (relay, relay_requests) = mpsc::sync_channel(64);
    let (ready_sender, ready) = mpsc::sync_channel(1);
    let join = std::thread::spawn(move || {
        let worker_span = tracing::info_span!(
            "managed_runtime.worker",
            module.id = expectation.module_id(),
            runtime.generation = expectation.runtime_generation(),
            grant.epoch = expectation.grant_epoch(),
            control.transport = ?control_transport,
        );
        let _worker_guard = worker_span.enter();
        tracing::info!(event = "managed_runtime.worker.started");
        record_worker_result(
            &inner,
            &registration_id,
            managed_child_supervisor::run_until_shutdown(
                managed_child_supervisor::ManagedChildRunInput {
                    staged_executable: &staged_executable,
                    arguments: &arguments,
                    expectation: &expectation,
                    policy: &policy,
                    control_transport,
                    shutdown_requested: &shutdown_requested,
                    stop_requested: &worker_stop_requested,
                    relay_requests: &relay_requests,
                    control_handlers:
                        crate::runtime::lifecycle::control::ManagedRuntimeControlHandlers {
                            vault_route: vault_route_handler.as_deref(),
                            event_credential: event_credential_handler.as_deref(),
                            provider_credential: provider_credential_handler.as_deref(),
                            owner_derived_key: owner_derived_key_handler.as_deref(),
                            blob_session: blob_session_handler.as_deref(),
                            blob_custody_release: blob_custody_release_handler.as_deref(),
                            module_query: module_query_handler.as_deref(),
                            module_request: module_request_handler.as_deref(),
                            client_realtime: client_realtime_handler.as_deref(),
                        },
                    ready_sender: &ready_sender,
                    ready_state: &worker_ready_state,
                },
            )
            .map(|_| ()),
        );
        remove_staged_launch(staged_executable, contracts, cleanup);
    });
    ActiveWorker {
        join,
        relay,
        ready: Mutex::new(Some(ready)),
        ready_state,
        stop_requested,
    }
}

pub(super) fn remove_staged_launch(
    staged_executable: StagedNativeArtifact,
    contracts: Option<StagedRuntimeContracts>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
) {
    let _ = staged_executable.remove();
    if let Some(contracts) = contracts {
        let _ = contracts.remove();
    }
    if let Some(cleanup) = cleanup {
        cleanup();
    }
}

fn record_worker_result(inner: &Inner, registration_id: &str, result: Result<(), String>) {
    match result {
        Ok(()) => tracing::info!(event = "managed_runtime.worker.stopped"),
        Err(error) => {
            tracing::error!(
                event = "managed_runtime.worker.failed",
                error.class = "managed_runtime_failure",
                error.message = %error,
            );
            let _ = inner
                .failures
                .lock()
                .map(|mut failures| failures.insert(registration_id.to_owned(), error));
        }
    }
}
