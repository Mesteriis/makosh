//! Kernel-inherited process root for the canonical Communications domain.

use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_communications_retained_evidence_replay_persistence::RetainedCommunicationsReplayErrorV1;
use makosh_communications_runtime::{
    admission::{communications_module_descriptor_v1, communications_settings_schema_bytes_v1},
    consumer::CommunicationsDeliveryErrorV1,
    event_runtime::{
        CommunicationsEventRuntimeErrorV1, CommunicationsEventRuntimeV1,
        CommunicationsRuntimeAdmissionV1,
    },
    retained_evidence_replay_result::CommunicationsReplayResultRelayErrorV1,
    storage_bundle::communications_runtime_storage_bundle_v1,
};
use makosh_runtime_protocol::{
    v1::ManagedDomainRuntimeConfigurationV1,
    validation::{
        descriptor::decode_settings_schema_v1,
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
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
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-storage-bundle") => {
            export_storage_bundle(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            export_module_descriptor(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            export_settings_schema(&mut arguments)
        }
        _ => Err("Communications runtime command is unavailable".to_owned()),
    }
}

fn export_storage_bundle<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Communications runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communications_runtime_storage_bundle_v1()
            .map_err(|_| "Communications storage bundle is unavailable".to_owned())?
            .encode_to_vec(),
    )
    .map_err(|_| "Communications storage bundle is unavailable".to_owned())
}

fn export_module_descriptor<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let build_id = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Communications descriptor build id is required".to_owned())?;
    if arguments.next().is_some() {
        return Err("Communications runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communications_module_descriptor_v1(&build_id).encode_to_vec(),
    )
    .map_err(|_| "Communications module descriptor is unavailable".to_owned())
}

fn export_settings_schema<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Communications runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communications_settings_schema_bytes_v1(),
    )
    .map_err(|_| "Communications settings schema is unavailable".to_owned())
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Communications runtime settings schema is invalid".to_owned())?;
    let configuration = ManagedDomainRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Communications runtime configuration is invalid".to_owned())?;
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "Communications runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Communications runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Communications runtime configuration is invalid".to_owned())?;
    let admission = CommunicationsRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Communications runtime executor is unavailable".to_owned())?;
    let control_channel = inherited_control_channel()?;
    let mut runtime = executor
        .block_on(CommunicationsEventRuntimeV1::open(
            control_channel,
            descriptor,
            schema_bytes,
            &admission,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
            storage,
        ))
        .map_err(|error| {
            format!(
                "Communications runtime startup failed: {}",
                runtime_startup_reason_code(error),
            )
        })?;
    let mut maintenance = executor.block_on(async { maintenance_interval() });
    loop {
        executor.block_on(consume_or_tick(&mut runtime, &mut maintenance))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Communications runtime clock is unavailable".to_owned())?;
        let now = i64::try_from(now.as_secs())
            .map_err(|_| "Communications runtime clock is unavailable".to_owned())?;
        if let Err(error) = executor.block_on(runtime.relay_domain_outbox(now)) {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_runtime_outbox_error={error:?}");
            }
            if !error.is_retryable() {
                if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_communications_runtime_outbox_terminal=true");
                }
                return Err("Communications runtime outbox relay failed".to_owned());
            }
        }
        if let Err(error) = executor.block_on(runtime.relay_replay_result(now)) {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_runtime_replay_result_error={error:?}");
            }
            match error {
                CommunicationsReplayResultRelayErrorV1::EventUnavailable
                | CommunicationsReplayResultRelayErrorV1::Persistence(
                    makosh_communications_retained_evidence_replay_persistence::RetainedCommunicationsReplayErrorV1::StorageUnavailable,
                ) => {}
                _ => return Err("Communications replay result relay failed".to_owned()),
            }
        }
    }
}

fn maintenance_interval() -> tokio::time::Interval {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

const fn runtime_startup_reason_code(error: CommunicationsEventRuntimeErrorV1) -> &'static str {
    match error {
        CommunicationsEventRuntimeErrorV1::Admission => "admission_rejected",
        CommunicationsEventRuntimeErrorV1::Unavailable => "dependency_unavailable",
    }
}

const MAX_DERIVED_INDEX_JOBS_PER_MAINTENANCE_TICK: usize = 64;
const MAX_BODY_CUSTODY_TRANSFERS_PER_MAINTENANCE_TICK: usize = 64;

async fn consume_or_tick(
    runtime: &mut CommunicationsEventRuntimeV1,
    maintenance: &mut tokio::time::Interval,
) -> Result<(), String> {
    let client_delivery = runtime
        .try_handle_control_delivery()
        .await
        .map_err(|error| {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_runtime_client_delivery_error={error:?}");
            }
            "Communications runtime client delivery failed".to_owned()
        })?;
    match runtime.publish_call_evidence_realtime().await {
        Ok(_) | Err(CommunicationsEventRuntimeErrorV1::Unavailable) => {}
        Err(CommunicationsEventRuntimeErrorV1::Admission) => {
            return Err("Communications call evidence realtime admission failed".to_owned());
        }
    }
    if client_delivery {
        if tokio::time::timeout(Duration::from_millis(1), maintenance.tick())
            .await
            .is_ok()
        {
            run_maintenance_tick(runtime).await?;
        }
        return Ok(());
    }
    match runtime.consume_next().await {
        Ok(()) => Ok(()),
        Err(CommunicationsDeliveryErrorV1::Unavailable) => {
            tokio::select! {
                _ = maintenance.tick() => run_maintenance_tick(runtime).await,
                _ = tokio::time::sleep(Duration::from_millis(25)) => Ok(()),
            }
        }
        Err(error) => {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_runtime_terminal_delivery_error={error:?}");
            }
            Err("Communications runtime event delivery failed".to_owned())
        }
    }
}

async fn run_maintenance_tick(runtime: &mut CommunicationsEventRuntimeV1) -> Result<(), String> {
    let indexed_at_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .ok_or_else(|| "Communications runtime replay index clock is unavailable".to_owned())?;
    match runtime
        .index_retained_attachment_safety_events(indexed_at_unix_seconds)
        .await
    {
        Ok(_) | Err(RetainedCommunicationsReplayErrorV1::StorageUnavailable) => {}
        Err(error) => {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_runtime_replay_index_error={error:?}");
            }
            return Err("Communications runtime replay index maintenance failed".to_owned());
        }
    }
    let mut custody_processed = 0_usize;
    for _ in 0..MAX_BODY_CUSTODY_TRANSFERS_PER_MAINTENANCE_TICK {
        let processed = runtime
            .process_next_body_custody_transfer()
            .await
            .map_err(|error| maintenance_error("body_custody", error))?;
        if !processed {
            break;
        }
        custody_processed = custody_processed.saturating_add(1);
    }
    if custody_processed > 0 && std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_runtime_custody_processed={custody_processed}");
    }
    runtime
        .reconcile_search_projection_jobs()
        .await
        .map_err(|error| maintenance_error("search_reconcile", error))?;
    for _ in 0..MAX_DERIVED_INDEX_JOBS_PER_MAINTENANCE_TICK {
        let processed = runtime
            .process_next_derived_index_job()
            .await
            .map_err(|error| maintenance_error("search_worker", error))?;
        if !processed {
            break;
        }
    }
    Ok(())
}

fn maintenance_error(stage: &str, error: CommunicationsEventRuntimeErrorV1) -> String {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!(
            "developer_communications_runtime_maintenance_error stage={stage} error={error:?}"
        );
    }
    format!("Communications runtime {stage} maintenance failed")
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("Communications runtime arguments are invalid".to_owned());
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
        return Err("Communications runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Communications runtime arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Communications runtime inherited control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Communications runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Communications runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Communications runtime contract is unavailable".to_owned())
}

#[cfg(test)]
mod maintenance_bounds_tests {
    use super::{
        MAX_BODY_CUSTODY_TRANSFERS_PER_MAINTENANCE_TICK,
        MAX_DERIVED_INDEX_JOBS_PER_MAINTENANCE_TICK, maintenance_interval,
    };

    #[test]
    fn custody_and_index_maintenance_have_independent_exact_bounds() {
        assert_eq!(MAX_BODY_CUSTODY_TRANSFERS_PER_MAINTENANCE_TICK, 64);
        assert_eq!(MAX_DERIVED_INDEX_JOBS_PER_MAINTENANCE_TICK, 64);
    }

    #[tokio::test]
    async fn delayed_maintenance_skips_missed_ticks_instead_of_starving_event_delivery() {
        let interval = maintenance_interval();
        assert_eq!(
            interval.missed_tick_behavior(),
            tokio::time::MissedTickBehavior::Skip,
        );
    }
}
