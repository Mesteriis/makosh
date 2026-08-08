//! Binds an approved registration to one verified module artifact in this macOS app release.

use std::path::Path;

use makosh_kernel_control_store::{BundledManagedLaunchBinding, OperationIdV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::DistributionArtifactKindV1;
use sha2::{Digest, Sha256};

use crate::distribution::bundle_verifier::VerifiedDistributionBundle;
use crate::distribution::bundled_launch;
use crate::modules::registration::registry::{
    self, BundledArtifactProposal, BundledArtifactUpgrade,
};
use crate::platform::macos::native_launch;

const MACOS_AARCH64_TARGET: &str = "aarch64-apple-darwin";

pub fn propose_current_installed_artifact(
    store: &SqliteControlStore,
    artifact_id: &str,
    expected_distribution_id: &str,
    expected_distribution_generation: u64,
    operation_id: [u8; 16],
) -> Result<BundledArtifactProposal, String> {
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    propose_installed_artifact(
        store,
        artifact_id,
        expected_distribution_id,
        expected_distribution_generation,
        operation_id,
        &kernel_executable,
    )
}

pub fn upgrade_current_installed_artifact(
    store: &SqliteControlStore,
    registration_id: &str,
    artifact_id: &str,
    expected_distribution_id: &str,
    expected_distribution_generation: u64,
) -> Result<BundledArtifactUpgrade, String> {
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    let bundle = native_launch::verify_selected_installed_bundle_artifact_ids(
        &kernel_executable,
        MACOS_AARCH64_TARGET,
        &[artifact_id],
    )?;
    if bundle.manifest().distribution_id != expected_distribution_id
        || bundle.manifest().generation != expected_distribution_generation
    {
        return Err("installed distribution does not match the expected release".to_owned());
    }
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    let descriptor_bytes = artifact
        .module_descriptor_bytes()
        .ok_or_else(|| "managed launch artifact is not a module runtime".to_owned())?;
    registry::upgrade_bundled_artifact(store, registration_id, descriptor_bytes)
}

pub fn read_current_installed_storage_artifact(
    artifact_id: &str,
    expected_distribution_id: &str,
    expected_distribution_generation: u64,
) -> Result<Vec<u8>, String> {
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    let bundle = native_launch::verify_selected_installed_bundle_artifact_ids(
        &kernel_executable,
        MACOS_AARCH64_TARGET,
        &[artifact_id],
    )?;
    if bundle.manifest().distribution_id != expected_distribution_id
        || bundle.manifest().generation != expected_distribution_generation
    {
        return Err("installed distribution does not match the expected release".to_owned());
    }
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .filter(|artifact| artifact.artifact_kind() == DistributionArtifactKindV1::StorageBundle)
        .ok_or_else(|| {
            "managed Storage artifact is absent from distribution manifest".to_owned()
        })?;
    artifact.read_verified_bytes()
}

pub fn propose_installed_artifact(
    store: &SqliteControlStore,
    artifact_id: &str,
    expected_distribution_id: &str,
    expected_distribution_generation: u64,
    operation_id: [u8; 16],
    kernel_executable: &Path,
) -> Result<BundledArtifactProposal, String> {
    let bundle = native_launch::verify_selected_installed_bundle_artifact_ids(
        kernel_executable,
        MACOS_AARCH64_TARGET,
        &[artifact_id],
    )?;
    propose_verified_artifact(
        store,
        artifact_id,
        expected_distribution_id,
        expected_distribution_generation,
        operation_id,
        &bundle,
    )
}

pub fn propose_verified_artifact(
    store: &SqliteControlStore,
    artifact_id: &str,
    expected_distribution_id: &str,
    expected_distribution_generation: u64,
    operation_id: [u8; 16],
    bundle: &VerifiedDistributionBundle,
) -> Result<BundledArtifactProposal, String> {
    if bundle.manifest().distribution_id != expected_distribution_id
        || bundle.manifest().generation != expected_distribution_generation
    {
        return Err("installed distribution does not match the expected release".to_owned());
    }
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    let descriptor_bytes = artifact
        .module_descriptor_bytes()
        .ok_or_else(|| "managed launch artifact is not a module runtime".to_owned())?;
    let descriptor_sha256 = artifact
        .descriptor_sha256()
        .ok_or_else(|| "managed launch artifact is not a module runtime".to_owned())?;
    let request_digest =
        proposal_request_digest(expected_distribution_id, artifact_id, descriptor_sha256);
    registry::propose_bundled_artifact(
        store,
        descriptor_bytes,
        OperationIdV1::new(operation_id),
        request_digest,
        expected_distribution_id,
        expected_distribution_generation,
        artifact_id,
    )
}

pub fn bind_current_installed_release(
    store: &SqliteControlStore,
    registration_id: &str,
    artifact_id: &str,
) -> Result<BundledManagedLaunchBinding, String> {
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    bind_installed_release(store, registration_id, artifact_id, &kernel_executable)
}

pub fn bind_installed_release(
    store: &SqliteControlStore,
    registration_id: &str,
    artifact_id: &str,
    kernel_executable: &Path,
) -> Result<BundledManagedLaunchBinding, String> {
    let bundle = native_launch::verify_selected_installed_bundle_artifact_ids(
        kernel_executable,
        MACOS_AARCH64_TARGET,
        &[artifact_id],
    )?;
    bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    bundled_launch::admit(store, registration_id, &bundle, artifact_id)
}

fn proposal_request_digest(
    distribution_id: &str,
    artifact_id: &str,
    descriptor_sha256: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.bundled-managed-artifact-proposal.v2");
    update_length_prefixed(&mut digest, distribution_id.as_bytes());
    update_length_prefixed(&mut digest, artifact_id.as_bytes());
    digest.update(descriptor_sha256);
    digest.finalize().into()
}

fn update_length_prefixed(digest: &mut Sha256, value: &[u8]) {
    digest.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
mod tests {
    use super::proposal_request_digest;

    #[test]
    fn proposal_identity_is_descriptor_bound() {
        let descriptor = [7_u8; 32];
        assert_ne!(
            proposal_request_digest("distribution", "artifact", &descriptor),
            proposal_request_digest("distribution", "artifact", &[8_u8; 32]),
        );
    }
}
