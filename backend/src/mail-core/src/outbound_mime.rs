//! Deterministic bounded RFC822/MIME composition for Mail delivery.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use makosh_mail_api::{MailContractError, OutgoingMailV1, valid_mailbox, valid_message_bytes};
use sha2::{Digest, Sha256};

pub const MAX_OUTBOUND_ATTACHMENT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_OUTBOUND_RFC822_BYTES: usize = 24 * 1024 * 1024;
const MAX_FILENAME_BYTES: usize = 512;
const MAX_MEDIA_TYPE_BYTES: usize = 256;
const BASE64_LINE_BYTES: usize = 76;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboundAttachmentDispositionV1 {
    Attachment,
    Inline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboundAttachmentV1 {
    pub anchor_id: [u8; 16],
    pub filename: Option<String>,
    pub media_type: String,
    pub disposition: OutboundAttachmentDispositionV1,
    pub bytes: Vec<u8>,
}

pub fn compose_rfc822_with_attachments(
    from_address: &str,
    message: &OutgoingMailV1,
    attachments: &[OutboundAttachmentV1],
) -> Result<String, MailContractError> {
    validate_message(from_address, message)?;
    if attachments.is_empty() {
        return Ok(plain_text_message(from_address, message));
    }
    if attachments.len() > makosh_mail_api::MAX_DELIVERY_ATTACHMENTS {
        return Err(MailContractError::InvalidPayload);
    }
    if attachments.iter().enumerate().any(|(index, attachment)| {
        attachments[..index]
            .iter()
            .any(|prior| prior.anchor_id == attachment.anchor_id)
    }) {
        return Err(MailContractError::InvalidPayload);
    }
    let total_attachment_bytes = attachments.iter().try_fold(0_usize, |total, attachment| {
        validate_attachment(attachment)?;
        total
            .checked_add(attachment.bytes.len())
            .ok_or(MailContractError::InvalidPayload)
    })?;
    if total_attachment_bytes > MAX_OUTBOUND_ATTACHMENT_BYTES {
        return Err(MailContractError::InvalidPayload);
    }

    let boundary = mime_boundary(message, attachments);
    let recipients = message.recipients.join(", ");
    let cc_header = recipient_header("Cc", &message.cc_recipients);
    let mut rendered = format!(
        "From: {}\r\nTo: {recipients}\r\n{cc_header}Subject: {}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n",
        from_address, message.subject,
    );
    push_boundary(&mut rendered, &boundary);
    rendered.push_str(
        "Content-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n",
    );
    push_base64(&mut rendered, normalize_crlf(&message.text_body).as_bytes());

    for attachment in attachments {
        push_boundary(&mut rendered, &boundary);
        rendered.push_str("Content-Type: ");
        rendered.push_str(&attachment.media_type);
        rendered.push_str("\r\nContent-Transfer-Encoding: base64\r\nContent-Disposition: ");
        rendered.push_str(match attachment.disposition {
            OutboundAttachmentDispositionV1::Attachment => "attachment",
            OutboundAttachmentDispositionV1::Inline => "inline",
        });
        if let Some(filename) = attachment.filename.as_deref() {
            if quoted_ascii_filename(filename) {
                rendered.push_str("; filename=\"");
                rendered.push_str(filename);
                rendered.push('"');
            }
            rendered.push_str("; filename*=UTF-8''");
            rendered.push_str(&percent_encode_parameter(filename));
        }
        rendered.push_str("\r\n\r\n");
        push_base64(&mut rendered, &attachment.bytes);
    }
    rendered.push_str("--");
    rendered.push_str(&boundary);
    rendered.push_str("--\r\n");
    if rendered.len() > MAX_OUTBOUND_RFC822_BYTES {
        return Err(MailContractError::InvalidPayload);
    }
    Ok(rendered)
}

pub(crate) fn validate_message(
    from_address: &str,
    message: &OutgoingMailV1,
) -> Result<(), MailContractError> {
    if message.operation_id.trim().is_empty()
        || message.connection_id.trim().is_empty()
        || message.provider_conversation_id.trim().is_empty()
        || !valid_mailbox(from_address)
        || message.recipients.is_empty()
        || message
            .recipients
            .iter()
            .chain(&message.cc_recipients)
            .chain(&message.bcc_recipients)
            .any(|recipient| !valid_mailbox(recipient))
        || invalid_header(&message.subject)
        || message.subject.len() > 998
        || !valid_message_bytes(message.text_body.len())
    {
        return Err(MailContractError::InvalidPayload);
    }
    Ok(())
}

pub(crate) fn plain_text_message(from_address: &str, message: &OutgoingMailV1) -> String {
    let recipients = message.recipients.join(", ");
    let cc_header = recipient_header("Cc", &message.cc_recipients);
    format!(
        "From: {}\r\nTo: {recipients}\r\n{cc_header}Subject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{}",
        from_address,
        message.subject,
        normalize_crlf(&message.text_body),
    )
}

fn recipient_header(name: &str, recipients: &[String]) -> String {
    if recipients.is_empty() {
        String::new()
    } else {
        format!("{name}: {}\r\n", recipients.join(", "))
    }
}

fn validate_attachment(attachment: &OutboundAttachmentV1) -> Result<(), MailContractError> {
    if attachment.anchor_id.iter().all(|byte| *byte == 0)
        || attachment.bytes.is_empty()
        || attachment.bytes.len() > MAX_OUTBOUND_ATTACHMENT_BYTES
        || !valid_media_type(&attachment.media_type)
        || attachment.filename.as_deref().is_some_and(|filename| {
            filename.is_empty()
                || filename.len() > MAX_FILENAME_BYTES
                || filename.contains(['\r', '\n', '\0'])
        })
    {
        return Err(MailContractError::InvalidPayload);
    }
    Ok(())
}

fn valid_media_type(value: &str) -> bool {
    value.is_ascii()
        && !value.is_empty()
        && value.len() <= MAX_MEDIA_TYPE_BYTES
        && !value.contains(char::is_whitespace)
        && !value.contains(['\r', '\n', '\0', '"', ';'])
        && value
            .split_once('/')
            .is_some_and(|(kind, subtype)| !kind.is_empty() && !subtype.is_empty())
}

fn mime_boundary(message: &OutgoingMailV1, attachments: &[OutboundAttachmentV1]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.mail.outbound-mime-boundary.v1\0");
    hasher.update(message.operation_id.as_bytes());
    for attachment in attachments {
        hasher.update(attachment.anchor_id);
        hasher.update(Sha256::digest(&attachment.bytes));
    }
    let digest = hasher.finalize();
    let mut boundary = String::from("makosh-");
    for byte in &digest[..16] {
        use std::fmt::Write as _;
        write!(&mut boundary, "{byte:02x}").expect("writing to String cannot fail");
    }
    boundary
}

fn push_boundary(rendered: &mut String, boundary: &str) {
    rendered.push_str("--");
    rendered.push_str(boundary);
    rendered.push_str("\r\n");
}

fn push_base64(rendered: &mut String, bytes: &[u8]) {
    let encoded = STANDARD.encode(bytes);
    for chunk in encoded.as_bytes().chunks(BASE64_LINE_BYTES) {
        rendered.push_str(std::str::from_utf8(chunk).expect("base64 is ASCII"));
        rendered.push_str("\r\n");
    }
}

fn percent_encode_parameter(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            )
        {
            encoded.push(char::from(*byte));
        } else {
            use std::fmt::Write as _;
            write!(&mut encoded, "%{byte:02X}").expect("writing to String cannot fail");
        }
    }
    encoded
}

fn quoted_ascii_filename(value: &str) -> bool {
    value.is_ascii()
        && value
            .bytes()
            .all(|byte| (0x20..=0x7e).contains(&byte) && !matches!(byte, b'"' | b'\\'))
}

fn invalid_header(value: &str) -> bool {
    value.is_empty() || value.contains(['\r', '\n', '\0'])
}

fn normalize_crlf(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message() -> OutgoingMailV1 {
        OutgoingMailV1 {
            operation_id: "operation".to_owned(),
            connection_id: "connection".to_owned(),
            provider_conversation_id: "thread-1".to_owned(),
            recipients: vec!["recipient@example.test".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Report".to_owned(),
            text_body: "line one\nline two".to_owned(),
        }
    }

    fn attachment() -> OutboundAttachmentV1 {
        OutboundAttachmentV1 {
            anchor_id: [7; 16],
            filename: Some("report 2026.pdf".to_owned()),
            media_type: "application/pdf".to_owned(),
            disposition: OutboundAttachmentDispositionV1::Attachment,
            bytes: b"pdf-bytes".to_vec(),
        }
    }

    #[test]
    fn composes_deterministic_bounded_multipart_message() {
        let first =
            compose_rfc822_with_attachments("owner@example.test", &message(), &[attachment()])
                .expect("valid MIME");
        let second =
            compose_rfc822_with_attachments("owner@example.test", &message(), &[attachment()])
                .expect("valid MIME");

        assert_eq!(first, second);
        assert!(first.contains("Content-Type: multipart/mixed"));
        assert!(first.contains("filename=\"report 2026.pdf\""));
        assert!(first.contains("filename*=UTF-8''report%202026.pdf"));
        assert!(first.contains("\r\ncGRmLWJ5dGVz\r\n"));
        assert!(first.len() <= MAX_OUTBOUND_RFC822_BYTES);
    }

    #[test]
    fn rejects_header_metadata_and_total_size_violations() {
        let mut invalid = attachment();
        invalid.filename = Some("report.pdf\r\nBcc: attacker@example.test".to_owned());
        assert_eq!(
            compose_rfc822_with_attachments("owner@example.test", &message(), &[invalid]),
            Err(MailContractError::InvalidPayload)
        );

        let oversized = OutboundAttachmentV1 {
            bytes: vec![1; MAX_OUTBOUND_ATTACHMENT_BYTES + 1],
            ..attachment()
        };
        assert_eq!(
            compose_rfc822_with_attachments("owner@example.test", &message(), &[oversized]),
            Err(MailContractError::InvalidPayload)
        );
    }

    #[test]
    fn rejects_duplicate_anchor_and_invalid_media_type_before_rendering() {
        let first = attachment();
        let duplicate = attachment();
        assert_eq!(
            compose_rfc822_with_attachments("owner@example.test", &message(), &[first, duplicate],),
            Err(MailContractError::InvalidPayload)
        );

        let invalid_media_type = OutboundAttachmentV1 {
            anchor_id: [8; 16],
            media_type: "text/plain\r\nBcc: attacker@example.test".to_owned(),
            ..attachment()
        };
        assert_eq!(
            compose_rfc822_with_attachments(
                "owner@example.test",
                &message(),
                &[invalid_media_type],
            ),
            Err(MailContractError::InvalidPayload)
        );
    }
}
