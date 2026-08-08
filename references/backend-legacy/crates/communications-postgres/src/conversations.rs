use makosh_communications_api::conversations::{
    CanonicalConversationMemberRecord, CanonicalConversationRecord, CanonicalIdentityRecord,
    ConversationReadError, ConversationReadPort,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct ConversationReadStore {
    pool: PgPool,
}

impl ConversationReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ConversationReadPort for ConversationReadStore {
    async fn list_conversations(
        &self,
        account_id: Option<&str>,
        channel_kinds: &[&str],
        title_query: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalConversationRecord>, ConversationReadError> {
        let rows = sqlx::query(
            r#"SELECT conversation_id, account_id, channel_kind,
                      provider_conversation_id, title, last_message_at, metadata,
                      created_at, updated_at
               FROM communication_conversations
               WHERE channel_kind = ANY($1)
                 AND ($2::text IS NULL OR account_id = $2)
                 AND ($3::text IS NULL OR title ILIKE $3)
               ORDER BY COALESCE(last_message_at, updated_at) DESC, conversation_id ASC
               LIMIT $4"#,
        )
        .bind(channel_kinds)
        .bind(account_id.map(str::trim).filter(|value| !value.is_empty()))
        .bind(title_query)
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter().map(row_to_conversation).collect()
    }

    async fn get_conversation(
        &self,
        conversation_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<CanonicalConversationRecord>, ConversationReadError> {
        let row = sqlx::query(
            r#"SELECT conversation_id, account_id, channel_kind,
                      provider_conversation_id, title, last_message_at, metadata,
                      created_at, updated_at
               FROM communication_conversations
               WHERE (conversation_id = $1 OR provider_conversation_id = $1)
                 AND channel_kind = ANY($2)"#,
        )
        .bind(conversation_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?;
        row.map(row_to_conversation).transpose()
    }

    async fn list_conversation_members(
        &self,
        conversation_id: &str,
        channel_kinds: &[&str],
        query: Option<&str>,
        role: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<CanonicalConversationMemberRecord>, ConversationReadError> {
        let rows = sqlx::query(
            r#"SELECT participant.participant_id, participant.display_name,
                      participant.role, participant.address,
                      participant.metadata AS participant_metadata,
                      identity.provider_identity_id, identity.identity_kind,
                      identity.metadata AS identity_metadata,
                      conversation.last_message_at
               FROM communication_conversation_participants participant
               JOIN communication_conversations conversation
                 ON conversation.conversation_id = participant.conversation_id
               LEFT JOIN communication_identities identity
                 ON identity.identity_id = participant.identity_id
               WHERE participant.conversation_id = $1
                 AND conversation.channel_kind = ANY($2)
                 AND ($3::text IS NULL OR participant.role = $3)
                 AND ($4::text IS NULL OR participant.display_name ILIKE $4
                      OR participant.address ILIKE $4
                      OR identity.provider_identity_id ILIKE $4)
               ORDER BY participant.created_at ASC, participant.participant_id ASC
               OFFSET $5 LIMIT $6"#,
        )
        .bind(conversation_id.trim())
        .bind(channel_kinds)
        .bind(role)
        .bind(query)
        .bind(offset.max(0))
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter().map(row_to_member).collect()
    }

    async fn get_conversation_from_message_projection(
        &self,
        conversation_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<CanonicalConversationRecord>, ConversationReadError> {
        let row = sqlx::query(
            r#"SELECT conversation_id, account_id, channel_kind,
                      MAX(COALESCE(occurred_at, projected_at)) AS last_message_at,
                      MIN(projected_at) AS created_at, MAX(projected_at) AS updated_at
               FROM communication_messages
               WHERE conversation_id = $1 AND channel_kind = ANY($2)
               GROUP BY conversation_id, account_id, channel_kind
               ORDER BY last_message_at DESC NULLS LAST LIMIT 1"#,
        )
        .bind(conversation_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?;
        row.map(|row| {
            let conversation_id: String = row.try_get("conversation_id").map_err(storage_error)?;
            Ok(CanonicalConversationRecord {
                provider_conversation_id: conversation_id.clone(),
                conversation_id,
                account_id: row.try_get("account_id").map_err(storage_error)?,
                channel_kind: row.try_get("channel_kind").map_err(storage_error)?,
                title: String::new(),
                last_message_at: row.try_get("last_message_at").map_err(storage_error)?,
                metadata: serde_json::json!({"chat_kind": "group", "source": "message_projection_fallback"}),
                created_at: row.try_get("created_at").map_err(storage_error)?,
                updated_at: row.try_get("updated_at").map_err(storage_error)?,
            })
        }).transpose()
    }

    async fn list_members_for_provider_conversation(
        &self,
        account_id: &str,
        provider_conversation_id: &str,
        limit: i64,
    ) -> Result<Vec<CanonicalConversationMemberRecord>, ConversationReadError> {
        let rows = sqlx::query(
            r#"SELECT participant.participant_id, conversation.conversation_id,
                      conversation.account_id, conversation.provider_conversation_id,
                      participant.display_name, participant.role, participant.address,
                      participant.metadata AS participant_metadata,
                      identity.provider_identity_id, identity.identity_kind,
                      identity.metadata AS identity_metadata, NULL::timestamptz AS last_message_at
               FROM communication_conversation_participants participant
               JOIN communication_conversations conversation
                 ON conversation.conversation_id = participant.conversation_id
               LEFT JOIN communication_identities identity
                 ON identity.identity_id = participant.identity_id
               WHERE conversation.account_id = $1
                 AND conversation.provider_conversation_id = $2
                 AND conversation.channel_kind = 'whatsapp_web'
               ORDER BY participant.created_at ASC, participant.participant_id ASC
               LIMIT $3"#,
        )
        .bind(account_id.trim())
        .bind(provider_conversation_id.trim())
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalConversationMemberRecord {
                    participant_id: row.try_get("participant_id").map_err(storage_error)?,
                    display_name: row.try_get("display_name").map_err(storage_error)?,
                    role: row.try_get("role").map_err(storage_error)?,
                    address: row.try_get("address").map_err(storage_error)?,
                    participant_metadata: row
                        .try_get("participant_metadata")
                        .map_err(storage_error)?,
                    provider_identity_id: row
                        .try_get("provider_identity_id")
                        .map_err(storage_error)?,
                    identity_kind: row.try_get("identity_kind").map_err(storage_error)?,
                    identity_metadata: row.try_get("identity_metadata").map_err(storage_error)?,
                    last_message_at: row.try_get("last_message_at").map_err(storage_error)?,
                    conversation_id: row.try_get("conversation_id").map_err(storage_error)?,
                    account_id: row.try_get("account_id").map_err(storage_error)?,
                    provider_conversation_id: row
                        .try_get("provider_conversation_id")
                        .map_err(storage_error)?,
                })
            })
            .collect()
    }

    async fn list_presence(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<
        Vec<makosh_communications_api::conversations::CanonicalPresenceRecord>,
        ConversationReadError,
    > {
        let rows = sqlx::query(
            r#"SELECT identity.identity_id, identity.account_id, channel.channel_kind,
            identity.provider_identity_id, identity.identity_kind, identity.display_name,
            identity.address, identity.metadata
            FROM communication_identities identity
            JOIN communication_channels channel ON channel.channel_id = identity.channel_id
            WHERE identity.account_id = $1 AND channel.channel_kind = 'whatsapp_web'
              AND identity.metadata ? 'presence_state'
              AND ($2::text IS NULL OR identity.metadata->>'presence_provider_chat_id' = $2)
            ORDER BY COALESCE(identity.metadata->>'presence_observed_at', ''), identity.identity_id
            LIMIT $3"#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.map(str::trim).filter(|v| !v.is_empty()))
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter()
            .map(|row| {
                Ok(
                    makosh_communications_api::conversations::CanonicalPresenceRecord {
                        identity_id: row.try_get("identity_id").map_err(storage_error)?,
                        account_id: row.try_get("account_id").map_err(storage_error)?,
                        channel_kind: row.try_get("channel_kind").map_err(storage_error)?,
                        provider_identity_id: row
                            .try_get("provider_identity_id")
                            .map_err(storage_error)?,
                        identity_kind: row.try_get("identity_kind").map_err(storage_error)?,
                        display_name: row.try_get("display_name").map_err(storage_error)?,
                        address: row.try_get("address").map_err(storage_error)?,
                        metadata: row.try_get("metadata").map_err(storage_error)?,
                    },
                )
            })
            .collect()
    }

    async fn list_whatsapp_identities(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<CanonicalIdentityRecord>, ConversationReadError> {
        let rows = sqlx::query(
            r#"SELECT identity.identity_id, identity.account_id,
            channel.channel_kind, identity.provider_identity_id, identity.identity_kind,
            identity.display_name, identity.address, identity.metadata
            FROM communication_identities identity
            JOIN communication_channels channel ON channel.channel_id = identity.channel_id
            WHERE identity.account_id = $1 AND channel.channel_kind = 'whatsapp_web'
            ORDER BY identity.updated_at DESC, identity.identity_id ASC LIMIT $2"#,
        )
        .bind(account_id.trim())
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalIdentityRecord {
                    identity_id: row.try_get("identity_id").map_err(storage_error)?,
                    account_id: row.try_get("account_id").map_err(storage_error)?,
                    channel_kind: row.try_get("channel_kind").map_err(storage_error)?,
                    provider_identity_id: row
                        .try_get("provider_identity_id")
                        .map_err(storage_error)?,
                    identity_kind: row.try_get("identity_kind").map_err(storage_error)?,
                    display_name: row.try_get("display_name").map_err(storage_error)?,
                    address: row.try_get("address").map_err(storage_error)?,
                    metadata: row.try_get("metadata").map_err(storage_error)?,
                })
            })
            .collect()
    }
}

fn row_to_conversation(
    row: sqlx::postgres::PgRow,
) -> Result<CanonicalConversationRecord, ConversationReadError> {
    Ok(CanonicalConversationRecord {
        conversation_id: row.try_get("conversation_id").map_err(storage_error)?,
        account_id: row.try_get("account_id").map_err(storage_error)?,
        channel_kind: row.try_get("channel_kind").map_err(storage_error)?,
        provider_conversation_id: row
            .try_get("provider_conversation_id")
            .map_err(storage_error)?,
        title: row.try_get("title").map_err(storage_error)?,
        last_message_at: row.try_get("last_message_at").map_err(storage_error)?,
        metadata: row.try_get("metadata").map_err(storage_error)?,
        created_at: row.try_get("created_at").map_err(storage_error)?,
        updated_at: row.try_get("updated_at").map_err(storage_error)?,
    })
}

fn storage_error(error: sqlx::Error) -> ConversationReadError {
    ConversationReadError::Storage(error.to_string())
}

fn row_to_member(
    row: sqlx::postgres::PgRow,
) -> Result<CanonicalConversationMemberRecord, ConversationReadError> {
    Ok(CanonicalConversationMemberRecord {
        participant_id: row.try_get("participant_id").map_err(storage_error)?,
        display_name: row.try_get("display_name").map_err(storage_error)?,
        role: row.try_get("role").map_err(storage_error)?,
        address: row.try_get("address").map_err(storage_error)?,
        participant_metadata: row.try_get("participant_metadata").map_err(storage_error)?,
        provider_identity_id: row.try_get("provider_identity_id").map_err(storage_error)?,
        identity_kind: row.try_get("identity_kind").map_err(storage_error)?,
        identity_metadata: row.try_get("identity_metadata").map_err(storage_error)?,
        last_message_at: row.try_get("last_message_at").map_err(storage_error)?,
        conversation_id: None,
        account_id: None,
        provider_conversation_id: None,
    })
}
