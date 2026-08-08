//! Typed private Kernel control contracts.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/makosh.gateway.v1.rs"));
}

pub mod owner_control_client;
pub mod owner_control_proof;
pub mod validation;
