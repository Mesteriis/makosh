use makosh_attachment_preview_renderer_contract::AttachmentPreviewRendererErrorV1;
use quick_xml::{Reader, escape::unescape, events::Event};

const MAX_DOCUMENT_TEXT_BYTES_V1: u64 = 8 * 1024 * 1024;

pub(crate) fn extract_document_text_v1(
    xml: &[u8],
) -> Result<String, AttachmentPreviewRendererErrorV1> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut output = String::new();
    let mut in_text = false;
    loop {
        match reader
            .read_event()
            .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?
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
                    .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
                push_sanitized_text_v1(&mut output, &decoded);
            }
            Event::GeneralRef(reference) if in_text => {
                let reference = reference
                    .decode()
                    .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
                let encoded = format!("&{reference};");
                let decoded = unescape(&encoded)
                    .map_err(|_| AttachmentPreviewRendererErrorV1::InvalidContent)?;
                push_sanitized_text_v1(&mut output, &decoded);
            }
            Event::End(element) => match element.local_name().as_ref() {
                b"t" => in_text = false,
                b"p" => push_separator_v1(&mut output, '\n'),
                b"tc" => push_separator_v1(&mut output, '\t'),
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        if output.len() as u64 > MAX_DOCUMENT_TEXT_BYTES_V1 {
            return Err(AttachmentPreviewRendererErrorV1::OutputTooLarge);
        }
    }
    while output.ends_with(['\n', '\t', ' ']) {
        output.pop();
    }
    if output.trim().is_empty() {
        Err(AttachmentPreviewRendererErrorV1::InvalidContent)
    } else {
        Ok(output)
    }
}

fn push_sanitized_text_v1(output: &mut String, value: &str) {
    output.extend(value.chars().map(|character| {
        if character.is_control() && !matches!(character, '\n' | '\t') {
            '\u{fffd}'
        } else {
            character
        }
    }));
}

fn push_separator_v1(output: &mut String, separator: char) {
    if !output.is_empty() && !output.ends_with(separator) {
        output.push(separator);
    }
}
