#![forbid(unsafe_code)]

pub mod admission;
mod client_port;
mod contracts;
mod managed_runtime;
mod realtime;

pub use admission::{
    REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1, review_attention_module_descriptor_v1,
    review_attention_settings_schema_bytes_v1, review_attention_settings_schema_v1,
};
pub use managed_runtime::{
    ReviewAttentionManagedRuntimeErrorV1, ReviewAttentionManagedRuntimeV1,
    ReviewAttentionRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-review-attention-runtime";
