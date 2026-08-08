#![forbid(unsafe_code)]

use makosh_attachment_text_extraction_parser_contract::{
    AttachmentTextParserErrorV1, AttachmentTextParserKindV1, AttachmentTextParserOutputV1,
    bounded_parser_output_v1, validate_source_bound,
};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-plain";

pub fn extract_plain_text_v1(
    source: &[u8],
) -> Result<AttachmentTextParserOutputV1, AttachmentTextParserErrorV1> {
    validate_source_bound(source)?;
    let text =
        std::str::from_utf8(source).map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
    if text.contains('\0') {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    bounded_parser_output_v1(AttachmentTextParserKindV1::PlainUtf8, normalized, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_bounded_utf8_without_interpreting_a_declared_type() {
        let output = extract_plain_text_v1(b"name,value\r\nmakosh,1\r").expect("plain text");
        assert_eq!(output.parser, AttachmentTextParserKindV1::PlainUtf8);
        assert_eq!(output.text_utf8, b"name,value\nmakosh,1\n");
        assert!(!output.extraction_truncated);
    }

    #[test]
    fn binary_content_fails_closed() {
        assert_eq!(
            extract_plain_text_v1(&[0xff, 0xfe]),
            Err(AttachmentTextParserErrorV1::InvalidContent)
        );
        assert_eq!(
            extract_plain_text_v1(b"before\0after"),
            Err(AttachmentTextParserErrorV1::InvalidContent)
        );
    }
}
