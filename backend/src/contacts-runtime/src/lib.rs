#![forbid(unsafe_code)]

mod admission;
mod command;
mod event_outbox;
mod managed_runtime;
mod provider_link;
mod source;

pub use admission::{
    CONTACTS_STORAGE_CAPABILITY_ID_V1, contacts_module_descriptor_v1,
    contacts_settings_schema_bytes_v1, contacts_settings_schema_v1,
};
pub use managed_runtime::{
    ContactsManagedRuntimeErrorV1, ContactsManagedRuntimeV1, ContactsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-contacts-runtime";
