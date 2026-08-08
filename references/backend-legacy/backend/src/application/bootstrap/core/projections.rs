//! Event consumers that materialize core evidence, review, and Signal Hub state.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::super::{ApplicationBootstrapContext, runtime_allows_processing};

static COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSONA_DERIVED_EVIDENCE_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PROJECT_LINK_REVIEW_EFFECTS_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER_DATABASES: LazyLock<
    Mutex<HashSet<String>>,
> = LazyLock::new(|| Mutex::new(HashSet::new()));
static REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER_DATABASES: LazyLock<
    Mutex<HashSet<String>>,
> = LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_HUB_RAW_SIGNAL_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME: &str =
    "communication_provider_observation_projection";
const PERSONA_DERIVED_EVIDENCE_RUNTIME: &str = "persona_derived_evidence";
const PERSONA_IDENTITY_REVIEW_INBOX_RUNTIME: &str = "persona_identity_review_inbox";
const PROJECT_LINK_REVIEW_EFFECTS_RUNTIME: &str = "project_link_review_effects";
const REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME: &str =
    "realtime_conversation_transcript_execution";
const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME: &str =
    "realtime_conversation_transcript_projection";
const SIGNAL_HUB_RAW_SIGNAL_RUNTIME: &str = "signal_hub_raw_signal_dispatcher";
const REALTIME_SIGNAL_PROJECTION_TICK_MILLIS: u64 = 250;

pub(super) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        communication_provider_observation_projection_task(context.clone()),
        signal_hub_raw_signal_dispatcher_task(context.clone()),
        realtime_conversation_transcript_execution_task(context.clone()),
        realtime_conversation_transcript_projection_task(context.clone()),
        persona_derived_evidence_projection_task(context.clone()),
        persona_identity_review_inbox_projection_task(context.clone()),
        project_link_review_effects_projection_task(context),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn communication_provider_observation_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_communication_provider_observation_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::domains::communications::messages::provider_observation_projection::COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_millis(
                REALTIME_SIGNAL_PROJECTION_TICK_MILLIS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME,
                    json!({
                        "label": "Communications accepted-signal consumer",
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
                        crate::domains::communications::messages::provider_observation_projection::project_provider_observation_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "communication provider observation projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME,
        RuntimeTaskClass::Essential,
        RuntimeExitPolicy::ShutdownRuntime,
        task,
    ))
}

fn persona_derived_evidence_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_persona_derived_evidence_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::persona_derived_evidence::PERSONA_DERIVED_EVIDENCE_CONSUMER,
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
                    "system",
                    PERSONA_DERIVED_EVIDENCE_RUNTIME,
                    json!({
                        "label": "Persona derived evidence consumer",
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
                    crate::workflows::persona_derived_evidence::project_persona_derived_evidence_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "persona derived evidence projection consumer tick failed"
                );
            }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        PERSONA_DERIVED_EVIDENCE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn persona_identity_review_inbox_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_persona_identity_review_inbox_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::review_inbox::PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER,
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
                    "system",
                    PERSONA_IDENTITY_REVIEW_INBOX_RUNTIME,
                    json!({
                        "label": "Persona identity review inbox consumer",
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
                        crate::workflows::review_inbox::project_persona_identity_review_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "persona identity review inbox projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        PERSONA_IDENTITY_REVIEW_INBOX_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn project_link_review_effects_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_project_link_review_effects_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
            pool.clone(),
            makosh_events_postgres::consumers::EventConsumerConfig::new(
                crate::workflows::project_link_review_effects::PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
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
                    "system",
                    PROJECT_LINK_REVIEW_EFFECTS_RUNTIME,
                    json!({
                        "label": "Project link review effects consumer",
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
                    crate::workflows::project_link_review_effects::project_link_review_effect_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "project link review effects projection consumer tick failed"
                );
            }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        PROJECT_LINK_REVIEW_EFFECTS_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn realtime_conversation_transcript_execution_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    if !crate::workflows::realtime_conversation_transcript_execution::realtime_conversation_transcriber_is_configured() {
        tracing::info!(
            "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER is not configured; realtime conversation transcript execution consumer is disabled"
        );
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_realtime_conversation_transcript_execution_consumer(&database_url) {
        return None;
    }
    let event_bus = context.event_bus.clone();

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::realtime_conversation_transcript_execution::REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER,
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
                    "system",
                    REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME,
                    json!({
                        "label": "Realtime conversation transcript execution consumer",
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
                        crate::workflows::realtime_conversation_transcript_execution::execute_realtime_conversation_transcript_request_event(
                            handler_pool.clone(),
                            handler_event_bus.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "realtime conversation transcript execution consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn realtime_conversation_transcript_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_realtime_conversation_transcript_projection_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::realtime_conversation_transcript_projection::REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER,
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
                    "system",
                    REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME,
                    json!({
                        "label": "Realtime conversation transcript projection consumer",
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
                        crate::workflows::realtime_conversation_transcript_projection::project_realtime_conversation_transcript_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "realtime conversation transcript projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn signal_hub_raw_signal_dispatcher_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_signal_hub_raw_signal_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = makosh_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                makosh_events_postgres::consumers::EventConsumerConfig::new(
                    crate::domains::signal_hub::service::SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_millis(
                REALTIME_SIGNAL_PROJECTION_TICK_MILLIS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    SIGNAL_HUB_RAW_SIGNAL_RUNTIME,
                    json!({
                        "label": "Signal Hub raw signal dispatcher",
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
                        crate::domains::signal_hub::service::process_signal_hub_raw_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "signal hub raw signal dispatcher consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        SIGNAL_HUB_RAW_SIGNAL_RUNTIME,
        RuntimeTaskClass::Essential,
        RuntimeExitPolicy::ShutdownRuntime,
        task,
    ))
}

fn register_communication_provider_observation_consumer(database_url: &str) -> bool {
    match COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "communication provider observation consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_persona_derived_evidence_consumer(database_url: &str) -> bool {
    match PERSONA_DERIVED_EVIDENCE_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "persona derived evidence consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_persona_identity_review_inbox_consumer(database_url: &str) -> bool {
    match PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "persona identity review inbox consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_project_link_review_effects_consumer(database_url: &str) -> bool {
    match PROJECT_LINK_REVIEW_EFFECTS_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "project link review effects consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_realtime_conversation_transcript_execution_consumer(database_url: &str) -> bool {
    match REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(_) => {
            tracing::warn!(
                "realtime conversation transcript execution consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_realtime_conversation_transcript_projection_consumer(database_url: &str) -> bool {
    match REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "realtime conversation transcript projection consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_signal_hub_raw_signal_consumer(database_url: &str) -> bool {
    match SIGNAL_HUB_RAW_SIGNAL_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "signal hub raw signal consumer registry is unavailable"
            );
            false
        }
    }
}
