use std::{
    ffi::{OsStr, OsString},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    time::Duration,
};

use makosh_attachment_text_extraction_api::ATTACHMENT_TEXT_EXTRACTION_OWNER_V1;
use makosh_attachment_text_extraction_persistence::attachment_text_extraction_storage_bundle_v1;
use makosh_attachment_text_extraction_runtime::{
    AttachmentTextExtractionParserRuntimeV1, attachment_text_extraction_module_descriptor_v1,
    attachment_text_extraction_settings_schema_bytes_v1,
    prepare_attachment_text_extraction_ocr_resources_v1,
    runtime::{
        AttachmentTextExtractionManagedRuntimeV1, AttachmentTextExtractionRuntimeAdmissionV1,
        current_runtime_time_v1,
    },
};
use makosh_runtime_protocol::{
    v1::ManagedWorkflowRuntimeConfigurationV1,
    validation::{
        descriptor::decode_settings_schema_v1,
        managed_workflow_runtime::validate_managed_workflow_runtime_configuration,
    },
};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("export-storage-bundle") => {
            no_trailing(&mut arguments)?;
            write_stdout(
                &attachment_text_extraction_storage_bundle_v1().encode_to_vec(),
                "storage bundle",
            )
        }
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            let build_id = arguments
                .next()
                .and_then(|value| value.into_string().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Attachment Text Extraction build id is required".to_owned())?;
            no_trailing(&mut arguments)?;
            write_stdout(
                &attachment_text_extraction_module_descriptor_v1(&build_id).encode_to_vec(),
                "descriptor",
            )
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            no_trailing(&mut arguments)?;
            write_stdout(
                &attachment_text_extraction_settings_schema_bytes_v1(),
                "settings schema",
            )
        }
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(parse_paths(&mut arguments.peekable())?)
        }
        _ => Err("Attachment Text Extraction runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited(paths: InheritedPaths) -> Result<(), String> {
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Attachment Text Extraction settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Attachment Text Extraction runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Attachment Text Extraction runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Attachment Text Extraction runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Attachment Text Extraction storage configuration is missing".to_owned())?;
    let admission = AttachmentTextExtractionRuntimeAdmissionV1 {
        module_owner_id: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
        logical_human_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Attachment Text Extraction executor is unavailable".to_owned())?;
    let ocr_resources =
        prepare_attachment_text_extraction_ocr_resources_v1(&configuration.runtime_artifacts)
            .map_err(|_| {
                "Attachment Text Extraction OCR runtime resources are unavailable".to_owned()
            })?;
    let parser =
        AttachmentTextExtractionParserRuntimeV1::new(Some(ocr_resources.configuration().clone()));
    let mut runtime = executor
        .block_on(AttachmentTextExtractionManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            schema_bytes,
            &admission,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
            storage,
            parser,
        ))
        .map_err(|error| {
            diagnostic("startup", error);
            "Attachment Text Extraction runtime admission was rejected".to_owned()
        })?;
    let mut interval =
        executor.block_on(async { tokio::time::interval(Duration::from_millis(250)) });
    loop {
        executor.block_on(interval.tick());
        let (now_millis, nanos) = current_runtime_time_v1()
            .map_err(|_| "Attachment Text Extraction clock is unavailable".to_owned())?;
        for (stage, result) in [
            (
                "client-delivery",
                executor.block_on(runtime.pump_control_once(now_millis)),
            ),
            (
                "consume",
                executor.block_on(runtime.consume_next(now_millis)),
            ),
        ] {
            if let Err(error) = result {
                diagnostic(stage, error);
            }
        }
        if let Err(error) =
            executor.block_on(runtime.materialize_pending_custody_requests(now_millis, nanos))
        {
            diagnostic("custody-materialize", error);
        }
        if let Err(error) = executor.block_on(runtime.relay_custody_outbox(now_millis)) {
            diagnostic("custody-outbox", error);
        }
        if let Err(error) = executor.block_on(runtime.relay_translation_source_outbox(now_millis)) {
            diagnostic("translation-source-outbox", error);
        }
        if let Err(error) = executor.block_on(runtime.process_next_job(now_millis)) {
            diagnostic("extract", error);
        }
        if let Err(error) = executor.block_on(runtime.pump_client_realtime_once()) {
            diagnostic("client-realtime", error);
        }
    }
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() {
        return Err("Attachment Text Extraction runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
        runtime_configuration,
        runtime_instance_id,
    })
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    required_string(arguments, name).map(PathBuf::from)
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Attachment Text Extraction runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Attachment Text Extraction runtime arguments are invalid".to_owned())
}

fn no_trailing<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Attachment Text Extraction runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Attachment Text Extraction control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Attachment Text Extraction contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_BYTES
    {
        return Err("Attachment Text Extraction contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Attachment Text Extraction contract is unavailable".to_owned())
}

fn write_stdout(bytes: &[u8], artifact: &str) -> Result<(), String> {
    std::io::Write::write_all(&mut std::io::stdout(), bytes)
        .map_err(|_| format!("Attachment Text Extraction {artifact} is unavailable"))
}

fn diagnostic(
    stage: &str,
    error: makosh_attachment_text_extraction_runtime::runtime::AttachmentTextExtractionRuntimeErrorV1,
) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!(
            "developer_attachment_text_extraction_runtime_error stage={stage} error={error:?}"
        );
    }
}
