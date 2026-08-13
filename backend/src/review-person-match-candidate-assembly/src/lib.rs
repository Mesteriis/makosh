#![forbid(unsafe_code)]

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use makosh_review_person_match_candidate_persistence::review_person_match_candidate_storage_bundle_v1;
use makosh_review_person_match_candidate_runtime::{
    review_person_match_candidate_module_descriptor_v1,
    review_person_match_candidate_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const PACKAGE: &str = "makosh-review-person-match-candidate-assembly";
pub const DESCRIPTOR_FILE_V1: &str = "review-person-match-candidate.runtime.descriptor.pb";
pub const SETTINGS_FILE_V1: &str = "review-person-match-candidate.runtime.settings.pb";
pub const STORAGE_FILE_V1: &str = "review-person-match-candidate.storage.bundle.pb";
pub const FRAGMENT_FILE_V1: &str = "review-person-match-candidate.release-artifacts.json";

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
pub enum ReviewPersonMatchCandidateArtifactV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl ReviewPersonMatchCandidateArtifactV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewPersonMatchCandidateArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<ReviewPersonMatchCandidateArtifactV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidateAssemblyPathsV1 {
    pub runtime: PathBuf,
    pub descriptor: PathBuf,
    pub settings: PathBuf,
    pub storage: PathBuf,
    pub fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidateAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
}

pub fn materialize_review_person_match_candidate_assembly_v1(
    output: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<ReviewPersonMatchCandidateAssemblyPathsV1, ReviewPersonMatchCandidateAssemblyErrorV1> {
    materialize_review_person_match_candidate_assembly_with_runtime_reader_v1(
        output,
        build_id,
        runtime_source,
        |path| fs::read(path),
    )
}

fn materialize_review_person_match_candidate_assembly_with_runtime_reader_v1<F>(
    output: &Path,
    build_id: &str,
    runtime_source: &Path,
    read_runtime: F,
) -> Result<ReviewPersonMatchCandidateAssemblyPathsV1, ReviewPersonMatchCandidateAssemblyErrorV1>
where
    F: FnOnce(&Path) -> std::io::Result<Vec<u8>>,
{
    validate_inputs(output, build_id, runtime_source)?;
    let descriptor = review_person_match_candidate_module_descriptor_v1(build_id);
    let settings = review_person_match_candidate_settings_schema_v1();
    let storage = review_person_match_candidate_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(ReviewPersonMatchCandidateAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = ReviewPersonMatchCandidateAssemblyPathsV1 {
        runtime: output.join("makosh-review-person-match-candidate-runtime"),
        descriptor: output.join(DESCRIPTOR_FILE_V1),
        settings: output.join(SETTINGS_FILE_V1),
        storage: output.join(STORAGE_FILE_V1),
        fragment: output.join(FRAGMENT_FILE_V1),
    };
    let mut artifacts = vec![
        ReviewPersonMatchCandidateArtifactV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_id: "review.person-match-candidate.runtime.v1".to_owned(),
            artifact_kind: "module_runtime".to_owned(),
            relative_path: "bin/makosh-review-person-match-candidate-runtime".to_owned(),
            source_path: utf8_path(&paths.runtime)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: "contracts/review-person-match-candidate.runtime.descriptor.pb"
                    .to_owned(),
                source_path: utf8_path(&paths.descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: "contracts/review-person-match-candidate.runtime.settings.pb"
                    .to_owned(),
                source_path: utf8_path(&paths.settings)?,
            },
        }),
        ReviewPersonMatchCandidateArtifactV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_id: "review.person-match-candidate.storage.v1".to_owned(),
            artifact_kind: "storage_bundle".to_owned(),
            relative_path: "storage/review-person-match-candidate.storage.bundle.pb".to_owned(),
            source_path: utf8_path(&paths.storage)?,
            required: true,
        }),
    ];
    artifacts.sort_by(|left, right| left.artifact_id().cmp(right.artifact_id()));
    let fragment = ReviewPersonMatchCandidateArtifactFragmentV1 {
        version: 1,
        owner_id: descriptor.owner_id.clone(),
        module_id: descriptor.module_id.clone(),
        artifacts,
    };
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| ReviewPersonMatchCandidateAssemblyErrorV1::InvalidCanonicalArtifact)?;
    let runtime_bytes = read_runtime(runtime_source)
        .map_err(|_| ReviewPersonMatchCandidateAssemblyErrorV1::InvalidInput)?;
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output)
        .map_err(|_| ReviewPersonMatchCandidateAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (&paths.runtime, runtime_bytes),
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings, settings.encode_to_vec()),
        (&paths.storage, storage.encode_to_vec()),
        (&paths.fragment, fragment_bytes),
    ] {
        if write_new(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output);
            return Err(ReviewPersonMatchCandidateAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn utf8_path(path: &Path) -> Result<String, ReviewPersonMatchCandidateAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(ReviewPersonMatchCandidateAssemblyErrorV1::InvalidInput)
}

fn validate_inputs(
    output: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), ReviewPersonMatchCandidateAssemblyErrorV1> {
    let runtime_ok = fs::symlink_metadata(runtime_source)
        .ok()
        .is_some_and(|metadata| {
            metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
        });
    if !output.is_absolute()
        || output.exists()
        || output.parent().is_none()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !runtime_ok
    {
        Err(ReviewPersonMatchCandidateAssemblyErrorV1::InvalidInput)
    } else {
        Ok(())
    }
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
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(1);

    fn temporary() -> PathBuf {
        std::env::temp_dir().join(format!(
            "makosh-review-person-match-assembly-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ))
    }

    #[test]
    fn assembly_is_deterministic_unsigned_and_cleans_partial_failure() {
        let root = temporary();
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let first = root.join("first");
        let second = root.join("second");
        let first_paths =
            materialize_review_person_match_candidate_assembly_v1(&first, "build-1", &runtime)
                .expect("first");
        let second_paths =
            materialize_review_person_match_candidate_assembly_v1(&second, "build-1", &runtime)
                .expect("second");
        assert_eq!(
            fs::read(&first_paths.runtime).expect("staged runtime"),
            b"runtime"
        );
        assert_eq!(
            fs::read(&first_paths.descriptor).expect("first descriptor"),
            fs::read(&second_paths.descriptor).expect("second descriptor")
        );
        let first_fragment: ReviewPersonMatchCandidateArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(first_paths.fragment).expect("fragment"))
                .expect("typed fragment");
        assert_eq!(first_fragment.owner_id, "review");
        assert_eq!(
            first_fragment.module_id,
            "makosh-review-person-match-candidate-runtime"
        );
        assert!(
            first_fragment
                .artifacts
                .windows(2)
                .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
        );
        let ReviewPersonMatchCandidateArtifactV1::ModuleRuntime(runtime_artifact) =
            &first_fragment.artifacts[0]
        else {
            panic!("module runtime must be first")
        };
        assert!(Path::new(&runtime_artifact.source_path).is_absolute());
        assert!(Path::new(&runtime_artifact.descriptor.source_path).is_file());
        assert!(Path::new(&runtime_artifact.settings_schema.source_path).is_file());
        assert_eq!(
            materialize_review_person_match_candidate_assembly_v1(&first, "build-1", &runtime),
            Err(ReviewPersonMatchCandidateAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn runtime_read_failure_never_creates_an_empty_output_directory() {
        let root = temporary();
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let output = root.join("failed");
        assert_eq!(
            materialize_review_person_match_candidate_assembly_with_runtime_reader_v1(
                &output,
                "build-1",
                &runtime,
                |_| Err(std::io::Error::other("forced runtime read failure")),
            ),
            Err(ReviewPersonMatchCandidateAssemblyErrorV1::InvalidInput)
        );
        assert!(!output.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }
}
