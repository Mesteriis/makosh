use makosh_attachment_preview_api::wire::{
    AttachmentPreviewContentTypeV1, AttachmentPreviewErrorCodeV1, AttachmentPreviewKindV1,
    AttachmentPreviewStateV1,
};

use crate::validate_preview_output_v1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewStatusV1 {
    pub state: AttachmentPreviewStateV1,
    pub state_revision: u64,
    pub preview_kind: Option<AttachmentPreviewKindV1>,
    pub content_type: Option<AttachmentPreviewContentTypeV1>,
    pub preview_size_bytes: u64,
    pub truncated: bool,
    pub error: Option<AttachmentPreviewErrorCodeV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewTransitionV1 {
    AwaitEvidence,
    BeginRendering,
    Complete {
        preview_kind: AttachmentPreviewKindV1,
        content_type: AttachmentPreviewContentTypeV1,
        preview_size_bytes: u64,
        truncated: bool,
    },
    MarkUnsupported,
    Reject(AttachmentPreviewErrorCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewTransitionErrorV1 {
    InvalidCurrentStatus,
    InvalidState,
    InvalidTransition,
    InvalidResult,
}

#[must_use]
pub const fn accepted_attachment_preview_status_v1() -> AttachmentPreviewStatusV1 {
    AttachmentPreviewStatusV1 {
        state: AttachmentPreviewStateV1::Accepted,
        state_revision: 1,
        preview_kind: None,
        content_type: None,
        preview_size_bytes: 0,
        truncated: false,
        error: None,
    }
}

pub fn transition_attachment_preview_status_v1(
    current: &AttachmentPreviewStatusV1,
    transition: AttachmentPreviewTransitionV1,
) -> Result<AttachmentPreviewStatusV1, AttachmentPreviewTransitionErrorV1> {
    if !validate_attachment_preview_status_v1(current) {
        return Err(AttachmentPreviewTransitionErrorV1::InvalidCurrentStatus);
    }
    let (state, preview_kind, content_type, preview_size_bytes, truncated, error) =
        match (current.state, transition) {
            (AttachmentPreviewStateV1::Accepted, AttachmentPreviewTransitionV1::AwaitEvidence) => (
                AttachmentPreviewStateV1::AwaitingEvidence,
                None,
                None,
                0,
                false,
                None,
            ),
            (
                AttachmentPreviewStateV1::Accepted | AttachmentPreviewStateV1::AwaitingEvidence,
                AttachmentPreviewTransitionV1::BeginRendering,
            ) => (
                AttachmentPreviewStateV1::Rendering,
                None,
                None,
                0,
                false,
                None,
            ),
            (
                AttachmentPreviewStateV1::Rendering,
                AttachmentPreviewTransitionV1::Complete {
                    preview_kind,
                    content_type,
                    preview_size_bytes,
                    truncated,
                },
            ) if valid_completed_result(preview_kind, content_type, preview_size_bytes) => (
                AttachmentPreviewStateV1::Ready,
                Some(preview_kind),
                Some(content_type),
                preview_size_bytes,
                truncated,
                None,
            ),
            (
                AttachmentPreviewStateV1::Rendering,
                AttachmentPreviewTransitionV1::Complete { .. },
            ) => {
                return Err(AttachmentPreviewTransitionErrorV1::InvalidResult);
            }
            (
                AttachmentPreviewStateV1::AwaitingEvidence | AttachmentPreviewStateV1::Rendering,
                AttachmentPreviewTransitionV1::MarkUnsupported,
            ) => (
                AttachmentPreviewStateV1::Unsupported,
                None,
                None,
                0,
                false,
                Some(AttachmentPreviewErrorCodeV1::Unsupported),
            ),
            (
                AttachmentPreviewStateV1::Accepted
                | AttachmentPreviewStateV1::AwaitingEvidence
                | AttachmentPreviewStateV1::Rendering,
                AttachmentPreviewTransitionV1::Reject(error),
            ) if valid_terminal_error(error) => (
                AttachmentPreviewStateV1::Rejected,
                None,
                None,
                0,
                false,
                Some(error),
            ),
            _ => return Err(AttachmentPreviewTransitionErrorV1::InvalidTransition),
        };
    let next = AttachmentPreviewStatusV1 {
        state,
        state_revision: current
            .state_revision
            .checked_add(1)
            .ok_or(AttachmentPreviewTransitionErrorV1::InvalidTransition)?,
        preview_kind,
        content_type,
        preview_size_bytes,
        truncated,
        error,
    };
    if validate_attachment_preview_status_v1(&next) {
        Ok(next)
    } else {
        Err(AttachmentPreviewTransitionErrorV1::InvalidTransition)
    }
}

#[must_use]
pub fn validate_attachment_preview_status_v1(status: &AttachmentPreviewStatusV1) -> bool {
    status.state_revision > 0
        && match status.state {
            AttachmentPreviewStateV1::Accepted
            | AttachmentPreviewStateV1::AwaitingEvidence
            | AttachmentPreviewStateV1::Rendering => empty_result(status),
            AttachmentPreviewStateV1::Ready => {
                let (Some(preview_kind), Some(content_type)) =
                    (status.preview_kind, status.content_type)
                else {
                    return false;
                };
                status.error.is_none()
                    && valid_completed_result(preview_kind, content_type, status.preview_size_bytes)
            }
            AttachmentPreviewStateV1::Unsupported => {
                empty_payload(status)
                    && status.error == Some(AttachmentPreviewErrorCodeV1::Unsupported)
            }
            AttachmentPreviewStateV1::Rejected => {
                empty_payload(status) && status.error.is_some_and(valid_terminal_error)
            }
            AttachmentPreviewStateV1::Unspecified => false,
        }
}

pub fn transition_attachment_preview_v1(
    current: AttachmentPreviewStateV1,
    next: AttachmentPreviewStateV1,
) -> Result<AttachmentPreviewStateV1, AttachmentPreviewTransitionErrorV1> {
    use AttachmentPreviewStateV1::{
        Accepted, AwaitingEvidence, Ready, Rejected, Rendering, Unsupported,
    };
    if current == next && current != AttachmentPreviewStateV1::Unspecified {
        return Ok(current);
    }
    if matches!(
        (current, next),
        (Accepted, AwaitingEvidence)
            | (Accepted, Rejected)
            | (AwaitingEvidence, Rendering)
            | (AwaitingEvidence, Unsupported)
            | (AwaitingEvidence, Rejected)
            | (Rendering, Ready)
            | (Rendering, Unsupported)
            | (Rendering, Rejected)
    ) {
        Ok(next)
    } else if current == AttachmentPreviewStateV1::Unspecified
        || next == AttachmentPreviewStateV1::Unspecified
    {
        Err(AttachmentPreviewTransitionErrorV1::InvalidState)
    } else {
        Err(AttachmentPreviewTransitionErrorV1::InvalidTransition)
    }
}

fn empty_result(status: &AttachmentPreviewStatusV1) -> bool {
    empty_payload(status) && status.error.is_none()
}

fn empty_payload(status: &AttachmentPreviewStatusV1) -> bool {
    status.preview_kind.is_none()
        && status.content_type.is_none()
        && status.preview_size_bytes == 0
        && !status.truncated
}

fn valid_completed_result(
    preview_kind: AttachmentPreviewKindV1,
    content_type: AttachmentPreviewContentTypeV1,
    preview_size_bytes: u64,
) -> bool {
    preview_kind != AttachmentPreviewKindV1::Unspecified
        && validate_preview_output_v1(content_type, preview_size_bytes).is_ok()
}

fn valid_terminal_error(error: AttachmentPreviewErrorCodeV1) -> bool {
    !matches!(
        error,
        AttachmentPreviewErrorCodeV1::Unspecified
            | AttachmentPreviewErrorCodeV1::NotFound
            | AttachmentPreviewErrorCodeV1::Unsupported
            | AttachmentPreviewErrorCodeV1::TicketExpired
            | AttachmentPreviewErrorCodeV1::TicketUsed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_requires_rendering_and_a_bounded_typed_artifact() {
        let awaiting = transition_attachment_preview_status_v1(
            &accepted_attachment_preview_status_v1(),
            AttachmentPreviewTransitionV1::AwaitEvidence,
        )
        .expect("awaiting");
        let rendering = transition_attachment_preview_status_v1(
            &awaiting,
            AttachmentPreviewTransitionV1::BeginRendering,
        )
        .expect("rendering");
        let ready = transition_attachment_preview_status_v1(
            &rendering,
            AttachmentPreviewTransitionV1::Complete {
                preview_kind: AttachmentPreviewKindV1::Document,
                content_type: AttachmentPreviewContentTypeV1::Png,
                preview_size_bytes: 42,
                truncated: false,
            },
        )
        .expect("ready");
        assert_eq!(ready.state, AttachmentPreviewStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
        assert_eq!(
            transition_attachment_preview_status_v1(
                &rendering,
                AttachmentPreviewTransitionV1::Complete {
                    preview_kind: AttachmentPreviewKindV1::Document,
                    content_type: AttachmentPreviewContentTypeV1::Png,
                    preview_size_bytes: 0,
                    truncated: false,
                },
            ),
            Err(AttachmentPreviewTransitionErrorV1::InvalidResult)
        );
    }

    #[test]
    fn unsupported_and_ticket_errors_cannot_mutate_run_truth() {
        let rendering = transition_attachment_preview_status_v1(
            &accepted_attachment_preview_status_v1(),
            AttachmentPreviewTransitionV1::BeginRendering,
        )
        .expect("rendering");
        assert_eq!(
            transition_attachment_preview_status_v1(
                &rendering,
                AttachmentPreviewTransitionV1::MarkUnsupported,
            )
            .expect("unsupported")
            .state,
            AttachmentPreviewStateV1::Unsupported
        );
        assert_eq!(
            transition_attachment_preview_status_v1(
                &rendering,
                AttachmentPreviewTransitionV1::Reject(AttachmentPreviewErrorCodeV1::TicketExpired,),
            ),
            Err(AttachmentPreviewTransitionErrorV1::InvalidTransition)
        );
    }
}
