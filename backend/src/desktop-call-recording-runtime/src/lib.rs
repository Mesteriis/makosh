#![forbid(unsafe_code)]

pub mod admission;
mod blob;
pub mod client_port;
mod client_realtime;
mod host_port;
mod host_transport;
mod managed_runtime;
mod outbox;
pub mod settings;

pub const PACKAGE: &str = "makosh-desktop-call-recording-runtime";

pub use managed_runtime::{
    DesktopRecordingManagedRuntimeErrorV1, DesktopRecordingManagedRuntimeV1,
    DesktopRecordingRuntimeAdmissionV1,
};
