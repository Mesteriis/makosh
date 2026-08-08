#![forbid(unsafe_code)]

mod model;
mod repository;
pub mod schema;

pub use model::{
    PersistedSpeechToTextRequestV1, PersistedSpeechToTextRunV1,
    PersistedSpeechTranscriptArtifactV1, SPEECH_TO_TEXT_RECOVERY_LIMIT_V1,
    SpeechToTextPersistenceErrorV1, SpeechToTextPersistenceOutcomeV1, SpeechToTextTransitionV1,
};
pub use repository::SpeechToTextPersistenceV1;

pub const PACKAGE: &str = "makosh-speech-to-text-persistence";
