#![forbid(unsafe_code)]

mod admission;
mod client;
pub mod command;
mod consumer;
mod event_outbox;
mod execution;
mod managed_runtime;
pub mod transport;

pub use admission::{
    PERSONS_STORAGE_CAPABILITY_ID_V1, persons_module_descriptor_v1,
    persons_settings_schema_bytes_v1, persons_settings_schema_v1,
};
pub use client::dispatch_persons_client_request_v1;
pub use consumer::{PersonsCommandConsumerErrorV1, consume_persons_command_once_v1};
pub use event_outbox::{PersonsEventRelayErrorV1, relay_persons_outbox_once_v1};
pub use execution::{
    PersonsCommandExecutionErrorV1, PersonsCommandRuntimeContextV1,
    execute_persons_command_record_v1,
};
pub use managed_runtime::{
    PersonsManagedRuntimeErrorV1, PersonsManagedRuntimeV1, PersonsRuntimeAdmissionV1,
};
pub use transport::{PersonsEnvelopeContextV1, build_persons_command_outbox_record_v1};

pub const PACKAGE: &str = "makosh-persons-runtime";
