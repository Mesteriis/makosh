//! Magic-based renderer selection; provider metadata is not accepted.

use makosh_attachment_preview_docx::{AttachmentPreviewDocxRendererV1, FIXED_FONT_SHA256_V1};
use makosh_attachment_preview_image::AttachmentPreviewImageRendererV1;
use makosh_attachment_preview_media::AttachmentPreviewMediaRendererV1;
use makosh_attachment_preview_pdf::AttachmentPreviewPdfRendererV1;
use makosh_attachment_preview_renderer_contract::{
    AttachmentPreviewRenderRequestV1, AttachmentPreviewRenderResultV1,
    AttachmentPreviewRendererErrorV1, AttachmentPreviewRendererV1, AttachmentPreviewSourceFormatV1,
    detect_attachment_preview_source_format_v1,
};
use makosh_attachment_preview_text::AttachmentPreviewTextRendererV1;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Default)]
pub struct AttachmentPreviewRendererRuntimeV1;

impl AttachmentPreviewRendererRuntimeV1 {
    pub fn render(
        &self,
        source_bytes: &[u8],
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        let source_format = detect_attachment_preview_source_format_v1(source_bytes)?;
        let request = AttachmentPreviewRenderRequestV1 {
            source_format,
            source_bytes,
        };
        match source_format {
            AttachmentPreviewSourceFormatV1::PlainUtf8 => {
                AttachmentPreviewTextRendererV1.render(request)
            }
            AttachmentPreviewSourceFormatV1::Png
            | AttachmentPreviewSourceFormatV1::Jpeg
            | AttachmentPreviewSourceFormatV1::Gif
            | AttachmentPreviewSourceFormatV1::Webp => {
                AttachmentPreviewImageRendererV1.render(request)
            }
            AttachmentPreviewSourceFormatV1::Pdf => AttachmentPreviewPdfRendererV1.render(request),
            AttachmentPreviewSourceFormatV1::DocxContainerCandidate => {
                AttachmentPreviewDocxRendererV1.render(request)
            }
            AttachmentPreviewSourceFormatV1::Mp3 | AttachmentPreviewSourceFormatV1::Mp4 => {
                AttachmentPreviewMediaRendererV1.render(request)
            }
        }
    }
}

#[must_use]
pub fn attachment_preview_renderer_identity_v1() -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-preview.renderer-runtime.v1\0");
    digest.update(env!("CARGO_PKG_VERSION").as_bytes());
    digest.update(makosh_attachment_preview_text::PACKAGE.as_bytes());
    digest.update(makosh_attachment_preview_image::PACKAGE.as_bytes());
    digest.update(makosh_attachment_preview_pdf::PACKAGE.as_bytes());
    digest.update(makosh_attachment_preview_docx::PACKAGE.as_bytes());
    digest.update(FIXED_FONT_SHA256_V1.as_bytes());
    digest.update(makosh_attachment_preview_media::PACKAGE.as_bytes());
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use makosh_attachment_preview_api::wire::{
        AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1,
    };

    use super::*;

    #[test]
    fn dispatches_by_magic_without_a_provider_hint() {
        let rendered = AttachmentPreviewRendererRuntimeV1
            .render(b"hello\r\nmakosh\r\n")
            .expect("text preview");
        assert_eq!(rendered.preview_kind, AttachmentPreviewKindV1::Text);
        assert_eq!(
            rendered.content_type,
            AttachmentPreviewContentTypeV1::TextUtf8
        );
        assert_eq!(rendered.bytes, b"hello\nmakosh\n");
    }

    #[test]
    fn renderer_identity_is_stable_and_nonzero() {
        let first = attachment_preview_renderer_identity_v1();
        assert_eq!(first, attachment_preview_renderer_identity_v1());
        assert!(first.iter().any(|byte| *byte != 0));
    }
}
