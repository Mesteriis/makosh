#![forbid(unsafe_code)]

mod query;
mod realtime;
mod repository;
pub mod schema;

pub use query::{ReviewAttentionListFilterV1, ReviewAttentionPageV1};
pub use realtime::{
    REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1, ReviewAttentionRealtimeTransitionV1,
};
pub use repository::{
    ApplyReviewAttentionOperationV1, ReviewAttentionPersistenceErrorV1,
    ReviewAttentionPersistenceOutcomeV1, ReviewAttentionPersistenceV1,
};

pub const PACKAGE: &str = "makosh-review-attention-persistence";
