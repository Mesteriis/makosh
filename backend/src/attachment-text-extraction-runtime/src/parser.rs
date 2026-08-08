use makosh_attachment_text_extraction_core::{
    AttachmentTextFormatV1, normalize_attachment_text_v1,
};
use makosh_attachment_text_extraction_docx::extract_docx_text_v1;
use makosh_attachment_text_extraction_ocr::{TesseractOcrConfigurationV1, extract_image_text_v1};
use makosh_attachment_text_extraction_parser_contract::{
    AttachmentTextParserErrorV1, AttachmentTextParserKindV1, detect_attachment_text_parser_v1,
};
use makosh_attachment_text_extraction_pdf::extract_pdf_text_v1;
use makosh_attachment_text_extraction_plain::extract_plain_text_v1;
use sha2::{Digest, Sha256};

pub struct AttachmentTextExtractionParserRuntimeV1 {
    ocr: Option<TesseractOcrConfigurationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextRuntimeParseResultV1 {
    pub text_utf8: Vec<u8>,
    pub format: AttachmentTextFormatV1,
    pub extraction_truncated: bool,
    pub parser_identity_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextRuntimeParseErrorV1 {
    Unsupported,
    SourceTooLarge,
    InvalidContent,
    ParserUnavailable,
    ParserFailed,
}

impl AttachmentTextExtractionParserRuntimeV1 {
    #[must_use]
    pub const fn new(ocr: Option<TesseractOcrConfigurationV1>) -> Self {
        Self { ocr }
    }

    pub fn extract(
        &self,
        source: &[u8],
    ) -> Result<AttachmentTextRuntimeParseResultV1, AttachmentTextRuntimeParseErrorV1> {
        let parser = detect_attachment_text_parser_v1(source).map_err(map_error)?;
        let output = match parser {
            AttachmentTextParserKindV1::PlainUtf8 => extract_plain_text_v1(source),
            AttachmentTextParserKindV1::Pdf => extract_pdf_text_v1(source),
            AttachmentTextParserKindV1::Docx => extract_docx_text_v1(source),
            AttachmentTextParserKindV1::Ocr => self
                .ocr
                .as_ref()
                .ok_or(AttachmentTextParserErrorV1::ParserUnavailable)
                .and_then(|configuration| extract_image_text_v1(source, configuration)),
        }
        .map_err(map_error)?;
        let normalized =
            normalize_attachment_text_v1(&output.text_utf8, output.extraction_truncated)
                .map_err(|_| AttachmentTextRuntimeParseErrorV1::InvalidContent)?;
        Ok(AttachmentTextRuntimeParseResultV1 {
            text_utf8: normalized.bytes,
            format: format(parser),
            extraction_truncated: normalized.extraction_truncated,
            parser_identity_sha256: self.parser_identity_v1(parser),
        })
    }

    #[must_use]
    pub fn matches_artifact_identity_v1(
        &self,
        format: AttachmentTextFormatV1,
        artifact_identity_sha256: [u8; 32],
    ) -> bool {
        self.parser_identity_v1(parser_kind(format)) == artifact_identity_sha256
    }

    fn parser_identity_v1(&self, parser: AttachmentTextParserKindV1) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"makosh.attachment-text-extraction.parser-identity.v1\0");
        match parser {
            AttachmentTextParserKindV1::PlainUtf8 => digest.update(b"plain-v1"),
            AttachmentTextParserKindV1::Pdf => digest.update(b"pdf-text-extract-0.2.0-v1"),
            AttachmentTextParserKindV1::Docx => digest.update(b"docx-quick-xml-0.41.0-v1"),
            AttachmentTextParserKindV1::Ocr => {
                digest.update(b"tesseract-eng-rus-v1\0");
                let Some(configuration) = self.ocr.as_ref() else {
                    return [0; 32];
                };
                digest.update(configuration.executable_sha256);
                digest.update(configuration.english_model_sha256);
                digest.update(configuration.russian_model_sha256);
            }
        }
        digest.finalize().into()
    }
}

const fn format(parser: AttachmentTextParserKindV1) -> AttachmentTextFormatV1 {
    match parser {
        AttachmentTextParserKindV1::PlainUtf8 => AttachmentTextFormatV1::PlainUtf8,
        AttachmentTextParserKindV1::Pdf => AttachmentTextFormatV1::Pdf,
        AttachmentTextParserKindV1::Docx => AttachmentTextFormatV1::Docx,
        AttachmentTextParserKindV1::Ocr => AttachmentTextFormatV1::Ocr,
    }
}

const fn parser_kind(format: AttachmentTextFormatV1) -> AttachmentTextParserKindV1 {
    match format {
        AttachmentTextFormatV1::PlainUtf8 => AttachmentTextParserKindV1::PlainUtf8,
        AttachmentTextFormatV1::Pdf => AttachmentTextParserKindV1::Pdf,
        AttachmentTextFormatV1::Docx => AttachmentTextParserKindV1::Docx,
        AttachmentTextFormatV1::Ocr => AttachmentTextParserKindV1::Ocr,
    }
}

const fn map_error(error: AttachmentTextParserErrorV1) -> AttachmentTextRuntimeParseErrorV1 {
    match error {
        AttachmentTextParserErrorV1::SourceTooLarge => {
            AttachmentTextRuntimeParseErrorV1::SourceTooLarge
        }
        AttachmentTextParserErrorV1::UnsupportedFormat => {
            AttachmentTextRuntimeParseErrorV1::Unsupported
        }
        AttachmentTextParserErrorV1::ParserUnavailable
        | AttachmentTextParserErrorV1::ParserTimedOut => {
            AttachmentTextRuntimeParseErrorV1::ParserUnavailable
        }
        AttachmentTextParserErrorV1::ParserFailed => {
            AttachmentTextRuntimeParseErrorV1::ParserFailed
        }
        AttachmentTextParserErrorV1::EmptySource
        | AttachmentTextParserErrorV1::InvalidContent
        | AttachmentTextParserErrorV1::EncryptedContent
        | AttachmentTextParserErrorV1::OutputTooLarge => {
            AttachmentTextRuntimeParseErrorV1::InvalidContent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_plain_bytes_to_the_exact_adapter_and_normalizes_output() {
        let runtime = AttachmentTextExtractionParserRuntimeV1::new(None);
        let result = runtime
            .extract(b"first\r\nsecond")
            .expect("plain extraction");
        assert_eq!(result.format, AttachmentTextFormatV1::PlainUtf8);
        assert_eq!(result.text_utf8, b"first\nsecond");
        assert!(result.parser_identity_sha256.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn image_without_verified_ocr_configuration_fails_closed() {
        let runtime = AttachmentTextExtractionParserRuntimeV1::new(None);
        assert_eq!(
            runtime.extract(b"\x89PNG\r\n\x1a\nbody"),
            Err(AttachmentTextRuntimeParseErrorV1::ParserUnavailable)
        );
        assert!(!runtime.matches_artifact_identity_v1(AttachmentTextFormatV1::Ocr, [1; 32]));
    }

    #[test]
    fn artifact_identity_is_exact_for_the_current_parser_revision() {
        let runtime = AttachmentTextExtractionParserRuntimeV1::new(None);
        let parsed = runtime.extract(b"makosh").expect("plain extraction");
        assert!(runtime.matches_artifact_identity_v1(
            AttachmentTextFormatV1::PlainUtf8,
            parsed.parser_identity_sha256,
        ));
        assert!(
            !runtime.matches_artifact_identity_v1(AttachmentTextFormatV1::PlainUtf8, [0x55; 32],)
        );

        let first =
            AttachmentTextExtractionParserRuntimeV1::new(Some(TesseractOcrConfigurationV1 {
                executable: "runner".into(),
                executable_sha256: [1; 32],
                tessdata_directory: "tessdata".into(),
                english_model_sha256: [2; 32],
                russian_model_sha256: [3; 32],
                private_work_directory: "work".into(),
                timeout_millis: 1,
            }));
        let second =
            AttachmentTextExtractionParserRuntimeV1::new(Some(TesseractOcrConfigurationV1 {
                english_model_sha256: [4; 32],
                ..first.ocr.clone().expect("first OCR configuration")
            }));
        assert_ne!(
            first.parser_identity_v1(AttachmentTextParserKindV1::Ocr),
            second.parser_identity_v1(AttachmentTextParserKindV1::Ocr),
        );
    }
}
