use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{TimeZone, Utc};
use serde_json::json;

use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::email_sync::{EmailSyncBatch, FetchedCommunicationSourceMessage};
use makosh_hub_backend::domains::communications::fixtures::export::export_fixture_messages_from_sync_batch;
use makosh_hub_backend::domains::communications::fixtures::export::models::EmailFixtureExportOptions;

#[test]
fn imap_raw_message_exports_redacted_fixture_without_personal_content() {
    let raw = concat!(
        "From: Alice Example <alice@company.test>\r\n",
        "To: Bob Example <bob@company.test>\r\n",
        "Subject: Confidential roadmap\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Real customer and roadmap details."
    );
    let batch = sync_batch_with_raw_message(raw);

    let fixtures =
        export_fixture_messages_from_sync_batch(&batch, EmailFixtureExportOptions::default())
            .expect("export fixture");

    assert_eq!(fixtures.len(), 1);
    let fixture = &fixtures[0];
    assert_eq!(fixture.provider_record_id, "43");
    assert_eq!(
        fixture.sent_at,
        Utc.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).single()
    );
    assert!(fixture.subject.starts_with("Redacted subject "));
    assert!(fixture.from.ends_with("@example.invalid"));
    assert_eq!(fixture.to.len(), 1);
    assert!(fixture.to[0].ends_with("@example.invalid"));
    assert!(fixture.body_text.contains("Redacted body fixture"));

    let fixture_json = serde_json::to_string(&fixtures).expect("fixture JSON");
    assert!(!fixture_json.contains("alice@company.test"));
    assert!(!fixture_json.contains("bob@company.test"));
    assert!(!fixture_json.contains("Confidential roadmap"));
    assert!(!fixture_json.contains("Real customer"));
}

#[test]
fn imap_multipart_quoted_printable_message_exports_redacted_fixture() {
    let raw = concat!(
        "From: Sender <sender@example.test>\r\n",
        "To: Team <team@example.test>\r\n",
        "Subject: =?UTF-8?Q?Q2_update?=\r\n",
        "Content-Type: multipart/alternative; boundary=\"boundary-1\"\r\n",
        "\r\n",
        "--boundary-1\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "Hello=2C team=21\r\n",
        "--boundary-1\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "\r\n",
        "<p>Hello, team!</p>\r\n",
        "--boundary-1--\r\n"
    );
    let batch = sync_batch_with_raw_message(raw);

    let fixtures =
        export_fixture_messages_from_sync_batch(&batch, EmailFixtureExportOptions::default())
            .expect("export fixture");

    assert_eq!(fixtures.len(), 1);
    assert!(fixtures[0].subject.starts_with("Redacted subject "));
    assert!(fixtures[0].body_text.contains("original_chars=12"));
}

fn sync_batch_with_raw_message(raw: &str) -> EmailSyncBatch {
    EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Icloud,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({
            "provider": "imap",
            "mailbox": "INBOX",
            "uid_validity": 999,
            "last_seen_uid": 43
        })),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: "43".to_owned(),
            source_fingerprint: "sha256:test-message".to_owned(),
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).single(),
            payload: json!({
                "provider": "icloud",
                "transport": "imap",
                "mailbox": "INBOX",
                "uid": 43,
                "uid_validity": 999,
                "raw_rfc822_base64": BASE64_STANDARD.encode(raw.as_bytes()),
                "rfc822_size": raw.len()
            }),
        }],
    }
}
