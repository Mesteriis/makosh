#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    CalendarEnvelopeBuildErrorV1, CalendarEnvelopeContextV1,
    build_calendar_event_changed_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-calendar-api";
pub const CALENDAR_OWNER_ID_V1: &str = "calendar";
pub const CALENDAR_MODULE_ID_V1: &str = "makosh-calendar-runtime";
pub const CALENDAR_CLIENT_CAPABILITY_ID_V1: &str = "calendar.client.v1";
pub const CALENDAR_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "calendar.lifecycle.event.v1";
pub const CALENDAR_SCHEDULER_DUE_CAPABILITY_ID_V1: &str = "calendar.scheduler.due.v1";
pub const CALENDAR_SCHEDULER_RECEIPT_CAPABILITY_ID_V1: &str = "calendar.scheduler.receipt.v1";
pub const CALENDAR_SCHEDULER_SCHEDULE_COMMAND_CAPABILITY_ID_V1: &str =
    "calendar.scheduler.schedule-command.v1";
pub const CALENDAR_SCHEDULER_SCHEDULE_RESULT_CAPABILITY_ID_V1: &str =
    "calendar.scheduler.schedule-result.v1";
pub const CALENDAR_STORAGE_CAPABILITY_ID_V1: &str = "calendar.storage.v1";
pub const CALENDAR_LIFECYCLE_EVENT_CONTRACT_NAME_V1: &str = "calendar_event_changed";
pub const CALENDAR_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const CALENDAR_CLIENT_CONTRACT_REVISION_V1: u32 = 1;

pub const CALENDAR_CREATE_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/Create";
pub const CALENDAR_UPDATE_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/Update";
pub const CALENDAR_SET_STATE_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/SetState";
pub const CALENDAR_ADD_PARTICIPANT_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/AddParticipant";
pub const CALENDAR_UPDATE_PARTICIPANT_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/UpdateParticipant";
pub const CALENDAR_REMOVE_PARTICIPANT_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/RemoveParticipant";
pub const CALENDAR_SET_CONSTRAINTS_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/SetConstraints";
pub const CALENDAR_ADD_REMINDER_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/AddReminder";
pub const CALENDAR_REMOVE_REMINDER_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/RemoveReminder";
pub const CALENDAR_RECORD_OUTCOME_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarCommandService/RecordOutcome";
pub const CALENDAR_GET_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarQueryService/Get";
pub const CALENDAR_LIST_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarQueryService/List";
pub const CALENDAR_SEARCH_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarQueryService/Search";
pub const CALENDAR_LIST_PARTICIPANTS_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarQueryService/ListParticipants";
pub const CALENDAR_LIST_REMINDERS_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarQueryService/ListReminders";
pub const CALENDAR_LIST_OUTCOMES_CONNECT_PATH_V1: &str =
    "/makosh.calendar.client.v1.CalendarQueryService/ListOutcomes";

pub mod client_wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.calendar.client.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/calendar_client_schema.rs"));

pub const CALENDAR_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/calendar-client-v1.bin"));

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CALENDAR_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: CALENDAR_CLIENT_CONTRACT_MAJOR_V1,
        revision: CALENDAR_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: CALENDAR_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

macro_rules! client_contract {
    ($function:ident, $name:literal) => {
        #[must_use]
        pub fn $function() -> ContractReferenceV1 {
            contract_reference($name)
        }
    };
}

client_contract!(
    calendar_client_create_contract_reference_v1,
    "calendar_client_create"
);
client_contract!(
    calendar_client_update_contract_reference_v1,
    "calendar_client_update"
);
client_contract!(
    calendar_client_set_state_contract_reference_v1,
    "calendar_client_set_state"
);
client_contract!(
    calendar_client_add_participant_contract_reference_v1,
    "calendar_client_add_participant"
);
client_contract!(
    calendar_client_update_participant_contract_reference_v1,
    "calendar_client_update_participant"
);
client_contract!(
    calendar_client_remove_participant_contract_reference_v1,
    "calendar_client_remove_participant"
);
client_contract!(
    calendar_client_set_constraints_contract_reference_v1,
    "calendar_client_set_constraints"
);
client_contract!(
    calendar_client_add_reminder_contract_reference_v1,
    "calendar_client_add_reminder"
);
client_contract!(
    calendar_client_remove_reminder_contract_reference_v1,
    "calendar_client_remove_reminder"
);
client_contract!(
    calendar_client_record_outcome_contract_reference_v1,
    "calendar_client_record_outcome"
);
client_contract!(
    calendar_client_get_contract_reference_v1,
    "calendar_client_get"
);
client_contract!(
    calendar_client_list_contract_reference_v1,
    "calendar_client_list"
);
client_contract!(
    calendar_client_search_contract_reference_v1,
    "calendar_client_search"
);
client_contract!(
    calendar_client_list_participants_contract_reference_v1,
    "calendar_client_list_participants"
);
client_contract!(
    calendar_client_list_reminders_contract_reference_v1,
    "calendar_client_list_reminders"
);
client_contract!(
    calendar_client_list_outcomes_contract_reference_v1,
    "calendar_client_list_outcomes"
);

#[must_use]
pub fn calendar_lifecycle_event_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CALENDAR_LIFECYCLE_EVENT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn calendar_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(calendar_lifecycle_event_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[must_use]
pub fn calendar_client_routes_v1() -> [(ContractReferenceV1, &'static str); 16] {
    [
        (
            calendar_client_create_contract_reference_v1(),
            CALENDAR_CREATE_CONNECT_PATH_V1,
        ),
        (
            calendar_client_update_contract_reference_v1(),
            CALENDAR_UPDATE_CONNECT_PATH_V1,
        ),
        (
            calendar_client_set_state_contract_reference_v1(),
            CALENDAR_SET_STATE_CONNECT_PATH_V1,
        ),
        (
            calendar_client_add_participant_contract_reference_v1(),
            CALENDAR_ADD_PARTICIPANT_CONNECT_PATH_V1,
        ),
        (
            calendar_client_update_participant_contract_reference_v1(),
            CALENDAR_UPDATE_PARTICIPANT_CONNECT_PATH_V1,
        ),
        (
            calendar_client_remove_participant_contract_reference_v1(),
            CALENDAR_REMOVE_PARTICIPANT_CONNECT_PATH_V1,
        ),
        (
            calendar_client_set_constraints_contract_reference_v1(),
            CALENDAR_SET_CONSTRAINTS_CONNECT_PATH_V1,
        ),
        (
            calendar_client_add_reminder_contract_reference_v1(),
            CALENDAR_ADD_REMINDER_CONNECT_PATH_V1,
        ),
        (
            calendar_client_remove_reminder_contract_reference_v1(),
            CALENDAR_REMOVE_REMINDER_CONNECT_PATH_V1,
        ),
        (
            calendar_client_record_outcome_contract_reference_v1(),
            CALENDAR_RECORD_OUTCOME_CONNECT_PATH_V1,
        ),
        (
            calendar_client_get_contract_reference_v1(),
            CALENDAR_GET_CONNECT_PATH_V1,
        ),
        (
            calendar_client_list_contract_reference_v1(),
            CALENDAR_LIST_CONNECT_PATH_V1,
        ),
        (
            calendar_client_search_contract_reference_v1(),
            CALENDAR_SEARCH_CONNECT_PATH_V1,
        ),
        (
            calendar_client_list_participants_contract_reference_v1(),
            CALENDAR_LIST_PARTICIPANTS_CONNECT_PATH_V1,
        ),
        (
            calendar_client_list_reminders_contract_reference_v1(),
            CALENDAR_LIST_REMINDERS_CONNECT_PATH_V1,
        ),
        (
            calendar_client_list_outcomes_contract_reference_v1(),
            CALENDAR_LIST_OUTCOMES_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_and_capability_surface_is_exact_and_provider_neutral() {
        assert_eq!(calendar_client_routes_v1().len(), 16);
        assert_eq!(
            [
                CALENDAR_CLIENT_CAPABILITY_ID_V1,
                CALENDAR_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
                CALENDAR_SCHEDULER_DUE_CAPABILITY_ID_V1,
                CALENDAR_SCHEDULER_RECEIPT_CAPABILITY_ID_V1,
                CALENDAR_SCHEDULER_SCHEDULE_COMMAND_CAPABILITY_ID_V1,
                CALENDAR_SCHEDULER_SCHEDULE_RESULT_CAPABILITY_ID_V1,
                CALENDAR_STORAGE_CAPABILITY_ID_V1,
            ],
            [
                "calendar.client.v1",
                "calendar.lifecycle.event.v1",
                "calendar.scheduler.due.v1",
                "calendar.scheduler.receipt.v1",
                "calendar.scheduler.schedule-command.v1",
                "calendar.scheduler.schedule-result.v1",
                "calendar.storage.v1",
            ]
        );
        let source = include_str!("../proto/makosh/calendar/client/v1/calendar.proto");
        for private in [
            "provider",
            "credential",
            "private_locator",
            "google",
            "apple",
            "caldav",
        ] {
            assert!(!source.to_ascii_lowercase().contains(private), "{private}");
        }
    }
}
