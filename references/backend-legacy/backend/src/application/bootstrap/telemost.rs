//! Yandex Telemost task factories, retention, and matching runtime ownership.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use sqlx::postgres::PgPool;
use tokio_util::sync::CancellationToken;

use super::{ApplicationBootstrapContext, runtime_allows_processing};
use crate::platform::events::bus::InMemoryEventBus;

static YANDEX_TELEMOST_RETENTION_CLEANUP_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
const YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME: &str = "yandex_telemost_retention_cleanup";
const YANDEX_TELEMOST_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
const YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME: &str = "yandex_telemost_calendar_matching";
pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        yandex_telemost_retention_cleanup_task(context.clone()),
        yandex_telemost_calendar_matching_projection_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("yandex_telemost"))
    .collect()
}

fn yandex_telemost_retention_cleanup_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_yandex_telemost_retention_cleanup_scheduler(&database_url) {
        return None;
    }
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(
                YANDEX_TELEMOST_RETENTION_CLEANUP_TICK_SECONDS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "yandex_telemost",
                    YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME,
                    json!({
                        "label": "Yandex Telemost retention cleanup",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                match run_yandex_telemost_retention_cleanup_once(&pool, &event_bus).await {
                    Ok(result)
                        if result.accounts_checked > 0
                            || result.accounts_cleaned > 0
                            || result.audio_files_removed > 0
                            || result.speaker_hint_files_removed > 0 =>
                    {
                        tracing::info!(
                            accounts_checked = result.accounts_checked,
                            accounts_cleaned = result.accounts_cleaned,
                            audio_files_removed = result.audio_files_removed,
                            speaker_hint_files_removed = result.speaker_hint_files_removed,
                            limit_per_account = result.limit_per_account,
                            "yandex telemost retention cleanup scheduler tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "yandex telemost retention cleanup scheduler tick failed"
                        );
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn yandex_telemost_calendar_matching_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_yandex_telemost_calendar_matching_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::yandex_telemost_calendar_matching::YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER,
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
                    "yandex_telemost",
                    YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME,
                    json!({
                        "label": "Yandex Telemost calendar matching consumer",
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
                        crate::workflows::yandex_telemost_calendar_matching::project_yandex_telemost_calendar_matching_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "yandex telemost calendar matching projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn register_yandex_telemost_retention_cleanup_scheduler(database_url: &str) -> bool {
    match YANDEX_TELEMOST_RETENTION_CLEANUP_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "yandex telemost retention cleanup scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_yandex_telemost_calendar_matching_consumer(database_url: &str) -> bool {
    match YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "yandex telemost calendar matching consumer registry is unavailable"
            );
            false
        }
    }
}

struct YandexTelemostRetentionCleanupSchedulerResult {
    accounts_checked: usize,
    accounts_cleaned: usize,
    audio_files_removed: usize,
    speaker_hint_files_removed: usize,
    limit_per_account: i64,
}

async fn run_yandex_telemost_retention_cleanup_once(
    pool: &PgPool,
    event_bus: &InMemoryEventBus,
) -> Result<YandexTelemostRetentionCleanupSchedulerResult, String> {
    let service =
        crate::application::provider_runtime_factories::yandex_telemost_provider_runtime_service(
            pool.clone(),
            event_bus.clone(),
        );
    let accounts = service
        .list_accounts(false)
        .await
        .map_err(|error| error.to_string())?
        .items;
    let mut result = YandexTelemostRetentionCleanupSchedulerResult {
        accounts_checked: 0,
        accounts_cleaned: 0,
        audio_files_removed: 0,
        speaker_hint_files_removed: 0,
        limit_per_account: YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
    };

    for account in accounts {
        if account.provider_kind != "yandex_telemost_user" {
            continue;
        }
        result.accounts_checked += 1;
        let response = service
            .cleanup_retention(
                &account.account_id,
                &crate::integrations::yandex_telemost::client::models::YandexTelemostRetentionCleanupRequest {
                    remove_audio: true,
                    remove_speaker_hints: true,
                    limit: YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        if response.bundles_cleaned > 0 {
            result.accounts_cleaned += 1;
        }
        result.audio_files_removed += response.audio_files_removed;
        result.speaker_hint_files_removed += response.speaker_hint_files_removed;
    }

    Ok(result)
}
