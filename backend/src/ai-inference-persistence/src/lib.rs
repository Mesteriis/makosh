#![forbid(unsafe_code)]

mod attachment_translation_model;
mod attachment_translation_repository;
mod explanation_model;
mod explanation_repository;
mod model;
mod repository;
pub mod schema;
mod summary_model;
mod summary_repository;
mod translation_model;
mod translation_repository;

pub use attachment_translation_model::{
    AiAttachmentTranslationPersistenceOutcomeV1, AiAttachmentTranslationTransitionV1,
    PersistedAiAttachmentTranslationRunV1,
};
pub use explanation_model::{
    AiExplanationPersistenceOutcomeV1, AiExplanationTransitionV1, PersistedAiExplanationRunV1,
};
pub use model::{
    AI_INFERENCE_RECOVERY_LIMIT_V1, AiInferencePersistenceErrorV1, AiInferencePersistenceOutcomeV1,
    AiInferenceTransitionV1, PersistedAiInferenceRunV1,
};
pub use repository::AiInferencePersistenceV1;
pub use summary_model::{
    AiSummaryPersistenceOutcomeV1, AiSummaryTransitionV1, PersistedAiSummaryRunV1,
};
pub use translation_model::{
    AiTranslationPersistenceOutcomeV1, AiTranslationTransitionV1, PersistedAiTranslationRunV1,
};

pub const PACKAGE: &str = "makosh-ai-inference-persistence";
