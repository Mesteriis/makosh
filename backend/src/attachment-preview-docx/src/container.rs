use std::io::{Cursor, Read};

use makosh_attachment_preview_renderer_contract::AttachmentPreviewRendererErrorV1;
use quick_xml::{Reader, XmlVersion, events::Event};
use zip::{CompressionMethod, ZipArchive};

const MAX_ZIP_ENTRIES_V1: usize = 10_000;
const MAX_ZIP_UNCOMPRESSED_BYTES_V1: u64 = 50 * 1024 * 1024;
const MAX_DOCUMENT_XML_BYTES_V1: u64 = 8 * 1024 * 1024;
const MAX_RELATIONSHIPS_XML_BYTES_V1: u64 = 1024 * 1024;
const DOCX_MAIN_CONTENT_TYPE_V1: &[u8] =
    b"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
const FORBIDDEN_ENTRY_MARKERS_V1: [&str; 12] = [
    "vbaproject",
    "vbadata",
    "activex/",
    "embeddings/",
    "oleobject",
    "afchunk",
    "customui/",
    "webextensions/",
    "macros/",
    "word/fonts/",
    ".odttf",
    ".bin",
];
const FORBIDDEN_XML_MARKERS_V1: [&[u8]; 10] = [
    b"<!DOCTYPE",
    b"<!ENTITY",
    b"<w:altChunk",
    b"<w:object",
    b"<w:control",
    b"<o:OLEObject",
    b"<v:imagedata",
    b"<a:blip",
    b"macrosEnabled",
    b"macroEnabled",
];

pub(crate) fn read_bounded_docx_v1(
    source: &[u8],
) -> Result<Vec<u8>, AttachmentPreviewRendererErrorV1> {
    let mut archive = ZipArchive::new(Cursor::new(source))
        .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
    if archive.is_empty() || archive.len() > MAX_ZIP_ENTRIES_V1 {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    let mut total_uncompressed = 0_u64;
    let mut document_xml = None;
    let mut content_types_seen = false;
    let mut root_relationships_seen = false;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let name = canonical_entry_name_v1(entry.name())?;
        if entry.encrypted() || is_symlink_v1(entry.unix_mode()) {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        total_uncompressed = total_uncompressed
            .checked_add(entry.size())
            .ok_or(AttachmentPreviewRendererErrorV1::SourceTooLarge)?;
        if total_uncompressed > MAX_ZIP_UNCOMPRESSED_BYTES_V1 {
            return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
        }
        if FORBIDDEN_ENTRY_MARKERS_V1
            .iter()
            .any(|marker| name.contains(marker))
        {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        if name == "[content_types].xml" {
            if content_types_seen {
                return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
            }
            let xml = read_zip_entry_bounded_v1(&mut entry, MAX_RELATIONSHIPS_XML_BYTES_V1)?;
            validate_content_types_v1(&xml)?;
            content_types_seen = true;
        } else if name.ends_with(".rels") {
            let xml = read_zip_entry_bounded_v1(&mut entry, MAX_RELATIONSHIPS_XML_BYTES_V1)?;
            let has_office_document = validate_relationships_v1(&xml)?;
            if name == "_rels/.rels" {
                if root_relationships_seen || !has_office_document {
                    return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
                }
                root_relationships_seen = true;
            }
        } else if name == "word/document.xml" {
            if document_xml.is_some() || entry.size() == 0 {
                return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
            }
            let xml = read_zip_entry_bounded_v1(&mut entry, MAX_DOCUMENT_XML_BYTES_V1)?;
            reject_forbidden_xml_v1(&xml)?;
            document_xml = Some(xml);
        }
    }
    if !content_types_seen || !root_relationships_seen {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    document_xml.ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)
}

fn canonical_entry_name_v1(name: &str) -> Result<String, AttachmentPreviewRendererErrorV1> {
    let normalized = name.replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.split('/').any(|part| part == "..")
        || normalized.contains(':')
    {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    Ok(normalized)
}

fn is_symlink_v1(mode: Option<u32>) -> bool {
    mode.is_some_and(|value| value & 0o170000 == 0o120000)
}

fn read_zip_entry_bounded_v1<R: Read>(
    reader: &mut R,
    max_bytes: u64,
) -> Result<Vec<u8>, AttachmentPreviewRendererErrorV1> {
    let mut bytes = Vec::new();
    reader
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
    if bytes.is_empty() || bytes.len() as u64 > max_bytes {
        return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
    }
    Ok(bytes)
}

fn validate_content_types_v1(xml: &[u8]) -> Result<(), AttachmentPreviewRendererErrorV1> {
    reject_forbidden_xml_v1(xml)?;
    let mut reader = Reader::from_reader(xml);
    let mut canonical_main_part = false;
    loop {
        let event = reader
            .read_event()
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let element = match event {
            Event::Start(element) | Event::Empty(element)
                if element.local_name().as_ref() == b"Override" =>
            {
                element
            }
            Event::Eof => break,
            _ => continue,
        };
        let mut part_name = None;
        let mut content_type = None;
        for attribute in element.attributes().with_checks(true) {
            let attribute =
                attribute.map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?
                .into_owned();
            match attribute.key.local_name().as_ref() {
                b"PartName" => part_name = Some(value),
                b"ContentType" => content_type = Some(value),
                _ => {}
            }
        }
        if part_name.as_deref() == Some("/word/document.xml")
            && content_type.as_deref().map(str::as_bytes) == Some(DOCX_MAIN_CONTENT_TYPE_V1)
        {
            canonical_main_part = true;
        }
    }
    if canonical_main_part {
        Ok(())
    } else {
        Err(AttachmentPreviewRendererErrorV1::InvalidContent)
    }
}

fn validate_relationships_v1(xml: &[u8]) -> Result<bool, AttachmentPreviewRendererErrorV1> {
    reject_forbidden_xml_v1(xml)?;
    let mut reader = Reader::from_reader(xml);
    let mut has_office_document = false;
    loop {
        let event = reader
            .read_event()
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let element = match event {
            Event::Start(element) | Event::Empty(element)
                if element.local_name().as_ref() == b"Relationship" =>
            {
                element
            }
            Event::Eof => break,
            _ => continue,
        };
        let mut target = None;
        let mut relationship_type = None;
        let mut target_mode = None;
        for attribute in element.attributes().with_checks(true) {
            let attribute =
                attribute.map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?
                .into_owned();
            match attribute.key.local_name().as_ref() {
                b"Target" => target = Some(value),
                b"Type" => relationship_type = Some(value),
                b"TargetMode" => target_mode = Some(value),
                _ => {}
            }
        }
        if target_mode
            .as_deref()
            .is_some_and(|mode| mode.eq_ignore_ascii_case("external"))
        {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        let target = target.ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)?;
        let lower_target = target.to_ascii_lowercase();
        if lower_target.starts_with("http:")
            || lower_target.starts_with("https:")
            || lower_target.starts_with("file:")
            || lower_target.starts_with("//")
            || lower_target.starts_with("\\\\")
        {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        if relationship_type
            .as_deref()
            .is_some_and(|value| value.ends_with("/officeDocument"))
            && matches!(target.as_str(), "word/document.xml" | "/word/document.xml")
        {
            has_office_document = true;
        }
    }
    Ok(has_office_document)
}

fn reject_forbidden_xml_v1(xml: &[u8]) -> Result<(), AttachmentPreviewRendererErrorV1> {
    if FORBIDDEN_XML_MARKERS_V1
        .iter()
        .any(|marker| contains_bytes_v1(xml, marker))
    {
        return Err(AttachmentPreviewRendererErrorV1::Unsupported);
    }
    Ok(())
}

fn contains_bytes_v1(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|candidate| candidate == needle)
}
