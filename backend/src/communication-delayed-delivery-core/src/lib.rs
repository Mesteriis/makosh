#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-core";
pub const MIN_DELIVERY_DELAY_MILLIS_V1: u64 = 5_000;
pub const MAX_DELIVERY_DELAY_MILLIS_V1: u64 = 366 * 24 * 60 * 60 * 1_000;
pub const MAX_DELIVERY_BODY_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryDraftV1 {
    pub delayed_operation_id: [u8; 16],
    pub delivery_operation_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub reply_to_message_id: Option<[u8; 16]>,
    pub body_utf8: Vec<u8>,
    pub deliver_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryOperationV1 {
    delayed_operation_id: [u8; 16],
    delivery_operation_id: [u8; 16],
    conversation_id: [u8; 16],
    reply_to_message_id: Option<[u8; 16]>,
    deliver_at_unix_millis: u64,
}

impl DelayedDeliveryOperationV1 {
    #[must_use]
    pub const fn delayed_operation_id(&self) -> &[u8; 16] {
        &self.delayed_operation_id
    }

    #[must_use]
    pub const fn delivery_operation_id(&self) -> &[u8; 16] {
        &self.delivery_operation_id
    }

    #[must_use]
    pub const fn conversation_id(&self) -> &[u8; 16] {
        &self.conversation_id
    }

    #[must_use]
    pub const fn reply_to_message_id(&self) -> Option<&[u8; 16]> {
        self.reply_to_message_id.as_ref()
    }

    #[must_use]
    pub const fn deliver_at_unix_millis(&self) -> u64 {
        self.deliver_at_unix_millis
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryStateV1 {
    Accepted,
    SchedulePending,
    Scheduled,
    Due,
    Dispatching,
    DeliveryAccepted,
    CancelRequested,
    Cancelled,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryLifecycleV1 {
    pub state: DelayedDeliveryStateV1,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerCancelOutcomeV1 {
    Cancelled,
    TooLate,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryPolicyErrorV1 {
    InvalidDelayedOperationId,
    InvalidDeliveryOperationId,
    InvalidConversationId,
    InvalidReplyId,
    InvalidBody,
    BodyLimitExceeded,
    DueTimeTooSoon,
    DueTimeTooFar,
    StaleRevision,
    InvalidTransition,
}

pub fn validate_delayed_delivery_v1(
    draft: DelayedDeliveryDraftV1,
    authoritative_now_unix_millis: u64,
) -> Result<DelayedDeliveryDraftV1, DelayedDeliveryPolicyErrorV1> {
    if zero_id(&draft.delayed_operation_id) {
        return Err(DelayedDeliveryPolicyErrorV1::InvalidDelayedOperationId);
    }
    if zero_id(&draft.delivery_operation_id) {
        return Err(DelayedDeliveryPolicyErrorV1::InvalidDeliveryOperationId);
    }
    if zero_id(&draft.conversation_id) {
        return Err(DelayedDeliveryPolicyErrorV1::InvalidConversationId);
    }
    if draft.reply_to_message_id.as_ref().is_some_and(zero_id) {
        return Err(DelayedDeliveryPolicyErrorV1::InvalidReplyId);
    }
    if draft.body_utf8.len() > MAX_DELIVERY_BODY_BYTES_V1 {
        return Err(DelayedDeliveryPolicyErrorV1::BodyLimitExceeded);
    }
    let body = std::str::from_utf8(&draft.body_utf8)
        .map_err(|_| DelayedDeliveryPolicyErrorV1::InvalidBody)?;
    if body.trim().is_empty() {
        return Err(DelayedDeliveryPolicyErrorV1::InvalidBody);
    }
    let earliest = authoritative_now_unix_millis.saturating_add(MIN_DELIVERY_DELAY_MILLIS_V1);
    if draft.deliver_at_unix_millis < earliest {
        return Err(DelayedDeliveryPolicyErrorV1::DueTimeTooSoon);
    }
    let latest = authoritative_now_unix_millis.saturating_add(MAX_DELIVERY_DELAY_MILLIS_V1);
    if draft.deliver_at_unix_millis > latest {
        return Err(DelayedDeliveryPolicyErrorV1::DueTimeTooFar);
    }
    Ok(draft)
}

pub fn prepare_delayed_delivery_v1(
    draft: DelayedDeliveryDraftV1,
    authoritative_now_unix_millis: u64,
) -> Result<DelayedDeliveryOperationV1, DelayedDeliveryPolicyErrorV1> {
    let validated = validate_delayed_delivery_v1(draft, authoritative_now_unix_millis)?;
    Ok(DelayedDeliveryOperationV1 {
        delayed_operation_id: validated.delayed_operation_id,
        delivery_operation_id: validated.delivery_operation_id,
        conversation_id: validated.conversation_id,
        reply_to_message_id: validated.reply_to_message_id,
        deliver_at_unix_millis: validated.deliver_at_unix_millis,
    })
}

pub fn request_cancellation_v1(
    lifecycle: DelayedDeliveryLifecycleV1,
    expected_revision: u64,
) -> Result<DelayedDeliveryLifecycleV1, DelayedDeliveryPolicyErrorV1> {
    if lifecycle.revision != expected_revision {
        return Err(DelayedDeliveryPolicyErrorV1::StaleRevision);
    }
    match lifecycle.state {
        DelayedDeliveryStateV1::Accepted
        | DelayedDeliveryStateV1::SchedulePending
        | DelayedDeliveryStateV1::Scheduled => {
            transition(lifecycle, DelayedDeliveryStateV1::CancelRequested)
        }
        DelayedDeliveryStateV1::CancelRequested => Ok(lifecycle),
        _ => Err(DelayedDeliveryPolicyErrorV1::InvalidTransition),
    }
}

pub fn apply_scheduler_cancel_result_v1(
    lifecycle: DelayedDeliveryLifecycleV1,
    outcome: SchedulerCancelOutcomeV1,
) -> Result<DelayedDeliveryLifecycleV1, DelayedDeliveryPolicyErrorV1> {
    if lifecycle.state != DelayedDeliveryStateV1::CancelRequested {
        return Err(DelayedDeliveryPolicyErrorV1::InvalidTransition);
    }
    let next = match outcome {
        SchedulerCancelOutcomeV1::Cancelled => DelayedDeliveryStateV1::Cancelled,
        SchedulerCancelOutcomeV1::TooLate => DelayedDeliveryStateV1::Scheduled,
        SchedulerCancelOutcomeV1::Rejected => DelayedDeliveryStateV1::Failed,
    };
    transition(lifecycle, next)
}

fn transition(
    lifecycle: DelayedDeliveryLifecycleV1,
    state: DelayedDeliveryStateV1,
) -> Result<DelayedDeliveryLifecycleV1, DelayedDeliveryPolicyErrorV1> {
    Ok(DelayedDeliveryLifecycleV1 {
        state,
        revision: lifecycle
            .revision
            .checked_add(1)
            .ok_or(DelayedDeliveryPolicyErrorV1::InvalidTransition)?,
    })
}

fn zero_id(value: &[u8; 16]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(deliver_at_unix_millis: u64) -> DelayedDeliveryDraftV1 {
        DelayedDeliveryDraftV1 {
            delayed_operation_id: [1; 16],
            delivery_operation_id: [2; 16],
            conversation_id: [3; 16],
            reply_to_message_id: Some([4; 16]),
            body_utf8: b"private body".to_vec(),
            deliver_at_unix_millis,
        }
    }

    #[test]
    fn validates_exact_kernel_clock_bounds() {
        let now = 1_000_000;
        let earliest = now + MIN_DELIVERY_DELAY_MILLIS_V1;
        assert_eq!(
            validate_delayed_delivery_v1(draft(earliest), now),
            Ok(draft(earliest))
        );
        let prepared = prepare_delayed_delivery_v1(draft(earliest), now).expect("valid operation");
        assert_eq!(prepared.delayed_operation_id(), &[1; 16]);
        assert_eq!(prepared.delivery_operation_id(), &[2; 16]);
        assert_eq!(
            validate_delayed_delivery_v1(draft(earliest - 1), now),
            Err(DelayedDeliveryPolicyErrorV1::DueTimeTooSoon)
        );
        assert_eq!(
            validate_delayed_delivery_v1(draft(now + MAX_DELIVERY_DELAY_MILLIS_V1 + 1), now,),
            Err(DelayedDeliveryPolicyErrorV1::DueTimeTooFar)
        );
    }

    #[test]
    fn rejects_invalid_identity_and_private_body() {
        let now = 1_000_000;
        let mut invalid = draft(now + MIN_DELIVERY_DELAY_MILLIS_V1);
        invalid.delayed_operation_id = [0; 16];
        assert_eq!(
            validate_delayed_delivery_v1(invalid, now),
            Err(DelayedDeliveryPolicyErrorV1::InvalidDelayedOperationId)
        );
        let mut oversized = draft(now + MIN_DELIVERY_DELAY_MILLIS_V1);
        oversized.body_utf8 = vec![b'x'; MAX_DELIVERY_BODY_BYTES_V1 + 1];
        assert_eq!(
            validate_delayed_delivery_v1(oversized, now),
            Err(DelayedDeliveryPolicyErrorV1::BodyLimitExceeded)
        );
    }

    #[test]
    fn scheduler_owns_the_cancellation_race() {
        let scheduled = DelayedDeliveryLifecycleV1 {
            state: DelayedDeliveryStateV1::Scheduled,
            revision: 7,
        };
        let requested = request_cancellation_v1(scheduled, 7).expect("current revision");
        assert_eq!(requested.state, DelayedDeliveryStateV1::CancelRequested);
        assert_eq!(
            apply_scheduler_cancel_result_v1(requested, SchedulerCancelOutcomeV1::TooLate),
            Ok(DelayedDeliveryLifecycleV1 {
                state: DelayedDeliveryStateV1::Scheduled,
                revision: 9,
            })
        );
        assert_eq!(
            request_cancellation_v1(scheduled, 6),
            Err(DelayedDeliveryPolicyErrorV1::StaleRevision)
        );
    }
}
