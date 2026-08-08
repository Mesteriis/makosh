#![forbid(unsafe_code)]

use std::io::{Cursor, Read};

use makosh_attachment_text_extraction_parser_contract::{
    ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1, AttachmentTextParserErrorV1,
    AttachmentTextParserKindV1, AttachmentTextParserOutputV1, bounded_parser_output_v1,
    validate_source_bound,
};
use quick_xml::{Reader, escape::unescape, events::Event};
use zip::ZipArchive;

pub const PACKAGE: &str = "makosh-attachment-text-extraction-docx";
const MAX_DOCUMENT_XML_BYTES_V1: u64 = 8 * 1024 * 1024;

pub fn extract_docx_text_v1(
    source: &[u8],
) -> Result<AttachmentTextParserOutputV1, AttachmentTextParserErrorV1> {
    validate_source_bound(source)?;
    if !source.starts_with(b"PK\x03\x04") {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    let mut archive = ZipArchive::new(Cursor::new(source))
        .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
    let document = archive
        .by_name("word/document.xml")
        .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
    if document.encrypted() {
        return Err(AttachmentTextParserErrorV1::EncryptedContent);
    }
    if document.size() == 0 || document.size() > MAX_DOCUMENT_XML_BYTES_V1 {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    let mut xml = Vec::with_capacity(document.size() as usize);
    document
        .take(MAX_DOCUMENT_XML_BYTES_V1 + 1)
        .read_to_end(&mut xml)
        .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
    if xml.len() as u64 > MAX_DOCUMENT_XML_BYTES_V1 {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    let text = extract_document_xml_text(&xml)?;
    bounded_parser_output_v1(AttachmentTextParserKindV1::Docx, text, false)
}

fn extract_document_xml_text(xml: &[u8]) -> Result<String, AttachmentTextParserErrorV1> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut output = String::new();
    let mut in_text = false;
    loop {
        match reader
            .read_event()
            .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?
        {
            Event::Start(element) => match element.local_name().as_ref() {
                b"t" => in_text = true,
                b"tab" if in_text => output.push('\t'),
                b"br" if in_text => output.push('\n'),
                _ => {}
            },
            Event::Empty(element) => match element.local_name().as_ref() {
                b"tab" => output.push('\t'),
                b"br" => output.push('\n'),
                _ => {}
            },
            Event::Text(text) if in_text => {
                let decoded = text
                    .decode()
                    .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
                output.push_str(&decoded);
            }
            Event::GeneralRef(reference) if in_text => {
                let reference = reference
                    .decode()
                    .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
                let encoded = format!("&{reference};");
                let decoded =
                    unescape(&encoded).map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?;
                output.push_str(&decoded);
            }
            Event::End(element) => match element.local_name().as_ref() {
                b"t" => in_text = false,
                b"p" => push_separator(&mut output, '\n'),
                b"tc" => push_separator(&mut output, '\t'),
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        if output.len() > ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1 {
            return Err(AttachmentTextParserErrorV1::OutputTooLarge);
        }
    }
    while output.ends_with('\n') || output.ends_with('\t') {
        output.pop();
    }
    Ok(output)
}

fn push_separator(output: &mut String, separator: char) {
    if !output.is_empty() && !output.ends_with(separator) {
        output.push(separator);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    #[test]
    fn extracts_text_runs_paragraphs_entities_and_tabs() {
        let source = docx_bytes(
            br#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="urn:w"><w:body>
              <w:p><w:r><w:t>Hello &amp; makosh</w:t></w:r></w:p>
              <w:p><w:r><w:t>second</w:t></w:r><w:tab/><w:r><w:t>cell</w:t></w:r></w:p>
            </w:body></w:document>"#,
        );
        let output = extract_docx_text_v1(&source).expect("DOCX text");
        assert_eq!(output.parser, AttachmentTextParserKindV1::Docx);
        assert_eq!(output.text_utf8, b"Hello & makosh\nsecond\tcell");
    }

    #[test]
    fn rejects_zip_without_canonical_document_part() {
        let mut bytes = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut bytes));
            writer
                .start_file("other.xml", SimpleFileOptions::default())
                .expect("entry");
            writer.write_all(b"<other/>").expect("write");
            writer.finish().expect("finish");
        }
        assert_eq!(
            extract_docx_text_v1(&bytes),
            Err(AttachmentTextParserErrorV1::InvalidContent)
        );
    }

    fn docx_bytes(document_xml: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut bytes));
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer
                .start_file("word/document.xml", options)
                .expect("document entry");
            writer.write_all(document_xml).expect("document XML");
            writer.finish().expect("finish");
        }
        bytes
    }
}
