//! WhatsApp-owned operational projections, bounded queries and host-ingestion
//! transaction. Communications tables and provider session state are outside
//! this module.

use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_whatsapp_api::{
    WhatsAppDialog, WhatsAppMessage, WhatsAppParticipant, WhatsAppProviderEvent,
    operational::{
        WhatsAppOperationalPageV1, WhatsAppOperationalQueryResponseV1, WhatsAppOperationalQueryV1,
        WhatsAppOperationalRuntimeStatusV1, validate_operational_query,
    },
    operational_wire::{decode_provider_event, encode_provider_event},
    provider_event_account_id, provider_event_chat_id, provider_event_kind,
    realtime::{
        WhatsAppOperationalReplayFrameV1, WhatsAppOperationalReplayRequestV1,
        WhatsAppOperationalReplayResponseV1, validate_operational_replay_request,
        validate_operational_replay_response,
    },
    validate_event,
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, QueryBuilder, Row, Transaction};

use crate::{
    WhatsAppDurablePersistence, WhatsAppDurablePersistenceError, WhatsAppHostObservationRecordV1,
};

const TOMBSTONE_KIND_MESSAGE: i16 = 1;
const TOMBSTONE_KIND_PARTICIPANT: i16 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhatsAppOperationalObservationV1 {
    Event {
        provider_event_id: String,
        event: WhatsAppProviderEvent,
    },
    ResyncState {
        provider_event_id: String,
        account_id: String,
        observed_at_unix_seconds: i64,
        complete: bool,
    },
}

impl WhatsAppDurablePersistence {
    pub async fn record_host_observation_projection_and_enqueue(
        &self,
        observation: &WhatsAppHostObservationRecordV1,
        operational: Option<&WhatsAppOperationalObservationV1>,
        outbox: Option<&OutboxRecordV1>,
        delivery_route_locator: Option<&crate::WhatsAppDeliveryRouteLocatorV1>,
        created_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        if let Some(operational @ WhatsAppOperationalObservationV1::ResyncState { .. }) =
            operational
        {
            if outbox.is_some() {
                return Err(WhatsAppDurablePersistenceError::InvalidRow);
            }
            return self
                .record_operational_resync_control(observation, operational)
                .await;
        }
        validate_host_record(observation)?;
        let operational_sha256 = operational
            .map(|value| operational_digest(observation, value))
            .transpose()?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.whatsapp_host_observations (account_id, provider_event_id, evidence_kind, observed_at_unix_seconds, operational_sha256) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (account_id, provider_event_id) DO NOTHING RETURNING account_id",
        )
        .bind(&observation.account_id)
        .bind(&observation.provider_event_id)
        .bind(observation.evidence_kind)
        .bind(observation.observed_at_unix_seconds)
        .bind(operational_sha256.as_ref().map(<[u8; 32]>::as_slice))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if inserted.is_none() {
            verify_existing_observation(&mut transaction, observation, operational_sha256.as_ref())
                .await?;
            transaction
                .commit()
                .await
                .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            return Ok(false);
        }

        if let Some(operational) = operational {
            persist_operational_observation(
                &mut transaction,
                observation,
                operational,
                operational_sha256
                    .as_ref()
                    .ok_or(WhatsAppDurablePersistenceError::InvalidRow)?,
            )
            .await?;
        }
        if let Some(record) = outbox {
            sqlx::query(
                "INSERT INTO makosh_data.whatsapp_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING",
            )
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        if let Some(locator) = delivery_route_locator {
            crate::delivery_intent::upsert_delivery_route_locator(
                &mut transaction,
                locator,
                created_at_unix_seconds,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn execute_operational_query(
        &self,
        query: &WhatsAppOperationalQueryV1,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppDurablePersistenceError> {
        validate_operational_query(query)
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        match query {
            WhatsAppOperationalQueryV1::ListMessages {
                account_id,
                provider_chat_id,
                cursor,
                limit,
            } => {
                self.list_messages(
                    account_id,
                    provider_chat_id.as_deref(),
                    None,
                    cursor,
                    *limit,
                )
                .await
            }
            WhatsAppOperationalQueryV1::SearchMessages {
                account_id,
                provider_chat_id,
                query,
                cursor,
                limit,
            } => {
                self.list_messages(
                    account_id,
                    provider_chat_id.as_deref(),
                    Some(query),
                    cursor,
                    *limit,
                )
                .await
            }
            WhatsAppOperationalQueryV1::ListDialogs {
                account_id,
                cursor,
                limit,
            } => self.list_dialogs(account_id, cursor, *limit).await,
            WhatsAppOperationalQueryV1::ListParticipants {
                account_id,
                provider_chat_id,
                cursor,
                limit,
            } => {
                self.list_participants(account_id, provider_chat_id, cursor, *limit)
                    .await
            }
            WhatsAppOperationalQueryV1::ListEvents {
                account_id,
                kind,
                provider_chat_id,
                cursor,
                limit,
            } => {
                self.list_events(
                    account_id,
                    *kind,
                    provider_chat_id.as_deref(),
                    cursor,
                    *limit,
                )
                .await
            }
            WhatsAppOperationalQueryV1::GetRuntimeStatus { account_id } => {
                self.operational_runtime_status(account_id).await
            }
        }
    }

    pub async fn replay_operational_events(
        &self,
        request: &WhatsAppOperationalReplayRequestV1,
    ) -> Result<WhatsAppOperationalReplayResponseV1, WhatsAppDurablePersistenceError> {
        validate_operational_replay_request(request)
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let bounds = sqlx::query(
            "SELECT MIN(sequence) AS earliest_sequence, MAX(sequence) AS latest_sequence FROM makosh_data.whatsapp_operational_events WHERE account_id = $1",
        )
        .bind(&request.account_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let earliest = row_optional_i64(&bounds, "earliest_sequence")?;
        let latest = row_optional_i64(&bounds, "latest_sequence")?;
        let after_sequence = i64::try_from(request.after_sequence)
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let cursor_exists = if request.after_sequence == 0 {
            true
        } else {
            sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM makosh_data.whatsapp_operational_events WHERE account_id = $1 AND sequence = $2)",
            )
            .bind(&request.account_id)
            .bind(after_sequence)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?
        };
        let reset_required = request.after_sequence != 0 && !cursor_exists;
        if reset_required {
            return replay_response(&request.account_id, earliest, latest, Vec::new(), 0, true);
        }
        let rows = sqlx::query(
            "SELECT sequence, exact_event_bytes, event_sha256 FROM makosh_data.whatsapp_operational_events WHERE account_id = $1 AND sequence > $2 ORDER BY sequence ASC LIMIT $3",
        )
        .bind(&request.account_id)
        .bind(after_sequence)
        .bind(i64::from(request.limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut frames = Vec::with_capacity(rows.len());
        for row in rows {
            frames.push(WhatsAppOperationalReplayFrameV1 {
                sequence: u64::try_from(row_i64(&row, "sequence")?)
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                event: event_from_row(&row, &request.account_id)?,
            });
        }
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

    async fn record_operational_resync_control(
        &self,
        host: &WhatsAppHostObservationRecordV1,
        operational: &WhatsAppOperationalObservationV1,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        let WhatsAppOperationalObservationV1::ResyncState {
            provider_event_id,
            account_id,
            observed_at_unix_seconds,
            complete,
        } = operational
        else {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        };
        if provider_event_id.trim().is_empty()
            || account_id.trim().is_empty()
            || provider_event_id != &host.provider_event_id
            || account_id != &host.account_id
            || observed_at_unix_seconds != &host.observed_at_unix_seconds
            || *observed_at_unix_seconds <= 0
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let content_sha256 = operational_digest(host, operational)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.whatsapp_operational_controls (account_id, provider_event_id, control_kind, content_sha256, observed_at_unix_seconds) VALUES ($1, $2, 1, $3, $4) ON CONFLICT (account_id, provider_event_id) DO NOTHING RETURNING account_id",
        )
        .bind(account_id)
        .bind(provider_event_id)
        .bind(content_sha256.as_slice())
        .bind(*observed_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if inserted.is_none() {
            let row = sqlx::query(
                "SELECT content_sha256, observed_at_unix_seconds FROM makosh_data.whatsapp_operational_controls WHERE account_id = $1 AND provider_event_id = $2",
            )
            .bind(account_id)
            .bind(provider_event_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            let stored_sha256: Vec<u8> = row
                .try_get("content_sha256")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let stored_observed_at: i64 = row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            if stored_sha256.as_slice() != content_sha256
                || stored_observed_at != *observed_at_unix_seconds
            {
                return Err(WhatsAppDurablePersistenceError::ObservationConflict);
            }
            transaction
                .commit()
                .await
                .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            return Ok(false);
        }
        let latest_sequence: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sequence), 0) FROM makosh_data.whatsapp_operational_events WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        sqlx::query(
            "INSERT INTO makosh_data.whatsapp_operational_runtime_status (account_id, runtime_state, projection_ready, observed_at_unix_seconds, last_sequence) VALUES ($1, NULL, $2, $3, $4) ON CONFLICT (account_id) DO UPDATE SET projection_ready = EXCLUDED.projection_ready, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, last_sequence = GREATEST(makosh_data.whatsapp_operational_runtime_status.last_sequence, EXCLUDED.last_sequence) WHERE EXCLUDED.observed_at_unix_seconds >= makosh_data.whatsapp_operational_runtime_status.observed_at_unix_seconds",
        )
        .bind(account_id)
        .bind(*complete)
        .bind(*observed_at_unix_seconds)
        .bind(latest_sequence)
        .execute(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(true)
    }

    async fn list_messages(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        search: Option<&str>,
        cursor: &Option<String>,
        limit: u32,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppDurablePersistenceError> {
        let scope = cursor_scope(
            if search.is_some() {
                "message_search"
            } else {
                "messages"
            },
            &[
                account_id,
                provider_chat_id.unwrap_or(""),
                search.unwrap_or(""),
            ],
        );
        let before = decode_cursor(cursor.as_deref(), "messages", &scope)?;
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT account_id, provider_chat_id, provider_message_id, sender_id, sender_display_name, body_text, reply_to_provider_message_id, delivery_state, occurred_at_unix_seconds, last_sequence FROM makosh_data.whatsapp_operational_messages WHERE account_id = ",
        );
        builder.push_bind(account_id);
        if let Some(provider_chat_id) = provider_chat_id {
            builder
                .push(" AND provider_chat_id = ")
                .push_bind(provider_chat_id);
        }
        if let Some(search) = search {
            builder
                .push(" AND body_text ILIKE ")
                .push_bind(search_pattern(search))
                .push(" ESCAPE '\\'");
        }
        builder
            .push(" AND last_sequence < ")
            .push_bind(before)
            .push(" ORDER BY last_sequence DESC LIMIT ")
            .push_bind(i64::from(limit));
        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut items = Vec::with_capacity(rows.len());
        let mut last_sequence = None;
        for row in rows {
            last_sequence = Some(row_i64(&row, "last_sequence")?);
            items.push(WhatsAppMessage {
                account_id: row_string(&row, "account_id")?,
                provider_chat_id: row_string(&row, "provider_chat_id")?,
                provider_message_id: row_string(&row, "provider_message_id")?,
                sender_id: row_string(&row, "sender_id")?,
                sender_display_name: row_string(&row, "sender_display_name")?,
                text: row_optional_string(&row, "body_text")?,
                reply_to_provider_message_id: row_optional_string(
                    &row,
                    "reply_to_provider_message_id",
                )?,
                occurred_at_unix_seconds: row_i64(&row, "occurred_at_unix_seconds")?,
                delivery_state: row_optional_string(&row, "delivery_state")?,
            });
        }
        Ok(WhatsAppOperationalQueryResponseV1::Messages(
            WhatsAppOperationalPageV1 {
                next_cursor: next_cursor(last_sequence, "messages", &scope),
                items,
            },
        ))
    }

    async fn list_dialogs(
        &self,
        account_id: &str,
        cursor: &Option<String>,
        limit: u32,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppDurablePersistenceError> {
        let scope = cursor_scope("dialogs", &[account_id]);
        let before = decode_cursor(cursor.as_deref(), "dialogs", &scope)?;
        let rows = sqlx::query(
            "SELECT account_id, provider_chat_id, title, dialog_kind, is_archived, is_pinned, is_muted, is_unread, unread_count, participant_count, observed_at_unix_seconds, last_sequence FROM makosh_data.whatsapp_operational_dialogs WHERE account_id = $1 AND last_sequence < $2 ORDER BY last_sequence DESC LIMIT $3",
        )
        .bind(account_id)
        .bind(before)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut items = Vec::with_capacity(rows.len());
        let mut last_sequence = None;
        for row in rows {
            last_sequence = Some(row_i64(&row, "last_sequence")?);
            items.push(WhatsAppDialog {
                account_id: row_string(&row, "account_id")?,
                provider_chat_id: row_string(&row, "provider_chat_id")?,
                title: row_string(&row, "title")?,
                kind: row_string(&row, "dialog_kind")?,
                is_archived: row_optional_bool(&row, "is_archived")?,
                is_pinned: row_optional_bool(&row, "is_pinned")?,
                is_muted: row_optional_bool(&row, "is_muted")?,
                is_unread: row_optional_bool(&row, "is_unread")?,
                unread_count: row_optional_u64(&row, "unread_count")?,
                participant_count: row_optional_u64(&row, "participant_count")?,
                observed_at_unix_seconds: row_i64(&row, "observed_at_unix_seconds")?,
            });
        }
        Ok(WhatsAppOperationalQueryResponseV1::Dialogs(
            WhatsAppOperationalPageV1 {
                next_cursor: next_cursor(last_sequence, "dialogs", &scope),
                items,
            },
        ))
    }

    async fn list_participants(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        cursor: &Option<String>,
        limit: u32,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppDurablePersistenceError> {
        let scope = cursor_scope("participants", &[account_id, provider_chat_id]);
        let before = decode_cursor(cursor.as_deref(), "participants", &scope)?;
        let rows = sqlx::query(
            "SELECT account_id, provider_chat_id, provider_identity_id, display_name, participant_role, participant_status, is_self, observed_at_unix_seconds, last_sequence FROM makosh_data.whatsapp_operational_participants WHERE account_id = $1 AND provider_chat_id = $2 AND last_sequence < $3 ORDER BY last_sequence DESC LIMIT $4",
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(before)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut items = Vec::with_capacity(rows.len());
        let mut last_sequence = None;
        for row in rows {
            last_sequence = Some(row_i64(&row, "last_sequence")?);
            items.push(WhatsAppParticipant {
                account_id: row_string(&row, "account_id")?,
                provider_chat_id: row_string(&row, "provider_chat_id")?,
                provider_identity_id: row_string(&row, "provider_identity_id")?,
                display_name: row_string(&row, "display_name")?,
                role: row_string(&row, "participant_role")?,
                status: row_string(&row, "participant_status")?,
                is_self: row_bool(&row, "is_self")?,
                observed_at_unix_seconds: row_i64(&row, "observed_at_unix_seconds")?,
            });
        }
        Ok(WhatsAppOperationalQueryResponseV1::Participants(
            WhatsAppOperationalPageV1 {
                next_cursor: next_cursor(last_sequence, "participants", &scope),
                items,
            },
        ))
    }

    async fn list_events(
        &self,
        account_id: &str,
        kind: Option<makosh_whatsapp_api::WhatsAppProviderEventKind>,
        provider_chat_id: Option<&str>,
        cursor: &Option<String>,
        limit: u32,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppDurablePersistenceError> {
        let kind_scope = kind
            .map(|value| value.storage_code().to_string())
            .unwrap_or_default();
        let scope = cursor_scope(
            "events",
            &[
                account_id,
                kind_scope.as_str(),
                provider_chat_id.unwrap_or(""),
            ],
        );
        let before = decode_cursor(cursor.as_deref(), "events", &scope)?;
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT sequence, exact_event_bytes, event_sha256 FROM makosh_data.whatsapp_operational_events WHERE account_id = ",
        );
        builder.push_bind(account_id);
        if let Some(kind) = kind {
            builder
                .push(" AND event_kind = ")
                .push_bind(kind.storage_code());
        }
        if let Some(provider_chat_id) = provider_chat_id {
            builder
                .push(" AND provider_chat_id = ")
                .push_bind(provider_chat_id);
        }
        builder
            .push(" AND sequence < ")
            .push_bind(before)
            .push(" ORDER BY sequence DESC LIMIT ")
            .push_bind(i64::from(limit));
        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let mut items = Vec::with_capacity(rows.len());
        let mut last_sequence = None;
        for row in rows {
            last_sequence = Some(row_i64(&row, "sequence")?);
            items.push(event_from_row(&row, account_id)?);
        }
        Ok(WhatsAppOperationalQueryResponseV1::Events(
            WhatsAppOperationalPageV1 {
                next_cursor: next_cursor(last_sequence, "events", &scope),
                items,
            },
        ))
    }

    async fn operational_runtime_status(
        &self,
        account_id: &str,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT runtime_state, projection_ready FROM makosh_data.whatsapp_operational_runtime_status WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let latest_event_sequence: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sequence), 0) FROM makosh_data.whatsapp_operational_events WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let (runtime_state, projection_ready) = match row {
            Some(row) => (
                row_optional_string(&row, "runtime_state")?,
                row_bool(&row, "projection_ready")?,
            ),
            None => (None, false),
        };
        Ok(WhatsAppOperationalQueryResponseV1::RuntimeStatus(
            WhatsAppOperationalRuntimeStatusV1 {
                account_id: account_id.to_owned(),
                runtime_state,
                projection_ready,
                latest_event_sequence: u64::try_from(latest_event_sequence)
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
            },
        ))
    }
}

fn replay_response(
    account_id: &str,
    earliest: Option<i64>,
    latest: Option<i64>,
    frames: Vec<WhatsAppOperationalReplayFrameV1>,
    next_sequence: u64,
    reset_required: bool,
) -> Result<WhatsAppOperationalReplayResponseV1, WhatsAppDurablePersistenceError> {
    let response = WhatsAppOperationalReplayResponseV1 {
        account_id: account_id.to_owned(),
        earliest_available_sequence: earliest
            .map(u64::try_from)
            .transpose()
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
        latest_available_sequence: latest
            .map(u64::try_from)
            .transpose()
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
        frames,
        next_sequence,
        reset_required,
    };
    validate_operational_replay_response(&response)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    Ok(response)
}

fn event_from_row(
    row: &sqlx::postgres::PgRow,
    account_id: &str,
) -> Result<WhatsAppProviderEvent, WhatsAppDurablePersistenceError> {
    let exact_event_bytes: Vec<u8> = row
        .try_get("exact_event_bytes")
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    let event_sha256: Vec<u8> = row
        .try_get("event_sha256")
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    if event_sha256.as_slice() != Sha256::digest(&exact_event_bytes).as_slice() {
        return Err(WhatsAppDurablePersistenceError::InvalidRow);
    }
    let event = decode_provider_event(&exact_event_bytes)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    if provider_event_account_id(&event) != account_id {
        return Err(WhatsAppDurablePersistenceError::InvalidRow);
    }
    Ok(event)
}

async fn verify_existing_observation(
    transaction: &mut Transaction<'_, Postgres>,
    observation: &WhatsAppHostObservationRecordV1,
    operational_sha256: Option<&[u8; 32]>,
) -> Result<(), WhatsAppDurablePersistenceError> {
    let row = sqlx::query(
        "SELECT evidence_kind, observed_at_unix_seconds, operational_sha256 FROM makosh_data.whatsapp_host_observations WHERE account_id = $1 AND provider_event_id = $2",
    )
    .bind(&observation.account_id)
    .bind(&observation.provider_event_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
    let evidence_kind: i16 = row
        .try_get("evidence_kind")
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    let observed_at_unix_seconds: i64 = row
        .try_get("observed_at_unix_seconds")
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    let stored_operational_sha256: Option<Vec<u8>> = row
        .try_get("operational_sha256")
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    if evidence_kind != observation.evidence_kind
        || observed_at_unix_seconds != observation.observed_at_unix_seconds
        || stored_operational_sha256.as_deref() != operational_sha256.map(<[u8; 32]>::as_slice)
    {
        return Err(WhatsAppDurablePersistenceError::ObservationConflict);
    }
    Ok(())
}

async fn persist_operational_observation(
    transaction: &mut Transaction<'_, Postgres>,
    host: &WhatsAppHostObservationRecordV1,
    operational: &WhatsAppOperationalObservationV1,
    operational_sha256: &[u8; 32],
) -> Result<(), WhatsAppDurablePersistenceError> {
    match operational {
        WhatsAppOperationalObservationV1::Event {
            provider_event_id,
            event,
        } => {
            let exact_event_bytes = encode_provider_event(event);
            let sequence: i64 = sqlx::query_scalar(
                "INSERT INTO makosh_data.whatsapp_operational_events (account_id, provider_event_id, event_kind, provider_chat_id, exact_event_bytes, event_sha256, observed_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING sequence",
            )
            .bind(&host.account_id)
            .bind(provider_event_id)
            .bind(provider_event_kind(event).storage_code())
            .bind(provider_event_chat_id(event))
            .bind(&exact_event_bytes)
            .bind(operational_sha256.as_slice())
            .bind(host.observed_at_unix_seconds)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
            apply_projection(transaction, event, host.observed_at_unix_seconds, sequence).await
        }
        WhatsAppOperationalObservationV1::ResyncState { .. } => {
            Err(WhatsAppDurablePersistenceError::InvalidRow)
        }
    }
}

async fn apply_projection(
    transaction: &mut Transaction<'_, Postgres>,
    event: &WhatsAppProviderEvent,
    observed_at_unix_seconds: i64,
    sequence: i64,
) -> Result<(), WhatsAppDurablePersistenceError> {
    match event {
        WhatsAppProviderEvent::RuntimeStateChanged {
            account_id, state, ..
        } => {
            sqlx::query(
                "INSERT INTO makosh_data.whatsapp_operational_runtime_status (account_id, runtime_state, projection_ready, observed_at_unix_seconds, last_sequence) VALUES ($1, $2, FALSE, $3, $4) ON CONFLICT (account_id) DO UPDATE SET runtime_state = EXCLUDED.runtime_state, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, last_sequence = EXCLUDED.last_sequence WHERE EXCLUDED.observed_at_unix_seconds >= makosh_data.whatsapp_operational_runtime_status.observed_at_unix_seconds",
            )
            .bind(account_id)
            .bind(runtime_state_name(*state))
            .bind(observed_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::MessageObserved(value) => {
            if projection_is_suppressed_by_tombstone(
                transaction,
                &value.account_id,
                TOMBSTONE_KIND_MESSAGE,
                &value.provider_chat_id,
                &value.provider_message_id,
                observed_at_unix_seconds,
            )
            .await?
            {
                return Ok(());
            }
            sqlx::query(
                "INSERT INTO makosh_data.whatsapp_operational_messages (account_id, provider_chat_id, provider_message_id, sender_id, sender_display_name, body_text, reply_to_provider_message_id, delivery_state, occurred_at_unix_seconds, observed_at_unix_seconds, last_sequence) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT (account_id, provider_chat_id, provider_message_id) DO UPDATE SET sender_id = EXCLUDED.sender_id, sender_display_name = EXCLUDED.sender_display_name, body_text = EXCLUDED.body_text, reply_to_provider_message_id = EXCLUDED.reply_to_provider_message_id, delivery_state = EXCLUDED.delivery_state, occurred_at_unix_seconds = EXCLUDED.occurred_at_unix_seconds, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, last_sequence = EXCLUDED.last_sequence WHERE EXCLUDED.observed_at_unix_seconds >= makosh_data.whatsapp_operational_messages.observed_at_unix_seconds",
            )
            .bind(&value.account_id)
            .bind(&value.provider_chat_id)
            .bind(&value.provider_message_id)
            .bind(&value.sender_id)
            .bind(&value.sender_display_name)
            .bind(&value.text)
            .bind(&value.reply_to_provider_message_id)
            .bind(&value.delivery_state)
            .bind(value.occurred_at_unix_seconds)
            .bind(observed_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
            ..
        } => {
            sqlx::query(
                "UPDATE makosh_data.whatsapp_operational_messages SET body_text = COALESCE($4, body_text), observed_at_unix_seconds = $5, last_sequence = $6 WHERE account_id = $1 AND provider_chat_id = $2 AND provider_message_id = $3 AND observed_at_unix_seconds <= $5",
            )
            .bind(account_id)
            .bind(provider_chat_id)
            .bind(provider_message_id)
            .bind(text)
            .bind(observed_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        } => {
            record_tombstone(
                transaction,
                account_id,
                TOMBSTONE_KIND_MESSAGE,
                provider_chat_id,
                provider_message_id,
                observed_at_unix_seconds,
                sequence,
            )
            .await?;
            sqlx::query(
                "DELETE FROM makosh_data.whatsapp_operational_messages WHERE account_id = $1 AND provider_chat_id = $2 AND provider_message_id = $3 AND observed_at_unix_seconds <= $4",
            )
            .bind(account_id)
            .bind(provider_chat_id)
            .bind(provider_message_id)
            .bind(observed_at_unix_seconds)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::ReceiptChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            delivery_state,
            ..
        } => {
            sqlx::query(
                "UPDATE makosh_data.whatsapp_operational_messages SET delivery_state = $4, observed_at_unix_seconds = $5, last_sequence = $6 WHERE account_id = $1 AND provider_chat_id = $2 AND provider_message_id = $3 AND observed_at_unix_seconds <= $5",
            )
            .bind(account_id)
            .bind(provider_chat_id)
            .bind(provider_message_id)
            .bind(delivery_state)
            .bind(observed_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::DialogObserved(value) => {
            let unread_count = value
                .unread_count
                .map(i64::try_from)
                .transpose()
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            let participant_count = value
                .participant_count
                .map(i64::try_from)
                .transpose()
                .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            sqlx::query(
                "INSERT INTO makosh_data.whatsapp_operational_dialogs (account_id, provider_chat_id, title, dialog_kind, is_archived, is_pinned, is_muted, is_unread, unread_count, participant_count, observed_at_unix_seconds, last_sequence) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT (account_id, provider_chat_id) DO UPDATE SET title = EXCLUDED.title, dialog_kind = EXCLUDED.dialog_kind, is_archived = EXCLUDED.is_archived, is_pinned = EXCLUDED.is_pinned, is_muted = EXCLUDED.is_muted, is_unread = EXCLUDED.is_unread, unread_count = EXCLUDED.unread_count, participant_count = EXCLUDED.participant_count, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, last_sequence = EXCLUDED.last_sequence WHERE EXCLUDED.observed_at_unix_seconds >= makosh_data.whatsapp_operational_dialogs.observed_at_unix_seconds",
            )
            .bind(&value.account_id)
            .bind(&value.provider_chat_id)
            .bind(&value.title)
            .bind(&value.kind)
            .bind(value.is_archived)
            .bind(value.is_pinned)
            .bind(value.is_muted)
            .bind(value.is_unread)
            .bind(unread_count)
            .bind(participant_count)
            .bind(value.observed_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::ParticipantObserved(value) => {
            if projection_is_suppressed_by_tombstone(
                transaction,
                &value.account_id,
                TOMBSTONE_KIND_PARTICIPANT,
                &value.provider_chat_id,
                &value.provider_identity_id,
                observed_at_unix_seconds,
            )
            .await?
            {
                return Ok(());
            }
            sqlx::query(
                "INSERT INTO makosh_data.whatsapp_operational_participants (account_id, provider_chat_id, provider_identity_id, display_name, participant_role, participant_status, is_self, observed_at_unix_seconds, last_sequence) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (account_id, provider_chat_id, provider_identity_id) DO UPDATE SET display_name = EXCLUDED.display_name, participant_role = EXCLUDED.participant_role, participant_status = EXCLUDED.participant_status, is_self = EXCLUDED.is_self, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, last_sequence = EXCLUDED.last_sequence WHERE EXCLUDED.observed_at_unix_seconds >= makosh_data.whatsapp_operational_participants.observed_at_unix_seconds",
            )
            .bind(&value.account_id)
            .bind(&value.provider_chat_id)
            .bind(&value.provider_identity_id)
            .bind(&value.display_name)
            .bind(&value.role)
            .bind(&value.status)
            .bind(value.is_self)
            .bind(value.observed_at_unix_seconds)
            .bind(sequence)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        WhatsAppProviderEvent::ParticipantRemoved {
            account_id,
            provider_chat_id,
            provider_identity_id,
            ..
        } => {
            record_tombstone(
                transaction,
                account_id,
                TOMBSTONE_KIND_PARTICIPANT,
                provider_chat_id,
                provider_identity_id,
                observed_at_unix_seconds,
                sequence,
            )
            .await?;
            sqlx::query(
                "DELETE FROM makosh_data.whatsapp_operational_participants WHERE account_id = $1 AND provider_chat_id = $2 AND provider_identity_id = $3 AND observed_at_unix_seconds <= $4",
            )
            .bind(account_id)
            .bind(provider_chat_id)
            .bind(provider_identity_id)
            .bind(observed_at_unix_seconds)
            .execute(&mut **transaction)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        }
        _ => {}
    }
    Ok(())
}

async fn projection_is_suppressed_by_tombstone(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    entity_kind: i16,
    provider_chat_id: &str,
    provider_entity_id: &str,
    observed_at_unix_seconds: i64,
) -> Result<bool, WhatsAppDurablePersistenceError> {
    let tombstone_observed_at: Option<i64> = sqlx::query_scalar(
        "SELECT observed_at_unix_seconds FROM makosh_data.whatsapp_operational_tombstones WHERE account_id = $1 AND entity_kind = $2 AND provider_chat_id = $3 AND provider_entity_id = $4",
    )
    .bind(account_id)
    .bind(entity_kind)
    .bind(provider_chat_id)
    .bind(provider_entity_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
    let Some(tombstone_observed_at) = tombstone_observed_at else {
        return Ok(false);
    };
    if tombstone_observed_at >= observed_at_unix_seconds {
        return Ok(true);
    }
    sqlx::query(
        "DELETE FROM makosh_data.whatsapp_operational_tombstones WHERE account_id = $1 AND entity_kind = $2 AND provider_chat_id = $3 AND provider_entity_id = $4",
    )
    .bind(account_id)
    .bind(entity_kind)
    .bind(provider_chat_id)
    .bind(provider_entity_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
    Ok(false)
}

async fn record_tombstone(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    entity_kind: i16,
    provider_chat_id: &str,
    provider_entity_id: &str,
    observed_at_unix_seconds: i64,
    sequence: i64,
) -> Result<(), WhatsAppDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.whatsapp_operational_tombstones (account_id, entity_kind, provider_chat_id, provider_entity_id, observed_at_unix_seconds, last_sequence) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (account_id, entity_kind, provider_chat_id, provider_entity_id) DO UPDATE SET observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, last_sequence = EXCLUDED.last_sequence WHERE EXCLUDED.observed_at_unix_seconds >= makosh_data.whatsapp_operational_tombstones.observed_at_unix_seconds",
    )
    .bind(account_id)
    .bind(entity_kind)
    .bind(provider_chat_id)
    .bind(provider_entity_id)
    .bind(observed_at_unix_seconds)
    .bind(sequence)
    .execute(&mut **transaction)
    .await
    .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
    Ok(())
}

fn validate_host_record(
    observation: &WhatsAppHostObservationRecordV1,
) -> Result<(), WhatsAppDurablePersistenceError> {
    if observation.account_id.trim().is_empty()
        || observation.provider_event_id.trim().is_empty()
        || !(1..=11).contains(&observation.evidence_kind)
        || observation.observed_at_unix_seconds <= 0
    {
        return Err(WhatsAppDurablePersistenceError::InvalidRow);
    }
    Ok(())
}

fn operational_digest(
    host: &WhatsAppHostObservationRecordV1,
    operational: &WhatsAppOperationalObservationV1,
) -> Result<[u8; 32], WhatsAppDurablePersistenceError> {
    let exact = match operational {
        WhatsAppOperationalObservationV1::Event {
            provider_event_id,
            event,
        } => {
            validate_event(event).map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
            if provider_event_id != &host.provider_event_id
                || provider_event_account_id(event) != host.account_id
            {
                return Err(WhatsAppDurablePersistenceError::InvalidRow);
            }
            encode_provider_event(event)
        }
        WhatsAppOperationalObservationV1::ResyncState {
            provider_event_id,
            account_id,
            observed_at_unix_seconds,
            complete,
        } => {
            if provider_event_id != &host.provider_event_id
                || account_id != &host.account_id
                || observed_at_unix_seconds != &host.observed_at_unix_seconds
            {
                return Err(WhatsAppDurablePersistenceError::InvalidRow);
            }
            vec![u8::from(*complete)]
        }
    };
    Ok(Sha256::digest(exact).into())
}

fn cursor_scope(label: &str, parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    digest.update(label.as_bytes());
    for part in parts {
        digest.update([0]);
        digest.update(part.as_bytes());
    }
    let bytes = digest.finalize();
    bytes[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_cursor(
    cursor: Option<&str>,
    kind: &str,
    scope: &str,
) -> Result<i64, WhatsAppDurablePersistenceError> {
    let Some(cursor) = cursor else {
        return Ok(i64::MAX);
    };
    let parts = cursor.split('.').collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != "v1" || parts[1] != kind || parts[2] != scope {
        return Err(WhatsAppDurablePersistenceError::InvalidRow);
    }
    let sequence = parts[3]
        .parse::<i64>()
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
    if sequence <= 0 {
        return Err(WhatsAppDurablePersistenceError::InvalidRow);
    }
    Ok(sequence)
}

fn next_cursor(sequence: Option<i64>, kind: &str, scope: &str) -> Option<String> {
    sequence.map(|sequence| format!("v1.{kind}.{scope}.{sequence}"))
}

fn search_pattern(value: &str) -> String {
    format!(
        "%{}%",
        value
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn runtime_state_name(value: makosh_whatsapp_api::WhatsAppRuntimeState) -> &'static str {
    match value {
        makosh_whatsapp_api::WhatsAppRuntimeState::Stopped => "stopped",
        makosh_whatsapp_api::WhatsAppRuntimeState::Starting => "starting",
        makosh_whatsapp_api::WhatsAppRuntimeState::Running => "running",
        makosh_whatsapp_api::WhatsAppRuntimeState::Degraded => "degraded",
        makosh_whatsapp_api::WhatsAppRuntimeState::Blocked => "blocked",
    }
}

fn row_string(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<String, WhatsAppDurablePersistenceError> {
    row.try_get(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

fn row_optional_string(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<String>, WhatsAppDurablePersistenceError> {
    row.try_get(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

fn row_i64(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<i64, WhatsAppDurablePersistenceError> {
    row.try_get(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

fn row_optional_i64(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<i64>, WhatsAppDurablePersistenceError> {
    row.try_get(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

fn row_optional_u64(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<u64>, WhatsAppDurablePersistenceError> {
    row.try_get::<Option<i64>, _>(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?
        .map(u64::try_from)
        .transpose()
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

fn row_bool(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<bool, WhatsAppDurablePersistenceError> {
    row.try_get(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

fn row_optional_bool(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<bool>, WhatsAppDurablePersistenceError> {
    row.try_get(column)
        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_is_scoped_to_exact_query_shape() {
        let scope = cursor_scope("messages", &["account-1", "chat-1", ""]);
        let cursor = next_cursor(Some(42), "messages", &scope).expect("cursor");
        assert_eq!(decode_cursor(Some(&cursor), "messages", &scope), Ok(42));
        assert_eq!(
            decode_cursor(Some(&cursor), "events", &scope),
            Err(WhatsAppDurablePersistenceError::InvalidRow)
        );
        assert_eq!(
            decode_cursor(
                Some(&cursor),
                "messages",
                &cursor_scope("messages", &["account-2", "chat-1", ""])
            ),
            Err(WhatsAppDurablePersistenceError::InvalidRow)
        );
    }

    #[test]
    fn search_pattern_treats_provider_text_as_literal() {
        assert_eq!(search_pattern(r#"50%_done\ok"#), r#"%50\%\_done\\ok%"#);
    }

    #[test]
    fn metadata_only_resync_digest_is_content_bound() {
        let host = WhatsAppHostObservationRecordV1 {
            account_id: "account-1".to_owned(),
            provider_event_id: "resync-1".to_owned(),
            evidence_kind: 1,
            observed_at_unix_seconds: 1_700_000_000,
        };
        let incomplete = WhatsAppOperationalObservationV1::ResyncState {
            provider_event_id: "resync-1".to_owned(),
            account_id: "account-1".to_owned(),
            observed_at_unix_seconds: 1_700_000_000,
            complete: false,
        };
        let complete = WhatsAppOperationalObservationV1::ResyncState {
            provider_event_id: "resync-1".to_owned(),
            account_id: "account-1".to_owned(),
            observed_at_unix_seconds: 1_700_000_000,
            complete: true,
        };
        assert_ne!(
            operational_digest(&host, &incomplete),
            operational_digest(&host, &complete)
        );
    }

    #[test]
    fn query_account_helper_remains_owner_scoped() {
        let query = WhatsAppOperationalQueryV1::GetRuntimeStatus {
            account_id: "account-1".to_owned(),
        };
        assert_eq!(
            makosh_whatsapp_api::operational::operational_query_account_id(&query),
            "account-1"
        );
    }
}
