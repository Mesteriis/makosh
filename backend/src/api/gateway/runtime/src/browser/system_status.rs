use makosh_gateway_protocol::v1::{
    ClientSystemComponentIdV1 as WireSystemComponentId,
    ClientSystemComponentStateV1 as WireSystemComponentState, ClientSystemComponentStatusV1,
};
use makosh_gateway_session_contract::{
    ClientSystemComponentIdV1 as ProjectionSystemComponentId,
    ClientSystemComponentStateV1 as ProjectionSystemComponentState,
    ClientSystemComponentStatusProjectionV1,
};

pub(crate) fn wire_system_statuses(
    statuses: &[ClientSystemComponentStatusProjectionV1],
) -> Vec<ClientSystemComponentStatusV1> {
    statuses.iter().map(wire_system_status).collect()
}

fn wire_system_status(
    status: &ClientSystemComponentStatusProjectionV1,
) -> ClientSystemComponentStatusV1 {
    ClientSystemComponentStatusV1 {
        component_id: system_component_id(status.component_id()) as i32,
        state: system_component_state(status.state()) as i32,
        sanitized_reason_code: status
            .sanitized_reason_code()
            .unwrap_or_default()
            .to_owned(),
    }
}

fn system_component_id(value: ProjectionSystemComponentId) -> WireSystemComponentId {
    match value {
        ProjectionSystemComponentId::Kernel => WireSystemComponentId::Kernel,
        ProjectionSystemComponentId::ControlStore => WireSystemComponentId::ControlStore,
        ProjectionSystemComponentId::ModuleControlPlane => {
            WireSystemComponentId::ModuleControlPlane
        }
        ProjectionSystemComponentId::Gateway => WireSystemComponentId::Gateway,
        ProjectionSystemComponentId::Vault => WireSystemComponentId::Vault,
        ProjectionSystemComponentId::StorageControl => WireSystemComponentId::StorageControl,
        ProjectionSystemComponentId::Postgresql => WireSystemComponentId::Postgresql,
        ProjectionSystemComponentId::Pgbouncer => WireSystemComponentId::Pgbouncer,
        ProjectionSystemComponentId::Nats => WireSystemComponentId::Nats,
        ProjectionSystemComponentId::EventHub => WireSystemComponentId::EventHub,
        ProjectionSystemComponentId::Scheduler => WireSystemComponentId::Scheduler,
        ProjectionSystemComponentId::Clock => WireSystemComponentId::Clock,
        ProjectionSystemComponentId::Blob => WireSystemComponentId::Blob,
        ProjectionSystemComponentId::Telemetry => WireSystemComponentId::Telemetry,
        ProjectionSystemComponentId::Sse => WireSystemComponentId::Sse,
    }
}

fn system_component_state(value: ProjectionSystemComponentState) -> WireSystemComponentState {
    match value {
        ProjectionSystemComponentState::Healthy => WireSystemComponentState::Healthy,
        ProjectionSystemComponentState::Degraded => WireSystemComponentState::Degraded,
        ProjectionSystemComponentState::Unavailable => WireSystemComponentState::Unavailable,
        ProjectionSystemComponentState::NotAdmitted => WireSystemComponentState::NotAdmitted,
    }
}
