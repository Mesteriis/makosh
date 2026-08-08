//! Provider-neutral runtime tasks owned by the desktop composition layer.

use makosh_desktop_runtime::RuntimeTaskSpec;

use super::ApplicationBootstrapContext;

pub(crate) mod dispatchers;
pub(crate) mod projections;

pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    let mut tasks = projections::runtime_task_specs(context.clone());
    tasks.extend(dispatchers::runtime_task_specs(context));
    tasks
}
