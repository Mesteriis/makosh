use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgRow;
use sqlx::postgres::Postgres;

use super::errors::ConsistencyError;
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;

pub(super) struct ActivePersonaFactClaim {
    pub(super) fact_id: String,
    pub(super) persona_id: String,
    pub(super) claim_type: String,
    pub(super) value: String,
    pub(super) confidence: f64,
    pub(super) email_address: String,
}

pub(super) fn row_to_active_persona_fact_claim(
    row: PgRow,
) -> Result<ActivePersonaFactClaim, ConsistencyError> {
    Ok(ActivePersonaFactClaim {
        fact_id: row.try_get("fact_id")?,
        persona_id: row.try_get("persona_id")?,
        claim_type: row.try_get("fact_type")?,
        value: row.try_get("value")?,
        confidence: row.try_get("confidence")?,
        email_address: normalize_email_address_for_match(
            row.try_get::<String, _>("email_address")?.as_str(),
        ),
    })
}

pub(super) struct MessageEvidence {
    pub(super) message_id: String,
    pub(super) sender_email_address: String,
    pub(super) text: String,
}

pub(super) fn row_to_message_evidence(row: PgRow) -> Result<MessageEvidence, ConsistencyError> {
    let subject = row.try_get::<String, _>("subject")?;
    let body_text = row.try_get::<String, _>("body_text")?;

    Ok(MessageEvidence {
        message_id: row.try_get("message_id")?,
        sender_email_address: normalize_email_address_for_match(
            row.try_get::<String, _>("sender")?.as_str(),
        ),
        text: format!("{subject}\n{body_text}"),
    })
}

pub(super) struct ChannelMessageEvidence {
    pub(super) message_id: String,
    pub(super) persona_id: String,
    pub(super) text: String,
}

pub(super) fn row_to_channel_message_evidence(
    row: PgRow,
) -> Result<ChannelMessageEvidence, ConsistencyError> {
    let subject = row.try_get::<String, _>("subject")?;
    let body_text = row.try_get::<String, _>("body_text")?;

    Ok(ChannelMessageEvidence {
        message_id: row.try_get("message_id")?,
        persona_id: row.try_get("persona_id")?,
        text: format!("{subject}\n{body_text}"),
    })
}

pub(super) struct DocumentEvidence {
    pub(super) document_id: String,
    pub(super) observation_id: Option<String>,
    pub(super) normalized_text: String,
    pub(super) text: String,
}

impl DocumentEvidence {
    pub(super) fn references_email_address(&self, email_address: &str) -> bool {
        self.normalized_text.contains(email_address)
    }
}

pub(super) fn row_to_document_evidence(row: PgRow) -> Result<DocumentEvidence, ConsistencyError> {
    let title = row.try_get::<String, _>("title")?;
    let extracted_text = row.try_get::<String, _>("extracted_text")?;
    let text = format!("{title}\n{extracted_text}");

    Ok(DocumentEvidence {
        document_id: row.try_get("document_id")?,
        observation_id: row.try_get("observation_id")?,
        normalized_text: text.to_ascii_lowercase(),
        text,
    })
}

pub(super) struct MeetingNoteEvidence {
    pub(super) note_id: String,
    pub(super) persona_id: String,
    pub(super) text: String,
}

pub(super) fn row_to_meeting_note_evidence(
    row: PgRow,
) -> Result<MeetingNoteEvidence, ConsistencyError> {
    let title = row.try_get::<String, _>("title")?;
    let content = row.try_get::<String, _>("content")?;

    Ok(MeetingNoteEvidence {
        note_id: row.try_get("note_id")?,
        persona_id: row.try_get("persona_id")?,
        text: format!("{title}\n{content}"),
    })
}

pub(super) struct CallTranscriptEvidence {
    pub(super) transcript_id: String,
    pub(super) persona_id: String,
    pub(super) text: String,
}

pub(super) fn row_to_call_transcript_evidence(
    row: PgRow,
) -> Result<CallTranscriptEvidence, ConsistencyError> {
    Ok(CallTranscriptEvidence {
        transcript_id: row.try_get("transcript_id")?,
        persona_id: row.try_get("persona_id")?,
        text: row.try_get("transcript_text")?,
    })
}

fn normalize_email_address_for_match(email_address: &str) -> String {
    email_addr_spec(email_address).trim().to_ascii_lowercase()
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<')
        && let Some((addr, _)) = tail.split_once('>')
    {
        return addr.trim();
    }
    value.trim_matches('"')
}

pub(crate) async fn link_consistency_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: serde_json::Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "consistency",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
