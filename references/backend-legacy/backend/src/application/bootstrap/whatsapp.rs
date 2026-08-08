//! WhatsApp-owned runtime task factories and duplicate-registration guards.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{ApplicationBootstrapContext, runtime_allows_processing};

static WHATSAPP_RUNTIME_EVENT_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
const WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME: &str = "whatsapp_runtime_event_projection";
const WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME: &str =
    "whatsapp_provider_observation_reconciliation";
pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        whatsapp_runtime_event_projection_task(context.clone()),
        whatsapp_provider_observation_reconciliation_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("whatsapp"))
    .collect()
}

fn whatsapp_runtime_event_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_whatsapp_runtime_event_consumer(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::application::whatsapp_runtime_event_projection::WHATSAPP_RUNTIME_EVENT_CONSUMER,
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
                    "whatsapp",
                    WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME,
                    json!({
                        "label": "WhatsApp runtime-event projection consumer",
                        "scope": "consumer",
                    }),
                )
                .await
                {
                    continue;
                }
                let handler_pool = pool.clone();
                let handler_vault = vault.clone();
                if let Err(error) = runner
                    .process_next_batch(|event| {
                        let handler_pool = handler_pool.clone();
                        let handler_vault = handler_vault.clone();
                        async move {
                            crate::application::whatsapp_runtime_event_projection::project_whatsapp_runtime_event(
                                handler_pool.clone(),
                                handler_vault.clone(),
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
                        "whatsapp runtime-event projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn whatsapp_provider_observation_reconciliation_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_whatsapp_provider_observation_reconciliation_consumer(&database_url) {
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
                    crate::application::whatsapp_provider_observation_reconciliation::WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER,
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
                    "whatsapp",
                    WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME,
                    json!({
                        "label": "WhatsApp provider observation reconciliation consumer",
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
                            crate::application::whatsapp_provider_observation_reconciliation::reconcile_whatsapp_provider_observation_event(
                                handler_pool.clone(),
                                handler_event_bus.clone(),
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
                        "whatsapp provider observation reconciliation consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn register_whatsapp_runtime_event_consumer(database_url: &str) -> bool {
    match WHATSAPP_RUNTIME_EVENT_CONSUMER_DATABASES.lock() {
        Ok(mut urls) => urls.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp runtime-event projection consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_whatsapp_provider_observation_reconciliation_consumer(database_url: &str) -> bool {
    match WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_DATABASES.lock() {
        Ok(mut urls) => urls.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp provider observation reconciliation consumer registry is unavailable"
            );
            false
        }
    }
}
