//! Fixed-shape developer diagnostics for the Preview runtime.

use std::fmt::{self, Display, Formatter};

use makosh_attachment_preview_runtime::runtime::AttachmentPreviewRuntimeErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachmentPreviewDiagnosticStageV1 {
    Startup,
    ClientDelivery,
    Consume,
    CustodyMaterialize,
    CustodyOutbox,
    Render,
    ClientRealtime,
}

impl AttachmentPreviewDiagnosticStageV1 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::ClientDelivery => "client-delivery",
            Self::Consume => "consume",
            Self::CustodyMaterialize => "custody-materialize",
            Self::CustodyOutbox => "custody-outbox",
            Self::Render => "render",
            Self::ClientRealtime => "client-realtime",
        }
    }
}

struct AttachmentPreviewDiagnosticV1 {
    stage: AttachmentPreviewDiagnosticStageV1,
    error: AttachmentPreviewRuntimeErrorV1,
}

impl Display for AttachmentPreviewDiagnosticV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "developer_attachment_preview_runtime_error stage={} reason={}",
            self.stage.as_str(),
            self.error.sanitized_reason_code()
        )
    }
}

pub(crate) fn emit_attachment_preview_diagnostic_v1(
    stage: AttachmentPreviewDiagnosticStageV1,
    error: AttachmentPreviewRuntimeErrorV1,
) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("{}", AttachmentPreviewDiagnosticV1 { stage, error });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_fields_are_closed_and_sanitized() {
        let stages = [
            AttachmentPreviewDiagnosticStageV1::Startup,
            AttachmentPreviewDiagnosticStageV1::ClientDelivery,
            AttachmentPreviewDiagnosticStageV1::Consume,
            AttachmentPreviewDiagnosticStageV1::CustodyMaterialize,
            AttachmentPreviewDiagnosticStageV1::CustodyOutbox,
            AttachmentPreviewDiagnosticStageV1::Render,
            AttachmentPreviewDiagnosticStageV1::ClientRealtime,
        ];
        assert_eq!(
            stages.map(AttachmentPreviewDiagnosticStageV1::as_str),
            [
                "startup",
                "client-delivery",
                "consume",
                "custody-materialize",
                "custody-outbox",
                "render",
                "client-realtime",
            ]
        );
        assert_eq!(
            AttachmentPreviewRuntimeErrorV1::Admission.sanitized_reason_code(),
            "attachment_preview_runtime_admission_rejected"
        );
        assert_eq!(
            AttachmentPreviewRuntimeErrorV1::InvalidDelivery.sanitized_reason_code(),
            "attachment_preview_runtime_invalid_delivery"
        );
        assert_eq!(
            AttachmentPreviewRuntimeErrorV1::InvalidJob.sanitized_reason_code(),
            "attachment_preview_runtime_invalid_job"
        );
        assert_eq!(
            AttachmentPreviewRuntimeErrorV1::Unavailable.sanitized_reason_code(),
            "attachment_preview_runtime_unavailable"
        );
        let lines = [
            AttachmentPreviewRuntimeErrorV1::Admission,
            AttachmentPreviewRuntimeErrorV1::InvalidDelivery,
            AttachmentPreviewRuntimeErrorV1::InvalidJob,
            AttachmentPreviewRuntimeErrorV1::Unavailable,
        ]
        .map(|error| {
            AttachmentPreviewDiagnosticV1 {
                stage: AttachmentPreviewDiagnosticStageV1::Render,
                error,
            }
            .to_string()
        });
        assert!(lines.iter().all(|line| {
            line.starts_with("developer_attachment_preview_runtime_error stage=render reason=")
                && !line.contains("source")
                && !line.contains("blob")
                && !line.contains("receipt")
                && !line.contains("proof")
                && !line.contains("ticket")
                && !line.contains("provider")
        }));
    }
}
