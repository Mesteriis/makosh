//! Verifies exact artifacts from a signed installed distribution bundle.

use std::fs::{File, Metadata};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::{
    DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
    ModuleDescriptorV1, SettingsSchemaV1,
};
use makosh_runtime_protocol::validation::descriptor::{
    decode_descriptor_v1, decode_settings_schema_v1,
};
use sha2::{Digest, Sha256};

use crate::distribution::manifest_verifier as distribution_manifest_verifier;
use crate::distribution::trust_root::ReleaseTrustRoot;

pub struct VerifiedDistributionArtifact {
    artifact_id: String,
    artifact_kind: DistributionArtifactKindV1,
    canonical_path: PathBuf,
    size_bytes: u64,
    expected_sha256: [u8; 32],
    module_contract: Option<VerifiedModuleContract>,
}

struct VerifiedModuleContract {
    descriptor_sha256: [u8; 32],
    descriptor_bytes: Vec<u8>,
    descriptor: ModuleDescriptorV1,
    settings_schema_sha256: Option<[u8; 32]>,
    settings_schema_bytes: Option<Vec<u8>>,
    settings_schema: Option<SettingsSchemaV1>,
}

impl VerifiedDistributionArtifact {
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    #[must_use]
    pub fn artifact_kind(&self) -> DistributionArtifactKindV1 {
        self.artifact_kind
    }

    #[must_use]
    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    #[must_use]
    pub fn expected_sha256(&self) -> &[u8; 32] {
        &self.expected_sha256
    }

    #[must_use]
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Re-reads exact bytes with the same non-symlink, inode/metadata and
    /// digest checks used during bundle verification. Consumers retain the
    /// returned bytes in memory instead of serving a mutable bundle path.
    pub fn read_verified_bytes(&self) -> Result<Vec<u8>, String> {
        read_verified_file(
            &self.canonical_path,
            self.size_bytes,
            &self.expected_sha256,
            "distribution artifact",
        )
    }

    #[must_use]
    pub fn module_descriptor(&self) -> Option<&ModuleDescriptorV1> {
        self.module_contract
            .as_ref()
            .map(|contract| &contract.descriptor)
    }

    #[must_use]
    pub fn descriptor_sha256(&self) -> Option<&[u8; 32]> {
        self.module_contract
            .as_ref()
            .map(|contract| &contract.descriptor_sha256)
    }

    #[must_use]
    pub fn module_descriptor_bytes(&self) -> Option<&[u8]> {
        self.module_contract
            .as_ref()
            .map(|contract| contract.descriptor_bytes.as_slice())
    }

    #[must_use]
    pub fn settings_schema(&self) -> Option<&SettingsSchemaV1> {
        self.module_contract
            .as_ref()
            .and_then(|contract| contract.settings_schema.as_ref())
    }

    #[must_use]
    pub fn settings_schema_sha256(&self) -> Option<&[u8; 32]> {
        self.module_contract
            .as_ref()
            .and_then(|contract| contract.settings_schema_sha256.as_ref())
    }

    #[must_use]
    pub fn settings_schema_bytes(&self) -> Option<&[u8]> {
        self.module_contract
            .as_ref()
            .and_then(|contract| contract.settings_schema_bytes.as_deref())
    }
}

pub struct VerifiedDistributionBundle {
    manifest: DistributionManifestV1,
    artifacts: Vec<VerifiedDistributionArtifact>,
}

impl VerifiedDistributionBundle {
    #[must_use]
    pub fn manifest(&self) -> &DistributionManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn artifacts(&self) -> &[VerifiedDistributionArtifact] {
        &self.artifacts
    }
}

pub fn verify(
    bundle_root: &Path,
    signed_manifest_bytes: &[u8],
    trust_root: &ReleaseTrustRoot,
    expected_target_triple: &str,
) -> Result<VerifiedDistributionBundle, String> {
    let manifest = verify_manifest(signed_manifest_bytes, trust_root, expected_target_triple)?;
    ensure_bundle_root(bundle_root)?;
    let artifacts = manifest
        .artifacts
        .iter()
        .map(|artifact| verify_artifact(bundle_root, artifact))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifiedDistributionBundle {
        manifest,
        artifacts,
    })
}

/// Verifies the signed manifest and only artifacts of the requested kinds.
///
/// This is for consumers such as the browser bootstrap server that never stage
/// or execute the other artifacts in the distribution. The returned manifest
/// remains complete and signed; `artifacts()` contains only the verified
/// selection.
pub fn verify_artifact_kinds(
    bundle_root: &Path,
    signed_manifest_bytes: &[u8],
    trust_root: &ReleaseTrustRoot,
    expected_target_triple: &str,
    artifact_kinds: &[DistributionArtifactKindV1],
) -> Result<VerifiedDistributionBundle, String> {
    if artifact_kinds.is_empty() {
        return Err("distribution artifact kind selection must not be empty".to_owned());
    }
    let manifest = verify_manifest(signed_manifest_bytes, trust_root, expected_target_triple)?;
    for kind in artifact_kinds {
        if !manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_kind == *kind as i32)
        {
            return Err("selected distribution artifact kind is unavailable".to_owned());
        }
    }
    ensure_bundle_root(bundle_root)?;
    let artifacts = manifest
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact_kinds
                .iter()
                .any(|kind| artifact.artifact_kind == *kind as i32)
        })
        .map(|artifact| verify_artifact(bundle_root, artifact))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifiedDistributionBundle {
        manifest,
        artifacts,
    })
}

/// Verifies the signed manifest and only the exact artifact IDs requested by a
/// launch or transport consumer.
pub fn verify_artifact_ids(
    bundle_root: &Path,
    signed_manifest_bytes: &[u8],
    trust_root: &ReleaseTrustRoot,
    expected_target_triple: &str,
    artifact_ids: &[&str],
) -> Result<VerifiedDistributionBundle, String> {
    if artifact_ids.is_empty()
        || artifact_ids
            .iter()
            .any(|artifact_id| artifact_id.is_empty())
    {
        return Err("distribution artifact ID selection must not be empty".to_owned());
    }
    let manifest = verify_manifest(signed_manifest_bytes, trust_root, expected_target_triple)?;
    for artifact_id in artifact_ids {
        if !manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_id == *artifact_id)
        {
            return Err("selected distribution artifact ID is unavailable".to_owned());
        }
    }
    ensure_bundle_root(bundle_root)?;
    let artifacts = manifest
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact_ids
                .iter()
                .any(|artifact_id| artifact.artifact_id == *artifact_id)
        })
        .map(|artifact| verify_artifact(bundle_root, artifact))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifiedDistributionBundle {
        manifest,
        artifacts,
    })
}

fn verify_manifest(
    signed_manifest_bytes: &[u8],
    trust_root: &ReleaseTrustRoot,
    expected_target_triple: &str,
) -> Result<DistributionManifestV1, String> {
    let manifest = distribution_manifest_verifier::verify(signed_manifest_bytes, trust_root)?;
    if manifest.target_triple != expected_target_triple {
        return Err("distribution manifest target triple does not match this Kernel".to_owned());
    }
    Ok(manifest)
}

fn ensure_bundle_root(bundle_root: &Path) -> Result<(), String> {
    if !bundle_root.is_absolute() {
        return Err("distribution bundle root must be an absolute path".to_owned());
    }
    let mut current = PathBuf::new();
    for component in bundle_root.components() {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("distribution bundle root must not traverse a symlink".to_owned());
        }
    }
    let metadata = std::fs::symlink_metadata(bundle_root).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("distribution bundle root must be a non-symlink directory".to_owned());
    }
    Ok(())
}

fn verify_artifact(
    bundle_root: &Path,
    artifact: &DistributionManifestArtifactV1,
) -> Result<VerifiedDistributionArtifact, String> {
    let path = artifact_path(bundle_root, &artifact.relative_path)?;
    verify_digest(
        &path,
        artifact.size_bytes,
        &artifact.sha256,
        "distribution artifact",
    )?;
    let module_contract =
        if artifact.artifact_kind == DistributionArtifactKindV1::ModuleRuntime as i32 {
            Some(verify_module_contract(bundle_root, artifact)?)
        } else {
            None
        };
    Ok(VerifiedDistributionArtifact {
        artifact_id: artifact.artifact_id.clone(),
        artifact_kind: DistributionArtifactKindV1::try_from(artifact.artifact_kind)
            .map_err(|_| "distribution artifact kind is invalid".to_owned())?,
        canonical_path: path,
        size_bytes: artifact.size_bytes,
        expected_sha256: artifact.sha256.as_slice().try_into().map_err(|_| {
            "distribution artifact digest does not match manifest schema".to_owned()
        })?,
        module_contract,
    })
}

fn verify_module_contract(
    bundle_root: &Path,
    artifact: &DistributionManifestArtifactV1,
) -> Result<VerifiedModuleContract, String> {
    let descriptor_path = artifact_path(bundle_root, &artifact.descriptor_relative_path)?;
    let descriptor_bytes = read_verified_file(
        &descriptor_path,
        artifact.descriptor_size_bytes,
        &artifact.descriptor_sha256,
        "distribution descriptor",
    )?;
    let descriptor = decode_descriptor_v1(&descriptor_bytes)
        .map_err(|_| "distribution descriptor is invalid".to_owned())?;
    let (settings_schema_sha256, settings_schema_bytes, settings_schema) =
        if artifact.settings_schema_sha256.is_empty() {
            (None, None, None)
        } else {
            let path = artifact_path(bundle_root, &artifact.settings_schema_relative_path)?;
            let bytes = read_verified_file(
                &path,
                artifact.settings_schema_size_bytes,
                &artifact.settings_schema_sha256,
                "distribution settings schema",
            )?;
            let digest = artifact
                .settings_schema_sha256
                .as_slice()
                .try_into()
                .map_err(|_| "distribution settings schema digest is invalid".to_owned())?;
            let schema = decode_settings_schema_v1(&bytes)
                .map_err(|_| "distribution settings schema is invalid".to_owned())?;
            (Some(digest), Some(bytes), Some(schema))
        };
    validate_settings_schema_binding(artifact, &descriptor, settings_schema.as_ref())?;
    Ok(VerifiedModuleContract {
        descriptor_sha256: artifact
            .descriptor_sha256
            .as_slice()
            .try_into()
            .map_err(|_| "distribution descriptor digest is invalid".to_owned())?,
        descriptor_bytes,
        descriptor,
        settings_schema_sha256,
        settings_schema_bytes,
        settings_schema,
    })
}

fn validate_settings_schema_binding(
    artifact: &DistributionManifestArtifactV1,
    descriptor: &ModuleDescriptorV1,
    schema: Option<&SettingsSchemaV1>,
) -> Result<(), String> {
    match (descriptor.settings_schema_ref.as_ref(), schema) {
        (None, None) => Ok(()),
        (Some(reference), Some(schema))
            if reference.major == schema.major
                && reference.revision == schema.revision
                && reference.artifact_size_bytes == artifact.settings_schema_size_bytes
                && reference.sha256 == artifact.settings_schema_sha256 =>
        {
            Ok(())
        }
        _ => Err("distribution settings schema does not match module descriptor".to_owned()),
    }
}

fn verify_digest(
    path: &Path,
    expected_size: u64,
    expected_sha256: &[u8],
    label: &str,
) -> Result<(), String> {
    inspect_file(path, expected_size, expected_sha256, label, false).map(|_| ())
}

fn read_verified_file(
    path: &Path,
    expected_size: u64,
    expected_sha256: &[u8],
    label: &str,
) -> Result<Vec<u8>, String> {
    inspect_file(path, expected_size, expected_sha256, label, true)?
        .ok_or_else(|| "distribution contract reader did not retain verified bytes".to_owned())
}

fn inspect_file(
    path: &Path,
    expected_size: u64,
    expected_sha256: &[u8],
    label: &str,
    retain_bytes: bool,
) -> Result<Option<Vec<u8>>, String> {
    let path_before = regular_file_metadata(path, label)?;
    if path_before.len() != expected_size {
        return Err(format!("{label} size does not match manifest"));
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let opened = file.metadata().map_err(|error| error.to_string())?;
    if !same_file(&path_before, &opened) {
        return Err(format!("{label} changed while it was opened"));
    }
    let (digest, bytes) = sha256(&mut file, retain_bytes)?;
    let opened_after = file.metadata().map_err(|error| error.to_string())?;
    let path_after = regular_file_metadata(path, label)?;
    if !same_file(&opened, &opened_after) || !same_file(&opened, &path_after) {
        return Err(format!("{label} changed while it was read"));
    }
    if digest.as_slice() != expected_sha256 {
        return Err(format!("{label} digest does not match manifest"));
    }
    Ok(bytes)
}

fn artifact_path(bundle_root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let mut current = bundle_root.to_path_buf();
    for component in relative_path.split('/') {
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("distribution bundle artifact path must not traverse a symlink".to_owned());
        }
    }
    Ok(current)
}

fn regular_file_metadata(path: &Path, label: &str) -> Result<Metadata, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label} must be a regular non-symlink file"));
    }
    Ok(metadata)
}

fn sha256(file: &mut File, retain_bytes: bool) -> Result<([u8; 32], Option<Vec<u8>>), String> {
    let mut digest = Sha256::new();
    let mut bytes = retain_bytes.then(Vec::new);
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        if let Some(bytes) = &mut bytes {
            bytes.extend_from_slice(&buffer[..read]);
        }
    }
    Ok((digest.finalize().into(), bytes))
}

fn same_file(left: &Metadata, right: &Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}
