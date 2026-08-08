#![forbid(unsafe_code)]

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use makosh_attachment_text_extraction_parser_contract::{
    ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1, AttachmentTextParserErrorV1,
    AttachmentTextParserKindV1, AttachmentTextParserOutputV1, bounded_parser_output_v1,
    detect_attachment_text_parser_v1,
};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-ocr";
pub const ATTACHMENT_TEXT_OCR_LANGUAGES_V1: &str = "eng+rus";
const MAX_EXECUTABLE_BYTES_V1: u64 = 256 * 1024 * 1024;
const MAX_MODEL_BYTES_V1: u64 = 128 * 1024 * 1024;
const MAX_STDERR_BYTES_V1: usize = 16 * 1024;
const MAX_TIMEOUT_MILLIS_V1: u64 = 120_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TesseractOcrConfigurationV1 {
    pub executable: PathBuf,
    pub executable_sha256: [u8; 32],
    pub tessdata_directory: PathBuf,
    pub english_model_sha256: [u8; 32],
    pub russian_model_sha256: [u8; 32],
    pub private_work_directory: PathBuf,
    pub timeout_millis: u64,
}

pub fn extract_image_text_v1(
    source: &[u8],
    configuration: &TesseractOcrConfigurationV1,
) -> Result<AttachmentTextParserOutputV1, AttachmentTextParserErrorV1> {
    if detect_attachment_text_parser_v1(source) != Ok(AttachmentTextParserKindV1::Ocr) {
        return Err(AttachmentTextParserErrorV1::InvalidContent);
    }
    let verified = verify_configuration(configuration)?;
    let mut child = Command::new(&verified.executable)
        .args([
            "stdin",
            "stdout",
            "--tessdata-dir",
            verified
                .tessdata_directory
                .to_str()
                .ok_or(AttachmentTextParserErrorV1::ParserUnavailable)?,
            "-l",
            ATTACHMENT_TEXT_OCR_LANGUAGES_V1,
            "--psm",
            "3",
        ])
        .env_clear()
        .env("LANG", "C.UTF-8")
        .current_dir(&verified.private_work_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| AttachmentTextParserErrorV1::ParserUnavailable)?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or(AttachmentTextParserErrorV1::ParserUnavailable)?;
    let source = source.to_vec();
    let writer = thread::spawn(move || stdin.write_all(&source));
    let stdout = child
        .stdout
        .take()
        .ok_or(AttachmentTextParserErrorV1::ParserUnavailable)?;
    let stderr = child
        .stderr
        .take()
        .ok_or(AttachmentTextParserErrorV1::ParserUnavailable)?;
    let output_reader =
        thread::spawn(move || read_bounded(stdout, ATTACHMENT_TEXT_PARSER_MAX_OUTPUT_BYTES_V1));
    let error_reader = thread::spawn(move || read_bounded(stderr, MAX_STDERR_BYTES_V1));

    let deadline = Instant::now() + Duration::from_millis(configuration.timeout_millis);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| AttachmentTextParserErrorV1::ParserFailed)?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = writer.join();
            let _ = output_reader.join();
            let _ = error_reader.join();
            return Err(AttachmentTextParserErrorV1::ParserTimedOut);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let write_result = writer
        .join()
        .map_err(|_| AttachmentTextParserErrorV1::ParserFailed)?;
    let output = output_reader
        .join()
        .map_err(|_| AttachmentTextParserErrorV1::ParserFailed)??;
    let _bounded_stderr = error_reader
        .join()
        .map_err(|_| AttachmentTextParserErrorV1::ParserFailed)??;
    if write_result.is_err() || !status.success() {
        return Err(AttachmentTextParserErrorV1::ParserFailed);
    }
    let normalized = std::str::from_utf8(&output)
        .map_err(|_| AttachmentTextParserErrorV1::InvalidContent)?
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    bounded_parser_output_v1(AttachmentTextParserKindV1::Ocr, normalized.trim(), false)
}

struct VerifiedTesseractConfigurationV1 {
    executable: PathBuf,
    tessdata_directory: PathBuf,
    private_work_directory: PathBuf,
}

fn verify_configuration(
    configuration: &TesseractOcrConfigurationV1,
) -> Result<VerifiedTesseractConfigurationV1, AttachmentTextParserErrorV1> {
    if configuration.timeout_millis == 0
        || configuration.timeout_millis > MAX_TIMEOUT_MILLIS_V1
        || !valid_digest(&configuration.executable_sha256)
        || !valid_digest(&configuration.english_model_sha256)
        || !valid_digest(&configuration.russian_model_sha256)
    {
        return Err(AttachmentTextParserErrorV1::ParserUnavailable);
    }
    let executable = verified_file(
        &configuration.executable,
        configuration.executable_sha256,
        MAX_EXECUTABLE_BYTES_V1,
    )?;
    let tessdata_directory = verified_directory(&configuration.tessdata_directory)?;
    verified_file(
        &tessdata_directory.join("eng.traineddata"),
        configuration.english_model_sha256,
        MAX_MODEL_BYTES_V1,
    )?;
    verified_file(
        &tessdata_directory.join("rus.traineddata"),
        configuration.russian_model_sha256,
        MAX_MODEL_BYTES_V1,
    )?;
    let private_work_directory = verified_directory(&configuration.private_work_directory)?;
    Ok(VerifiedTesseractConfigurationV1 {
        executable,
        tessdata_directory,
        private_work_directory,
    })
}

fn verified_file(
    path: &Path,
    expected_sha256: [u8; 32],
    max_bytes: u64,
) -> Result<PathBuf, AttachmentTextParserErrorV1> {
    if !path.is_absolute() {
        return Err(AttachmentTextParserErrorV1::ParserUnavailable);
    }
    let canonical =
        fs::canonicalize(path).map_err(|_| AttachmentTextParserErrorV1::ParserUnavailable)?;
    let metadata =
        fs::metadata(&canonical).map_err(|_| AttachmentTextParserErrorV1::ParserUnavailable)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(AttachmentTextParserErrorV1::ParserUnavailable);
    }
    let mut file =
        File::open(&canonical).map_err(|_| AttachmentTextParserErrorV1::ParserUnavailable)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| AttachmentTextParserErrorV1::ParserUnavailable)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let observed: [u8; 32] = hasher.finalize().into();
    if observed != expected_sha256 {
        return Err(AttachmentTextParserErrorV1::ParserUnavailable);
    }
    Ok(canonical)
}

fn verified_directory(path: &Path) -> Result<PathBuf, AttachmentTextParserErrorV1> {
    if !path.is_absolute() {
        return Err(AttachmentTextParserErrorV1::ParserUnavailable);
    }
    let canonical =
        fs::canonicalize(path).map_err(|_| AttachmentTextParserErrorV1::ParserUnavailable)?;
    if !canonical.is_dir() {
        return Err(AttachmentTextParserErrorV1::ParserUnavailable);
    }
    Ok(canonical)
}

fn read_bounded(reader: impl Read, maximum: usize) -> Result<Vec<u8>, AttachmentTextParserErrorV1> {
    let mut output = Vec::new();
    reader
        .take(maximum as u64 + 1)
        .read_to_end(&mut output)
        .map_err(|_| AttachmentTextParserErrorV1::ParserFailed)?;
    if output.len() > maximum {
        return Err(AttachmentTextParserErrorV1::OutputTooLarge);
    }
    Ok(output)
}

fn valid_digest(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_or_unpinned_configuration_never_spawns_a_parser() {
        let configuration = TesseractOcrConfigurationV1 {
            executable: PathBuf::from("tesseract"),
            executable_sha256: [0; 32],
            tessdata_directory: PathBuf::from("tessdata"),
            english_model_sha256: [0; 32],
            russian_model_sha256: [0; 32],
            private_work_directory: PathBuf::from("work"),
            timeout_millis: 30_000,
        };
        assert_eq!(
            extract_image_text_v1(b"\x89PNG\r\n\x1a\ninvalid", &configuration),
            Err(AttachmentTextParserErrorV1::ParserUnavailable)
        );
    }

    #[test]
    fn non_image_bytes_are_rejected_before_configuration_access() {
        let configuration = TesseractOcrConfigurationV1 {
            executable: PathBuf::new(),
            executable_sha256: [0; 32],
            tessdata_directory: PathBuf::new(),
            english_model_sha256: [0; 32],
            russian_model_sha256: [0; 32],
            private_work_directory: PathBuf::new(),
            timeout_millis: 0,
        };
        assert_eq!(
            extract_image_text_v1(b"plain", &configuration),
            Err(AttachmentTextParserErrorV1::InvalidContent)
        );
    }
}
