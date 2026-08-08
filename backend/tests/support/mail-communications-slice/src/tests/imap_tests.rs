use hermes_mail_api::IMAP_PORT;
use hermes_mail_api::MAX_WINDOW;
use hermes_mail_imap::{
    ImapMessage, ImapSyncAccessV1, ImapSyncRequestV1, supports_read_only_sync, sync_inbox,
};

#[test]
fn sync_requires_password() {
    let result = sync_inbox(
        ImapSyncAccessV1 {
            host: "mail.example.com",
            port: IMAP_PORT,
            username: "user",
            password: None,
        },
        ImapSyncRequestV1 {
            window: MAX_WINDOW,
            windows: 1,
            priority_uids: &[],
        },
        |_| Ok(()),
    );
    assert!(matches!(result, Err(error) if error == "imap password is required"));
}

#[test]
fn supports_only_read_windows() {
    assert!(supports_read_only_sync(MAX_WINDOW));
    assert!(!supports_read_only_sync(0));
    assert!(!supports_read_only_sync(MAX_WINDOW + 1));
    assert!(
        ImapMessage {
            uid: 1,
            subject: "s".to_owned(),
            sender: None,
            recipients: Vec::new(),
            snippet: "p".to_owned(),
            sent_at_unix_seconds: None,
            flags: Vec::new(),
            has_plain_text: true,
            plain_text_body: None,
            body_content: None,
            attachments: Vec::new(),
        } != ImapMessage {
            uid: 2,
            subject: "s".to_owned(),
            sender: None,
            recipients: Vec::new(),
            snippet: "p".to_owned(),
            sent_at_unix_seconds: None,
            flags: Vec::new(),
            has_plain_text: true,
            plain_text_body: None,
            body_content: None,
            attachments: Vec::new(),
        }
    );
}
