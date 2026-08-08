#![forbid(unsafe_code)]

use makosh_attachment_text_extraction_parser_contract::{
    AttachmentTextParserErrorV1, AttachmentTextParserKindV1, AttachmentTextParserOutputV1,
    bounded_parser_output_v1, validate_source_bound,
};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-pdf";

pub fn extract_pdf_text_v1(
    source: &[u8],
) -> Result<AttachmentTextParserOutputV1, AttachmentTextParserErrorV1> {
    validate_source_bound(source)?;
    if !source.starts_with(b"%PDF-") {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    let extracted = pdf_text_extract::pdf_to_text(source)
        .map_err(|_| AttachmentTextParserErrorV1::ParserFailed)?;
    let normalized = extracted.replace("\r\n", "\n").replace('\r', "\n");
    bounded_parser_output_v1(AttachmentTextParserKindV1::Pdf, normalized.trim(), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_embedded_pdf_text_from_memory_only() {
        let source = simple_pdf("Hello makosh");
        let output = extract_pdf_text_v1(&source).expect("PDF text");
        assert_eq!(output.parser, AttachmentTextParserKindV1::Pdf);
        assert!(
            std::str::from_utf8(&output.text_utf8)
                .expect("UTF-8")
                .contains("Hello makosh")
        );
    }

    #[test]
    fn rejects_declared_pdf_without_a_valid_document() {
        assert_eq!(
            extract_pdf_text_v1(b"%PDF-1.7\ninvalid"),
            Err(AttachmentTextParserErrorV1::ParserFailed)
        );
    }

    fn simple_pdf(text: &str) -> Vec<u8> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        let stream = format!("BT /F1 12 Tf 72 720 Td ({escaped}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
            format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()),
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }
}
