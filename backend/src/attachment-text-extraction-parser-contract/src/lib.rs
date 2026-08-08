#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-text-extraction-parser-contract";
pub const ATTACHMENT_TEXT_PARSER_MAX_SOURCE_BYTES_V1: usize = 100 * 1024 * 1024;
pub const ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextParserKindV1 {
    PlainUtf8,
    Pdf,
    Docx,
    Ocr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextParserErrorV1 {
    EmptySource,
    SourceTooLarge,
    UnsupportedFormat,
    InvalidContent,
    EncryptedContent,
    OutputTooLarge,
    ParserUnavailable,
    ParserTimedOut,
    ParserFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextParserOutputV1 {
    pub parser: AttachmentTextParserKindV1,
    pub text_utf8: Vec<u8>,
    pub extraction_truncated: bool,
}

pub fn detect_attachment_text_parser_v1(
    source: &[u8],
) -> Result<AttachmentTextParserKindV1, AttachmentTextParserErrorV1> {
    validate_source_bound(source)?;
    if source.starts_with(b"%PDF-") {
        return Ok(AttachmentTextParserKindV1::Pdf);
    }
    if source.starts_with(b"PK\x03\x04") && contains_docx_marker(source) {
        return Ok(AttachmentTextParserKindV1::Docx);
    }
    if is_supported_image(source) {
        return Ok(AttachmentTextParserKindV1::Ocr);
    }
    if std::str::from_utf8(source).is_ok() && !source.contains(&0) {
        return Ok(AttachmentTextParserKindV1::PlainUtf8);
    }
    Err(AttachmentTextParserErrorV1::UnsupportedFormat)
}

pub fn validate_source_bound(source: &[u8]) -> Result<(), AttachmentTextParserErrorV1> {
    if source.is_empty() {
        return Err(AttachmentTextParserErrorV1::EmptySource);
    }
    if source.len() > ATTACHMENT_TEXT_PARSER_MAX_SOURCE_BYTES_V1 {
        return Err(AttachmentTextParserErrorV1::SourceTooLarge);
    }
    Ok(())
}

pub fn bounded_parser_output_v1(
    parser: AttachmentTextParserKindV1,
    text: impl AsRef<[u8]>,
    extraction_truncated: bool,
) -> Result<AttachmentTextParserOutputV1, AttachmentTextParserErrorV1> {
    let text = text.as_ref();
    if text.is_empty() || std::str::from_utf8(text).is_err() || text.contains(&0) {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    if text.len() > ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1 {
        return Err(AttachmentTextParserErrorV1::OutputTooLarge);
    }
    Ok(AttachmentTextParserOutputV1 {
        parser,
        text_utf8: text.to_vec(),
        extraction_truncated,
    })
}

fn contains_docx_marker(source: &[u8]) -> bool {
    source
        .windows(b"word/document.xml".len())
        .any(|window| window == b"word/document.xml")
}

fn is_supported_image(source: &[u8]) -> bool {
    source.starts_with(b"\x89PNG\r\n\x1a\n")
        || source.starts_with(b"\xff\xd8\xff")
        || source.starts_with(b"II*\0")
        || source.starts_with(b"MM\0*")
        || source.starts_with(b"BM")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_uses_bytes_not_caller_metadata() {
        assert_eq!(
            detect_attachment_text_parser_v1(b"%PDF-1.7\n"),
            Ok(AttachmentTextParserKindV1::Pdf)
        );
        assert_eq!(
            detect_attachment_text_parser_v1(b"plain text"),
            Ok(AttachmentTextParserKindV1::PlainUtf8)
        );
        assert_eq!(
            detect_attachment_text_parser_v1(b"\x89PNG\r\n\x1a\nrest"),
            Ok(AttachmentTextParserKindV1::Ocr)
        );
        let mut docx = b"PK\x03\x04".to_vec();
        docx.extend_from_slice(b"word/document.xml");
        assert_eq!(
            detect_attachment_text_parser_v1(&docx),
            Ok(AttachmentTextParserKindV1::Docx)
        );
    }

    #[test]
    fn output_rejects_binary_empty_and_over_limit_content() {
        assert_eq!(
            bounded_parser_output_v1(AttachmentTextParserKindV1::PlainUtf8, [], false),
            Err(AttachmentTextParserErrorV1::InvalidContent)
        );
        assert_eq!(
            bounded_parser_output_v1(
                AttachmentTextParserKindV1::PlainUtf8,
                vec![b'x'; ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1 + 1],
                false,
            ),
            Err(AttachmentTextParserErrorV1::OutputTooLarge)
        );
    }
}
