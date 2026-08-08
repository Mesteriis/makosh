//! Supervised worker execution for the desktop composition root.
//!
//! Worker factories are supplied by composition. This crate owns the spawned
//! supervisor task, cancellation and terminal-failure signal; it never sees
//! database pools, providers, vaults or application state.

use std::future::Future;

use makosh_desktop_runtime::{RuntimeShutdownReport, RuntimeSupervisor, RuntimeSupervisorError};
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct WorkerRuntime {
    shutdown: CancellationToken,
    termination: CancellationToken,
    join: JoinHandle<Result<RuntimeShutdownReport, WorkerRuntimeError>>,
}

impl WorkerRuntime {
    pub fn start_with<Prelude>(supervisor: RuntimeSupervisor, prelude: Prelude) -> Self
    where
        Prelude: Future<Output = Result<(), String>> + Send + 'static,
    {
        let shutdown = CancellationToken::new();
        let termination = CancellationToken::new();
        let task_shutdown = shutdown.clone();
        let task_termination = termination.clone();
        let join = tokio::spawn(async move {
            if let Err(error) = prelude.await {
                task_termination.cancel();
                return Err(WorkerRuntimeError::Prelude(error));
            }

            match supervisor.run_until(task_shutdown.cancelled()).await {
                Ok(report) => Ok(report),
                Err(error) => {
                    task_termination.cancel();
                    Err(WorkerRuntimeError::Supervisor(error))
                }
            }
        });

        Self {
            shutdown,
            termination,
            join,
        }
    }

    pub fn termination_signal(&self) -> CancellationToken {
        self.termination.clone()
    }

    pub async fn shutdown(self) -> Result<RuntimeShutdownReport, WorkerRuntimeError> {
        self.shutdown.cancel();
        self.join.await.map_err(WorkerRuntimeError::Join)?
    }
}

#[derive(Debug, Error)]
pub enum WorkerRuntimeError {
    #[error("worker runtime prelude failed: {0}")]
    Prelude(String),
    #[error("worker supervisor failed: {0}")]
    Supervisor(#[from] RuntimeSupervisorError),
    #[error("worker runtime join failed: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use makosh_desktop_runtime::{
        NoopRuntimeLifecycleObserver, RuntimeShutdownConfig, RuntimeTaskClass, RuntimeTaskError,
        RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
    };

    use super::*;

    #[tokio::test]
    async fn prelude_failure_terminates_without_starting_workers() {
        let task: RuntimeTaskFactory = Arc::new(|_| {
            Box::pin(async {
                Err(RuntimeTaskError::Coded {
                    code: "unexpected".into(),
                })
            }) as RuntimeTaskFuture
        });
        let supervisor = RuntimeSupervisor::new(
            vec![RuntimeTaskSpec::new(
                "worker",
                RuntimeTaskClass::Background,
                makosh_desktop_runtime::RuntimeExitPolicy::MarkDegraded,
                task,
            )],
            RuntimeShutdownConfig {
                quiesce_timeout: Duration::from_millis(5),
                checkpoint_timeout: Duration::from_millis(5),
                provider_stop_timeout: Duration::from_millis(5),
            },
            Arc::new(NoopRuntimeLifecycleObserver),
        )
        .expect("supervisor");
        let runtime = WorkerRuntime::start_with(supervisor, async { Err("lease failed".into()) });

        runtime.termination_signal().cancelled().await;
        let error = runtime.shutdown().await.expect_err("prelude failure");
        assert!(matches!(error, WorkerRuntimeError::Prelude(message) if message == "lease failed"));
    }

    #[tokio::test]
    async fn shutdown_cancels_the_supervised_worker_catalog() {
        let task: RuntimeTaskFactory = Arc::new(|token| {
            Box::pin(async move {
                token.cancelled().await;
                Ok(())
            }) as RuntimeTaskFuture
        });
        let supervisor = RuntimeSupervisor::new(
            vec![RuntimeTaskSpec::new(
                "worker",
                RuntimeTaskClass::Background,
                makosh_desktop_runtime::RuntimeExitPolicy::MarkDegraded,
                task,
            )],
            RuntimeShutdownConfig {
                quiesce_timeout: Duration::from_millis(5),
                checkpoint_timeout: Duration::from_millis(5),
                provider_stop_timeout: Duration::from_millis(5),
            },
            Arc::new(NoopRuntimeLifecycleObserver),
        )
        .expect("supervisor");
        let runtime = WorkerRuntime::start_with(supervisor, async { Ok(()) });

        let report = runtime.shutdown().await.expect("graceful shutdown");
        assert!(report.aborted_tasks.is_empty());
    }
}
