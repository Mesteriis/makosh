use chrono::{DateTime, Utc};
use makosh_communications_api::accounts::ProviderAccountLookupPort;
use makosh_communications_api::commands::ProviderCommandProjectionPort;
use serde_json::json;
use sqlx::PgPool;

use super::account_scope::live_whatsapp_account_ids;
use super::command_conversion::row_to_whatsapp_provider_write_command;
use super::{WhatsAppProviderWriteCommand, WhatsappWebError};

pub(super) fn clamp_limit(limit: i64) -> i64 {
    limit.clamp(1, 200)
}

pub(super) async fn mirror_canonical_provider_command_for_pool(
    pool: &PgPool,
    command: &WhatsAppProviderWriteCommand,
) -> Result<(), WhatsappWebError> {
    let mirror =
        makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore::new(
            pool.clone(),
        );
    mirror
        .mirror_runtime_command(&super::command_conversion::communication_provider_command(
            command,
        ))
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))
}

pub(crate) async fn claim_due_live_commands_for_execution(
    pool: &PgPool,
    account_lookup: &dyn ProviderAccountLookupPort,
    now: DateTime<Utc>,
    limit: i64,
    account_id: Option<&str>,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let eligible_accounts = live_whatsapp_account_ids(account_lookup, account_id).await?;
    if eligible_accounts.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT command.command_id
            FROM whatsapp_provider_write_commands command
            WHERE command.status IN ('queued', 'retrying')
              AND command.retry_count < command.max_retries
              AND (command.next_attempt_at IS NULL OR command.next_attempt_at <= $1)
              AND command.confirmation_decision IN ('confirmed', 'not_required')
              AND command.capability_state IN ('available', 'degraded')
              AND (COALESCE(command.audit_metadata->>'session_restore_available', 'false') = 'true'
                   OR command.result_payload ? 'manual_retry_at')
              AND command.account_id = ANY($4)
              AND command.command_kind IN (
                'send_text', 'reply', 'forward', 'send_media', 'download_media', 'send_voice_note',
                'edit', 'delete', 'react', 'unreact', 'mark_read', 'mark_unread', 'archive',
                'unarchive', 'mute', 'unmute', 'pin', 'unpin', 'join_group', 'leave_group',
                'publish_status')
            ORDER BY COALESCE(command.next_attempt_at, command.created_at), command.created_at,
                     command.command_id
            LIMIT $2 FOR UPDATE SKIP LOCKED
        )
        UPDATE whatsapp_provider_write_commands command
        SET status = 'executing', retry_count = command.retry_count + 1, last_attempt_at = $1,
            locked_at = $1, locked_by = $3, last_error = NULL,
            reconciliation_status = 'awaiting_provider', updated_at = $1
        FROM due
        WHERE command.command_id = due.command_id
        RETURNING command.*
        "#,
    )
    .bind(now)
    .bind(limit)
    .bind("whatsapp-runtime-bridge-worker")
    .bind(eligible_accounts)
    .fetch_all(pool)
    .await?;
    let commands = rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(commands)
}

pub(crate) async fn import_canonical_provider_commands(
    pool: &PgPool,
    projection: &dyn ProviderCommandProjectionPort,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let canonical = projection
        .list_for_runtime_import("whatsapp", limit)
        .await
        .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
    let mut imported = Vec::with_capacity(canonical.len());
    for command in canonical {
        let Some(provider_chat_id) =
            super::command_conversion::canonical_provider_chat_id(&command)
        else {
            continue;
        };
        let status = if command.status == "confirmed" {
            "queued"
        } else {
            command.status.as_str()
        };
        let mut audit_metadata = command.audit_metadata.clone();
        if let Some(object) = audit_metadata.as_object_mut() {
            object.insert(
                "imported_from_canonical_provider_command".to_owned(),
                json!(true),
            );
            object.insert("canonical_imported_at".to_owned(), json!(now));
        }
        let row = sqlx::query(
            r#"UPDATE whatsapp_provider_write_commands
            SET provider_chat_id=$2, provider_message_id=$3, capability_state=$4,
                action_class=$5, confirmation_decision=$6, target_ref=$7, payload=$8,
                status=$9, retry_count=$10, max_retries=$11, last_error=$12,
                result_payload=$13, audit_metadata=$14, completed_at=$15,
                updated_at=GREATEST(updated_at, $16)
            WHERE command_id=$1 AND status IN ('queued','retrying','cancelled') RETURNING *"#,
        )
        .bind(&command.command_id)
        .bind(provider_chat_id)
        .bind(&command.provider_message_id)
        .bind(&command.capability_state)
        .bind(&command.action_class)
        .bind(&command.confirmation_decision)
        .bind(&command.target_ref)
        .bind(&command.payload)
        .bind(status)
        .bind(command.retry_count)
        .bind(command.max_retries)
        .bind(&command.last_error)
        .bind(&command.result_payload)
        .bind(&audit_metadata)
        .bind(command.completed_at)
        .bind(now)
        .fetch_optional(pool)
        .await?;
        let row = match row {
            Some(row) => Some(row),
            None => sqlx::query(
                r#"INSERT INTO whatsapp_provider_write_commands
                (command_id, account_id, command_kind, idempotency_key, provider_chat_id,
                 provider_message_id, target_ref, payload, capability_state, action_class,
                 confirmation_decision, status, retry_count, max_retries, last_error,
                 result_payload, audit_metadata, actor_id, happened_at, completed_at, created_at, updated_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
                ON CONFLICT (command_id) DO NOTHING RETURNING *"#,
            )
            .bind(&command.command_id)
            .bind(&command.account_id)
            .bind(&command.command_kind)
            .bind(&command.idempotency_key)
            .bind(provider_chat_id)
            .bind(&command.provider_message_id)
            .bind(&command.target_ref)
            .bind(&command.payload)
            .bind(&command.capability_state)
            .bind(&command.action_class)
            .bind(&command.confirmation_decision)
            .bind(status)
            .bind(command.retry_count)
            .bind(command.max_retries)
            .bind(&command.last_error)
            .bind(&command.result_payload)
            .bind(&audit_metadata)
            .bind(&command.actor_id)
            .bind(command.happened_at)
            .bind(command.completed_at)
            .bind(command.created_at)
            .bind(now)
            .fetch_optional(pool)
            .await?,
        };
        if let Some(row) = row {
            let imported_command = row_to_whatsapp_provider_write_command(row)?;
            mirror_canonical_provider_command_for_pool(pool, &imported_command).await?;
            imported.push(imported_command);
        }
    }
    Ok(imported)
}
