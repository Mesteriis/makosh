use makosh_calendar_core::{
    CalendarEventDraftV1, CalendarEventStateV1, CalendarOutcomeKindV1,
    CalendarParticipantResponseV1, CalendarParticipantRoleV1, CalendarTimestampV1,
};
use sha2::{Digest, Sha256};

pub const CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub semantic_kind: i16,
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarLifecycleMutationV1 {
    Create(CalendarEventDraftV1),
    Update {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        title: Option<String>,
        description: Option<String>,
        starts_at: Option<CalendarTimestampV1>,
        ends_at: Option<CalendarTimestampV1>,
        timezone: Option<String>,
        changed_at: CalendarTimestampV1,
    },
    SetState {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        state: CalendarEventStateV1,
        changed_at: CalendarTimestampV1,
    },
    AddParticipant {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        display_name: String,
        address: String,
        role: CalendarParticipantRoleV1,
        response: CalendarParticipantResponseV1,
        changed_at: CalendarTimestampV1,
    },
    UpdateParticipant {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        participant_id: [u8; 16],
        display_name: Option<String>,
        address: Option<String>,
        role: Option<CalendarParticipantRoleV1>,
        response: Option<CalendarParticipantResponseV1>,
        changed_at: CalendarTimestampV1,
    },
    RemoveParticipant {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        participant_id: [u8; 16],
        changed_at: CalendarTimestampV1,
    },
    SetConstraints {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        earliest_start: CalendarTimestampV1,
        latest_end: CalendarTimestampV1,
        minimum_duration_minutes: u32,
        timezone: String,
        changed_at: CalendarTimestampV1,
    },
    AddReminder {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        due_at: CalendarTimestampV1,
        changed_at: CalendarTimestampV1,
    },
    RemoveReminder {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        reminder_id: [u8; 16],
        changed_at: CalendarTimestampV1,
    },
    RecordOutcome {
        operation_id: [u8; 16],
        calendar_event_id: [u8; 16],
        expected_revision: u64,
        kind: CalendarOutcomeKindV1,
        note: String,
        recorded_at: CalendarTimestampV1,
    },
}

impl CalendarLifecycleMutationV1 {
    #[must_use]
    pub fn operation_id(&self) -> [u8; 16] {
        match self {
            Self::Create(value) => value.operation_id,
            Self::Update { operation_id, .. }
            | Self::SetState { operation_id, .. }
            | Self::AddParticipant { operation_id, .. }
            | Self::UpdateParticipant { operation_id, .. }
            | Self::RemoveParticipant { operation_id, .. }
            | Self::SetConstraints { operation_id, .. }
            | Self::AddReminder { operation_id, .. }
            | Self::RemoveReminder { operation_id, .. }
            | Self::RecordOutcome { operation_id, .. } => *operation_id,
        }
    }

    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create(_) => 1,
            Self::Update { .. } => 2,
            Self::SetState { .. } => 3,
            Self::AddParticipant { .. } => 4,
            Self::UpdateParticipant { .. } => 5,
            Self::RemoveParticipant { .. } => 6,
            Self::SetConstraints { .. } => 7,
            Self::AddReminder { .. } => 8,
            Self::RemoveReminder { .. } => 9,
            Self::RecordOutcome { .. } => 10,
        }
    }

    #[must_use]
    pub fn calendar_event_id(&self) -> Option<[u8; 16]> {
        match self {
            Self::Create(_) => None,
            Self::Update {
                calendar_event_id, ..
            }
            | Self::SetState {
                calendar_event_id, ..
            }
            | Self::AddParticipant {
                calendar_event_id, ..
            }
            | Self::UpdateParticipant {
                calendar_event_id, ..
            }
            | Self::RemoveParticipant {
                calendar_event_id, ..
            }
            | Self::SetConstraints {
                calendar_event_id, ..
            }
            | Self::AddReminder {
                calendar_event_id, ..
            }
            | Self::RemoveReminder {
                calendar_event_id, ..
            }
            | Self::RecordOutcome {
                calendar_event_id, ..
            } => Some(*calendar_event_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: CalendarLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub outbox: Vec<CalendarOutboxRecordV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarLifecycleOperationOutcomeV1 {
    Applied {
        event: Box<makosh_calendar_core::CalendarEventRecordV1>,
        response_bytes: Vec<u8>,
    },
    Replayed {
        response_bytes: Vec<u8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarSchedulerInputV1 {
    pub logical_owner_id: String,
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
    pub operation_kind: i16,
    pub reminder_id: [u8; 16],
    pub expected_command_message_id: Option<[u8; 16]>,
    pub lease_expires_at_unix_millis: Option<i64>,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarSchedulerCommitV1 {
    pub outbox: Vec<CalendarOutboxRecordV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarSchedulerInputOutcomeV1 {
    Applied,
    Replayed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    NotFound,
    OperationConflict,
    RevisionConflict,
    OutboxConflict,
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_operation(value: &CalendarLifecycleOperationV1) -> bool {
    value.operation_id == value.mutation.operation_id()
        && valid_owner(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.request_sha256)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.request_bytes).as_slice() == value.request_sha256
        && value.received_at_unix_millis > 0
}

pub(crate) fn valid_commit(value: &CalendarLifecycleCommitV1) -> bool {
    nonzero(&value.response_sha256)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.response_bytes).as_slice() == value.response_sha256
        && !value.outbox.is_empty()
        && value.outbox.len() <= 4
        && value.outbox.iter().all(|record| {
            nonzero(&record.message_id)
                && (1..=4).contains(&record.semantic_kind)
                && nonzero(&record.envelope_sha256)
                && !record.envelope_bytes.is_empty()
                && record.envelope_bytes.len() <= CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1
                && Sha256::digest(&record.envelope_bytes).as_slice() == record.envelope_sha256
        })
}

pub(crate) fn valid_scheduler_input(value: &CalendarSchedulerInputV1) -> bool {
    valid_owner(&value.logical_owner_id)
        && nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
        && (1..=3).contains(&value.operation_kind)
        && nonzero(&value.reminder_id)
        && value
            .expected_command_message_id
            .is_none_or(|value| nonzero(&value))
        && value
            .lease_expires_at_unix_millis
            .is_none_or(|value| value > 0)
        && value.completed_at_unix_millis > 0
}

pub(crate) fn valid_scheduler_commit(value: &CalendarSchedulerCommitV1) -> bool {
    !value.outbox.is_empty()
        && value.outbox.len() <= 4
        && value.outbox.iter().all(|record| {
            nonzero(&record.message_id)
                && (1..=4).contains(&record.semantic_kind)
                && nonzero(&record.envelope_sha256)
                && !record.envelope_bytes.is_empty()
                && record.envelope_bytes.len() <= CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1
                && Sha256::digest(&record.envelope_bytes).as_slice() == record.envelope_sha256
        })
}

pub(crate) fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_request_and_outbox_hashes_are_required() {
        let operation = CalendarLifecycleOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            request_sha256: Sha256::digest(b"request").into(),
            request_bytes: b"request".to_vec(),
            received_at_unix_millis: 1,
            mutation: CalendarLifecycleMutationV1::Create(CalendarEventDraftV1 {
                operation_id: [1; 16],
                logical_owner_id: "owner-1".to_owned(),
                title: "Calendar".to_owned(),
                description: String::new(),
                starts_at: CalendarTimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                },
                ends_at: CalendarTimestampV1 {
                    unix_seconds: 20,
                    nanos: 0,
                },
                timezone: "UTC".to_owned(),
                created_at: CalendarTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            }),
        };
        assert!(valid_operation(&operation));
        let mut changed = operation;
        changed.request_bytes.push(0);
        assert!(!valid_operation(&changed));
    }
}
