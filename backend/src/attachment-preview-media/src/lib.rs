#![forbid(unsafe_code)]

use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1, ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1,
    wire::{AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1},
};
use makosh_attachment_preview_renderer_contract::{
    AttachmentPreviewRenderRequestV1, AttachmentPreviewRenderResultV1,
    AttachmentPreviewRendererErrorV1, AttachmentPreviewRendererV1, AttachmentPreviewSourceFormatV1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-media";
const MAX_MP4_BOXES_V1: usize = 1_024;

#[derive(Clone, Copy, Debug, Default)]
pub struct AttachmentPreviewMediaRendererV1;

impl AttachmentPreviewRendererV1 for AttachmentPreviewMediaRendererV1 {
    fn render(
        &self,
        request: AttachmentPreviewRenderRequestV1<'_>,
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        let (preview_kind, content_type, limit) = match request.source_format {
            AttachmentPreviewSourceFormatV1::Mp3 => {
                validate_mp3_v1(request.source_bytes)?;
                (
                    AttachmentPreviewKindV1::Audio,
                    AttachmentPreviewContentTypeV1::MpegAudio,
                    ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1,
                )
            }
            AttachmentPreviewSourceFormatV1::Mp4 => {
                validate_mp4_v1(request.source_bytes)?;
                (
                    AttachmentPreviewKindV1::Video,
                    AttachmentPreviewContentTypeV1::Mp4Video,
                    ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1,
                )
            }
            _ => return Err(AttachmentPreviewRendererErrorV1::Unsupported),
        };
        if request.source_bytes.len() as u64 > limit {
            return Err(AttachmentPreviewRendererErrorV1::OutputTooLarge);
        }
        Ok(AttachmentPreviewRenderResultV1 {
            preview_kind,
            content_type,
            bytes: request.source_bytes.to_vec(),
            truncated: false,
        })
    }
}

fn validate_mp3_v1(bytes: &[u8]) -> Result<(), AttachmentPreviewRendererErrorV1> {
    if bytes.len() < 4 {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    let frame_offset = if bytes.starts_with(b"ID3") {
        if bytes.len() < 10 || bytes[6..10].iter().any(|byte| byte & 0x80 != 0) {
            return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
        }
        let tag_size = bytes[6..10]
            .iter()
            .fold(0_usize, |value, byte| (value << 7) | usize::from(*byte));
        10_usize
            .checked_add(tag_size)
            .ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)?
    } else {
        0
    };
    let header = bytes
        .get(frame_offset..frame_offset + 4)
        .ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)?;
    if header[0] != 0xff || header[1] & 0xe0 != 0xe0 || header[1] & 0x06 == 0 {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    Ok(())
}

fn validate_mp4_v1(bytes: &[u8]) -> Result<(), AttachmentPreviewRendererErrorV1> {
    let mut offset = 0_usize;
    let mut boxes = 0_usize;
    let mut allowed_brand = false;
    let mut has_moov = false;
    let mut has_mdat = false;
    while offset < bytes.len() {
        boxes += 1;
        if boxes > MAX_MP4_BOXES_V1 {
            return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
        }
        let header = bytes
            .get(offset..offset + 8)
            .ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let size = u32::from_be_bytes(header[..4].try_into().expect("box size")) as usize;
        if size < 8 || offset.checked_add(size).is_none_or(|end| end > bytes.len()) {
            return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
        }
        let kind = &header[4..8];
        if boxes == 1 && kind != b"ftyp" {
            return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
        }
        if kind == b"ftyp" {
            let payload = &bytes[offset + 8..offset + size];
            if payload.len() < 8 || !payload[8..].len().is_multiple_of(4) {
                return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
            }
            allowed_brand = allowed_mp4_brand(&payload[..4])
                || payload[8..].chunks_exact(4).any(allowed_mp4_brand);
        } else if kind == b"moov" {
            has_moov = true;
        } else if kind == b"mdat" {
            has_mdat = size > 8;
        }
        offset += size;
    }
    if offset == bytes.len() && allowed_brand && has_moov && has_mdat {
        Ok(())
    } else {
        Err(AttachmentPreviewRendererErrorV1::InvalidContent)
    }
}

fn allowed_mp4_brand(brand: &[u8]) -> bool {
    matches!(
        brand,
        b"isom" | b"iso2" | b"mp41" | b"mp42" | b"M4V " | b"avc1"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mp4_box(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut bytes = ((8 + payload.len()) as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn accepts_bounded_mp3_with_a_real_frame_header() {
        let result = AttachmentPreviewMediaRendererV1
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Mp3,
                source_bytes: &[0xff, 0xfb, 0x90, 0x64],
            })
            .unwrap();
        assert_eq!(
            result.content_type,
            AttachmentPreviewContentTypeV1::MpegAudio
        );
    }

    #[test]
    fn mp4_requires_allowed_ftyp_moov_and_nonempty_mdat() {
        let mut source = mp4_box(b"ftyp", b"isom\0\0\0\0isom");
        source.extend(mp4_box(b"moov", b""));
        source.extend(mp4_box(b"mdat", b"frame"));
        let result = AttachmentPreviewMediaRendererV1
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Mp4,
                source_bytes: &source,
            })
            .unwrap();
        assert_eq!(
            result.content_type,
            AttachmentPreviewContentTypeV1::Mp4Video
        );
        let without_moov = [
            mp4_box(b"ftyp", b"isom\0\0\0\0isom"),
            mp4_box(b"mdat", b"frame"),
        ]
        .concat();
        assert_eq!(
            AttachmentPreviewMediaRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Mp4,
                source_bytes: &without_moov,
            }),
            Err(AttachmentPreviewRendererErrorV1::InvalidContent)
        );
    }
}
