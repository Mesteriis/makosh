//! Rebuildable provider-neutral sender insights owned by Communications.

use sqlx::Row;

use crate::{CanonicalReadPageV1, CommunicationsDurablePersistence};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSenderInsightV1 {
    pub sender_id: [u8; 16],
    pub display_label: Option<String>,
    pub message_count: u64,
    pub conversation_count: u64,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationsSenderInsightAfterV1 {
    pub message_count: u64,
    pub last_observed_at_unix_seconds: i64,
    pub sender_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSenderInsightsErrorV1 {
    Invalid,
    AccountNotFound,
    StorageUnavailable,
}

impl CommunicationsDurablePersistence {
    pub async fn list_sender_insights(
        &self,
        account_id: Option<[u8; 16]>,
        after: Option<CommunicationsSenderInsightAfterV1>,
        limit: u16,
    ) -> Result<
        CanonicalReadPageV1<CommunicationsSenderInsightV1>,
        CommunicationsSenderInsightsErrorV1,
    > {
        validate_request(account_id, after, limit)?;
        if let Some(account_id) = account_id {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM makosh_data.communications_accounts \
                 WHERE account_id = $1)",
            )
            .bind(account_id.as_slice())
            .fetch_one(&self.pool)
            .await
            .map_err(|_| unavailable())?;
            if !exists {
                return Err(CommunicationsSenderInsightsErrorV1::AccountNotFound);
            }
        }
        let after_count = after
            .map(|value| i64::try_from(value.message_count).map_err(|_| invalid()))
            .transpose()?;
        let after_last_observed = after.map(|value| value.last_observed_at_unix_seconds);
        let after_sender_id = after.map(|value| value.sender_id.to_vec());
        let rows = sqlx::query(
            "WITH sender_aggregate AS ( \
               SELECT facts.sender_id, profiles.display_label, \
                 COUNT(*)::BIGINT AS message_count, \
                 COUNT(DISTINCT messages.conversation_id)::BIGINT AS conversation_count, \
                 MIN(facts.first_observed_at_unix_seconds)::BIGINT \
                   AS first_observed_at_unix_seconds, \
                 MAX(facts.last_observed_at_unix_seconds)::BIGINT \
                   AS last_observed_at_unix_seconds \
               FROM makosh_data.communications_message_sender_facts facts \
               JOIN makosh_data.communications_sender_profiles profiles \
                 ON profiles.sender_id = facts.sender_id \
               JOIN makosh_data.communications_messages messages \
                 ON messages.message_id = facts.message_id \
               WHERE messages.lifecycle_state = 1 \
                 AND messages.direction = 1 \
                 AND ($1::BYTEA IS NULL OR facts.account_id = $1) \
               GROUP BY facts.sender_id, profiles.display_label \
             ) \
             SELECT sender_id, display_label, message_count, conversation_count, \
               first_observed_at_unix_seconds, last_observed_at_unix_seconds \
             FROM sender_aggregate \
             WHERE ($2::BIGINT IS NULL \
               OR message_count < $2 \
               OR (message_count = $2 AND last_observed_at_unix_seconds < $3) \
               OR (message_count = $2 AND last_observed_at_unix_seconds = $3 \
                 AND sender_id > $4)) \
             ORDER BY message_count DESC, last_observed_at_unix_seconds DESC, \
               sender_id ASC \
             LIMIT $5",
        )
        .bind(account_id.map(|value| value.to_vec()))
        .bind(after_count)
        .bind(after_last_observed)
        .bind(after_sender_id)
        .bind(i64::from(limit) + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| unavailable())?;
        let mut items = rows
            .into_iter()
            .map(sender_insight_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > usize::from(limit);
        items.truncate(usize::from(limit));
        Ok(CanonicalReadPageV1 { items, has_more })
    }
}

fn validate_request(
    account_id: Option<[u8; 16]>,
    after: Option<CommunicationsSenderInsightAfterV1>,
    limit: u16,
) -> Result<(), CommunicationsSenderInsightsErrorV1> {
    if limit == 0
        || limit > 100
        || account_id.is_some_and(|value| value.iter().all(|byte| *byte == 0))
        || after.is_some_and(|value| {
            value.message_count == 0
                || value.sender_id.iter().all(|byte| *byte == 0)
                || !valid_timestamp(value.last_observed_at_unix_seconds)
        })
    {
        return Err(invalid());
    }
    Ok(())
}

fn sender_insight_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationsSenderInsightV1, CommunicationsSenderInsightsErrorV1> {
    let sender_id: Vec<u8> = row.try_get("sender_id").map_err(|_| unavailable())?;
    let display_label: Option<String> = row.try_get("display_label").map_err(|_| unavailable())?;
    let message_count: i64 = row.try_get("message_count").map_err(|_| unavailable())?;
    let conversation_count: i64 = row
        .try_get("conversation_count")
        .map_err(|_| unavailable())?;
    let first_observed_at_unix_seconds: i64 = row
        .try_get("first_observed_at_unix_seconds")
        .map_err(|_| unavailable())?;
    let last_observed_at_unix_seconds: i64 = row
        .try_get("last_observed_at_unix_seconds")
        .map_err(|_| unavailable())?;
    let sender_id: [u8; 16] = sender_id.as_slice().try_into().map_err(|_| unavailable())?;
    let message_count = u64::try_from(message_count).map_err(|_| unavailable())?;
    let conversation_count = u64::try_from(conversation_count).map_err(|_| unavailable())?;
    if sender_id.iter().all(|byte| *byte == 0)
        || message_count == 0
        || conversation_count == 0
        || !valid_timestamp(first_observed_at_unix_seconds)
        || !valid_timestamp(last_observed_at_unix_seconds)
        || first_observed_at_unix_seconds > last_observed_at_unix_seconds
        || display_label.as_ref().is_some_and(|label| {
            label.is_empty() || label.len() > 256 || label.chars().any(char::is_control)
        })
    {
        return Err(unavailable());
    }
    Ok(CommunicationsSenderInsightV1 {
        sender_id,
        display_label,
        message_count,
        conversation_count,
        first_observed_at_unix_seconds,
        last_observed_at_unix_seconds,
    })
}

fn valid_timestamp(value: i64) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&value)
}

const fn invalid() -> CommunicationsSenderInsightsErrorV1 {
    CommunicationsSenderInsightsErrorV1::Invalid
}

const fn unavailable() -> CommunicationsSenderInsightsErrorV1 {
    CommunicationsSenderInsightsErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_validation_rejects_zero_ids_and_unbounded_pages() {
        assert_eq!(
            validate_request(Some([0; 16]), None, 20),
            Err(CommunicationsSenderInsightsErrorV1::Invalid)
        );
        assert_eq!(
            validate_request(None, None, 0),
            Err(CommunicationsSenderInsightsErrorV1::Invalid)
        );
        assert_eq!(
            validate_request(
                None,
                Some(CommunicationsSenderInsightAfterV1 {
                    message_count: 0,
                    last_observed_at_unix_seconds: 1,
                    sender_id: [1; 16],
                }),
                20,
            ),
            Err(CommunicationsSenderInsightsErrorV1::Invalid)
        );
    }
}
