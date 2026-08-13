#![forbid(unsafe_code)]
use makosh_consistency_persistence::consistency_storage_bundle_v1;
use makosh_consistency_runtime::{
    consistency_module_descriptor_v1, consistency_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    decode_settings_schema_v1, validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
pub const PACKAGE: &str = "makosh-consistency-assembly";
pub const CONSISTENCY_FRAGMENT_FILE_V1: &str = "consistency.release-artifacts.json";
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContractInputV1 {
    pub relative_path: String,
    pub source_path: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
    pub descriptor: ContractInputV1,
    pub settings_schema: ContractInputV1,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ArtifactV1 {
    Runtime(RuntimeInputV1),
    Storage(StorageInputV1),
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<ArtifactV1>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssemblyErrorV1 {
    InvalidInput,
    InvalidArtifact,
    OutputUnavailable,
    WriteFailed,
}
pub fn materialize_consistency_release_assembly_v1(
    output: &Path,
    build: &str,
    runtime: &Path,
) -> Result<PathsV1, AssemblyErrorV1> {
    if !output.is_absolute() || output.exists() || build.is_empty() || !runtime.is_absolute() {
        return Err(AssemblyErrorV1::InvalidInput);
    }
    let runtime_bytes = fs::read(runtime).map_err(|_| AssemblyErrorV1::InvalidInput)?;
    if runtime_bytes.is_empty() {
        return Err(AssemblyErrorV1::InvalidInput);
    }
    let descriptor = consistency_module_descriptor_v1(build);
    let settings_bytes = consistency_settings_schema_bytes_v1();
    let settings =
        decode_settings_schema_v1(&settings_bytes).map_err(|_| AssemblyErrorV1::InvalidArtifact)?;
    let storage = consistency_storage_bundle_v1();
    validate_descriptor_v1(&descriptor).map_err(|_| AssemblyErrorV1::InvalidArtifact)?;
    validate_settings_schema_v1(&settings).map_err(|_| AssemblyErrorV1::InvalidArtifact)?;
    validate_storage_bundle(&storage).map_err(|_| AssemblyErrorV1::InvalidArtifact)?;
    let paths = PathsV1 {
        descriptor: output.join("consistency.runtime.descriptor.pb"),
        settings_schema: output.join("consistency.runtime.settings.pb"),
        storage_bundle: output.join("consistency.storage.bundle.pb"),
        artifact_fragment: output.join(CONSISTENCY_FRAGMENT_FILE_V1),
    };
    let utf8 = |path: &Path| {
        path.to_str()
            .map(str::to_owned)
            .ok_or(AssemblyErrorV1::InvalidInput)
    };
    let fragment = FragmentV1 {
        version: 1,
        owner_id: descriptor.owner_id.clone(),
        module_id: descriptor.module_id.clone(),
        artifacts: vec![
            ArtifactV1::Runtime(RuntimeInputV1 {
                artifact_kind: "module_runtime".into(),
                artifact_id: "consistency.runtime.v1".into(),
                relative_path: "bin/makosh-consistency-runtime".into(),
                source_path: utf8(runtime)?,
                required: true,
                descriptor: ContractInputV1 {
                    relative_path: "contracts/consistency.runtime.descriptor.pb".into(),
                    source_path: utf8(&paths.descriptor)?,
                },
                settings_schema: ContractInputV1 {
                    relative_path: "contracts/consistency.runtime.settings.pb".into(),
                    source_path: utf8(&paths.settings_schema)?,
                },
            }),
            ArtifactV1::Storage(StorageInputV1 {
                artifact_kind: "storage_bundle".into(),
                artifact_id: "consistency.storage.v1".into(),
                relative_path: "storage/consistency.storage.bundle.pb".into(),
                source_path: utf8(&paths.storage_bundle)?,
                required: true,
            }),
        ],
    };
    let bytes =
        serde_json::to_vec_pretty(&fragment).map_err(|_| AssemblyErrorV1::InvalidArtifact)?;
    let mut dir = DirBuilder::new();
    dir.mode(0o700);
    dir.create(output)
        .map_err(|_| AssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings_bytes),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, bytes),
    ] {
        if write(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output);
            return Err(AssemblyErrorV1::WriteFailed);
        }
    }
    Ok(paths)
}
fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn materializes_compiler_shape() {
        let root = std::env::temp_dir().join(format!("makosh-consistency-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").unwrap();
        let paths =
            materialize_consistency_release_assembly_v1(&root.join("out"), "build", &runtime)
                .unwrap();
        let fragment: FragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).unwrap()).unwrap();
        assert_eq!(fragment.artifacts.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
