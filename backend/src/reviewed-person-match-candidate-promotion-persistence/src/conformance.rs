use crate::{
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    ReviewedPersonMatchCandidatePromotionPersistenceV1,
    reviewed_person_match_candidate_promotion_storage_bundle_v1,
};
use sqlx::{
    Executor,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::str::FromStr;

pub struct ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionCountsV1 {
    pub requests: i64,
    pub result_inbox: i64,
    pub outbox: i64,
    pub pending_outbox: i64,
}
impl ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1 {
    pub async fn connect_url(
        url: &str,
    ) -> Result<
        ReviewedPersonMatchCandidatePromotionPersistenceV1,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        if url.trim().is_empty() {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(url)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|_| storage())?;
        Ok(ReviewedPersonMatchCandidatePromotionPersistenceV1::new(
            pool,
        ))
    }
    pub async fn install_schema(
        p: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    ) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
        reset_schema(p).await?;
        let step = reviewed_person_match_candidate_promotion_storage_bundle_v1()
            .steps
            .into_iter()
            .next()
            .ok_or(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)?;
        let sql = String::from_utf8(step.forward_sql_utf8)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
            .execute(p.pool())
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }
    pub async fn counts(
        p: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
        owner: &str,
    ) -> Result<
        ReviewedPersonMatchCandidatePromotionCountsV1,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        let mut tx = p.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
            .bind(owner)
            .execute(&mut *tx)
            .await
            .map_err(|_| storage())?;
        let requests=sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.reviewed_person_match_candidate_promotion_requests WHERE logical_owner_id=$1").bind(owner).fetch_one(&mut *tx).await.map_err(|_|storage())?;
        let result_inbox=sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.reviewed_person_match_candidate_promotion_result_inbox WHERE logical_owner_id=$1").bind(owner).fetch_one(&mut *tx).await.map_err(|_|storage())?;
        let outbox=sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.reviewed_person_match_candidate_promotion_outbox WHERE logical_owner_id=$1").bind(owner).fetch_one(&mut *tx).await.map_err(|_|storage())?;
        let pending_outbox=sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.reviewed_person_match_candidate_promotion_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL").bind(owner).fetch_one(&mut *tx).await.map_err(|_|storage())?;
        tx.rollback().await.map_err(|_| storage())?;
        Ok(ReviewedPersonMatchCandidatePromotionCountsV1 {
            requests,
            result_inbox,
            outbox,
            pending_outbox,
        })
    }
}
async fn reset_schema(
    p: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    let expected = std::env::var("MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_DISPOSABLE_DATABASE")
        .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    let sentinel = std::env::var("MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_DISPOSABLE_SENTINEL")
        .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    if !expected.starts_with("makosh_reviewed_pm_conf_")
        || sentinel.len() != 64
        || !sentinel.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    let current: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(p.pool())
        .await
        .map_err(|_| storage())?;
    let actual:String=sqlx::query_scalar("SELECT token FROM public.makosh_reviewed_person_match_promotion_disposable_sentinel WHERE sentinel_id=1").fetch_one(p.pool()).await.map_err(|_|storage())?;
    if current != expected || actual != sentinel {
        return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    p.pool()
        .execute("DROP SCHEMA IF EXISTS makosh_data CASCADE")
        .await
        .map_err(|_| storage())?;
    p.pool()
        .execute("CREATE SCHEMA makosh_data")
        .await
        .map(|_| ())
        .map_err(|_| storage())
}
const fn storage() -> ReviewedPersonMatchCandidatePromotionPersistenceErrorV1 {
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::StorageUnavailable
}
