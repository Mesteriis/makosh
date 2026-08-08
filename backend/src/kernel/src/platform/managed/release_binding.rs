//! Admits one exact platform process from a verified signed release bundle.

use std::path::Path;

use makosh_kernel_control_store::PlatformManagedProcessBinding;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::ModuleKindV1;

use crate::distribution::bundle_verifier::{
    VerifiedDistributionArtifact, VerifiedDistributionBundle,
};
use crate::platform::macos::native_launch;

pub struct PlatformReleaseIdentity {
    pub process_id: &'static str,
    pub artifact_id: &'static str,
    pub module_id: &'static str,
    pub owner_id: &'static str,
    pub target_triple: &'static str,
    pub label: &'static str,
}

pub fn bind_current_installed_release(
    store: &SqliteControlStore,
    identity: &PlatformReleaseIdentity,
) -> Result<PlatformManagedProcessBinding, String> {
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    bind_installed_release(store, &kernel, identity)
}

pub fn bind_installed_release(
    store: &SqliteControlStore,
    kernel: &Path,
    identity: &PlatformReleaseIdentity,
) -> Result<PlatformManagedProcessBinding, String> {
    let bundle = native_launch::verify_selected_installed_bundle_artifact_ids(
        kernel,
        identity.target_triple,
        &[identity.artifact_id],
    )?;
    admit(store, &bundle, identity)
}

pub fn admit(
    store: &SqliteControlStore,
    bundle: &VerifiedDistributionBundle,
    identity: &PlatformReleaseIdentity,
) -> Result<PlatformManagedProcessBinding, String> {
    let artifact = designated_artifact(bundle, identity)?;
    let binding = PlatformManagedProcessBinding::new(
        identity.process_id,
        next_binding_revision(store, identity)?,
        &bundle.manifest().distribution_id,
        artifact.artifact_id(),
        *artifact.expected_sha256(),
        *artifact.descriptor_sha256().ok_or_else(|| {
            format!(
                "{} release artifact lacks a module descriptor",
                identity.label
            )
        })?,
        artifact.settings_schema_sha256().copied(),
    );
    store
        .record_platform_managed_process_binding(&binding)
        .map_err(|error| format!("{error:?}"))?;
    Ok(binding)
}

fn designated_artifact<'a>(
    bundle: &'a VerifiedDistributionBundle,
    identity: &PlatformReleaseIdentity,
) -> Result<&'a VerifiedDistributionArtifact, String> {
    let candidates = bundle
        .artifacts()
        .iter()
        .filter(|artifact| matches_identity(artifact, identity))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [artifact] => Ok(*artifact),
        _ => Err(format!(
            "signed release must contain exactly one designated {} artifact",
            identity.label
        )),
    }
}

fn matches_identity(
    artifact: &VerifiedDistributionArtifact,
    identity: &PlatformReleaseIdentity,
) -> bool {
    if artifact.artifact_id() != identity.artifact_id {
        return false;
    }
    let Some(descriptor) = artifact.module_descriptor() else {
        return false;
    };
    descriptor.module_id == identity.module_id
        && descriptor.owner_id == identity.owner_id
        && descriptor.module_kind == ModuleKindV1::Platform as i32
}

fn next_binding_revision(
    store: &SqliteControlStore,
    identity: &PlatformReleaseIdentity,
) -> Result<u64, String> {
    store
        .platform_managed_process_binding(identity.process_id)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |binding| {
            binding
                .binding_revision()
                .checked_add(1)
                .ok_or_else(|| format!("{} launch binding revision overflowed", identity.label))
        })
}
