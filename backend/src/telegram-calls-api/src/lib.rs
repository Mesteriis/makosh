pub mod contract;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.telegram.calls.v1.rs"));
}

pub use contract::*;

pub const PACKAGE: &str = "makosh-telegram-calls-api";
