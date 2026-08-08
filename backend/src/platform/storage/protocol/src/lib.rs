//! Vendor-neutral Storage Control contracts.

mod model;

pub mod validation;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/makosh.storage.v1.rs"));
}

pub use model::{
    StorageAccessProfileV1, StorageBindingAccessV1, StorageBindingErrorV1, StorageBindingFencesV1,
    StorageBindingIdentityV1, StorageBindingV1, StorageEffectiveBudgetsV1,
    StorageNamespaceRequestV1, StorageRequestErrorV1, storage_runtime_pool_alias,
};
