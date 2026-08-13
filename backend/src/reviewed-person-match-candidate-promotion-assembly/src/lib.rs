#![forbid(unsafe_code)]

use makosh_reviewed_person_match_candidate_promotion_persistence::reviewed_person_match_candidate_promotion_storage_bundle_v1;
use makosh_reviewed_person_match_candidate_promotion_runtime::{
    reviewed_person_match_candidate_promotion_module_descriptor_v1,
    reviewed_person_match_candidate_promotion_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
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

pub const PACKAGE: &str = "makosh-reviewed-person-match-candidate-promotion-assembly";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseContractInputV1 {
    pub relative_path: String,
    pub source_path: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleRuntimeArtifactInputV1 {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
    pub descriptor: ReleaseContractInputV1,
    pub settings_schema: ReleaseContractInputV1,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageBundleArtifactInputV1 {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ReviewedPersonMatchCandidatePromotionArtifactV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}
impl ReviewedPersonMatchCandidatePromotionArtifactV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedPersonMatchCandidatePromotionFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<ReviewedPersonMatchCandidatePromotionArtifactV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionAssemblyPathsV1 {
    pub runtime: PathBuf,
    pub descriptor: PathBuf,
    pub settings: PathBuf,
    pub storage: PathBuf,
    pub fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedPersonMatchCandidatePromotionAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
}

pub fn materialize_reviewed_person_match_candidate_promotion_assembly_v1(
    output: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<
    ReviewedPersonMatchCandidatePromotionAssemblyPathsV1,
    ReviewedPersonMatchCandidatePromotionAssemblyErrorV1,
> {
    materialize_reviewed_person_match_candidate_promotion_assembly_with_runtime_reader_v1(
        output,
        build_id,
        runtime_source,
        |path| fs::read(path),
    )
}

fn materialize_reviewed_person_match_candidate_promotion_assembly_with_runtime_reader_v1<F>(
    output: &Path,
    build_id: &str,
    runtime_source: &Path,
    read_runtime: F,
) -> Result<
    ReviewedPersonMatchCandidatePromotionAssemblyPathsV1,
    ReviewedPersonMatchCandidatePromotionAssemblyErrorV1,
>
where
    F: FnOnce(&Path) -> std::io::Result<Vec<u8>>,
{
    if !output.is_absolute()
        || output.exists()
        || output.parent().is_none()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !fs::symlink_metadata(runtime_source)
            .ok()
            .is_some_and(|m| m.is_file() && !m.file_type().is_symlink() && m.len() > 0)
    {
        return Err(ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidInput);
    }
    let descriptor = reviewed_person_match_candidate_promotion_module_descriptor_v1(build_id);
    let settings = reviewed_person_match_candidate_promotion_settings_schema_v1();
    let storage = reviewed_person_match_candidate_promotion_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = ReviewedPersonMatchCandidatePromotionAssemblyPathsV1 {
        runtime: output.join("makosh-reviewed-person-match-candidate-promotion-runtime"),
        descriptor: output.join("reviewed-person-match-candidate-promotion.runtime.descriptor.pb"),
        settings: output.join("reviewed-person-match-candidate-promotion.runtime.settings.pb"),
        storage: output.join("reviewed-person-match-candidate-promotion.storage.bundle.pb"),
        fragment: output.join("reviewed-person-match-candidate-promotion.release-artifacts.json"),
    };
    let mut artifacts = vec![
        ReviewedPersonMatchCandidatePromotionArtifactV1::ModuleRuntime(
            ModuleRuntimeArtifactInputV1 {
                artifact_id: "reviewed-person-match-candidate-promotion.runtime.v1".into(),
                artifact_kind: "module_runtime".into(),
                relative_path: "bin/makosh-reviewed-person-match-candidate-promotion-runtime"
                    .into(),
                source_path: utf8_path(&paths.runtime)?,
                required: true,
                descriptor: ReleaseContractInputV1 {
                    relative_path:
                        "contracts/reviewed-person-match-candidate-promotion.runtime.descriptor.pb"
                            .into(),
                    source_path: utf8_path(&paths.descriptor)?,
                },
                settings_schema: ReleaseContractInputV1 {
                    relative_path:
                        "contracts/reviewed-person-match-candidate-promotion.runtime.settings.pb"
                            .into(),
                    source_path: utf8_path(&paths.settings)?,
                },
            },
        ),
        ReviewedPersonMatchCandidatePromotionArtifactV1::StorageBundle(
            StorageBundleArtifactInputV1 {
                artifact_id: "reviewed-person-match-candidate-promotion.storage.v1".into(),
                artifact_kind: "storage_bundle".into(),
                relative_path:
                    "storage/reviewed-person-match-candidate-promotion.storage.bundle.pb".into(),
                source_path: utf8_path(&paths.storage)?,
                required: true,
            },
        ),
    ];
    artifacts.sort_by(|a, b| a.artifact_id().cmp(b.artifact_id()));
    let fragment = ReviewedPersonMatchCandidatePromotionFragmentV1 {
        version: 1,
        owner_id: descriptor.owner_id.clone(),
        module_id: descriptor.module_id.clone(),
        artifacts,
    };
    let fragment_bytes = serde_json::to_vec_pretty(&fragment).map_err(|_| {
        ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidCanonicalArtifact
    })?;
    let runtime_bytes = read_runtime(runtime_source)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidInput)?;
    let mut dir = DirBuilder::new();
    dir.mode(0o700);
    dir.create(output)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (&paths.runtime, runtime_bytes),
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings, settings.encode_to_vec()),
        (&paths.storage, storage.encode_to_vec()),
        (&paths.fragment, fragment_bytes),
    ] {
        if write_new(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output);
            return Err(ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn utf8_path(path: &Path) -> Result<String, ReviewedPersonMatchCandidatePromotionAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidInput)
}
fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
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
    fn deterministic_unsigned_fragment_and_failure_cleanup() {
        let root = std::env::temp_dir().join(format!(
            "makosh-reviewed-person-promotion-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let first = root.join("first");
        let second = root.join("second");
        let paths = materialize_reviewed_person_match_candidate_promotion_assembly_v1(
            &first, "build-1", &runtime,
        )
        .expect("assembly");
        let second_paths = materialize_reviewed_person_match_candidate_promotion_assembly_v1(
            &second, "build-1", &runtime,
        )
        .expect("second assembly");
        assert_eq!(
            fs::read(&paths.runtime).expect("staged runtime"),
            b"runtime"
        );
        assert_eq!(
            fs::read(&paths.descriptor).expect("first descriptor"),
            fs::read(&second_paths.descriptor).expect("second descriptor")
        );
        let fragment: ReviewedPersonMatchCandidatePromotionFragmentV1 =
            serde_json::from_slice(&fs::read(paths.fragment).expect("fragment")).expect("typed");
        assert_eq!(
            fragment.owner_id,
            "reviewed_person_match_candidate_promotion"
        );
        assert!(
            fragment
                .artifacts
                .windows(2)
                .all(|p| p[0].artifact_id() < p[1].artifact_id())
        );
        let ReviewedPersonMatchCandidatePromotionArtifactV1::ModuleRuntime(runtime_artifact) =
            &fragment.artifacts[0]
        else {
            panic!("module runtime must be first")
        };
        assert!(Path::new(&runtime_artifact.source_path).is_absolute());
        assert!(Path::new(&runtime_artifact.descriptor.source_path).is_file());
        assert!(Path::new(&runtime_artifact.settings_schema.source_path).is_file());
        assert_eq!(
            materialize_reviewed_person_match_candidate_promotion_assembly_v1(
                &first, "build-1", &runtime
            ),
            Err(ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn runtime_read_failure_never_creates_an_empty_output_directory() {
        let root = std::env::temp_dir().join(format!(
            "makosh-reviewed-person-promotion-read-failure-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let output = root.join("failed");
        assert_eq!(
            materialize_reviewed_person_match_candidate_promotion_assembly_with_runtime_reader_v1(
                &output,
                "build-1",
                &runtime,
                |_| Err(std::io::Error::other("forced runtime read failure")),
            ),
            Err(ReviewedPersonMatchCandidatePromotionAssemblyErrorV1::InvalidInput)
        );
        assert!(!output.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }
}
