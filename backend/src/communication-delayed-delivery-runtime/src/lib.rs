#![forbid(unsafe_code)]

mod admission;
mod body_cleanup;
mod client_port;
mod client_realtime;
mod contracts;
mod due_execution;
mod managed_runtime;
pub mod scheduler_outbox;
mod scheduler_results;

pub use admission::{
    COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1,
    communication_delayed_delivery_module_descriptor_v1,
    communication_delayed_delivery_settings_schema_bytes_v1,
    communication_delayed_delivery_settings_schema_v1,
};
pub use client_port::{
    DelayedDeliveryClientContextV1, cancel_delayed_delivery_payload_v1,
    get_delayed_delivery_status_payload_v1, schedule_delayed_delivery_payload_v1,
};
pub use contracts::{
    delayed_delivery_cancel_command_contract_v1, delayed_delivery_query_contract_v1,
    delayed_delivery_realtime_contract_v1, delayed_delivery_schedule_command_contract_v1,
};
pub use managed_runtime::{
    DelayedDeliveryManagedRuntimeErrorV1, DelayedDeliveryManagedRuntimeV1,
    DelayedDeliveryRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-runtime";
pub const COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1: &str =
    "communication.delayed_delivery.blob.v1";
