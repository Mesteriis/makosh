use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use makosh_kernel_control_store::{ControlStore, RecoveryFences};
use sha2::{Digest, Sha256};

const FILE_NAME: &str = ".makosh-recovery-fence-v1";
const MAGIC: &[u8; 8] = b"MAKOSHF1";
const RECORD_BYTES: usize = 81;
const STATE_COMMITTED: u8 = 1;
const STATE_RESERVED: u8 = 2;

#[derive(Clone)]
pub struct RecoveryFenceRecord {
    state: u8,
    instance_id: String,
    fences: RecoveryFences,
    staged_sha256: [u8; 32],
}

pub fn path(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE_NAME)
}

pub fn initialize(
    data_dir: &Path,
    instance_id: &str,
    fences: RecoveryFences,
) -> Result<(), String> {
    let record = RecoveryFenceRecord {
        state: STATE_COMMITTED,
        instance_id: instance_id.to_owned(),
        fences,
        staged_sha256: [0; 32],
    };
    write_record(data_dir, &record)
}

pub fn reserve(
    data_dir: &Path,
    instance_id: &str,
    fences: RecoveryFences,
    staged_sha256: [u8; 32],
) -> Result<RecoveryFenceRecord, String> {
    let record = RecoveryFenceRecord {
        state: STATE_RESERVED,
        instance_id: instance_id.to_owned(),
        fences,
        staged_sha256,
    };
    write_record(data_dir, &record)?;
    Ok(record)
}

pub fn commit(data_dir: &Path, reservation: &RecoveryFenceRecord) -> Result<(), String> {
    let mut committed = reservation.clone();
    committed.state = STATE_COMMITTED;
    committed.staged_sha256 = [0; 32];
    write_record(data_dir, &committed)
}

pub fn read(data_dir: &Path) -> Result<RecoveryFenceRecord, String> {
    let fence_path = path(data_dir);
    validate_file(&fence_path)?;
    let mut bytes = Vec::with_capacity(RECORD_BYTES);
    File::open(&fence_path)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .map_err(|error| error.to_string())?;
    decode(&bytes)
}

pub fn verify_or_finalize(
    data_dir: &Path,
    store_path: &Path,
    store: &ControlStore,
) -> Result<(), String> {
    let record = read(data_dir)?;
    if record.instance_id != store.instance_id() || !matches_store(&record, store) {
        return Err("recovery fence does not match the Control Store".to_owned());
    }
    match record.state {
        STATE_COMMITTED => Ok(()),
        STATE_RESERVED => {
            let digest: [u8; 32] =
                Sha256::digest(std::fs::read(store_path).map_err(|error| error.to_string())?)
                    .into();
            if digest != record.staged_sha256 {
                return Err("reserved recovery fence has no matching staged store".to_owned());
            }
            commit(data_dir, &record)
        }
        _ => Err("recovery fence state is invalid".to_owned()),
    }
}

pub fn verify_committed_source(
    record: &RecoveryFenceRecord,
    store: &ControlStore,
) -> Result<(), String> {
    if record.state != STATE_COMMITTED
        || record.instance_id != store.instance_id()
        || !matches_store(record, store)
    {
        return Err("recovery fence does not match the Control Store".to_owned());
    }
    Ok(())
}

pub fn next(
    record: &RecoveryFenceRecord,
    current: Option<&ControlStore>,
    source: &ControlStore,
) -> Result<RecoveryFences, String> {
    if record.instance_id != source.instance_id()
        || current.is_some_and(|store| store.instance_id() != source.instance_id())
    {
        return Err("recovery fence instance does not match the restore source".to_owned());
    }
    let generation = maximum(
        record.fences.generation(),
        current.map(ControlStore::generation),
        source.generation(),
    )?;
    let identity_epoch = maximum(
        record.fences.identity_epoch(),
        current.map(ControlStore::identity_epoch),
        source.identity_epoch(),
    )?;
    let grant_epoch = maximum(
        record.fences.grant_epoch(),
        current.map(ControlStore::grant_epoch),
        source.grant_epoch(),
    )?;
    Ok(RecoveryFences::new(generation, identity_epoch, grant_epoch))
}

fn maximum(anchor: u64, current: Option<u64>, source: u64) -> Result<u64, String> {
    anchor
        .max(current.unwrap_or(0))
        .max(source)
        .checked_add(1)
        .ok_or_else(|| "recovery fence overflow".to_owned())
}

fn matches_store(record: &RecoveryFenceRecord, store: &ControlStore) -> bool {
    record.fences.generation() == store.generation()
        && record.fences.identity_epoch() == store.identity_epoch()
        && record.fences.grant_epoch() == store.grant_epoch()
}

fn validate_file(fence_path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(fence_path).map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o177 != 0
    {
        return Err("recovery fence must be an owner-private regular file".to_owned());
    }
    Ok(())
}

fn write_record(data_dir: &Path, record: &RecoveryFenceRecord) -> Result<(), String> {
    if decode_instance_id(&record.instance_id).is_none() {
        return Err("recovery fence instance is invalid".to_owned());
    }
    let destination = path(data_dir);
    let temporary = temporary_path(data_dir)?;
    let result = write_temporary(&temporary, &encode(record))
        .and_then(|()| std::fs::rename(&temporary, &destination).map_err(|error| error.to_string()))
        .and_then(|()| {
            File::open(data_dir)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| error.to_string())
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn temporary_path(data_dir: &Path) -> Result<PathBuf, String> {
    for attempt in 0..16 {
        let candidate = data_dir.join(format!(
            ".{FILE_NAME}.{}.{}.tmp",
            std::process::id(),
            attempt
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&candidate)
        {
            Ok(file) => {
                drop(file);
                return Ok(candidate);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("unable to create a recovery fence temporary file".to_owned())
}

fn write_temporary(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

fn encode(record: &RecoveryFenceRecord) -> [u8; RECORD_BYTES] {
    let mut output = [0; RECORD_BYTES];
    output[..8].copy_from_slice(MAGIC);
    output[8] = record.state;
    if let Some(instance_id) = decode_instance_id(&record.instance_id) {
        output[9..25].copy_from_slice(&instance_id);
    }
    output[25..33].copy_from_slice(&record.fences.generation().to_be_bytes());
    output[33..41].copy_from_slice(&record.fences.identity_epoch().to_be_bytes());
    output[41..49].copy_from_slice(&record.fences.grant_epoch().to_be_bytes());
    output[49..81].copy_from_slice(&record.staged_sha256);
    output
}

fn decode(bytes: &[u8]) -> Result<RecoveryFenceRecord, String> {
    if bytes.len() != RECORD_BYTES || &bytes[..8] != MAGIC {
        return Err("recovery fence encoding is invalid".to_owned());
    }
    let instance_bytes: [u8; 16] = bytes[9..25]
        .try_into()
        .map_err(|_| "recovery fence instance is invalid".to_owned())?;
    let generation = u64::from_be_bytes(bytes[25..33].try_into().unwrap_or([0; 8]));
    let identity_epoch = u64::from_be_bytes(bytes[33..41].try_into().unwrap_or([0; 8]));
    let grant_epoch = u64::from_be_bytes(bytes[41..49].try_into().unwrap_or([0; 8]));
    let staged_sha256 = bytes[49..81]
        .try_into()
        .map_err(|_| "recovery fence digest is invalid".to_owned())?;
    if !matches!(bytes[8], STATE_COMMITTED | STATE_RESERVED)
        || generation == 0
        || identity_epoch == 0
        || grant_epoch == 0
    {
        return Err("recovery fence values are invalid".to_owned());
    }
    Ok(RecoveryFenceRecord {
        state: bytes[8],
        instance_id: encode_instance_id(&instance_bytes),
        fences: RecoveryFences::new(generation, identity_epoch, grant_epoch),
        staged_sha256,
    })
}

fn decode_instance_id(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let mut bytes = [0; 16];
    for (index, item) in bytes.iter_mut().enumerate() {
        *item = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

fn encode_instance_id(value: &[u8; 16]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}
