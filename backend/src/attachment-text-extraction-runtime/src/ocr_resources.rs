use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use makosh_attachment_text_extraction_ocr::TesseractOcrConfigurationV1;
use makosh_runtime_protocol::v1::{ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_TEXT_EXTRACTION_OCR_CAPABILITY_ID_V1: &str =
    "attachment_text_extraction.ocr_runtime.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1: &str =
    "attachment_text_extraction.ocr.eng.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1: &str =
    "attachment_text_extraction.ocr.runner.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1: &str =
    "attachment_text_extraction.ocr.rus.v1";

const OCR_WORK_ROOT_V1: &str = "ocr-work-v1";
const OCR_TESSDATA_DIRECTORY_V1: &str = "tessdata";
const OCR_PROCESS_WORK_DIRECTORY_V1: &str = "work";
const OCR_ENGLISH_MODEL_FILE_V1: &str = "eng.traineddata";
const OCR_RUSSIAN_MODEL_FILE_V1: &str = "rus.traineddata";
const OCR_TIMEOUT_MILLIS_V1: u64 = 120_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionOcrResourcesErrorV1 {
    InvalidBindings,
    ArtifactUnavailable,
    WorkRootUnavailable,
}

pub struct PreparedAttachmentTextExtractionOcrResourcesV1 {
    configuration: TesseractOcrConfigurationV1,
    work_root: PathBuf,
}

impl PreparedAttachmentTextExtractionOcrResourcesV1 {
    #[must_use]
    pub fn configuration(&self) -> &TesseractOcrConfigurationV1 {
        &self.configuration
    }
}

impl Drop for PreparedAttachmentTextExtractionOcrResourcesV1 {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.work_root);
    }
}

pub fn prepare_attachment_text_extraction_ocr_resources_v1(
    bindings: &[ManagedRuntimeArtifactBindingV1],
) -> Result<
    PreparedAttachmentTextExtractionOcrResourcesV1,
    AttachmentTextExtractionOcrResourcesErrorV1,
> {
    let [english, runner, russian] = bindings else {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings);
    };
    validate_binding(
        english,
        ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
        RuntimeArtifactUseV1::ReadOnlyData,
    )?;
    validate_binding(
        runner,
        ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
        RuntimeArtifactUseV1::NativeExecutable,
    )?;
    validate_binding(
        russian,
        ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
        RuntimeArtifactUseV1::ReadOnlyData,
    )?;

    let english_path = Path::new(&english.staged_path);
    let runner_path = Path::new(&runner.staged_path);
    let russian_path = Path::new(&russian.staged_path);
    let artifact_root = common_canonical_parent([english_path, runner_path, russian_path])?;
    verify_staged_file(english_path, english, 0o400)?;
    verify_staged_file(runner_path, runner, 0o500)?;
    verify_staged_file(russian_path, russian, 0o400)?;

    let work_root = artifact_root.join(OCR_WORK_ROOT_V1);
    let tessdata = work_root.join(OCR_TESSDATA_DIRECTORY_V1);
    let process_work = work_root.join(OCR_PROCESS_WORK_DIRECTORY_V1);
    create_private_directory(&work_root)?;
    if let Err(error) = create_private_directory(&tessdata)
        .and_then(|()| create_private_directory(&process_work))
        .and_then(|()| {
            copy_verified_model(
                english_path,
                english,
                &tessdata.join(OCR_ENGLISH_MODEL_FILE_V1),
            )
        })
        .and_then(|()| {
            copy_verified_model(
                russian_path,
                russian,
                &tessdata.join(OCR_RUSSIAN_MODEL_FILE_V1),
            )
        })
    {
        let _ = fs::remove_dir_all(&work_root);
        return Err(error);
    }

    Ok(PreparedAttachmentTextExtractionOcrResourcesV1 {
        configuration: TesseractOcrConfigurationV1 {
            executable: runner_path.to_owned(),
            executable_sha256: digest_array(runner)?,
            tessdata_directory: tessdata,
            english_model_sha256: digest_array(english)?,
            russian_model_sha256: digest_array(russian)?,
            private_work_directory: process_work,
            timeout_millis: OCR_TIMEOUT_MILLIS_V1,
        },
        work_root,
    })
}

fn validate_binding(
    binding: &ManagedRuntimeArtifactBindingV1,
    artifact_id: &str,
    use_kind: RuntimeArtifactUseV1,
) -> Result<(), AttachmentTextExtractionOcrResourcesErrorV1> {
    if binding.artifact_id != artifact_id
        || binding.r#use != use_kind as i32
        || binding.size_bytes == 0
        || binding.sha256.len() != 32
        || !binding.sha256.iter().any(|byte| *byte != 0)
        || !Path::new(&binding.staged_path).is_absolute()
    {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings);
    }
    Ok(())
}

fn common_canonical_parent<const N: usize>(
    paths: [&Path; N],
) -> Result<PathBuf, AttachmentTextExtractionOcrResourcesErrorV1> {
    let first_parent = paths[0]
        .parent()
        .ok_or(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings)?;
    let first = first_parent
        .canonicalize()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    for path in paths.iter().skip(1) {
        let declared_parent = path
            .parent()
            .ok_or(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings)?;
        let parent = declared_parent
            .canonicalize()
            .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
        if parent != first {
            return Err(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings);
        }
    }
    Ok(first)
}

fn verify_staged_file(
    path: &Path,
    binding: &ManagedRuntimeArtifactBindingV1,
    expected_mode: u32,
) -> Result<(), AttachmentTextExtractionOcrResourcesErrorV1> {
    let path_metadata = fs::symlink_metadata(path)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    if path_metadata.file_type().is_symlink()
        || !path_metadata.is_file()
        || path_metadata.len() != binding.size_bytes
        || path_metadata.permissions().mode() & 0o777 != expected_mode
    {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable);
    }
    let mut file = File::open(path)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    if !same_file(&path_metadata, &opened) {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable);
    }
    let observed = digest_reader(&mut file)?;
    let opened_after = file
        .metadata()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    let path_after = fs::symlink_metadata(path)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    if !same_file(&opened, &opened_after)
        || !same_file(&opened, &path_after)
        || observed.as_slice() != binding.sha256.as_slice()
    {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable);
    }
    Ok(())
}

fn copy_verified_model(
    source: &Path,
    binding: &ManagedRuntimeArtifactBindingV1,
    destination: &Path,
) -> Result<(), AttachmentTextExtractionOcrResourcesErrorV1> {
    let source_metadata = fs::symlink_metadata(source)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    let mut input = File::open(source)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    let opened = input
        .metadata()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    if !same_file(&source_metadata, &opened) {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable);
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::WorkRootUnavailable)?;
    let mut digest = Sha256::new();
    let mut written = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::WorkRootUnavailable)?;
        digest.update(&buffer[..read]);
        written = written.saturating_add(read as u64);
        if written > binding.size_bytes {
            return Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable);
        }
    }
    output
        .sync_all()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::WorkRootUnavailable)?;
    let opened_after = input
        .metadata()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    let source_after = fs::symlink_metadata(source)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
    if !same_file(&opened, &opened_after)
        || !same_file(&opened, &source_after)
        || written != binding.size_bytes
        || digest.finalize().as_slice() != binding.sha256.as_slice()
    {
        return Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable);
    }
    drop(output);
    fs::set_permissions(destination, fs::Permissions::from_mode(0o400))
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::WorkRootUnavailable)
}

fn create_private_directory(
    path: &Path,
) -> Result<(), AttachmentTextExtractionOcrResourcesErrorV1> {
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::WorkRootUnavailable)
}

fn digest_reader(
    reader: &mut File,
) -> Result<[u8; 32], AttachmentTextExtractionOcrResourcesErrorV1> {
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn digest_array(
    binding: &ManagedRuntimeArtifactBindingV1,
) -> Result<[u8; 32], AttachmentTextExtractionOcrResourcesErrorV1> {
    binding
        .sha256
        .as_slice()
        .try_into()
        .map_err(|_| AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings)
}

fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

#[cfg(test)]
mod tests {
    use std::{
        os::unix::fs::symlink,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_private_models_from_three_verified_bindings() {
        let root = std::env::temp_dir().join(format!(
            "makosh-ocr-bindings-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("test root");
        let bindings = [
            staged_binding(
                &root,
                ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
                b"english-model",
                RuntimeArtifactUseV1::ReadOnlyData,
                0o400,
            ),
            staged_binding(
                &root,
                ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
                b"runner",
                RuntimeArtifactUseV1::NativeExecutable,
                0o500,
            ),
            staged_binding(
                &root,
                ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
                b"russian-model",
                RuntimeArtifactUseV1::ReadOnlyData,
                0o400,
            ),
        ];
        let prepared = prepare_attachment_text_extraction_ocr_resources_v1(&bindings)
            .expect("prepare exact OCR resources");
        assert_eq!(
            fs::read(
                prepared
                    .configuration()
                    .tessdata_directory
                    .join("eng.traineddata")
            )
            .expect("english model"),
            b"english-model"
        );
        assert_eq!(
            fs::metadata(
                prepared
                    .configuration()
                    .tessdata_directory
                    .join("rus.traineddata")
            )
            .expect("russian model")
            .permissions()
            .mode()
                & 0o777,
            0o400
        );
        let work_root = prepared.work_root.clone();
        drop(prepared);
        assert!(!work_root.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rejects_missing_or_mistyped_bindings_before_materialization() {
        let binding = ManagedRuntimeArtifactBindingV1 {
            artifact_id: ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1.to_owned(),
            r#use: RuntimeArtifactUseV1::ReadOnlyData as i32,
            staged_path: "/private/runner".to_owned(),
            size_bytes: 1,
            sha256: vec![1; 32],
        };
        assert!(matches!(
            prepare_attachment_text_extraction_ocr_resources_v1(&[binding]),
            Err(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings)
        ));
    }

    #[test]
    fn rejects_digest_drift_and_symlinked_artifact_parents() {
        let root = std::env::temp_dir().join(format!(
            "makosh-ocr-binding-negatives-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let real = root.join("real");
        fs::create_dir_all(&real).expect("real artifact root");
        let mut bindings = [
            staged_binding(
                &real,
                ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
                b"english-model",
                RuntimeArtifactUseV1::ReadOnlyData,
                0o400,
            ),
            staged_binding(
                &real,
                ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
                b"runner",
                RuntimeArtifactUseV1::NativeExecutable,
                0o500,
            ),
            staged_binding(
                &real,
                ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
                b"russian-model",
                RuntimeArtifactUseV1::ReadOnlyData,
                0o400,
            ),
        ];
        bindings[0].sha256 = vec![9; 32];
        assert!(matches!(
            prepare_attachment_text_extraction_ocr_resources_v1(&bindings),
            Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)
        ));

        bindings[0].sha256 = Sha256::digest(b"english-model").to_vec();
        let linked = root.join("linked");
        fs::create_dir(&linked).expect("linked artifact root");
        for binding in &mut bindings {
            let name = Path::new(&binding.staged_path)
                .file_name()
                .expect("artifact name");
            let linked_path = linked.join(name);
            symlink(Path::new(&binding.staged_path), &linked_path).expect("linked artifact");
            binding.staged_path = linked_path.to_str().expect("linked path").to_owned();
        }
        assert!(matches!(
            prepare_attachment_text_extraction_ocr_resources_v1(&bindings),
            Err(AttachmentTextExtractionOcrResourcesErrorV1::ArtifactUnavailable)
                | Err(AttachmentTextExtractionOcrResourcesErrorV1::InvalidBindings)
        ));
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn staged_binding(
        root: &Path,
        artifact_id: &str,
        bytes: &[u8],
        use_kind: RuntimeArtifactUseV1,
        mode: u32,
    ) -> ManagedRuntimeArtifactBindingV1 {
        let path = root.join(artifact_id);
        fs::write(&path, bytes).expect("staged bytes");
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("staged mode");
        ManagedRuntimeArtifactBindingV1 {
            artifact_id: artifact_id.to_owned(),
            r#use: use_kind as i32,
            staged_path: path.to_str().expect("path").to_owned(),
            size_bytes: bytes.len() as u64,
            sha256: Sha256::digest(bytes).to_vec(),
        }
    }
}
