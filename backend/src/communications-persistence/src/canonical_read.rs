//! Keyset-paged metadata reads owned by the Communications domain.

use hermes_communications_api::{
    CommunicationAccountSummaryV1, CommunicationAttachmentAnchorSummaryV1,
    CommunicationConversationIdV1, CommunicationConversationSummaryV1, CommunicationMessageIdV1,
    CommunicationMessageReferenceSummaryV1, CommunicationMessageSummaryV1,
    CommunicationObservationIdV1, CommunicationObservedParticipantSummaryV1,
    CommunicationSourceCursorV1,
};
use sqlx::Row;

use crate::{
    CommunicationsDurablePersistence, CommunicationsPersistenceError,
    durable::{
        account_from_row, anchor_from_row, conversation_from_row, id16, message_from_row,
        participant_from_row, reference_from_row,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalReadAfterV1 {
    pub observed_at_unix_seconds: i64,
    pub canonical_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalReferenceReadAfterV1 {
    pub observed_at_unix_seconds: i64,
    pub reference_kind: i16,
    pub reference_id: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalReferenceReadItemV1 {
    pub summary: CommunicationMessageReferenceSummaryV1,
    pub reference_id: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalReadPageV1<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

impl CommunicationsDurablePersistence {
    pub async fn canonical_message_id_for_evidence(
        &self,
        evidence_id: CommunicationObservationIdV1,
    ) -> Result<Option<CommunicationMessageIdV1>, CommunicationsPersistenceError> {
        let row = sqlx::query(
            "SELECT message.message_id \
             FROM hermes_data.communications_evidence_summaries evidence \
             JOIN hermes_data.communications_messages message \
               ON message.source_cursor_sha256 = evidence.source_cursor_sha256 \
             WHERE evidence.observation_id = $1",
        )
        .bind(evidence_id.bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        row.map(|row| {
            let value = row
                .try_get::<Vec<u8>, _>("message_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            Ok(CommunicationMessageIdV1::new(id16(&value)?))
        })
        .transpose()
    }

    pub async fn canonical_message(
        &self,
        message_id: CommunicationMessageIdV1,
    ) -> Result<Option<CommunicationMessageSummaryV1>, CommunicationsPersistenceError> {
        let row = sqlx::query(
            "SELECT message_id, conversation_id, source_cursor_sha256, \
             canonical_body_state AS body_state, direction, lifecycle_state, \
             first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id \
             FROM hermes_data.communications_messages WHERE message_id = $1",
        )
        .bind(message_id.bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        row.map(message_from_row).transpose()
    }

    pub async fn canonical_accounts_page(
        &self,
        after: Option<CanonicalReadAfterV1>,
        limit: u16,
    ) -> Result<CanonicalReadPageV1<CommunicationAccountSummaryV1>, CommunicationsPersistenceError>
    {
        let (after_observed_at, after_id) = descending_after(after);
        let rows = sqlx::query(
            "SELECT account_id, account_cursor_sha256, provider, \
             first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id \
             FROM hermes_data.communications_accounts \
             WHERE ($1::BIGINT IS NULL \
                OR last_observed_at_unix_seconds < $1 \
                OR (last_observed_at_unix_seconds = $1 AND account_id > $2)) \
             ORDER BY last_observed_at_unix_seconds DESC, account_id ASC LIMIT $3",
        )
        .bind(after_observed_at)
        .bind(after_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        finish_page(
            rows.into_iter()
                .map(account_from_row)
                .collect::<Result<_, _>>()?,
            limit,
        )
    }

    pub async fn canonical_conversations_page(
        &self,
        account_cursor: Option<CommunicationSourceCursorV1>,
        after: Option<CanonicalReadAfterV1>,
        limit: u16,
    ) -> Result<
        CanonicalReadPageV1<CommunicationConversationSummaryV1>,
        CommunicationsPersistenceError,
    > {
        let (after_observed_at, after_id) = descending_after(after);
        let rows = sqlx::query(
            "SELECT conversation_id, account_cursor_sha256, conversation_cursor_sha256, provider, \
             first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id \
             FROM hermes_data.communications_conversations \
             WHERE ($1::bytea IS NULL OR account_cursor_sha256 = $1) \
               AND ($2::BIGINT IS NULL \
                 OR last_observed_at_unix_seconds < $2 \
                 OR (last_observed_at_unix_seconds = $2 AND conversation_id > $3)) \
             ORDER BY last_observed_at_unix_seconds DESC, conversation_id ASC LIMIT $4",
        )
        .bind(account_cursor.map(|value| value.bytes().to_vec()))
        .bind(after_observed_at)
        .bind(after_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        finish_page(
            rows.into_iter()
                .map(conversation_from_row)
                .collect::<Result<_, _>>()?,
            limit,
        )
    }

    pub async fn canonical_messages_page(
        &self,
        conversation_id: CommunicationConversationIdV1,
        after: Option<CanonicalReadAfterV1>,
        limit: u16,
    ) -> Result<CanonicalReadPageV1<CommunicationMessageSummaryV1>, CommunicationsPersistenceError>
    {
        let (after_observed_at, after_id) = descending_after(after);
        let rows = sqlx::query(
            "SELECT message_id, conversation_id, source_cursor_sha256, \
             canonical_body_state AS body_state, direction, lifecycle_state, \
             first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id \
             FROM hermes_data.communications_messages \
             WHERE conversation_id = $1 \
               AND ($2::BIGINT IS NULL \
                 OR last_observed_at_unix_seconds < $2 \
                 OR (last_observed_at_unix_seconds = $2 AND message_id > $3)) \
             ORDER BY last_observed_at_unix_seconds DESC, message_id ASC LIMIT $4",
        )
        .bind(conversation_id.bytes().as_slice())
        .bind(after_observed_at)
        .bind(after_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        finish_page(
            rows.into_iter()
                .map(message_from_row)
                .collect::<Result<_, _>>()?,
            limit,
        )
    }

    pub async fn canonical_participants_page(
        &self,
        conversation_id: CommunicationConversationIdV1,
        after: Option<CanonicalReadAfterV1>,
        limit: u16,
    ) -> Result<
        CanonicalReadPageV1<CommunicationObservedParticipantSummaryV1>,
        CommunicationsPersistenceError,
    > {
        let (after_observed_at, after_id) = descending_after(after);
        let rows = sqlx::query(
            "SELECT participant_id, conversation_id, participant_cursor_sha256, display_label, \
             first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id \
             FROM hermes_data.communications_observed_participants \
             WHERE conversation_id = $1 \
               AND ($2::BIGINT IS NULL \
                 OR last_observed_at_unix_seconds < $2 \
                 OR (last_observed_at_unix_seconds = $2 AND participant_id > $3)) \
             ORDER BY last_observed_at_unix_seconds DESC, participant_id ASC LIMIT $4",
        )
        .bind(conversation_id.bytes().as_slice())
        .bind(after_observed_at)
        .bind(after_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        finish_page(
            rows.into_iter()
                .map(participant_from_row)
                .collect::<Result<_, _>>()?,
            limit,
        )
    }

    pub async fn canonical_attachment_anchors_page(
        &self,
        message_id: CommunicationMessageIdV1,
        after: Option<CanonicalReadAfterV1>,
        limit: u16,
    ) -> Result<
        CanonicalReadPageV1<CommunicationAttachmentAnchorSummaryV1>,
        CommunicationsPersistenceError,
    > {
        let (after_observed_at, after_id) = descending_after(after);
        let rows = sqlx::query(
            "SELECT attachment_anchor_id, message_id, media_cursor_sha256, anchor_state, \
             attachment_filename, attachment_media_type, attachment_declared_bytes, \
             attachment_sha256, attachment_disposition, first_observed_at_unix_seconds, \
             last_observed_at_unix_seconds, last_evidence_id \
             FROM hermes_data.communications_attachment_anchors \
             WHERE message_id = $1 \
               AND ($2::BIGINT IS NULL \
                 OR last_observed_at_unix_seconds < $2 \
                 OR (last_observed_at_unix_seconds = $2 AND attachment_anchor_id > $3)) \
             ORDER BY last_observed_at_unix_seconds DESC, attachment_anchor_id ASC LIMIT $4",
        )
        .bind(message_id.bytes().as_slice())
        .bind(after_observed_at)
        .bind(after_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        finish_page(
            rows.into_iter()
                .map(anchor_from_row)
                .collect::<Result<_, _>>()?,
            limit,
        )
    }

    pub async fn canonical_references_page(
        &self,
        message_id: CommunicationMessageIdV1,
        after: Option<CanonicalReferenceReadAfterV1>,
        limit: u16,
    ) -> Result<CanonicalReadPageV1<CanonicalReferenceReadItemV1>, CommunicationsPersistenceError>
    {
        let (after_observed_at, after_kind, after_reference_id) = after
            .map(|value| {
                (
                    Some(value.observed_at_unix_seconds),
                    Some(value.reference_kind),
                    Some(value.reference_id.to_vec()),
                )
            })
            .unwrap_or((None, None, None));
        let rows = sqlx::query(
            "SELECT reference.reference_id, reference.source_message_id, reference.reference_kind, \
             reference.target_source_cursor_sha256, target.message_id AS target_message_id, \
             reference.observed_at_unix_seconds, reference.evidence_id \
             FROM hermes_data.communications_message_references reference \
             LEFT JOIN hermes_data.communications_messages target \
               ON target.source_cursor_sha256 = reference.target_source_cursor_sha256 \
             WHERE reference.source_message_id = $1 \
               AND ($2::BIGINT IS NULL \
                 OR reference.observed_at_unix_seconds > $2 \
                 OR (reference.observed_at_unix_seconds = $2 \
                   AND (reference.reference_kind > $3 \
                     OR (reference.reference_kind = $3 AND reference.reference_id > $4)))) \
             ORDER BY reference.observed_at_unix_seconds ASC, \
               reference.reference_kind ASC, reference.reference_id ASC LIMIT $5",
        )
        .bind(message_id.bytes().as_slice())
        .bind(after_observed_at)
        .bind(after_kind)
        .bind(after_reference_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        finish_page(
            rows.into_iter()
                .map(reference_read_item_from_row)
                .collect::<Result<_, _>>()?,
            limit,
        )
    }

    pub async fn canonical_message_evidence_page(
        &self,
        message_id: CommunicationMessageIdV1,
        after: Option<CanonicalReadAfterV1>,
        limit: u16,
    ) -> Result<CanonicalReadPageV1<CommunicationObservationIdV1>, CommunicationsPersistenceError>
    {
        let (after_observed_at, after_id) = descending_after(after);
        let rows = sqlx::query(
            "SELECT summary.observation_id \
             FROM hermes_data.communications_evidence_summaries summary \
             INNER JOIN hermes_data.communications_messages message \
               ON message.source_cursor_sha256 = summary.source_cursor_sha256 \
             WHERE message.message_id = $1 \
               AND ($2::BIGINT IS NULL \
                 OR summary.observed_at_unix_seconds < $2 \
                 OR (summary.observed_at_unix_seconds = $2 \
                   AND summary.observation_id > $3)) \
             ORDER BY summary.observed_at_unix_seconds DESC, summary.observation_id ASC LIMIT $4",
        )
        .bind(message_id.bytes().as_slice())
        .bind(after_observed_at)
        .bind(after_id)
        .bind(page_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let items = rows
            .into_iter()
            .map(|row| {
                let value: Vec<u8> = row
                    .try_get("observation_id")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
                id16(&value).map(CommunicationObservationIdV1::new)
            })
            .collect::<Result<Vec<_>, _>>()?;
        finish_page(items, limit)
    }
}

fn reference_read_item_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CanonicalReferenceReadItemV1, CommunicationsPersistenceError> {
    let reference_id: Vec<u8> = row
        .try_get("reference_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(CanonicalReferenceReadItemV1 {
        summary: reference_from_row(row)?,
        reference_id: reference_id
            .try_into()
            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
    })
}

fn descending_after(after: Option<CanonicalReadAfterV1>) -> (Option<i64>, Option<Vec<u8>>) {
    after
        .map(|value| {
            (
                Some(value.observed_at_unix_seconds),
                Some(value.canonical_id.to_vec()),
            )
        })
        .unwrap_or((None, None))
}

fn page_query_limit(limit: u16) -> Result<i64, CommunicationsPersistenceError> {
    if !(1..=100).contains(&limit) {
        return Err(CommunicationsPersistenceError::InvalidRow);
    }
    Ok(i64::from(limit) + 1)
}

fn finish_page<T>(
    mut items: Vec<T>,
    limit: u16,
) -> Result<CanonicalReadPageV1<T>, CommunicationsPersistenceError> {
    if !(1..=100).contains(&limit) {
        return Err(CommunicationsPersistenceError::InvalidRow);
    }
    let has_more = items.len() > usize::from(limit);
    items.truncate(usize::from(limit));
    Ok(CanonicalReadPageV1 { items, has_more })
}

#[cfg(test)]
mod tests {
    use super::{CanonicalReadAfterV1, descending_after, page_query_limit};

    #[test]
    fn keyset_page_limit_is_bounded_before_sql() {
        assert_eq!(page_query_limit(1), Ok(2));
        assert_eq!(page_query_limit(100), Ok(101));
        assert!(page_query_limit(0).is_err());
        assert!(page_query_limit(101).is_err());
    }

    #[test]
    fn descending_anchor_preserves_exact_canonical_id() {
        let id = [7; 16];
        assert_eq!(
            descending_after(Some(CanonicalReadAfterV1 {
                observed_at_unix_seconds: 42,
                canonical_id: id,
            })),
            (Some(42), Some(id.to_vec())),
        );
    }
}
