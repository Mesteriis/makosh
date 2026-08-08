//! Sanitized current status of the managed Events credential authority.

use makosh_kernel_control_store::{
    PlatformEventHubTopologyV1, PlatformEventsAuthorityConfigurationV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
        EventsAuthorityRuntimeStateV1, EventsAuthorityRuntimeStatusV1,
        GetEventsAuthorityRuntimeStatusRequestV1,
        events_authority_runtime_control_request_v1::Operation,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::events_authority::validate_events_authority_runtime_status,
};
use prost::Message;

use crate::platform::events::authority::{binding::EVENTS_AUTHORITY_PROCESS_ID, launch};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

pub struct ManagedEventsAuthorityStatus {
    runtime_generation: u64,
}

impl ManagedEventsAuthorityStatus {
    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
}

pub fn read_current(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<ManagedEventsAuthorityStatus, String> {
    let launch = launch::current_launch(store)?;
    let configuration = current_configuration(store)?;
    let request = EventsAuthorityRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(
            GetEventsAuthorityRuntimeStatusRequestV1 {},
        )),
    };
    let response = relay.relay(EVENTS_AUTHORITY_PROCESS_ID, request.encode_to_vec())?;
    let response = EventsAuthorityRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Events authority status response is invalid".to_owned())?;
    parse_current(response, launch.runtime_generation(), &configuration)
}

pub(crate) fn current_configuration(
    store: &SqliteControlStore,
) -> Result<PlatformEventsAuthorityConfigurationV1, String> {
    store
        .platform_events_authority_configuration()
        .map_err(|_| "Events authority configuration is unavailable".to_owned())?
        .ok_or_else(|| "Events authority configuration is unavailable".to_owned())
}

pub(crate) fn current_topology(
    store: &SqliteControlStore,
) -> Result<PlatformEventHubTopologyV1, String> {
    store
        .platform_event_hub_topology()
        .map_err(|_| "Event Hub topology is unavailable".to_owned())?
        .ok_or_else(|| "Event Hub topology is unavailable".to_owned())
}

pub(crate) fn parse_current(
    response: EventsAuthorityRuntimeControlResponseV1,
    expected_generation: u64,
    configuration: &PlatformEventsAuthorityConfigurationV1,
) -> Result<ManagedEventsAuthorityStatus, String> {
    if !response.error_code.is_empty() {
        return Err("managed Events authority status is unavailable".to_owned());
    }
    let Some(ResponseResult::Status(status)) = response.result else {
        return Err("managed Events authority status response is invalid".to_owned());
    };
    validate_current_status(status, expected_generation, configuration)
}

fn validate_current_status(
    status: EventsAuthorityRuntimeStatusV1,
    expected_generation: u64,
    configuration: &PlatformEventsAuthorityConfigurationV1,
) -> Result<ManagedEventsAuthorityStatus, String> {
    validate_events_authority_runtime_status(&status)
        .map_err(|_| "managed Events authority status response is invalid".to_owned())?;
    if EventsAuthorityRuntimeStateV1::try_from(status.state).ok()
        != Some(EventsAuthorityRuntimeStateV1::Ready)
        || status.runtime_generation != expected_generation
        || status.signer_credential_revision != configuration.signer_credential_revision()
    {
        return Err("managed Events authority status is stale or unavailable".to_owned());
    }
    Ok(ManagedEventsAuthorityStatus {
        runtime_generation: status.runtime_generation,
    })
}
