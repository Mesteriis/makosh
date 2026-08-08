//! Builds an isolated signed macOS release bundle for managed-process conformance.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::{
    DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
    ModuleDescriptorV1, ReleaseTrustRootKeyV1, ReleaseTrustRootV1, SignedDistributionManifestV1,
};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use prost::Message;
use sha2::{Digest, Sha256};

const TARGET_TRIPLE: &str = "aarch64-apple-darwin";
const SIGNING_KEY_ID: &str = "managed-runtime-test-key";

pub(crate) struct SignedRuntimeArtifact {
    artifact_id: &'static str,
    binary: PathBuf,
    descriptor: Vec<u8>,
    settings_schema: Option<Vec<u8>>,
}

pub(crate) struct SignedNativeDependency {
    artifact_id: &'static str,
    binary: PathBuf,
    bound_module_id: &'static str,
}

pub(crate) struct SignedRuntimeResource {
    artifact_id: &'static str,
    source: PathBuf,
    bound_module_id: &'static str,
    artifact_kind: DistributionArtifactKindV1,
}

impl SignedRuntimeArtifact {
    pub(crate) fn new(artifact_id: &'static str, binary: PathBuf, descriptor: Vec<u8>) -> Self {
        Self {
            artifact_id,
            binary,
            descriptor,
            settings_schema: None,
        }
    }

    pub(crate) fn with_settings_schema(mut self, settings_schema: Vec<u8>) -> Self {
        self.settings_schema = Some(settings_schema);
        self
    }
}

impl SignedNativeDependency {
    pub(crate) fn new(
        artifact_id: &'static str,
        binary: PathBuf,
        bound_module_id: &'static str,
    ) -> Self {
        Self {
            artifact_id,
            binary,
            bound_module_id,
        }
    }
}

impl SignedRuntimeResource {
    pub(crate) fn native_executable(
        artifact_id: &'static str,
        source: PathBuf,
        bound_module_id: &'static str,
    ) -> Self {
        Self {
            artifact_id,
            source,
            bound_module_id,
            artifact_kind: DistributionArtifactKindV1::ModuleRuntimeNativeExecutable,
        }
    }

    pub(crate) fn read_only_data(
        artifact_id: &'static str,
        source: PathBuf,
        bound_module_id: &'static str,
    ) -> Self {
        Self {
            artifact_id,
            source,
            bound_module_id,
            artifact_kind: DistributionArtifactKindV1::ModuleRuntimeReadOnlyData,
        }
    }
}

pub(crate) struct InstalledSignedBundle {
    kernel: PathBuf,
}

impl InstalledSignedBundle {
    pub(crate) fn install(
        root: &Path,
        artifacts: &[SignedRuntimeArtifact],
    ) -> Result<Self, String> {
        Self::install_with_native_dependencies(root, artifacts, &[])
    }

    pub(crate) fn install_with_native_dependencies(
        root: &Path,
        artifacts: &[SignedRuntimeArtifact],
        native_dependencies: &[SignedNativeDependency],
    ) -> Result<Self, String> {
        Self::install_with_runtime_resources(root, artifacts, native_dependencies, &[])
    }

    pub(crate) fn install_with_runtime_resources(
        root: &Path,
        artifacts: &[SignedRuntimeArtifact],
        native_dependencies: &[SignedNativeDependency],
        runtime_resources: &[SignedRuntimeResource],
    ) -> Result<Self, String> {
        if artifacts.is_empty() {
            return Err("signed release must contain managed artifacts".to_owned());
        }
        let kernel = root.join("Макошь.app/Contents/MacOS/makosh-kernel");
        let resources = root.join("Макошь.app/Contents/Resources/makosh-kernel-release");
        let distribution = resources.join("distribution");
        std::fs::create_dir_all(
            kernel
                .parent()
                .ok_or_else(|| "signed release Kernel path is invalid".to_owned())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&distribution).map_err(|error| error.to_string())?;
        std::fs::write(&kernel, b"test-kernel").map_err(|error| error.to_string())?;
        let manifest = install_artifacts(
            &distribution,
            artifacts,
            native_dependencies,
            runtime_resources,
        )?;
        write_release_signature(&resources, &manifest)?;
        Ok(Self { kernel })
    }

    #[must_use]
    pub(crate) fn kernel(&self) -> &Path {
        &self.kernel
    }
}

fn install_artifacts(
    distribution: &Path,
    artifacts: &[SignedRuntimeArtifact],
    native_dependencies: &[SignedNativeDependency],
    runtime_resources: &[SignedRuntimeResource],
) -> Result<DistributionManifestV1, String> {
    let mut manifest_artifacts = artifacts
        .iter()
        .map(|artifact| install_artifact(distribution, artifact))
        .collect::<Result<Vec<_>, _>>()?;
    manifest_artifacts.extend(
        native_dependencies
            .iter()
            .map(|dependency| install_native_dependency(distribution, dependency))
            .collect::<Result<Vec<_>, _>>()?,
    );
    manifest_artifacts.extend(
        runtime_resources
            .iter()
            .map(|resource| install_runtime_resource(distribution, resource))
            .collect::<Result<Vec<_>, _>>()?,
    );
    manifest_artifacts.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    Ok(DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "makosh-managed-runtime-conformance".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "managed-runtime-conformance".to_owned(),
        target_triple: TARGET_TRIPLE.to_owned(),
        generation: 1,
        artifacts: manifest_artifacts,
    })
}

fn install_runtime_resource(
    distribution: &Path,
    resource: &SignedRuntimeResource,
) -> Result<DistributionManifestArtifactV1, String> {
    let directory = match resource.artifact_kind {
        DistributionArtifactKindV1::ModuleRuntimeNativeExecutable => "native-bin",
        DistributionArtifactKindV1::ModuleRuntimeReadOnlyData => "data",
        _ => return Err("signed runtime resource kind is invalid".to_owned()),
    };
    let relative_path = format!("{directory}/{}", resource.artifact_id);
    let path = distribution.join(&relative_path);
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| "signed runtime resource path is invalid".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::copy(&resource.source, &path).map_err(|error| error.to_string())?;
    let mode = match resource.artifact_kind {
        DistributionArtifactKindV1::ModuleRuntimeNativeExecutable => 0o700,
        DistributionArtifactKindV1::ModuleRuntimeReadOnlyData => 0o600,
        _ => unreachable!("validated runtime resource kind"),
    };
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
        .map_err(|error| error.to_string())?;
    let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
    Ok(DistributionManifestArtifactV1 {
        artifact_kind: resource.artifact_kind as i32,
        artifact_id: resource.artifact_id.to_owned(),
        relative_path,
        size_bytes: bytes.len() as u64,
        sha256: Sha256::digest(&bytes).to_vec(),
        descriptor_sha256: Vec::new(),
        settings_schema_sha256: Vec::new(),
        required: true,
        descriptor_relative_path: String::new(),
        descriptor_size_bytes: 0,
        settings_schema_relative_path: String::new(),
        settings_schema_size_bytes: 0,
        bound_module_id: resource.bound_module_id.to_owned(),
    })
}

fn install_native_dependency(
    distribution: &Path,
    dependency: &SignedNativeDependency,
) -> Result<DistributionManifestArtifactV1, String> {
    let relative_path = format!("lib/{}", dependency.artifact_id);
    let path = distribution.join(&relative_path);
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| "signed native dependency path is invalid".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::copy(&dependency.binary, &path).map_err(|error| error.to_string())?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
    Ok(DistributionManifestArtifactV1 {
        artifact_kind: DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32,
        artifact_id: dependency.artifact_id.to_owned(),
        relative_path,
        size_bytes: bytes.len() as u64,
        sha256: Sha256::digest(&bytes).to_vec(),
        descriptor_sha256: Vec::new(),
        settings_schema_sha256: Vec::new(),
        required: true,
        descriptor_relative_path: String::new(),
        descriptor_size_bytes: 0,
        settings_schema_relative_path: String::new(),
        settings_schema_size_bytes: 0,
        bound_module_id: dependency.bound_module_id.to_owned(),
    })
}

fn install_artifact(
    distribution: &Path,
    artifact: &SignedRuntimeArtifact,
) -> Result<DistributionManifestArtifactV1, String> {
    let binary_relative_path = format!("bin/{}", artifact.artifact_id);
    let binary_path = distribution.join(&binary_relative_path);
    std::fs::create_dir_all(
        binary_path
            .parent()
            .ok_or_else(|| "signed release artifact path is invalid".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::copy(&artifact.binary, &binary_path).map_err(|error| error.to_string())?;
    std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let descriptor_relative_path = format!("contracts/{}.descriptor.pb", artifact.artifact_id);
    let descriptor_path = distribution.join(&descriptor_relative_path);
    std::fs::create_dir_all(
        descriptor_path
            .parent()
            .ok_or_else(|| "signed release descriptor path is invalid".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(&descriptor_path, &artifact.descriptor).map_err(|error| error.to_string())?;
    let binary_bytes = std::fs::read(&binary_path).map_err(|error| error.to_string())?;
    let descriptor = ModuleDescriptorV1::decode(artifact.descriptor.as_slice())
        .map_err(|_| "signed release descriptor is invalid".to_owned())?;
    if descriptor.module_id.is_empty() || descriptor.owner_id.is_empty() {
        return Err("signed release descriptor has no platform identity".to_owned());
    }
    let (settings_schema_relative_path, settings_schema_size_bytes, settings_schema_sha256) =
        install_settings_schema(distribution, artifact)?;
    Ok(DistributionManifestArtifactV1 {
        artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
        artifact_id: artifact.artifact_id.to_owned(),
        relative_path: binary_relative_path,
        size_bytes: binary_bytes.len() as u64,
        sha256: Sha256::digest(&binary_bytes).to_vec(),
        descriptor_sha256: Sha256::digest(&artifact.descriptor).to_vec(),
        settings_schema_sha256,
        required: true,
        descriptor_relative_path,
        descriptor_size_bytes: artifact.descriptor.len() as u64,
        settings_schema_relative_path,
        settings_schema_size_bytes,
        bound_module_id: String::new(),
    })
}

fn install_settings_schema(
    distribution: &Path,
    artifact: &SignedRuntimeArtifact,
) -> Result<(String, u64, Vec<u8>), String> {
    let Some(schema) = &artifact.settings_schema else {
        return Ok((String::new(), 0, Vec::new()));
    };
    let relative_path = format!("contracts/{}.settings.pb", artifact.artifact_id);
    let path = distribution.join(&relative_path);
    std::fs::write(&path, schema).map_err(|error| error.to_string())?;
    Ok((
        relative_path,
        schema.len() as u64,
        Sha256::digest(schema).to_vec(),
    ))
}

fn write_release_signature(
    resources: &Path,
    manifest: &DistributionManifestV1,
) -> Result<(), String> {
    let signing_key = SigningKey::from_bytes((&[43_u8; 32]).into())
        .map_err(|_| "test signing key is invalid".to_owned())?;
    let raw_manifest_bytes = manifest.encode_to_vec();
    let signature: Signature = signing_key.sign(&raw_manifest_bytes);
    let signed = SignedDistributionManifestV1 {
        verification_key_id: SIGNING_KEY_ID.to_owned(),
        raw_manifest_bytes,
        signature_raw: signature.to_bytes().to_vec(),
    };
    let trust_root = ReleaseTrustRootV1 {
        major: 1,
        revision: 1,
        verification_keys: vec![ReleaseTrustRootKeyV1 {
            key_id: SIGNING_KEY_ID.to_owned(),
            public_key_sec1: signing_key
                .verifying_key()
                .to_sec1_point(false)
                .as_bytes()
                .to_vec(),
        }],
    };
    std::fs::write(
        resources.join("makosh-signed-distribution-manifest.pb"),
        signed.encode_to_vec(),
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        resources.join("makosh-release-trust-root.pb"),
        trust_root.encode_to_vec(),
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use makosh_runtime_protocol::validation::distribution::decode_distribution_manifest_v1;

    use super::*;

    #[test]
    fn signs_native_dependency_as_an_exact_module_bound_release_artifact() {
        let root = std::env::temp_dir().join(format!(
            "makosh-signed-native-dependency-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("test root");
        let runtime = root.join("runtime");
        let dependency = root.join("libtdjson.dylib");
        std::fs::write(&runtime, b"runtime").expect("runtime fixture");
        std::fs::write(&dependency, b"dependency").expect("dependency fixture");
        let descriptor = ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_id: "makosh-telegram-runtime".to_owned(),
            owner_id: "telegram".to_owned(),
            ..Default::default()
        }
        .encode_to_vec();

        InstalledSignedBundle::install_with_native_dependencies(
            &root,
            &[SignedRuntimeArtifact::new(
                "makosh-telegram-runtime",
                runtime,
                descriptor,
            )],
            &[SignedNativeDependency::new(
                "telegram-tdjson-v1",
                dependency,
                "makosh-telegram-runtime",
            )],
        )
        .expect("signed bundle");

        let signed = SignedDistributionManifestV1::decode(
            std::fs::read(
                root.join(
                    "Макошь.app/Contents/Resources/makosh-kernel-release/makosh-signed-distribution-manifest.pb",
                ),
            )
            .expect("signed manifest")
            .as_slice(),
        )
        .expect("signed manifest encoding");
        let manifest =
            decode_distribution_manifest_v1(&signed.raw_manifest_bytes).expect("valid manifest");
        let native = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_id == "telegram-tdjson-v1")
            .expect("native dependency artifact");
        assert_eq!(
            native.artifact_kind,
            DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32
        );
        assert_eq!(native.bound_module_id, "makosh-telegram-runtime");
        assert_eq!(native.relative_path, "lib/telegram-tdjson-v1");
        assert!(native.descriptor_relative_path.is_empty());
        assert!(native.settings_schema_relative_path.is_empty());

        std::fs::remove_dir_all(root).expect("remove test root");
    }

    #[test]
    fn signs_executable_and_read_only_runtime_resources_as_distinct_kinds() {
        let root = std::env::temp_dir().join(format!(
            "makosh-signed-runtime-resources-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("test root");
        let runtime = root.join("runtime");
        let runner = root.join("runner");
        let model = root.join("model");
        std::fs::write(&runtime, b"runtime").expect("runtime fixture");
        std::fs::write(&runner, b"runner").expect("runner fixture");
        std::fs::write(&model, b"model").expect("model fixture");
        let descriptor = ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_id: "makosh-attachment-text-extraction-runtime".to_owned(),
            owner_id: "attachment_text_extraction".to_owned(),
            ..Default::default()
        }
        .encode_to_vec();

        InstalledSignedBundle::install_with_runtime_resources(
            &root,
            &[SignedRuntimeArtifact::new(
                "attachment_text_extraction.runtime.v1",
                runtime,
                descriptor,
            )],
            &[],
            &[
                SignedRuntimeResource::native_executable(
                    "attachment_text_extraction.ocr.runner.v1",
                    runner,
                    "makosh-attachment-text-extraction-runtime",
                ),
                SignedRuntimeResource::read_only_data(
                    "attachment_text_extraction.ocr.eng.v1",
                    model,
                    "makosh-attachment-text-extraction-runtime",
                ),
            ],
        )
        .expect("signed bundle");

        let signed = SignedDistributionManifestV1::decode(
            std::fs::read(
                root.join(
                    "Макошь.app/Contents/Resources/makosh-kernel-release/makosh-signed-distribution-manifest.pb",
                ),
            )
            .expect("signed manifest")
            .as_slice(),
        )
        .expect("signed manifest encoding");
        let manifest =
            decode_distribution_manifest_v1(&signed.raw_manifest_bytes).expect("valid manifest");
        let runner = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_id.ends_with("runner.v1"))
            .expect("runner resource");
        let model = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_id.ends_with("eng.v1"))
            .expect("model resource");
        assert_eq!(
            runner.artifact_kind,
            DistributionArtifactKindV1::ModuleRuntimeNativeExecutable as i32
        );
        assert_eq!(
            model.artifact_kind,
            DistributionArtifactKindV1::ModuleRuntimeReadOnlyData as i32
        );
        assert_eq!(runner.bound_module_id, model.bound_module_id);

        std::fs::remove_dir_all(root).expect("remove test root");
    }
}
