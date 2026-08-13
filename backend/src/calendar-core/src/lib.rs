#![forbid(unsafe_code)]

mod lifecycle;

pub use lifecycle::{
    CalendarConstraintsV1, CalendarEventDraftV1, CalendarEventRecordV1, CalendarEventStateV1,
    CalendarLifecycleErrorV1, CalendarOutcomeKindV1, CalendarOutcomeV1,
    CalendarParticipantResponseV1, CalendarParticipantRoleV1, CalendarParticipantV1,
    CalendarReminderStateV1, CalendarReminderV1, CalendarTimestampV1, MAX_ADDRESS_CHARS_V1,
    MAX_DESCRIPTION_CHARS_V1, MAX_OUTCOME_NOTE_CHARS_V1, MAX_TIMEZONE_CHARS_V1,
    add_calendar_participant_v1, add_calendar_reminder_v1, create_calendar_event_v1,
    derive_calendar_event_id_v1, derive_calendar_outcome_id_v1, derive_calendar_participant_id_v1,
    derive_calendar_reminder_id_v1, fire_calendar_reminder_v1, record_calendar_outcome_v1,
    remove_calendar_participant_v1, remove_calendar_reminder_v1, set_calendar_constraints_v1,
    set_calendar_event_state_v1, update_calendar_event_v1, update_calendar_participant_v1,
    validate_calendar_event_record_v1,
};

pub const PACKAGE: &str = "makosh-calendar-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_TITLE_CHARS_V1: usize = 240;
