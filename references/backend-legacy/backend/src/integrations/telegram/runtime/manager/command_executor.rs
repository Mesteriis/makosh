use chrono::{Duration, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::commands;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::store::TelegramStore;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use makosh_events_postgres::store::EventStore;

use super::TelegramRuntimeManager;
use super::command_executor_dispatch::{DispatchOutcome, dispatch_command};
use super::command_executor_media::{emit_media_upload_event, media_upload_progress_payload};
use super::realtime_events::command_event_payload;
use super::topic_events::upsert_topic_snapshot;

const RETRY_BASE_DELAY_SECONDS: i64 = 30;
const RETRY_MAX_DELAY_SECONDS: i64 = 15 * 60;
const STALE_EXECUTION_LOCK_SECONDS: i64 = 120;

/// Processes due provider-write commands for active Telegram account actors.
///
/// Actor dispatch does not mean provider success. Commands that only get an ACK
/// from TDLib stay `executing` with `reconciliation_status=awaiting_provider`;
/// `completed` is reserved for provider-observed state or TDLib calls that
/// return a provider message snapshot.
pub async fn execute_queued_commands(
    telegram_store: &TelegramStore,
    runtime: &TelegramRuntimeManager,
    event_bus: &InMemoryEventBus,
    per_account_limit: i64,
) {
    let pool = telegram_store.pool();
    let now = Utc::now();
    let stale_before = now - Duration::seconds(STALE_EXECUTION_LOCK_SECONDS);
    match commands::recover_stale_executing_commands(pool, now, stale_before).await {
        Ok(commands) => {
            for command in commands {
                let status = command.status.clone();
                emit_command_event(
                    event_bus,
                    pool,
                    &command,
                    &status,
                    json!({"source": "stale_recovery", "error": command.last_error.clone()}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "command executor: failed to recover stale commands");
        }
    }

    let account_ids = match runtime.active_account_ids() {
        Ok(ids) => ids,
        Err(error) => {
            tracing::warn!(error = %error, "command executor: failed to list active accounts");
            return;
        }
    };

    for account_id in account_ids {
        execute_account_commands(
            telegram_store,
            runtime,
            event_bus,
            &account_id,
            per_account_limit,
        )
        .await;
    }
}

async fn execute_account_commands(
    telegram_store: &TelegramStore,
    runtime: &TelegramRuntimeManager,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    limit: i64,
) {
    let pool = telegram_store.pool();
    let now = Utc::now();
    let commands =
        match commands::claim_due_commands_for_execution(pool, account_id, now, limit).await {
            Ok(commands) => commands,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    account_id = %account_id,
                    "command executor: failed to claim due commands"
                );
                return;
            }
        };

    if commands.is_empty() {
        return;
    }

    let command_tx = match runtime.actor_command_tx(account_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(
                error = %error,
                account_id = %account_id,
                "command executor: failed to get actor command channel"
            );
            return;
        }
    };

    for command in commands {
        emit_command_event(
            event_bus,
            pool,
            &command,
            "executing",
            json!({"source": "command_executor", "phase": "claimed"}),
        )
        .await;
        if command.command_kind == "send_media" {
            emit_media_upload_event(
                event_bus,
                pool,
                &command,
                telegram_event_types::MEDIA_UPLOAD_PROGRESS,
                media_upload_progress_payload(
                    &command,
                    "dispatching_to_provider",
                    "Uploading local media to Telegram",
                ),
            )
            .await;
        }

        let result = dispatch_command(pool, &command, command_tx.clone()).await;
        handle_dispatch_result(telegram_store, event_bus, &command, result).await;
    }
}

async fn handle_dispatch_result(
    telegram_store: &TelegramStore,
    event_bus: &InMemoryEventBus,
    command: &TelegramProviderWriteCommand,
    result: Result<DispatchOutcome, TelegramError>,
) {
    let pool = telegram_store.pool();
    let now = Utc::now();
    match result {
        Ok(DispatchOutcome::ObservedMessage(snapshot)) => {
            let import_batch_id = format!(
                "telegram-command:{}:{}",
                command.account_id,
                command.command_id.trim()
            );
            let projection = match telegram_store
                .ingest_tdlib_message_snapshot(&command.account_id, &snapshot, &import_batch_id)
                .await
            {
                Ok(result) => result,
                Err(error) => {
                    handle_command_error(pool, event_bus, command, error, now).await;
                    return;
                }
            };
            if let Err(error) = telegram_store
                .publish_observed_message_raw_signal(&projection, Some(event_bus))
                .await
            {
                handle_command_error(pool, event_bus, command, error, now).await;
                return;
            }
            let provider_state = json!({
                "provider_chat_id": snapshot.provider_chat_id,
                "provider_message_id": snapshot.provider_message_id,
                "delivery_state": snapshot.delivery_state.as_str(),
                "observed_via": "tdlib_returned_message",
                "raw_record_id": projection.raw_record_id.clone(),
                "message_id": projection.message_id.clone(),
            });
            let result_payload = json!({
                "provider_chat_id": snapshot.provider_chat_id,
                "provider_message_id": snapshot.provider_message_id,
                "delivery_state": snapshot.delivery_state.as_str(),
                "raw_record_id": projection.raw_record_id.clone(),
                "message_id": projection.message_id.clone(),
            });
            if let Err(error) = commands::mark_command_reconciled(
                pool,
                &command.command_id,
                now,
                provider_state,
                result_payload,
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to mark command reconciled"
                );
            }
            let reconciled_event_payload = json!({
                "source": "command_executor",
                "reconciliation_status": "observed",
                "provider_observed_at": now,
                "reconciled_at": now,
            });
            emit_command_event(
                event_bus,
                pool,
                command,
                "completed",
                reconciled_event_payload.clone(),
            )
            .await;
            emit_command_event_type(
                event_bus,
                pool,
                command,
                telegram_event_types::COMMAND_RECONCILED,
                "completed",
                reconciled_event_payload,
            )
            .await;
            if command.command_kind == "send_media" {
                emit_media_upload_event(
                    event_bus,
                    pool,
                    command,
                    telegram_event_types::MEDIA_UPLOAD_COMPLETED,
                    json!({
                        "status": "completed",
                        "provider_message_id": snapshot.provider_message_id,
                        "delivery_state": snapshot.delivery_state.as_str(),
                        "message_id": projection.message_id,
                    }),
                )
                .await;
            }
        }
        Ok(DispatchOutcome::ObservedTopic(snapshot)) => {
            let topic = match upsert_topic_snapshot(
                telegram_store,
                &command.account_id,
                &command.provider_chat_id,
                &snapshot,
            )
            .await
            {
                Ok(Some(topic)) => topic,
                Ok(None) => {
                    handle_command_error(
                        pool,
                        event_bus,
                        command,
                        TelegramError::InvalidRequest(
                            "topic create observed for unknown telegram chat".to_owned(),
                        ),
                        now,
                    )
                    .await;
                    return;
                }
                Err(error) => {
                    handle_command_error(pool, event_bus, command, error, now).await;
                    return;
                }
            };
            let provider_state = json!({
                "provider_chat_id": command.provider_chat_id,
                "provider_topic_id": snapshot.provider_topic_id,
                "topic_id": topic.topic_id,
                "title": topic.title,
                "is_closed": topic.is_closed,
                "observed_via": "tdlib_returned_topic",
            });
            let result_payload = json!({
                "provider_chat_id": command.provider_chat_id,
                "provider_topic_id": snapshot.provider_topic_id,
                "topic_id": topic.topic_id,
                "title": topic.title,
                "is_closed": topic.is_closed,
            });
            if let Err(error) = commands::mark_command_reconciled(
                pool,
                &command.command_id,
                now,
                provider_state,
                result_payload,
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to mark topic command reconciled"
                );
            }
            emit_command_event(
                event_bus,
                pool,
                command,
                "completed",
                json!({
                    "source": "command_executor",
                    "reconciliation_status": "observed",
                    "provider_observed_at": now,
                    "reconciled_at": now,
                    "provider_topic_id": snapshot.provider_topic_id,
                    "topic_id": topic.topic_id,
                }),
            )
            .await;
            emit_command_event_type(
                event_bus,
                pool,
                command,
                telegram_event_types::COMMAND_RECONCILED,
                "completed",
                json!({
                    "source": "command_executor",
                    "reconciliation_status": "observed",
                    "provider_observed_at": now,
                    "reconciled_at": now,
                    "provider_topic_id": snapshot.provider_topic_id,
                    "topic_id": topic.topic_id,
                }),
            )
            .await;
        }
        Ok(DispatchOutcome::AwaitingProvider) => {
            if let Err(error) = commands::mark_command_awaiting_provider(
                pool,
                &command.command_id,
                now,
                json!({"dispatch": "accepted", "awaiting_provider_observed": true}),
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to persist awaiting-provider state"
                );
            }
        }
        Err(error) => {
            handle_command_error(pool, event_bus, command, error, now).await;
        }
    }
}

async fn handle_command_error(
    pool: &PgPool,
    event_bus: &InMemoryEventBus,
    command: &TelegramProviderWriteCommand,
    error: TelegramError,
    now: chrono::DateTime<Utc>,
) {
    tracing::warn!(
        error = %error,
        command_id = %command.command_id,
        command_kind = %command.command_kind,
        "command executor: command failed"
    );
    if is_dead_letter_error(&error) || command.retry_count >= command.max_retries {
        if let Err(update_error) =
            commands::dead_letter_command(pool, &command.command_id, now, &error.to_string()).await
        {
            tracing::warn!(
                error = %update_error,
                command_id = %command.command_id,
                "command executor: failed to dead-letter command"
            );
        }
        emit_command_event(
            event_bus,
            pool,
            command,
            "dead_letter",
            json!({"source": "command_executor", "error": error.to_string(), "dead_lettered_at": now}),
        )
        .await;
        if command.command_kind == "send_media" {
            emit_media_upload_event(
                event_bus,
                pool,
                command,
                telegram_event_types::MEDIA_UPLOAD_FAILED,
                json!({"status": "dead_letter", "error": error.to_string(), "dead_lettered_at": now}),
            )
            .await;
        }
    } else {
        let next_attempt_at = next_attempt_at(now, command.retry_count);
        if let Err(update_error) = commands::schedule_command_retry(
            pool,
            &command.command_id,
            now,
            next_attempt_at,
            &error.to_string(),
        )
        .await
        {
            tracing::warn!(
                error = %update_error,
                command_id = %command.command_id,
                "command executor: failed to schedule command retry"
            );
        }
        emit_command_event(
            event_bus,
            pool,
            command,
            "retrying",
            json!({
                "source": "command_executor",
                "error": error.to_string(),
                "next_attempt_at": next_attempt_at,
            }),
        )
        .await;
        if command.command_kind == "send_media" {
            emit_media_upload_event(
                event_bus,
                pool,
                command,
                telegram_event_types::MEDIA_UPLOAD_FAILED,
                json!({"status": "retrying", "error": error.to_string(), "next_attempt_at": next_attempt_at}),
            )
            .await;
        }
    }
}

fn next_attempt_at(now: chrono::DateTime<Utc>, completed_attempts: i32) -> chrono::DateTime<Utc> {
    let retry_index = completed_attempts.saturating_sub(1).max(0) as u32;
    let factor = 1_i64.checked_shl(retry_index.min(30)).unwrap_or(i64::MAX);
    let delay_seconds = RETRY_BASE_DELAY_SECONDS
        .saturating_mul(factor)
        .min(RETRY_MAX_DELAY_SECONDS);
    now + Duration::seconds(delay_seconds)
}

fn is_dead_letter_error(error: &TelegramError) -> bool {
    matches!(error, TelegramError::InvalidRequest(_))
}

async fn emit_command_event(
    event_bus: &InMemoryEventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    status: &str,
    extra_payload: Value,
) {
    emit_command_event_type(
        event_bus,
        pool,
        command,
        telegram_event_types::COMMAND_STATUS_CHANGED,
        status,
        extra_payload,
    )
    .await;
}

async fn emit_command_event_type(
    event_bus: &InMemoryEventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    status: &str,
    extra_payload: Value,
) {
    let now = Utc::now();
    let payload = command_event_payload(command, status, extra_payload);

    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_command"}),
    )
    .payload(payload)
    .build();

    let Ok(event) = event else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "command executor: failed to append event");
    }

    let _ = event_bus.broadcast(event);
}
