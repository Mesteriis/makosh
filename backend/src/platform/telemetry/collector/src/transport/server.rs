//! One bounded telemetry record per private local connection.

use std::io::Read;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

use makosh_telemetry_protocol::{
    TelemetryPriorityV1, TelemetrySignalKindV1, TelemetrySignalV1, TelemetrySourceV1,
};

use crate::storage::TelemetrySegmentStore;

use super::quota::TelemetryQuotaV1;

const SOCKET_NAME: &str = "telemetry.sock";
const MAX_FRAME_BYTES: usize = 1024;

pub fn serve(runtime_dir: &Path, store: TelemetrySegmentStore) -> Result<(), String> {
    serve_listener(runtime_dir, store)
}

pub fn serve_with_control(
    runtime_dir: &Path,
    store: TelemetrySegmentStore,
    control: UnixStream,
) -> Result<(), String> {
    let control_store = store.clone();
    std::thread::spawn(move || {
        let _ = crate::control::serve_diagnostics(control, control_store);
    });
    serve_listener(runtime_dir, store)
}

fn serve_listener(runtime_dir: &Path, store: TelemetrySegmentStore) -> Result<(), String> {
    ensure_private_runtime_directory(runtime_dir)?;
    let socket = runtime_dir.join(SOCKET_NAME);
    remove_owned_socket(&socket)?;
    let listener =
        UnixListener::bind(&socket).map_err(|_| "Telemetry listener is unavailable".to_owned())?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "Telemetry listener is unavailable".to_owned())?;
    let mut quota = TelemetryQuotaV1::default();
    for stream in listener.incoming().flatten() {
        if let Ok(signal) = ingest(stream)
            && quota.admit(&signal)
        {
            let _ = store.append(&signal);
        }
    }
    Err("Telemetry listener stopped".to_owned())
}

fn ensure_private_runtime_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|_| "Telemetry runtime directory is unavailable".to_owned())?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| "Telemetry runtime directory is unavailable".to_owned())?;
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Telemetry runtime directory is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err("Telemetry runtime directory is unavailable".to_owned());
    }
    Ok(())
}

fn remove_owned_socket(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Telemetry socket is unavailable".to_owned())?;
    if metadata.file_type().is_socket() {
        return std::fs::remove_file(path)
            .map_err(|_| "Telemetry socket is unavailable".to_owned());
    }
    Err("Telemetry socket is unavailable".to_owned())
}

fn ingest(mut stream: UnixStream) -> Result<TelemetrySignalV1, String> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .map_err(|_| "Telemetry frame is unavailable".to_owned())?;
    let mut buffer = vec![0_u8; MAX_FRAME_BYTES + 1];
    let size = stream
        .read(&mut buffer)
        .map_err(|_| "Telemetry frame is unavailable".to_owned())?;
    if size == 0 || size > MAX_FRAME_BYTES {
        return Err("Telemetry frame is invalid".to_owned());
    }
    let frame = std::str::from_utf8(&buffer[..size])
        .map_err(|_| "Telemetry frame is invalid".to_owned())?;
    decode(frame.trim_end_matches('\n'))
}

fn decode(frame: &str) -> Result<TelemetrySignalV1, String> {
    let fields = frame.split('|').collect::<Vec<_>>();
    let [
        timestamp,
        kind,
        priority,
        runtime,
        component,
        operation,
        error,
        trace,
        dropped,
    ] = fields.as_slice()
    else {
        return Err("Telemetry frame is invalid".to_owned());
    };
    TelemetrySignalV1::new(makosh_telemetry_protocol::TelemetrySignalInputV1 {
        observed_at_utc_millis: timestamp
            .parse()
            .map_err(|_| "Telemetry frame is invalid".to_owned())?,
        source: TelemetrySourceV1::new((*runtime).to_owned(), (*component).to_owned())
            .map_err(|_| "Telemetry frame is invalid".to_owned())?,
        kind: parse_kind(kind)?,
        priority: parse_priority(priority)?,
        operation: (*operation).to_owned(),
        error_class: optional(error),
        trace_id: optional(trace),
        dropped_count: dropped
            .parse()
            .map_err(|_| "Telemetry frame is invalid".to_owned())?,
    })
    .map_err(|_| "Telemetry frame is invalid".to_owned())
}

fn optional(value: &str) -> Option<String> {
    (value != "-").then(|| value.to_owned())
}

fn parse_kind(value: &str) -> Result<TelemetrySignalKindV1, String> {
    match value {
        "Log" => Ok(TelemetrySignalKindV1::Log),
        "Metric" => Ok(TelemetrySignalKindV1::Metric),
        "Trace" => Ok(TelemetrySignalKindV1::Trace),
        "Lifecycle" => Ok(TelemetrySignalKindV1::Lifecycle),
        _ => Err("Telemetry frame is invalid".to_owned()),
    }
}

fn parse_priority(value: &str) -> Result<TelemetryPriorityV1, String> {
    match value {
        "Debug" => Ok(TelemetryPriorityV1::Debug),
        "Info" => Ok(TelemetryPriorityV1::Info),
        "Warning" => Ok(TelemetryPriorityV1::Warning),
        "Error" => Ok(TelemetryPriorityV1::Error),
        "Crash" => Ok(TelemetryPriorityV1::Crash),
        _ => Err("Telemetry frame is invalid".to_owned()),
    }
}
