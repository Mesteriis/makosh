#![forbid(unsafe_code)]

mod admission;
mod blob_materialization;
mod client_port;
mod client_realtime;
mod contracts;
mod event_outbox;
mod inference;
mod managed_runtime;
mod recovery;
mod source_results;

pub use admission::{
    ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1,
    ATTACHMENT_TRANSLATION_INFERENCE_CAPABILITY_ID_V1,
    ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1, attachment_translation_module_descriptor_v1,
    attachment_translation_settings_schema_bytes_v1, attachment_translation_settings_schema_v1,
};
pub use blob_materialization::{
    AttachmentTranslationBlobErrorV1, AttachmentTranslationSourceBlobReceiptV1,
};
pub use client_port::{
    AttachmentTranslationClientPortErrorV1, dispatch_attachment_translation_client_request_v1,
};
pub use event_outbox::{
    AttachmentTranslationEventRelayErrorV1, relay_source_prepare_outbox_once_v1,
};
pub use inference::{
    AttachmentTranslationInferenceErrorV1, AttachmentTranslationInferenceExecutionV1,
    complete_attachment_translation_inference_v1,
};
pub use managed_runtime::{
    AttachmentTranslationManagedRuntimeErrorV1, AttachmentTranslationManagedRuntimeV1,
    AttachmentTranslationRuntimeAdmissionV1,
};
pub use recovery::recover_attachment_translation_once_v1;
pub use source_results::{
    AttachmentTranslationSourceResultErrorV1, consume_translation_source_prepared_once_v1,
    consume_translation_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-attachment-translation-runtime";
