use super::*;
use crate::domains::personas::core::identities::PersonaIdentityStore;

impl WhatsappFixtureIngestApplicationService {
    pub(super) async fn whatsapp_conversation_last_message_at(
        &self,
        conversation_id: &str,
    ) -> Result<Option<Option<chrono::DateTime<chrono::Utc>>>, CommunicationFixtureIngestError>
    {
        let last_message_at = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT last_message_at FROM communication_conversations WHERE conversation_id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(last_message_at)
    }

    pub(super) async fn restore_whatsapp_conversation_last_message_at(
        &self,
        account_id: &str,
        conversation_id: &str,
        previous_last_message_at: Option<Option<chrono::DateTime<chrono::Utc>>>,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let Some(previous_last_message_at) = previous_last_message_at else {
            return Ok(());
        };
        sqlx::query(
            r#"
            UPDATE communication_conversations conversation
            SET last_message_at = $3,
                updated_at = now()
            WHERE conversation.conversation_id = $2
              AND conversation.account_id = $1
            "#,
        )
        .bind(account_id)
        .bind(conversation_id)
        .bind(previous_last_message_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub(super) async fn project_whatsapp_message_refs(
        &self,
        request: &NewWhatsappWebMessage,
        message_id: &str,
        raw_record_id: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let store = ProviderChannelMessagePort::new(self.pool.clone());

        if let Some(reply_to_provider_message_id) = request
            .reply_to_provider_message_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let target_message = store
                .message_by_provider_record_id(
                    &request.account_id,
                    reply_to_provider_message_id,
                    WHATSAPP_CHANNEL_KINDS,
                )
                .await?
                .ok_or_else(|| {
                    CommunicationFixtureIngestError::SignalControlBlocked(format!(
                        "whatsapp reply target `{reply_to_provider_message_id}` is not projected"
                    ))
                })?;
            let reply_ref_id = whatsapp_message_ref_id(
                &request.account_id,
                "reply",
                &request.provider_message_id,
                Some(reply_to_provider_message_id),
            );
            sqlx::query(
                r#"
                INSERT INTO communication_message_refs (
                    message_ref_id, ref_kind, source_message_id, target_message_id, account_id,
                    provider_conversation_id, source_provider_id, target_provider_id, depth,
                    metadata, provenance
                )
                VALUES ($1, 'reply', $2, $3, $4, $5, $6, $7, 1, $8, $9)
                ON CONFLICT (message_ref_id) DO NOTHING
                "#,
            )
            .bind(&reply_ref_id)
            .bind(message_id)
            .bind(&target_message.message_id)
            .bind(&request.account_id)
            .bind(&request.provider_chat_id)
            .bind(&request.provider_message_id)
            .bind(reply_to_provider_message_id)
            .bind(json!({
                "provider": account_context.provider_kind,
                "provider_chat_id": request.provider_chat_id,
                "raw_record_id": raw_record_id,
                "reply_to_provider_message_id": reply_to_provider_message_id,
            }))
            .bind(json!({
                "raw_record_id": raw_record_id,
                "relationship_kind": "reply",
            }))
            .execute(&self.pool)
            .await?;
        }

        let forward_origin_message_id = request
            .forward_origin_message_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let forward_origin_chat_id = request
            .forward_origin_chat_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if forward_origin_message_id.is_some() || forward_origin_chat_id.is_some() {
            let target_message = if let Some(origin_message_id) = forward_origin_message_id {
                store
                    .message_by_provider_record_id(
                        &request.account_id,
                        origin_message_id,
                        WHATSAPP_CHANNEL_KINDS,
                    )
                    .await?
            } else {
                None
            };
            let forward_ref_id = whatsapp_message_ref_id(
                &request.account_id,
                "forward",
                &request.provider_message_id,
                forward_origin_message_id,
            );
            sqlx::query(
                r#"
                INSERT INTO communication_message_refs (
                    message_ref_id, ref_kind, source_message_id, target_message_id, account_id,
                    provider_conversation_id, source_provider_id, target_provider_id, depth,
                    metadata, provenance
                )
                VALUES ($1, 'forward', $2, $3, $4, $5, $6, $7, 1, $8, $9)
                ON CONFLICT (message_ref_id) DO NOTHING
                "#,
            )
            .bind(&forward_ref_id)
            .bind(message_id)
            .bind(
                target_message
                    .as_ref()
                    .map(|message| message.message_id.as_str()),
            )
            .bind(&request.account_id)
            .bind(&request.provider_chat_id)
            .bind(&request.provider_message_id)
            .bind(forward_origin_message_id)
            .bind(json!({
                "provider": account_context.provider_kind,
                "provider_chat_id": request.provider_chat_id,
                "raw_record_id": raw_record_id,
                "forward_origin_chat_id": forward_origin_chat_id,
                "forward_origin_message_id": forward_origin_message_id,
                "forward_origin_sender_id": request.forward_origin_sender_id,
                "forward_origin_sender_name": request.forward_origin_sender_name,
                "forwarded_at": request.forwarded_at,
            }))
            .bind(json!({
                "raw_record_id": raw_record_id,
                "relationship_kind": "forward",
            }))
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub(super) async fn publish_whatsapp_command_reconciled_events(
        &self,
        commands: Vec<crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand>,
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            let payload = json!({
                "account_id": command.account_id,
                "command_id": command.command_id,
                "idempotency_key": command.idempotency_key,
                "command_kind": command.command_kind,
                "action": command.command_kind,
                "provider_chat_id": command.provider_chat_id,
                "provider_message_id": command.provider_message_id,
                "capability_state": command.capability_state,
                "action_class": command.action_class,
                "confirmation_decision": command.confirmation_decision,
                "status": command.status,
                "retry_count": command.retry_count,
                "max_retries": command.max_retries,
                "last_error": command.last_error,
                "result_payload": command.result_payload,
                "audit_metadata": command.audit_metadata,
                "provider_state": command.provider_state,
                "reconciliation_status": command.reconciliation_status,
                "next_attempt_at": command.next_attempt_at,
                "last_attempt_at": command.last_attempt_at,
                "provider_observed_at": command.provider_observed_at,
                "reconciled_at": command.reconciled_at,
                "dead_lettered_at": command.dead_lettered_at,
                "completed_at": command.completed_at,
                "source": source,
            });
            self.publish_whatsapp_command_event(
                whatsapp_event_types::COMMAND_STATUS_CHANGED,
                &command.command_id,
                &command.account_id,
                payload.clone(),
            )
            .await?;
            self.publish_whatsapp_command_event(
                whatsapp_event_types::COMMAND_RECONCILED,
                &command.command_id,
                &command.account_id,
                payload,
            )
            .await?;
        }
        Ok(())
    }

    pub(super) async fn publish_whatsapp_status_runtime_events(
        &self,
        commands: &[crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand],
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            if command.command_kind != "publish_status" {
                continue;
            }
            let observed_at = command
                .provider_observed_at
                .or(command.completed_at)
                .unwrap_or_else(Utc::now);
            self.capture_runtime_lifecycle_event(
                &command.account_id,
                &format!(
                    "{}:status.publish.{}:{}",
                    command.command_id,
                    command.status,
                    observed_at.timestamp_micros()
                ),
                &format!("status.publish.{}", command.status),
                None,
                Some(command.status.as_str()),
                Some(if command.status == "failed" {
                    "warning"
                } else {
                    "info"
                }),
                json!({
                    "command_id": command.command_id,
                    "command_kind": command.command_kind,
                    "provider_chat_id": command.provider_chat_id,
                    "provider_status_id": command
                        .result_payload
                        .get("provider_status_id")
                        .cloned()
                        .or_else(|| command.provider_state.get("provider_status_id").cloned()),
                    "status": command.status,
                    "source": source,
                }),
                "status_publish_observed",
                observed_at,
            )
            .await?;
        }
        Ok(())
    }

    pub(super) async fn publish_whatsapp_conversation_runtime_events(
        &self,
        commands: &[crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand],
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            let Some(runtime_event_kind) = (match command.command_kind.as_str() {
                "archive" => Some("conversation.archive.completed"),
                "unarchive" => Some("conversation.unarchive.completed"),
                "pin" => Some("conversation.pin.completed"),
                "unpin" => Some("conversation.unpin.completed"),
                "mute" => Some("conversation.mute.completed"),
                "unmute" => Some("conversation.unmute.completed"),
                "mark_read" => Some("conversation.mark_read.completed"),
                "mark_unread" => Some("conversation.mark_unread.completed"),
                _ => None,
            }) else {
                continue;
            };
            let observed_at = command
                .provider_observed_at
                .or(command.completed_at)
                .unwrap_or_else(Utc::now);
            self.capture_runtime_lifecycle_event(
                &command.account_id,
                &format!(
                    "{}:{}:{}",
                    command.command_id,
                    runtime_event_kind,
                    observed_at.timestamp_micros()
                ),
                runtime_event_kind,
                None,
                Some("completed"),
                Some("info"),
                json!({
                    "command_id": command.command_id,
                    "command_kind": command.command_kind,
                    "provider_chat_id": command.provider_chat_id,
                    "status": command.status,
                    "source": source,
                }),
                "conversation_command_observed",
                observed_at,
            )
            .await?;
        }
        Ok(())
    }

    pub(super) async fn publish_whatsapp_group_runtime_events(
        &self,
        commands: &[crate::integrations::whatsapp::runtime::contracts::WhatsAppProviderCommand],
        source: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        for command in commands {
            let Some(runtime_event_kind) = (match command.command_kind.as_str() {
                "join_group" => Some("group.join.completed"),
                "leave_group" => Some("group.leave.completed"),
                _ => None,
            }) else {
                continue;
            };
            let observed_at = command
                .provider_observed_at
                .or(command.completed_at)
                .unwrap_or_else(Utc::now);
            self.capture_runtime_lifecycle_event(
                &command.account_id,
                &format!(
                    "{}:{}:{}",
                    command.command_id,
                    runtime_event_kind,
                    observed_at.timestamp_micros()
                ),
                runtime_event_kind,
                None,
                Some("completed"),
                Some("info"),
                json!({
                    "command_id": command.command_id,
                    "command_kind": command.command_kind,
                    "provider_chat_id": command.provider_chat_id,
                    "status": command.status,
                    "source": source,
                }),
                "group_command_observed",
                observed_at,
            )
            .await?;
        }
        Ok(())
    }

    pub(super) async fn publish_whatsapp_command_event(
        &self,
        event_type: &str,
        command_id: &str,
        account_id: &str,
        payload: Value,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let now = Utc::now();
        let source = payload
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("fixture_reconcile");
        let command_kind = payload
            .get("command_kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let status = payload
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let source_id = format!(
            "{}:{}:{}:{}:{}",
            command_id,
            command_kind,
            status,
            source,
            now.timestamp_micros()
        );
        let event = NewEventEnvelope::builder(
            whatsapp_fixture_event_id("command", command_id, now),
            event_type.to_owned(),
            now,
            json!({
                "channel": "whatsapp",
                "account_id": account_id,
                "actor_id": AUDIT_ACTOR_ID,
                "kind": "whatsapp_provider_commands",
                "source_id": source_id,
            }),
            json!({
                "id": command_id,
                "entity_id": command_id,
                "kind": "whatsapp_provider_command",
            }),
        )
        .payload(payload)
        .build()
        .expect("WhatsApp command reconciliation event envelope must be valid");
        self.event_store.append(&event).await?;
        let _ = self.event_bus.broadcast(event);
        Ok(())
    }

    pub(super) async fn record_and_accept_whatsapp_raw(
        &self,
        raw: &makosh_communications_api::evidence::NewRawCommunicationRecord,
    ) -> Result<AcceptedWhatsappRawRecord, CommunicationFixtureIngestError> {
        let stored_raw = CommunicationRawEvidencePort::new(self.pool.clone())
            .record_raw_source(raw)
            .await?;
        self.ensure_canonical_communication_account(&stored_raw.account_id)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        Ok(AcceptedWhatsappRawRecord {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: accepted_event.event_id,
            observation_id: stored_raw.observation_id,
        })
    }

    pub(super) async fn ensure_canonical_communication_account(
        &self,
        account_id: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        sqlx::query(
            r#"
            INSERT INTO communication_accounts (
                account_id, provider_kind, display_name, external_account_id,
                config, metadata, created_at, updated_at
            )
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                jsonb_build_object('source_table', 'communication_provider_accounts'),
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub(super) async fn ensure_whatsapp_channel(
        &self,
        account_id: &str,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let channel_id = whatsapp_channel_id(account_id);
        crate::domains::communications::fixtures::whatsapp_projection::ensure_whatsapp_channel(
            &self.pool,
            account_id,
            &channel_id,
            &account_context.channel_kind,
        )
        .await?;
        Ok(channel_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn upsert_whatsapp_conversation(
        &self,
        account_id: &str,
        channel_id: &str,
        provider_chat_id: &str,
        chat_title: &str,
        chat_kind: &str,
        is_archived: Option<bool>,
        is_pinned: Option<bool>,
        is_muted: Option<bool>,
        is_unread: Option<bool>,
        unread_count: Option<i64>,
        participant_count: Option<i64>,
        community_parent_chat_id: Option<&str>,
        community_parent_title: Option<&str>,
        invite_link: Option<&str>,
        is_community_root: Option<bool>,
        is_broadcast: Option<bool>,
        is_newsletter: Option<bool>,
        avatar_metadata: &Value,
        provider_labels: &[String],
        observed_at: chrono::DateTime<chrono::Utc>,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let conversation_id = whatsapp_conversation_id(account_id, provider_chat_id);
        crate::domains::communications::fixtures::whatsapp_projection::upsert_whatsapp_conversation(
            &self.pool, account_id, &conversation_id, channel_id, provider_chat_id, chat_title, chat_kind,
            &account_context.provider_kind, &account_context.channel_kind, is_archived, is_pinned,
            is_muted, is_unread, unread_count, participant_count, community_parent_chat_id,
            community_parent_title, invite_link, is_community_root, is_broadcast, is_newsletter,
            avatar_metadata, provider_labels, observed_at, &stored_raw.raw_record_id,
            &stored_raw.accepted_event_id,
        ).await?;
        Ok(conversation_id)
    }

    pub(super) async fn upsert_whatsapp_identity(
        &self,
        account_id: &str,
        channel_id: &str,
        request: &NewWhatsappWebParticipant,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<String, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let identity_id = whatsapp_identity_id(
            account_id,
            &request.identity_kind,
            &request.provider_identity_id,
        );
        let existing_row: Option<(Option<String>, Value)> = sqlx::query_as(
            r#"
            SELECT display_name, metadata
            FROM communication_identities
            WHERE account_id = $1
              AND identity_kind = $2
              AND provider_identity_id = $3
            LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(&request.identity_kind)
        .bind(&request.provider_identity_id)
        .fetch_optional(&self.pool)
        .await?;
        let merged_metadata = if let Some((current_display_name, current_metadata)) = existing_row {
            merged_identity_display_name_metadata(
                current_display_name.as_deref(),
                &current_metadata,
                Some(request.display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "provider_member_id": request.provider_member_id,
                    "push_name": request.push_name,
                    "business_profile": request.business_profile,
                    "profile_photo_ref": request.profile_photo_ref,
                    "status": request.status,
                    "is_self": request.is_self,
                    "is_admin": request.is_admin,
                    "is_owner": request.is_owner,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.observed_at,
            )?
        } else {
            merged_identity_display_name_metadata(
                None,
                &json!({}),
                Some(request.display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "provider_member_id": request.provider_member_id,
                    "push_name": request.push_name,
                    "business_profile": request.business_profile,
                    "profile_photo_ref": request.profile_photo_ref,
                    "status": request.status,
                    "is_self": request.is_self,
                    "is_admin": request.is_admin,
                    "is_owner": request.is_owner,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.observed_at,
            )?
        };
        crate::domains::communications::fixtures::whatsapp_projection::upsert_whatsapp_identity(
            &self.pool,
            crate::domains::communications::fixtures::whatsapp_projection::WhatsappIdentityUpsert {
                identity_id: &identity_id,
                account_id,
                channel_id,
                identity_kind: &request.identity_kind,
                provider_identity_id: &request.provider_identity_id,
                display_name: &request.display_name,
                address: request.address.as_deref(),
                metadata: merged_metadata,
            },
        )
        .await?;
        Ok(identity_id)
    }

    pub(super) async fn upsert_whatsapp_status_identity(
        &self,
        account_id: &str,
        request: &NewWhatsappWebStatus,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<Option<String>, CommunicationFixtureIngestError> {
        let account_context = self.whatsapp_account_projection_context(account_id).await?;
        let Some(identity_kind) = request
            .sender_identity_kind
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };
        let channel_id = self.ensure_whatsapp_channel(account_id).await?;
        let identity_id = whatsapp_identity_id(account_id, identity_kind, &request.sender_id);
        let existing_row: Option<(Option<String>, Value)> = sqlx::query_as(
            r#"
            SELECT display_name, metadata
            FROM communication_identities
            WHERE account_id = $1
              AND identity_kind = $2
              AND provider_identity_id = $3
            LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(identity_kind)
        .bind(&request.sender_id)
        .fetch_optional(&self.pool)
        .await?;
        let merged_metadata = if let Some((current_display_name, current_metadata)) = existing_row {
            merged_identity_display_name_metadata(
                current_display_name.as_deref(),
                &current_metadata,
                Some(request.sender_display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "push_name": request.sender_push_name,
                    "business_profile": request.sender_business_profile,
                    "profile_photo_ref": request.sender_profile_photo_ref,
                    "status_author": true,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.occurred_at,
            )?
        } else {
            merged_identity_display_name_metadata(
                None,
                &json!({}),
                Some(request.sender_display_name.as_str()),
                json!({
                    "provider": account_context.provider_kind,
                    "push_name": request.sender_push_name,
                    "business_profile": request.sender_business_profile,
                    "profile_photo_ref": request.sender_profile_photo_ref,
                    "status_author": true,
                    "raw_record_id": stored_raw.raw_record_id,
                    "accepted_signal_event_id": stored_raw.accepted_event_id,
                }),
                request.occurred_at,
            )?
        };
        crate::domains::communications::fixtures::whatsapp_projection::upsert_whatsapp_identity(
            &self.pool,
            crate::domains::communications::fixtures::whatsapp_projection::WhatsappIdentityUpsert {
                identity_id: &identity_id,
                account_id,
                channel_id: &channel_id,
                identity_kind,
                provider_identity_id: &request.sender_id,
                display_name: &request.sender_display_name,
                address: request.sender_address.as_deref(),
                metadata: merged_metadata,
            },
        )
        .await?;
        Ok(Some(identity_id))
    }

    pub(super) async fn upsert_whatsapp_persona_identity_traces_for_participant(
        &self,
        request: &NewWhatsappWebParticipant,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let participant_trace_metadata = json!({
            "whatsapp_participant_evidence": {
                "provider": account_context.provider_kind,
                "account_id": request.account_id,
                "provider_chat_id": request.provider_chat_id,
                "provider_member_id": request.effective_provider_member_id(),
                "provider_identity_id": request.provider_identity_id,
                "identity_kind": request.identity_kind,
                "display_name": request.display_name,
                "push_name": request.push_name,
                "address": request.address,
                "business_profile": request.business_profile,
                "profile_photo_ref": request.profile_photo_ref,
                "role": request.role,
                "status": request.status,
                "is_self": request.is_self,
                "is_admin": request.is_admin,
                "is_owner": request.is_owner,
                "raw_record_id": stored_raw.raw_record_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
            }
        });
        self.upsert_persona_identity_trace_with_metadata(
            "whatsapp",
            Some(request.provider_identity_id.as_str()),
            participant_trace_metadata.clone(),
            stored_raw,
        )
        .await?;
        self.upsert_persona_identity_trace_with_metadata(
            "phone",
            request.address.as_deref(),
            participant_trace_metadata.clone(),
            stored_raw,
        )
        .await?;
        let trace_value = format!(
            "whatsapp_participant:v1:{}:{}:{}",
            request.account_id,
            request.provider_chat_id,
            request.effective_provider_member_id()
        );
        self.upsert_persona_identity_trace_with_metadata(
            "message_participant",
            Some(trace_value.as_str()),
            participant_trace_metadata,
            stored_raw,
        )
        .await?;
        Ok(())
    }

    pub(super) async fn upsert_whatsapp_persona_identity_traces_for_status(
        &self,
        request: &NewWhatsappWebStatus,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let status_trace_metadata = json!({
            "whatsapp_status_author_evidence": {
                "provider": account_context.provider_kind,
                "account_id": request.account_id,
                "provider_status_id": request.provider_status_id,
                "sender_id": request.sender_id,
                "sender_display_name": request.sender_display_name,
                "sender_identity_kind": request.sender_identity_kind,
                "sender_address": request.sender_address,
                "sender_push_name": request.sender_push_name,
                "sender_business_profile": request.sender_business_profile,
                "sender_profile_photo_ref": request.sender_profile_photo_ref,
                "raw_record_id": stored_raw.raw_record_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
            }
        });
        self.upsert_persona_identity_trace_with_metadata(
            "whatsapp",
            Some(request.sender_id.as_str()),
            status_trace_metadata.clone(),
            stored_raw,
        )
        .await?;
        self.upsert_persona_identity_trace_with_metadata(
            "phone",
            request.sender_address.as_deref(),
            status_trace_metadata,
            stored_raw,
        )
        .await?;
        Ok(())
    }

    pub(super) async fn upsert_persona_identity_trace(
        &self,
        identity_type: &str,
        identity_value: Option<&str>,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        self.upsert_persona_identity_trace_with_metadata(
            identity_type,
            identity_value,
            json!({}),
            stored_raw,
        )
        .await
    }

    pub(super) async fn upsert_persona_identity_trace_with_metadata(
        &self,
        identity_type: &str,
        identity_value: Option<&str>,
        metadata: Value,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let Some(identity_value) = identity_value
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        PersonaIdentityStore::new(self.pool.clone())
            .create_unattached_with_metadata_and_observation(
                identity_type,
                identity_value,
                "communication_projection",
                metadata,
                &stored_raw.observation_id,
            )
            .await?;
        Ok(())
    }

    pub(super) async fn upsert_whatsapp_persona_identity_traces_for_message(
        &self,
        request: &NewWhatsappWebMessage,
        observation_id: &str,
    ) -> Result<(), CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let Some(contact_card) = request.message_metadata.get("contact_card") else {
            return Ok(());
        };
        let Some(phones) = contact_card.get("phones").and_then(Value::as_array) else {
            return Ok(());
        };
        let store = PersonaIdentityStore::new(self.pool.clone());
        let contact_card_display_name = contact_card
            .get("display_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        for phone in phones
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            store
                .create_unattached_with_metadata_and_observation(
                    "phone",
                    phone,
                    "communication_projection",
                    json!({
                        "whatsapp_contact_card_evidence": {
                            "provider": account_context.provider_kind,
                            "account_id": request.account_id,
                            "provider_chat_id": request.provider_chat_id,
                            "provider_message_id": request.provider_message_id,
                            "sender_id": request.sender_id,
                            "sender_display_name": request.sender_display_name,
                            "contact_card": {
                                "display_name": contact_card_display_name.clone(),
                                "phones": phones,
                            }
                        }
                    }),
                    observation_id,
                )
                .await?;
        }
        Ok(())
    }

    pub(super) async fn upsert_whatsapp_conversation_participant(
        &self,
        conversation_id: &str,
        identity_id: &str,
        request: &NewWhatsappWebParticipant,
        stored_raw: &AcceptedWhatsappRawRecord,
    ) -> Result<WhatsappParticipantUpsertOutcome, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let provider_member_id = request.effective_provider_member_id();
        let participant_id =
            whatsapp_conversation_participant_id(conversation_id, provider_member_id);
        let previous_row = sqlx::query(
            r#"
            SELECT role, metadata
            FROM communication_conversation_participants
            WHERE participant_id = $1
            "#,
        )
        .bind(&participant_id)
        .fetch_optional(&self.pool)
        .await?;
        let previous_role = previous_row
            .as_ref()
            .and_then(|row| row.try_get::<Option<String>, _>("role").ok())
            .flatten();
        let previous_metadata = previous_row
            .as_ref()
            .and_then(|row| row.try_get::<Option<Value>, _>("metadata").ok())
            .flatten()
            .unwrap_or_else(|| json!({}));
        let previous_status = previous_metadata
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let role_changed = previous_role
            .as_deref()
            .is_some_and(|previous| previous != request.role);
        let membership_changed = previous_status
            .as_deref()
            .is_some_and(|previous| previous != request.status);
        let mut metadata = json!({
            "provider": account_context.provider_kind,
            "provider_member_id": provider_member_id,
            "push_name": request.push_name,
            "business_profile": request.business_profile,
            "profile_photo_ref": request.profile_photo_ref,
            "status": request.status,
            "is_self": request.is_self,
            "is_admin": request.is_admin,
            "is_owner": request.is_owner,
            "role_observed_at": request.observed_at,
            "status_observed_at": request.observed_at,
            "last_membership_observed_at": request.observed_at,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        });
        if let Some(previous_role) = previous_role.as_deref() {
            metadata["previous_role"] = json!(previous_role);
        }
        if role_changed {
            metadata["last_role_change_at"] = json!(request.observed_at);
            metadata["role_changed"] = json!(true);
        }
        if let Some(previous_status) = previous_status.as_deref() {
            metadata["previous_status"] = json!(previous_status);
        }
        if membership_changed {
            metadata["last_membership_change_at"] = json!(request.observed_at);
            metadata["membership_changed"] = json!(true);
        }
        crate::domains::communications::fixtures::whatsapp_projection::upsert_whatsapp_conversation_participant(
            &self.pool,
            crate::domains::communications::fixtures::whatsapp_projection::WhatsappConversationParticipantUpsert {
                participant_id: &participant_id,
                conversation_id,
                identity_id,
                role: &request.role,
                display_name: &request.display_name,
                address: request.address.as_deref(),
                metadata,
            },
        )
        .await?;
        Ok(WhatsappParticipantUpsertOutcome {
            participant_id,
            previous_role,
            previous_status,
            role_changed,
            membership_changed,
        })
    }

    pub(super) async fn whatsapp_account_projection_context(
        &self,
        account_id: &str,
    ) -> Result<WhatsappAccountProjectionContext, CommunicationFixtureIngestError> {
        let provider_kind: String = sqlx::query_scalar(
            r#"
            SELECT provider_kind
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;
        let channel_kind = match provider_kind.as_str() {
            "whatsapp_web" => provider_kind.clone(),
            _ => "whatsapp_web".to_owned(),
        };
        Ok(WhatsappAccountProjectionContext {
            provider_kind,
            channel_kind,
        })
    }
}
