#![forbid(unsafe_code)]

mod card;
mod container;
mod document_text;

use std::panic::AssertUnwindSafe;

use makosh_attachment_preview_api::wire::{
    AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1,
};
use makosh_attachment_preview_renderer_contract::{
    AttachmentPreviewRenderRequestV1, AttachmentPreviewRenderResultV1,
    AttachmentPreviewRendererErrorV1, AttachmentPreviewRendererV1, AttachmentPreviewSourceFormatV1,
};

pub use card::FIXED_FONT_SHA256_V1;

pub const PACKAGE: &str = "makosh-attachment-preview-docx";
const MAX_DOCX_SOURCE_BYTES_V1: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct AttachmentPreviewDocxRendererV1;

impl AttachmentPreviewRendererV1 for AttachmentPreviewDocxRendererV1 {
    fn render(
        &self,
        request: AttachmentPreviewRenderRequestV1<'_>,
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        if request.source_format != AttachmentPreviewSourceFormatV1::DocxContainerCandidate {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        validate_source_bound_v1(request.source_bytes)?;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            let document_xml = container::read_bounded_docx_v1(request.source_bytes)?;
            let text = document_text::extract_document_text_v1(&document_xml)?;
            let rendered = card::render_docx_card_v1(&text)?;
            Ok(AttachmentPreviewRenderResultV1 {
                preview_kind: AttachmentPreviewKindV1::Document,
                content_type: AttachmentPreviewContentTypeV1::Png,
                bytes: rendered.bytes,
                truncated: rendered.truncated,
            })
        }))
        .map_err(|_| AttachmentPreviewRendererErrorV1::Failed)?
    }
}

fn validate_source_bound_v1(source: &[u8]) -> Result<(), AttachmentPreviewRendererErrorV1> {
    if source.is_empty() {
        return Err(AttachmentPreviewRendererErrorV1::Empty);
    }
    if source.len() > MAX_DOCX_SOURCE_BYTES_V1 {
        return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
    }
    if !source.starts_with(b"PK\x03\x04") {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use image::ImageFormat;
    use makosh_attachment_preview_api::ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    use super::*;

    #[test]
    fn renders_bounded_fixed_font_card_from_canonical_docx() {
        let source = docx_bytes_v1(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="urn:w"><w:body>
              <w:p><w:r><w:t>Hello &amp; Макошь</w:t></w:r></w:p>
              <w:p><w:r><w:t>Безопасный просмотр</w:t></w:r></w:p>
            </w:body></w:document>"#
                .as_bytes(),
            None,
            None,
        );
        let result = AttachmentPreviewDocxRendererV1
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::DocxContainerCandidate,
                source_bytes: &source,
            })
            .expect("DOCX preview");
        assert!(result.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(result.preview_kind, AttachmentPreviewKindV1::Document);
        assert_eq!(result.content_type, AttachmentPreviewContentTypeV1::Png);
        assert!(!result.truncated);
        assert!(result.bytes.len() as u64 <= ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1);
        let rendered = image::load_from_memory_with_format(&result.bytes, ImageFormat::Png)
            .expect("rendered PNG")
            .into_rgba8();
        let body_ink = rendered
            .enumerate_pixels()
            .filter(|(_, y, pixel)| {
                *y >= card::BODY_TOP_V1 as u32 && pixel.0 != [248, 250, 252, 255]
            })
            .count();
        assert!(body_ink > 100, "fixed font must rasterize DOCX body text");
        assert_eq!(FIXED_FONT_SHA256_V1.len(), 64);
    }

    #[test]
    fn rejects_external_relationships_and_active_office_parts() {
        let external = docx_bytes_v1(
            br#"<w:document xmlns:w="urn:w"><w:body><w:p><w:r><w:t>text</w:t></w:r></w:p></w:body></w:document>"#,
            Some(br#"<Relationships><Relationship TargetMode = "External" Target = "https://example.invalid/font.ttf"/></Relationships>"#),
            None,
        );
        assert_eq!(
            render_v1(&external),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
        let macro_document = docx_bytes_v1(
            br#"<w:document xmlns:w="urn:w"><w:body><w:p><w:r><w:t>text</w:t></w:r></w:p></w:body></w:document>"#,
            None,
            Some(("word/vbaProject.bin", b"active")),
        );
        assert_eq!(
            render_v1(&macro_document),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
    }

    #[test]
    fn rejects_non_docx_zip_and_wrong_format() {
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
            render_v1(&bytes),
            Err(AttachmentPreviewRendererErrorV1::InvalidContent)
        );
        assert_eq!(
            AttachmentPreviewDocxRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Pdf,
                source_bytes: b"PK\x03\x04",
            }),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
    }

    fn render_v1(
        bytes: &[u8],
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        AttachmentPreviewDocxRendererV1.render(AttachmentPreviewRenderRequestV1 {
            source_format: AttachmentPreviewSourceFormatV1::DocxContainerCandidate,
            source_bytes: bytes,
        })
    }

    fn docx_bytes_v1(
        document_xml: &[u8],
        relationships: Option<&[u8]>,
        extra: Option<(&str, &[u8])>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut bytes));
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer
                .start_file("[Content_Types].xml", options)
                .expect("content types");
            writer
                .write_all(
                    br#"<Types><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
                )
                .expect("content types XML");
            writer
                .start_file("_rels/.rels", options)
                .expect("root relationships");
            writer
                .write_all(
                    br#"<Relationships><Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
                )
                .expect("root relationships XML");
            writer
                .start_file("word/document.xml", options)
                .expect("document");
            writer.write_all(document_xml).expect("document XML");
            if let Some(relationships) = relationships {
                writer
                    .start_file("word/_rels/document.xml.rels", options)
                    .expect("relationships");
                writer.write_all(relationships).expect("relationships XML");
            }
            if let Some((name, content)) = extra {
                writer.start_file(name, options).expect("extra entry");
                writer.write_all(content).expect("extra content");
            }
            writer.finish().expect("finish");
        }
        bytes
    }
}
