#![forbid(unsafe_code)]
//! Managed workflow composition for bounded attachment text extraction.

mod admission;
mod blob;
mod client_port;
mod client_realtime;
mod contracts;
mod event_decode;
mod ocr_resources;
mod outbox;
mod parser;
pub mod runtime;
mod translation_source;

pub use admission::{
    ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
    attachment_text_extraction_module_descriptor_v1,
    attachment_text_extraction_settings_schema_bytes_v1,
    attachment_text_extraction_settings_schema_v1,
};
pub use ocr_resources::{
    ATTACHMENT_TEXT_EXTRACTION_OCR_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
    AttachmentTextExtractionOcrResourcesErrorV1, PreparedAttachmentTextExtractionOcrResourcesV1,
    prepare_attachment_text_extraction_ocr_resources_v1,
};
pub use parser::{
    AttachmentTextExtractionParserRuntimeV1, AttachmentTextRuntimeParseErrorV1,
    AttachmentTextRuntimeParseResultV1,
};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-runtime";
