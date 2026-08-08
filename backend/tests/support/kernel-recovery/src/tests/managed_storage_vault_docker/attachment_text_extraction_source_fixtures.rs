//! In-memory supported-format sources for managed Text Extraction conformance.

use std::io::{Cursor, Write};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub(super) fn attachment_text_pdf_source_v1(text: &str) -> Vec<u8> {
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

pub(super) fn attachment_text_docx_source_v1(text: &str) -> Vec<u8> {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="urn:w"><w:body><w:p><w:r><w:t>{escaped}</w:t></w:r></w:p></w:body></w:document>"#,
    );
    let mut bytes = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut bytes));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer
            .start_file("word/document.xml", options)
            .expect("DOCX document entry");
        writer.write_all(xml.as_bytes()).expect("DOCX document XML");
        writer.finish().expect("finish DOCX fixture");
    }
    bytes
}

pub(super) fn attachment_text_ocr_png_source_v1() -> Vec<u8> {
    const WIDTH: usize = 320;
    const HEIGHT: usize = 160;
    const SCALE: usize = 8;
    const GLYPH_WIDTH: usize = 5;
    const GLYPH_HEIGHT: usize = 7;
    const LEFT: usize = 20;
    const TOP: usize = 14;
    const LINE_GAP: usize = 20;
    const LETTER_GAP: usize = 8;
    const WORD: [[u8; GLYPH_HEIGHT]; 6] = [
        [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
    ];
    let row_bytes = WIDTH.div_ceil(8);
    let mut scanlines = vec![0xff; (row_bytes + 1) * HEIGHT];
    for row in 0..HEIGHT {
        scanlines[row * (row_bytes + 1)] = 0;
    }
    for line in 0..2 {
        let line_top = TOP + line * (GLYPH_HEIGHT * SCALE + LINE_GAP);
        for (glyph_index, glyph) in WORD.iter().enumerate() {
            let glyph_left = LEFT + glyph_index * (GLYPH_WIDTH * SCALE + LETTER_GAP);
            for (row, bits) in glyph.iter().enumerate() {
                for column in 0..GLYPH_WIDTH {
                    if bits & (1 << (GLYPH_WIDTH - 1 - column)) == 0 {
                        continue;
                    }
                    for y in 0..SCALE {
                        for x in 0..SCALE {
                            let image_y = line_top + row * SCALE + y;
                            let image_x = glyph_left + column * SCALE + x;
                            let byte = image_y * (row_bytes + 1) + 1 + image_x / 8;
                            scanlines[byte] &= !(1 << (7 - image_x % 8));
                        }
                    }
                }
            }
        }
    }
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(WIDTH as u32).to_be_bytes());
    ihdr.extend_from_slice(&(HEIGHT as u32).to_be_bytes());
    ihdr.extend_from_slice(&[1, 0, 0, 0, 0]);
    append_png_chunk_v1(&mut png, b"IHDR", &ihdr);
    append_png_chunk_v1(&mut png, b"IDAT", &zlib_stored_v1(&scanlines));
    append_png_chunk_v1(&mut png, b"IEND", &[]);
    png
}

fn zlib_stored_v1(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() <= usize::from(u16::MAX));
    let length = bytes.len() as u16;
    let mut encoded = Vec::with_capacity(bytes.len() + 11);
    encoded.extend_from_slice(&[0x78, 0x01, 0x01]);
    encoded.extend_from_slice(&length.to_le_bytes());
    encoded.extend_from_slice(&(!length).to_le_bytes());
    encoded.extend_from_slice(bytes);
    encoded.extend_from_slice(&adler32_v1(bytes).to_be_bytes());
    encoded
}

fn append_png_chunk_v1(png: &mut Vec<u8>, kind: &[u8; 4], payload: &[u8]) {
    png.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(payload);
    let mut checked = Vec::with_capacity(kind.len() + payload.len());
    checked.extend_from_slice(kind);
    checked.extend_from_slice(payload);
    png.extend_from_slice(&crc32_v1(&checked).to_be_bytes());
}

fn adler32_v1(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);
    for byte in bytes {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn crc32_v1(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_fixtures_have_exact_magic_and_private_payloads() {
        let pdf = attachment_text_pdf_source_v1("makosh PDF");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.windows(10).any(|window| window == b"makosh PDF"));
        let docx = attachment_text_docx_source_v1("makosh DOCX");
        assert!(docx.starts_with(b"PK\x03\x04"));
        assert!(
            docx.windows(b"word/document.xml".len())
                .any(|window| window == b"word/document.xml")
        );
        let png = attachment_text_ocr_png_source_v1();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() < 8 * 1024);
    }
}
