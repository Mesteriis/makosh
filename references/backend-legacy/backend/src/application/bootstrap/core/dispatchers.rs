//! Durable event transport and Signal Hub replay dispatchers.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskError, RuntimeTaskFactory, RuntimeTaskFuture,
    RuntimeTaskSpec,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::super::{ApplicationBootstrapContext, runtime_allows_processing};

static EVENT_OUTBOX_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_REPLAY_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const EVENT_OUTBOX_DISPATCHER_RUNTIME: &str = "event_outbox_dispatcher";
const SIGNAL_REPLAY_DISPATCHER_RUNTIME: &str = "signal_replay_dispatcher";

pub(super) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        event_outbox_dispatcher_task(context.clone()),
        signal_replay_dispatcher_task(context),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn event_outbox_dispatcher_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    let Some(nats_server_url) = context.nats_server_url else {
        tracing::info!(
            "event outbox dispatcher skipped because MAKOSH_NATS_SERVER_URL is not configured"
        );
        return None;
    };
    if !register_event_outbox_dispatcher(&database_url) {
        return None;
    }

    let realtime_bus = context.event_bus.clone();
    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let realtime_bus = realtime_bus.clone();
        let nats_server_url = nats_server_url.clone();
        Box::pin(async move {
            let bus = tokio::select! {
                _ = cancellation.cancelled() => return Ok(()),
                result = makosh_events_nats::jetstream::NatsJetStreamEventBus::connect(&nats_server_url) => {
                    result.map_err(|error| {
                        tracing::warn!(
                            error = %error,
                            nats_server_url,
                            "event outbox dispatcher failed to initialize JetStream bus"
                        );
                        RuntimeTaskError::Coded {
                            code: "event_outbox_nats_connect_failed".to_owned(),
                        }
                    })?
                }
            };
            let dispatcher = crate::platform::events::dispatcher::EventOutboxDispatcher::new(
                makosh_events_postgres::store::EventStore::new(pool.clone()),
                bus,
            )
            .with_realtime_bus(realtime_bus);
            let mut tick = tokio::time::interval(Duration::from_secs(2));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    EVENT_OUTBOX_DISPATCHER_RUNTIME,
                    json!({
                        "label": "Event outbox dispatcher",
                        "scope": "dispatcher",
                        "transport": "nats_jetstream",
                    }),
                )
                .await
                {
                    continue;
                }
                match dispatcher.dispatch_pending_once().await {
                    Ok(report) if report.claimed > 0 || report.recovered > 0 => {
                        tracing::info!(
                            recovered = report.recovered,
                            claimed = report.claimed,
                            published = report.published,
                            retried = report.retried,
                            "event outbox dispatcher tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "event outbox dispatcher tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        EVENT_OUTBOX_DISPATCHER_RUNTIME,
        RuntimeTaskClass::Essential,
        RuntimeExitPolicy::ShutdownRuntime,
        task,
    ))
}

fn signal_replay_dispatcher_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_signal_replay_dispatcher(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let replay_service = crate::application::signal_hub_replay::SignalHubReplayService::new(
                crate::domains::signal_hub::store::SignalHubStore::new(pool.clone()),
                makosh_events_postgres::store::EventStore::new(pool.clone()),
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
                    "system",
                    SIGNAL_REPLAY_DISPATCHER_RUNTIME,
                    json!({
                        "label": "Signal replay dispatcher",
                        "scope": "dispatcher",
                    }),
                )
                .await
                {
                    continue;
                }

                match replay_service.process_next_request().await {
                    Ok(Some(report)) => {
                        tracing::info!(
                            request_id = %report.request_id,
                            replayed_count = report.replayed_count,
                            "signal replay dispatcher tick completed"
                        );
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "signal replay dispatcher tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        SIGNAL_REPLAY_DISPATCHER_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

pub(crate) fn register_event_outbox_dispatcher(database_url: &str) -> bool {
    match EVENT_OUTBOX_DISPATCHER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "event outbox dispatcher registry is unavailable"
            );
            false
        }
    }
}

pub(crate) fn register_signal_replay_dispatcher(database_url: &str) -> bool {
    match SIGNAL_REPLAY_DISPATCHER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "signal replay dispatcher registry is unavailable"
            );
            false
        }
    }
}
