use super::{categories::EmailCategory, models::EmailAnalysis, service::EmailIntelligenceService};
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use chrono::Utc;
use serde_json::json;

fn test_message(subject: &str, body: &str) -> ProjectedMessage {
    ProjectedMessage {
        message_id: "msg:test:1".into(),
        raw_record_id: "raw:1".into(),
        observation_id: "observation:1".into(),
        account_id: "acct:1".into(),
        provider_record_id: "prov:1".into(),
        subject: subject.into(),
        sender: "sender@example.com".into(),
        recipients: vec!["recipient@example.com".into()],
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
fn heuristic_score_urgent_subject() {
    let msg = test_message("URGENT: Action Required", "Please respond ASAP");
    let score = EmailIntelligenceService::heuristic_score(&msg);
    assert!(score >= 35, "got {score}");
}

#[test]
fn heuristic_score_finance_body() {
    let msg = test_message(
        "Update",
        "Please find the invoice attached for payment. Amount due: $500",
    );
    let score = EmailIntelligenceService::heuristic_score(&msg);
    assert!(score >= 50, "got {score}");
}

#[test]
fn heuristic_score_marketing_body() {
    let msg = test_message(
        "Digest",
        "Click here. To unsubscribe, click here. If you no longer wish to receive...",
    );
    let score = EmailIntelligenceService::heuristic_score(&msg);
    assert!(score <= 30, "got {score}");
}

#[test]
fn heuristic_category_finance() {
    let msg = test_message("Invoice #123", "Here is your invoice for services");
    assert_eq!(
        EmailIntelligenceService::heuristic_category(&msg).as_deref(),
        Some("finance")
    );
}

#[test]
fn heuristic_category_legal() {
    let msg = test_message("Contract", "Please review the NDA and agreement");
    assert_eq!(
        EmailIntelligenceService::heuristic_category(&msg).as_deref(),
        Some("legal")
    );
}

#[test]
fn heuristic_category_none() {
    let msg = test_message("Hello", "Just checking in");
    assert!(EmailIntelligenceService::heuristic_category(&msg).is_none());
}

#[test]
fn email_analysis_accepts_model_output_without_runtime_metadata() {
    let analysis: EmailAnalysis = serde_json::from_value(json!({
        "category": "work",
        "summary": "Follow up with the sender.",
        "importance_score": 42,
        "is_spam": false,
        "is_phishing": false,
        "suggested_action": null,
        "extracted_deadline": null,
        "language": "en"
    }))
    .expect("runtime metadata is owned by Макошь, not requested from the model");

    assert!(analysis.model.is_empty());
    assert!(analysis.prompt_version.is_empty());
}

#[test]
fn heuristic_structured_summary_extracts_key_points_actions_risks_and_deadlines() {
    let msg = test_message(
        "Action Required: Contract review deadline",
        "Please review the NDA by Friday. The payment risk remains open. Confirm approval before EOD.",
    );

    let summary = EmailIntelligenceService::heuristic_structured_summary(&msg);

    assert!(
        summary
            .key_points
            .contains(&"Action Required: Contract review deadline".to_owned())
    );
    assert!(
        summary
            .action_items
            .iter()
            .any(|item| item.contains("Please review the NDA"))
    );
    assert!(
        summary
            .risks
            .iter()
            .any(|risk| risk.contains("payment risk"))
    );
    assert!(
        summary
            .deadlines
            .iter()
            .any(|deadline| deadline.contains("Friday"))
    );
}

#[test]
fn heuristic_structured_summary_extracts_mail_knowledge_candidates() {
    let msg = test_message(
        "Contract review with Acme Corp",
        "From: Ada Lovelace <ada@acme.example>\nPlease review the attached MSA and NDA before Friday. Meeting on Monday at 10:00 with Acme Corp.",
    );

    let summary = EmailIntelligenceService::heuristic_structured_summary(&msg);

    assert!(
        summary
            .event_candidates
            .iter()
            .any(|candidate| candidate.title.contains("Meeting on Monday"))
    );
    assert!(
        summary
            .persona_candidates
            .iter()
            .any(|candidate| candidate.title.contains("Ada Lovelace")
                && candidate.kind.as_deref() == Some("persona"))
    );
    assert!(
        summary
            .organization_candidates
            .iter()
            .any(|candidate| candidate.title.contains("acme.example"))
    );
    assert!(
        summary
            .document_candidates
            .iter()
            .any(|candidate| candidate.title.contains("MSA"))
    );
    assert!(
        summary
            .agreement_candidates
            .iter()
            .any(|candidate| candidate.title.contains("NDA"))
    );
}

#[test]
fn heuristic_structured_summary_is_bounded_and_deduplicated() {
    let msg = test_message(
        "Deadline reminder",
        "Deadline reminder. Deadline reminder. Please confirm. Please confirm.",
    );

    let summary = EmailIntelligenceService::heuristic_structured_summary(&msg);

    assert_eq!(summary.key_points, vec!["Deadline reminder"]);
    assert_eq!(summary.action_items, vec!["Please confirm"]);
    assert_eq!(summary.deadlines, vec!["Deadline reminder"]);
    assert!(summary.event_candidates.is_empty());
    assert_eq!(summary.persona_candidates.len(), 1);
    assert_eq!(summary.persona_candidates[0].title, "sender@example.com");
    assert_eq!(
        summary.persona_candidates[0].kind.as_deref(),
        Some("persona")
    );
}

#[test]
fn email_category_from_str_all_valid() {
    assert_eq!(
        EmailCategory::parse("critical"),
        Some(EmailCategory::Critical)
    );
    assert_eq!(EmailCategory::parse("spam"), Some(EmailCategory::Spam));
    assert_eq!(
        EmailCategory::parse("finance"),
        Some(EmailCategory::Finance)
    );
}
