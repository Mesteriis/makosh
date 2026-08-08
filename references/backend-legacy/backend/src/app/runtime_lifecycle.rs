use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeLifecycleEvent, RuntimeLifecycleObserver, RuntimeTaskClass,
    RuntimeTaskError, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskLifecycleTarget,
    RuntimeTaskSpec,
};
use makosh_signal_hub_api::runtime_lifecycle::{RuntimeLifecyclePort, RuntimeLifecycleUpdate};
use makosh_signal_hub_postgres::runtime_lifecycle::RuntimeLifecycleStore;
use serde_json::json;
use sqlx::postgres::PgPool;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const SIGNAL_HUB_RUNTIME_LIFECYCLE_PROJECTION: &str = "signal_hub_runtime_lifecycle_projection";

pub(crate) struct LifecycleProjectionRuntime {
    pub(crate) observer: Arc<dyn RuntimeLifecycleObserver>,
    pub(crate) task: RuntimeTaskSpec,
}

pub(crate) fn build_lifecycle_projection_runtime(
    tasks: &[RuntimeTaskSpec],
    pool: Option<PgPool>,
) -> Option<LifecycleProjectionRuntime> {
    let pool = pool?;
    let targets = lifecycle_targets(tasks);
    if targets.is_empty() {
        return None;
    }

    let (sender, receiver) = mpsc::unbounded_channel();
    let receiver = Arc::new(Mutex::new(Some(receiver)));
    let observer = Arc::new(SignalHubLifecycleObserver { targets, sender });
    let task_receiver = Arc::clone(&receiver);
    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let receiver = Arc::clone(&task_receiver);
        let pool = pool.clone();
        Box::pin(async move {
            let mut receiver = receiver
                .lock()
                .map_err(|_| RuntimeTaskError::Coded {
                    code: "signal_hub_lifecycle_receiver_poisoned".to_owned(),
                })?
                .take()
                .ok_or_else(|| RuntimeTaskError::Coded {
                    code: "signal_hub_lifecycle_receiver_already_started".to_owned(),
                })?;
            let store = RuntimeLifecycleStore::new(pool);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    update = receiver.recv() => match update {
                        Some(update) => {
                            if let Err(error) = store.record_runtime_lifecycle(&update).await {
                                tracing::error!(
                                    error = %error,
                                    source_code = %update.source_code,
                                    runtime_kind = %update.runtime_kind,
                                    "signal hub runtime lifecycle projection failed"
                                );
                                return Err(RuntimeTaskError::Coded {
                                    code: "signal_hub_lifecycle_projection_failed".to_owned(),
                                });
                            }
                        }
                        None => return Ok(()),
                    },
                }
            }
        }) as RuntimeTaskFuture
    });

    Some(LifecycleProjectionRuntime {
        observer,
        task: RuntimeTaskSpec::new(
            SIGNAL_HUB_RUNTIME_LIFECYCLE_PROJECTION,
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        ),
    })
}

struct SignalHubLifecycleObserver {
    targets: HashMap<String, RuntimeTaskLifecycleTarget>,
    sender: mpsc::UnboundedSender<RuntimeLifecycleUpdate>,
}

impl RuntimeLifecycleObserver for SignalHubLifecycleObserver {
    fn on_event(&self, event: RuntimeLifecycleEvent) {
        let Some((task_name, target)) = lifecycle_target_for_event(&self.targets, &event) else {
            return;
        };
        let update = lifecycle_update(task_name, target, event);
        if self.sender.send(update).is_err() {
            tracing::error!(
                task_name,
                "signal hub lifecycle projection receiver is unavailable"
            );
        }
    }
}

fn lifecycle_targets(tasks: &[RuntimeTaskSpec]) -> HashMap<String, RuntimeTaskLifecycleTarget> {
    tasks
        .iter()
        .filter_map(|task| {
            task.lifecycle_target
                .as_ref()
                .map(|target| (task.name.clone(), target.clone()))
        })
        .collect()
}

fn lifecycle_target_for_event<'a>(
    targets: &'a HashMap<String, RuntimeTaskLifecycleTarget>,
    event: &RuntimeLifecycleEvent,
) -> Option<(&'a str, &'a RuntimeTaskLifecycleTarget)> {
    let task_name = match event {
        RuntimeLifecycleEvent::Started { task_name, .. }
        | RuntimeLifecycleEvent::Degraded { task_name, .. }
        | RuntimeLifecycleEvent::Stopped { task_name }
        | RuntimeLifecycleEvent::Aborted { task_name, .. } => task_name,
    };
    targets
        .get_key_value(task_name)
        .map(|(task_name, target)| (task_name.as_str(), target))
}

fn lifecycle_update(
    task_name: &str,
    target: &RuntimeTaskLifecycleTarget,
    event: RuntimeLifecycleEvent,
) -> RuntimeLifecycleUpdate {
    match event {
        RuntimeLifecycleEvent::Started { class, .. } => RuntimeLifecycleUpdate::running(
            &target.source_code,
            &target.runtime_kind,
            task_name,
            json!({ "class": runtime_task_class_name(class) }),
        ),
        RuntimeLifecycleEvent::Degraded { error_code, .. } => RuntimeLifecycleUpdate::degraded(
            &target.source_code,
            &target.runtime_kind,
            task_name,
            error_code,
            json!({ "state": "error" }),
        ),
        RuntimeLifecycleEvent::Stopped { .. } => RuntimeLifecycleUpdate::stopped(
            &target.source_code,
            &target.runtime_kind,
            task_name,
            json!({}),
        ),
        RuntimeLifecycleEvent::Aborted { phase, .. } => RuntimeLifecycleUpdate::stopped(
            &target.source_code,
            &target.runtime_kind,
            task_name,
            json!({ "shutdown_phase": format!("{phase:?}").to_lowercase() }),
        ),
    }
}

fn runtime_task_class_name(class: RuntimeTaskClass) -> &'static str {
    match class {
        RuntimeTaskClass::Essential => "essential",
        RuntimeTaskClass::Background => "background",
        RuntimeTaskClass::Startup => "startup",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use makosh_desktop_runtime::{
        RuntimeExitPolicy, RuntimeLifecycleEvent, RuntimeTaskClass, RuntimeTaskFactory,
        RuntimeTaskFuture, RuntimeTaskSpec,
    };
    use serde_json::json;
    use tokio_util::sync::CancellationToken;

    use super::{lifecycle_target_for_event, lifecycle_targets, lifecycle_update};

    fn task() -> RuntimeTaskSpec {
        let task: RuntimeTaskFactory =
            Arc::new(|_: CancellationToken| Box::pin(async { Ok(()) }) as RuntimeTaskFuture);
        RuntimeTaskSpec::new(
            "zulip_event_ingest",
            RuntimeTaskClass::Background,
            RuntimeExitPolicy::MarkDegraded,
            task,
        )
        .with_lifecycle_target("zulip", "zulip_event_ingest")
    }

    #[test]
    fn lifecycle_projection_uses_explicit_task_target() {
        let targets = lifecycle_targets(&[task()]);
        let event = RuntimeLifecycleEvent::Degraded {
            task_name: "zulip_event_ingest".to_owned(),
            error_code: "provider_unavailable".to_owned(),
        };
        let (task_name, target) =
            lifecycle_target_for_event(&targets, &event).expect("explicit lifecycle target");
        let update = lifecycle_update(task_name, target, event);

        assert_eq!(update.source_code, "zulip");
        assert_eq!(update.runtime_kind, "zulip_event_ingest");
        assert_eq!(update.error_code.as_deref(), Some("provider_unavailable"));
        assert_eq!(update.metadata, json!({ "state": "error" }));
    }
}
