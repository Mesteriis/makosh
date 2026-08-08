// §4.2-4.4: Reply-all, Forward-EML, Send-later, Undo-send, Quoting
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplyConfig {
    pub quote_original: bool,
    pub include_attachments: bool,
    pub reply_all: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForwardConfig {
    pub as_eml: bool,
    pub attachments_only: bool,
    pub include_ai_summary: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledSend {
    pub send_at: DateTime<Utc>,
    pub draft_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UndoSendWindow {
    pub window_seconds: i32,
    pub enabled: bool,
}

impl Default for ReplyConfig {
    fn default() -> Self {
        Self {
            quote_original: true,
            include_attachments: false,
            reply_all: false,
        }
    }
}

/// Build a reply body with optional quoting.
pub fn build_reply_body(
    original_sender: &str,
    original_date: &str,
    original_body: &str,
    reply_text: &str,
    quote: bool,
) -> String {
    if !quote {
        return reply_text.to_owned();
    }
    let quoted = original_body
        .lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{reply_text}\n\nOn {original_date}, {original_sender} wrote:\n{quoted}")
}

/// Build a forward body.
pub fn build_forward_body(
    original_sender: &str,
    original_date: &str,
    original_subject: &str,
    original_body: &str,
    note: Option<&str>,
) -> String {
    let header =
        format!("From: {original_sender}\nDate: {original_date}\nSubject: {original_subject}");
    let note_line = note.map(|n| format!("{n}\n\n")).unwrap_or_default();
    format!("{note_line}--- Forwarded message ---\n{header}\n\n{original_body}")
}

/// Build an EML representation of a forwarded message.
pub fn build_eml_forward(
    original_sender: &str,
    original_date: &str,
    original_subject: &str,
    original_body: &str,
    forward_to: &[String],
) -> String {
    let to = forward_to.join(", ");
    format!(
        "From: makosh@local\r\nTo: {to}\r\nSubject: Fwd: {original_subject}\r\nDate: {}\r\nContent-Type: message/rfc822\r\n\r\nFrom: {original_sender}\r\nDate: {original_date}\r\nSubject: {original_subject}\r\n\r\n{original_body}",
        Utc::now().to_rfc2822()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reply_with_quote() {
        let b = build_reply_body(
            "alice@ex.com",
            "Mon, 01 Jan 2026",
            "Hello\nHow are you?",
            "I'm fine",
            true,
        );
        assert!(b.contains("> Hello"));
        assert!(b.contains("I'm fine"));
    }
    #[test]
    fn reply_without_quote() {
        let b = build_reply_body("a@b.com", "d", "orig", "reply", false);
        assert_eq!(b, "reply");
    }
    #[test]
    fn forward_with_note() {
        let b = build_forward_body("s@e.com", "d", "subj", "body", Some("FYI"));
        assert!(b.contains("FYI"));
        assert!(b.contains("--- Forwarded"));
    }
    #[test]
    fn forward_eml_format() {
        let b = build_eml_forward("s@e.com", "d", "subj", "body", &["to@e.com".into()]);
        assert!(b.contains("Content-Type: message/rfc822"));
    }
    #[test]
    fn reply_config_defaults() {
        let c = ReplyConfig::default();
        assert!(c.quote_original);
        assert!(!c.reply_all);
    }
}
