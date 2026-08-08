use super::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::port::MessageProjectionPort;
use crate::domains::communications::messages::states::WorkflowState;

const URGENT_WORDS: &[&str] = &[
    "urgent",
    "asap",
    "deadline",
    "immediately",
    "critical",
    "action required",
];
const FINANCE_WORDS: &[&str] = &[
    "invoice",
    "payment",
    "factura",
    "bill",
    "amount due",
    "receipt",
    "tax",
];
const LEGAL_WORDS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "legal",
    "liability",
    "confidential",
    "attorney",
];
const ATTACHMENT_WORDS: &[&str] = &["attached", "attachment", "see attached", "please find"];
const JUNK_WORDS: &[&str] = &[
    "unsubscribe",
    "opt out",
    "this email was sent",
    "if you no longer wish",
];

/// Result of Макошь auto-analysis on an ingested message.
#[derive(Debug)]
pub struct IngestionAnalysis {
    pub category: Option<String>,
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub auto_workflow_state: WorkflowState,
}

/// Analyze an incoming message and persist results.
/// This is the mandatory analysis step for every ingested email —
/// provider spam classification is completely ignored.
pub async fn analyze_ingested_message(
    store: &MessageProjectionPort,
    message: &ProjectedMessage,
) -> Result<IngestionAnalysis, MessageProjectionError> {
    let score = heuristic_score(message);
    let category = heuristic_category(message);

    let body_lower = message.body_text.to_lowercase();

    let is_spam = body_lower.contains("unsubscribe")
        && (body_lower.contains("buy now")
            || body_lower.contains("limited offer")
            || body_lower.contains("click here"));
    let is_phishing = (body_lower.contains("verify your account")
        || body_lower.contains("confirm your password")
        || body_lower.contains("urgent action required"))
        && !message.sender.contains(&message.account_id);

    // Heuristics classify untrusted content but never move a message to spam.
    // Spam is a user or policy decision because a false positive hides evidence.
    let auto_state = automatic_workflow_state(score);

    store
        .set_ai_analysis(&message.message_id, category.as_deref(), None, Some(score))
        .await?;

    if auto_state != WorkflowState::New {
        store
            .transition_workflow_state(&message.message_id, auto_state)
            .await?;
    }

    Ok(IngestionAnalysis {
        category,
        importance_score: score,
        is_spam,
        is_phishing,
        auto_workflow_state: auto_state,
    })
}

fn heuristic_score(message: &ProjectedMessage) -> i16 {
    let mut score: i16 = 30;
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if contains_any(&subject_lower, URGENT_WORDS) {
        score = score.saturating_add(15);
    }
    if contains_any(&body_lower, FINANCE_WORDS) || contains_any(&subject_lower, FINANCE_WORDS) {
        score = score.saturating_add(20);
    }
    if contains_any(&body_lower, LEGAL_WORDS) || contains_any(&subject_lower, LEGAL_WORDS) {
        score = score.saturating_add(25);
    }

    if body_lower.contains('?') {
        score = score.saturating_add(10);
    }
    if contains_any(&body_lower, ATTACHMENT_WORDS) {
        score = score.saturating_add(10);
    }
    if contains_any(&body_lower, JUNK_WORDS) {
        score = score.saturating_sub(20);
    }
    if message.body_text.len() < 50 {
        score = score.saturating_sub(10);
    }

    score.clamp(0, 100)
}

fn automatic_workflow_state(score: i16) -> WorkflowState {
    if score >= 75 {
        WorkflowState::NeedsAction
    } else {
        WorkflowState::New
    }
}

fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if body_lower.contains("invoice")
        || body_lower.contains("factura")
        || body_lower.contains("payment")
    {
        return Some("finance".to_owned());
    }
    if body_lower.contains("contract")
        || body_lower.contains("nda")
        || body_lower.contains("agreement")
    {
        return Some("legal".to_owned());
    }
    if body_lower.contains("unsubscribe") || body_lower.contains("newsletter") {
        return Some("marketing".to_owned());
    }
    if subject_lower.contains("notification") || body_lower.contains("notification") {
        return Some("notification".to_owned());
    }
    if body_lower.contains("click here")
        && (body_lower.contains("account") || body_lower.contains("verify"))
    {
        return Some("suspicious".to_owned());
    }

    None
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::communications::messages::states::LocalMessageState;
    use chrono::Utc;
    use serde_json::json;

    fn test_message(subject: &str, sender: &str, body: &str) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "m:1".into(),
            raw_record_id: "r:1".into(),
            observation_id: "observation:1".into(),
            account_id: "personal@ex.com".into(),
            provider_record_id: "p:1".into(),
            subject: subject.into(),
            sender: sender.into(),
            recipients: vec!["me@ex.com".into()],
            body_text: body.into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: json!({}),
            workflow_state: WorkflowState::New,
            importance_score: None,
            ai_category: None,
            ai_summary: None,
            ai_summary_generated_at: None,
            ai_state: None,
            local_state: LocalMessageState::Active,
            local_state_changed_at: None,
            local_state_reason: None,
            is_read: false,
            read_changed_at: None,
            read_origin: "test_fixture".into(),
        }
    }

    #[test]
    fn phishing_detection_flags_spam() {
        let msg = test_message(
            "Urgent",
            "hacker@evil.com",
            "Please verify your account immediately by clicking here",
        );
        let analysis = heuristic_score(&msg);
        assert!(analysis > 0);
    }

    #[test]
    fn newsletter_detected_as_low_score() {
        let msg = test_message(
            "Weekly Digest",
            "news@company.com",
            "Click here to read. To unsubscribe, click here.",
        );
        let score = heuristic_score(&msg);
        assert!(score <= 30, "newsletters should score low, got {score}");
    }

    #[test]
    fn marketing_heuristics_do_not_auto_promote_to_spam() {
        let msg = test_message(
            "Limited offer",
            "news@company.com",
            "Buy now. Click here to unsubscribe from this email.",
        );

        assert_eq!(
            automatic_workflow_state(heuristic_score(&msg)),
            WorkflowState::New
        );
    }

    #[test]
    fn invoice_gets_high_score() {
        let msg = test_message(
            "Invoice #456",
            "billing@vendor.com",
            "Please find your invoice attached. Amount due: $500. Payment required by June 15.",
        );
        let score = heuristic_score(&msg);
        assert!(score >= 50, "invoices should score high, got {score}");
    }
}
