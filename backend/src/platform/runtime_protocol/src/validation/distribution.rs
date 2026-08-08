//! Strict structural validation for raw signed distribution manifest bytes.

use prost::Message;

use crate::v1::{
    DistributionArtifactKindV1, DistributionManifestV1, ReleaseTrustRootV1,
    SignedDistributionManifestV1,
};
use crate::validation::descriptor::{
    MAX_DESCRIPTOR_BYTES, MAX_IDENTIFIER_BYTES, MAX_SETTINGS_SCHEMA_BYTES,
};

pub const MAX_DISTRIBUTION_MANIFEST_BYTES: usize = 512 * 1024;
pub const MAX_DISTRIBUTION_ARTIFACTS: usize = 256;
pub const MAX_RELEASE_TRUST_ROOT_BYTES: usize = 16 * 1024;
pub const MAX_RELEASE_VERIFICATION_KEYS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistributionManifestValidationError {
    TooLarge,
    InvalidEncoding,
    InvalidVersion,
    InvalidIdentifier,
    InvalidTarget,
    TooManyArtifacts,
    UnorderedArtifacts,
    InvalidArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignedDistributionManifestValidationError {
    TooLarge,
    InvalidEncoding,
    InvalidKeyId,
    InvalidSignature,
    InvalidManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseTrustRootValidationError {
    TooLarge,
    InvalidEncoding,
    InvalidVersion,
    InvalidKeySet,
    UnorderedKeys,
    InvalidKey,
}

pub fn decode_distribution_manifest_v1(
    bytes: &[u8],
) -> Result<DistributionManifestV1, DistributionManifestValidationError> {
    if bytes.len() > MAX_DISTRIBUTION_MANIFEST_BYTES {
        return Err(DistributionManifestValidationError::TooLarge);
    }
    let manifest = DistributionManifestV1::decode(bytes)
        .map_err(|_| DistributionManifestValidationError::InvalidEncoding)?;
    validate_distribution_manifest_v1(&manifest)?;
    Ok(manifest)
}

pub fn validate_distribution_manifest_v1(
    manifest: &DistributionManifestV1,
) -> Result<(), DistributionManifestValidationError> {
    if manifest.major != 1 || manifest.revision == 0 || manifest.generation == 0 {
        return Err(DistributionManifestValidationError::InvalidVersion);
    }
    for value in [
        &manifest.distribution_id,
        &manifest.release_version,
        &manifest.build_id,
    ] {
        valid_id(value)?;
    }
    if !valid_target(&manifest.target_triple) {
        return Err(DistributionManifestValidationError::InvalidTarget);
    }
    if manifest.artifacts.is_empty() || manifest.artifacts.len() > MAX_DISTRIBUTION_ARTIFACTS {
        return Err(DistributionManifestValidationError::TooManyArtifacts);
    }
    let mut previous = "";
    for artifact in &manifest.artifacts {
        valid_id(&artifact.artifact_id)?;
        if artifact.artifact_id.as_str() <= previous {
            return Err(DistributionManifestValidationError::UnorderedArtifacts);
        }
        previous = &artifact.artifact_id;
        if DistributionArtifactKindV1::try_from(artifact.artifact_kind)
            .ok()
            .filter(|kind| *kind != DistributionArtifactKindV1::Unspecified)
            .is_none()
            || !valid_relative_path(&artifact.relative_path)
            || artifact.size_bytes == 0
            || artifact.sha256.len() != 32
        {
            return Err(DistributionManifestValidationError::InvalidArtifact);
        }
        let module = artifact.artifact_kind == DistributionArtifactKindV1::ModuleRuntime as i32;
        let bound_runtime_dependency = matches!(
            DistributionArtifactKindV1::try_from(artifact.artifact_kind).ok(),
            Some(
                DistributionArtifactKindV1::ModuleRuntimeNativeDependency
                    | DistributionArtifactKindV1::ModuleRuntimeNativeExecutable
                    | DistributionArtifactKindV1::ModuleRuntimeReadOnlyData
            )
        );
        if bound_runtime_dependency {
            if !valid_module_id(&artifact.bound_module_id) {
                return Err(DistributionManifestValidationError::InvalidArtifact);
            }
        } else if !artifact.bound_module_id.is_empty() {
            return Err(DistributionManifestValidationError::InvalidArtifact);
        }
        validate_module_contract_artifacts(artifact, module)?;
    }
    Ok(())
}

fn validate_module_contract_artifacts(
    artifact: &crate::v1::DistributionManifestArtifactV1,
    module: bool,
) -> Result<(), DistributionManifestValidationError> {
    if !module {
        if !artifact.descriptor_sha256.is_empty()
            || !artifact.descriptor_relative_path.is_empty()
            || artifact.descriptor_size_bytes != 0
            || !artifact.settings_schema_sha256.is_empty()
            || !artifact.settings_schema_relative_path.is_empty()
            || artifact.settings_schema_size_bytes != 0
        {
            return Err(DistributionManifestValidationError::InvalidArtifact);
        }
        return Ok(());
    }
    if artifact.descriptor_sha256.len() != 32
        || !valid_relative_path(&artifact.descriptor_relative_path)
        || artifact.descriptor_size_bytes == 0
        || artifact.descriptor_size_bytes > MAX_DESCRIPTOR_BYTES as u64
    {
        return Err(DistributionManifestValidationError::InvalidArtifact);
    }
    let has_settings_schema = !artifact.settings_schema_sha256.is_empty();
    if has_settings_schema
        && (artifact.settings_schema_sha256.len() != 32
            || !valid_relative_path(&artifact.settings_schema_relative_path)
            || artifact.settings_schema_size_bytes == 0
            || artifact.settings_schema_size_bytes > MAX_SETTINGS_SCHEMA_BYTES as u64)
    {
        return Err(DistributionManifestValidationError::InvalidArtifact);
    }
    if !has_settings_schema
        && (!artifact.settings_schema_relative_path.is_empty()
            || artifact.settings_schema_size_bytes != 0)
    {
        return Err(DistributionManifestValidationError::InvalidArtifact);
    }
    Ok(())
}

pub fn decode_signed_distribution_manifest_v1(
    bytes: &[u8],
) -> Result<SignedDistributionManifestV1, SignedDistributionManifestValidationError> {
    if bytes.len() > MAX_DISTRIBUTION_MANIFEST_BYTES + 1024 {
        return Err(SignedDistributionManifestValidationError::TooLarge);
    }
    let signed = SignedDistributionManifestV1::decode(bytes)
        .map_err(|_| SignedDistributionManifestValidationError::InvalidEncoding)?;
    if signed.verification_key_id.is_empty()
        || signed.verification_key_id.len() > MAX_IDENTIFIER_BYTES
        || !signed.verification_key_id.is_ascii()
    {
        return Err(SignedDistributionManifestValidationError::InvalidKeyId);
    }
    if signed.signature_raw.len() != 64 {
        return Err(SignedDistributionManifestValidationError::InvalidSignature);
    }
    decode_distribution_manifest_v1(&signed.raw_manifest_bytes)
        .map_err(|_| SignedDistributionManifestValidationError::InvalidManifest)?;
    Ok(signed)
}

pub fn decode_release_trust_root_v1(
    bytes: &[u8],
) -> Result<ReleaseTrustRootV1, ReleaseTrustRootValidationError> {
    if bytes.len() > MAX_RELEASE_TRUST_ROOT_BYTES {
        return Err(ReleaseTrustRootValidationError::TooLarge);
    }
    let root = ReleaseTrustRootV1::decode(bytes)
        .map_err(|_| ReleaseTrustRootValidationError::InvalidEncoding)?;
    validate_release_trust_root_v1(&root)?;
    Ok(root)
}

pub fn validate_release_trust_root_v1(
    root: &ReleaseTrustRootV1,
) -> Result<(), ReleaseTrustRootValidationError> {
    if root.major != 1 || root.revision == 0 {
        return Err(ReleaseTrustRootValidationError::InvalidVersion);
    }
    if root.verification_keys.is_empty()
        || root.verification_keys.len() > MAX_RELEASE_VERIFICATION_KEYS
    {
        return Err(ReleaseTrustRootValidationError::InvalidKeySet);
    }
    let mut previous = "";
    for key in &root.verification_keys {
        if key.key_id.is_empty()
            || key.key_id.len() > MAX_IDENTIFIER_BYTES
            || !key.key_id.is_ascii()
            || key.public_key_sec1.len() != 65
            || key.public_key_sec1.first() != Some(&4)
        {
            return Err(ReleaseTrustRootValidationError::InvalidKey);
        }
        if key.key_id.as_str() <= previous {
            return Err(ReleaseTrustRootValidationError::UnorderedKeys);
        }
        previous = &key.key_id;
    }
    Ok(())
}

fn valid_id(value: &str) -> Result<(), DistributionManifestValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() {
        Err(DistributionManifestValidationError::InvalidIdentifier)
    } else {
        Ok(())
    }
}

fn valid_module_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_target(value: &str) -> bool {
    value.len() <= MAX_IDENTIFIER_BYTES
        && value.is_ascii()
        && value.contains('-')
        && !value.contains(['/', '\\', ' '])
}
fn valid_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && !value.starts_with('/')
        && !value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

#[cfg(test)]
mod tests {
    use super::validate_distribution_manifest_v1;
    use crate::v1::{
        DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
    };

    #[test]
    fn runtime_dependencies_require_one_exact_bound_module() {
        let mut manifest = DistributionManifestV1 {
            major: 1,
            revision: 1,
            distribution_id: "makosh-desktop".to_owned(),
            release_version: "1.0.0".to_owned(),
            build_id: "build-1".to_owned(),
            target_triple: "aarch64-apple-darwin".to_owned(),
            generation: 1,
            artifacts: vec![DistributionManifestArtifactV1 {
                artifact_kind: DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32,
                artifact_id: "telegram.tdjson.v1".to_owned(),
                relative_path: "lib/libtdjson.dylib".to_owned(),
                size_bytes: 1,
                sha256: vec![7; 32],
                required: true,
                bound_module_id: "makosh-telegram-runtime".to_owned(),
                ..Default::default()
            }],
        };
        assert_eq!(validate_distribution_manifest_v1(&manifest), Ok(()));

        for kind in [
            DistributionArtifactKindV1::ModuleRuntimeNativeExecutable,
            DistributionArtifactKindV1::ModuleRuntimeReadOnlyData,
        ] {
            manifest.artifacts[0].artifact_kind = kind as i32;
            assert_eq!(validate_distribution_manifest_v1(&manifest), Ok(()));
        }

        manifest.artifacts[0].artifact_kind =
            DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32;

        manifest.artifacts[0].bound_module_id.clear();
        assert!(validate_distribution_manifest_v1(&manifest).is_err());

        manifest.artifacts[0].bound_module_id = "Макошь Telegram".to_owned();
        assert!(validate_distribution_manifest_v1(&manifest).is_err());

        manifest.artifacts[0].artifact_kind = DistributionArtifactKindV1::BrowserClientAsset as i32;
        manifest.artifacts[0].bound_module_id = "makosh-telegram-runtime".to_owned();
        assert!(validate_distribution_manifest_v1(&manifest).is_err());
    }
}
