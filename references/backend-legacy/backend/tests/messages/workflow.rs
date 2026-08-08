use makosh_hub_backend::domains::communications::messages::models::WorkflowStateCount;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::states::{
    LocalMessageState, WorkflowState,
};

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

#[test]
fn workflow_state_from_str_all_valid() {
    for (input, expected) in [
        ("new", WorkflowState::New),
        ("reviewed", WorkflowState::Reviewed),
        ("needs_action", WorkflowState::NeedsAction),
        ("waiting", WorkflowState::Waiting),
        ("done", WorkflowState::Done),
        ("archived", WorkflowState::Archived),
        ("muted", WorkflowState::Muted),
        ("spam", WorkflowState::Spam),
    ] {
        let state = input.parse::<WorkflowState>().expect("valid state");
        assert_eq!(state, expected, "from_str({input}) should match");
    }
}

#[test]
fn workflow_state_from_str_invalid() {
    assert!("".parse::<WorkflowState>().is_err());
    assert!("invalid_state".parse::<WorkflowState>().is_err());
    assert!("NEW".parse::<WorkflowState>().is_err());
}

#[test]
fn workflow_state_as_str_roundtrips() {
    let states = [
        WorkflowState::New,
        WorkflowState::Reviewed,
        WorkflowState::NeedsAction,
        WorkflowState::Waiting,
        WorkflowState::Done,
        WorkflowState::Archived,
        WorkflowState::Muted,
        WorkflowState::Spam,
    ];

    for state in &states {
        let s = state.as_str();
        let roundtripped = s.parse::<WorkflowState>().expect("roundtrip");
        assert_eq!(*state, roundtripped, "roundtrip for {s}");
    }
}

#[test]
fn workflow_state_valid_transitions() {
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Reviewed
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::NeedsAction
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Muted
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Spam
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Reviewed,
        &WorkflowState::New
    ));

    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Done
    ));
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Waiting
    ));

    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Done
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Waiting
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Spam,
        &WorkflowState::New
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Done,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Archived,
        &WorkflowState::Reviewed
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Archived,
        &WorkflowState::NeedsAction
    ));

    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::New
    ));
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::Done,
        &WorkflowState::Done
    ));
}

#[test]
fn workflow_state_serde_roundtrips() {
    let json = serde_json::to_string(&WorkflowState::NeedsAction).expect("serialize");
    assert_eq!(json, "\"needs_action\"");

    let deserialized: WorkflowState =
        serde_json::from_str("\"needs_action\"").expect("deserialize");
    assert_eq!(deserialized, WorkflowState::NeedsAction);

    let deserialized_new: WorkflowState = serde_json::from_str("\"new\"").expect("deserialize");
    assert_eq!(deserialized_new, WorkflowState::New);
}

#[tokio::test]
async fn message_workflow_state_transition_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("workflow state transition").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_workflow_{suffix}");
    let raw_record_id = format!("raw_workflow_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Workflow Gmail",
        format!("workflow-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &format!("provider-workflow-{suffix}"),
        "Workflow test subject",
        "Workflow test body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");
    assert_eq!(projected.workflow_state.as_str(), "new");

    let updated = message_store
        .transition_workflow_state(&projected.message_id, WorkflowState::NeedsAction)
        .await
        .expect("transition to needs_action");
    assert_eq!(updated.workflow_state.as_str(), "needs_action");

    let updated = message_store
        .transition_workflow_state(&updated.message_id, WorkflowState::Done)
        .await
        .expect("transition to done");
    assert_eq!(updated.workflow_state.as_str(), "done");

    let updated = message_store
        .transition_workflow_state(&updated.message_id, WorkflowState::Archived)
        .await
        .expect("transition to archived");
    assert_eq!(updated.workflow_state.as_str(), "archived");

    let fetched = message_store
        .message(&projected.message_id)
        .await
        .expect("fetch message")
        .expect("message exists");
    assert_eq!(fetched.workflow_state.as_str(), "archived");
}

#[tokio::test]
async fn message_state_counts_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message state counts").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_counts_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Counts Gmail",
        format!("counts-{suffix}@example.com"),
    )
    .await;

    for i in 0..2 {
        let raw = record_raw_email_message(
            &communication_store,
            &account_id,
            &format!("raw_counts_{suffix}_{i}"),
            &format!("provider-counts-{suffix}-{i}"),
            &format!("Counts subject {i}"),
            &format!("Counts body {i}"),
        )
        .await;
        project_raw_email_message(&message_store, &raw)
            .await
            .expect("project message");
    }

    let counts = message_store
        .count_messages_by_state(Some(&account_id))
        .await
        .expect("count messages");

    let new_count = counts
        .iter()
        .find(|c| c.state.as_str() == "new")
        .map(|c| c.count)
        .unwrap_or(0);
    assert!(new_count >= 2, "expected at least 2 new messages");

    let messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            None,
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list messages");
    assert!(!messages.is_empty());

    message_store
        .transition_workflow_state(&messages[0].message.message_id, WorkflowState::Done)
        .await
        .expect("transition to done");

    let counts = message_store
        .count_messages_by_state(Some(&account_id))
        .await
        .expect("count messages after transition");

    let done_count = counts
        .iter()
        .find(|c| c.state.as_str() == "done")
        .map(|c| c.count)
        .unwrap_or(0);
    assert_eq!(done_count, 1, "expected 1 done message");
}

#[tokio::test]
async fn message_list_filtering_by_state_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message list filtering").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_filter_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Filter Gmail",
        format!("filter-{suffix}@example.com"),
    )
    .await;

    for i in 0..3 {
        let raw = record_raw_email_message(
            &communication_store,
            &account_id,
            &format!("raw_filter_{suffix}_{i}"),
            &format!("provider-filter-{suffix}-{i}"),
            &format!("Filter subject {i}"),
            &format!("Filter body {i}"),
        )
        .await;
        project_raw_email_message(&message_store, &raw)
            .await
            .expect("project message");
    }

    let new_messages = message_store
        .list_messages(
            Some(&account_id),
            Some(WorkflowState::New),
            None,
            None,
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list new messages");
    assert!(new_messages.len() >= 3, "expected at least 3 new messages");

    let done_messages = message_store
        .list_messages(
            Some(&account_id),
            Some(WorkflowState::Done),
            None,
            None,
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list done messages");
    assert_eq!(done_messages.len(), 0, "expected 0 done messages");
}

#[test]
fn workflow_state_count_serialization() {
    let count = WorkflowStateCount {
        state: WorkflowState::NeedsAction,
        count: 42,
    };
    let json = serde_json::to_value(&count).expect("serialize");
    assert_eq!(json["state"], "needs_action");
    assert_eq!(json["count"], 42);
}
