use crate::repository::{
    ReviewAttentionPersistenceErrorV1, ReviewAttentionPersistenceV1, attention_from_row,
    disposition_code, importance_code,
};
use makosh_review_attention_core::{
    ReviewAttentionV1, ReviewDispositionV1, ReviewImportanceV1, STABLE_ID_BYTES_V1,
};

pub const REVIEW_ATTENTION_MAX_PAGE_SIZE_V1: u16 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewAttentionListFilterV1 {
    pub after_attention_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub disposition: Option<ReviewDispositionV1>,
    pub pinned: Option<bool>,
    pub importance: Option<ReviewImportanceV1>,
    pub snoozed: Option<bool>,
    pub limit: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttentionPageV1 {
    pub attention: Vec<ReviewAttentionV1>,
    pub next_cursor: Option<[u8; STABLE_ID_BYTES_V1]>,
}

impl ReviewAttentionPersistenceV1 {
    pub async fn get_attention(
        &self,
        logical_owner_id: &str,
        attention_id: &[u8; STABLE_ID_BYTES_V1],
    ) -> Result<Option<ReviewAttentionV1>, ReviewAttentionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || attention_id.iter().all(|byte| *byte == 0) {
            return Err(ReviewAttentionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT attention_id, source_evidence_id, state_revision, disposition,
                    pinned, importance, snoozed_until_unix_seconds,
                    snoozed_until_nanos, updated_at_unix_seconds, updated_at_nanos
             FROM makosh_data.review_attention_state
             WHERE logical_owner_id = $1 AND attention_id = $2",
        )
        .bind(logical_owner_id)
        .bind(attention_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ReviewAttentionPersistenceErrorV1::StorageUnavailable)?
        .map(|row| attention_from_row(&row))
        .transpose()
    }

    pub async fn list_attention(
        &self,
        logical_owner_id: &str,
        filter: ReviewAttentionListFilterV1,
    ) -> Result<ReviewAttentionPageV1, ReviewAttentionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || filter.limit == 0
            || filter.limit > REVIEW_ATTENTION_MAX_PAGE_SIZE_V1
            || filter
                .after_attention_id
                .is_some_and(|value| value.iter().all(|byte| *byte == 0))
        {
            return Err(ReviewAttentionPersistenceErrorV1::InvalidInput);
        }
        let fetch_limit = i64::from(filter.limit) + 1;
        let rows = sqlx::query(
            "SELECT attention_id, source_evidence_id, state_revision, disposition,
                    pinned, importance, snoozed_until_unix_seconds,
                    snoozed_until_nanos, updated_at_unix_seconds, updated_at_nanos
             FROM makosh_data.review_attention_state
             WHERE logical_owner_id = $1
               AND ($2::BYTEA IS NULL OR attention_id > $2)
               AND ($3::SMALLINT IS NULL OR disposition = $3)
               AND ($4::BOOLEAN IS NULL OR pinned = $4)
               AND ($5::SMALLINT IS NULL OR importance = $5)
               AND (
                 $6::BOOLEAN IS NULL
                 OR ($6 = TRUE AND snoozed_until_unix_seconds IS NOT NULL)
                 OR ($6 = FALSE AND snoozed_until_unix_seconds IS NULL)
               )
             ORDER BY attention_id ASC
             LIMIT $7",
        )
        .bind(logical_owner_id)
        .bind(filter.after_attention_id.map(|value| value.to_vec()))
        .bind(filter.disposition.map(disposition_code))
        .bind(filter.pinned)
        .bind(filter.importance.map(importance_code))
        .bind(filter.snoozed)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ReviewAttentionPersistenceErrorV1::StorageUnavailable)?;
        let mut attention = rows
            .iter()
            .map(attention_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = attention.len() > usize::from(filter.limit);
        if has_more {
            attention.truncate(usize::from(filter.limit));
        }
        let next_cursor = has_more
            .then(|| attention.last().map(|item| item.attention_id))
            .flatten();
        Ok(ReviewAttentionPageV1 {
            attention,
            next_cursor,
        })
    }
}

fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_limit_and_cursor_are_bounded() {
        assert_eq!(REVIEW_ATTENTION_MAX_PAGE_SIZE_V1, 100);
        assert!(!valid_owner("review/provider"));
        assert!(valid_owner("owner-1"));
    }
}
