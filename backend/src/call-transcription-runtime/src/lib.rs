#![forbid(unsafe_code)]
//! Managed workflow ports for call transcription.
//!
//! This unit consumes target-owned recording events, coordinates the public
//! Speech-to-Text request contract, and keeps audio/transcript bytes in Blob.
//! It deliberately has no dependency on Communications, recording, STT, or
//! provider implementations.

pub mod admission;
pub mod blob;
pub mod client_port;
pub mod client_realtime;
pub mod event_consumer;
pub mod ingress;
pub mod managed_runtime;
pub mod recovery;
pub mod stt;

pub const PACKAGE: &str = "makosh-call-transcription-runtime";

pub use admission::{module_descriptor_v1, settings_schema_bytes_v1};
pub use managed_runtime::{
    CallTranscriptionManagedRuntimeErrorV1, CallTranscriptionManagedRuntimeV1,
    CallTranscriptionRuntimeAdmissionV1,
};
