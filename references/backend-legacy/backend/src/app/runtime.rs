use super::router::ApplicationComponents;
use makosh_communications_postgres::runtime_leases::ProviderRuntimeLeaseStore;
use makosh_desktop_runtime::{
    NoopRuntimeLifecycleObserver, RuntimeShutdownConfig, RuntimeSupervisor,
};
use makosh_worker_runtime::WorkerRuntime;
use serde_json::Value;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

#[path = "runtime_task_catalog.rs"]
mod runtime_task_catalog;

pub(crate) struct ApplicationRuntime {
    termination: CancellationToken,
    worker_runtime: Option<WorkerRuntime>,
    startup_error: Option<String>,
    lease_pool: Option<PgPool>,
}

impl ApplicationRuntime {
    pub(crate) fn termination_signal(&self) -> CancellationToken {
        self.termination.clone()
    }

    pub(crate) async fn shutdown(mut self) -> Option<String> {
        let mut runtime_error = self.startup_error.take();
        if let Some(runtime) = self.worker_runtime.take()
            && let Err(error) = runtime.shutdown().await
        {
            tracing::warn!(error = %error, "provider worker runtime stopped during shutdown");
            runtime_error.get_or_insert_with(|| error.to_string());
        }
        if let Some(pool) = self.lease_pool.take()
            && let Err(error) = revoke_runtime_leases(&pool).await
        {
            tracing::warn!(error = %error, "provider runtime lease revoke failed during shutdown");
        }
        runtime_error
    }
}

pub(crate) fn start_application_runtime(components: &ApplicationComponents) -> ApplicationRuntime {
    let mut runtime_tasks = runtime_task_catalog::collect(components);
    let lifecycle_runtime = crate::app::runtime_lifecycle::build_lifecycle_projection_runtime(
        &runtime_tasks,
        components.state.database.pool().cloned(),
    );
    if let Some(lifecycle_runtime) = lifecycle_runtime.as_ref() {
        runtime_tasks.push(lifecycle_runtime.task.clone());
    }
    let lifecycle_observer = lifecycle_runtime
        .map(|lifecycle_runtime| lifecycle_runtime.observer)
        .unwrap_or_else(|| std::sync::Arc::new(NoopRuntimeLifecycleObserver));
    let supervisor = RuntimeSupervisor::new(
        runtime_tasks,
        RuntimeShutdownConfig::default(),
        lifecycle_observer,
    );
    let lease_pool = components.state.database.pool().cloned();
    let (worker_runtime, termination, startup_error) = match supervisor {
        Ok(supervisor) => {
            let runtime = WorkerRuntime::start_with(supervisor, async move {
                match lease_pool {
                    Some(pool) => acquire_in_process_leases(&pool).await,
                    None => Ok(()),
                }
            });
            let termination = runtime.termination_signal();
            (Some(runtime), termination, None)
        }
        Err(error) => {
            tracing::error!(error = %error, "failed to construct provider runtime supervisor");
            let termination = CancellationToken::new();
            termination.cancel();
            (None, termination, Some(error.to_string()))
        }
    };

    ApplicationRuntime {
        termination,
        worker_runtime,
        startup_error,
        lease_pool: components.state.database.pool().cloned(),
    }
}

async fn acquire_in_process_leases(pool: &sqlx::PgPool) -> Result<(), String> {
    let accounts = sqlx::query_as::<_, (String, String, Value)>(
        "SELECT account_id, provider_kind, config FROM communication_provider_accounts",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;
    let leases = ProviderRuntimeLeaseStore::new(pool.clone());
    for (account_id, provider, config) in accounts {
        let topology = config
            .get("runtime_topology")
            .and_then(Value::as_str)
            .filter(|value| {
                matches!(
                    *value,
                    "in_process" | "shared_connector" | "per_account_connector"
                )
            })
            .unwrap_or("in_process");
        leases
            .acquire(
                provider.as_str(),
                account_id.as_str(),
                topology,
                "makosh-backend",
                chrono::Duration::hours(1),
            )
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

async fn revoke_runtime_leases(pool: &PgPool) -> Result<(), String> {
    let accounts = sqlx::query_as::<_, (String, String)>(
        "SELECT account_id, provider_kind FROM communication_provider_accounts",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;
    let leases = ProviderRuntimeLeaseStore::new(pool.clone());
    for (account_id, provider) in accounts {
        leases
            .revoke(provider.as_str(), account_id.as_str(), "makosh-backend")
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
