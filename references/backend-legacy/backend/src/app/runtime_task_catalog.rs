use makosh_desktop_runtime::RuntimeTaskSpec;

use crate::app::router::ApplicationComponents;

pub(crate) fn collect(components: &ApplicationComponents) -> Vec<RuntimeTaskSpec> {
    let mut tasks =
        crate::app::vault_reconciliation::lifecycle::host_vault_manifest_reconciliation_task(
            &components.state,
        )
        .into_iter()
        .collect::<Vec<_>>();
    let bootstrap = components.bootstrap.clone();
    tasks.extend(crate::application::bootstrap::zulip::runtime_task_specs(
        bootstrap.clone(),
    ));
    tasks.extend(crate::application::bootstrap::mail::runtime_task_specs(
        bootstrap.clone(),
    ));
    tasks.extend(crate::application::bootstrap::whatsapp::runtime_task_specs(
        bootstrap.clone(),
    ));
    tasks.extend(crate::application::bootstrap::telegram::runtime_task_specs(
        bootstrap.clone(),
    ));
    tasks.extend(crate::application::bootstrap::zoom::tasks::runtime_task_specs(bootstrap.clone()));
    tasks.extend(crate::application::bootstrap::telemost::runtime_task_specs(
        bootstrap.clone(),
    ));
    tasks.extend(crate::application::bootstrap::core::runtime_task_specs(
        bootstrap,
    ));
    tasks
}
