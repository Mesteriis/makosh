//! Explicit hooks for live workflow-persistence conformance on disposable PostgreSQL.

use sqlx::PgPool;

use crate::ReviewedObligationCandidatePromotionPersistenceV1;

pub struct ReviewedObligationCandidatePromotionPersistenceConformanceV1;

impl ReviewedObligationCandidatePromotionPersistenceConformanceV1 {
    #[must_use]
    pub fn from_disposable_pool(pool: PgPool) -> ReviewedObligationCandidatePromotionPersistenceV1 {
        ReviewedObligationCandidatePromotionPersistenceV1 { pool }
    }
}
