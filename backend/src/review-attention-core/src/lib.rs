#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-review-attention-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const MAX_SNOOZE_SECONDS_V1: i64 = 366 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewDispositionV1 {
    Pending,
    Reviewed,
    Dismissed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewImportanceV1 {
    Normal,
    Important,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttentionV1 {
    pub attention_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub revision: u64,
    pub disposition: ReviewDispositionV1,
    pub pinned: bool,
    pub importance: ReviewImportanceV1,
    pub snoozed_until: Option<ReviewTimestampV1>,
    pub updated_at: ReviewTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewAttentionCommandV1 {
    MarkPending,
    MarkReviewed,
    Dismiss,
    SetPinned(bool),
    SetImportance(ReviewImportanceV1),
    SnoozeUntil(ReviewTimestampV1),
    ClearSnooze,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyReviewAttentionV1 {
    pub logical_owner_id: String,
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub expected_revision: u64,
    pub command: ReviewAttentionCommandV1,
    pub applied_at: ReviewTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttentionOutcomeV1 {
    pub attention: ReviewAttentionV1,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewAttentionErrorV1 {
    InvalidOwner,
    InvalidSourceEvidence,
    InvalidTimestamp,
    RevisionConflict,
    DismissedAttention,
    InvalidSnoozeDeadline,
}

pub fn apply_review_attention_v1(
    current: Option<&ReviewAttentionV1>,
    request: &ApplyReviewAttentionV1,
) -> Result<ReviewAttentionOutcomeV1, ReviewAttentionErrorV1> {
    validate_request(request)?;
    let attention_id =
        derive_attention_id_v1(&request.logical_owner_id, &request.source_evidence_id)?;
    let mut attention = match current {
        Some(current) => {
            if current.attention_id != attention_id
                || current.source_evidence_id != request.source_evidence_id
            {
                return Err(ReviewAttentionErrorV1::InvalidSourceEvidence);
            }
            if current.revision != request.expected_revision {
                return Err(ReviewAttentionErrorV1::RevisionConflict);
            }
            current.clone()
        }
        None => {
            if request.expected_revision != 0 {
                return Err(ReviewAttentionErrorV1::RevisionConflict);
            }
            ReviewAttentionV1 {
                attention_id,
                source_evidence_id: request.source_evidence_id,
                revision: 0,
                disposition: ReviewDispositionV1::Pending,
                pinned: false,
                importance: ReviewImportanceV1::Normal,
                snoozed_until: None,
                updated_at: request.applied_at,
            }
        }
    };
    let before = attention.clone();
    apply_command(&mut attention, request.command, request.applied_at)?;
    let changed = attention.disposition != before.disposition
        || attention.pinned != before.pinned
        || attention.importance != before.importance
        || attention.snoozed_until != before.snoozed_until
        || current.is_none();
    if changed {
        attention.revision = before
            .revision
            .checked_add(1)
            .ok_or(ReviewAttentionErrorV1::RevisionConflict)?;
        attention.updated_at = request.applied_at;
    }
    Ok(ReviewAttentionOutcomeV1 { attention, changed })
}

pub fn derive_attention_id_v1(
    logical_owner_id: &str,
    source_evidence_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], ReviewAttentionErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(ReviewAttentionErrorV1::InvalidOwner);
    }
    if source_evidence_id.iter().all(|byte| *byte == 0) {
        return Err(ReviewAttentionErrorV1::InvalidSourceEvidence);
    }
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.communication-attention.v1");
    digest.update([0]);
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(source_evidence_id);
    let digest = digest.finalize();
    let mut attention_id = [0_u8; STABLE_ID_BYTES_V1];
    attention_id.copy_from_slice(&digest[..STABLE_ID_BYTES_V1]);
    Ok(attention_id)
}

fn validate_request(request: &ApplyReviewAttentionV1) -> Result<(), ReviewAttentionErrorV1> {
    derive_attention_id_v1(&request.logical_owner_id, &request.source_evidence_id)?;
    valid_timestamp(request.applied_at)
        .then_some(())
        .ok_or(ReviewAttentionErrorV1::InvalidTimestamp)?;
    Ok(())
}

fn apply_command(
    attention: &mut ReviewAttentionV1,
    command: ReviewAttentionCommandV1,
    applied_at: ReviewTimestampV1,
) -> Result<(), ReviewAttentionErrorV1> {
    if attention.disposition == ReviewDispositionV1::Dismissed
        && !matches!(command, ReviewAttentionCommandV1::MarkPending)
    {
        return Err(ReviewAttentionErrorV1::DismissedAttention);
    }
    match command {
        ReviewAttentionCommandV1::MarkPending => {
            attention.disposition = ReviewDispositionV1::Pending;
        }
        ReviewAttentionCommandV1::MarkReviewed => {
            attention.disposition = ReviewDispositionV1::Reviewed;
            attention.snoozed_until = None;
        }
        ReviewAttentionCommandV1::Dismiss => {
            attention.disposition = ReviewDispositionV1::Dismissed;
            attention.pinned = false;
            attention.snoozed_until = None;
        }
        ReviewAttentionCommandV1::SetPinned(pinned) => {
            attention.pinned = pinned;
        }
        ReviewAttentionCommandV1::SetImportance(importance) => {
            attention.importance = importance;
        }
        ReviewAttentionCommandV1::SnoozeUntil(until) => {
            if !valid_timestamp(until)
                || until.unix_seconds <= applied_at.unix_seconds
                || until.unix_seconds - applied_at.unix_seconds > MAX_SNOOZE_SECONDS_V1
            {
                return Err(ReviewAttentionErrorV1::InvalidSnoozeDeadline);
            }
            attention.snoozed_until = Some(until);
        }
        ReviewAttentionCommandV1::ClearSnooze => {
            attention.snoozed_until = None;
        }
    }
    Ok(())
}

fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

const fn valid_timestamp(timestamp: ReviewTimestampV1) -> bool {
    timestamp.unix_seconds > 0 && timestamp.nanos >= 0 && timestamp.nanos < 1_000_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        command: ReviewAttentionCommandV1,
        expected_revision: u64,
    ) -> ApplyReviewAttentionV1 {
        ApplyReviewAttentionV1 {
            logical_owner_id: "owner-1".to_owned(),
            source_evidence_id: [7; STABLE_ID_BYTES_V1],
            expected_revision,
            command,
            applied_at: ReviewTimestampV1 {
                unix_seconds: 1_783_100_000 + i64::try_from(expected_revision).expect("revision"),
                nanos: 1,
            },
        }
    }

    #[test]
    fn first_action_creates_one_owner_scoped_attention_record() {
        let outcome =
            apply_review_attention_v1(None, &request(ReviewAttentionCommandV1::SetPinned(true), 0))
                .expect("create attention");
        assert!(outcome.changed);
        assert_eq!(outcome.attention.revision, 1);
        assert!(outcome.attention.pinned);
        assert_eq!(outcome.attention.disposition, ReviewDispositionV1::Pending);
        assert_eq!(
            outcome.attention.attention_id,
            derive_attention_id_v1("owner-1", &[7; STABLE_ID_BYTES_V1]).expect("attention id")
        );
    }

    #[test]
    fn no_op_preserves_revision_and_update_time() {
        let current =
            apply_review_attention_v1(None, &request(ReviewAttentionCommandV1::SetPinned(true), 0))
                .expect("create")
                .attention;
        let outcome = apply_review_attention_v1(
            Some(&current),
            &request(ReviewAttentionCommandV1::SetPinned(true), 1),
        )
        .expect("replay semantic no-op");
        assert!(!outcome.changed);
        assert_eq!(outcome.attention, current);
    }

    #[test]
    fn dismissal_clears_pin_and_snooze_and_requires_explicit_restore() {
        let snoozed = apply_review_attention_v1(
            None,
            &request(
                ReviewAttentionCommandV1::SnoozeUntil(ReviewTimestampV1 {
                    unix_seconds: 1_783_100_100,
                    nanos: 0,
                }),
                0,
            ),
        )
        .expect("snooze")
        .attention;
        let dismissed = apply_review_attention_v1(
            Some(&snoozed),
            &request(ReviewAttentionCommandV1::Dismiss, 1),
        )
        .expect("dismiss")
        .attention;
        assert_eq!(dismissed.disposition, ReviewDispositionV1::Dismissed);
        assert_eq!(dismissed.snoozed_until, None);
        assert_eq!(
            apply_review_attention_v1(
                Some(&dismissed),
                &request(ReviewAttentionCommandV1::SetPinned(true), 2),
            ),
            Err(ReviewAttentionErrorV1::DismissedAttention)
        );
        let restored = apply_review_attention_v1(
            Some(&dismissed),
            &request(ReviewAttentionCommandV1::MarkPending, 2),
        )
        .expect("restore")
        .attention;
        assert_eq!(restored.disposition, ReviewDispositionV1::Pending);
    }

    #[test]
    fn revision_source_and_owner_conflicts_fail_closed() {
        let current =
            apply_review_attention_v1(None, &request(ReviewAttentionCommandV1::MarkReviewed, 0))
                .expect("create")
                .attention;
        assert_eq!(
            apply_review_attention_v1(
                Some(&current),
                &request(ReviewAttentionCommandV1::MarkPending, 0),
            ),
            Err(ReviewAttentionErrorV1::RevisionConflict)
        );
        let mut another_source = request(ReviewAttentionCommandV1::MarkPending, 1);
        another_source.source_evidence_id = [8; STABLE_ID_BYTES_V1];
        assert_eq!(
            apply_review_attention_v1(Some(&current), &another_source),
            Err(ReviewAttentionErrorV1::InvalidSourceEvidence)
        );
        let mut invalid_owner = request(ReviewAttentionCommandV1::MarkPending, 1);
        invalid_owner.logical_owner_id = "communications/provider".to_owned();
        assert_eq!(
            apply_review_attention_v1(Some(&current), &invalid_owner),
            Err(ReviewAttentionErrorV1::InvalidOwner)
        );
    }

    #[test]
    fn snooze_is_future_bounded_and_timestamp_validated() {
        assert_eq!(
            apply_review_attention_v1(
                None,
                &request(
                    ReviewAttentionCommandV1::SnoozeUntil(ReviewTimestampV1 {
                        unix_seconds: 1_783_100_000,
                        nanos: 0,
                    }),
                    0,
                ),
            ),
            Err(ReviewAttentionErrorV1::InvalidSnoozeDeadline)
        );
        let mut invalid_clock = request(ReviewAttentionCommandV1::MarkReviewed, 0);
        invalid_clock.applied_at.nanos = 1_000_000_000;
        assert_eq!(
            apply_review_attention_v1(None, &invalid_clock),
            Err(ReviewAttentionErrorV1::InvalidTimestamp)
        );
    }
}
