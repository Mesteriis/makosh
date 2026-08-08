//! Zulip-owned operational projection, bounded queries and replay journal.

use std::collections::BTreeMap;

use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_zulip_api::{
    ZulipAttachmentV1, ZulipEventV1, ZulipHistoryPageV1, ZulipMessageSnapshotV1,
    ZulipReactionOperationV1,
    account::ZulipCredentialBindingStateV1,
    operational::{
        ZulipAccountStatusV1, ZulipConversationKindV1, ZulipConversationV1, ZulipHistoryStateV1,
        ZulipMessageV1, ZulipOperationalEventKindV1, ZulipOperationalEventV1,
        ZulipOperationalPageV1, ZulipOperationalQueryResponseV1, ZulipOperationalQueryV1,
        ZulipReactionStateV1, validate_operational_query,
    },
    operational_wire::{decode_operational_event, encode_operational_event},
    realtime::{
        ZulipOperationalReplayFrameV1, ZulipOperationalReplayRequestV1,
        ZulipOperationalReplayResponseV1, validate_operational_replay_request,
        validate_operational_replay_response,
    },
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, QueryBuilder, Row, Transaction};

use crate::{
    ZulipDurablePersistence, ZulipDurablePersistenceError, ZulipQueueCursorV1, validate_cursor,
};

pub struct ZulipOperationalIngestV1<'a> {
    pub cursor: &'a ZulipQueueCursorV1,
    pub events: &'a [ZulipEventV1],
    pub communications_outbox: &'a [OutboxRecordV1],
    pub delivery_route_locators: &'a [crate::ZulipDeliveryRouteLocatorV1],
    pub observed_at_unix_seconds: i64,
}

impl ZulipDurablePersistence {
    pub async fn mark_history_degraded(
        &self,
        account_id: &str,
        updated_at_unix_seconds: i64,
    ) -> Result<(), ZulipDurablePersistenceError> {
        if account_id.trim().is_empty() || updated_at_unix_seconds <= 0 {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "INSERT INTO makosh_data.zulip_operational_account_state \
             (account_id, history_state, projection_ready, updated_at_unix_seconds) \
             VALUES ($1, 4, FALSE, $2) ON CONFLICT (account_id) DO UPDATE SET \
             history_state = CASE WHEN makosh_data.zulip_operational_account_state.history_state = 3 \
                                  THEN 3 ELSE 4 END, \
             projection_ready = makosh_data.zulip_operational_account_state.projection_ready, \
             updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
        )
        .bind(account_id)
        .bind(updated_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| ZulipDurablePersistenceError::Database)
    }

    pub async fn record_operational_events_and_enqueue(
        &self,
        ingest: &ZulipOperationalIngestV1<'_>,
    ) -> Result<bool, ZulipDurablePersistenceError> {
        validate_cursor(ingest.cursor)?;
        if ingest.observed_at_unix_seconds <= 0
            || ingest
                .events
                .iter()
                .any(|event| event_account_id(event) != ingest.cursor.account_id)
        {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        if !advance_cursor_in_transaction(&mut transaction, ingest.cursor).await? {
            transaction
                .commit()
                .await
                .map_err(|_| ZulipDurablePersistenceError::Database)?;
            return Ok(false);
        }
        for event in ingest.events {
            persist_operational_event(&mut transaction, event, ingest.observed_at_unix_seconds)
                .await?;
        }
        for record in ingest.communications_outbox {
            sqlx::query(
                "INSERT INTO makosh_data.zulip_communications_outbox \
                 (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) \
                 VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING",
            )
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(ingest.observed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        }
        for locator in ingest.delivery_route_locators {
            crate::delivery_intent::upsert_delivery_route_locator(
                &mut transaction,
                locator,
                ingest.observed_at_unix_seconds,
            )
            .await?;
        }
        sqlx::query(
            "INSERT INTO makosh_data.zulip_operational_account_state \
             (account_id, history_state, last_provider_event_id, projection_ready, updated_at_unix_seconds) \
             VALUES ($1, 1, $2, FALSE, $3) \
             ON CONFLICT (account_id) DO UPDATE SET \
               last_provider_event_id = GREATEST(makosh_data.zulip_operational_account_state.last_provider_event_id, EXCLUDED.last_provider_event_id), \
               updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
        )
        .bind(&ingest.cursor.account_id)
        .bind(ingest.cursor.last_event_id)
        .bind(ingest.observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn record_history_page(
        &self,
        account_id: &str,
        page: &ZulipHistoryPageV1,
        observed_at_unix_seconds: i64,
    ) -> Result<(), ZulipDurablePersistenceError> {
        if account_id.trim().is_empty()
            || observed_at_unix_seconds <= 0
            || page
                .messages
                .iter()
                .any(|message| message.account_id != account_id || !valid_provider_id(message))
        {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        for message in &page.messages {
            persist_history_message(&mut transaction, message).await?;
        }
        let history_state = if page.found_oldest { 3_i16 } else { 2_i16 };
        sqlx::query(
            "INSERT INTO makosh_data.zulip_operational_account_state \
             (account_id, history_state, oldest_provider_message_id, projection_ready, updated_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (account_id) DO UPDATE SET \
               history_state = EXCLUDED.history_state, \
               oldest_provider_message_id = COALESCE(EXCLUDED.oldest_provider_message_id, makosh_data.zulip_operational_account_state.oldest_provider_message_id), \
               projection_ready = EXCLUDED.projection_ready, \
               updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
        )
        .bind(account_id)
        .bind(history_state)
        .bind(page.oldest_provider_message_id.as_deref())
        .bind(page.found_oldest)
        .bind(observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)
    }

    pub async fn execute_operational_query(
        &self,
        query: &ZulipOperationalQueryV1,
    ) -> Result<ZulipOperationalQueryResponseV1, ZulipDurablePersistenceError> {
        validate_operational_query(query).map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
        match query {
            ZulipOperationalQueryV1::ListMessages {
                account_id,
                provider_conversation_id,
                cursor,
                limit,
            } => {
                self.list_messages(
                    account_id,
                    provider_conversation_id.as_deref(),
                    None,
                    cursor.as_deref(),
                    *limit,
                )
                .await
            }
            ZulipOperationalQueryV1::SearchMessages {
                account_id,
                provider_conversation_id,
                query,
                cursor,
                limit,
            } => {
                self.list_messages(
                    account_id,
                    provider_conversation_id.as_deref(),
                    Some(query),
                    cursor.as_deref(),
                    *limit,
                )
                .await
            }
            ZulipOperationalQueryV1::ListConversations {
                account_id,
                cursor,
                limit,
            } => {
                self.list_conversations(account_id, cursor.as_deref(), *limit)
                    .await
            }
            ZulipOperationalQueryV1::ListEvents {
                account_id,
                kind,
                provider_conversation_id,
                cursor,
                limit,
            } => {
                self.list_events(
                    account_id,
                    *kind,
                    provider_conversation_id.as_deref(),
                    cursor.as_deref(),
                    *limit,
                )
                .await
            }
            ZulipOperationalQueryV1::GetAccountStatus { account_id } => {
                self.operational_account_status(account_id).await
            }
        }
    }

    pub async fn replay_operational_events(
        &self,
        request: &ZulipOperationalReplayRequestV1,
    ) -> Result<ZulipOperationalReplayResponseV1, ZulipDurablePersistenceError> {
        validate_operational_replay_request(request)
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
        let bounds = sqlx::query(
            "SELECT MIN(sequence) AS earliest_sequence, MAX(sequence) AS latest_sequence \
             FROM makosh_data.zulip_operational_events WHERE account_id = $1",
        )
        .bind(&request.account_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let earliest = row_optional_i64(&bounds, "earliest_sequence")?;
        let latest = row_optional_i64(&bounds, "latest_sequence")?;
        let after_sequence = i64::try_from(request.after_sequence)
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
        let cursor_exists = if request.after_sequence == 0 {
            true
        } else {
            sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM makosh_data.zulip_operational_events \
                 WHERE account_id = $1 AND sequence = $2)",
            )
            .bind(&request.account_id)
            .bind(after_sequence)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?
        };
        if request.after_sequence != 0 && !cursor_exists {
            return replay_response(&request.account_id, earliest, latest, Vec::new(), 0, true);
        }
        let rows = sqlx::query(
            "SELECT sequence, exact_event_bytes, event_sha256 \
             FROM makosh_data.zulip_operational_events \
             WHERE account_id = $1 AND sequence > $2 ORDER BY sequence ASC LIMIT $3",
        )
        .bind(&request.account_id)
        .bind(after_sequence)
        .bind(i64::from(request.limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let frames = rows
            .iter()
            .map(|row| {
                let sequence = u64::try_from(row_i64(row, "sequence")?)
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
                Ok(ZulipOperationalReplayFrameV1 {
                    sequence,
                    event: event_from_row(row, &request.account_id)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let next_sequence = frames
            .last()
            .map(|frame| frame.sequence)
            .unwrap_or(request.after_sequence);
        replay_response(
            &request.account_id,
            earliest,
            latest,
            frames,
            next_sequence,
            false,
        )
    }

    async fn list_messages(
        &self,
        account_id: &str,
        provider_conversation_id: Option<&str>,
        search: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<ZulipOperationalQueryResponseV1, ZulipDurablePersistenceError> {
        let scope = cursor_scope(&[
            "messages",
            account_id,
            provider_conversation_id.unwrap_or(""),
            search.unwrap_or(""),
        ]);
        let (cursor_sequence, cursor_message_id) = decode_pair_cursor("z1m", &scope, cursor)?;
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT account_id, provider_message_id, provider_conversation_id, sender_id, \
             is_outgoing, content, sent_at_unix_seconds, edited_at_unix_seconds, deleted, \
             last_event_sequence FROM makosh_data.zulip_operational_messages WHERE account_id = ",
        );
        builder.push_bind(account_id);
        if let Some(conversation_id) = provider_conversation_id {
            builder
                .push(" AND provider_conversation_id = ")
                .push_bind(conversation_id);
        }
        if let Some(search) = search {
            builder
                .push(" AND content IS NOT NULL AND POSITION(lower(")
                .push_bind(search)
                .push(") IN lower(content)) > 0");
        }
        builder
            .push(" AND (last_event_sequence, (provider_message_id)::BIGINT) < (")
            .push_bind(cursor_sequence)
            .push(", ")
            .push_bind(cursor_message_id)
            .push(") ORDER BY last_event_sequence DESC, (provider_message_id)::BIGINT DESC LIMIT ")
            .push_bind(i64::from(limit));
        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let provider_message_ids = rows
            .iter()
            .map(|row| row_string(row, "provider_message_id"))
            .collect::<Result<Vec<_>, _>>()?;
        let attachments = self
            .attachments_for_messages(account_id, &provider_message_ids)
            .await?;
        let reactions = self
            .reactions_for_messages(account_id, &provider_message_ids)
            .await?;
        let mut messages = Vec::with_capacity(rows.len());
        for row in &rows {
            let provider_message_id = row_string(row, "provider_message_id")?;
            messages.push(ZulipMessageV1 {
                account_id: row_string(row, "account_id")?,
                provider_message_id: provider_message_id.clone(),
                provider_conversation_id: row_string(row, "provider_conversation_id")?,
                sender_id: row_string(row, "sender_id")?,
                is_outgoing: row_bool(row, "is_outgoing")?,
                content: row_optional_string(row, "content")?,
                sent_at_unix_seconds: row_optional_i64(row, "sent_at_unix_seconds")?,
                edited_at_unix_seconds: row_optional_i64(row, "edited_at_unix_seconds")?,
                deleted: row_bool(row, "deleted")?,
                attachments: attachments
                    .get(&provider_message_id)
                    .cloned()
                    .unwrap_or_default(),
                reactions: reactions
                    .get(&provider_message_id)
                    .cloned()
                    .unwrap_or_default(),
                last_event_sequence: u64::try_from(row_i64(row, "last_event_sequence")?)
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            });
        }
        let next_cursor = (messages.len() == limit as usize)
            .then(|| messages.last())
            .flatten()
            .map(|message| {
                encode_pair_cursor(
                    "z1m",
                    &scope,
                    message.last_event_sequence,
                    &message.provider_message_id,
                )
            });
        Ok(ZulipOperationalQueryResponseV1::Messages(
            ZulipOperationalPageV1 {
                items: messages,
                next_cursor,
            },
        ))
    }

    async fn list_conversations(
        &self,
        account_id: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<ZulipOperationalQueryResponseV1, ZulipDurablePersistenceError> {
        let scope = cursor_scope(&["conversations", account_id]);
        let (cursor_sequence, cursor_message_id) = decode_pair_cursor("z1c", &scope, cursor)?;
        let rows = sqlx::query(
            "SELECT account_id, provider_conversation_id, conversation_kind, stream_id, \
             stream_name, topic, direct_recipient_id, latest_provider_message_id, latest_event_sequence \
             FROM makosh_data.zulip_operational_conversations WHERE account_id = $1 \
             AND (latest_event_sequence, (COALESCE(latest_provider_message_id, '0'))::BIGINT) < ($2, $3) \
             ORDER BY latest_event_sequence DESC, (COALESCE(latest_provider_message_id, '0'))::BIGINT DESC \
             LIMIT $4",
        )
        .bind(account_id)
        .bind(cursor_sequence)
        .bind(cursor_message_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let conversations = rows
            .iter()
            .map(conversation_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = (conversations.len() == limit as usize)
            .then(|| conversations.last())
            .flatten()
            .map(|conversation| {
                encode_pair_cursor(
                    "z1c",
                    &scope,
                    conversation.latest_event_sequence,
                    conversation
                        .latest_provider_message_id
                        .as_deref()
                        .unwrap_or("0"),
                )
            });
        Ok(ZulipOperationalQueryResponseV1::Conversations(
            ZulipOperationalPageV1 {
                items: conversations,
                next_cursor,
            },
        ))
    }

    async fn list_events(
        &self,
        account_id: &str,
        kind: Option<ZulipOperationalEventKindV1>,
        provider_conversation_id: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<ZulipOperationalQueryResponseV1, ZulipDurablePersistenceError> {
        let kind_scope = kind.map(event_kind_id).unwrap_or(0).to_string();
        let scope = cursor_scope(&[
            "events",
            account_id,
            &kind_scope,
            provider_conversation_id.unwrap_or(""),
        ]);
        let cursor_sequence = decode_sequence_cursor("z1e", &scope, cursor)?;
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT sequence, exact_event_bytes, event_sha256 FROM \
             makosh_data.zulip_operational_events WHERE account_id = ",
        );
        builder.push_bind(account_id);
        if let Some(kind) = kind {
            builder
                .push(" AND event_kind = ")
                .push_bind(event_kind_id(kind));
        }
        if let Some(conversation_id) = provider_conversation_id {
            builder
                .push(" AND provider_conversation_id = ")
                .push_bind(conversation_id);
        }
        builder
            .push(" AND sequence < ")
            .push_bind(cursor_sequence)
            .push(" ORDER BY sequence DESC LIMIT ")
            .push_bind(i64::from(limit));
        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let events = rows
            .iter()
            .map(|row| event_from_row(row, account_id))
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = (events.len() == limit as usize)
            .then(|| rows.last())
            .flatten()
            .map(|row| {
                row_i64(row, "sequence")
                    .map(|sequence| encode_sequence_cursor("z1e", &scope, sequence))
            })
            .transpose()?;
        Ok(ZulipOperationalQueryResponseV1::Events(
            ZulipOperationalPageV1 {
                items: events,
                next_cursor,
            },
        ))
    }

    async fn operational_account_status(
        &self,
        account_id: &str,
    ) -> Result<ZulipOperationalQueryResponseV1, ZulipDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT COALESCE(state.account_id, binding.account_id) AS account_id, \
             COALESCE(state.history_state, 1::SMALLINT) AS history_state, \
             state.oldest_provider_message_id, state.last_provider_event_id, \
             COALESCE(state.projection_ready, FALSE) AS projection_ready, \
             binding.credential_revision, COALESCE(binding.binding_revision, 0) AS binding_revision, \
             binding.state AS credential_state, binding.applied_runtime_generation, \
             COALESCE(MAX(events.sequence), 0) AS latest_event_sequence \
             FROM makosh_data.zulip_operational_account_state state \
             FULL OUTER JOIN makosh_data.zulip_account_credential_bindings binding \
               ON binding.account_id = state.account_id \
             LEFT JOIN makosh_data.zulip_operational_events events \
               ON events.account_id = COALESCE(state.account_id, binding.account_id) \
             WHERE COALESCE(state.account_id, binding.account_id) = $1 \
             GROUP BY state.account_id, state.history_state, state.oldest_provider_message_id, \
             state.last_provider_event_id, state.projection_ready, binding.account_id, \
             binding.credential_revision, binding.binding_revision, binding.state, \
             binding.applied_runtime_generation",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let status = match row {
            Some(row) => ZulipAccountStatusV1 {
                account_id: row_string(&row, "account_id")?,
                projection_ready: row_bool(&row, "projection_ready")?,
                history_state: history_state_from_id(
                    row.try_get::<i16, _>("history_state")
                        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                )?,
                oldest_provider_message_id: row_optional_string(
                    &row,
                    "oldest_provider_message_id",
                )?,
                last_provider_event_id: row_optional_i64(&row, "last_provider_event_id")?,
                latest_event_sequence: u64::try_from(row_i64(&row, "latest_event_sequence")?)
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                credential_state: credential_state_from_optional_i16(
                    row.try_get::<Option<i16>, _>("credential_state")
                        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                )?,
                credential_revision: optional_u64(&row, "credential_revision")?,
                binding_revision: u64::try_from(row_i64(&row, "binding_revision")?)
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                applied_runtime_generation: optional_u64(&row, "applied_runtime_generation")?,
            },
            None => ZulipAccountStatusV1 {
                account_id: account_id.to_owned(),
                projection_ready: false,
                history_state: ZulipHistoryStateV1::NotStarted,
                oldest_provider_message_id: None,
                last_provider_event_id: None,
                latest_event_sequence: 0,
                credential_state: ZulipCredentialBindingStateV1::Unconfigured,
                credential_revision: None,
                binding_revision: 0,
                applied_runtime_generation: None,
            },
        };
        Ok(ZulipOperationalQueryResponseV1::AccountStatus(status))
    }

    async fn attachments_for_messages(
        &self,
        account_id: &str,
        provider_message_ids: &[String],
    ) -> Result<BTreeMap<String, Vec<ZulipAttachmentV1>>, ZulipDurablePersistenceError> {
        if provider_message_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let rows = sqlx::query(
            "SELECT provider_message_id, provider_attachment_id, filename \
             FROM makosh_data.zulip_operational_attachments \
             WHERE account_id = $1 AND provider_message_id = ANY($2) \
             ORDER BY provider_message_id, provider_attachment_id",
        )
        .bind(account_id)
        .bind(provider_message_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let mut values = BTreeMap::new();
        for row in &rows {
            values
                .entry(row_string(row, "provider_message_id")?)
                .or_insert_with(Vec::new)
                .push(ZulipAttachmentV1 {
                    provider_attachment_id: row_string(row, "provider_attachment_id")?,
                    filename: row_optional_string(row, "filename")?,
                });
        }
        Ok(values)
    }

    async fn reactions_for_messages(
        &self,
        account_id: &str,
        provider_message_ids: &[String],
    ) -> Result<BTreeMap<String, Vec<ZulipReactionStateV1>>, ZulipDurablePersistenceError> {
        if provider_message_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let rows = sqlx::query(
            "SELECT provider_message_id, actor_id, emoji_name, emoji_code, reaction_type \
             FROM makosh_data.zulip_operational_reactions \
             WHERE account_id = $1 AND provider_message_id = ANY($2) AND present \
             ORDER BY provider_message_id, actor_id, emoji_name, emoji_code, reaction_type",
        )
        .bind(account_id)
        .bind(provider_message_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let mut values = BTreeMap::new();
        for row in &rows {
            let emoji_code = row_string(row, "emoji_code")?;
            let reaction_type = row_string(row, "reaction_type")?;
            values
                .entry(row_string(row, "provider_message_id")?)
                .or_insert_with(Vec::new)
                .push(ZulipReactionStateV1 {
                    actor_id: row_string(row, "actor_id")?,
                    emoji_name: row_string(row, "emoji_name")?,
                    emoji_code: (!emoji_code.is_empty()).then_some(emoji_code),
                    reaction_type: (!reaction_type.is_empty()).then_some(reaction_type),
                });
        }
        Ok(values)
    }
}

fn credential_state_from_optional_i16(
    state: Option<i16>,
) -> Result<ZulipCredentialBindingStateV1, ZulipDurablePersistenceError> {
    match state {
        None => Ok(ZulipCredentialBindingStateV1::Unconfigured),
        Some(2) => Ok(ZulipCredentialBindingStateV1::PendingRestart),
        Some(3) => Ok(ZulipCredentialBindingStateV1::Active),
        Some(4) => Ok(ZulipCredentialBindingStateV1::Retired),
        Some(_) => Err(ZulipDurablePersistenceError::InvalidRow),
    }
}

fn optional_u64(
    row: &sqlx::postgres::PgRow,
    field: &str,
) -> Result<Option<u64>, ZulipDurablePersistenceError> {
    row.try_get::<Option<i64>, _>(field)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?
        .map(|value| u64::try_from(value).map_err(|_| ZulipDurablePersistenceError::InvalidRow))
        .transpose()
}

async fn advance_cursor_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    cursor: &ZulipQueueCursorV1,
) -> Result<bool, ZulipDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.zulip_provider_cursor (account_id, queue_id, last_event_id) \
         VALUES ($1, $2, $3) ON CONFLICT (account_id) DO UPDATE \
         SET queue_id = EXCLUDED.queue_id, last_event_id = EXCLUDED.last_event_id \
         WHERE zulip_provider_cursor.queue_id <> EXCLUDED.queue_id \
            OR zulip_provider_cursor.last_event_id < EXCLUDED.last_event_id \
         RETURNING account_id",
    )
    .bind(&cursor.account_id)
    .bind(&cursor.queue_id)
    .bind(cursor.last_event_id)
    .fetch_optional(&mut **transaction)
    .await
    .map(|row| row.is_some())
    .map_err(|_| ZulipDurablePersistenceError::Database)
}

async fn persist_operational_event(
    transaction: &mut Transaction<'_, Postgres>,
    provider_event: &ZulipEventV1,
    observed_at_unix_seconds: i64,
) -> Result<(), ZulipDurablePersistenceError> {
    let event = operational_event(provider_event, observed_at_unix_seconds)?;
    let exact_event_bytes = encode_operational_event(&event);
    let event_sha256: [u8; 32] = Sha256::digest(&exact_event_bytes).into();
    let sequence = sqlx::query(
        "INSERT INTO makosh_data.zulip_operational_events \
         (account_id, provider_event_id, provider_message_id, provider_conversation_id, \
          event_kind, exact_event_bytes, event_sha256, observed_at_unix_seconds) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (account_id, provider_event_id, provider_message_id, event_kind) DO NOTHING \
         RETURNING sequence",
    )
    .bind(&event.account_id)
    .bind(event.provider_event_id)
    .bind(&event.provider_message_id)
    .bind(event.provider_conversation_id.as_deref())
    .bind(event_kind_id(event.kind))
    .bind(&exact_event_bytes)
    .bind(event_sha256.as_slice())
    .bind(observed_at_unix_seconds)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| ZulipDurablePersistenceError::Database)?
    .map(|row| row_i64(&row, "sequence"))
    .transpose()?;
    let Some(sequence) = sequence else {
        return Ok(());
    };
    apply_provider_event(transaction, provider_event, sequence).await
}

async fn apply_provider_event(
    transaction: &mut Transaction<'_, Postgres>,
    event: &ZulipEventV1,
    sequence: i64,
) -> Result<(), ZulipDurablePersistenceError> {
    match event {
        ZulipEventV1::Message {
            account_id,
            provider_message_id,
            provider_conversation_id,
            conversation_kind,
            stream_id,
            stream_name,
            topic,
            direct_recipient_id,
            sender_id,
            is_outgoing,
            content,
            sent_at_unix_seconds,
            attachments,
            reactions,
            ..
        } => {
            let result = sqlx::query(
                "INSERT INTO makosh_data.zulip_operational_messages \
                 (account_id, provider_message_id, provider_conversation_id, conversation_kind, \
                  stream_id, stream_name, topic, direct_recipient_id, sender_id, is_outgoing, \
                  content, sent_at_unix_seconds, deleted, last_event_sequence) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, FALSE, $13) \
                 ON CONFLICT (account_id, provider_message_id) DO UPDATE SET \
                   provider_conversation_id = EXCLUDED.provider_conversation_id, \
                   conversation_kind = EXCLUDED.conversation_kind, stream_id = EXCLUDED.stream_id, \
                   stream_name = EXCLUDED.stream_name, topic = EXCLUDED.topic, \
                   direct_recipient_id = EXCLUDED.direct_recipient_id, sender_id = EXCLUDED.sender_id, \
                   is_outgoing = EXCLUDED.is_outgoing, content = EXCLUDED.content, \
                   sent_at_unix_seconds = EXCLUDED.sent_at_unix_seconds, deleted = FALSE, \
                   last_event_sequence = EXCLUDED.last_event_sequence \
                 WHERE makosh_data.zulip_operational_messages.last_event_sequence < EXCLUDED.last_event_sequence",
            )
            .bind(account_id)
            .bind(provider_message_id)
            .bind(provider_conversation_id)
            .bind(conversation_kind_id(*conversation_kind))
            .bind(stream_id.as_deref())
            .bind(stream_name.as_deref())
            .bind(topic.as_deref())
            .bind(direct_recipient_id.as_deref())
            .bind(sender_id)
            .bind(is_outgoing)
            .bind(content.as_deref())
            .bind(sent_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
            if result.rows_affected() == 1 {
                replace_attachments(transaction, account_id, provider_message_id, attachments)
                    .await?;
                merge_reactions(
                    transaction,
                    account_id,
                    provider_message_id,
                    reactions,
                    sequence,
                )
                .await?;
                upsert_conversation_from_message(transaction, account_id, provider_message_id)
                    .await?;
            }
        }
        ZulipEventV1::MessageUpdated {
            account_id,
            provider_message_id,
            content,
            topic,
            edited_at_unix_seconds,
            ..
        } => {
            persist_message_mutation(
                transaction,
                account_id,
                provider_message_id,
                content.as_deref(),
                topic.as_deref(),
                *edited_at_unix_seconds,
                false,
                sequence,
            )
            .await?;
            apply_message_mutation(transaction, account_id, provider_message_id).await?;
            upsert_conversation_from_message(transaction, account_id, provider_message_id).await?;
        }
        ZulipEventV1::MessageDeleted {
            account_id,
            provider_message_id,
            ..
        } => {
            persist_message_mutation(
                transaction,
                account_id,
                provider_message_id,
                None,
                None,
                None,
                true,
                sequence,
            )
            .await?;
            apply_message_mutation(transaction, account_id, provider_message_id).await?;
        }
        ZulipEventV1::ReactionChanged {
            account_id,
            provider_message_id,
            reaction,
            operation,
            ..
        } => {
            sqlx::query(
                "INSERT INTO makosh_data.zulip_operational_reactions \
                 (account_id, provider_message_id, actor_id, emoji_name, emoji_code, reaction_type, \
                  present, last_event_sequence) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                 ON CONFLICT (account_id, provider_message_id, actor_id, emoji_name, emoji_code, reaction_type) \
                 DO UPDATE SET present = EXCLUDED.present, last_event_sequence = EXCLUDED.last_event_sequence \
                 WHERE makosh_data.zulip_operational_reactions.last_event_sequence < EXCLUDED.last_event_sequence",
            )
            .bind(account_id)
            .bind(provider_message_id)
            .bind(&reaction.actor_id)
            .bind(&reaction.emoji_name)
            .bind(reaction.emoji_code.as_deref().unwrap_or(""))
            .bind(reaction.reaction_type.as_deref().unwrap_or(""))
            .bind(matches!(operation, ZulipReactionOperationV1::Add))
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        }
    }
    Ok(())
}

async fn persist_history_message(
    transaction: &mut Transaction<'_, Postgres>,
    message: &ZulipMessageSnapshotV1,
) -> Result<(), ZulipDurablePersistenceError> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.zulip_operational_messages \
         (account_id, provider_message_id, provider_conversation_id, conversation_kind, \
          stream_id, stream_name, topic, direct_recipient_id, sender_id, is_outgoing, content, \
          sent_at_unix_seconds, edited_at_unix_seconds, deleted, last_event_sequence) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, FALSE, 0) \
         ON CONFLICT (account_id, provider_message_id) DO NOTHING",
    )
    .bind(&message.account_id)
    .bind(&message.provider_message_id)
    .bind(&message.provider_conversation_id)
    .bind(conversation_kind_id(message.conversation_kind))
    .bind(message.stream_id.as_deref())
    .bind(message.stream_name.as_deref())
    .bind(message.topic.as_deref())
    .bind(message.direct_recipient_id.as_deref())
    .bind(&message.sender_id)
    .bind(message.is_outgoing)
    .bind(message.content.as_deref())
    .bind(message.sent_at_unix_seconds)
    .bind(message.edited_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ZulipDurablePersistenceError::Database)?;
    if inserted.rows_affected() == 1 {
        replace_attachments(
            transaction,
            &message.account_id,
            &message.provider_message_id,
            &message.attachments,
        )
        .await?;
        merge_reactions(
            transaction,
            &message.account_id,
            &message.provider_message_id,
            &message.reactions,
            0,
        )
        .await?;
        apply_message_mutation(
            transaction,
            &message.account_id,
            &message.provider_message_id,
        )
        .await?;
        upsert_conversation_from_message(
            transaction,
            &message.account_id,
            &message.provider_message_id,
        )
        .await?;
    }
    Ok(())
}

async fn replace_attachments(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
    attachments: &[ZulipAttachmentV1],
) -> Result<(), ZulipDurablePersistenceError> {
    sqlx::query(
        "DELETE FROM makosh_data.zulip_operational_attachments \
         WHERE account_id = $1 AND provider_message_id = $2",
    )
    .bind(account_id)
    .bind(provider_message_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ZulipDurablePersistenceError::Database)?;
    for attachment in attachments {
        if attachment.provider_attachment_id.trim().is_empty() {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "INSERT INTO makosh_data.zulip_operational_attachments \
             (account_id, provider_message_id, provider_attachment_id, filename) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(account_id)
        .bind(provider_message_id)
        .bind(&attachment.provider_attachment_id)
        .bind(attachment.filename.as_deref())
        .execute(&mut **transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
    }
    Ok(())
}

async fn merge_reactions(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
    reactions: &[ZulipReactionStateV1],
    sequence: i64,
) -> Result<(), ZulipDurablePersistenceError> {
    for reaction in reactions {
        sqlx::query(
            "INSERT INTO makosh_data.zulip_operational_reactions \
             (account_id, provider_message_id, actor_id, emoji_name, emoji_code, reaction_type, \
              present, last_event_sequence) VALUES ($1, $2, $3, $4, $5, $6, TRUE, $7) \
             ON CONFLICT (account_id, provider_message_id, actor_id, emoji_name, emoji_code, reaction_type) \
             DO UPDATE SET present = TRUE, last_event_sequence = EXCLUDED.last_event_sequence \
             WHERE makosh_data.zulip_operational_reactions.last_event_sequence < EXCLUDED.last_event_sequence",
        )
        .bind(account_id)
        .bind(provider_message_id)
        .bind(&reaction.actor_id)
        .bind(&reaction.emoji_name)
        .bind(reaction.emoji_code.as_deref().unwrap_or(""))
        .bind(reaction.reaction_type.as_deref().unwrap_or(""))
        .bind(sequence)
        .execute(&mut **transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn persist_message_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
    content: Option<&str>,
    topic: Option<&str>,
    edited_at_unix_seconds: Option<i64>,
    deleted: bool,
    sequence: i64,
) -> Result<(), ZulipDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.zulip_operational_message_mutations \
         (account_id, provider_message_id, content, topic, edited_at_unix_seconds, deleted, last_event_sequence) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (account_id, provider_message_id) DO UPDATE SET \
           content = COALESCE(EXCLUDED.content, makosh_data.zulip_operational_message_mutations.content), \
           topic = COALESCE(EXCLUDED.topic, makosh_data.zulip_operational_message_mutations.topic), \
           edited_at_unix_seconds = COALESCE(EXCLUDED.edited_at_unix_seconds, makosh_data.zulip_operational_message_mutations.edited_at_unix_seconds), \
           deleted = makosh_data.zulip_operational_message_mutations.deleted OR EXCLUDED.deleted, \
           last_event_sequence = EXCLUDED.last_event_sequence \
         WHERE makosh_data.zulip_operational_message_mutations.last_event_sequence < EXCLUDED.last_event_sequence",
    )
    .bind(account_id)
    .bind(provider_message_id)
    .bind(content)
    .bind(topic)
    .bind(edited_at_unix_seconds)
    .bind(deleted)
    .bind(sequence)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ZulipDurablePersistenceError::Database)
}

async fn apply_message_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
) -> Result<(), ZulipDurablePersistenceError> {
    sqlx::query(
        "UPDATE makosh_data.zulip_operational_messages message SET \
           content = CASE WHEN mutation.deleted THEN NULL ELSE COALESCE(mutation.content, message.content) END, \
           topic = COALESCE(mutation.topic, message.topic), \
           provider_conversation_id = CASE \
             WHEN mutation.topic IS NOT NULL AND message.conversation_kind = 1 \
             THEN 'stream:' || message.stream_id || ':' || mutation.topic \
             ELSE message.provider_conversation_id END, \
           edited_at_unix_seconds = COALESCE(mutation.edited_at_unix_seconds, message.edited_at_unix_seconds), \
           deleted = mutation.deleted, last_event_sequence = mutation.last_event_sequence \
         FROM makosh_data.zulip_operational_message_mutations mutation \
         WHERE message.account_id = mutation.account_id \
           AND message.provider_message_id = mutation.provider_message_id \
           AND message.account_id = $1 AND message.provider_message_id = $2 \
           AND message.last_event_sequence < mutation.last_event_sequence",
    )
    .bind(account_id)
    .bind(provider_message_id)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ZulipDurablePersistenceError::Database)
}

async fn upsert_conversation_from_message(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
) -> Result<(), ZulipDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.zulip_operational_conversations \
         (account_id, provider_conversation_id, conversation_kind, stream_id, stream_name, topic, \
          direct_recipient_id, latest_provider_message_id, latest_event_sequence) \
         SELECT account_id, provider_conversation_id, conversation_kind, stream_id, stream_name, topic, \
                direct_recipient_id, provider_message_id, last_event_sequence \
         FROM makosh_data.zulip_operational_messages \
         WHERE account_id = $1 AND provider_message_id = $2 \
         ON CONFLICT (account_id, provider_conversation_id) DO UPDATE SET \
           conversation_kind = EXCLUDED.conversation_kind, stream_id = EXCLUDED.stream_id, \
           stream_name = COALESCE(EXCLUDED.stream_name, makosh_data.zulip_operational_conversations.stream_name), \
           topic = EXCLUDED.topic, direct_recipient_id = EXCLUDED.direct_recipient_id, \
           latest_provider_message_id = EXCLUDED.latest_provider_message_id, \
           latest_event_sequence = EXCLUDED.latest_event_sequence \
         WHERE (makosh_data.zulip_operational_conversations.latest_event_sequence, \
                (COALESCE(makosh_data.zulip_operational_conversations.latest_provider_message_id, '0'))::BIGINT) \
             < (EXCLUDED.latest_event_sequence, (COALESCE(EXCLUDED.latest_provider_message_id, '0'))::BIGINT)",
    )
    .bind(account_id)
    .bind(provider_message_id)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ZulipDurablePersistenceError::Database)
}

fn operational_event(
    event: &ZulipEventV1,
    observed_at_unix_seconds: i64,
) -> Result<ZulipOperationalEventV1, ZulipDurablePersistenceError> {
    let value = match event {
        ZulipEventV1::Message {
            account_id,
            event_id,
            provider_message_id,
            provider_conversation_id,
            sender_id,
            content,
            topic,
            ..
        } => ZulipOperationalEventV1 {
            account_id: account_id.clone(),
            provider_event_id: *event_id,
            provider_message_id: provider_message_id.clone(),
            provider_conversation_id: Some(provider_conversation_id.clone()),
            actor_id: Some(sender_id.clone()),
            kind: ZulipOperationalEventKindV1::MessageUpserted,
            content: content.clone(),
            topic: topic.clone(),
            reaction: None,
            observed_at_unix_seconds,
        },
        ZulipEventV1::MessageUpdated {
            account_id,
            event_id,
            provider_message_id,
            content,
            topic,
            ..
        } => ZulipOperationalEventV1 {
            account_id: account_id.clone(),
            provider_event_id: *event_id,
            provider_message_id: provider_message_id.clone(),
            provider_conversation_id: None,
            actor_id: None,
            kind: ZulipOperationalEventKindV1::MessageUpdated,
            content: content.clone(),
            topic: topic.clone(),
            reaction: None,
            observed_at_unix_seconds,
        },
        ZulipEventV1::MessageDeleted {
            account_id,
            event_id,
            provider_message_id,
        } => ZulipOperationalEventV1 {
            account_id: account_id.clone(),
            provider_event_id: *event_id,
            provider_message_id: provider_message_id.clone(),
            provider_conversation_id: None,
            actor_id: None,
            kind: ZulipOperationalEventKindV1::MessageDeleted,
            content: None,
            topic: None,
            reaction: None,
            observed_at_unix_seconds,
        },
        ZulipEventV1::ReactionChanged {
            account_id,
            event_id,
            provider_message_id,
            actor_id,
            reaction,
            operation,
        } => ZulipOperationalEventV1 {
            account_id: account_id.clone(),
            provider_event_id: *event_id,
            provider_message_id: provider_message_id.clone(),
            provider_conversation_id: None,
            actor_id: Some(actor_id.clone()),
            kind: match operation {
                ZulipReactionOperationV1::Add => ZulipOperationalEventKindV1::ReactionAdded,
                ZulipReactionOperationV1::Remove => ZulipOperationalEventKindV1::ReactionRemoved,
            },
            content: None,
            topic: None,
            reaction: Some(reaction.clone()),
            observed_at_unix_seconds,
        },
    };
    if value.provider_event_id <= 0
        || value.account_id.trim().is_empty()
        || value
            .provider_message_id
            .parse::<i64>()
            .ok()
            .filter(|id| *id > 0)
            .is_none()
    {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    Ok(value)
}

fn event_account_id(event: &ZulipEventV1) -> &str {
    match event {
        ZulipEventV1::Message { account_id, .. }
        | ZulipEventV1::MessageUpdated { account_id, .. }
        | ZulipEventV1::MessageDeleted { account_id, .. }
        | ZulipEventV1::ReactionChanged { account_id, .. } => account_id,
    }
}

fn valid_provider_id(message: &ZulipMessageSnapshotV1) -> bool {
    message
        .provider_message_id
        .parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .is_some()
        && !message.provider_conversation_id.trim().is_empty()
        && !message.sender_id.trim().is_empty()
}

const fn conversation_kind_id(kind: ZulipConversationKindV1) -> i16 {
    match kind {
        ZulipConversationKindV1::StreamTopic => 1,
        ZulipConversationKindV1::Direct => 2,
    }
}

fn conversation_kind_from_id(
    value: i16,
) -> Result<ZulipConversationKindV1, ZulipDurablePersistenceError> {
    match value {
        1 => Ok(ZulipConversationKindV1::StreamTopic),
        2 => Ok(ZulipConversationKindV1::Direct),
        _ => Err(ZulipDurablePersistenceError::InvalidRow),
    }
}

const fn event_kind_id(kind: ZulipOperationalEventKindV1) -> i16 {
    match kind {
        ZulipOperationalEventKindV1::MessageUpserted => 1,
        ZulipOperationalEventKindV1::MessageUpdated => 2,
        ZulipOperationalEventKindV1::MessageDeleted => 3,
        ZulipOperationalEventKindV1::ReactionAdded => 4,
        ZulipOperationalEventKindV1::ReactionRemoved => 5,
    }
}

fn history_state_from_id(value: i16) -> Result<ZulipHistoryStateV1, ZulipDurablePersistenceError> {
    match value {
        1 => Ok(ZulipHistoryStateV1::NotStarted),
        2 => Ok(ZulipHistoryStateV1::Syncing),
        3 => Ok(ZulipHistoryStateV1::Ready),
        4 => Ok(ZulipHistoryStateV1::Degraded),
        _ => Err(ZulipDurablePersistenceError::InvalidRow),
    }
}

fn conversation_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<ZulipConversationV1, ZulipDurablePersistenceError> {
    Ok(ZulipConversationV1 {
        account_id: row_string(row, "account_id")?,
        provider_conversation_id: row_string(row, "provider_conversation_id")?,
        kind: conversation_kind_from_id(
            row.try_get::<i16, _>("conversation_kind")
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
        )?,
        stream_id: row_optional_string(row, "stream_id")?,
        stream_name: row_optional_string(row, "stream_name")?,
        topic: row_optional_string(row, "topic")?,
        direct_recipient_id: row_optional_string(row, "direct_recipient_id")?,
        latest_provider_message_id: row_optional_string(row, "latest_provider_message_id")?,
        latest_event_sequence: u64::try_from(row_i64(row, "latest_event_sequence")?)
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
    })
}

fn event_from_row(
    row: &sqlx::postgres::PgRow,
    account_id: &str,
) -> Result<ZulipOperationalEventV1, ZulipDurablePersistenceError> {
    let exact_event_bytes: Vec<u8> = row
        .try_get("exact_event_bytes")
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
    let event_sha256: Vec<u8> = row
        .try_get("event_sha256")
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
    if event_sha256.as_slice() != Sha256::digest(&exact_event_bytes).as_slice() {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    let event = decode_operational_event(&exact_event_bytes)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
    if event.account_id != account_id {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    Ok(event)
}

fn replay_response(
    account_id: &str,
    earliest: Option<i64>,
    latest: Option<i64>,
    frames: Vec<ZulipOperationalReplayFrameV1>,
    next_sequence: u64,
    reset_required: bool,
) -> Result<ZulipOperationalReplayResponseV1, ZulipDurablePersistenceError> {
    let response = ZulipOperationalReplayResponseV1 {
        earliest_available_sequence: earliest
            .map(u64::try_from)
            .transpose()
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
        latest_available_sequence: latest
            .map(u64::try_from)
            .transpose()
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
        frames,
        next_sequence,
        reset_required,
        account_id: account_id.to_owned(),
    };
    validate_operational_replay_response(&response)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
    Ok(response)
}

fn cursor_scope(parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    digest
        .finalize()
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_pair_cursor(
    prefix: &str,
    scope: &str,
    cursor: Option<&str>,
) -> Result<(i64, i64), ZulipDurablePersistenceError> {
    let Some(cursor) = cursor else {
        return Ok((i64::MAX, i64::MAX));
    };
    let parts = cursor.split('.').collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != prefix || parts[1] != scope {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    let sequence = parts[2]
        .parse::<i64>()
        .ok()
        .filter(|value| *value >= 0)
        .ok_or(ZulipDurablePersistenceError::InvalidRow)?;
    let message_id = parts[3]
        .parse::<i64>()
        .ok()
        .filter(|value| *value >= 0)
        .ok_or(ZulipDurablePersistenceError::InvalidRow)?;
    Ok((sequence, message_id))
}

fn encode_pair_cursor(prefix: &str, scope: &str, sequence: u64, message_id: &str) -> String {
    format!("{prefix}.{scope}.{sequence}.{message_id}")
}

fn decode_sequence_cursor(
    prefix: &str,
    scope: &str,
    cursor: Option<&str>,
) -> Result<i64, ZulipDurablePersistenceError> {
    let Some(cursor) = cursor else {
        return Ok(i64::MAX);
    };
    let parts = cursor.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0] != prefix || parts[1] != scope {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    parts[2]
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ZulipDurablePersistenceError::InvalidRow)
}

fn encode_sequence_cursor(prefix: &str, scope: &str, sequence: i64) -> String {
    format!("{prefix}.{scope}.{sequence}")
}

fn row_string(
    row: &sqlx::postgres::PgRow,
    field: &str,
) -> Result<String, ZulipDurablePersistenceError> {
    row.try_get(field)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
}

fn row_optional_string(
    row: &sqlx::postgres::PgRow,
    field: &str,
) -> Result<Option<String>, ZulipDurablePersistenceError> {
    row.try_get(field)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
}

fn row_i64(row: &sqlx::postgres::PgRow, field: &str) -> Result<i64, ZulipDurablePersistenceError> {
    row.try_get(field)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
}

fn row_optional_i64(
    row: &sqlx::postgres::PgRow,
    field: &str,
) -> Result<Option<i64>, ZulipDurablePersistenceError> {
    row.try_get(field)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
}

fn row_bool(
    row: &sqlx::postgres::PgRow,
    field: &str,
) -> Result<bool, ZulipDurablePersistenceError> {
    row.try_get(field)
        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursors_are_query_scoped_and_reject_cross_surface_reuse() {
        let scope = cursor_scope(&["messages", "account", "conversation", "query"]);
        let cursor = encode_pair_cursor("z1m", &scope, 7, "42");
        assert_eq!(
            decode_pair_cursor("z1m", &scope, Some(&cursor)),
            Ok((7, 42))
        );
        assert_eq!(
            decode_pair_cursor("z1c", &scope, Some(&cursor)),
            Err(ZulipDurablePersistenceError::InvalidRow)
        );
    }

    #[test]
    fn operational_event_preserves_reaction_operation() {
        let event = ZulipEventV1::ReactionChanged {
            account_id: "account".into(),
            event_id: 9,
            provider_message_id: "42".into(),
            actor_id: "7".into(),
            reaction: ZulipReactionStateV1 {
                actor_id: "7".into(),
                emoji_name: "thumbs_up".into(),
                emoji_code: Some("1f44d".into()),
                reaction_type: Some("unicode_emoji".into()),
            },
            operation: ZulipReactionOperationV1::Remove,
        };
        assert_eq!(
            operational_event(&event, 11).expect("event").kind,
            ZulipOperationalEventKindV1::ReactionRemoved
        );
    }
}
