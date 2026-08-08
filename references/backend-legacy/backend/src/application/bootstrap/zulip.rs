use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{ApplicationBootstrapContext, host_vault_is_unlocked, runtime_allows_processing};

static COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static EVENT_INGEST_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ATTACHMENT_DOWNLOAD_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static OBSERVATION_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const COMMAND_EXECUTOR_RUNTIME: &str = "zulip_command_executor";
const EVENT_INGEST_RUNTIME: &str = "zulip_event_ingest";
const ATTACHMENT_DOWNLOAD_RUNTIME: &str = "zulip_attachment_download";
const OBSERVATION_RECONCILIATION_RUNTIME: &str = "zulip_provider_observation_reconciliation";

pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        command_executor_task(context.clone()),
        event_ingest_task(context.clone()),
        attachment_download_task(context.clone()),
        observation_reconciliation_task(context),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn command_executor_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_once(
        &COMMAND_EXECUTOR_DATABASES,
        &database_url,
        "zulip command executor",
    ) {
        return None;
    }
    let runtime_pool = pool.clone();
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let runtime_pool = runtime_pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &runtime_pool,
                    "zulip",
                    COMMAND_EXECUTOR_RUNTIME,
                    json!({ "label": "Zulip command executor", "scope": "runtime" }),
                )
                .await
                {
                    continue;
                }
                if !host_vault_is_unlocked(&vault) {
                    continue;
                }
                let worker = crate::application::zulip_command_executor::ZulipCommandWorker::new(
                    pool.clone(),
                    vault.clone(),
                );
                match worker.execute_due(Utc::now(), 10).await {
                    Ok(report)
                        if report.claimed > 0
                            || report.accounts_failed > 0
                            || report.dead_lettered > 0 =>
                    {
                        tracing::info!(
                            accounts_scanned = report.accounts_scanned,
                            accounts_failed = report.accounts_failed,
                            claimed = report.claimed,
                            completed = report.completed,
                            retrying = report.retrying,
                            dead_lettered = report.dead_lettered,
                            "zulip command executor tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "zulip command executor tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(
        RuntimeTaskSpec::new(
            COMMAND_EXECUTOR_RUNTIME,
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        )
        .with_lifecycle_target("zulip", COMMAND_EXECUTOR_RUNTIME),
    )
}

fn event_ingest_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_once(&EVENT_INGEST_DATABASES, &database_url, "zulip event ingest") {
        return None;
    }
    let runtime_pool = pool.clone();
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let runtime_pool = runtime_pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &runtime_pool,
                    "zulip",
                    EVENT_INGEST_RUNTIME,
                    json!({ "label": "Zulip event ingest", "scope": "runtime" }),
                )
                .await
                {
                    continue;
                }
                if !host_vault_is_unlocked(&vault) {
                    continue;
                }
                let worker = crate::application::zulip_event_ingest::ZulipEventIngestWorker::new(
                    pool.clone(),
                    vault.clone(),
                );
                match worker.poll_due(Utc::now()).await {
                    Ok(report)
                        if report.events_received > 0
                            || report.accounts_failed > 0
                            || report.queues_registered > 0 =>
                    {
                        tracing::info!(
                            accounts_scanned = report.accounts_scanned,
                            accounts_failed = report.accounts_failed,
                            queues_registered = report.queues_registered,
                            events_received = report.events_received,
                            raw_records_recorded = report.raw_records_recorded,
                            accepted_signals = report.accepted_signals,
                            checkpoints_saved = report.checkpoints_saved,
                            "zulip event ingest tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "zulip event ingest tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(
        RuntimeTaskSpec::new(
            EVENT_INGEST_RUNTIME,
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        )
        .with_lifecycle_target("zulip", EVENT_INGEST_RUNTIME),
    )
}

fn attachment_download_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_once(
        &ATTACHMENT_DOWNLOAD_DATABASES,
        &database_url,
        "zulip attachment download",
    ) {
        return None;
    }
    let runtime_pool = pool.clone();
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let runtime_pool = runtime_pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &runtime_pool,
                    "zulip",
                    ATTACHMENT_DOWNLOAD_RUNTIME,
                    json!({ "label": "Zulip attachment download", "scope": "runtime" }),
                )
                .await
                {
                    continue;
                }
                if !host_vault_is_unlocked(&vault) {
                    continue;
                }
                let worker =
                    crate::application::zulip_attachment_download::ZulipAttachmentDownloadWorker::new(
                        pool.clone(),
                        vault.clone(),
                    );
                match worker.download_due(Utc::now(), 10).await {
                    Ok(report)
                        if report.candidates_seen > 0
                            || report.attachments_materialized > 0
                            || report.attachments_failed > 0
                            || report.accounts_failed > 0 =>
                    {
                        tracing::info!(
                            accounts_scanned = report.accounts_scanned,
                            accounts_failed = report.accounts_failed,
                            candidates_seen = report.candidates_seen,
                            attachments_downloaded = report.attachments_downloaded,
                            attachments_materialized = report.attachments_materialized,
                            attachments_skipped = report.attachments_skipped,
                            attachments_failed = report.attachments_failed,
                            "zulip attachment download tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "zulip attachment download tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(
        RuntimeTaskSpec::new(
            ATTACHMENT_DOWNLOAD_RUNTIME,
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        )
        .with_lifecycle_target("zulip", ATTACHMENT_DOWNLOAD_RUNTIME),
    )
}

fn observation_reconciliation_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_once(
        &OBSERVATION_RECONCILIATION_DATABASES,
        &database_url,
        "zulip provider observation reconciliation consumer",
    ) {
        return None;
    }
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::application::zulip_provider_observation_reconciliation::ZULIP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "zulip",
                    OBSERVATION_RECONCILIATION_RUNTIME,
                    json!({
                        "label": "Zulip provider observation reconciliation consumer",
                        "scope": "consumer",
                    }),
                )
                .await
                {
                    continue;
                }
                let handler_pool = pool.clone();
                let handler_event_bus = event_bus.clone();
                if let Err(error) = runner
                    .process_next_batch(|event| {
                        let handler_pool = handler_pool.clone();
                        let handler_event_bus = handler_event_bus.clone();
                        async move {
                            crate::application::zulip_provider_observation_reconciliation::reconcile_zulip_provider_observation_event(
                                handler_pool,
                                handler_event_bus,
                                event,
                            )
                            .await
                            .map_err(makosh_events_postgres::errors::EventStoreError::ConsumerHandlerFailed)
                        }
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "zulip provider observation reconciliation consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(
        RuntimeTaskSpec::new(
            OBSERVATION_RECONCILIATION_RUNTIME,
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        )
        .with_lifecycle_target("zulip", OBSERVATION_RECONCILIATION_RUNTIME),
    )
}

fn register_once(
    registry: &LazyLock<Mutex<HashSet<String>>>,
    database_url: &str,
    task_label: &str,
) -> bool {
    match registry.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, task_label, "zulip runtime registry is unavailable");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{
        ATTACHMENT_DOWNLOAD_DATABASES, COMMAND_EXECUTOR_DATABASES, EVENT_INGEST_DATABASES,
        OBSERVATION_RECONCILIATION_DATABASES, register_once,
    };

    #[test]
    fn each_zulip_runtime_registration_is_once_per_database_url() {
        for (registry, label) in [
            (&COMMAND_EXECUTOR_DATABASES, "command-executor"),
            (&EVENT_INGEST_DATABASES, "event-ingest"),
            (&ATTACHMENT_DOWNLOAD_DATABASES, "attachment-download"),
            (
                &OBSERVATION_RECONCILIATION_DATABASES,
                "observation-reconciliation",
            ),
        ] {
            let database_url = format!(
                "postgres://zulip-{label}-test/{}",
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            );

            assert!(register_once(registry, &database_url, label));
            assert!(!register_once(registry, &database_url, label));
        }
    }
}
