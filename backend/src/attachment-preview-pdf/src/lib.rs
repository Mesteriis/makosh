#![forbid(unsafe_code)]

use std::{io::Cursor, panic::AssertUnwindSafe};

use hayro::{
    RenderCache, RenderSettings, hayro_interpret::InterpreterSettings, hayro_syntax::Pdf, render,
};
use image::{DynamicImage, ImageFormat, RgbaImage};
use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1,
    wire::{AttachmentPreviewContentTypeV1, AttachmentPreviewKindV1},
};
use makosh_attachment_preview_renderer_contract::{
    ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1, AttachmentPreviewRenderRequestV1,
    AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1, AttachmentPreviewRendererV1,
    AttachmentPreviewSourceFormatV1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-pdf";
const MAX_PDF_SOURCE_BYTES_V1: usize = 32 * 1024 * 1024;
const MAX_RENDER_DIMENSION_V1: f32 = 2_048.0;
const MAX_RENDER_SCALE_V1: f32 = 2.0;
const FORBIDDEN_ACTIVE_MARKERS_V1: [&[u8]; 8] = [
    b"/JavaScript",
    b"/OpenAction",
    b"/Launch",
    b"/EmbeddedFile",
    b"/RichMedia",
    b"/SubmitForm",
    b"/ImportData",
    b"/XFA",
];

#[derive(Clone, Copy, Debug, Default)]
pub struct AttachmentPreviewPdfRendererV1;

impl AttachmentPreviewRendererV1 for AttachmentPreviewPdfRendererV1 {
    fn render(
        &self,
        request: AttachmentPreviewRenderRequestV1<'_>,
    ) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
        if request.source_format != AttachmentPreviewSourceFormatV1::Pdf {
            return Err(AttachmentPreviewRendererErrorV1::Unsupported);
        }
        validate_source_v1(request.source_bytes)?;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            render_first_page_v1(request.source_bytes)
        }))
        .map_err(|_| AttachmentPreviewRendererErrorV1::Failed)?
    }
}

fn validate_source_v1(source: &[u8]) -> Result<(), AttachmentPreviewRendererErrorV1> {
    if source.is_empty() {
        return Err(AttachmentPreviewRendererErrorV1::Empty);
    }
    if source.len() > MAX_PDF_SOURCE_BYTES_V1 {
        return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
    }
    if source.len() < 8
        || !source.starts_with(b"%PDF-")
        || !matches!(source[5], b'1' | b'2')
        || source[6] != b'.'
        || !source[7].is_ascii_digit()
    {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    if FORBIDDEN_ACTIVE_MARKERS_V1
        .iter()
        .any(|marker| contains_bytes_v1(source, marker))
    {
        return Err(AttachmentPreviewRendererErrorV1::Unsupported);
    }
    Ok(())
}

fn render_first_page_v1(
    source: &[u8],
) -> Result<AttachmentPreviewRenderResultV1, AttachmentPreviewRendererErrorV1> {
    let pdf =
        Pdf::new(source.to_vec()).map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
    let page = pdf
        .pages()
        .first()
        .ok_or(AttachmentPreviewRendererErrorV1::InvalidContent)?;
    let (page_width, page_height) = page.render_dimensions();
    let (width, height, scale) = bounded_dimensions_v1(page_width, page_height)?;
    let pixmap = render(
        page,
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &RenderSettings {
            x_scale: scale,
            y_scale: scale,
            width: Some(width),
            height: Some(height),
            bg_color: hayro::vello_cpu::color::palette::css::WHITE,
        },
    );
    let image = RgbaImage::from_raw(
        u32::from(pixmap.width()),
        u32::from(pixmap.height()),
        pixmap.data_as_u8_slice().to_vec(),
    )
    .ok_or(AttachmentPreviewRendererErrorV1::Failed)?;
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|_| AttachmentPreviewRendererErrorV1::Failed)?;
    let bytes = output.into_inner();
    if bytes.is_empty() || bytes.len() as u64 > ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1 {
        return Err(AttachmentPreviewRendererErrorV1::OutputTooLarge);
    }
    Ok(AttachmentPreviewRenderResultV1 {
        preview_kind: AttachmentPreviewKindV1::Document,
        content_type: AttachmentPreviewContentTypeV1::Png,
        bytes,
        truncated: true,
    })
}

fn bounded_dimensions_v1(
    page_width: f32,
    page_height: f32,
) -> Result<(u16, u16, f32), AttachmentPreviewRendererErrorV1> {
    if !page_width.is_finite()
        || !page_height.is_finite()
        || page_width <= 0.0
        || page_height <= 0.0
    {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    let scale = MAX_RENDER_SCALE_V1
        .min(MAX_RENDER_DIMENSION_V1 / page_width)
        .min(MAX_RENDER_DIMENSION_V1 / page_height);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(AttachmentPreviewRendererErrorV1::InvalidContent);
    }
    let width = (page_width * scale)
        .round()
        .clamp(1.0, MAX_RENDER_DIMENSION_V1) as u16;
    let height = (page_height * scale)
        .round()
        .clamp(1.0, MAX_RENDER_DIMENSION_V1) as u16;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1 {
        return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
    }
    Ok((width, height, scale))
}

fn contains_bytes_v1(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|candidate| candidate == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_only_the_first_page_to_a_bounded_png() {
        let source = minimal_pdf_v1();
        let result = AttachmentPreviewPdfRendererV1
            .render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Pdf,
                source_bytes: &source,
            })
            .unwrap();
        assert!(result.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(result.preview_kind, AttachmentPreviewKindV1::Document);
        assert_eq!(result.content_type, AttachmentPreviewContentTypeV1::Png);
        assert!(result.truncated);
    }

    #[test]
    fn malformed_active_and_wrong_format_inputs_fail_closed() {
        assert_eq!(
            AttachmentPreviewPdfRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Pdf,
                source_bytes: b"%PDF-1.7\nnot-a-document",
            }),
            Err(AttachmentPreviewRendererErrorV1::InvalidContent)
        );
        assert_eq!(
            AttachmentPreviewPdfRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Pdf,
                source_bytes: b"%PDF-1.7\n/OpenAction 1 0 R",
            }),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
        assert_eq!(
            AttachmentPreviewPdfRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Png,
                source_bytes: b"%PDF-1.7\n",
            }),
            Err(AttachmentPreviewRendererErrorV1::Unsupported)
        );
    }

    #[test]
    fn oversized_source_fails_before_pdf_parsing() {
        let mut source = vec![b' '; MAX_PDF_SOURCE_BYTES_V1 + 1];
        source[..8].copy_from_slice(b"%PDF-1.7");
        assert_eq!(
            AttachmentPreviewPdfRendererV1.render(AttachmentPreviewRenderRequestV1 {
                source_format: AttachmentPreviewSourceFormatV1::Pdf,
                source_bytes: &source,
            }),
            Err(AttachmentPreviewRendererErrorV1::SourceTooLarge)
        );
    }

    fn minimal_pdf_v1() -> Vec<u8> {
        let mut bytes = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for object in [
            b"<< /Type /Catalog /Pages 2 0 R >>".as_slice(),
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".as_slice(),
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] /Resources <<>> /Contents 4 0 R >>".as_slice(),
            b"<< /Length 0 >>\nstream\n\nendstream".as_slice(),
        ] {
            offsets.push(bytes.len());
            let number = offsets.len();
            bytes.extend_from_slice(format!("{number} 0 obj\n").as_bytes());
            bytes.extend_from_slice(object);
            bytes.extend_from_slice(b"\nendobj\n");
        }
        let xref = bytes.len();
        bytes.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
        for offset in offsets {
            bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        bytes.extend_from_slice(
            format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
        );
        bytes
    }
}
