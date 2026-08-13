#![forbid(unsafe_code)]
pub const PACKAGE: &str = "makosh-risk-core";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskProjectionEntryV1 {
    pub event_id: [u8; 16],
    pub logical_owner_id: String,
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
    pub source_revision: u64,
    pub reason_code: String,
    pub severity: u32,
    pub occurred_at_unix_millis: i64,
    pub expires_at_unix_millis: i64,
    pub cleared: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskCoreErrorV1 {
    InvalidOwner,
    InvalidKind,
    InvalidId,
    InvalidRevision,
    InvalidTime,
    InvalidState,
}
pub fn validate_risk_projection_entry_v1(
    value: &RiskProjectionEntryV1,
) -> Result<(), RiskCoreErrorV1> {
    atom(&value.logical_owner_id).map_err(|_| RiskCoreErrorV1::InvalidOwner)?;
    atom(&value.source_owner).map_err(|_| RiskCoreErrorV1::InvalidOwner)?;
    atom(&value.entity_kind).map_err(|_| RiskCoreErrorV1::InvalidKind)?;
    if value.event_id.iter().all(|byte| *byte == 0) || value.entity_id.iter().all(|byte| *byte == 0)
    {
        return Err(RiskCoreErrorV1::InvalidId);
    }
    if value.source_revision == 0 {
        return Err(RiskCoreErrorV1::InvalidRevision);
    }
    if value.occurred_at_unix_millis <= 0 {
        return Err(RiskCoreErrorV1::InvalidTime);
    }
    if value.cleared {
        if !value.reason_code.is_empty() || value.severity != 0 || value.expires_at_unix_millis != 0
        {
            return Err(RiskCoreErrorV1::InvalidState);
        }
    } else {
        atom(&value.reason_code).map_err(|_| RiskCoreErrorV1::InvalidState)?;
        if !(1..=5).contains(&value.severity)
            || value.expires_at_unix_millis <= value.occurred_at_unix_millis
        {
            return Err(RiskCoreErrorV1::InvalidState);
        }
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
    fn entry_is_structural_and_cleared_is_explicit() {
        let mut value = RiskProjectionEntryV1 {
            event_id: [1; 16],
            logical_owner_id: "owner-1".into(),
            source_owner: "tasks".into(),
            entity_kind: "task".into(),
            entity_id: [2; 16],
            source_revision: 2,
            reason_code: "open_task".into(),
            severity: 1,
            occurred_at_unix_millis: 1000,
            expires_at_unix_millis: 2000,
            cleared: false,
        };
        assert_eq!(validate_risk_projection_entry_v1(&value), Ok(()));
        value.cleared = true;
        assert_eq!(
            validate_risk_projection_entry_v1(&value),
            Err(RiskCoreErrorV1::InvalidState)
        );
        value.reason_code.clear();
        value.severity = 0;
        value.expires_at_unix_millis = 0;
        assert_eq!(validate_risk_projection_entry_v1(&value), Ok(()));
    }
}
