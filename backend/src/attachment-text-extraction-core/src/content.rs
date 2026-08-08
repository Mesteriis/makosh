use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1,
    ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedAttachmentTextV1 {
    pub bytes: Vec<u8>,
    pub extraction_truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextContentErrorV1 {
    Empty,
    InvalidUtf8,
    OutputTooLarge,
    ContainsNul,
}

pub fn normalize_attachment_text_v1(
    bytes: &[u8],
    extraction_truncated: bool,
) -> Result<NormalizedAttachmentTextV1, AttachmentTextContentErrorV1> {
    if bytes.is_empty() {
        return Err(AttachmentTextContentErrorV1::Empty);
    }
    if bytes.len() > ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1 {
        return Err(AttachmentTextContentErrorV1::OutputTooLarge);
    }
    let source =
        std::str::from_utf8(bytes).map_err(|_| AttachmentTextContentErrorV1::InvalidUtf8)?;
    if source.contains('\0') {
        return Err(AttachmentTextContentErrorV1::ContainsNul);
    }
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.is_empty() {
        return Err(AttachmentTextContentErrorV1::Empty);
    }
    Ok(NormalizedAttachmentTextV1 {
        bytes: normalized.into_bytes(),
        extraction_truncated,
    })
}

#[must_use]
pub fn visible_attachment_text_v1(bytes: &[u8]) -> (&[u8], bool) {
    if bytes.len() <= ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1 {
        return (bytes, false);
    }
    let mut boundary = ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1;
    while boundary > 0 && std::str::from_utf8(&bytes[..boundary]).is_err() {
        boundary -= 1;
    }
    (&bytes[..boundary], true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_is_bounded_utf8_and_canonicalizes_line_endings() {
        let result = normalize_attachment_text_v1(b"first\r\nsecond\rthird\n", false)
            .expect("normalized text");
        assert_eq!(result.bytes, b"first\nsecond\nthird\n");
        assert!(!result.extraction_truncated);
        assert_eq!(
            normalize_attachment_text_v1(&[0xff], false),
            Err(AttachmentTextContentErrorV1::InvalidUtf8)
        );
        assert_eq!(
            normalize_attachment_text_v1(b"a\0b", false),
            Err(AttachmentTextContentErrorV1::ContainsNul)
        );
        assert_eq!(
            normalize_attachment_text_v1(
                &vec![b'x'; ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1 + 1],
                false,
            ),
            Err(AttachmentTextContentErrorV1::OutputTooLarge)
        );
    }

    #[test]
    fn visible_slice_never_splits_utf8() {
        let mut bytes = vec![b'a'; ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1 - 1];
        bytes.extend_from_slice("€tail".as_bytes());
        let (visible, truncated) = visible_attachment_text_v1(&bytes);
        assert!(truncated);
        assert!(std::str::from_utf8(visible).is_ok());
        assert_eq!(
            visible.len(),
            ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1 - 1
        );
    }
}
