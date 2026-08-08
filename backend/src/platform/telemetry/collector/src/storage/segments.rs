//! Private append-only segment rotation and retention.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_telemetry_protocol::TelemetrySignalV1;

use super::TelemetryRetentionV1;

#[derive(Clone, Debug)]
pub struct TelemetrySegmentStore {
    directory: PathBuf,
    retention: TelemetryRetentionV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryDiagnostics {
    segment_count: u32,
    total_bytes: u64,
}

impl TelemetryDiagnostics {
    #[must_use]
    pub const fn segment_count(self) -> u32 {
        self.segment_count
    }

    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
}

impl TelemetrySegmentStore {
    pub fn open(directory: PathBuf, retention: TelemetryRetentionV1) -> Result<Self, String> {
        ensure_private_directory(&directory)?;
        let retention = TelemetryRetentionV1::new(
            retention.max_segment_bytes(),
            retention.max_total_bytes(),
            retention.max_age_seconds(),
        )?;
        let store = Self {
            directory,
            retention,
        };
        store.prune()?;
        Ok(store)
    }

    pub fn append(&self, signal: &TelemetrySignalV1) -> Result<(), String> {
        let record = encode_record(signal)?;
        let path = self.current_segment(record.len() as u64)?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .mode(0o600)
            .open(path)
            .map_err(|_| "Telemetry segment is unavailable".to_owned())?;
        file.write_all(&record)
            .map_err(|_| "Telemetry segment is unavailable".to_owned())?;
        file.sync_data()
            .map_err(|_| "Telemetry segment is unavailable".to_owned())?;
        self.prune()
    }

    pub fn diagnostics(&self) -> Result<TelemetryDiagnostics, String> {
        let paths = self.segment_paths()?;
        let total_bytes = paths
            .iter()
            .try_fold(0_u64, |total, path| {
                fs::metadata(path).map(|metadata| total.saturating_add(metadata.len()))
            })
            .map_err(|_| "Telemetry segment is unavailable".to_owned())?;
        Ok(TelemetryDiagnostics {
            segment_count: u32::try_from(paths.len())
                .map_err(|_| "Telemetry diagnostics are unavailable".to_owned())?,
            total_bytes,
        })
    }

    fn current_segment(&self, incoming_bytes: u64) -> Result<PathBuf, String> {
        let latest = self.segment_paths()?.pop();
        if let Some(path) = latest {
            let size = fs::metadata(&path)
                .map_err(|_| "Telemetry segment is unavailable".to_owned())?
                .len();
            if size.saturating_add(incoming_bytes) <= self.retention.max_segment_bytes() {
                return Ok(path);
            }
        }
        self.next_segment_path()
    }

    fn next_segment_path(&self) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Telemetry clock is unavailable".to_owned())?
            .as_millis();
        for suffix in 0..=999_u16 {
            let path = self
                .directory
                .join(format!("{millis:020}-{suffix:03}.segment"));
            if !path.exists() {
                return Ok(path);
            }
        }
        Err("Telemetry segment rotation is unavailable".to_owned())
    }

    fn prune(&self) -> Result<(), String> {
        let now = SystemTime::now();
        let mut paths = self.segment_paths()?;
        for path in &paths {
            let modified = fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .map_err(|_| "Telemetry segment is unavailable".to_owned())?;
            if now.duration_since(modified).unwrap_or_default().as_secs()
                > self.retention.max_age_seconds()
            {
                fs::remove_file(path)
                    .map_err(|_| "Telemetry retention is unavailable".to_owned())?;
            }
        }
        paths = self.segment_paths()?;
        let mut total = paths
            .iter()
            .try_fold(0_u64, |sum, path| {
                fs::metadata(path).map(|metadata| sum.saturating_add(metadata.len()))
            })
            .map_err(|_| "Telemetry segment is unavailable".to_owned())?;
        for path in paths {
            if total <= self.retention.max_total_bytes() {
                break;
            }
            let size = fs::metadata(&path)
                .map_err(|_| "Telemetry segment is unavailable".to_owned())?
                .len();
            fs::remove_file(path).map_err(|_| "Telemetry retention is unavailable".to_owned())?;
            total = total.saturating_sub(size);
        }
        Ok(())
    }

    fn segment_paths(&self) -> Result<Vec<PathBuf>, String> {
        let mut paths = fs::read_dir(&self.directory)
            .map_err(|_| "Telemetry directory is unavailable".to_owned())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "segment")
            })
            .collect::<Vec<_>>();
        paths.sort();
        Ok(paths)
    }
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|_| "Telemetry directory is unavailable".to_owned())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Telemetry directory is unavailable".to_owned())?;
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "Telemetry directory is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err("Telemetry directory is unavailable".to_owned());
    }
    Ok(())
}

fn encode_record(signal: &TelemetrySignalV1) -> Result<Vec<u8>, String> {
    let error = signal.error_class().unwrap_or("-");
    let trace = signal.trace_id().unwrap_or("-");
    let record = format!(
        "{}|{:?}|{:?}|{}|{}|{}|{}|{}|{}\n",
        signal.observed_at_utc_millis(),
        signal.kind(),
        signal.priority(),
        signal.source().runtime_id(),
        signal.source().component(),
        signal.operation(),
        error,
        trace,
        signal.dropped_count(),
    );
    if record.len() > 1024 {
        return Err("Telemetry record is too large".to_owned());
    }
    Ok(record.into_bytes())
}
