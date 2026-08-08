use std::io::Cursor;

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use makosh_attachment_preview_api::ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1;
use makosh_attachment_preview_renderer_contract::{
    ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1, AttachmentPreviewRendererErrorV1,
};
use swash::{
    FontRef,
    scale::{Render, ScaleContext, Source, image::Content},
    zeno::Format,
};

const FIXED_FONT_BYTES_V1: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
pub const FIXED_FONT_SHA256_V1: &str =
    "7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954";
const PREVIEW_WIDTH_V1: u32 = 1_200;
const PREVIEW_HEIGHT_V1: u32 = 1_500;
const PREVIEW_MAX_LINES_V1: usize = 52;
const PAGE_MARGIN_X_V1: f32 = 72.0;
pub(crate) const BODY_TOP_V1: f32 = 156.0;
const BODY_FONT_SIZE_V1: f32 = 24.0;
const BODY_LINE_HEIGHT_V1: f32 = 34.0;
const TITLE_FONT_SIZE_V1: f32 = 32.0;
const TITLE_V1: &str = "Макошь safe DOCX preview";
const TRUNCATED_V1: &str = "[Preview truncated by Макошь safety limit]";

pub(crate) struct RenderedDocxCardV1 {
    pub(crate) bytes: Vec<u8>,
    pub(crate) truncated: bool,
}

#[derive(Clone, Copy, Debug)]
struct TextStyleV1 {
    x: f32,
    y: f32,
    font_size: f32,
    color: [u8; 4],
}

pub(crate) fn render_docx_card_v1(
    text: &str,
) -> Result<RenderedDocxCardV1, AttachmentPreviewRendererErrorV1> {
    let font = FontRef::from_index(FIXED_FONT_BYTES_V1, 0)
        .ok_or(AttachmentPreviewRendererErrorV1::Failed)?;
    let (mut lines, mut truncated) = wrap_text_v1(
        &font,
        text,
        BODY_FONT_SIZE_V1,
        PREVIEW_WIDTH_V1 as f32 - PAGE_MARGIN_X_V1 * 2.0,
        PREVIEW_MAX_LINES_V1,
    );
    if lines.len() > PREVIEW_MAX_LINES_V1 {
        lines.truncate(PREVIEW_MAX_LINES_V1);
        truncated = true;
    }
    if truncated && let Some(last_line) = lines.last_mut() {
        last_line.clear();
        last_line.push_str(TRUNCATED_V1);
    }

    let pixels = u64::from(PREVIEW_WIDTH_V1) * u64::from(PREVIEW_HEIGHT_V1);
    if pixels > ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1 {
        return Err(AttachmentPreviewRendererErrorV1::SourceTooLarge);
    }
    let mut image = RgbaImage::from_pixel(
        PREVIEW_WIDTH_V1,
        PREVIEW_HEIGHT_V1,
        Rgba([248, 250, 252, 255]),
    );
    let mut scale_context = ScaleContext::new();
    draw_text_v1(
        &mut image,
        &font,
        &mut scale_context,
        TITLE_V1,
        TextStyleV1 {
            x: PAGE_MARGIN_X_V1,
            y: 64.0,
            font_size: TITLE_FONT_SIZE_V1,
            color: [15, 23, 42, 255],
        },
    );
    draw_horizontal_rule_v1(
        &mut image,
        72,
        120,
        PREVIEW_WIDTH_V1 - 72,
        [203, 213, 225, 255],
    );
    for (index, line) in lines.iter().enumerate() {
        draw_text_v1(
            &mut image,
            &font,
            &mut scale_context,
            line,
            TextStyleV1 {
                x: PAGE_MARGIN_X_V1,
                y: BODY_TOP_V1 + index as f32 * BODY_LINE_HEIGHT_V1,
                font_size: BODY_FONT_SIZE_V1,
                color: [30, 41, 59, 255],
            },
        );
    }

    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|_| AttachmentPreviewRendererErrorV1::Failed)?;
    let bytes = output.into_inner();
    if bytes.is_empty() || bytes.len() as u64 > ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1 {
        return Err(AttachmentPreviewRendererErrorV1::OutputTooLarge);
    }
    Ok(RenderedDocxCardV1 { bytes, truncated })
}

fn wrap_text_v1(
    font: &FontRef<'_>,
    text: &str,
    font_size: f32,
    max_width: f32,
    max_lines: usize,
) -> (Vec<String>, bool) {
    let charmap = font.charmap();
    let metrics = font.glyph_metrics(&[]).scale(font_size);
    let mut lines = Vec::new();
    let mut truncated = false;
    for paragraph in text.lines() {
        let mut current = String::new();
        let mut current_width = 0.0_f32;
        for character in paragraph
            .chars()
            .flat_map(|character| {
                if character == '\t' {
                    [' ', ' ', ' ', ' '].into_iter()
                } else {
                    [character, '\0', '\0', '\0'].into_iter()
                }
            })
            .filter(|character| *character != '\0')
        {
            let advance = metrics.advance_width(charmap.map(character));
            if !current.is_empty() && current_width + advance > max_width {
                lines.push(std::mem::take(&mut current));
                current_width = 0.0;
                if lines.len() >= max_lines {
                    return (lines, true);
                }
            }
            current.push(character);
            current_width += advance;
        }
        lines.push(current);
        if lines.len() >= max_lines {
            truncated = text.lines().count() > lines.len();
            break;
        }
    }
    (lines, truncated)
}

fn draw_text_v1(
    image: &mut RgbaImage,
    font: &FontRef<'_>,
    scale_context: &mut ScaleContext,
    text: &str,
    style: TextStyleV1,
) {
    let charmap = font.charmap();
    let metrics = font.glyph_metrics(&[]).scale(style.font_size);
    let mut scaler = scale_context
        .builder(*font)
        .size(style.font_size)
        .hint(true)
        .build();
    let sources = [Source::Outline];
    let mut renderer = Render::new(&sources);
    renderer.format(Format::Alpha);
    let baseline = style.y + font.metrics(&[]).scale(style.font_size).ascent;
    let mut caret_x = style.x;
    for character in text.chars() {
        let glyph_id = charmap.map(character);
        if let Some(glyph) = renderer.render(&mut scaler, glyph_id)
            && glyph.content == Content::Mask
            && glyph.placement.width > 0
        {
            let origin_x = caret_x.round() as i32 + glyph.placement.left;
            let origin_y = baseline.round() as i32 - glyph.placement.top;
            for (index, coverage) in glyph.data.iter().copied().enumerate() {
                let offset_x = index as u32 % glyph.placement.width;
                let offset_y = index as u32 / glyph.placement.width;
                blend_pixel_v1(
                    image,
                    origin_x + offset_x as i32,
                    origin_y + offset_y as i32,
                    style.color,
                    f32::from(coverage) / 255.0,
                );
            }
        }
        caret_x += metrics.advance_width(glyph_id);
    }
}

fn blend_pixel_v1(image: &mut RgbaImage, x: i32, y: i32, color: [u8; 4], coverage: f32) {
    let Ok(x) = u32::try_from(x) else { return };
    let Ok(y) = u32::try_from(y) else { return };
    let Some(destination) = image.get_pixel_mut_checked(x, y) else {
        return;
    };
    let alpha = coverage.clamp(0.0, 1.0);
    for channel in 0..3 {
        destination[channel] = (f32::from(destination[channel]) * (1.0 - alpha)
            + f32::from(color[channel]) * alpha)
            .round() as u8;
    }
}

fn draw_horizontal_rule_v1(image: &mut RgbaImage, x0: u32, y: u32, x1: u32, color: [u8; 4]) {
    if y >= image.height() {
        return;
    }
    for x in x0.min(image.width())..x1.min(image.width()) {
        image.put_pixel(x, y, Rgba(color));
        if y + 1 < image.height() {
            image.put_pixel(x, y + 1, Rgba(color));
        }
    }
}
