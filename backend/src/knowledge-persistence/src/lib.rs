#![forbid(unsafe_code)]

mod lifecycle_repository;
mod model;
mod repository;
mod row_codec;
mod schema;

pub use model::{
    CompleteReviewedCandidateKnowledgeNoteV1, KnowledgeBlobCleanupV1, KnowledgeBlobReceiptV1,
    KnowledgeLifecycleCommitV1, KnowledgeLifecycleMutationV1, KnowledgeLifecycleOperationOutcomeV1,
    KnowledgeLifecycleOperationV1, KnowledgeOutboxRecordV1, KnowledgePersistenceErrorV1,
    PersistReviewedCandidateMaterializationV1, PersistedReviewedCandidateCommandV1,
    RejectReviewedCandidateKnowledgeNoteV1, ReserveReviewedCandidateCommandOutcomeV1,
    ReserveReviewedCandidateCommandV1,
};
pub use repository::{KnowledgeOutboxPublishClaimV1, KnowledgePersistenceV1};
pub use schema::{
    KNOWLEDGE_SCHEMA_V1, KNOWLEDGE_SCHEMA_V2, KNOWLEDGE_STORAGE_BUNDLE_REVISION_V1,
    KNOWLEDGE_STORAGE_BUNDLE_REVISION_V2, knowledge_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-knowledge-persistence";
