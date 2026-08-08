//! Crash-safe idempotency ledger for Blob custody release reservations.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::Mutex,
};

use makosh_blob_protocol::{BlobAccessFenceV1, BlobCustodyScopeV1, BlobRefV1};
use sha2::{Digest, Sha256};

use crate::{metadata::BlobMetadataLedger, storage::root};

const MAGIC: &[u8; 8] = b"HBRLSV1\0";
const RECORD_BYTES: usize = 8 + 16 + 32 + 8 + 1 + 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobCustodyReleaseOutcomeV1 {
    Accepted,
    Existing,
    AlreadyReleased,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobCustodyReleaseErrorV1 {
    InvalidInput,
    Conflict,
    Unavailable,
}

pub struct BlobCustodyReleaseRequestV1<'a> {
    pub operation_id: [u8; 16],
    pub fingerprint: [u8; 32],
    pub reference: &'a BlobRefV1,
    pub access: &'a BlobAccessFenceV1,
    pub custody: &'a BlobCustodyScopeV1,
    pub not_before_unix_ms: u64,
}

pub struct BlobCustodyReleaseLedgerV1 {
    root: PathBuf,
    lock: Mutex<()>,
}

impl BlobCustodyReleaseLedgerV1 {
    pub fn open(data_dir: &Path) -> Result<Self, BlobCustodyReleaseErrorV1> {
        let root = root::prepare_release_root(data_dir)
            .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
        recover_staged(&root)?;
        Ok(Self {
            root,
            lock: Mutex::new(()),
        })
    }

    pub fn reserve(
        &self,
        metadata: &BlobMetadataLedger,
        request: BlobCustodyReleaseRequestV1<'_>,
    ) -> Result<BlobCustodyReleaseOutcomeV1, BlobCustodyReleaseErrorV1> {
        if request.operation_id.iter().all(|byte| *byte == 0)
            || request.fingerprint.iter().all(|byte| *byte == 0)
            || request.not_before_unix_ms == 0
        {
            return Err(BlobCustodyReleaseErrorV1::InvalidInput);
        }
        let _guard = self
            .lock
            .lock()
            .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
        let path = self.record_path(request.reference.reference_id());
        let existing = read_record(&path)?;
        let (record, outcome) = match existing {
            Some(record)
                if record.operation_id == request.operation_id
                    && record.fingerprint == request.fingerprint
                    && record.not_before_unix_ms == request.not_before_unix_ms =>
            {
                (record, BlobCustodyReleaseOutcomeV1::Existing)
            }
            Some(record) if record.operation_id == request.operation_id => {
                return Err(BlobCustodyReleaseErrorV1::Conflict);
            }
            Some(record) => {
                return record
                    .committed
                    .then_some(BlobCustodyReleaseOutcomeV1::AlreadyReleased)
                    .ok_or(BlobCustodyReleaseErrorV1::Conflict);
            }
            None => {
                let record = ReleaseRecordV1 {
                    operation_id: request.operation_id,
                    fingerprint: request.fingerprint,
                    not_before_unix_ms: request.not_before_unix_ms,
                    committed: false,
                };
                write_new(&path, &record)?;
                (record, BlobCustodyReleaseOutcomeV1::Accepted)
            }
        };
        if let Err(error) = metadata.reserve_deletion_exact(
            request.reference,
            request.access,
            request.custody,
            record.not_before_unix_ms,
        ) {
            let error = map_lifecycle(error);
            if error == BlobCustodyReleaseErrorV1::Conflict && !record.committed {
                remove_pending(&path)?;
            }
            return Err(error);
        }
        if !record.committed {
            write_replace(
                &path,
                &ReleaseRecordV1 {
                    committed: true,
                    ..record
                },
            )?;
        }
        Ok(outcome)
    }

    fn record_path(&self, reference_id: &[u8; 16]) -> PathBuf {
        self.root.join(hex(reference_id))
    }
}

#[derive(Clone, Copy)]
struct ReleaseRecordV1 {
    operation_id: [u8; 16],
    fingerprint: [u8; 32],
    not_before_unix_ms: u64,
    committed: bool,
}

fn encode(record: &ReleaseRecordV1) -> [u8; RECORD_BYTES] {
    let mut bytes = [0_u8; RECORD_BYTES];
    bytes[..8].copy_from_slice(MAGIC);
    bytes[8..24].copy_from_slice(&record.operation_id);
    bytes[24..56].copy_from_slice(&record.fingerprint);
    bytes[56..64].copy_from_slice(&record.not_before_unix_ms.to_be_bytes());
    bytes[64] = u8::from(record.committed);
    let checksum: [u8; 32] = Sha256::digest(&bytes[..65]).into();
    bytes[65..].copy_from_slice(&checksum);
    bytes
}

fn decode(bytes: &[u8]) -> Result<ReleaseRecordV1, BlobCustodyReleaseErrorV1> {
    if bytes.len() != RECORD_BYTES || &bytes[..8] != MAGIC {
        return Err(BlobCustodyReleaseErrorV1::Unavailable);
    }
    let expected: [u8; 32] = Sha256::digest(&bytes[..65]).into();
    if bytes[65..] != expected {
        return Err(BlobCustodyReleaseErrorV1::Unavailable);
    }
    let operation_id = bytes[8..24]
        .try_into()
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    let fingerprint = bytes[24..56]
        .try_into()
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    let not_before_unix_ms = u64::from_be_bytes(
        bytes[56..64]
            .try_into()
            .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?,
    );
    let committed = match bytes[64] {
        0 => false,
        1 => true,
        _ => return Err(BlobCustodyReleaseErrorV1::Unavailable),
    };
    Ok(ReleaseRecordV1 {
        operation_id,
        fingerprint,
        not_before_unix_ms,
        committed,
    })
}

fn read_record(path: &Path) -> Result<Option<ReleaseRecordV1>, BlobCustodyReleaseErrorV1> {
    match fs::symlink_metadata(path) {
        Ok(_) => root::validate_private_regular_file(path)
            .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(BlobCustodyReleaseErrorV1::Unavailable),
    }
    let mut file = match OpenOptions::new().read(true).open(path) {
        Ok(file) => file,
        Err(_) => return Err(BlobCustodyReleaseErrorV1::Unavailable),
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    decode(&bytes).map(Some)
}

fn write_new(path: &Path, record: &ReleaseRecordV1) -> Result<(), BlobCustodyReleaseErrorV1> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    file.write_all(&encode(record))
        .and_then(|_| file.sync_all())
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    root::validate_private_regular_file(path)
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    sync_root(path)
}

fn write_replace(path: &Path, record: &ReleaseRecordV1) -> Result<(), BlobCustodyReleaseErrorV1> {
    let staged = path.with_extension("pending");
    if staged.exists() {
        fs::remove_file(&staged).map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&staged)
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    file.write_all(&encode(record))
        .and_then(|_| file.sync_all())
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    root::validate_private_regular_file(&staged)
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    fs::rename(&staged, path).map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    sync_root(path)
}

fn remove_pending(path: &Path) -> Result<(), BlobCustodyReleaseErrorV1> {
    root::validate_private_regular_file(path)
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    fs::remove_file(path).map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
    sync_root(path)
}

fn recover_staged(root: &Path) -> Result<(), BlobCustodyReleaseErrorV1> {
    for entry in fs::read_dir(root).map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)? {
        let path = entry
            .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?
            .path();
        if path
            .extension()
            .is_some_and(|extension| extension == "pending")
        {
            let record = read_record(&path)?.ok_or(BlobCustodyReleaseErrorV1::Unavailable)?;
            if !record.committed {
                return Err(BlobCustodyReleaseErrorV1::Unavailable);
            }
            let target = path.with_extension("");
            fs::rename(&path, target).map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)?;
            sync_root(&path)?;
        }
    }
    Ok(())
}

fn sync_root(path: &Path) -> Result<(), BlobCustodyReleaseErrorV1> {
    let root = path
        .parent()
        .ok_or(BlobCustodyReleaseErrorV1::Unavailable)?;
    File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| BlobCustodyReleaseErrorV1::Unavailable)
}

fn hex(bytes: &[u8; 16]) -> String {
    use std::fmt::Write as _;
    let mut value = String::with_capacity(32);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

fn map_lifecycle(error: crate::metadata::BlobMetadataError) -> BlobCustodyReleaseErrorV1 {
    use crate::metadata::BlobMetadataError;

    match error {
        BlobMetadataError::FenceMismatch
        | BlobMetadataError::InvalidGracePeriod
        | BlobMetadataError::NotFound
        | BlobMetadataError::ReservationMismatch => BlobCustodyReleaseErrorV1::Conflict,
        BlobMetadataError::AlreadyExists
        | BlobMetadataError::Filesystem
        | BlobMetadataError::MalformedRecord
        | BlobMetadataError::QuotaExceeded
        | BlobMetadataError::Unavailable
        | BlobMetadataError::UnsafePath => BlobCustodyReleaseErrorV1::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    use makosh_blob_protocol::{
        BlobAccessFenceV1, BlobBackupClassV1, BlobCustodyScopeV1, BlobQuotaGrantV1, BlobRefV1,
    };

    use super::*;

    #[test]
    fn exact_retry_is_existing_and_conflicting_operation_is_already_released() {
        let root = test_root();
        let metadata = BlobMetadataLedger::open(&root).expect("metadata");
        let ledger = BlobCustodyReleaseLedgerV1::open(&root).expect("release ledger");
        let reference = BlobRefV1::new([1; 16], "owner-1", 12, None, BlobBackupClassV1::Required)
            .expect("reference");
        let access = BlobAccessFenceV1::new(
            "owner-1",
            "registration-1",
            "blob.release.v1",
            "runtime-1",
            2,
            3,
        )
        .expect("access");
        let custody = BlobCustodyScopeV1::new("owner-1", "scope-1").expect("custody");
        let quota = BlobQuotaGrantV1::new(
            "owner-1",
            "registration-1",
            "blob.release.v1",
            3,
            1024,
            custody.clone(),
        )
        .expect("quota");
        let write = metadata
            .reserve_write(&reference, &access, &custody, &quota)
            .expect("reserve write");
        metadata
            .commit_write(&write, &reference, &custody)
            .expect("commit write");
        let foreign_access = BlobAccessFenceV1::new(
            "owner-2",
            "registration-2",
            "blob.release.v1",
            "runtime-2",
            2,
            3,
        )
        .expect("foreign access");
        let foreign_custody =
            BlobCustodyScopeV1::new("owner-2", "scope-2").expect("foreign custody");

        assert_eq!(
            ledger.reserve(
                &metadata,
                request(
                    [4; 16],
                    [8; 32],
                    &reference,
                    &foreign_access,
                    &foreign_custody,
                )
            ),
            Err(BlobCustodyReleaseErrorV1::Conflict)
        );

        assert_eq!(
            ledger.reserve(
                &metadata,
                request([4; 16], [5; 32], &reference, &access, &custody)
            ),
            Ok(BlobCustodyReleaseOutcomeV1::Accepted)
        );
        assert_eq!(
            ledger.reserve(
                &metadata,
                request([4; 16], [5; 32], &reference, &access, &custody)
            ),
            Ok(BlobCustodyReleaseOutcomeV1::Existing)
        );
        assert_eq!(
            ledger.reserve(
                &metadata,
                request([4; 16], [8; 32], &reference, &access, &custody)
            ),
            Err(BlobCustodyReleaseErrorV1::Conflict)
        );
        let reopened = BlobCustodyReleaseLedgerV1::open(&root).expect("reopen release ledger");
        assert_eq!(
            reopened.reserve(
                &metadata,
                request([4; 16], [5; 32], &reference, &access, &custody)
            ),
            Ok(BlobCustodyReleaseOutcomeV1::Existing)
        );
        assert_eq!(
            reopened.reserve(
                &metadata,
                request([6; 16], [7; 32], &reference, &access, &custody)
            ),
            Ok(BlobCustodyReleaseOutcomeV1::AlreadyReleased)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn request<'a>(
        operation_id: [u8; 16],
        fingerprint: [u8; 32],
        reference: &'a BlobRefV1,
        access: &'a BlobAccessFenceV1,
        custody: &'a BlobCustodyScopeV1,
    ) -> BlobCustodyReleaseRequestV1<'a> {
        BlobCustodyReleaseRequestV1 {
            operation_id,
            fingerprint,
            reference,
            access,
            custody,
            not_before_unix_ms: 50_000,
        }
    }

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "makosh-blob-release-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create root");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private root");
        root
    }
}
