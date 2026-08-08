//! Bounded RFC822/MIME extraction shared by Mail adapters.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hermes_mail_api::{
    MAX_PLAIN_TEXT_BYTES,
    operational::{
        MAX_OPERATIONAL_ADDRESS_BYTES, MAX_OPERATIONAL_RECIPIENTS, MAX_OPERATIONAL_SNIPPET_BYTES,
        MAX_OPERATIONAL_SUBJECT_BYTES,
    },
};

const MAX_RFC822_BYTES: usize = 16 * 1024 * 1024;
const MAX_MIME_DEPTH: u8 = 8;
const MAX_MIME_PARTS: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentDispositionV1 {
    Attachment,
    Inline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentMetadataV1 {
    pub part_id: u16,
    pub filename: Option<String>,
    pub media_type: String,
    pub declared_bytes: u64,
    pub disposition: AttachmentDispositionV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPartExtractionErrorV1 {
    InvalidSource,
    InvalidPart,
    NotFound,
    InvalidEncoding,
    TooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rfc822OperationalPreviewV1 {
    pub subject: Option<String>,
    pub sender: Option<String>,
    pub recipients: Vec<String>,
    pub snippet: Option<String>,
    pub has_plain_text: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rfc822BodyContentV1 {
    pub media_type: &'static str,
    pub bytes: Vec<u8>,
}

/// Extracts only bounded, display-safe operational fields. Raw MIME and HTML are
/// intentionally excluded from the returned snapshot.
#[must_use]
pub fn operational_preview(raw_message: &[u8]) -> Option<Rfc822OperationalPreviewV1> {
    if raw_message.is_empty() || raw_message.len() > MAX_RFC822_BYTES {
        return None;
    }
    let (headers, _) = split_headers_and_body(raw_message)?;
    let headers = unfolded_headers(headers)?;
    let subject = last_operational_header(&headers, "subject", MAX_OPERATIONAL_SUBJECT_BYTES);
    let sender = last_operational_header(&headers, "from", MAX_OPERATIONAL_ADDRESS_BYTES);
    let recipients = headers
        .iter()
        .filter(|(name, _)| matches!(name.as_str(), "to" | "cc" | "bcc"))
        .filter_map(|(_, value)| operational_text(value, MAX_OPERATIONAL_ADDRESS_BYTES))
        .take(MAX_OPERATIONAL_RECIPIENTS)
        .collect();
    let plaintext = readable_text_body(raw_message);
    let snippet = plaintext
        .as_deref()
        .and_then(|body| std::str::from_utf8(body).ok())
        .and_then(|body| operational_text(body, MAX_OPERATIONAL_SNIPPET_BYTES));
    Some(Rfc822OperationalPreviewV1 {
        subject,
        sender,
        recipients,
        snippet,
        has_plain_text: plaintext.is_some(),
    })
}

fn unfolded_headers(headers: &[u8]) -> Option<Vec<(String, String)>> {
    let text = std::str::from_utf8(headers).ok()?;
    let mut fields = Vec::<(String, String)>::new();
    for line in text.lines() {
        if line.starts_with([' ', '\t']) {
            let (_, value) = fields.last_mut()?;
            if value.len().saturating_add(line.len()).saturating_add(1) > MAX_RFC822_BYTES {
                return None;
            }
            value.push(' ');
            value.push_str(line.trim());
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            continue;
        }
        fields.push((name, value.trim().to_owned()));
    }
    Some(fields)
}

fn last_operational_header(
    headers: &[(String, String)],
    name: &str,
    max_bytes: usize,
) -> Option<String> {
    headers
        .iter()
        .rev()
        .find(|(header_name, _)| header_name == name)
        .and_then(|(_, value)| operational_text(&decode_rfc2047_words(value), max_bytes))
}

fn operational_text(value: &str, max_bytes: usize) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() || normalized.contains('\0') {
        return None;
    }
    if normalized.len() <= max_bytes {
        return Some(normalized);
    }
    let mut end = max_bytes;
    while !normalized.is_char_boundary(end) {
        end -= 1;
    }
    let truncated = normalized[..end].trim_end();
    (!truncated.is_empty()).then(|| truncated.to_owned())
}

/// Extracts the first bounded `text/plain` MIME leaf that is not an attachment.
/// Malformed, oversized and unsupported encodings are rejected rather than letting
/// raw RFC822 or attachment bytes enter the Communications body pipeline.
pub fn direct_plain_text_body(raw_message: &[u8]) -> Option<Vec<u8>> {
    if raw_message.is_empty() || raw_message.len() > MAX_RFC822_BYTES {
        return None;
    }
    let (headers, body) = split_headers_and_body(raw_message)?;
    extract_plain_text_leaf(headers, body, 0, &mut 0)
}

/// Extracts bounded readable text for the provider experience. A real
/// `text/plain` leaf remains authoritative; HTML-only messages fall back to
/// visible text extracted from the first non-attachment `text/html` leaf.
/// Raw MIME, markup and transfer-encoded bytes are never returned.
#[must_use]
pub fn readable_text_body(raw_message: &[u8]) -> Option<Vec<u8>> {
    direct_plain_text_body(raw_message)
        .or_else(|| {
            let (headers, body) = split_headers_and_body(raw_message)?;
            extract_html_text_leaf(headers, body, 0, &mut 0)
        })
        .or_else(|| loose_mime_text_leaf(raw_message, "text/plain"))
        .or_else(|| loose_mime_text_leaf(raw_message, "text/html"))
}

/// Extracts the reference-view body without collapsing HTML markup. The caller
/// must preserve the media type and the client must sanitize HTML before render.
#[must_use]
pub fn readable_body_content(raw_message: &[u8]) -> Option<Rfc822BodyContentV1> {
    if raw_message.is_empty() || raw_message.len() > MAX_RFC822_BYTES {
        return None;
    }
    let (headers, body) = split_headers_and_body(raw_message)?;
    extract_html_leaf(headers, body, 0, &mut 0)
        .or_else(|| loose_mime_body_leaf(raw_message, "text/html"))
        .map(|bytes| Rfc822BodyContentV1 {
            media_type: "text/html",
            bytes,
        })
        .or_else(|| {
            direct_plain_text_body(raw_message)
                .or_else(|| loose_mime_body_leaf(raw_message, "text/plain"))
                .map(|bytes| Rfc822BodyContentV1 {
                    media_type: "text/plain",
                    bytes,
                })
        })
}

fn loose_mime_text_leaf(raw_message: &[u8], expected_media_type: &str) -> Option<Vec<u8>> {
    loose_mime_leaf(raw_message, expected_media_type, true)
}

fn loose_mime_body_leaf(raw_message: &[u8], expected_media_type: &str) -> Option<Vec<u8>> {
    loose_mime_leaf(raw_message, expected_media_type, false)
}

fn loose_mime_leaf(
    raw_message: &[u8],
    expected_media_type: &str,
    html_as_visible_text: bool,
) -> Option<Vec<u8>> {
    if raw_message.is_empty() || raw_message.len() > MAX_RFC822_BYTES {
        return None;
    }
    let mut offset = 0_usize;
    while offset < raw_message.len() {
        let relative_end = raw_message[offset..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(raw_message.len() - offset, |position| position + 1);
        let next = offset.saturating_add(relative_end);
        let line = &raw_message[offset..next];
        let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
        let normalized = without_lf
            .strip_suffix(b"\r")
            .unwrap_or(without_lf)
            .trim_ascii_end();
        if normalized.len() >= 4 && normalized.starts_with(b"--") {
            let boundary = normalized.strip_suffix(b"--").unwrap_or(normalized);
            if let Some(decoded) = decode_loose_mime_candidate(
                &raw_message[next..],
                boundary,
                expected_media_type,
                html_as_visible_text,
            ) {
                return Some(decoded);
            }
        }
        offset = next;
    }
    None
}

fn decode_loose_mime_candidate(
    candidate: &[u8],
    boundary: &[u8],
    expected_media_type: &str,
    html_as_visible_text: bool,
) -> Option<Vec<u8>> {
    let (headers, unbounded_body) = split_headers_and_body(candidate)?;
    let content_type = header_value(headers, "content-type")?;
    let media_type = content_type.split(';').next()?.trim();
    if !media_type.eq_ignore_ascii_case(expected_media_type) || is_attachment(headers) {
        return None;
    }
    let body = mime_body_before_boundary(unbounded_body, boundary);
    let decoded = decode_transfer_encoding(
        body,
        header_value(headers, "content-transfer-encoding").unwrap_or_default(),
    )?;
    let charset = header_parameter(headers, "content-type", "charset");
    let decoded = decode_text_bytes(&decoded, charset.as_deref());
    if expected_media_type == "text/html" && html_as_visible_text {
        valid_plaintext(visible_html_text(&decoded).into_bytes())
    } else {
        valid_plaintext(decoded.into_bytes())
    }
}

fn mime_body_before_boundary<'a>(body: &'a [u8], boundary: &[u8]) -> &'a [u8] {
    let mut offset = 0_usize;
    while offset < body.len() {
        let relative_end = body[offset..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(body.len() - offset, |position| position + 1);
        let next = offset.saturating_add(relative_end);
        let line = &body[offset..next];
        let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
        let normalized = without_lf
            .strip_suffix(b"\r")
            .unwrap_or(without_lf)
            .trim_ascii_end();
        if normalized == boundary
            || (normalized.starts_with(boundary) && normalized.get(boundary.len()..) == Some(b"--"))
        {
            return &body[..offset];
        }
        offset = next;
    }
    body
}

fn extract_plain_text_leaf(
    headers: &[u8],
    body: &[u8],
    depth: u8,
    parts: &mut usize,
) -> Option<Vec<u8>> {
    if depth > MAX_MIME_DEPTH || *parts >= MAX_MIME_PARTS || is_attachment(headers) {
        return None;
    }
    let content_type =
        header_value(headers, "content-type").unwrap_or_else(|| "text/plain".to_owned());
    let media_type = content_type.split(';').next()?.trim().to_ascii_lowercase();
    if media_type.starts_with("multipart/") {
        let boundary = header_parameter(headers, "content-type", "boundary")?;
        for part in multipart_parts(body, &boundary)? {
            *parts += 1;
            let Some((part_headers, part_body)) = split_headers_and_body(&part) else {
                continue;
            };
            if let Some(plaintext) =
                extract_plain_text_leaf(part_headers, part_body, depth + 1, parts)
            {
                return Some(plaintext);
            }
        }
        return None;
    }
    if media_type != "text/plain" {
        return None;
    }
    let decoded = decode_transfer_encoding(
        body,
        header_value(headers, "content-transfer-encoding").unwrap_or_default(),
    )?;
    let charset = header_parameter(headers, "content-type", "charset");
    valid_plaintext(decode_text_bytes(&decoded, charset.as_deref()).into_bytes())
}

fn extract_html_text_leaf(
    headers: &[u8],
    body: &[u8],
    depth: u8,
    parts: &mut usize,
) -> Option<Vec<u8>> {
    if depth > MAX_MIME_DEPTH || *parts >= MAX_MIME_PARTS || is_attachment(headers) {
        return None;
    }
    let content_type =
        header_value(headers, "content-type").unwrap_or_else(|| "text/plain".to_owned());
    let media_type = content_type.split(';').next()?.trim().to_ascii_lowercase();
    if media_type.starts_with("multipart/") {
        let boundary = header_parameter(headers, "content-type", "boundary")?;
        for part in multipart_parts(body, &boundary)? {
            *parts += 1;
            let Some((part_headers, part_body)) = split_headers_and_body(&part) else {
                continue;
            };
            if let Some(text) = extract_html_text_leaf(part_headers, part_body, depth + 1, parts) {
                return Some(text);
            }
        }
        return None;
    }
    if media_type != "text/html" {
        return None;
    }
    let decoded = decode_transfer_encoding(
        body,
        header_value(headers, "content-transfer-encoding").unwrap_or_default(),
    )?;
    let charset = header_parameter(headers, "content-type", "charset");
    let html = decode_text_bytes(&decoded, charset.as_deref());
    let text = visible_html_text(&html);
    valid_plaintext(text.into_bytes())
}

fn extract_html_leaf(headers: &[u8], body: &[u8], depth: u8, parts: &mut usize) -> Option<Vec<u8>> {
    if depth > MAX_MIME_DEPTH || *parts >= MAX_MIME_PARTS || is_attachment(headers) {
        return None;
    }
    let content_type =
        header_value(headers, "content-type").unwrap_or_else(|| "text/plain".to_owned());
    let media_type = content_type.split(';').next()?.trim().to_ascii_lowercase();
    if media_type.starts_with("multipart/") {
        let boundary = header_parameter(headers, "content-type", "boundary")?;
        for part in multipart_parts(body, &boundary)? {
            *parts += 1;
            let Some((part_headers, part_body)) = split_headers_and_body(&part) else {
                continue;
            };
            if let Some(html) = extract_html_leaf(part_headers, part_body, depth + 1, parts) {
                return Some(html);
            }
        }
        return None;
    }
    if media_type != "text/html" {
        return None;
    }
    let decoded = decode_transfer_encoding(
        body,
        header_value(headers, "content-transfer-encoding").unwrap_or_default(),
    )?;
    let charset = header_parameter(headers, "content-type", "charset");
    valid_plaintext(decode_text_bytes(&decoded, charset.as_deref()).into_bytes())
}

fn is_attachment(headers: &[u8]) -> bool {
    let disposition = header_value(headers, "content-disposition").unwrap_or_default();
    disposition
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("attachment"))
        || header_parameter(headers, "content-type", "name").is_some()
        || header_parameter(headers, "content-disposition", "filename").is_some()
}

fn multipart_parts(body: &[u8], boundary: &str) -> Option<Vec<Vec<u8>>> {
    if boundary.is_empty() || boundary.len() > 200 || !boundary.is_ascii() {
        return None;
    }
    let marker = format!("--{boundary}");
    let closing = format!("{marker}--");
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut inside_part = false;
    for line in body.split_inclusive(|byte| *byte == b'\n') {
        let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
        let normalized = without_lf.strip_suffix(b"\r").unwrap_or(without_lf);
        let normalized = normalized.trim_ascii_end();
        if normalized == marker.as_bytes() || normalized == closing.as_bytes() {
            if inside_part && !current.is_empty() {
                if parts.len() >= MAX_MIME_PARTS {
                    return None;
                }
                parts.push(std::mem::take(&mut current));
            }
            if normalized == closing.as_bytes() {
                return Some(parts);
            }
            inside_part = true;
        } else if inside_part {
            if current.len().saturating_add(line.len()) > MAX_RFC822_BYTES {
                return None;
            }
            current.extend_from_slice(line);
        }
    }
    if inside_part && !current.is_empty() {
        if parts.len() >= MAX_MIME_PARTS {
            return None;
        }
        parts.push(current);
    }
    (!parts.is_empty()).then_some(parts)
}

fn decode_transfer_encoding(body: &[u8], encoding: String) -> Option<Vec<u8>> {
    match encoding.trim().to_ascii_lowercase().as_str() {
        "" | "7bit" | "8bit" | "binary" => {
            (body.len() <= MAX_PLAIN_TEXT_BYTES).then(|| body.to_vec())
        }
        "base64" => decode_base64(body),
        "quoted-printable" => decode_quoted_printable(body),
        _ => None,
    }
}

fn decode_base64(body: &[u8]) -> Option<Vec<u8>> {
    let compact = body
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if compact.is_empty() || compact.len() > MAX_PLAIN_TEXT_BYTES.saturating_mul(2) {
        return None;
    }
    let decoded = STANDARD.decode(compact).ok()?;
    (decoded.len() <= MAX_PLAIN_TEXT_BYTES).then_some(decoded)
}

fn decode_quoted_printable(body: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(body.len().min(MAX_PLAIN_TEXT_BYTES));
    let mut index = 0;
    while index < body.len() {
        if body[index] != b'=' {
            decoded.push(body[index]);
            index += 1;
        } else if body.get(index + 1) == Some(&b'\r') && body.get(index + 2) == Some(&b'\n') {
            index += 3;
        } else if body.get(index + 1) == Some(&b'\n') {
            index += 2;
        } else {
            let value = body
                .get(index + 1)
                .and_then(|high| hex_value(*high))
                .zip(body.get(index + 2).and_then(|low| hex_value(*low)));
            if let Some((high, low)) = value {
                decoded.push((high << 4) | low);
                index += 3;
            } else {
                decoded.push(b'=');
                index += 1;
            }
        }
        if decoded.len() > MAX_PLAIN_TEXT_BYTES {
            return None;
        }
    }
    Some(decoded)
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn decode_rfc2047_words(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    let mut decoded_previous_word = false;
    while let Some(start) = rest.find("=?") {
        let prefix = &rest[..start];
        if !(decoded_previous_word && prefix.chars().all(char::is_whitespace)) {
            output.push_str(prefix);
        }
        let encoded_word = &rest[start + 2..];
        let Some(charset_end) = encoded_word.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let charset = &encoded_word[..charset_end];
        let after_charset = &encoded_word[charset_end + 1..];
        let Some(encoding_end) = after_charset.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoding = &after_charset[..encoding_end];
        let encoded = &after_charset[encoding_end + 1..];
        let Some(encoded_end) = encoded.find("?=") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let payload = &encoded[..encoded_end];
        let bytes = match encoding.to_ascii_lowercase().as_str() {
            "b" => STANDARD.decode(payload).ok(),
            "q" => decode_rfc2047_q(payload),
            _ => None,
        };
        if let Some(bytes) = bytes {
            output.push_str(&decode_text_bytes(&bytes, Some(charset)));
            decoded_previous_word = true;
        } else {
            output.push_str(&rest[start..start + 2]);
            decoded_previous_word = false;
        }
        rest = &encoded[encoded_end + 2..];
    }
    output.push_str(rest);
    output
}

fn decode_rfc2047_q(input: &str) -> Option<Vec<u8>> {
    decode_quoted_printable(input.replace('_', " ").as_bytes())
}

fn decode_text_bytes(bytes: &[u8], charset: Option<&str>) -> String {
    match charset
        .map(|label| label.trim_matches([' ', '"']).to_ascii_lowercase())
        .as_deref()
    {
        Some("windows-1251" | "cp1251") => decode_windows_1251(bytes),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn decode_windows_1251(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| match byte {
            0x00..=0x7f => char::from(*byte),
            0xa8 => 'Ё',
            0xb8 => 'ё',
            0xc0..=0xdf => char::from_u32(u32::from(*byte) - 0xc0 + 0x0410)
                .expect("Cyrillic upper-case range is valid"),
            0xe0..=0xff => char::from_u32(u32::from(*byte) - 0xe0 + 0x0430)
                .expect("Cyrillic lower-case range is valid"),
            _ => '�',
        })
        .collect()
}

fn visible_html_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len().min(MAX_PLAIN_TEXT_BYTES));
    let mut inside_tag = false;
    let mut previous_space = false;
    for character in html.chars() {
        match character {
            '<' => {
                inside_tag = true;
                if !previous_space {
                    text.push(' ');
                    previous_space = true;
                }
            }
            '>' => inside_tag = false,
            _ if inside_tag => {}
            _ if character.is_whitespace() => {
                if !previous_space {
                    text.push(' ');
                    previous_space = true;
                }
            }
            _ => {
                text.push(character);
                previous_space = false;
            }
        }
        if text.len() > MAX_PLAIN_TEXT_BYTES {
            return String::new();
        }
    }
    text.trim().to_owned()
}

fn valid_plaintext(mut body: Vec<u8>) -> Option<Vec<u8>> {
    while matches!(body.last(), Some(b'\r' | b'\n')) {
        body.pop();
    }
    (!body.is_empty() && body.len() <= MAX_PLAIN_TEXT_BYTES && std::str::from_utf8(&body).is_ok())
        .then_some(body)
}

pub fn attachment_metadata(raw_message: &[u8]) -> Vec<AttachmentMetadataV1> {
    let Some((headers, body)) = split_headers_and_body(raw_message) else {
        return Vec::new();
    };
    let Some(boundary) = header_parameter(headers, "content-type", "boundary") else {
        return Vec::new();
    };
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if !content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/")
    {
        return Vec::new();
    }
    let mut attachments = Vec::new();
    let mut next_part_id = 1_u16;
    visit_multipart_attachments(
        body,
        &boundary,
        0,
        &mut next_part_id,
        &mut |metadata, _, _| attachments.push(metadata),
    );
    attachments
}

pub fn extract_attachment_part(
    raw_message: &[u8],
    part_id: u16,
) -> Result<Vec<u8>, AttachmentPartExtractionErrorV1> {
    if raw_message.is_empty() || raw_message.len() > MAX_RFC822_BYTES {
        return Err(AttachmentPartExtractionErrorV1::InvalidSource);
    }
    if part_id == 0 {
        return Err(AttachmentPartExtractionErrorV1::InvalidPart);
    }
    let (headers, body) = split_headers_and_body(raw_message)
        .ok_or(AttachmentPartExtractionErrorV1::InvalidSource)?;
    let boundary = header_parameter(headers, "content-type", "boundary")
        .ok_or(AttachmentPartExtractionErrorV1::InvalidSource)?;
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if !content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/")
    {
        return Err(AttachmentPartExtractionErrorV1::InvalidSource);
    }
    let mut next_part_id = 1_u16;
    let mut result = None;
    visit_multipart_attachments(
        body,
        &boundary,
        0,
        &mut next_part_id,
        &mut |metadata, body, encoding| {
            if metadata.part_id == part_id {
                result = Some(decode_attachment_part(body, encoding.as_deref()));
            }
        },
    );
    result.ok_or(AttachmentPartExtractionErrorV1::NotFound)?
}

fn visit_multipart_attachments<F>(
    body: &[u8],
    boundary: &str,
    depth: u8,
    next_part_id: &mut u16,
    visitor: &mut F,
) where
    F: FnMut(AttachmentMetadataV1, &[u8], Option<String>),
{
    if depth >= MAX_MIME_DEPTH
        || boundary.is_empty()
        || boundary.len() > 200
        || !boundary.is_ascii()
    {
        return;
    }
    let marker = format!("--{boundary}");
    let closing_marker = format!("{marker}--");
    let mut current = Vec::new();
    let mut inside_part = false;
    for line in body.split(|byte| *byte == b'\n') {
        let normalized = line.strip_suffix(b"\r").unwrap_or(line);
        if normalized == marker.as_bytes() || normalized == closing_marker.as_bytes() {
            if inside_part {
                visit_attachment_from_part(&current, depth, next_part_id, visitor);
                current.clear();
            }
            if normalized == closing_marker.as_bytes() {
                break;
            }
            inside_part = true;
        } else if inside_part {
            current.extend_from_slice(line);
            current.push(b'\n');
        }
    }
}

fn visit_attachment_from_part<F>(part: &[u8], depth: u8, next_part_id: &mut u16, visitor: &mut F)
where
    F: FnMut(AttachmentMetadataV1, &[u8], Option<String>),
{
    let Some((headers, body)) = split_headers_and_body(part) else {
        return;
    };
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/")
    {
        if let Some(boundary) = header_parameter(headers, "content-type", "boundary") {
            visit_multipart_attachments(body, &boundary, depth + 1, next_part_id, visitor);
        }
        return;
    }
    let Some((metadata, transfer_encoding)) =
        attachment_metadata_from_part(headers, body, next_part_id)
    else {
        return;
    };
    visitor(metadata, body, transfer_encoding);
}

fn attachment_metadata_from_part(
    headers: &[u8],
    body: &[u8],
    next_part_id: &mut u16,
) -> Option<(AttachmentMetadataV1, Option<String>)> {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    let disposition = header_value(headers, "content-disposition")?;
    let disposition = match disposition
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "attachment" => AttachmentDispositionV1::Attachment,
        "inline" => AttachmentDispositionV1::Inline,
        _ => return None,
    };
    let media_type = content_type
        .split(';')
        .next()
        .map(str::trim)
        .filter(valid_media_type)?;
    let transfer_encoding = header_value(headers, "content-transfer-encoding");
    let declared_bytes = decoded_part_size(body, transfer_encoding.clone())?;
    let filename = header_parameter(headers, "content-disposition", "filename")
        .or_else(|| header_parameter(headers, "content-type", "name"))
        .filter(|value| !value.is_empty() && value.len() <= 512 && value.is_ascii());
    let part_id = *next_part_id;
    let next = next_part_id.checked_add(1)?;
    *next_part_id = next;
    Some((
        AttachmentMetadataV1 {
            part_id,
            filename,
            media_type: media_type.to_owned(),
            declared_bytes,
            disposition,
        },
        transfer_encoding,
    ))
}

fn decode_attachment_part(
    body: &[u8],
    transfer_encoding: Option<&str>,
) -> Result<Vec<u8>, AttachmentPartExtractionErrorV1> {
    let body = body
        .strip_suffix(b"\n")
        .unwrap_or(body)
        .strip_suffix(b"\r")
        .unwrap_or(body);
    match transfer_encoding
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        None | Some("") | Some("7bit") | Some("8bit") | Some("binary") => (body.len()
            <= MAX_RFC822_BYTES)
            .then(|| body.to_vec())
            .ok_or(AttachmentPartExtractionErrorV1::TooLarge),
        Some("base64") => {
            let compact = body
                .iter()
                .copied()
                .filter(|byte| !byte.is_ascii_whitespace())
                .collect::<Vec<_>>();
            if compact.is_empty() || compact.len() > MAX_RFC822_BYTES {
                return Err(AttachmentPartExtractionErrorV1::TooLarge);
            }
            let decoded = STANDARD
                .decode(compact)
                .map_err(|_| AttachmentPartExtractionErrorV1::InvalidEncoding)?;
            (decoded.len() <= MAX_RFC822_BYTES)
                .then_some(decoded)
                .ok_or(AttachmentPartExtractionErrorV1::TooLarge)
        }
        _ => Err(AttachmentPartExtractionErrorV1::InvalidEncoding),
    }
}

fn split_headers_and_body(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| (&bytes[..index], &bytes[index + 4..]))
        .or_else(|| {
            bytes
                .windows(2)
                .position(|window| window == b"\n\n")
                .map(|index| (&bytes[..index], &bytes[index + 2..]))
        })
}

fn header_value(headers: &[u8], name: &str) -> Option<String> {
    unfolded_headers(headers)?
        .into_iter()
        .rev()
        .find_map(|(key, value)| key.eq_ignore_ascii_case(name).then_some(value))
}

fn header_parameter(headers: &[u8], header: &str, parameter: &str) -> Option<String> {
    header_value(headers, header)?
        .split(';')
        .skip(1)
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case(parameter)
                .then(|| value.trim().trim_matches('"').to_owned())
        })
}

fn valid_media_type(value: &&str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.is_ascii()
        && value.contains('/')
        && !value.contains(char::is_whitespace)
}

fn decoded_part_size(body: &[u8], transfer_encoding: Option<String>) -> Option<u64> {
    let body = body
        .strip_suffix(b"\n")
        .unwrap_or(body)
        .strip_suffix(b"\r")
        .unwrap_or(body);
    match transfer_encoding
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        None | Some("") | Some("7bit") | Some("8bit") | Some("binary") => {
            u64::try_from(body.len()).ok()
        }
        Some("base64") => base64_decoded_size(body),
        _ => None,
    }
}

fn base64_decoded_size(body: &[u8]) -> Option<u64> {
    let bytes = body
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if padding > 2
        || bytes[..bytes.len() - padding]
            .iter()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(*byte, b'+' | b'/'))
        || bytes[..bytes.len() - padding].contains(&b'=')
    {
        return None;
    }
    u64::try_from((bytes.len() / 4) * 3 - padding).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_bounded_attachment_metadata() {
        let raw = b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\n\r\nbody\r\n--x\r\nContent-Type: application/pdf; name=a.pdf\r\nContent-Disposition: attachment; filename=a.pdf\r\nContent-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n--x--\r\n";
        assert_eq!(attachment_metadata(raw)[0].declared_bytes, 5);
    }

    #[test]
    fn extracts_only_bounded_operational_preview_fields() {
        let raw = b"From: Sender <sender@example.test>\r\n\
Subject:  A folded\r\n\t subject  \r\n\
To: Owner <owner@example.test>\r\n\
Cc: Team <team@example.test>\r\n\
Content-Type: text/plain\r\n\r\n\
line one\r\nline two\r\n";

        assert_eq!(
            operational_preview(raw),
            Some(Rfc822OperationalPreviewV1 {
                subject: Some("A folded subject".to_owned()),
                sender: Some("Sender <sender@example.test>".to_owned()),
                recipients: vec![
                    "Owner <owner@example.test>".to_owned(),
                    "Team <team@example.test>".to_owned(),
                ],
                snippet: Some("line one line two".to_owned()),
                has_plain_text: true,
            })
        );
    }

    #[test]
    fn extracts_the_same_base64_part_identified_by_metadata() {
        let raw = b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\n\r\nbody\r\n--x\r\nContent-Type: application/pdf; name=a.pdf\r\nContent-Disposition: attachment; filename=a.pdf\r\nContent-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n--x--\r\n";
        let metadata = attachment_metadata(raw);

        assert_eq!(metadata.len(), 1);
        assert_eq!(
            extract_attachment_part(raw, metadata[0].part_id),
            Ok(b"hello".to_vec())
        );
    }

    #[test]
    fn rejects_unknown_or_unsupported_attachment_parts() {
        let raw = b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\nContent-Type: application/pdf\r\nContent-Disposition: attachment\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nhello=20world\r\n--x--\r\n";

        assert_eq!(
            extract_attachment_part(raw, 1),
            Err(AttachmentPartExtractionErrorV1::NotFound)
        );
        assert_eq!(
            extract_attachment_part(raw, 0),
            Err(AttachmentPartExtractionErrorV1::InvalidPart)
        );
    }

    #[test]
    fn extracts_base64_plain_text_from_multipart() {
        let message = b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\nContent-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n--x\r\nContent-Type: application/pdf\r\nContent-Disposition: attachment\r\n\r\nnot-content\r\n--x--\r\n";
        assert_eq!(direct_plain_text_body(message), Some(b"hello".to_vec()));
    }

    #[test]
    fn decodes_quoted_printable_and_skips_attachment_text() {
        let message = b"Content-Type: multipart/alternative; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\nContent-Disposition: attachment\r\n\r\nignore\r\n--x\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nhello=20world=21\r\n--x--\r\n";
        assert_eq!(
            direct_plain_text_body(message),
            Some(b"hello world!".to_vec())
        );
    }

    #[test]
    fn decodes_rfc2047_headers_and_html_only_message_text() {
        let message = b"From: =?UTF-8?B?0J/RgNC40LLQtdGC?= <sender@example.test>\r\n\
Subject: =?UTF-8?B?0J/RgNC40LLQtdGC?=\r\n\
To: Owner <owner@example.test>\r\n\
Content-Type: multipart/alternative; boundary=x\r\n\r\n\
--x\r\nContent-Type: text/html; charset=utf-8\r\n\
Content-Transfer-Encoding: quoted-printable\r\n\r\n\
<html><body><h1>Hello</h1><p>real mail</p></body></html>\r\n\
--x--\r\n";

        let preview = operational_preview(message).expect("valid preview");
        assert_eq!(preview.subject.as_deref(), Some("Привет"));
        assert_eq!(
            preview.sender.as_deref(),
            Some("Привет <sender@example.test>")
        );
        assert_eq!(preview.snippet.as_deref(), Some("Hello real mail"));
        assert!(preview.has_plain_text);
        assert_eq!(
            readable_text_body(message),
            Some(b"Hello real mail".to_vec())
        );
        assert_eq!(direct_plain_text_body(message), None);
        assert_eq!(
            readable_body_content(message),
            Some(Rfc822BodyContentV1 {
                media_type: "text/html",
                bytes: b"<html><body><h1>Hello</h1><p>real mail</p></body></html>".to_vec(),
            })
        );
    }

    #[test]
    fn unfolds_mime_parameters_before_resolving_a_multipart_boundary() {
        let message = b"Content-Type: multipart/alternative;\r\n\
\tboundary=\"----=_Part_4618394_53314671.1755262501399\"\r\n\r\n\
------=_Part_4618394_53314671.1755262501399\r\n\
Content-Type: text/html;\r\n\
\tcharset=utf-8\r\n\
Content-Transfer-Encoding: quoted-printable\r\n\r\n\
<div style=3D\"display:none\"><span>Readable body</span></div>\r\n\
------=_Part_4618394_53314671.1755262501399--\r\n";

        let (headers, body) = split_headers_and_body(message).expect("top-level MIME body");
        let boundary = header_parameter(headers, "content-type", "boundary")
            .expect("folded boundary parameter");
        assert_eq!(boundary, "----=_Part_4618394_53314671.1755262501399");
        assert_eq!(
            multipart_parts(body, &boundary)
                .expect("closed multipart body")
                .len(),
            1
        );
        assert_eq!(readable_text_body(message), Some(b"Readable body".to_vec()));
    }

    #[test]
    fn reads_bounded_vendor_html_with_an_implicit_multipart_end_and_literal_equals() {
        let message = b"Content-Type: multipart/alternative; boundary=x\r\n\r\n\
--x \t\r\n\
Content-Type: text/html; charset=utf-8\r\n\
Content-Transfer-Encoding: quoted-printable\r\n\r\n\
<p>one=two</p>\r\n";

        assert_eq!(readable_text_body(message), Some(b"one=two".to_vec()));
    }

    #[test]
    fn recovers_vendor_html_leaf_when_outer_boundary_parameter_is_missing() {
        let message = b"Subject: vendor\r\n\
Content-Type: multipart/alternative\r\n\r\n\
------=_Part_4618394_53314671.1755262501399\r\n\
Content-ID: <vendor-body>\r\n\
Content-Type: text/html;charset=utf-8\r\n\
Content-Transfer-Encoding: quoted-printable\r\n\r\n\
<html><body><h1>Order</h1><p>Readable=20mail</p></body></html>\r\n\
------=_Part_4618394_53314671.1755262501399--\r\n";

        assert_eq!(
            readable_text_body(message),
            Some(b"Order Readable mail".to_vec())
        );
    }

    #[test]
    fn transcodes_declared_legacy_plain_text_charset_to_utf8() {
        let message = b"Content-Type: text/plain; charset=windows-1251\r\n\
Content-Transfer-Encoding: 8bit\r\n\r\n\
\xcf\xf0\xe8\xe2\xe5\xf2";

        assert_eq!(
            direct_plain_text_body(message),
            Some("Привет".as_bytes().to_vec())
        );
    }
}
