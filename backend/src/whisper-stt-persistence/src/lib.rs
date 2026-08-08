#![forbid(unsafe_code)]

mod model;
mod repository;
pub mod schema;

pub use model::{
    PersistedWhisperSttRunV1, WhisperSttPersistenceErrorV1, WhisperSttPersistenceOutcomeV1,
    WhisperSttReadyMetadataV1, WhisperSttRunIdentityV1, WhisperSttRunStateV1,
    WhisperSttTransitionV1,
};
pub use repository::WhisperSttPersistenceV1;

pub const PACKAGE: &str = "makosh-whisper-stt-persistence";
