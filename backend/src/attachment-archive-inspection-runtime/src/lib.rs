#![forbid(unsafe_code)]

pub mod admission;
mod blob;
mod client_port;
mod client_realtime;
mod contracts;
mod event_decode;
mod outbox;
pub mod runtime;
pub mod settings;

pub const PACKAGE: &str = "makosh-attachment-archive-inspection-runtime";
