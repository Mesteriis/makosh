use std::sync::Arc;

use crate::app::state::AppState;
use crate::vault::models::VaultMode;
use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use tokio_util::sync::CancellationToken;

use super::service::reconcile_host_vault_manifest;

pub(crate) const HOST_VAULT_MANIFEST_RECONCILIATION_RUNTIME: &str =
    "host_vault_manifest_reconciliation";

pub(crate) fn host_vault_manifest_reconciliation_task(state: &AppState) -> Option<RuntimeTaskSpec> {
    state.config.database_url()?;
    let Ok(status) = state.vault.status() else {
        tracing::warn!("host vault reconciliation skipped: vault status unavailable");
        return None;
    };
    if status.state != VaultMode::Unlocked {
        return None;
    }
    let pool = state.database.pool().cloned()?;
    let vault = state.vault.clone();

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            tokio::select! {
                _ = cancellation.cancelled() => Ok(()),
                result = reconcile_host_vault_manifest(pool, vault) => {
                    report_reconciliation_result(result);
                    Ok(())
                }
            }
        }) as RuntimeTaskFuture
    });

    Some(RuntimeTaskSpec::new(
        HOST_VAULT_MANIFEST_RECONCILIATION_RUNTIME,
        RuntimeTaskClass::Startup,
        RuntimeExitPolicy::ExpectedCompletion,
        task,
    ))
}

pub(crate) async fn reconcile_host_vault_manifest_now(state: &AppState) {
    if state.config.database_url().is_none() {
        return;
    }
    let Ok(status) = state.vault.status() else {
        tracing::warn!("host vault reconciliation skipped: vault status unavailable");
        return;
    };
    if status.state != VaultMode::Unlocked {
        return;
    }
    let Some(pool) = state.database.pool().cloned() else {
        return;
    };
    let vault = state.vault.clone();
    report_reconciliation_result(reconcile_host_vault_manifest(pool, vault).await);
}

fn report_reconciliation_result(
    result: Result<
        super::summary::HostVaultReconciliationSummary,
        super::errors::HostVaultReconciliationError,
    >,
) {
    match result {
        Ok(summary)
            if summary.restored_accounts > 0
                || summary.restored_calendar_accounts > 0
                || summary.restored_ai_providers > 0
                || summary.skipped_duplicate_provider_secrets > 0
                || summary.purged_duplicate_provider_secrets > 0 =>
        {
            tracing::info!(
                restored_accounts = summary.restored_accounts,
                restored_calendar_accounts = summary.restored_calendar_accounts,
                restored_ai_providers = summary.restored_ai_providers,
                skipped_duplicate_provider_secrets = summary.skipped_duplicate_provider_secrets,
                purged_duplicate_provider_secrets = summary.purged_duplicate_provider_secrets,
                "host vault manifest reconciliation completed"
            );
        }
        Ok(_) => {}
        Err(error) => {
            tracing::warn!(error = %error, "host vault manifest reconciliation failed");
        }
    }
}
