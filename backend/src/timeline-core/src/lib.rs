#![forbid(unsafe_code)]
pub const PACKAGE: &str = "makosh-timeline-core";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineProjectionEntryV1 {
    pub event_id: [u8; 16],
    pub logical_owner_id: String,
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
    pub source_revision: u64,
    pub lifecycle_state: String,
    pub occurred_at_unix_millis: i64,
    pub tombstone: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineCoreErrorV1 {
    InvalidOwner,
    InvalidKind,
    InvalidId,
    InvalidRevision,
    InvalidTime,
    InvalidState,
}
pub fn validate_timeline_projection_entry_v1(
    value: &TimelineProjectionEntryV1,
) -> Result<(), TimelineCoreErrorV1> {
    atom(&value.logical_owner_id).map_err(|_| TimelineCoreErrorV1::InvalidOwner)?;
    atom(&value.source_owner).map_err(|_| TimelineCoreErrorV1::InvalidOwner)?;
    atom(&value.entity_kind).map_err(|_| TimelineCoreErrorV1::InvalidKind)?;
    if value.event_id.iter().all(|byte| *byte == 0) || value.entity_id.iter().all(|byte| *byte == 0)
    {
        return Err(TimelineCoreErrorV1::InvalidId);
    }
    if value.source_revision == 0 {
        return Err(TimelineCoreErrorV1::InvalidRevision);
    }
    if value.occurred_at_unix_millis <= 0 {
        return Err(TimelineCoreErrorV1::InvalidTime);
    }
    if value.tombstone {
        if !value.lifecycle_state.is_empty() {
            return Err(TimelineCoreErrorV1::InvalidState);
        }
    } else {
        atom(&value.lifecycle_state).map_err(|_| TimelineCoreErrorV1::InvalidState)?;
    }
    Ok(())
}
fn atom(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(())
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entry_is_structural_and_tombstone_is_explicit() {
        let mut value = TimelineProjectionEntryV1 {
            event_id: [1; 16],
            logical_owner_id: "owner-1".into(),
            source_owner: "tasks".into(),
            entity_kind: "task".into(),
            entity_id: [2; 16],
            source_revision: 2,
            lifecycle_state: "task_state_active".into(),
            occurred_at_unix_millis: 1000,
            tombstone: false,
        };
        assert_eq!(validate_timeline_projection_entry_v1(&value), Ok(()));
        value.tombstone = true;
        assert_eq!(
            validate_timeline_projection_entry_v1(&value),
            Err(TimelineCoreErrorV1::InvalidState)
        );
        value.lifecycle_state.clear();
        assert_eq!(validate_timeline_projection_entry_v1(&value), Ok(()));
    }
}
