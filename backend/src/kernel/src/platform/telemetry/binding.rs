//! Telemetry-specific identity policy over shared signed-release admission.

use std::path::Path;

use makosh_kernel_control_store::PlatformManagedProcessBinding;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::bundle_verifier::VerifiedDistributionBundle;
use crate::platform::managed::release_binding::{self, PlatformReleaseIdentity};

pub const TELEMETRY_PROCESS_ID: &str = "telemetry";

const IDENTITY: PlatformReleaseIdentity = PlatformReleaseIdentity {
    process_id: TELEMETRY_PROCESS_ID,
    artifact_id: "platform.telemetry",
    module_id: "telemetry",
    owner_id: "telemetry",
    target_triple: "aarch64-apple-darwin",
    label: "Telemetry",
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
