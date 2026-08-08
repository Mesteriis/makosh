#![forbid(unsafe_code)]

use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1,
    wire::{AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1},
};
use makosh_attachment_preview_renderer_contract::{
    AttachmentPreviewRenderRequestV1, AttachmentPreviewRenderResultV1,
    AttachmentPreviewRendererErrorV1, AttachmentPreviewRendererV1, AttachmentPreviewSourceFormatV1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-text";

#[derive(Clone, Copy, Debug, Default)]
pub struct AttachmentPreviewTextRendererV1;

impl AttachmentPreviewRendererV1 for AttachmentPreviewTextRendererV1 {
    fn render(
        &self,
        request: AttachmentPreviewRenderRequestV1<'_>,
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        if request.source_format != AttachmentPreviewSourceFormatV1::PlainUtf8 {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        let source = std::str::from_utf8(request.source_bytes)
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
        if source.contains('\0') {
            return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
        }
        let (bytes, truncated) = normalized_visible_utf8_v1(source);
        if bytes.is_empty() {
            return Err(AttachmentPreviewRendererErrorV1::Empty);
        }
        Ok(AttachmentPreviewRenderResultV1 {
            preview_kind: AttachmentPreviewKindV1::Text,
            content_type: AttachmentPreviewContentTypeV1::TextUtf8,
            bytes,
            truncated,
        })
    }
}

fn normalized_visible_utf8_v1(source: &str) -> (Vec<u8>, bool) {
    let limit = ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1 as usize;
    let mut output = Vec::with_capacity(source.len().min(limit));
    let mut chars = source.chars().peekable();
    let mut truncated = false;
    while let Some(character) = chars.next() {
        let character = if character == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            '\n'
        } else {
            character
        };
        let mut encoded = [0_u8; 4];
        let encoded = character.encode_utf8(&mut encoded).as_bytes();
        if output.len() + encoded.len() > limit {
            truncated = true;
            break;
        }
        output.extend_from_slice(encoded);
    }
    if chars.next().is_some() {
        truncated = true;
    }
    (output, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_lines_and_never_splits_utf8() {
        let renderer = AttachmentPreviewTextRendererV1;
        let result = renderer
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::PlainUtf8,
                source_bytes: "one\r\ntwo\rтри".as_bytes(),
            })
            .unwrap();
        assert_eq!(std::str::from_utf8(&result.bytes).unwrap(), "one\ntwo\nтри");
        assert!(!result.truncated);
    }

    #[test]
    fn truncation_is_bounded_and_explicit() {
        let source = format!(
            "{}я",
            "a".repeat(ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1 as usize)
        );
        let result = AttachmentPreviewTextRendererV1
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::PlainUtf8,
                source_bytes: source.as_bytes(),
            })
            .unwrap();
        assert_eq!(
            result.bytes.len(),
            ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1 as usize
        );
        assert!(result.truncated);
        assert!(std::str::from_utf8(&result.bytes).is_ok());
    }
}
