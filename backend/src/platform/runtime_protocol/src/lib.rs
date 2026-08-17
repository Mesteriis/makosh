//! Runtime lifecycle and recovery Protobuf contract.

pub mod managed_control;
pub mod managed_runtime_poll;
pub mod platform_control;
pub mod validation;
pub mod vault_request_id;

/// Exact grant which allows one integration registration to receive a bounded
/// catalog of independently revisioned configuration instances.
pub const SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID: &str = "settings.configuration-catalog.v1";

#[allow(clippy::large_enum_variant)]
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/makosh.runtime.v1.rs"));
}
