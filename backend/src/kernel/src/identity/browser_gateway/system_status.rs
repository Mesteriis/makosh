use makosh_gateway_session_contract::{
    ClientSystemComponentIdV1, ClientSystemComponentStateV1,
    ClientSystemComponentStatusProjectionV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::platform::{
    blob::status as blob_status, events::authority::status as events_status,
    scheduler::status as scheduler_status, storage::status as storage_status,
    telemetry::diagnostics as telemetry_diagnostics, vault::status as vault_status,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(crate) fn client_system_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    developer_realtime_enabled: bool,
) -> Vec<ClientSystemComponentStatusProjectionV1> {
    use ClientSystemComponentIdV1::{ControlStore, Gateway, Kernel, ModuleControlPlane};
    use ClientSystemComponentStateV1::Healthy;

    let mut statuses = [Kernel, ControlStore, ModuleControlPlane, Gateway]
        .into_iter()
        .map(|component| ClientSystemComponentStatusProjectionV1::new(component, Healthy, None))
        .collect::<Vec<_>>();
    statuses.extend(platform_statuses(store, supervisor));
    statuses.push(realtime_status(developer_realtime_enabled));
    statuses
}

fn platform_statuses(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) -> Vec<ClientSystemComponentStatusProjectionV1> {
    use ClientSystemComponentIdV1::{
        Blob, Clock, EventHub, Nats, Pgbouncer, Postgresql, Scheduler, StorageControl, Telemetry,
        Vault,
    };
    let relay = supervisor.relay_port();
    let vault = runtime_status(
        store,
        Vault,
        "vault",
        vault_status::read_current(store, &relay).map(|_| ()),
    );
    let storage = runtime_status(
        store,
        StorageControl,
        "storage",
        storage_status::read_current(store, &relay).map(|_| ()),
    );
    let events = runtime_status(
        store,
        EventHub,
        "events_authority",
        events_status::read_current(store, &relay).map(|_| ()),
    );
    let mut statuses = vec![vault, storage.clone()];
    statuses.push(derived_platform_status(Postgresql, &storage));
    statuses.push(derived_platform_status(Pgbouncer, &storage));
    statuses.push(derived_platform_status(Nats, &events));
    statuses.push(events);
    let scheduler = runtime_status(
        store,
        Scheduler,
        "scheduler_developer",
        scheduler_status::read_current(store, &relay),
    );
    statuses.push(scheduler.clone());
    statuses.push(derived_platform_status(Clock, &scheduler));
    statuses.push(runtime_status(
        store,
        Blob,
        "blob",
        blob_status::read_current(store, &relay).map(|_| ()),
    ));
    statuses.push(runtime_status(
        store,
        Telemetry,
        "telemetry",
        telemetry_diagnostics::read(supervisor).map(|_| ()),
    ));
    statuses
}

fn runtime_status(
    store: &SqliteControlStore,
    component: ClientSystemComponentIdV1,
    process_id: &str,
    current: Result<(), String>,
) -> ClientSystemComponentStatusProjectionV1 {
    use ClientSystemComponentStateV1::{Degraded, Healthy, NotAdmitted, Unavailable};
    if current.is_ok() {
        return ClientSystemComponentStatusProjectionV1::new(component, Healthy, None);
    }
    match store.platform_managed_process_launch(process_id) {
        Ok(Some(_)) => ClientSystemComponentStatusProjectionV1::new(
            component,
            Degraded,
            Some("runtime_liveness_not_observed".to_owned()),
        ),
        Ok(None) => ClientSystemComponentStatusProjectionV1::new(
            component,
            NotAdmitted,
            Some("runtime_status_not_admitted".to_owned()),
        ),
        Err(_) => ClientSystemComponentStatusProjectionV1::new(
            component,
            Unavailable,
            Some("runtime_status_unavailable".to_owned()),
        ),
    }
}

fn realtime_status(developer_realtime_enabled: bool) -> ClientSystemComponentStatusProjectionV1 {
    use ClientSystemComponentIdV1::Sse;
    use ClientSystemComponentStateV1::{Healthy, Unavailable};
    if developer_realtime_enabled {
        ClientSystemComponentStatusProjectionV1::new(Sse, Healthy, None)
    } else {
        ClientSystemComponentStatusProjectionV1::new(
            Sse,
            Unavailable,
            Some("client_realtime_owner_not_admitted".to_owned()),
        )
    }
}

fn derived_platform_status(
    component_id: ClientSystemComponentIdV1,
    owner: &ClientSystemComponentStatusProjectionV1,
) -> ClientSystemComponentStatusProjectionV1 {
    ClientSystemComponentStatusProjectionV1::new(
        component_id,
        owner.state(),
        owner.sanitized_reason_code().map(str::to_owned),
    )
}

#[cfg(test)]
mod tests {
    use makosh_gateway_session_contract::{
        ClientSystemComponentIdV1, ClientSystemComponentStateV1,
    };

    use super::realtime_status;

    #[test]
    fn development_realtime_status_tracks_the_admitted_source() {
        let unavailable = realtime_status(false);
        assert_eq!(unavailable.component_id(), ClientSystemComponentIdV1::Sse);
        assert_eq!(
            unavailable.state(),
            ClientSystemComponentStateV1::Unavailable
        );
        assert_eq!(
            unavailable.sanitized_reason_code(),
            Some("client_realtime_owner_not_admitted")
        );

        let healthy = realtime_status(true);
        assert_eq!(healthy.component_id(), ClientSystemComponentIdV1::Sse);
        assert_eq!(healthy.state(), ClientSystemComponentStateV1::Healthy);
        assert_eq!(healthy.sanitized_reason_code(), None);
    }
}
