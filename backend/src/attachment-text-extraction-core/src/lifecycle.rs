use makosh_attachment_text_extraction_api::ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionStateV1 {
    Accepted,
    AwaitingEvidence,
    Extracting,
    Ready,
    Unsupported,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextFormatV1 {
    PlainUtf8,
    Pdf,
    Docx,
    Ocr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionErrorV1 {
    NotSafe,
    Unsupported,
    SourceTooLarge,
    InvalidContent,
    ParserUnavailable,
    ParserFailed,
    CustodyRejected,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentTextExtractionStatusV1 {
    pub state: AttachmentTextExtractionStateV1,
    pub state_revision: u64,
    pub format: Option<AttachmentTextFormatV1>,
    pub extracted_size_bytes: u64,
    pub extraction_truncated: bool,
    pub error: Option<AttachmentTextExtractionErrorV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionTransitionV1 {
    AwaitEvidence,
    BeginExtraction,
    Complete {
        format: AttachmentTextFormatV1,
        extracted_size_bytes: u64,
        extraction_truncated: bool,
    },
    MarkUnsupported,
    Reject(AttachmentTextExtractionErrorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionTransitionErrorV1 {
    InvalidCurrentStatus,
    InvalidTransition,
    InvalidResult,
}

#[must_use]
pub const fn accepted_attachment_text_status_v1() -> AttachmentTextExtractionStatusV1 {
    AttachmentTextExtractionStatusV1 {
        state: AttachmentTextExtractionStateV1::Accepted,
        state_revision: 1,
        format: None,
        extracted_size_bytes: 0,
        extraction_truncated: false,
        error: None,
    }
}

pub fn transition_attachment_text_status_v1(
    current: &AttachmentTextExtractionStatusV1,
    transition: AttachmentTextExtractionTransitionV1,
) -> Result<AttachmentTextExtractionStatusV1, AttachmentTextExtractionTransitionErrorV1> {
    if !validate_attachment_text_status_v1(current) {
        return Err(AttachmentTextExtractionTransitionErrorV1::InvalidCurrentStatus);
    }
    let (state, format, extracted_size_bytes, extraction_truncated, error) =
        match (current.state, transition) {
            (
                AttachmentTextExtractionStateV1::Accepted,
                AttachmentTextExtractionTransitionV1::AwaitEvidence,
            ) => (
                AttachmentTextExtractionStateV1::AwaitingEvidence,
                None,
                0,
                false,
                None,
            ),
            (
                AttachmentTextExtractionStateV1::Accepted
                | AttachmentTextExtractionStateV1::AwaitingEvidence,
                AttachmentTextExtractionTransitionV1::BeginExtraction,
            ) => (
                AttachmentTextExtractionStateV1::Extracting,
                None,
                0,
                false,
                None,
            ),
            (
                AttachmentTextExtractionStateV1::Extracting,
                AttachmentTextExtractionTransitionV1::Complete {
                    format,
                    extracted_size_bytes,
                    extraction_truncated,
                },
            ) if valid_completed_result(extracted_size_bytes) => (
                AttachmentTextExtractionStateV1::Ready,
                Some(format),
                extracted_size_bytes,
                extraction_truncated,
                None,
            ),
            (
                AttachmentTextExtractionStateV1::Extracting,
                AttachmentTextExtractionTransitionV1::Complete { .. },
            ) => return Err(AttachmentTextExtractionTransitionErrorV1::InvalidResult),
            (
                AttachmentTextExtractionStateV1::Extracting,
                AttachmentTextExtractionTransitionV1::MarkUnsupported,
            ) => (
                AttachmentTextExtractionStateV1::Unsupported,
                None,
                0,
                false,
                Some(AttachmentTextExtractionErrorV1::Unsupported),
            ),
            (
                AttachmentTextExtractionStateV1::Accepted
                | AttachmentTextExtractionStateV1::AwaitingEvidence
                | AttachmentTextExtractionStateV1::Extracting,
                AttachmentTextExtractionTransitionV1::Reject(error),
            ) if error != AttachmentTextExtractionErrorV1::Unsupported => (
                AttachmentTextExtractionStateV1::Rejected,
                None,
                0,
                false,
                Some(error),
            ),
            _ => return Err(AttachmentTextExtractionTransitionErrorV1::InvalidTransition),
        };
    let next = AttachmentTextExtractionStatusV1 {
        state,
        state_revision: current
            .state_revision
            .checked_add(1)
            .ok_or(AttachmentTextExtractionTransitionErrorV1::InvalidTransition)?,
        format,
        extracted_size_bytes,
        extraction_truncated,
        error,
    };
    if !validate_attachment_text_status_v1(&next) {
        return Err(AttachmentTextExtractionTransitionErrorV1::InvalidTransition);
    }
    Ok(next)
}

#[must_use]
pub fn validate_attachment_text_status_v1(status: &AttachmentTextExtractionStatusV1) -> bool {
    status.state_revision > 0
        && match status.state {
            AttachmentTextExtractionStateV1::Accepted
            | AttachmentTextExtractionStateV1::AwaitingEvidence
            | AttachmentTextExtractionStateV1::Extracting => {
                status.format.is_none()
                    && status.extracted_size_bytes == 0
                    && !status.extraction_truncated
                    && status.error.is_none()
            }
            AttachmentTextExtractionStateV1::Ready => {
                status.format.is_some()
                    && valid_completed_result(status.extracted_size_bytes)
                    && status.error.is_none()
            }
            AttachmentTextExtractionStateV1::Unsupported => {
                status.format.is_none()
                    && status.extracted_size_bytes == 0
                    && !status.extraction_truncated
                    && status.error == Some(AttachmentTextExtractionErrorV1::Unsupported)
            }
            AttachmentTextExtractionStateV1::Rejected => {
                status.format.is_none()
                    && status.extracted_size_bytes == 0
                    && !status.extraction_truncated
                    && status
                        .error
                        .is_some_and(|error| error != AttachmentTextExtractionErrorV1::Unsupported)
            }
        }
}

fn valid_completed_result(extracted_size_bytes: u64) -> bool {
    (1..=ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1 as u64).contains(&extracted_size_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_requires_extraction_and_a_bounded_non_empty_result() {
        let awaiting = transition_attachment_text_status_v1(
            &accepted_attachment_text_status_v1(),
            AttachmentTextExtractionTransitionV1::AwaitEvidence,
        )
        .expect("await evidence");
        let extracting = transition_attachment_text_status_v1(
            &awaiting,
            AttachmentTextExtractionTransitionV1::BeginExtraction,
        )
        .expect("extracting");
        let ready = transition_attachment_text_status_v1(
            &extracting,
            AttachmentTextExtractionTransitionV1::Complete {
                format: AttachmentTextFormatV1::Docx,
                extracted_size_bytes: 42,
                extraction_truncated: false,
            },
        )
        .expect("ready");
        assert_eq!(ready.state, AttachmentTextExtractionStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
        assert_eq!(
            transition_attachment_text_status_v1(
                &extracting,
                AttachmentTextExtractionTransitionV1::Complete {
                    format: AttachmentTextFormatV1::Pdf,
                    extracted_size_bytes: 0,
                    extraction_truncated: false,
                },
            ),
            Err(AttachmentTextExtractionTransitionErrorV1::InvalidResult)
        );
    }

    #[test]
    fn unsupported_and_rejected_are_distinct_terminal_states() {
        let extracting = transition_attachment_text_status_v1(
            &accepted_attachment_text_status_v1(),
            AttachmentTextExtractionTransitionV1::BeginExtraction,
        )
        .expect("extracting");
        let unsupported = transition_attachment_text_status_v1(
            &extracting,
            AttachmentTextExtractionTransitionV1::MarkUnsupported,
        )
        .expect("unsupported");
        assert_eq!(
            unsupported.state,
            AttachmentTextExtractionStateV1::Unsupported
        );
        let rejected = transition_attachment_text_status_v1(
            &extracting,
            AttachmentTextExtractionTransitionV1::Reject(
                AttachmentTextExtractionErrorV1::ParserFailed,
            ),
        )
        .expect("rejected");
        assert_eq!(rejected.state, AttachmentTextExtractionStateV1::Rejected);
        assert_eq!(
            transition_attachment_text_status_v1(
                &unsupported,
                AttachmentTextExtractionTransitionV1::BeginExtraction,
            ),
            Err(AttachmentTextExtractionTransitionErrorV1::InvalidTransition)
        );
    }
}
