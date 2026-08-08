use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Postgres, Row, Transaction};

use super::commands::mark_command_reconciled;
use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::models::chats::{NewTelegramChatParticipant, TelegramChatMember};
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;
use super::store::TelegramStore;
use super::validation::validate_chat_list_limit;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

fn row_to_provider_member(row: sqlx::postgres::PgRow) -> Result<TelegramChatMember, TelegramError> {
    Ok(TelegramChatMember {
        sender_id: row.try_get("provider_member_id")?,
        sender_display_name: row.try_get("display_name")?,
        message_count: 0,
        last_message_at: None,
        source: row.try_get("source")?,
        provider_member_id: row.try_get("provider_member_id")?,
        username: row.try_get("username")?,
        role: row.try_get("role")?,
        status: row.try_get("status")?,
        is_admin: row.try_get("is_admin")?,
        is_owner: row.try_get("is_owner")?,
        permissions: row.try_get("permissions")?,
        observed_at: row.try_get("observed_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn capture_chat_participant_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    telegram_chat_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    member: &TelegramChatMember,
    raw_payload: &serde_json::Value,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), TelegramError> {
    let entity_id = format!("{telegram_chat_id}:{}", member.provider_member_id);
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_CHAT_PARTICIPANT",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "telegram_chat_id": telegram_chat_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_member_id": member.provider_member_id,
                "display_name": member.sender_display_name,
                "username": member.username,
                "role": member.role,
                "status": member.status,
                "is_admin": member.is_admin,
                "is_owner": member.is_owner,
                "permissions": member.permissions,
                "source": member.source,
                "observed_at": member.observed_at,
                "raw_payload": raw_payload,
                "operation": relationship_kind,
            }),
            format!("telegram-chat-participant://{entity_id}/{relationship_kind}"),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "chat_participant",
        entity_id,
        relationship_kind,
        json!({
            "telegram_chat_id": telegram_chat_id,
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_member_id": member.provider_member_id,
            "role": member.role,
            "status": member.status,
            "source": member.source,
        }),
    )
    .await?;
    Ok(())
}

pub async fn upsert_chat_participant(
    pool: &PgPool,
    participant: &NewTelegramChatParticipant,
) -> Result<TelegramChatMember, TelegramError> {
    let now = Utc::now();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source, observed_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $15, $15)
        ON CONFLICT (telegram_chat_id, provider_member_id)
        DO UPDATE SET
            account_id         = EXCLUDED.account_id,
            provider_chat_id   = EXCLUDED.provider_chat_id,
            display_name       = EXCLUDED.display_name,
            username           = EXCLUDED.username,
            role               = EXCLUDED.role,
            status             = EXCLUDED.status,
            is_admin           = EXCLUDED.is_admin,
            is_owner           = EXCLUDED.is_owner,
            permissions        = EXCLUDED.permissions,
            raw_payload        = EXCLUDED.raw_payload,
            source             = EXCLUDED.source,
            observed_at        = EXCLUDED.observed_at,
            updated_at         = EXCLUDED.updated_at
        RETURNING provider_member_id, display_name, username, role, status, is_admin, is_owner,
                  permissions, source, observed_at
        "#,
    )
    .bind(&participant.participant_id)
    .bind(&participant.telegram_chat_id)
    .bind(&participant.account_id)
    .bind(&participant.provider_chat_id)
    .bind(&participant.provider_member_id)
    .bind(&participant.display_name)
    .bind(&participant.username)
    .bind(&participant.role)
    .bind(&participant.status)
    .bind(participant.is_admin)
    .bind(participant.is_owner)
    .bind(&participant.permissions)
    .bind(&participant.raw_payload)
    .bind(&participant.source)
    .bind(now)
    .fetch_one(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let member = row_to_provider_member(row)?;
    capture_chat_participant_observation_in_transaction(
        &mut transaction,
        &participant.telegram_chat_id,
        &participant.account_id,
        &participant.provider_chat_id,
        &member,
        &participant.raw_payload,
        "upsert",
        "telegram.client.participants.upsert_chat_participant",
        member.observed_at.unwrap_or(now),
    )
    .await?;
    transaction.commit().await?;
    Ok(member)
}

pub async fn mark_absent_members_from_exhaustive_roster(
    pool: &PgPool,
    telegram_chat_id: &str,
    observed_member_ids: &[String],
    observed_via: &str,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    let observed_at = Utc::now();
    let permissions_patch = json!({
        "membership_state": "absent_exhaustive",
        "observed_via": observed_via,
    });
    let raw_payload_patch = json!({
        "membership_state": "absent_exhaustive",
        "observed_via": observed_via,
    });
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        UPDATE telegram_chat_participants
        SET status = 'absent_exhaustive',
            permissions = COALESCE(permissions, '{}'::jsonb) || $3::jsonb,
            raw_payload = COALESCE(raw_payload, '{}'::jsonb) || $4::jsonb,
            observed_at = $2,
            updated_at = $2
        WHERE telegram_chat_id = $1
          AND source = 'tdlib'
          AND provider_member_id <> ALL($5)
          AND status IS DISTINCT FROM 'absent_exhaustive'
        RETURNING account_id, provider_chat_id, provider_member_id, display_name, username, role,
                  status, is_admin, is_owner, permissions, raw_payload, source, observed_at
        "#,
    )
    .bind(telegram_chat_id)
    .bind(observed_at)
    .bind(&permissions_patch)
    .bind(&raw_payload_patch)
    .bind(observed_member_ids)
    .fetch_all(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let mut members = Vec::with_capacity(rows.len());
    for row in rows {
        let account_id: String = row.try_get("account_id")?;
        let provider_chat_id: String = row.try_get("provider_chat_id")?;
        let raw_payload: serde_json::Value = row.try_get("raw_payload")?;
        let member = row_to_provider_member(row)?;
        capture_chat_participant_observation_in_transaction(
            &mut transaction,
            telegram_chat_id,
            &account_id,
            &provider_chat_id,
            &member,
            &raw_payload,
            "absent_exhaustive",
            "telegram.client.participants.mark_absent_members_from_exhaustive_roster",
            member.observed_at.unwrap_or(observed_at),
        )
        .await?;
        members.push(member);
    }
    transaction.commit().await?;
    Ok(members)
}

pub fn telegram_self_provider_member_id(external_account_id: &str) -> Option<String> {
    let value = external_account_id.trim();
    if value.is_empty() {
        return None;
    }

    if let Some(user_id) = value.strip_prefix("user:").filter(|id| is_numeric_id(id)) {
        return Some(format!("user:{user_id}"));
    }

    let user_id = value.strip_prefix("telegram:").unwrap_or(value).trim();
    is_numeric_id(user_id).then(|| format!("user:{user_id}"))
}

pub async fn reconcile_join_commands_from_provider_roster(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    observed_at: chrono::DateTime<Utc>,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_join_commands_from_provider_roster_with_source(
        pool,
        account_id,
        provider_chat_id,
        provider_member_id,
        observed_at,
        "tdlib.getSupergroupMembers",
    )
    .await
}

pub async fn reconcile_join_commands_from_provider_roster_with_source(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    observed_at: chrono::DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let provider_state = json!({
        "provider_chat_id": provider_chat_id,
        "provider_member_id": provider_member_id,
        "observed_via": observed_via,
        "membership_state": "present",
    });
    let result_payload = json!({
        "source": observed_via,
        "provider_chat_id": provider_chat_id,
        "provider_member_id": provider_member_id,
        "membership_state": "present",
        "provider_observed_at": observed_at,
    });
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'join'
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state.clone(),
                result_payload.clone(),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

#[allow(clippy::too_many_arguments)]
pub async fn reconcile_leave_commands_from_provider_roster(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    membership_state: &str,
    status: Option<&str>,
    role: Option<&str>,
    observed_at: chrono::DateTime<Utc>,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_leave_commands_from_provider_roster_with_source(
        pool,
        account_id,
        provider_chat_id,
        provider_member_id,
        membership_state,
        status,
        role,
        observed_at,
        "tdlib.getSupergroupMembers",
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn reconcile_leave_commands_from_provider_roster_with_source(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    membership_state: &str,
    status: Option<&str>,
    role: Option<&str>,
    observed_at: chrono::DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let provider_state = json!({
        "provider_chat_id": provider_chat_id,
        "provider_member_id": provider_member_id,
        "observed_via": observed_via,
        "membership_state": membership_state,
        "status": status,
        "role": role,
    });
    let result_payload = json!({
        "source": observed_via,
        "provider_chat_id": provider_chat_id,
        "provider_member_id": provider_member_id,
        "membership_state": membership_state,
        "status": status,
        "role": role,
        "provider_observed_at": observed_at,
    });
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'leave'
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state.clone(),
                result_payload.clone(),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

pub async fn reconcile_leave_commands_from_exhaustive_absence(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    observed_at: chrono::DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_leave_commands_from_provider_roster_with_source(
        pool,
        account_id,
        provider_chat_id,
        provider_member_id,
        "absent_exhaustive",
        None,
        None,
        observed_at,
        observed_via,
    )
    .await
}

pub fn inactive_roster_membership_state(item: &TelegramChatMember) -> Option<&'static str> {
    if matches!(item.status.as_deref(), Some("banned"))
        || matches!(item.role.as_deref(), Some("banned"))
    {
        return Some("banned");
    }
    if matches!(item.status.as_deref(), Some("left"))
        || matches!(item.role.as_deref(), Some("left"))
    {
        return Some("left");
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramObservedParticipantLifecycle {
    pub command_kind: String,
    pub provider_member_id: String,
    pub observed_via: String,
    pub membership_state: String,
}

pub fn tdlib_self_membership_lifecycle(
    external_account_id: &str,
    raw_message: &serde_json::Value,
) -> Option<TelegramObservedParticipantLifecycle> {
    let provider_member_id = telegram_self_provider_member_id(external_account_id)?;
    let self_user_id = provider_member_id.strip_prefix("user:")?;
    let content = raw_message.get("content")?;
    let content_type = content.get("@type").and_then(serde_json::Value::as_str)?;

    match content_type {
        "messageChatDeleteMember" => {
            let user_id = content
                .get("user_id")
                .and_then(serde_json::Value::as_i64)?
                .to_string();
            (user_id == self_user_id).then(|| TelegramObservedParticipantLifecycle {
                command_kind: "leave".to_owned(),
                provider_member_id,
                observed_via: "tdlib.messageChatDeleteMember".to_owned(),
                membership_state: "left".to_owned(),
            })
        }
        "messageChatAddMembers" => {
            let user_ids = content.get("member_user_ids")?.as_array()?;
            user_ids
                .iter()
                .filter_map(serde_json::Value::as_i64)
                .map(|value| value.to_string())
                .any(|value| value == self_user_id)
                .then(|| TelegramObservedParticipantLifecycle {
                    command_kind: "join".to_owned(),
                    provider_member_id,
                    observed_via: "tdlib.messageChatAddMembers".to_owned(),
                    membership_state: "present".to_owned(),
                })
        }
        _ => None,
    }
}

pub async fn reconcile_participant_commands_from_message_evidence(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    observed_at: chrono::DateTime<Utc>,
    lifecycle: &TelegramObservedParticipantLifecycle,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let provider_state = json!({
        "provider_chat_id": provider_chat_id,
        "provider_member_id": lifecycle.provider_member_id,
        "provider_message_id": provider_message_id,
        "observed_via": lifecycle.observed_via,
        "membership_state": lifecycle.membership_state,
    });
    let result_payload = json!({
        "source": lifecycle.observed_via,
        "provider_chat_id": provider_chat_id,
        "provider_member_id": lifecycle.provider_member_id,
        "provider_message_id": provider_message_id,
        "membership_state": lifecycle.membership_state,
        "provider_observed_at": observed_at,
    });
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = $3
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND happened_at <= $4
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(&lifecycle.command_kind)
    .bind(observed_at + chrono::Duration::seconds(5))
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state.clone(),
                result_payload.clone(),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

pub async fn provider_roster_exists(
    pool: &PgPool,
    telegram_chat_id: &str,
) -> Result<bool, TelegramError> {
    let exists: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM telegram_chat_participants WHERE telegram_chat_id = $1 LIMIT 1",
    )
    .bind(telegram_chat_id)
    .fetch_optional(pool)
    .await
    .map_err(TelegramError::from)?;
    Ok(exists.is_some())
}

pub async fn list_provider_chat_members(
    pool: &PgPool,
    telegram_chat_id: &str,
    query: Option<&str>,
    role: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    let limit = validate_chat_list_limit(limit)?;
    let query = normalized_query(query);
    let role = normalized_query(role);
    let pattern = query.as_ref().map(|value| format!("%{value}%"));
    let rows = sqlx::query(
        r#"
        SELECT provider_member_id, display_name, username, role, status, is_admin, is_owner,
               permissions, source, observed_at
        FROM telegram_chat_participants
        WHERE telegram_chat_id = $1
          AND coalesce(status, '') NOT IN ('left', 'banned', 'absent_exhaustive')
          AND coalesce(role, '') NOT IN ('left', 'banned')
          AND ($2::TEXT IS NULL OR role = $2)
          AND (
              $3::TEXT IS NULL
              OR lower(coalesce(display_name, '')) LIKE $3
              OR lower(coalesce(username, '')) LIKE $3
              OR lower(provider_member_id) LIKE $3
          )
        ORDER BY is_owner DESC, is_admin DESC, role ASC, lower(coalesce(display_name, provider_member_id)) ASC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(telegram_chat_id)
    .bind(role.as_deref())
    .bind(pattern.as_deref())
    .bind(limit)
    .bind(offset.max(0))
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter().map(row_to_provider_member).collect()
}

pub async fn list_message_heuristic_members(
    store: &TelegramStore,
    account_id: &str,
    provider_chat_id: &str,
    query: Option<&str>,
    role: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    if normalized_query(role).is_some() {
        return Ok(Vec::new());
    }

    let limit = validate_chat_list_limit(limit)?;
    let query = normalized_query(query);
    let members = store
        .provider_channel_message_store()
        .heuristic_members(
            account_id,
            provider_chat_id,
            query.as_deref(),
            TELEGRAM_CHANNEL_KINDS,
            limit,
            offset,
        )
        .await?;

    Ok(members
        .into_iter()
        .map(|member| TelegramChatMember {
            sender_id: member.sender_id.clone(),
            sender_display_name: member.sender_display_name,
            message_count: member.message_count,
            last_message_at: member.last_message_at,
            source: "message_heuristic".to_owned(),
            provider_member_id: member.sender_id,
            username: None,
            role: None,
            status: None,
            is_admin: false,
            is_owner: false,
            permissions: json!({}),
            observed_at: member.last_message_at,
        })
        .collect())
}

fn normalized_query(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
}

fn is_numeric_id(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests;
