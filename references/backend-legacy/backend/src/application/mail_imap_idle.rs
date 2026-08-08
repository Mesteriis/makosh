use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use sqlx::PgPool;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use super::bootstrap::{
    ApplicationBootstrapContext, host_vault_is_unlocked, runtime_allows_processing,
};
use crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::vault::HostVault;
use crate::workflows::mail_background_sync::{
    DEFAULT_GMAIL_API_BASE_URL, idle::MailImapIdleOutcome, models::progress::MailSyncTrigger,
    service::MailBackgroundSyncService, store::MailSyncStore,
};
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore;

static MAIL_IMAP_IDLE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const MAIL_IMAP_IDLE_RUNTIME: &str = "mail_imap_idle";
const MANAGER_TICK_SECONDS: u64 = 30;
const IDLE_TIMEOUT_SECONDS: u64 = 29 * 60;
const POLL_FALLBACK_SECONDS: u64 = 300;

pub(crate) fn runtime_task_spec(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let store = MailSyncStore::new(pool.clone());
            let service = MailBackgroundSyncService::new(
            pool.clone(),
            vault.clone(),
            DEFAULT_MAIL_SYNC_BLOB_ROOT,
            Arc::new(LiveEmailProviderSyncPort::new(
                pool.clone(),
                vault.clone(),
                Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
                DEFAULT_GMAIL_API_BASE_URL,
            )),
            Arc::new(
                crate::domains::communications::provider_resources::MailProviderResourceStore::new(
                    pool.clone(),
                ),
            ),
            Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
            Arc::new(
                makosh_communications_postgres::store::CommunicationIngestionStore::new(
                    pool.clone(),
                ),
            ),
        );
            let mut workers = JoinSet::new();
            let mut worker_cancellations = HashMap::<String, CancellationToken>::new();
            let mut tick = tokio::time::interval(Duration::from_secs(MANAGER_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => {
                        for worker_cancellation in worker_cancellations.values() {
                            worker_cancellation.cancel();
                        }
                        while workers.join_next().await.is_some() {}
                        return Ok(());
                    }
                    _ = tick.tick() => {}
                }
                while let Some(joined) = workers.try_join_next() {
                    if let Ok(account_id) = joined {
                        worker_cancellations.remove(&account_id);
                    }
                }
                let account_ids = match store.imap_idle_account_ids(100).await {
                    Ok(account_ids) => account_ids,
                    Err(error) => {
                        tracing::warn!(error = %error, "mail IMAP IDLE account discovery failed");
                        continue;
                    }
                };
                let eligible = account_ids.into_iter().collect::<HashSet<_>>();
                for (account_id, worker_cancellation) in &worker_cancellations {
                    if !eligible.contains(account_id) {
                        worker_cancellation.cancel();
                    }
                }

                for account_id in eligible {
                    if worker_cancellations.contains_key(&account_id) {
                        continue;
                    }
                    let worker_cancellation = cancellation.child_token();
                    worker_cancellations.insert(account_id.clone(), worker_cancellation.clone());
                    let worker_service = service.clone();
                    let worker_pool = pool.clone();
                    let worker_vault = vault.clone();
                    workers.spawn(async move {
                        run_worker(
                            account_id.clone(),
                            worker_service,
                            worker_pool,
                            worker_vault,
                            worker_cancellation,
                        )
                        .await;
                        account_id
                    });
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_IMAP_IDLE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

async fn run_worker(
    account_id: String,
    service: MailBackgroundSyncService,
    pool: PgPool,
    vault: HostVault,
    cancellation: CancellationToken,
) {
    let mut consecutive_failures = 0_u32;
    let mut unsupported_reported = false;
    loop {
        if cancellation.is_cancelled() {
            return;
        }
        if !runtime_allows_processing(
            &pool,
            "mail",
            MAIL_IMAP_IDLE_RUNTIME,
            json!({ "label": "Mail IMAP IDLE", "scope": "scheduler" }),
        )
        .await
            || !host_vault_is_unlocked(&vault)
        {
            tokio::select! {
                _ = cancellation.cancelled() => return,
                _ = tokio::time::sleep(Duration::from_secs(MANAGER_TICK_SECONDS)) => {}
            }
            continue;
        }

        let outcome = tokio::select! {
            _ = cancellation.cancelled() => return,
            outcome = service.wait_for_imap_change(&account_id, Duration::from_secs(IDLE_TIMEOUT_SECONDS)) => outcome,
        };
        match outcome {
            Ok(MailImapIdleOutcome::Changed) => {
                consecutive_failures = 0;
                unsupported_reported = false;
                if service
                    .run_account(&account_id, MailSyncTrigger::Scheduled)
                    .await
                    .is_err()
                {
                    tracing::warn!(
                        account_id,
                        "IMAP IDLE observed a change but the polling sync trigger failed"
                    );
                }
            }
            Ok(MailImapIdleOutcome::TimedOut) => {
                consecutive_failures = 0;
                unsupported_reported = false;
            }
            Ok(MailImapIdleOutcome::Unsupported) => {
                consecutive_failures = 0;
                if !unsupported_reported {
                    tracing::info!(
                        account_id,
                        "IMAP server does not advertise IDLE; polling fallback remains active"
                    );
                    unsupported_reported = true;
                }
                tokio::select! {
                    _ = cancellation.cancelled() => return,
                    _ = tokio::time::sleep(Duration::from_secs(POLL_FALLBACK_SECONDS)) => {}
                }
            }
            Ok(MailImapIdleOutcome::Disabled) => return,
            Err(_) => {
                consecutive_failures = consecutive_failures.saturating_add(1);
                let delay = reconnect_delay(&account_id, consecutive_failures);
                tracing::warn!(
                    account_id,
                    consecutive_failures,
                    reconnect_delay_ms = delay.as_millis(),
                    "IMAP IDLE connection failed; polling fallback remains active"
                );
                tokio::select! {
                    _ = cancellation.cancelled() => return,
                    _ = tokio::time::sleep(delay) => {}
                }
            }
        }
    }
}

fn reconnect_delay(account_id: &str, consecutive_failures: u32) -> Duration {
    let exponent = consecutive_failures.saturating_sub(1).min(6);
    let base_seconds = 5_u64.saturating_mul(1_u64 << exponent).min(300);
    let account_jitter = account_id
        .bytes()
        .fold(u64::from(consecutive_failures) * 137, |hash, byte| {
            hash.wrapping_mul(31).wrapping_add(u64::from(byte))
        })
        % 1_000;
    Duration::from_secs(base_seconds) + Duration::from_millis(account_jitter)
}

fn register_scheduler(database_url: &str) -> bool {
    match MAIL_IMAP_IDLE_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, "mail IMAP IDLE scheduler registry is unavailable");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn scheduler_registration_and_reconnect_backoff_are_bounded() {
        let database_url = format!(
            "postgres://imap-idle-scheduler-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        assert!(register_scheduler(&database_url));
        assert!(!register_scheduler(&database_url));

        let first = reconnect_delay("mail-account", 1);
        let second = reconnect_delay("mail-account", 2);
        let saturated = reconnect_delay("mail-account", 100);
        assert!(first >= Duration::from_secs(5));
        assert!(second >= Duration::from_secs(10));
        assert!(saturated >= Duration::from_secs(300));
        assert!(saturated < Duration::from_secs(301));
    }
}
