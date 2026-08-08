use makosh_hub_backend::domains::calendar::brain::CalendarBrainService;
use makosh_hub_backend::domains::calendar::core::{
    agendas::EventAgendaStore, checklists::EventChecklistStore,
    context_packs::EventContextPackStore, participants::EventParticipantStore,
    relations::EventRelationStore,
};
use makosh_hub_backend::domains::calendar::events::event_store::CalendarEventStore;
use makosh_hub_backend::domains::calendar::health::CalendarWatchtowerService;
use makosh_hub_backend::domains::calendar::intelligence::CalendarIntelligenceService;
use makosh_hub_backend::domains::calendar::rules::CalendarRuleStore;
use makosh_hub_backend::domains::calendar::scheduling::{DeadlineStore, FocusBlockStore};

use super::support::{disconnected_pool, live_pool};

#[test]
fn intelligence_classify_event() {
    assert_eq!(
        CalendarIntelligenceService::classify_event("Weekly sync meeting", 3, 60),
        "meeting"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Tax deadline AEAT", 1, 0),
        "deadline"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Focus: deep work", 1, 120),
        "focus"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Flight to Madrid", 1, 180),
        "travel"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Coffee", 1, 15),
        "personal"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Sprint planning", 4, 90),
        "planning"
    );
}

#[tokio::test]
async fn intelligence_calculate_importance() {
    let base = CalendarIntelligenceService::calculate_importance("Coffee", 1, false, false);
    assert!(base > 0.0 && base < 1.0);

    let urgent = CalendarIntelligenceService::calculate_importance(
        "URGENT: client escalation",
        3,
        true,
        true,
    );
    assert!(urgent > base);
    assert!(urgent <= 1.0);
}

#[tokio::test]
async fn intelligence_calculate_readiness() {
    let full = CalendarIntelligenceService::calculate_readiness(true, true, true, true, true);
    assert_eq!(full, 1.0);

    let none = CalendarIntelligenceService::calculate_readiness(false, false, false, false, false);
    assert_eq!(none, 0.0);

    let partial = CalendarIntelligenceService::calculate_readiness(true, false, true, false, true);
    assert!(partial > 0.0 && partial < 1.0);
}

#[tokio::test]
async fn intelligence_detect_risks() {
    let none = CalendarIntelligenceService::detect_risks(true, true, true, true, false);
    assert!(none.is_empty());

    let missing = CalendarIntelligenceService::detect_risks(false, false, false, false, true);
    assert_eq!(missing.len(), 5);
    assert!(missing.contains(&"No agenda prepared".to_string()));
}

#[tokio::test]
async fn health_services_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };

    let brief = CalendarWatchtowerService::weekly_brief(&pool).await;
    assert!(brief.is_ok());

    let prep = CalendarWatchtowerService::events_needing_preparation(&pool).await;
    assert!(prep.is_ok());

    let no_outcomes = CalendarWatchtowerService::events_without_outcomes(&pool).await;
    assert!(no_outcomes.is_ok());

    let load = CalendarWatchtowerService::meeting_load_analysis(&pool).await;
    assert!(load.is_ok());
}

#[tokio::test]
async fn brain_services_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };

    let overview = CalendarBrainService::answer(&pool, "show me this week").await;
    assert!(overview.is_ok());

    let search = CalendarBrainService::search_events(&pool, "meeting").await;
    assert!(search.is_ok());
}

#[test]
fn sync_ics_export() {
    let ics = makosh_hub_backend::domains::calendar::sync::export_event_ics(
        "Test Meeting",
        Some("Description"),
        Some("Office"),
        "20260101T100000",
        "20260101T110000",
        Some("Europe/Madrid"),
    );
    assert!(ics.contains("BEGIN:VCALENDAR"));
    assert!(ics.contains("SUMMARY:Test Meeting"));
    assert!(ics.contains("DTSTART"));
    assert!(ics.contains("DTEND"));
}

#[test]
fn sync_markdown_export() {
    let md = makosh_hub_backend::domains::calendar::sync::export_event_md(
        "Test Meeting",
        Some("Description"),
        Some("Office"),
        "2026-01-01T10:00:00+01:00",
        "2026-01-01T11:00:00+01:00",
        &["John".into(), "Jane".into()],
    );
    assert!(md.contains("# Test Meeting"));
    assert!(md.contains("John"));
    assert!(md.contains("Jane"));
    assert!(md.contains("Office"));
}

#[tokio::test]
async fn disconnected_pool_creates_store() {
    let pool = disconnected_pool();
    let _store = CalendarEventStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_core_stores() {
    let pool = disconnected_pool();
    let _p = EventParticipantStore::new(pool.clone());
    let _r = EventRelationStore::new(pool.clone());
    let _c = EventContextPackStore::new(pool.clone());
    let _a = EventAgendaStore::new(pool.clone());
    let _cl = EventChecklistStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_scheduling_stores() {
    let pool = disconnected_pool();
    let _d = DeadlineStore::new(pool.clone());
    let _f = FocusBlockStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_rules_store() {
    let _store = CalendarRuleStore::new(disconnected_pool());
}
