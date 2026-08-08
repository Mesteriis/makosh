#![forbid(unsafe_code)]

use makosh_attachment_preview_api::wire::{
    AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-renderer-contract";
pub const ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1: usize = 100 * 1024 * 1024;
pub const ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1: u64 = 16_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewSourceFormatV1 {
    PlainUtf8,
    Png,
    Jpeg,
    Gif,
    Webp,
    Pdf,
    DocxContainerCandidate,
    Mp3,
    Mp4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewRenderRequestV1<'a> {
    pub source_format: AttachmentPreviewSourceFormatV1,
    pub source_bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewRenderResultV1 {
    pub preview_kind: AttachmentPreviewKindV1,
    pub content_type: AttachmentPreviewContentTypeV1,
    pub bytes: Vec<u8>,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewRendererErrorV1 {
    Empty,
    SourceTooLarge,
    Unsupported,
    InvalidContent,
    OutputTooLarge,
    Failed,
}

pub trait AttachmentPreviewRendererV1 {
    fn render(
        &self,
        request: AttachmentPreviewRenderRequestV1<'_>,
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1>;
}

pub fn detect_attachment_preview_source_format_v1(
    bytes: &[u8],
) -> Result<AttachmentPreviewSourceFormatV1, AttachmentPreviewRendererErrorV1> {
    if bytes.is_empty() {
        return Err(AttachmentPreviewRendererErrorV1::Empty);
    }
    if bytes.len() > ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1 {
        return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok(AttachmentPreviewSourceFormatV1::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Ok(AttachmentPreviewSourceFormatV1::Jpeg);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Ok(AttachmentPreviewSourceFormatV1::Gif);
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Ok(AttachmentPreviewSourceFormatV1::Webp);
    }
    if bytes.starts_with(b"%PDF-") {
        return Ok(AttachmentPreviewSourceFormatV1::Pdf);
    }
    if bytes.starts_with(b"PK\x03\x04") {
        return Ok(AttachmentPreviewSourceFormatV1::DocxContainerCandidate);
    }
    if bytes.starts_with(b"ID3") || is_mpeg_audio_frame(bytes) {
        return Ok(AttachmentPreviewSourceFormatV1::Mp3);
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return Ok(AttachmentPreviewSourceFormatV1::Mp4);
    }
    if !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok() {
        return Ok(AttachmentPreviewSourceFormatV1::PlainUtf8);
    }
    Err(AttachmentPreviewRendererErrorV1::Unsupported)
}

fn is_mpeg_audio_frame(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0 && bytes[1] & 0x06 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_uses_bytes_not_metadata() {
        assert_eq!(
            detect_attachment_preview_source_format_v1(b"%PDF-1.7\n").unwrap(),
            AttachmentPreviewSourceFormatV1::Pdf
        );
        assert_eq!(
            detect_attachment_preview_source_format_v1(b"hello\r\n").unwrap(),
            AttachmentPreviewSourceFormatV1::PlainUtf8
        );
        assert_eq!(
            detect_attachment_preview_source_format_v1(b"PK\x03\x04container").unwrap(),
            AttachmentPreviewSourceFormatV1::DocxContainerCandidate
        );
    }

    #[test]
    fn active_or_opaque_content_is_not_text_fallback() {
        assert_eq!(
            detect_attachment_preview_source_format_v1(b"<svg>\0<script>"),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
        assert_eq!(
            detect_attachment_preview_source_format_v1(&[]),
            Err(AttachmentPreviewRendererErrorV1::Empty)
        );
    }
}
