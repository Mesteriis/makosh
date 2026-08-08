use makosh_events_api::NewEventEnvelope;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use super::errors::MailSyncError;
use super::models::runs::MailSyncRun;

const EVENT_TYPE_STARTED: &str = "mail.sync.started";
const EVENT_TYPE_PROGRESS: &str = "mail.sync.progress";
const EVENT_TYPE_COMPLETED: &str = "mail.sync.completed";
const EVENT_TYPE_FAILED: &str = "mail.sync.failed";
const EVENT_TYPE_SKIPPED: &str = "mail.sync.skipped";

pub(super) fn sync_run_started_event(run: &MailSyncRun) -> Result<NewEventEnvelope, MailSyncError> {
    sync_run_event(EVENT_TYPE_STARTED, run)
}

pub(super) fn sync_run_progress_event(
    run: &MailSyncRun,
) -> Result<NewEventEnvelope, MailSyncError> {
    sync_run_event(EVENT_TYPE_PROGRESS, run)
}

pub(super) fn sync_run_finished_event(
    run: &MailSyncRun,
) -> Result<NewEventEnvelope, MailSyncError> {
    let event_type = match run.status.as_str() {
        "completed" => EVENT_TYPE_COMPLETED,
        "failed" => EVENT_TYPE_FAILED,
        "skipped" => EVENT_TYPE_SKIPPED,
        _ => EVENT_TYPE_PROGRESS,
    };
    sync_run_event(event_type, run)
}

fn sync_run_event(event_type: &str, run: &MailSyncRun) -> Result<NewEventEnvelope, MailSyncError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_sync_event:{event_type}:{}:{:x}",
            run.run_id,
            system_time_nanos()
        ),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_background_sync" }),
        json!({
            "kind": "mail_sync_run",
            "id": run.run_id,
            "run_id": run.run_id,
            "account_id": run.account_id,
        }),
    )
    .payload(json!({
        "run_id": run.run_id,
        "account_id": run.account_id,
        "trigger": run.trigger,
        "status": run.status,
        "phase": run.phase,
        "progress_mode": run.progress_mode,
        "progress_percent": run.progress_percent,
        "processed_messages": run.processed_messages,
        "estimated_total_messages": run.estimated_total_messages,
        "current_batch_size": run.current_batch_size,
        "fetched_messages": run.fetched_messages,
        "projected_messages": run.projected_messages,
        "upserted_personas": run.upserted_personas,
        "upserted_organizations": run.upserted_organizations,
        "checkpoint_saved": run.checkpoint_saved,
        "error_code": run.error_code,
        "next_run_at": run.next_run_at,
    }))
    .provenance(json!({
        "source_kind": "mail_sync_run",
        "source_id": run.run_id,
    }))
    .correlation_id(run.account_id.clone())
    .build()?)
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
