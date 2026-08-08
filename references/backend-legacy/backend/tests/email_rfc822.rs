use makosh_hub_backend::platform::communications::rfc822::models::ParsedEmailAttachmentDisposition;
use makosh_hub_backend::platform::communications::rfc822::parser::parse_rfc822_message;

#[test]
fn rfc822_parser_extracts_nested_multipart_attachments_for_current_basic_slice() {
    let raw = concat!(
        "Subject: Nested attachments\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"outer-boundary\"\r\n",
        "\r\n",
        "--outer-boundary\r\n",
        "Content-Type: multipart/alternative; boundary=\"alt-boundary\"\r\n",
        "\r\n",
        "--alt-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "Nested=20plain=20body.\r\n",
        "--alt-boundary\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "\r\n",
        "<p>Nested HTML body.</p>\r\n",
        "--alt-boundary--\r\n",
        "--outer-boundary\r\n",
        "Content-Type: application/pdf; name*=UTF-8''report%20Q2.pdf\r\n",
        "Content-Disposition: attachment; filename*=UTF-8''report%20Q2.pdf\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQ=\r\n",
        "--outer-boundary\r\n",
        "Content-Type: text/plain; name=\"notes.txt\"\r\n",
        "Content-Disposition: inline; filename=\"notes.txt\"\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "note=20body=0Asecond=20line\r\n",
        "--outer-boundary--\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse nested multipart message");

    assert_eq!(parsed.subject, "Nested attachments");
    assert_eq!(parsed.body_text, "Nested plain body.");
    assert_eq!(
        parsed.body_html.as_deref(),
        Some("<p>Nested HTML body.</p>")
    );
    assert_eq!(parsed.attachments.len(), 2);

    let pdf = &parsed.attachments[0];
    assert_eq!(pdf.provider_attachment_id, "part-1");
    assert_eq!(pdf.filename.as_deref(), Some("report Q2.pdf"));
    assert_eq!(pdf.content_type, "application/pdf");
    assert_eq!(
        pdf.disposition,
        ParsedEmailAttachmentDisposition::Attachment
    );
    assert_eq!(pdf.body_bytes, b"%PDF-1.4");

    let notes = &parsed.attachments[1];
    assert_eq!(notes.provider_attachment_id, "part-2");
    assert_eq!(notes.filename.as_deref(), Some("notes.txt"));
    assert_eq!(notes.content_type, "text/plain");
    assert_eq!(notes.disposition, ParsedEmailAttachmentDisposition::Inline);
    assert_eq!(notes.body_bytes, b"note body\nsecond line");
}

#[test]
fn rfc822_parser_preserves_html_links_for_rich_mail_rendering() {
    let raw = concat!(
        "Subject: Rich links\r\n",
        "From: Fever <hello@example.invalid>\r\n",
        "To: User <user@example.invalid>\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "<p>Footer</p><a href=3D\"https://click.example.invalid/privacy?qs=3Dabc\">Privacy policy</a>",
        "<a href=3D\"https://click.example.invalid/contact?qs=3Dabc\">Contact us</a>",
        "<a href=3D\"https://click.example.invalid/unsub?qs=3Dabc\">Unsubscribe</a>\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse rich html message");

    assert!(parsed.body_text.contains("Privacy policy"));
    let html = parsed.body_html.as_deref().expect("body html");
    assert!(html.contains("href=\"https://click.example.invalid/privacy?qs=abc\""));
    assert!(html.contains(">Privacy policy</a>"));
    assert!(html.contains(">Contact us</a>"));
    assert!(html.contains(">Unsubscribe</a>"));
}

#[test]
fn rfc822_parser_preserves_source_headers_with_folded_values() {
    let raw = concat!(
        "Subject: Folded headers\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "X-Макошь-Trace: first line\r\n",
        "\tcontinued line\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Body.\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse folded header message");

    assert!(parsed.headers.contains(&(
        "X-Макошь-Trace".to_owned(),
        "first line continued line".to_owned()
    )));
    assert!(parsed.headers.contains(&(
        "Content-Type".to_owned(),
        "text/plain; charset=utf-8".to_owned()
    )));
}

#[test]
fn rfc822_parser_extracts_rfc2231_continued_attachment_filenames() {
    let raw = concat!(
        "Subject: Continued filename\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"continued-boundary\"\r\n",
        "\r\n",
        "--continued-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Body.\r\n",
        "--continued-boundary\r\n",
        "Content-Type: application/pdf;\r\n",
        " name*0*=UTF-8''quarterly%20;\r\n",
        " name*1*=%D1%84%D0%B0%D0%B9%D0%BB;\r\n",
        " name*2=.pdf\r\n",
        "Content-Disposition: attachment;\r\n",
        " filename*0*=UTF-8''quarterly%20;\r\n",
        " filename*1*=%D1%84%D0%B0%D0%B9%D0%BB;\r\n",
        " filename*2=.pdf\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQ=\r\n",
        "--continued-boundary--\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse continued filename message");

    assert_eq!(parsed.attachments.len(), 1);
    let attachment = &parsed.attachments[0];
    assert_eq!(attachment.filename.as_deref(), Some("quarterly файл.pdf"));
    assert_eq!(attachment.content_type, "application/pdf");
    assert_eq!(attachment.body_bytes, b"%PDF-1.4");
}

#[test]
fn rfc822_parser_decodes_legacy_cyrillic_message_bytes() {
    let mut raw = Vec::new();
    raw.extend_from_slice(b"Subject: ");
    raw.extend_from_slice(&[
        0xd2, 0xe5, 0xf1, 0xf2, 0xee, 0xe2, 0xee, 0xe5, 0x20, 0xef, 0xe8, 0xf1, 0xfc, 0xec, 0xee,
    ]);
    raw.extend_from_slice(b"\r\nFrom: ");
    raw.extend_from_slice(&[
        0xc8, 0xe2, 0xe0, 0xed, 0x20, 0xcf, 0xe5, 0xf2, 0xf0, 0xee, 0xe2,
    ]);
    raw.extend_from_slice(b" <ivan@example.invalid>\r\n");
    raw.extend_from_slice(b"To: Recipient <recipient@example.invalid>\r\n");
    raw.extend_from_slice(b"Content-Type: text/plain; charset=windows-1251\r\n");
    raw.extend_from_slice(b"\r\n");
    raw.extend_from_slice(&[
        0xcf, 0xf0, 0xe8, 0xe2, 0xe5, 0xf2, 0x2c, 0x20, 0xfd, 0xf2, 0xee, 0x20, 0xf1, 0xf2, 0xe0,
        0xf0, 0xee, 0xe5, 0x20, 0xef, 0xe8, 0xf1, 0xfc, 0xec, 0xee, 0x2e,
    ]);

    let parsed = parse_rfc822_message(&raw).expect("parse legacy cyrillic message");

    assert_eq!(parsed.subject, "Тестовое письмо");
    assert_eq!(parsed.from, "Иван Петров <ivan@example.invalid>");
    assert_eq!(parsed.body_text, "Привет, это старое письмо.");
    assert!(!parsed.body_text.contains('\u{fffd}'));
}
