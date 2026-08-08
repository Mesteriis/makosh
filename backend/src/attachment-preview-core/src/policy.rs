use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1, ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1,
    ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1, ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1,
    wire::AttachmentPreviewContentTypeV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewOutputPolicyErrorV1 {
    UnsupportedContentType,
    Empty,
    TooLarge,
}

pub fn preview_output_limit_v1(content_type: AttachmentPreviewContentTypeV1) -> Option<u64> {
    match content_type {
        AttachmentPreviewContentTypeV1::TextUtf8 => Some(ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1),
        AttachmentPreviewContentTypeV1::Png => Some(ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1),
        AttachmentPreviewContentTypeV1::MpegAudio => Some(ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1),
        AttachmentPreviewContentTypeV1::Mp4Video => Some(ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1),
        AttachmentPreviewContentTypeV1::Unspecified => None,
    }
}

pub fn validate_preview_output_v1(
    content_type: AttachmentPreviewContentTypeV1,
    size_bytes: u64,
) -> Result<(), AttachmentPreviewOutputPolicyErrorV1> {
    let limit = preview_output_limit_v1(content_type)
        .ok_or(AttachmentPreviewOutputPolicyErrorV1::UnsupportedContentType)?;
    if size_bytes == 0 {
        Err(AttachmentPreviewOutputPolicyErrorV1::Empty)
    } else if size_bytes > limit {
        Err(AttachmentPreviewOutputPolicyErrorV1::TooLarge)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_content_kind_has_an_exact_non_zero_bound() {
        for content_type in [
            AttachmentPreviewContentTypeV1::TextUtf8,
            AttachmentPreviewContentTypeV1::Png,
            AttachmentPreviewContentTypeV1::MpegAudio,
            AttachmentPreviewContentTypeV1::Mp4Video,
        ] {
            let limit = preview_output_limit_v1(content_type).unwrap();
            assert_eq!(validate_preview_output_v1(content_type, limit), Ok(()));
            assert_eq!(
                validate_preview_output_v1(content_type, limit + 1),
                Err(AttachmentPreviewOutputPolicyErrorV1::TooLarge)
            );
        }
    }
}
