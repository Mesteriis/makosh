//! Owner-authorized admission of canonical Storage bundles.

use makosh_gateway_protocol::v1::{
    AdmitBundledStorageArtifactRequestV1, AdmitBundledStorageArtifactResponseV1,
    AdmitStorageBundleRequestV1, AdmitStorageBundleResponseV1,
};
use makosh_kernel_control_store::PlatformStorageBundleV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_storage_protocol::{v1::StorageBundleV1, validation::validate_storage_bundle};
use prost::Message;
use sha2::{Digest, Sha256};

use super::super::{OwnerControlSessions, OwnerResult};

pub(super) fn admit_bundled(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: AdmitBundledStorageArtifactRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let bytes = crate::platform::macos::bundled_release::read_current_installed_storage_artifact(
        &request.artifact_id,
        &request.expected_distribution_id,
        request.expected_distribution_generation,
    )?;
    let bundle = admit_canonical_bundle(store, &bytes)?;
    Ok(OwnerResult::AdmitBundledStorageArtifact(
        AdmitBundledStorageArtifactResponseV1 {
            owner_id: bundle.owner_id().to_owned(),
            storage_bundle_revision: bundle.revision(),
            storage_bundle_digest: bundle.digest().to_vec(),
            distribution_id: request.expected_distribution_id,
            distribution_generation: request.expected_distribution_generation,
            artifact_id: request.artifact_id,
        },
    ))
}

pub(super) fn admit(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: AdmitStorageBundleRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        admit_canonical_bundle(store, &request.canonical_bundle)
    })()
    .map(|bundle| {
        OwnerResult::AdmitStorageBundle(AdmitStorageBundleResponseV1 {
            owner_id: bundle.owner_id().to_owned(),
            storage_bundle_revision: bundle.revision(),
            storage_bundle_digest: bundle.digest().to_vec(),
        })
    })
}

fn admit_canonical_bundle(
    store: &SqliteControlStore,
    bytes: &[u8],
) -> Result<PlatformStorageBundleV1, String> {
    let bundle = canonical_bundle(bytes)?;
    store
        .record_platform_storage_bundle(&bundle)
        .map_err(|_| "Storage bundle cannot be recorded".to_owned())?;
    Ok(bundle)
}

fn canonical_bundle(bytes: &[u8]) -> Result<PlatformStorageBundleV1, String> {
    if bytes.is_empty() || bytes.len() > 4 * 1024 * 1024 {
        return Err("Storage bundle is invalid".to_owned());
    }
    let bundle =
        StorageBundleV1::decode(bytes).map_err(|_| "Storage bundle is invalid".to_owned())?;
    validate_storage_bundle(&bundle).map_err(|_| "Storage bundle is invalid".to_owned())?;
    (bundle.encode_to_vec() == bytes)
        .then_some(())
        .ok_or_else(|| "Storage bundle is not canonical".to_owned())?;
    PlatformStorageBundleV1::new(
        bundle.owner_id,
        u64::from(bundle.revision),
        Sha256::digest(bytes).into(),
        bytes.to_vec(),
    )
    .map_err(|_| "Storage bundle is invalid".to_owned())
}
