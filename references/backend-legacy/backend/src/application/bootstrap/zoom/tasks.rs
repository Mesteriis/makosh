//! Zoom task factories and duplicate-registration guards.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::super::{
    ApplicationBootstrapContext, host_vault_is_unlocked, runtime_allows_processing,
};
use super::{
    run_zoom_recording_sync_once, run_zoom_retention_cleanup_once, run_zoom_token_maintenance_once,
};

static ZOOM_TOKEN_MAINTENANCE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_RECORDING_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_RETENTION_CLEANUP_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_SIGNAL_DETECTION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_PARTICIPANT_IDENTITY_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
const ZOOM_TOKEN_MAINTENANCE_RUNTIME: &str = "zoom_token_maintenance";
const ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS: u64 = 60;
const ZOOM_RECORDING_SYNC_RUNTIME: &str = "zoom_recording_sync";
const ZOOM_RECORDING_SYNC_TICK_SECONDS: u64 = 300;
const ZOOM_RETENTION_CLEANUP_RUNTIME: &str = "zoom_retention_cleanup";
const ZOOM_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const ZOOM_SIGNAL_DETECTION_RUNTIME: &str = "zoom_signal_detection";
const ZOOM_CALENDAR_MATCHING_RUNTIME: &str = "zoom_calendar_matching";
const ZOOM_PARTICIPANT_IDENTITY_RUNTIME: &str = "zoom_participant_identity";
pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        zoom_token_maintenance_task(context.clone()),
        zoom_recording_sync_task(context.clone()),
        zoom_retention_cleanup_task(context.clone()),
        zoom_calendar_matching_projection_task(context.clone()),
        zoom_signal_detection_projection_task(context.clone()),
        zoom_participant_identity_projection_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("zoom"))
    .collect()
}

fn zoom_token_maintenance_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    if !context.zoom_token_maintenance_scheduler_enabled {
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_token_maintenance_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "zoom",
                    ZOOM_TOKEN_MAINTENANCE_RUNTIME,
                    json!({
                        "label": "Zoom token maintenance",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                if !host_vault_is_unlocked(&vault) {
                    continue;
                }
                match run_zoom_token_maintenance_once(&pool, &vault, &event_bus).await {
                    Ok(result)
                        if result.checked_count > 0
                            || result.refreshed_count > 0
                            || result.failed_count > 0 =>
                    {
                        tracing::info!(
                            checked = result.checked_count,
                            refreshed = result.refreshed_count,
                            skipped = result.skipped_count,
                            failed = result.failed_count,
                            refresh_expiring_within_seconds =
                                result.refresh_expiring_within_seconds,
                            "zoom token maintenance scheduler tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "zoom token maintenance scheduler tick failed"
                        );
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_TOKEN_MAINTENANCE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_recording_sync_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    if !context.zoom_recording_sync_scheduler_enabled {
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_recording_sync_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(ZOOM_RECORDING_SYNC_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "zoom",
                    ZOOM_RECORDING_SYNC_RUNTIME,
                    json!({
                        "label": "Zoom recording sync",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                if !host_vault_is_unlocked(&vault) {
                    continue;
                }
                match run_zoom_recording_sync_once(&pool, &vault, &event_bus).await {
                    Ok(result)
                        if result.accounts_checked > 0
                            || result.accounts_synced > 0
                            || result.failed_count > 0
                            || result.meetings_recorded > 0
                            || result.recordings_recorded > 0
                            || result.media_downloads_recorded > 0
                            || result.transcripts_recorded > 0 =>
                    {
                        tracing::info!(
                            accounts_checked = result.accounts_checked,
                            accounts_synced = result.accounts_synced,
                            accounts_skipped = result.accounts_skipped,
                            failed = result.failed_count,
                            meetings_recorded = result.meetings_recorded,
                            recordings_recorded = result.recordings_recorded,
                            media_downloads_recorded = result.media_downloads_recorded,
                            transcripts_recorded = result.transcripts_recorded,
                            lookback_days = result.lookback_days,
                            "zoom recording sync scheduler tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "zoom recording sync scheduler tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_RECORDING_SYNC_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_retention_cleanup_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    if !context.zoom_retention_cleanup_scheduler_enabled {
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_retention_cleanup_scheduler(&database_url) {
        return None;
    }
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(ZOOM_RETENTION_CLEANUP_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "zoom",
                    ZOOM_RETENTION_CLEANUP_RUNTIME,
                    json!({
                        "label": "Zoom retention cleanup",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                match run_zoom_retention_cleanup_once(&pool, &event_bus).await {
                    Ok(result)
                        if result.accounts_checked > 0
                            || result.accounts_cleaned > 0
                            || result.recordings_removed > 0
                            || result.transcripts_removed > 0 =>
                    {
                        tracing::info!(
                            accounts_checked = result.accounts_checked,
                            accounts_cleaned = result.accounts_cleaned,
                            recordings_removed = result.recordings_removed,
                            transcripts_removed = result.transcripts_removed,
                            limit_per_account = result.limit_per_account,
                            "zoom retention cleanup scheduler tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "zoom retention cleanup scheduler tick failed"
                        );
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_RETENTION_CLEANUP_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_calendar_matching_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_calendar_matching_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::zoom_calendar_matching::ZOOM_CALENDAR_MATCHING_CONSUMER,
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
                    "zoom",
                    ZOOM_CALENDAR_MATCHING_RUNTIME,
                    json!({
                        "label": "Zoom calendar matching consumer",
                        "scope": "consumer",
                    }),
                )
                .await
                {
                    continue;
                }
                let handler_pool = pool.clone();
                if let Err(error) = runner
                    .process_next_batch(|event| {
                        crate::workflows::zoom_calendar_matching::project_zoom_calendar_matching_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "zoom calendar matching projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_CALENDAR_MATCHING_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_signal_detection_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_signal_detection_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::zoom_signal_detection::ZOOM_SIGNAL_DETECTION_CONSUMER,
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
                    "zoom",
                    ZOOM_SIGNAL_DETECTION_RUNTIME,
                    json!({
                        "label": "Zoom signal detection consumer",
                        "scope": "consumer",
                    }),
                )
                .await
                {
                    continue;
                }
                let handler_pool = pool.clone();
                if let Err(error) = runner
                    .process_next_batch(|event| {
                        crate::workflows::zoom_signal_detection::project_zoom_signal_detection_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "zoom signal detection projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_SIGNAL_DETECTION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_participant_identity_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_participant_identity_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::zoom_participant_identity::ZOOM_PARTICIPANT_IDENTITY_CONSUMER,
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
                    "zoom",
                    ZOOM_PARTICIPANT_IDENTITY_RUNTIME,
                    json!({
                        "label": "Zoom participant identity consumer",
                        "scope": "consumer",
                    }),
                )
                .await
                {
                    continue;
                }
                let handler_pool = pool.clone();
                if let Err(error) = runner
                    .process_next_batch(|event| {
                        crate::workflows::zoom_participant_identity::project_zoom_participant_identity_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "zoom participant identity projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_PARTICIPANT_IDENTITY_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

pub(crate) fn register_zoom_signal_detection_consumer(database_url: &str) -> bool {
    match ZOOM_SIGNAL_DETECTION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom signal detection consumer registry is unavailable"
            );
            false
        }
    }
}

pub(crate) fn register_zoom_calendar_matching_consumer(database_url: &str) -> bool {
    match ZOOM_CALENDAR_MATCHING_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom calendar matching consumer registry is unavailable"
            );
            false
        }
    }
}

pub(crate) fn register_zoom_participant_identity_consumer(database_url: &str) -> bool {
    match ZOOM_PARTICIPANT_IDENTITY_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom participant identity consumer registry is unavailable"
            );
            false
        }
    }
}

pub(crate) fn register_zoom_token_maintenance_scheduler(database_url: &str) -> bool {
    match ZOOM_TOKEN_MAINTENANCE_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom token maintenance scheduler registry is unavailable"
            );
            false
        }
    }
}

pub(crate) fn register_zoom_recording_sync_scheduler(database_url: &str) -> bool {
    match ZOOM_RECORDING_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom recording sync scheduler registry is unavailable"
            );
            false
        }
    }
}

pub(crate) fn register_zoom_retention_cleanup_scheduler(database_url: &str) -> bool {
    match ZOOM_RETENTION_CLEANUP_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom retention cleanup scheduler registry is unavailable"
            );
            false
        }
    }
}
