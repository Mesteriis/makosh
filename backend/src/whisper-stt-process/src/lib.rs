#![forbid(unsafe_code)]

use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use makosh_speech_to_text_api::wire::{SpeechLanguageV1, SpeechTranscriptCompletenessV1};
use makosh_whisper_stt_core::{
    WhisperSttExecutionOutcomeV1, WhisperSttExecutionPlanV1, WhisperSttTranscriptSegmentV1,
};
use serde::Deserialize;

pub const PACKAGE: &str = "makosh-whisper-stt-process";
const WHISPER_JSON_OVERHEAD_BYTES_V1: usize = 256 * 1024;
const PROCESS_POLL_INTERVAL_V1: Duration = Duration::from_millis(10);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttProcessConfigurationV1 {
    pub executable: PathBuf,
    pub model: PathBuf,
    pub private_work_root: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttProcessErrorV1 {
    InvalidConfiguration,
    InvalidAudio,
    WorkUnavailable,
    SpawnFailed,
    TimedOut,
    ProcessRejected,
    OutputUnavailable,
    InvalidOutput,
}

pub fn execute_whisper_stt_process_v1(
    configuration: &WhisperSttProcessConfigurationV1,
    plan: &WhisperSttExecutionPlanV1,
    audio: &[u8],
) -> Result<WhisperSttExecutionOutcomeV1, WhisperSttProcessErrorV1> {
    validate_configuration(configuration)?;
    validate_audio(plan, audio)?;
    let work =
        PrivateWorkDirectoryV1::create(&configuration.private_work_root, &plan.request.request_id)?;
    let audio_path = work.path().join("audio.wav");
    let output_prefix = work.path().join("transcript");
    write_private_file(&audio_path, audio)?;

    let mut command = Command::new(&configuration.executable);
    command
        .arg("--model")
        .arg(&configuration.model)
        .arg("--file")
        .arg(&audio_path)
        .arg("--threads")
        .arg(plan.thread_count.to_string())
        .arg("--language")
        .arg(language_argument(plan.request.requested_language)?)
        .arg("--output-json")
        .arg("--output-file")
        .arg(&output_prefix)
        .arg("--no-prints")
        .current_dir(work.path())
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|_| WhisperSttProcessErrorV1::SpawnFailed)?;
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(plan.timeout_millis))
        .ok_or(WhisperSttProcessErrorV1::InvalidConfiguration)?;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| WhisperSttProcessErrorV1::ProcessRejected)?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(WhisperSttProcessErrorV1::TimedOut);
        }
        std::thread::sleep(PROCESS_POLL_INTERVAL_V1);
    };
    if !status.success() {
        return Err(WhisperSttProcessErrorV1::ProcessRejected);
    }
    let maximum_json_bytes = usize::try_from(plan.request.maximum_transcript_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(WHISPER_JSON_OVERHEAD_BYTES_V1);
    let json = read_bounded(&output_prefix.with_extension("json"), maximum_json_bytes)?;
    parse_whisper_json_v1(&json, plan)
}

fn parse_whisper_json_v1(
    json: &[u8],
    plan: &WhisperSttExecutionPlanV1,
) -> Result<WhisperSttExecutionOutcomeV1, WhisperSttProcessErrorV1> {
    let output: WhisperJsonV1 =
        serde_json::from_slice(json).map_err(|_| WhisperSttProcessErrorV1::InvalidOutput)?;
    let detected_language = match output.result.language.as_str() {
        "en" => SpeechLanguageV1::English,
        "ru" => SpeechLanguageV1::Russian,
        "es" => SpeechLanguageV1::Spanish,
        _ => return Err(WhisperSttProcessErrorV1::InvalidOutput),
    };
    if output.transcription.is_empty()
        || output.transcription.len()
            > usize::try_from(plan.request.maximum_segments).unwrap_or(usize::MAX)
    {
        return Err(WhisperSttProcessErrorV1::InvalidOutput);
    }
    let segments = output
        .transcription
        .into_iter()
        .map(|segment| {
            if segment.offsets.to <= segment.offsets.from
                || segment.offsets.to > plan.request.duration_millis
                || segment.text.is_empty()
                || segment
                    .text
                    .as_bytes()
                    .iter()
                    .all(|byte| byte.is_ascii_whitespace())
            {
                return Err(WhisperSttProcessErrorV1::InvalidOutput);
            }
            Ok(WhisperSttTranscriptSegmentV1 {
                start_millis: segment.offsets.from,
                end_millis: segment.offsets.to,
                content_utf8: segment.text.into_bytes(),
            })
        })
        .collect::<Result<Vec<_>, WhisperSttProcessErrorV1>>()?;
    Ok(WhisperSttExecutionOutcomeV1 {
        detected_language,
        segments,
        completeness: SpeechTranscriptCompletenessV1::Complete,
        confidence_basis_points: 0,
    })
}

fn validate_configuration(
    configuration: &WhisperSttProcessConfigurationV1,
) -> Result<(), WhisperSttProcessErrorV1> {
    if !regular_absolute_file(&configuration.executable)
        || !regular_absolute_file(&configuration.model)
        || !configuration.private_work_root.is_absolute()
        || !configuration.private_work_root.is_dir()
        || fs::symlink_metadata(&configuration.private_work_root)
            .ok()
            .is_none_or(|metadata| metadata.file_type().is_symlink())
    {
        return Err(WhisperSttProcessErrorV1::InvalidConfiguration);
    }
    Ok(())
}

fn validate_audio(
    plan: &WhisperSttExecutionPlanV1,
    audio: &[u8],
) -> Result<(), WhisperSttProcessErrorV1> {
    let source = plan
        .request
        .source
        .as_ref()
        .ok_or(WhisperSttProcessErrorV1::InvalidAudio)?;
    if audio.len() as u64 != source.declared_bytes
        || audio.len() < 44
        || &audio[..4] != b"RIFF"
        || &audio[8..12] != b"WAVE"
    {
        return Err(WhisperSttProcessErrorV1::InvalidAudio);
    }
    Ok(())
}

fn language_argument(value: i32) -> Result<&'static str, WhisperSttProcessErrorV1> {
    match SpeechLanguageV1::try_from(value) {
        Ok(SpeechLanguageV1::Auto) => Ok("auto"),
        Ok(SpeechLanguageV1::English) => Ok("en"),
        Ok(SpeechLanguageV1::Russian) => Ok("ru"),
        Ok(SpeechLanguageV1::Spanish) => Ok("es"),
        _ => Err(WhisperSttProcessErrorV1::InvalidAudio),
    }
}

fn regular_absolute_file(path: &Path) -> bool {
    path.is_absolute()
        && fs::symlink_metadata(path).ok().is_some_and(|metadata| {
            metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
        })
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), WhisperSttProcessErrorV1> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| WhisperSttProcessErrorV1::WorkUnavailable)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| WhisperSttProcessErrorV1::WorkUnavailable)
}

fn read_bounded(path: &Path, maximum: usize) -> Result<Vec<u8>, WhisperSttProcessErrorV1> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| WhisperSttProcessErrorV1::OutputUnavailable)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum as u64
    {
        return Err(WhisperSttProcessErrorV1::OutputUnavailable);
    }
    let file = File::open(path).map_err(|_| WhisperSttProcessErrorV1::OutputUnavailable)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((maximum as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| WhisperSttProcessErrorV1::OutputUnavailable)?;
    if bytes.len() > maximum {
        return Err(WhisperSttProcessErrorV1::OutputUnavailable);
    }
    Ok(bytes)
}

struct PrivateWorkDirectoryV1 {
    path: PathBuf,
}

impl PrivateWorkDirectoryV1 {
    fn create(root: &Path, request_id: &[u8]) -> Result<Self, WhisperSttProcessErrorV1> {
        if request_id.len() != 16 {
            return Err(WhisperSttProcessErrorV1::WorkUnavailable);
        }
        let name = request_id
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let path = root.join(name);
        let mut builder = DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(&path)
            .map_err(|_| WhisperSttProcessErrorV1::WorkUnavailable)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for PrivateWorkDirectoryV1 {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Deserialize)]
struct WhisperJsonV1 {
    result: WhisperJsonResultV1,
    transcription: Vec<WhisperJsonSegmentV1>,
}

#[derive(Deserialize)]
struct WhisperJsonResultV1 {
    language: String,
}

#[derive(Deserialize)]
struct WhisperJsonSegmentV1 {
    offsets: WhisperJsonOffsetsV1,
    text: String,
}

#[derive(Deserialize)]
struct WhisperJsonOffsetsV1 {
    from: u64,
    to: u64,
}

#[cfg(test)]
mod tests {
    use std::{
        os::unix::fs::{PermissionsExt, symlink},
        sync::atomic::{AtomicU64, Ordering},
    };

    use makosh_speech_to_text_api::{
        seal_speech_to_text_request_v1,
        wire::{SpeechAudioFormatV1, SpeechAudioSourceReceiptV1, SpeechToTextRequestV1},
    };
    use makosh_whisper_stt_core::plan_whisper_stt_execution_v1;

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    fn plan() -> WhisperSttExecutionPlanV1 {
        let request = seal_speech_to_text_request_v1(SpeechToTextRequestV1 {
            protocol_major: 0,
            request_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source: Some(SpeechAudioSourceReceiptV1 {
                reference_id: vec![2; 16],
                declared_bytes: 44,
                sha256: vec![3; 32],
                custody_transfer_source_proof: vec![4; 32],
            }),
            audio_format: SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32,
            duration_millis: 2_000,
            requested_language: SpeechLanguageV1::Auto as i32,
            consent_receipt_id: vec![5; 16],
            consent_policy_revision: 1,
            maximum_transcript_bytes: 64 * 1024,
            maximum_segments: 8,
            request_digest: Vec::new(),
        })
        .expect("request");
        plan_whisper_stt_execution_v1(request, [6; 32], 1, 4, 30_000).expect("plan")
    }

    #[test]
    fn parses_only_bounded_supported_language_segments() {
        let json = br#"{
          "systeminfo":"private ignored diagnostics",
          "result":{"language":"en"},
          "transcription":[
            {"timestamps":{"from":"00:00:00,000","to":"00:00:01,000"},
             "offsets":{"from":0,"to":1000},"text":" hello"},
            {"offsets":{"from":1000,"to":2000},"text":" world"}
          ]
        }"#;
        let outcome = parse_whisper_json_v1(json, &plan()).expect("outcome");
        assert_eq!(outcome.detected_language, SpeechLanguageV1::English);
        assert_eq!(outcome.segments.len(), 2);

        let unsupported = br#"{"result":{"language":"de"},"transcription":[{"offsets":{"from":0,"to":1},"text":"x"}]}"#;
        assert_eq!(
            parse_whisper_json_v1(unsupported, &plan()),
            Err(WhisperSttProcessErrorV1::InvalidOutput)
        );
    }

    #[test]
    fn exact_cli_uses_no_shell_or_inherited_environment() {
        let source = include_str!("lib.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(production.contains("Command::new(&configuration.executable)"));
        assert!(production.contains(".env_clear()"));
        assert!(production.contains("--output-json"));
        assert!(!production.contains("Command::new(\"sh\")"));
        assert!(!production.contains(".arg(\"-c\")"));
    }

    #[test]
    fn real_process_output_is_parsed_and_private_work_is_removed() {
        let fixture = process_fixture(
            "while [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--output-file\" ]; then shift; output=$1; fi\n  shift\ndone\nprintf '%s' '{\"result\":{\"language\":\"en\"},\"transcription\":[{\"offsets\":{\"from\":0,\"to\":1000},\"text\":\"hello\"}]}' > \"${output}.json\"\n",
        );
        let outcome = execute_whisper_stt_process_v1(&fixture.configuration, &plan(), &wav())
            .expect("process outcome");
        assert_eq!(outcome.segments.len(), 1);
        assert_eq!(
            fs::read_dir(&fixture.configuration.private_work_root)
                .expect("work root")
                .count(),
            0
        );
    }

    #[test]
    fn malformed_output_and_non_zero_exit_are_rejected_and_cleaned() {
        let malformed = process_fixture(
            "while [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--output-file\" ]; then shift; output=$1; fi\n  shift\ndone\nprintf '%s' 'not-json' > \"${output}.json\"\n",
        );
        assert_eq!(
            execute_whisper_stt_process_v1(&malformed.configuration, &plan(), &wav()),
            Err(WhisperSttProcessErrorV1::InvalidOutput)
        );
        assert_eq!(
            fs::read_dir(&malformed.configuration.private_work_root)
                .expect("work root")
                .count(),
            0
        );

        let rejected = process_fixture("exit 7\n");
        assert_eq!(
            execute_whisper_stt_process_v1(&rejected.configuration, &plan(), &wav()),
            Err(WhisperSttProcessErrorV1::ProcessRejected)
        );
        assert_eq!(
            fs::read_dir(&rejected.configuration.private_work_root)
                .expect("work root")
                .count(),
            0
        );
    }

    #[test]
    fn timeout_kills_process_and_symlinked_runner_is_never_started() {
        let hanging = process_fixture("while :; do :; done\n");
        let mut short = plan();
        short.timeout_millis = 1_000;
        assert_eq!(
            execute_whisper_stt_process_v1(&hanging.configuration, &short, &wav()),
            Err(WhisperSttProcessErrorV1::TimedOut)
        );
        assert_eq!(
            fs::read_dir(&hanging.configuration.private_work_root)
                .expect("work root")
                .count(),
            0
        );

        let linked = process_fixture("exit 0\n");
        let symlink_path = linked.root.join("linked-runner");
        symlink(&linked.configuration.executable, &symlink_path).expect("symlink");
        let mut configuration = linked.configuration.clone();
        configuration.executable = symlink_path;
        assert_eq!(
            execute_whisper_stt_process_v1(&configuration, &plan(), &wav()),
            Err(WhisperSttProcessErrorV1::InvalidConfiguration)
        );
    }

    fn wav() -> Vec<u8> {
        let mut bytes = vec![0_u8; 44];
        bytes[..4].copy_from_slice(b"RIFF");
        bytes[8..12].copy_from_slice(b"WAVE");
        bytes
    }

    struct ProcessFixtureV1 {
        root: PathBuf,
        configuration: WhisperSttProcessConfigurationV1,
    }

    impl Drop for ProcessFixtureV1 {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn process_fixture(body: &str) -> ProcessFixtureV1 {
        let root = std::env::temp_dir().join(format!(
            "makosh-whisper-process-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("root");
        let executable = root.join("runner");
        fs::write(&executable, format!("#!/bin/sh\n{body}")).expect("runner");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o500)).expect("runner mode");
        let model = root.join("model");
        fs::write(&model, b"model").expect("model");
        fs::set_permissions(&model, fs::Permissions::from_mode(0o400)).expect("model mode");
        let work = root.join("work");
        fs::create_dir(&work).expect("work");
        fs::set_permissions(&work, fs::Permissions::from_mode(0o700)).expect("work mode");
        ProcessFixtureV1 {
            root,
            configuration: WhisperSttProcessConfigurationV1 {
                executable,
                model,
                private_work_root: work,
            },
        }
    }
}
