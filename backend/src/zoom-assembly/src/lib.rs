#![forbid(unsafe_code)]
use makosh_runtime_protocol::validation::descriptor::{
    decode_settings_schema_v1, validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use makosh_zoom_persistence::zoom_storage_bundle_v1;
use makosh_zoom_runtime::zoom_module_descriptor_v1;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
pub const PACKAGE: &str = "makosh-zoom-assembly";
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    relative_path: String,
    source_path: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Runtime {
    artifact_kind: String,
    artifact_id: String,
    relative_path: String,
    source_path: String,
    required: bool,
    descriptor: Contract,
    settings_schema: Contract,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Storage {
    artifact_kind: String,
    artifact_id: String,
    relative_path: String,
    source_path: String,
    required: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
enum Artifact {
    Runtime(Runtime),
    Storage(Storage),
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    version: u32,
    owner_id: String,
    module_id: String,
    artifacts: Vec<Artifact>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoomAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoomAssemblyErrorV1 {
    InvalidInput,
    InvalidArtifact,
    OutputUnavailable,
    WriteFailed,
}
pub fn materialize_zoom_release_assembly_v1(
    output: &Path,
    build: &str,
    runtime: &Path,
) -> Result<ZoomAssemblyPathsV1, ZoomAssemblyErrorV1> {
    if !output.is_absolute() || output.exists() || build.is_empty() || !runtime.is_absolute() {
        return Err(ZoomAssemblyErrorV1::InvalidInput);
    }
    let runtime_bytes = fs::read(runtime).map_err(|_| ZoomAssemblyErrorV1::InvalidInput)?;
    if runtime_bytes.is_empty() {
        return Err(ZoomAssemblyErrorV1::InvalidInput);
    }
    let descriptor = zoom_module_descriptor_v1(build);
    let settings = zoom_settings();
    let storage = zoom_storage_bundle_v1();
    validate_descriptor_v1(&descriptor).map_err(|_| ZoomAssemblyErrorV1::InvalidArtifact)?;
    let decoded =
        decode_settings_schema_v1(&settings).map_err(|_| ZoomAssemblyErrorV1::InvalidArtifact)?;
    validate_settings_schema_v1(&decoded).map_err(|_| ZoomAssemblyErrorV1::InvalidArtifact)?;
    validate_storage_bundle(&storage).map_err(|_| ZoomAssemblyErrorV1::InvalidArtifact)?;
    let paths = ZoomAssemblyPathsV1 {
        descriptor: output.join("zoom.runtime.descriptor.pb"),
        settings_schema: output.join("zoom.runtime.settings.pb"),
        storage_bundle: output.join("zoom.storage.bundle.pb"),
        artifact_fragment: output.join("zoom.release-artifacts.json"),
    };
    let utf8 = |p: &Path| {
        p.to_str()
            .map(str::to_owned)
            .ok_or(ZoomAssemblyErrorV1::InvalidInput)
    };
    let fragment = Fragment {
        version: 1,
        owner_id: descriptor.owner_id.clone(),
        module_id: descriptor.module_id.clone(),
        artifacts: vec![
            Artifact::Runtime(Runtime {
                artifact_kind: "module_runtime".into(),
                artifact_id: "zoom.runtime.v1".into(),
                relative_path: "bin/makosh-zoom-runtime".into(),
                source_path: utf8(runtime)?,
                required: true,
                descriptor: Contract {
                    relative_path: "contracts/zoom.runtime.descriptor.pb".into(),
                    source_path: utf8(&paths.descriptor)?,
                },
                settings_schema: Contract {
                    relative_path: "contracts/zoom.runtime.settings.pb".into(),
                    source_path: utf8(&paths.settings_schema)?,
                },
            }),
            Artifact::Storage(Storage {
                artifact_kind: "storage_bundle".into(),
                artifact_id: "zoom.storage.v1".into(),
                relative_path: "storage/zoom.storage.bundle.pb".into(),
                source_path: utf8(&paths.storage_bundle)?,
                required: true,
            }),
        ],
    };
    let fragment =
        serde_json::to_vec_pretty(&fragment).map_err(|_| ZoomAssemblyErrorV1::InvalidArtifact)?;
    let mut dir = DirBuilder::new();
    dir.mode(0o700);
    dir.create(output)
        .map_err(|_| ZoomAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, fragment),
    ] {
        if write(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output);
            return Err(ZoomAssemblyErrorV1::WriteFailed);
        }
    }
    Ok(paths)
}
fn zoom_settings() -> Vec<u8> {
    makosh_zoom_api::zoom_settings_schema_bytes_v1()
}
fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut o = OpenOptions::new();
    o.write(true).create_new(true).mode(0o600);
    let mut f = o.open(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn materializes_compiler_shape() {
        let root = std::env::temp_dir().join(format!("makosh-zoom-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").unwrap();
        let paths =
            materialize_zoom_release_assembly_v1(&root.join("out"), "build", &runtime).unwrap();
        let fragment: Fragment =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).unwrap()).unwrap();
        assert_eq!(fragment.artifacts.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
