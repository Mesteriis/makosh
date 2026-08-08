use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use connectrpc::{Response, ServiceResult};
use makosh_connectrpc_contracts::makosh::communications::v1::GetAttachmentPreviewResponse;

use super::communications_attachment_policy;
use super::communications_request_policy::invalid_argument_error;
use crate::domains::communications::storage::models::StoredCommunicationAttachmentWithBlob;

const MAX_TEXT_PREVIEW_BYTES: usize = 64 * 1024;
const MAX_IMAGE_PREVIEW_BYTES: usize = 5 * 1024 * 1024;
const MAX_AUDIO_PREVIEW_BYTES: usize = 24 * 1024 * 1024;
const MAX_VIDEO_PREVIEW_BYTES: usize = 32 * 1024 * 1024;

pub(super) fn text(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    let truncated = byte_count > MAX_TEXT_PREVIEW_BYTES;
    let preview_bytes = if truncated {
        &bytes[..MAX_TEXT_PREVIEW_BYTES]
    } else {
        &bytes
    };
    Response::ok(GetAttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "text".to_owned(),
        text: String::from_utf8_lossy(preview_bytes).into_owned(),
        data_url: None,
        truncated,
        byte_count: byte_count as u64,
        max_preview_bytes: MAX_TEXT_PREVIEW_BYTES as u64,
        ..Default::default()
    })
}

pub(super) fn image(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    binary(
        attachment,
        bytes,
        byte_count,
        "image",
        MAX_IMAGE_PREVIEW_BYTES,
        "image/png",
        communications_attachment_policy::image_content_type,
    )
}

pub(super) fn audio(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    binary(
        attachment,
        bytes,
        byte_count,
        "audio",
        MAX_AUDIO_PREVIEW_BYTES,
        "audio/mpeg",
        communications_attachment_policy::audio_content_type,
    )
}

pub(super) fn video(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    binary(
        attachment,
        bytes,
        byte_count,
        "video",
        MAX_VIDEO_PREVIEW_BYTES,
        "video/mp4",
        communications_attachment_policy::video_content_type,
    )
}

fn binary(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
    kind: &str,
    max_preview_bytes: usize,
    fallback: &str,
    content_type: fn(&StoredCommunicationAttachmentWithBlob) -> Option<&str>,
) -> ServiceResult<GetAttachmentPreviewResponse> {
    if byte_count > max_preview_bytes {
        return Err(invalid_argument_error(format!(
            "attachment {kind} preview exceeds size limit"
        )));
    }
    let data_url = format!(
        "data:{};base64,{}",
        content_type(&attachment).unwrap_or(fallback),
        BASE64_STANDARD.encode(bytes)
    );
    Response::ok(GetAttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: kind.to_owned(),
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count: byte_count as u64,
        max_preview_bytes: max_preview_bytes as u64,
        ..Default::default()
    })
}
