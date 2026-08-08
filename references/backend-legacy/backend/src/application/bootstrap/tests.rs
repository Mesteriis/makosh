use chrono::Utc;
use serde_json::json;

use super::*;

#[test]
fn telegram_runtime_reconciliation_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://telegram-runtime-reconciliation-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(telegram::register_telegram_runtime_reconciliation(
        &database_url
    ));
    assert!(!telegram::register_telegram_runtime_reconciliation(
        &database_url
    ));
}

#[test]
fn telegram_runtime_reconciliation_only_runs_enabled_runnable_accounts() {
    let now = Utc::now();
    let account = |config| makosh_communications_api::accounts::ProviderAccount {
        account_id: "telegram-account".to_owned(),
        provider_kind: makosh_communications_api::accounts::CommunicationProviderKind::TelegramUser,
        display_name: "Telegram".to_owned(),
        external_account_id: "telegram:1".to_owned(),
        config,
        created_at: now,
        updated_at: now,
    };

    assert!(telegram::runtime_reconciliation_enabled(&account(json!({
        "runtime": "fixture"
    }))));
    assert!(telegram::runtime_reconciliation_enabled(&account(json!({
        "runtime": "tdlib_qr_authorized"
    }))));
    assert!(!telegram::runtime_reconciliation_enabled(&account(json!({
        "runtime": "tdlib_qr_authorized",
        "runtime_enabled": false
    }))));
    assert!(!telegram::runtime_reconciliation_enabled(&account(json!({
        "runtime": "fixture",
        "lifecycle_state": "logged_out"
    }))));
    assert!(!telegram::runtime_reconciliation_enabled(&account(json!({
        "runtime": "live_blocked"
    }))));
}

#[test]
fn outbox_delivery_scheduler_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://outbox-scheduler-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(mail::register_mail_outbox_delivery_scheduler(&database_url));
    assert!(!mail::register_mail_outbox_delivery_scheduler(
        &database_url
    ));
}

#[test]
fn event_outbox_dispatcher_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://event-outbox-dispatcher-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(core::dispatchers::register_event_outbox_dispatcher(
        &database_url
    ));
    assert!(!core::dispatchers::register_event_outbox_dispatcher(
        &database_url
    ));
}

#[test]
fn signal_replay_dispatcher_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://signal-replay-dispatcher-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(core::dispatchers::register_signal_replay_dispatcher(
        &database_url
    ));
    assert!(!core::dispatchers::register_signal_replay_dispatcher(
        &database_url
    ));
}

#[test]
fn zoom_token_maintenance_scheduler_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://zoom-token-maintenance-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(zoom::tasks::register_zoom_token_maintenance_scheduler(
        &database_url
    ));
    assert!(!zoom::tasks::register_zoom_token_maintenance_scheduler(
        &database_url
    ));
}

#[test]
fn zoom_recording_sync_scheduler_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://zoom-recording-sync-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(zoom::tasks::register_zoom_recording_sync_scheduler(
        &database_url
    ));
    assert!(!zoom::tasks::register_zoom_recording_sync_scheduler(
        &database_url
    ));
}

#[test]
fn zoom_retention_cleanup_scheduler_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://zoom-retention-cleanup-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(zoom::tasks::register_zoom_retention_cleanup_scheduler(
        &database_url
    ));
    assert!(!zoom::tasks::register_zoom_retention_cleanup_scheduler(
        &database_url
    ));
}

#[test]
fn zoom_calendar_matching_consumer_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://zoom-calendar-matching-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(zoom::tasks::register_zoom_calendar_matching_consumer(
        &database_url
    ));
    assert!(!zoom::tasks::register_zoom_calendar_matching_consumer(
        &database_url
    ));
}

#[test]
fn zoom_signal_detection_consumer_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://zoom-signal-detection-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(zoom::tasks::register_zoom_signal_detection_consumer(
        &database_url
    ));
    assert!(!zoom::tasks::register_zoom_signal_detection_consumer(
        &database_url
    ));
}

#[test]
fn zoom_participant_identity_consumer_registration_is_once_per_database_url() {
    let database_url = format!(
        "postgres://zoom-participant-identity-test/{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    assert!(zoom::tasks::register_zoom_participant_identity_consumer(
        &database_url
    ));
    assert!(!zoom::tasks::register_zoom_participant_identity_consumer(
        &database_url
    ));
}
