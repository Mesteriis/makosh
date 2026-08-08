use makosh_communications_api::canonical::{
    CanonicalForwardReferenceRecord, CanonicalMessageReactionRecord, CanonicalMessageReadPort,
    CanonicalMessageReferenceSummaryRecord, CanonicalMessageTombstoneRecord,
    CanonicalMessageVersionRecord, CanonicalReadPortError, CanonicalReplyReferenceRecord,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct CanonicalMessageReadStore {
    pool: PgPool,
}

impl CanonicalMessageReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CanonicalMessageReadPort for CanonicalMessageReadStore {
    async fn list_message_versions(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalMessageVersionRecord>, CanonicalReadPortError> {
        let rows = sqlx::query(r#"SELECT version_id, message_id, account_id, provider_message_id, COALESCE(provider_conversation_id, '') AS provider_chat_id, version_number, body_text, edited_at AS edit_timestamp, source_event, diff_payload AS raw_diff_payload, provenance, created_at FROM communication_message_versions WHERE message_id = $1 ORDER BY version_number ASC, created_at ASC"#)
            .bind(message_id).fetch_all(&self.pool).await.map_err(|error| CanonicalReadPortError::Storage(error.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalMessageVersionRecord {
                    version_id: row
                        .try_get("version_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    message_id: row
                        .try_get("message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    account_id: row
                        .try_get("account_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_message_id: row
                        .try_get("provider_message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_chat_id: row
                        .try_get("provider_chat_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    version_number: row
                        .try_get("version_number")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    body_text: row
                        .try_get("body_text")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    edit_timestamp: row
                        .try_get("edit_timestamp")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    source_event: row
                        .try_get("source_event")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    raw_diff_payload: row
                        .try_get("raw_diff_payload")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provenance: row
                        .try_get("provenance")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn list_message_tombstones(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalMessageTombstoneRecord>, CanonicalReadPortError> {
        let rows = sqlx::query(r#"SELECT tombstone_id, message_id, account_id, provider_message_id, COALESCE(provider_conversation_id, '') AS provider_chat_id, reason_class, actor_class, observed_at, source_event, is_provider_delete, is_local_visible, metadata, provenance, created_at FROM communication_message_tombstones WHERE message_id = $1 ORDER BY observed_at ASC, created_at ASC"#)
            .bind(message_id).fetch_all(&self.pool).await.map_err(|error| CanonicalReadPortError::Storage(error.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalMessageTombstoneRecord {
                    tombstone_id: row
                        .try_get("tombstone_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    message_id: row
                        .try_get("message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    account_id: row
                        .try_get("account_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_message_id: row
                        .try_get("provider_message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_chat_id: row
                        .try_get("provider_chat_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    reason_class: row
                        .try_get("reason_class")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    actor_class: row
                        .try_get("actor_class")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    observed_at: row
                        .try_get("observed_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    source_event: row
                        .try_get("source_event")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    is_provider_delete: row
                        .try_get("is_provider_delete")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    is_local_visible: row
                        .try_get("is_local_visible")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    metadata: row
                        .try_get("metadata")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provenance: row
                        .try_get("provenance")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn list_message_reactions(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalMessageReactionRecord>, CanonicalReadPortError> {
        let rows = sqlx::query(r#"SELECT reaction_id, message_id, account_id, provider_message_id, COALESCE(provider_conversation_id, '') AS provider_chat_id, COALESCE(sender_identity_id, provider_actor_id, reaction_id) AS sender_id, sender_display_name, reaction AS reaction_emoji, is_active, observed_at, source_event, provider_actor_id, metadata, provenance, created_at, updated_at FROM communication_message_reactions WHERE message_id = $1 AND is_active = true ORDER BY observed_at DESC, created_at DESC"#)
            .bind(message_id).fetch_all(&self.pool).await.map_err(|error| CanonicalReadPortError::Storage(error.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalMessageReactionRecord {
                    reaction_id: row
                        .try_get("reaction_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    message_id: row
                        .try_get("message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    account_id: row
                        .try_get("account_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_message_id: row
                        .try_get("provider_message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_chat_id: row
                        .try_get("provider_chat_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    sender_id: row
                        .try_get("sender_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    sender_display_name: row
                        .try_get("sender_display_name")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    reaction_emoji: row
                        .try_get("reaction_emoji")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    is_active: row
                        .try_get("is_active")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    observed_at: row
                        .try_get("observed_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    source_event: row
                        .try_get("source_event")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_actor_id: row
                        .try_get("provider_actor_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    metadata: row
                        .try_get("metadata")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provenance: row
                        .try_get("provenance")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    updated_at: row
                        .try_get("updated_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn list_message_reference_summaries(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<CanonicalMessageReferenceSummaryRecord>, CanonicalReadPortError> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(r#"SELECT message_id, provider_record_id, conversation_id, subject, sender, sender_display_name, body_text, occurred_at FROM communication_messages WHERE message_id = ANY($1)"#)
            .bind(message_ids).fetch_all(&self.pool).await.map_err(|error| CanonicalReadPortError::Storage(error.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalMessageReferenceSummaryRecord {
                    message_id: row
                        .try_get("message_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_message_id: row
                        .try_get("provider_record_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    provider_chat_id: row
                        .try_get("conversation_id")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    chat_title: row
                        .try_get("subject")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    sender: row
                        .try_get("sender")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    sender_display_name: row
                        .try_get("sender_display_name")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    text: row
                        .try_get("body_text")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                    occurred_at: row
                        .try_get("occurred_at")
                        .map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn list_reply_references_by_target(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalReplyReferenceRecord>, CanonicalReadPortError> {
        list_reply_references(&self.pool, "target_message_id", message_id).await
    }

    async fn list_reply_references_by_source(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalReplyReferenceRecord>, CanonicalReadPortError> {
        list_reply_references(&self.pool, "source_message_id", message_id).await
    }

    async fn list_forward_references_by_source(
        &self,
        message_id: &str,
    ) -> Result<Vec<CanonicalForwardReferenceRecord>, CanonicalReadPortError> {
        let rows = sqlx::query(r#"SELECT message_ref_id AS forward_ref_id, source_message_id, target_message_id, account_id, COALESCE(provider_conversation_id, '') AS provider_chat_id, COALESCE(source_provider_id, '') AS source_provider_id, metadata->>'forward_origin_chat_id' AS forward_origin_chat_id, metadata->>'forward_origin_message_id' AS forward_origin_message_id, metadata->>'forward_origin_sender_id' AS forward_origin_sender_id, metadata->>'forward_origin_sender_name' AS forward_origin_sender_name, NULLIF(metadata->>'forwarded_at', '')::timestamptz AS forward_date, depth AS forward_depth, metadata, provenance, created_at FROM communication_message_refs WHERE ref_kind = 'forward' AND source_message_id = $1 ORDER BY created_at DESC"#)
            .bind(message_id).fetch_all(&self.pool).await.map_err(|e| CanonicalReadPortError::Storage(e.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalForwardReferenceRecord {
                    forward_ref_id: row.try_get("forward_ref_id").map_err(storage_error)?,
                    source_message_id: row.try_get("source_message_id").map_err(storage_error)?,
                    target_message_id: row.try_get("target_message_id").map_err(storage_error)?,
                    account_id: row.try_get("account_id").map_err(storage_error)?,
                    provider_chat_id: row.try_get("provider_chat_id").map_err(storage_error)?,
                    source_provider_id: row.try_get("source_provider_id").map_err(storage_error)?,
                    forward_origin_chat_id: row
                        .try_get("forward_origin_chat_id")
                        .map_err(storage_error)?,
                    forward_origin_message_id: row
                        .try_get("forward_origin_message_id")
                        .map_err(storage_error)?,
                    forward_origin_sender_id: row
                        .try_get("forward_origin_sender_id")
                        .map_err(storage_error)?,
                    forward_origin_sender_name: row
                        .try_get("forward_origin_sender_name")
                        .map_err(storage_error)?,
                    forward_date: row.try_get("forward_date").map_err(storage_error)?,
                    forward_depth: row.try_get("forward_depth").map_err(storage_error)?,
                    metadata: row.try_get("metadata").map_err(storage_error)?,
                    provenance: row.try_get("provenance").map_err(storage_error)?,
                    created_at: row.try_get("created_at").map_err(storage_error)?,
                })
            })
            .collect()
    }
}

fn storage_error(error: sqlx::Error) -> CanonicalReadPortError {
    CanonicalReadPortError::Storage(error.to_string())
}

async fn list_reply_references(
    pool: &PgPool,
    direction: &str,
    message_id: &str,
) -> Result<Vec<CanonicalReplyReferenceRecord>, CanonicalReadPortError> {
    let predicate = match direction {
        "source_message_id" | "target_message_id" => direction,
        _ => {
            return Err(CanonicalReadPortError::Storage(
                "invalid reference direction".into(),
            ));
        }
    };
    let query = format!(
        "SELECT message_ref_id AS reply_ref_id, source_message_id, target_message_id, account_id, COALESCE(provider_conversation_id, '') AS provider_chat_id, COALESCE(source_provider_id, '') AS source_provider_id, COALESCE(target_provider_id, '') AS target_provider_id, depth AS reply_depth, COALESCE((metadata->>'is_topic_reply')::boolean, false) AS is_topic_reply, metadata->>'topic_id' AS topic_id, metadata, provenance, created_at FROM communication_message_refs WHERE ref_kind = 'reply' AND {predicate} = $1 ORDER BY created_at DESC"
    );
    let rows = sqlx::query(&query)
        .bind(message_id)
        .fetch_all(pool)
        .await
        .map_err(storage_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(CanonicalReplyReferenceRecord {
                reply_ref_id: row.try_get("reply_ref_id").map_err(storage_error)?,
                source_message_id: row.try_get("source_message_id").map_err(storage_error)?,
                target_message_id: row.try_get("target_message_id").map_err(storage_error)?,
                account_id: row.try_get("account_id").map_err(storage_error)?,
                provider_chat_id: row.try_get("provider_chat_id").map_err(storage_error)?,
                source_provider_id: row.try_get("source_provider_id").map_err(storage_error)?,
                target_provider_id: row.try_get("target_provider_id").map_err(storage_error)?,
                reply_depth: row.try_get("reply_depth").map_err(storage_error)?,
                is_topic_reply: row.try_get("is_topic_reply").map_err(storage_error)?,
                topic_id: row.try_get("topic_id").map_err(storage_error)?,
                metadata: row.try_get("metadata").map_err(storage_error)?,
                provenance: row.try_get("provenance").map_err(storage_error)?,
                created_at: row.try_get("created_at").map_err(storage_error)?,
            })
        })
        .collect()
}
