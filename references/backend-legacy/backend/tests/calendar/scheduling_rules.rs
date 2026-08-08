use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::rules::CalendarRuleStore;
use makosh_hub_backend::domains::calendar::scheduling::{DeadlineStore, FocusBlockStore};
use serde_json::json;

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn deadlines_and_focus_blocks_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let deadline_store = DeadlineStore::new(pool.clone());
    let fb_store = FocusBlockStore::new(pool);
    let suffix = unique_suffix();
    let now = Utc::now();

    let deadline = deadline_store
        .create(
            &format!("Deadline {suffix}"),
            now + Duration::days(7),
            Some("high"),
            None,
            None,
        )
        .await
        .expect("create deadline");
    assert_eq!(deadline.severity, "high");
    assert_eq!(deadline.status, "active");

    let deadlines = deadline_store.list(None, 50).await.expect("list deadlines");
    assert!(
        deadlines
            .iter()
            .any(|item| item.title == format!("Deadline {suffix}"))
    );

    let focus_block = fb_store
        .create(
            &format!("Focus {suffix}"),
            now,
            now + Duration::hours(2),
            Some("Deep work"),
            None,
            Some("high"),
        )
        .await
        .expect("create focus block");
    assert_eq!(focus_block.protection_level, "high");

    let blocks = fb_store
        .list(None, None, 50)
        .await
        .expect("list focus blocks");
    assert!(blocks.iter().any(|b| b.title == format!("Focus {suffix}")));
}

#[tokio::test]
async fn calendar_rules_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = CalendarRuleStore::new(pool);
    let suffix = unique_suffix();

    let rule = store
        .create(
            &format!("Rule {suffix}"),
            Some("Auto-prepare meetings with clients"),
            json!({"trigger": "event_type=meeting", "action": "generate_brief"}),
            Some("suggest_only"),
        )
        .await
        .expect("create rule");
    assert!(rule.rule_id.starts_with("rule:v1:"));
    assert_eq!(rule.approval_mode, "suggest_only");

    let list = store.list().await.expect("list rules");
    assert!(list.iter().any(|r| r.rule_id == rule.rule_id));

    store.delete(&rule.rule_id).await.expect("delete rule");
    let list_after_delete = store.list().await.expect("list after delete");
    assert!(!list_after_delete.iter().any(|r| r.rule_id == rule.rule_id));
}
