use super::*;

pub(crate) async fn publish_whatsapp_command_event(
    state: &AppState,
    response: &WhatsAppProviderCommandResponse,
) -> Result<(), ApiError> {
    let event = event_builders::command_event(response);
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn publish_whatsapp_command_record_event(
    state: &AppState,
    command: &WhatsAppProviderCommand,
    source: &str,
) -> Result<(), ApiError> {
    let event = event_builders::command_record_event(command, source);
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(crate) async fn publish_whatsapp_media_event(
    state: &AppState,
    event_type: &str,
    command_id: &str,
    payload: serde_json::Value,
) -> Result<(), ApiError> {
    let now = Utc::now();
    if let Some(account_id) = payload.get("account_id").and_then(Value::as_str) {
        let _ = whatsapp_fixture_ingest_service(state)?
            .capture_media_lifecycle_event(
                account_id,
                command_id,
                event_type,
                payload.clone(),
                &format!("media_{}", event_type.replace('.', "_")),
                now,
            )
            .await?;
    }
    let event = NewEventEnvelope::builder(
        event_builders::event_id("media", command_id, now),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_provider_commands",
            "source_id": command_id,
        }),
        json!({
            "id": command_id,
            "entity_id": command_id,
            "kind": "whatsapp_media_command",
        }),
    )
    .payload(payload)
    .build()
    .expect("WhatsApp media event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(crate) async fn publish_whatsapp_sync_event(
    state: &AppState,
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let scope = payload
        .get("scope")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let source_id = format!("{subject_id}:{scope}");
    let event = NewEventEnvelope::builder(
        event_builders::event_id("sync", subject_id, now),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_sync_requests",
            "source_id": source_id,
        }),
        json!({
            "id": subject_id,
            "entity_id": subject_id,
            "kind": "whatsapp_sync",
        }),
    )
    .payload(payload)
    .build()
    .expect("WhatsApp sync event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn publish_whatsapp_projection_event(
    state: &AppState,
    event_type: &str,
    subject_kind: &str,
    subject_id: &str,
    provider_chat_id: Option<&str>,
    provider_message_id: Option<&str>,
    occurred_at: DateTime<Utc>,
    payload: serde_json::Value,
) -> Result<(), ApiError> {
    let source_id = payload
        .get("raw_record_id")
        .and_then(Value::as_str)
        .unwrap_or(subject_id);
    let source_kind = if payload
        .get("raw_record_id")
        .and_then(Value::as_str)
        .is_some()
    {
        "communication_raw_records"
    } else {
        "whatsapp_projection_events"
    };
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("projection", subject_id, occurred_at),
        event_type.to_owned(),
        occurred_at,
        json!({
            "channel": "whatsapp",
            "actor_id": AUDIT_ACTOR_ID,
            "kind": source_kind,
            "source_id": source_id,
        }),
        json!({
            "id": subject_id,
            "entity_id": subject_id,
            "kind": subject_kind,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
    )
    .payload(sanitize_event_payload(payload))
    .build()
    .expect("WhatsApp projection event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn publish_whatsapp_runtime_status_event(
    state: &AppState,
    status: &WhatsAppRuntimeStatus,
    source: &str,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        status.account_id,
        source,
        status.status,
        status.updated_at.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("runtime", &status.account_id, now),
        whatsapp_event_types::RUNTIME_STATUS_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": status.account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_runtime_status",
            "source_id": source_id,
        }),
        json!({
            "id": status.account_id,
            "entity_id": status.account_id,
            "kind": "whatsapp_runtime",
        }),
    )
    .payload(sanitize_event_payload(json!({
        "account_id": status.account_id,
        "provider_kind": status.provider_kind,
        "provider_shape": status.provider_shape,
        "runtime_kind": status.runtime_kind,
        "status": status.status,
        "fixture_runtime": status.fixture_runtime,
        "live_runtime_available": status.live_runtime_available,
        "live_send_available": status.live_send_available,
        "media_download_available": status.media_download_available,
        "media_upload_available": status.media_upload_available,
        "session_restore_available": status.session_restore_available,
        "runtime_blockers": status.runtime_blockers,
        "last_error": status.last_error,
        "source": source,
    })))
    .build()
    .expect("WhatsApp runtime status event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn publish_whatsapp_session_link_state_event(
    state: &AppState,
    account_id: &str,
    provider_shape: &str,
    runtime_kind: &str,
    link_state: &str,
    source: &str,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        account_id,
        source,
        link_state,
        observed_at.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("session", account_id, now),
        whatsapp_event_types::SESSION_LINK_STATE_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_session_link_state",
            "source_id": source_id,
        }),
        json!({
            "id": account_id,
            "entity_id": account_id,
            "kind": "whatsapp_session",
        }),
    )
    .payload(sanitize_event_payload(json!({
        "account_id": account_id,
        "provider_shape": provider_shape,
        "runtime_kind": runtime_kind,
        "link_state": link_state,
        "source": source,
    })))
    .build()
    .expect("WhatsApp session lifecycle event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn publish_whatsapp_runtime_event(
    state: &AppState,
    account_id: &str,
    provider_event_id: &str,
    runtime_event_kind: &str,
    runtime_status: Option<&str>,
    lifecycle_state: Option<&str>,
    severity: Option<&str>,
    metadata_keys: Vec<String>,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), ApiError> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        account_id,
        provider_event_id,
        runtime_event_kind,
        observed_at.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        whatsapp_event_id("runtime_event", provider_event_id, now),
        whatsapp_event_types::RUNTIME_EVENT.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": AUDIT_ACTOR_ID,
            "kind": "whatsapp_runtime_events",
            "source_id": source_id,
        }),
        json!({
            "id": provider_event_id,
            "entity_id": account_id,
            "kind": "whatsapp_runtime_event",
        }),
    )
    .payload(sanitize_event_payload(json!({
        "account_id": account_id,
        "provider_event_id": provider_event_id,
        "runtime_event_kind": runtime_event_kind,
        "runtime_status": runtime_status,
        "lifecycle_state": lifecycle_state,
        "severity": severity,
        "metadata_keys": metadata_keys,
        "observed_at": observed_at,
    })))
    .build()
    .expect("WhatsApp runtime event envelope must be valid");
    event_store(state)?.append(&event).await?;
    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn capture_whatsapp_runtime_lifecycle_signal(
    state: &AppState,
    status: &WhatsAppRuntimeStatus,
    source: &str,
) -> Result<(), ApiError> {
    let provider_event_id = format!(
        "{}:{}:{}",
        status.account_id,
        source,
        status.updated_at.timestamp_micros()
    );
    let metadata = json!({
        "source": source,
        "provider_kind": status.provider_kind,
        "provider_shape": status.provider_shape,
        "runtime_kind": status.runtime_kind,
        "fixture_runtime": status.fixture_runtime,
        "live_runtime_available": status.live_runtime_available,
        "live_send_available": status.live_send_available,
        "media_download_available": status.media_download_available,
        "media_upload_available": status.media_upload_available,
        "session_restore_available": status.session_restore_available,
        "runtime_blockers": status.runtime_blockers,
        "last_error": status.last_error,
    });
    let _ = whatsapp_fixture_ingest_service(state)?
        .capture_runtime_lifecycle_event(
            &status.account_id,
            &provider_event_id,
            source,
            Some(&status.status),
            Some(&status.status),
            Some(
                if status.status == "available" || status.status == "linked" {
                    "info"
                } else if status.status == "degraded" {
                    "warning"
                } else {
                    "blocked"
                },
            ),
            metadata,
            source,
            status.updated_at,
        )
        .await?;
    Ok(())
}

pub(crate) async fn capture_whatsapp_sync_runtime_signal(
    state: &AppState,
    account_id: &str,
    subject_id: &str,
    scope: &str,
    phase: &str,
    metadata: Value,
) -> Result<(), ApiError> {
    let observed_at = Utc::now();
    let provider_event_id = format!(
        "{}:{}:{}:{}",
        account_id,
        scope,
        phase,
        observed_at.timestamp_micros()
    );
    let runtime_status = match phase {
        "started" | "progress" => Some("syncing"),
        "completed" => Some("synced"),
        "failed" => Some("failed"),
        _ => None,
    };
    let severity = match phase {
        "failed" => Some("warning"),
        _ => Some("info"),
    };
    let _ = whatsapp_fixture_ingest_service(state)?
        .capture_runtime_lifecycle_event(
            account_id,
            &provider_event_id,
            &format!("sync.{scope}.{phase}"),
            runtime_status,
            runtime_status,
            severity,
            lifecycle_projection::merge_runtime_event_metadata(
                metadata,
                json!({
                    "subject_id": subject_id,
                    "phase": phase,
                }),
            ),
            &format!("sync_{scope}_{phase}"),
            observed_at,
        )
        .await?;
    Ok(())
}

pub(crate) async fn capture_whatsapp_status_publish_runtime_signal(
    state: &AppState,
    account_id: &str,
    command_id: &str,
    phase: &str,
    metadata: Value,
) -> Result<(), ApiError> {
    let observed_at = Utc::now();
    let provider_event_id = format!(
        "{}:status.publish:{}:{}",
        command_id,
        phase,
        observed_at.timestamp_micros()
    );
    let runtime_status = match phase {
        "failed" => Some("degraded"),
        _ => None,
    };
    let severity = match phase {
        "failed" => Some("warning"),
        _ => Some("info"),
    };
    let _ = whatsapp_fixture_ingest_service(state)?
        .capture_runtime_lifecycle_event(
            account_id,
            &provider_event_id,
            &format!("status.publish.{phase}"),
            runtime_status,
            Some(phase),
            severity,
            metadata,
            &format!("status_publish_{phase}"),
            observed_at,
        )
        .await?;
    Ok(())
}

pub(crate) async fn current_whatsapp_runtime_kind(
    state: &AppState,
    account_id: &str,
) -> Result<String, ApiError> {
    let status = whatsapp_provider_runtime_service(state)?
        .runtime_status(
            &whatsapp_secret_reference_store(state)?,
            &state.vault,
            account_id,
        )
        .await?;
    Ok(status.runtime_kind)
}

pub(crate) async fn ensure_whatsapp_sync_supported(
    state: &AppState,
    account_id: &str,
    operation: &'static str,
) -> Result<(), ApiError> {
    let _status = whatsapp_provider_runtime_service(state)?
        .runtime_status(
            &whatsapp_secret_reference_store(state)?,
            &state.vault,
            account_id,
        )
        .await?;
    let _ = operation;
    Ok(())
}

pub(super) async fn list_whatsapp_sync_members(
    state: &AppState,
    account_id: &str,
    provider_chat_id: &str,
    limit: i64,
) -> Result<Vec<WhatsAppMembersSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .list_members_for_provider_conversation(account_id, provider_chat_id, limit)
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?;

    rows.into_iter()
        .map(|row| {
            let participant_metadata = row.participant_metadata;
            let identity_metadata = row.identity_metadata;
            let provider_identity_id = row.provider_identity_id;
            let provider_member_id = provider_identity_id
                .clone()
                .unwrap_or_else(|| row.participant_id.clone());
            Ok(WhatsAppMembersSyncItem {
                participant_id: row.participant_id,
                conversation_id: row.conversation_id.unwrap_or_default(),
                account_id: row.account_id.unwrap_or_default(),
                provider_chat_id: row.provider_conversation_id.unwrap_or_default(),
                provider_member_id,
                provider_identity_id,
                sender_display_name: Some(row.display_name),
                role: row.role,
                status: Some("active".to_owned()),
                identity_kind: row.identity_kind,
                address: row.address,
                is_admin: participant_metadata
                    .get("is_admin")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_owner: participant_metadata
                    .get("is_owner")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                participant_metadata,
                identity_metadata: identity_metadata.unwrap_or_else(|| json!({})),
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

pub(super) async fn list_whatsapp_sync_presence(
    state: &AppState,
    account_id: &str,
    provider_chat_id: Option<&str>,
    limit: i64,
) -> Result<Vec<WhatsAppPresenceSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .list_presence(account_id, provider_chat_id, limit)
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?;

    rows.into_iter()
        .map(|row| {
            let identity_metadata = row.metadata;
            Ok(WhatsAppPresenceSyncItem {
                identity_id: row.identity_id,
                account_id: row.account_id,
                channel_kind: row.channel_kind,
                provider_chat_id: identity_metadata
                    .get("presence_provider_chat_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                provider_identity_id: row.provider_identity_id.unwrap_or_default(),
                identity_kind: row.identity_kind,
                display_name: row.display_name,
                address: row.address,
                presence_state: identity_metadata
                    .get("presence_state")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned(),
                last_seen_at: identity_metadata
                    .get("last_seen_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                observed_at: identity_metadata
                    .get("presence_observed_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                identity_metadata,
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

pub(super) async fn list_whatsapp_sync_calls(
    state: &AppState,
    account_id: &str,
    provider_chat_id: Option<&str>,
    limit: i64,
) -> Result<Vec<WhatsAppCallsSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = makosh_communications_postgres::calls::CanonicalCallReadStore::new(pool)
        .list_whatsapp_calls(account_id, provider_chat_id, limit)
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?;

    rows.into_iter()
        .map(|row| {
            let metadata = row.metadata;
            Ok(WhatsAppCallsSyncItem {
                call_id: row.call_id,
                account_id: row.account_id,
                provider_call_id: row.provider_call_id,
                provider_chat_id: row.provider_chat_id,
                direction: row.direction,
                call_state: row.call_state,
                started_at: row.started_at.map(|value| value.to_rfc3339()),
                ended_at: row.ended_at.map(|value| value.to_rfc3339()),
                observed_at: metadata
                    .get("observed_at")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                metadata,
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

pub(super) async fn list_whatsapp_sync_contacts_via_ports(
    state: &AppState,
    account_id: &str,
    limit: i64,
) -> Result<Vec<WhatsAppContactsSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let identities =
        makosh_communications_postgres::conversations::ConversationReadStore::new(pool.clone())
            .list_whatsapp_identities(account_id, limit)
            .await
            .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?;
    let whatsapp_values = identities
        .iter()
        .filter_map(|item| item.provider_identity_id.clone())
        .collect::<Vec<_>>();
    let phone_values = identities
        .iter()
        .filter_map(|item| item.address.clone())
        .collect::<Vec<_>>();
    let persona_query = makosh_personas_postgres::PersonaPostgresReadQuery::new(pool);
    let whatsapp_traces = persona_query
        .list_for_values("whatsapp", &whatsapp_values)
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?
        .into_iter()
        .map(|item| (item.identity_value, item.metadata))
        .collect::<std::collections::HashMap<_, _>>();
    let phone_traces = persona_query
        .list_for_values("phone", &phone_values)
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?
        .into_iter()
        .map(|item| (item.identity_value, item.metadata))
        .collect::<std::collections::HashMap<_, _>>();
    identities
        .into_iter()
        .map(|row| {
            let identity_metadata = row.metadata;
            let display_name_history = identity_metadata
                .get("display_name_history")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
            Ok(WhatsAppContactsSyncItem {
                identity_id: row.identity_id,
                account_id: row.account_id,
                channel_kind: row.channel_kind,
                provider_identity_id: row.provider_identity_id.clone().unwrap_or_default(),
                identity_kind: row.identity_kind,
                display_name: row.display_name,
                address: row.address.clone(),
                push_name: identity_metadata
                    .get("push_name")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                business_profile: identity_metadata
                    .get("business_profile")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                profile_photo_ref: identity_metadata
                    .get("profile_photo_ref")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                display_name_history,
                identity_metadata,
                whatsapp_trace_metadata: row
                    .provider_identity_id
                    .as_ref()
                    .and_then(|value| whatsapp_traces.get(value))
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                phone_trace_metadata: row
                    .address
                    .as_ref()
                    .and_then(|value| phone_traces.get(value))
                    .cloned()
                    .unwrap_or_else(|| json!({})),
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}

pub(super) async fn list_whatsapp_sync_media(
    state: &AppState,
    account_id: &str,
    provider_chat_id: Option<&str>,
    content_type: Option<&str>,
    limit: i64,
) -> Result<Vec<WhatsAppMediaSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows =
        makosh_communications_postgres::attachments::CanonicalMessageAttachmentReadStore::new(pool)
            .list_whatsapp_media(account_id, provider_chat_id, content_type, limit)
            .await
            .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(WhatsAppMediaSyncItem {
                attachment_id: row.attachment_id,
                message_id: row.message_id,
                raw_record_id: row.raw_record_id,
                account_id: row.account_id,
                channel_kind: row.channel_kind,
                provider_chat_id: Some(row.provider_chat_id),
                provider_message_id: row.provider_message_id,
                provider_attachment_id: row.provider_attachment_id,
                filename: row.filename,
                content_type: row.content_type,
                size_bytes: row.size_bytes,
                sha256: row.sha256,
                scan_status: row.scan_status,
                storage_kind: row.storage_kind,
                storage_path: row.storage_path,
                message_subject: row.message_subject.unwrap_or_default(),
                sender: row.sender.unwrap_or_default(),
                sender_display_name: row.sender_display_name,
                occurred_at: row.occurred_at.map(|value| value.to_rfc3339()),
                created_at: row.created_at.to_rfc3339(),
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}
use super::event_types::event_id as whatsapp_event_id;
