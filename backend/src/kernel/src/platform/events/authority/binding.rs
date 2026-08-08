//! Release identity for the managed Events credential authority.

use std::path::Path;

use makosh_kernel_control_store::PlatformManagedProcessBinding;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::bundle_verifier::VerifiedDistributionBundle;
use crate::platform::managed::release_binding::{self, PlatformReleaseIdentity};

pub const EVENTS_AUTHORITY_PROCESS_ID: &str = "events_authority";

const IDENTITY: PlatformReleaseIdentity = PlatformReleaseIdentity {
    process_id: EVENTS_AUTHORITY_PROCESS_ID,
    artifact_id: "platform.events-authority",
    module_id: "events",
    owner_id: "events",
    target_triple: "aarch64-apple-darwin",
    label: "Events authority",
};

pub fn bind_current_installed_release(
    store: &SqliteControlStore,
) -> Result<PlatformManagedProcessBinding, String> {
    release_binding::bind_current_installed_release(store, &IDENTITY)
}

pub fn bind_installed_release(
    store: &SqliteControlStore,
    kernel: &Path,
) -> Result<PlatformManagedProcessBinding, String> {
    release_binding::bind_installed_release(store, kernel, &IDENTITY)
}

pub fn admit(
    store: &SqliteControlStore,
    bundle: &VerifiedDistributionBundle,
) -> Result<PlatformManagedProcessBinding, String> {
    release_binding::admit(store, bundle, &IDENTITY)
}
