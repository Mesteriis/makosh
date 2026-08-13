use sha2::{Digest, Sha256};

use crate::{MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1, STABLE_ID_BYTES_V1};

pub const MAX_DESCRIPTION_CHARS_V1: usize = 8_000;
pub const MAX_DISPLAY_NAME_CHARS_V1: usize = 200;
pub const MAX_ADDRESS_CHARS_V1: usize = 320;
pub const MAX_TIMEZONE_CHARS_V1: usize = 128;
pub const MAX_OUTCOME_NOTE_CHARS_V1: usize = 2_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CalendarTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarEventStateV1 {
    Scheduled,
    Completed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarParticipantRoleV1 {
    Organizer,
    Required,
    Optional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarParticipantResponseV1 {
    Pending,
    Accepted,
    Declined,
    Tentative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarReminderStateV1 {
    Pending,
    Fired,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarOutcomeKindV1 {
    Completed,
    Cancelled,
    NoShow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarParticipantV1 {
    pub participant_id: [u8; STABLE_ID_BYTES_V1],
    pub display_name: String,
    pub address: String,
    pub role: CalendarParticipantRoleV1,
    pub response: CalendarParticipantResponseV1,
    pub updated_at_event_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarConstraintsV1 {
    pub earliest_start: CalendarTimestampV1,
    pub latest_end: CalendarTimestampV1,
    pub minimum_duration_minutes: u32,
    pub timezone: String,
    pub updated_at_event_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarReminderV1 {
    pub reminder_id: [u8; STABLE_ID_BYTES_V1],
    pub due_at: CalendarTimestampV1,
    pub state: CalendarReminderStateV1,
    pub updated_at_event_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarOutcomeV1 {
    pub outcome_id: [u8; STABLE_ID_BYTES_V1],
    pub kind: CalendarOutcomeKindV1,
    pub note: String,
    pub recorded_at: CalendarTimestampV1,
    pub recorded_at_event_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEventDraftV1 {
    pub operation_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub description: String,
    pub starts_at: CalendarTimestampV1,
    pub ends_at: CalendarTimestampV1,
    pub timezone: String,
    pub created_at: CalendarTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEventRecordV1 {
    pub calendar_event_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub description: String,
    pub starts_at: CalendarTimestampV1,
    pub ends_at: CalendarTimestampV1,
    pub timezone: String,
    pub state: CalendarEventStateV1,
    pub event_revision: u64,
    pub participants: Vec<CalendarParticipantV1>,
    pub constraints: Option<CalendarConstraintsV1>,
    pub reminders: Vec<CalendarReminderV1>,
    pub outcomes: Vec<CalendarOutcomeV1>,
    pub created_at: CalendarTimestampV1,
    pub updated_at: CalendarTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarLifecycleErrorV1 {
    InvalidOwner,
    InvalidOperationId,
    InvalidEventId,
    InvalidTitle,
    InvalidDescription,
    InvalidTimestamp,
    InvalidTimezone,
    InvalidRevision,
    RevisionOverflow,
    InvalidStateTransition,
    InvalidParticipant,
    ParticipantExists,
    ParticipantNotFound,
    OrganizerExists,
    InvalidConstraints,
    InvalidReminder,
    ReminderExists,
    ReminderNotFound,
    ReminderNotPending,
    InvalidOutcome,
    OutcomeExists,
}

pub fn derive_calendar_event_id_v1(
    logical_owner_id: &str,
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(CalendarLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(operation_id) {
        return Err(CalendarLifecycleErrorV1::InvalidOperationId);
    }
    Ok(derive_id(
        b"makosh.calendar.event-id.v1\0",
        &[logical_owner_id.as_bytes(), operation_id],
    ))
}

pub fn derive_calendar_participant_id_v1(
    calendar_event_id: &[u8; STABLE_ID_BYTES_V1],
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    derive_child_id(
        b"makosh.calendar.participant-id.v1\0",
        calendar_event_id,
        operation_id,
    )
}

pub fn derive_calendar_reminder_id_v1(
    calendar_event_id: &[u8; STABLE_ID_BYTES_V1],
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    derive_child_id(
        b"makosh.calendar.reminder-id.v1\0",
        calendar_event_id,
        operation_id,
    )
}

pub fn derive_calendar_outcome_id_v1(
    calendar_event_id: &[u8; STABLE_ID_BYTES_V1],
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    derive_child_id(
        b"makosh.calendar.outcome-id.v1\0",
        calendar_event_id,
        operation_id,
    )
}

pub fn create_calendar_event_v1(
    draft: CalendarEventDraftV1,
) -> Result<CalendarEventRecordV1, CalendarLifecycleErrorV1> {
    validate_text(&draft.title, MAX_TITLE_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidTitle)?;
    validate_optional_text(&draft.description, MAX_DESCRIPTION_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidDescription)?;
    validate_timezone(&draft.timezone)?;
    validate_interval(draft.starts_at, draft.ends_at)?;
    validate_timestamp(draft.created_at)?;
    let event = CalendarEventRecordV1 {
        calendar_event_id: derive_calendar_event_id_v1(
            &draft.logical_owner_id,
            &draft.operation_id,
        )?,
        logical_owner_id: draft.logical_owner_id,
        title: draft.title,
        description: draft.description,
        starts_at: draft.starts_at,
        ends_at: draft.ends_at,
        timezone: draft.timezone,
        state: CalendarEventStateV1::Scheduled,
        event_revision: 1,
        participants: Vec::new(),
        constraints: None,
        reminders: Vec::new(),
        outcomes: Vec::new(),
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_calendar_event_record_v1(&event)?;
    Ok(event)
}

#[allow(clippy::too_many_arguments)]
pub fn update_calendar_event_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    title: Option<String>,
    description: Option<String>,
    starts_at: Option<CalendarTimestampV1>,
    ends_at: Option<CalendarTimestampV1>,
    timezone: Option<String>,
    updated_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, updated_at)?;
    if title.is_none()
        && description.is_none()
        && starts_at.is_none()
        && ends_at.is_none()
        && timezone.is_none()
    {
        return Err(CalendarLifecycleErrorV1::InvalidDescription);
    }
    let next_title = title.unwrap_or_else(|| event.title.clone());
    let next_description = description.unwrap_or_else(|| event.description.clone());
    let next_starts_at = starts_at.unwrap_or(event.starts_at);
    let next_ends_at = ends_at.unwrap_or(event.ends_at);
    let next_timezone = timezone.unwrap_or_else(|| event.timezone.clone());
    validate_text(&next_title, MAX_TITLE_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidTitle)?;
    validate_optional_text(&next_description, MAX_DESCRIPTION_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidDescription)?;
    validate_timezone(&next_timezone)?;
    validate_interval(next_starts_at, next_ends_at)?;
    if event.reminders.iter().any(|reminder| {
        reminder.state == CalendarReminderStateV1::Pending && reminder.due_at > next_starts_at
    }) {
        return Err(CalendarLifecycleErrorV1::InvalidReminder);
    }
    event.title = next_title;
    event.description = next_description;
    event.starts_at = next_starts_at;
    event.ends_at = next_ends_at;
    event.timezone = next_timezone;
    advance(event, updated_at)
}

pub fn set_calendar_event_state_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    state: CalendarEventStateV1,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_revision_and_time(event, expected_revision, changed_at)?;
    if event.state != CalendarEventStateV1::Scheduled
        || !matches!(
            state,
            CalendarEventStateV1::Completed | CalendarEventStateV1::Cancelled
        )
    {
        return Err(CalendarLifecycleErrorV1::InvalidStateTransition);
    }
    event.state = state;
    let next_revision = next_revision(event)?;
    for reminder in &mut event.reminders {
        if reminder.state == CalendarReminderStateV1::Pending {
            reminder.state = CalendarReminderStateV1::Cancelled;
            reminder.updated_at_event_revision = next_revision;
        }
    }
    apply_revision(event, next_revision, changed_at)
}

#[allow(clippy::too_many_arguments)]
pub fn add_calendar_participant_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    operation_id: [u8; STABLE_ID_BYTES_V1],
    display_name: String,
    address: String,
    role: CalendarParticipantRoleV1,
    response: CalendarParticipantResponseV1,
    changed_at: CalendarTimestampV1,
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, changed_at)?;
    validate_participant(&display_name, &address)?;
    let participant_id =
        derive_calendar_participant_id_v1(&event.calendar_event_id, &operation_id)?;
    if event.participants.iter().any(|value| {
        value.participant_id == participant_id || value.address.eq_ignore_ascii_case(&address)
    }) {
        return Err(CalendarLifecycleErrorV1::ParticipantExists);
    }
    require_organizer_slot(event, None, role)?;
    let revision = next_revision(event)?;
    event.participants.push(CalendarParticipantV1 {
        participant_id,
        display_name,
        address,
        role,
        response,
        updated_at_event_revision: revision,
    });
    event.participants.sort_by_key(|value| value.participant_id);
    apply_revision(event, revision, changed_at)?;
    Ok(participant_id)
}

#[allow(clippy::too_many_arguments)]
pub fn update_calendar_participant_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    participant_id: [u8; STABLE_ID_BYTES_V1],
    display_name: Option<String>,
    address: Option<String>,
    role: Option<CalendarParticipantRoleV1>,
    response: Option<CalendarParticipantResponseV1>,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, changed_at)?;
    let index = event
        .participants
        .iter()
        .position(|value| value.participant_id == participant_id)
        .ok_or(CalendarLifecycleErrorV1::ParticipantNotFound)?;
    if display_name.is_none() && address.is_none() && role.is_none() && response.is_none() {
        return Err(CalendarLifecycleErrorV1::InvalidParticipant);
    }
    let next_name = display_name.unwrap_or_else(|| event.participants[index].display_name.clone());
    let next_address = address.unwrap_or_else(|| event.participants[index].address.clone());
    let next_role = role.unwrap_or(event.participants[index].role);
    let next_response = response.unwrap_or(event.participants[index].response);
    validate_participant(&next_name, &next_address)?;
    if event
        .participants
        .iter()
        .enumerate()
        .any(|(position, value)| {
            position != index && value.address.eq_ignore_ascii_case(&next_address)
        })
    {
        return Err(CalendarLifecycleErrorV1::ParticipantExists);
    }
    require_organizer_slot(event, Some(participant_id), next_role)?;
    let revision = next_revision(event)?;
    event.participants[index] = CalendarParticipantV1 {
        participant_id,
        display_name: next_name,
        address: next_address,
        role: next_role,
        response: next_response,
        updated_at_event_revision: revision,
    };
    apply_revision(event, revision, changed_at)
}

pub fn remove_calendar_participant_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    participant_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, changed_at)?;
    let index = event
        .participants
        .iter()
        .position(|value| value.participant_id == participant_id)
        .ok_or(CalendarLifecycleErrorV1::ParticipantNotFound)?;
    let revision = next_revision(event)?;
    event.participants.remove(index);
    apply_revision(event, revision, changed_at)
}

pub fn set_calendar_constraints_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    earliest_start: CalendarTimestampV1,
    latest_end: CalendarTimestampV1,
    minimum_duration_minutes: u32,
    timezone: String,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, changed_at)?;
    validate_interval(earliest_start, latest_end)?;
    validate_timezone(&timezone)?;
    let available_minutes = timestamp_millis(latest_end)?
        .checked_sub(timestamp_millis(earliest_start)?)
        .ok_or(CalendarLifecycleErrorV1::InvalidConstraints)?
        / 60_000;
    if minimum_duration_minutes == 0 || i128::from(minimum_duration_minutes) > available_minutes {
        return Err(CalendarLifecycleErrorV1::InvalidConstraints);
    }
    let revision = next_revision(event)?;
    event.constraints = Some(CalendarConstraintsV1 {
        earliest_start,
        latest_end,
        minimum_duration_minutes,
        timezone,
        updated_at_event_revision: revision,
    });
    apply_revision(event, revision, changed_at)
}

pub fn add_calendar_reminder_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    operation_id: [u8; STABLE_ID_BYTES_V1],
    due_at: CalendarTimestampV1,
    changed_at: CalendarTimestampV1,
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, changed_at)?;
    validate_timestamp(due_at)?;
    if due_at > event.starts_at {
        return Err(CalendarLifecycleErrorV1::InvalidReminder);
    }
    let reminder_id = derive_calendar_reminder_id_v1(&event.calendar_event_id, &operation_id)?;
    if event
        .reminders
        .iter()
        .any(|value| value.reminder_id == reminder_id)
    {
        return Err(CalendarLifecycleErrorV1::ReminderExists);
    }
    let revision = next_revision(event)?;
    event.reminders.push(CalendarReminderV1 {
        reminder_id,
        due_at,
        state: CalendarReminderStateV1::Pending,
        updated_at_event_revision: revision,
    });
    event.reminders.sort_by_key(|value| value.reminder_id);
    apply_revision(event, revision, changed_at)?;
    Ok(reminder_id)
}

pub fn remove_calendar_reminder_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    reminder_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, changed_at)?;
    let index = event
        .reminders
        .iter()
        .position(|value| value.reminder_id == reminder_id)
        .ok_or(CalendarLifecycleErrorV1::ReminderNotFound)?;
    if event.reminders[index].state != CalendarReminderStateV1::Pending {
        return Err(CalendarLifecycleErrorV1::ReminderNotPending);
    }
    let revision = next_revision(event)?;
    event.reminders[index].state = CalendarReminderStateV1::Cancelled;
    event.reminders[index].updated_at_event_revision = revision;
    apply_revision(event, revision, changed_at)
}

pub fn fire_calendar_reminder_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    reminder_id: [u8; STABLE_ID_BYTES_V1],
    fired_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_scheduled(event, expected_revision, fired_at)?;
    let index = event
        .reminders
        .iter()
        .position(|value| value.reminder_id == reminder_id)
        .ok_or(CalendarLifecycleErrorV1::ReminderNotFound)?;
    if event.reminders[index].state != CalendarReminderStateV1::Pending
        || fired_at < event.reminders[index].due_at
    {
        return Err(CalendarLifecycleErrorV1::ReminderNotPending);
    }
    let revision = next_revision(event)?;
    event.reminders[index].state = CalendarReminderStateV1::Fired;
    event.reminders[index].updated_at_event_revision = revision;
    apply_revision(event, revision, fired_at)
}

pub fn record_calendar_outcome_v1(
    event: &mut CalendarEventRecordV1,
    expected_revision: u64,
    operation_id: [u8; STABLE_ID_BYTES_V1],
    kind: CalendarOutcomeKindV1,
    note: String,
    recorded_at: CalendarTimestampV1,
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    require_revision_and_time(event, expected_revision, recorded_at)?;
    if event.state == CalendarEventStateV1::Scheduled {
        return Err(CalendarLifecycleErrorV1::InvalidOutcome);
    }
    validate_optional_text(&note, MAX_OUTCOME_NOTE_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidOutcome)?;
    let outcome_id = derive_calendar_outcome_id_v1(&event.calendar_event_id, &operation_id)?;
    if event
        .outcomes
        .iter()
        .any(|value| value.outcome_id == outcome_id)
    {
        return Err(CalendarLifecycleErrorV1::OutcomeExists);
    }
    let revision = next_revision(event)?;
    event.outcomes.push(CalendarOutcomeV1 {
        outcome_id,
        kind,
        note,
        recorded_at,
        recorded_at_event_revision: revision,
    });
    event.outcomes.sort_by_key(|value| value.outcome_id);
    apply_revision(event, revision, recorded_at)?;
    Ok(outcome_id)
}

pub fn validate_calendar_event_record_v1(
    event: &CalendarEventRecordV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    if !valid_owner(&event.logical_owner_id) {
        return Err(CalendarLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(&event.calendar_event_id) || event.event_revision == 0 {
        return Err(CalendarLifecycleErrorV1::InvalidEventId);
    }
    validate_text(&event.title, MAX_TITLE_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidTitle)?;
    validate_optional_text(&event.description, MAX_DESCRIPTION_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidDescription)?;
    validate_timezone(&event.timezone)?;
    validate_interval(event.starts_at, event.ends_at)?;
    validate_timestamp(event.created_at)?;
    validate_timestamp(event.updated_at)?;
    if event.updated_at < event.created_at {
        return Err(CalendarLifecycleErrorV1::InvalidTimestamp);
    }
    validate_sorted_unique(&event.participants, |value| value.participant_id)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidParticipant)?;
    if event
        .participants
        .iter()
        .filter(|value| value.role == CalendarParticipantRoleV1::Organizer)
        .count()
        > 1
    {
        return Err(CalendarLifecycleErrorV1::OrganizerExists);
    }
    for participant in &event.participants {
        validate_participant(&participant.display_name, &participant.address)?;
        if participant.updated_at_event_revision == 0
            || participant.updated_at_event_revision > event.event_revision
        {
            return Err(CalendarLifecycleErrorV1::InvalidParticipant);
        }
    }
    if let Some(constraints) = &event.constraints {
        validate_interval(constraints.earliest_start, constraints.latest_end)?;
        validate_timezone(&constraints.timezone)?;
        if constraints.minimum_duration_minutes == 0
            || constraints.updated_at_event_revision == 0
            || constraints.updated_at_event_revision > event.event_revision
        {
            return Err(CalendarLifecycleErrorV1::InvalidConstraints);
        }
    }
    validate_sorted_unique(&event.reminders, |value| value.reminder_id)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidReminder)?;
    for reminder in &event.reminders {
        validate_timestamp(reminder.due_at)?;
        if reminder.due_at > event.starts_at
            || reminder.updated_at_event_revision == 0
            || reminder.updated_at_event_revision > event.event_revision
        {
            return Err(CalendarLifecycleErrorV1::InvalidReminder);
        }
    }
    validate_sorted_unique(&event.outcomes, |value| value.outcome_id)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidOutcome)?;
    for outcome in &event.outcomes {
        validate_timestamp(outcome.recorded_at)?;
        validate_optional_text(&outcome.note, MAX_OUTCOME_NOTE_CHARS_V1)
            .map_err(|_| CalendarLifecycleErrorV1::InvalidOutcome)?;
        if outcome.recorded_at_event_revision == 0
            || outcome.recorded_at_event_revision > event.event_revision
        {
            return Err(CalendarLifecycleErrorV1::InvalidOutcome);
        }
    }
    Ok(())
}

fn require_scheduled(
    event: &CalendarEventRecordV1,
    expected_revision: u64,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    require_revision_and_time(event, expected_revision, changed_at)?;
    if event.state != CalendarEventStateV1::Scheduled {
        return Err(CalendarLifecycleErrorV1::InvalidStateTransition);
    }
    Ok(())
}

fn require_revision_and_time(
    event: &CalendarEventRecordV1,
    expected_revision: u64,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    validate_calendar_event_record_v1(event)?;
    if expected_revision == 0 || event.event_revision != expected_revision {
        return Err(CalendarLifecycleErrorV1::InvalidRevision);
    }
    validate_timestamp(changed_at)?;
    if changed_at < event.updated_at {
        return Err(CalendarLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn next_revision(event: &CalendarEventRecordV1) -> Result<u64, CalendarLifecycleErrorV1> {
    event
        .event_revision
        .checked_add(1)
        .ok_or(CalendarLifecycleErrorV1::RevisionOverflow)
}

fn advance(
    event: &mut CalendarEventRecordV1,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    let revision = next_revision(event)?;
    apply_revision(event, revision, changed_at)
}

fn apply_revision(
    event: &mut CalendarEventRecordV1,
    revision: u64,
    changed_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    event.event_revision = revision;
    event.updated_at = changed_at;
    validate_calendar_event_record_v1(event)
}

fn validate_interval(
    starts_at: CalendarTimestampV1,
    ends_at: CalendarTimestampV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    validate_timestamp(starts_at)?;
    validate_timestamp(ends_at)?;
    if starts_at >= ends_at {
        return Err(CalendarLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn validate_timestamp(value: CalendarTimestampV1) -> Result<(), CalendarLifecycleErrorV1> {
    timestamp_millis(value).map(|_| ())
}

fn timestamp_millis(value: CalendarTimestampV1) -> Result<i128, CalendarLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(CalendarLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(i128::from(value.unix_seconds) * 1_000 + i128::from(value.nanos) / 1_000_000)
}

fn validate_timezone(value: &str) -> Result<(), CalendarLifecycleErrorV1> {
    if value.is_empty()
        || value.chars().count() > MAX_TIMEZONE_CHARS_V1
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+'))
    {
        return Err(CalendarLifecycleErrorV1::InvalidTimezone);
    }
    Ok(())
}

fn validate_participant(display_name: &str, address: &str) -> Result<(), CalendarLifecycleErrorV1> {
    validate_text(display_name, MAX_DISPLAY_NAME_CHARS_V1)
        .map_err(|_| CalendarLifecycleErrorV1::InvalidParticipant)?;
    if address.trim() != address
        || address.is_empty()
        || address.chars().count() > MAX_ADDRESS_CHARS_V1
        || address.chars().any(char::is_control)
    {
        return Err(CalendarLifecycleErrorV1::InvalidParticipant);
    }
    Ok(())
}

fn require_organizer_slot(
    event: &CalendarEventRecordV1,
    except: Option<[u8; STABLE_ID_BYTES_V1]>,
    role: CalendarParticipantRoleV1,
) -> Result<(), CalendarLifecycleErrorV1> {
    if role == CalendarParticipantRoleV1::Organizer
        && event.participants.iter().any(|value| {
            value.role == CalendarParticipantRoleV1::Organizer
                && Some(value.participant_id) != except
        })
    {
        return Err(CalendarLifecycleErrorV1::OrganizerExists);
    }
    Ok(())
}

fn derive_child_id(
    domain: &[u8],
    calendar_event_id: &[u8; STABLE_ID_BYTES_V1],
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], CalendarLifecycleErrorV1> {
    if !nonzero(calendar_event_id) || !nonzero(operation_id) {
        return Err(CalendarLifecycleErrorV1::InvalidOperationId);
    }
    Ok(derive_id(domain, &[calendar_event_id, operation_id]))
}

fn derive_id(domain: &[u8], parts: &[&[u8]]) -> [u8; STABLE_ID_BYTES_V1] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .expect("fixed digest")
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LOGICAL_OWNER_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn validate_text(value: &str, max_chars: usize) -> Result<(), ()> {
    if value.trim().is_empty()
        || value.chars().count() > max_chars
        || value.chars().any(|character| character.is_control())
    {
        return Err(());
    }
    Ok(())
}

fn validate_optional_text(value: &str, max_chars: usize) -> Result<(), ()> {
    if value.chars().count() > max_chars
        || value
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
    {
        return Err(());
    }
    Ok(())
}

fn validate_sorted_unique<T>(
    values: &[T],
    key: impl Fn(&T) -> [u8; STABLE_ID_BYTES_V1],
) -> Result<(), ()> {
    if values.windows(2).any(|pair| key(&pair[0]) >= key(&pair[1])) {
        return Err(());
    }
    Ok(())
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: i64) -> CalendarTimestampV1 {
        CalendarTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }

    fn event() -> CalendarEventRecordV1 {
        create_calendar_event_v1(CalendarEventDraftV1 {
            operation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            title: "Design review".to_owned(),
            description: "Provider-neutral Calendar lifecycle".to_owned(),
            starts_at: at(10_000),
            ends_at: at(10_600),
            timezone: "Europe/Madrid".to_owned(),
            created_at: at(100),
        })
        .expect("event")
    }

    #[test]
    fn event_participant_constraint_and_reminder_lifecycle_is_checked() {
        let mut value = event();
        let organizer = add_calendar_participant_v1(
            &mut value,
            1,
            [2; 16],
            "Owner".to_owned(),
            "owner@example.test".to_owned(),
            CalendarParticipantRoleV1::Organizer,
            CalendarParticipantResponseV1::Accepted,
            at(101),
        )
        .expect("participant");
        assert_eq!(value.event_revision, 2);
        assert_eq!(
            add_calendar_participant_v1(
                &mut value,
                2,
                [3; 16],
                "Other organizer".to_owned(),
                "other@example.test".to_owned(),
                CalendarParticipantRoleV1::Organizer,
                CalendarParticipantResponseV1::Pending,
                at(102),
            ),
            Err(CalendarLifecycleErrorV1::OrganizerExists)
        );
        update_calendar_participant_v1(
            &mut value,
            2,
            organizer,
            None,
            None,
            None,
            Some(CalendarParticipantResponseV1::Tentative),
            at(102),
        )
        .expect("update participant");
        set_calendar_constraints_v1(
            &mut value,
            3,
            at(9_000),
            at(11_000),
            30,
            "Europe/Madrid".to_owned(),
            at(103),
        )
        .expect("constraints");
        let reminder =
            add_calendar_reminder_v1(&mut value, 4, [4; 16], at(9_900), at(104)).expect("reminder");
        fire_calendar_reminder_v1(&mut value, 5, reminder, at(9_900)).expect("fire");
        assert_eq!(value.reminders[0].state, CalendarReminderStateV1::Fired);
        assert_eq!(value.event_revision, 6);
    }

    #[test]
    fn terminal_outcomes_are_immutable_and_revisioned() {
        let mut value = event();
        set_calendar_event_state_v1(&mut value, 1, CalendarEventStateV1::Completed, at(10_600))
            .expect("complete");
        let outcome = record_calendar_outcome_v1(
            &mut value,
            2,
            [5; 16],
            CalendarOutcomeKindV1::Completed,
            "Reviewed".to_owned(),
            at(10_601),
        )
        .expect("outcome");
        assert_eq!(value.outcomes[0].outcome_id, outcome);
        assert_eq!(value.event_revision, 3);
        assert_eq!(
            record_calendar_outcome_v1(
                &mut value,
                3,
                [5; 16],
                CalendarOutcomeKindV1::Completed,
                "Changed".to_owned(),
                at(10_602),
            ),
            Err(CalendarLifecycleErrorV1::OutcomeExists)
        );
        assert_eq!(
            update_calendar_event_v1(
                &mut value,
                3,
                Some("Changed".to_owned()),
                None,
                None,
                None,
                None,
                at(10_602),
            ),
            Err(CalendarLifecycleErrorV1::InvalidStateTransition)
        );
    }

    #[test]
    fn stable_ids_and_overflow_fail_closed() {
        let first = event();
        let second = event();
        assert_eq!(first.calendar_event_id, second.calendar_event_id);
        let mut overflow = first;
        overflow.event_revision = u64::MAX;
        assert_eq!(
            set_calendar_event_state_v1(
                &mut overflow,
                u64::MAX,
                CalendarEventStateV1::Cancelled,
                at(101),
            ),
            Err(CalendarLifecycleErrorV1::RevisionOverflow)
        );
    }
}
