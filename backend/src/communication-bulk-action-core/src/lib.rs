#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub const PACKAGE: &str = "makosh-communication-bulk-action-core";
pub const MAX_BULK_TARGETS_V1: usize = 100;
pub const MAX_TARGET_BODY_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkDeliveryTargetDraftV1 {
    pub operation_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub reply_to_message_id: Option<[u8; 16]>,
    pub body_utf8: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkDeliveryDraftV1 {
    pub batch_id: [u8; 16],
    pub targets: Vec<BulkDeliveryTargetDraftV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDeliveryValidationErrorV1 {
    InvalidBatchId,
    InvalidTargetCount,
    InvalidTargetId,
    DuplicateTargetId,
    InvalidConversationId,
    InvalidReplyId,
    InvalidBody,
    BodyLimitExceeded,
}

pub fn validate_bulk_delivery_v1(
    draft: BulkDeliveryDraftV1,
) -> Result<BulkDeliveryDraftV1, BulkDeliveryValidationErrorV1> {
    if zero_id(&draft.batch_id) {
        return Err(BulkDeliveryValidationErrorV1::InvalidBatchId);
    }
    if draft.targets.is_empty() || draft.targets.len() > MAX_BULK_TARGETS_V1 {
        return Err(BulkDeliveryValidationErrorV1::InvalidTargetCount);
    }
    let mut operation_ids = BTreeSet::new();
    for target in &draft.targets {
        if zero_id(&target.operation_id) {
            return Err(BulkDeliveryValidationErrorV1::InvalidTargetId);
        }
        if !operation_ids.insert(target.operation_id) {
            return Err(BulkDeliveryValidationErrorV1::DuplicateTargetId);
        }
        if zero_id(&target.conversation_id) {
            return Err(BulkDeliveryValidationErrorV1::InvalidConversationId);
        }
        if target.reply_to_message_id.as_ref().is_some_and(zero_id) {
            return Err(BulkDeliveryValidationErrorV1::InvalidReplyId);
        }
        if target.body_utf8.len() > MAX_TARGET_BODY_BYTES_V1 {
            return Err(BulkDeliveryValidationErrorV1::BodyLimitExceeded);
        }
        let body = std::str::from_utf8(&target.body_utf8)
            .map_err(|_| BulkDeliveryValidationErrorV1::InvalidBody)?;
        if body.trim().is_empty() {
            return Err(BulkDeliveryValidationErrorV1::InvalidBody);
        }
    }
    Ok(draft)
}

fn zero_id(value: &[u8; 16]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: u8) -> BulkDeliveryTargetDraftV1 {
        BulkDeliveryTargetDraftV1 {
            operation_id: [id; 16],
            conversation_id: [2; 16],
            reply_to_message_id: None,
            body_utf8: b"private body".to_vec(),
        }
    }

    #[test]
    fn accepts_a_bounded_batch_without_provider_coordinates() {
        let draft = BulkDeliveryDraftV1 {
            batch_id: [9; 16],
            targets: vec![target(1), target(2)],
        };
        assert_eq!(validate_bulk_delivery_v1(draft.clone()), Ok(draft));
    }

    #[test]
    fn rejects_duplicate_target_operation_ids() {
        let error = validate_bulk_delivery_v1(BulkDeliveryDraftV1 {
            batch_id: [9; 16],
            targets: vec![target(1), target(1)],
        })
        .expect_err("duplicate operation ID");
        assert_eq!(error, BulkDeliveryValidationErrorV1::DuplicateTargetId);
    }

    #[test]
    fn rejects_oversized_or_empty_batches_and_bodies() {
        assert_eq!(
            validate_bulk_delivery_v1(BulkDeliveryDraftV1 {
                batch_id: [9; 16],
                targets: Vec::new(),
            }),
            Err(BulkDeliveryValidationErrorV1::InvalidTargetCount)
        );
        let mut oversized = target(1);
        oversized.body_utf8 = vec![b'x'; MAX_TARGET_BODY_BYTES_V1 + 1];
        assert_eq!(
            validate_bulk_delivery_v1(BulkDeliveryDraftV1 {
                batch_id: [9; 16],
                targets: vec![oversized],
            }),
            Err(BulkDeliveryValidationErrorV1::BodyLimitExceeded)
        );
    }
}
