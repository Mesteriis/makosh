//! Atomic filesystem ledger for Blob byte reservations and grace-period deletes.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use makosh_blob_protocol::{BlobAccessFenceV1, BlobCustodyScopeV1, BlobQuotaGrantV1, BlobRefV1};

use crate::storage::{BlobStorageError, root};

use super::codec;
use super::record::{
    BlobDeletionReservationV1, BlobDueDeletionV1, BlobMetadataRecordV1, BlobMetadataStateV1,
    BlobWriteReservationV1,
};

const METADATA_SUFFIX: &str = ".meta";

/// Blob-owned technical ledger; it deliberately does not contain owner metadata.
pub struct BlobMetadataLedger {
    metadata_root: PathBuf,
    operation_lock: Mutex<()>,
}

impl BlobMetadataLedger {
    pub fn open(data_dir: &Path) -> Result<Self, BlobMetadataError> {
        let metadata_root = root::prepare_metadata_root(data_dir).map_err(map_storage_error)?;
        recover_staged_records(&metadata_root)?;
        Ok(Self {
            metadata_root,
            operation_lock: Mutex::new(()),
        })
    }

    pub fn reserve_write(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        quota: &BlobQuotaGrantV1,
    ) -> Result<BlobWriteReservationV1, BlobMetadataError> {
        let _guard = self.lock()?;
        if !quota.matches(access)
            || quota.custody() != custody
            || reference.owner_id() != custody.owner_id()
        {
            return Err(BlobMetadataError::FenceMismatch);
        }
        if self.read(reference)?.is_some() {
            return Err(BlobMetadataError::AlreadyExists);
        }
        let used_bytes = self.used_bytes(custody)?;
        let total = used_bytes
            .checked_add(reference.declared_size())
            .ok_or(BlobMetadataError::QuotaExceeded)?;
        if total > quota.max_bytes() {
            return Err(BlobMetadataError::QuotaExceeded);
        }
        self.write(&BlobMetadataRecordV1::pending(
            reference.clone(),
            custody.clone(),
        ))?;
        Ok(BlobWriteReservationV1::new(reference))
    }

    pub fn commit_write(
        &self,
        reservation: &BlobWriteReservationV1,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
    ) -> Result<(), BlobMetadataError> {
        let _guard = self.lock()?;
        let mut record = self.required(reference)?;
        if !reservation.matches(reference)
            || !record.matches(reference, custody)
            || record.state() != BlobMetadataStateV1::PendingWrite
        {
            return Err(BlobMetadataError::ReservationMismatch);
        }
        record.activate();
        self.write(&record)
    }

    pub fn abandon_write(
        &self,
        reservation: &BlobWriteReservationV1,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
    ) -> Result<(), BlobMetadataError> {
        let _guard = self.lock()?;
        let record = self.required(reference)?;
        if !reservation.matches(reference)
            || !record.matches(reference, custody)
            || record.state() != BlobMetadataStateV1::PendingWrite
        {
            return Err(BlobMetadataError::ReservationMismatch);
        }
        self.remove(reference)
    }

    pub fn reserve_deletion(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        now_unix_ms: u64,
        grace_period_ms: u64,
    ) -> Result<BlobDeletionReservationV1, BlobMetadataError> {
        if grace_period_ms == 0
            || access.owner_id() != custody.owner_id()
            || reference.owner_id() != custody.owner_id()
        {
            return Err(BlobMetadataError::InvalidGracePeriod);
        }
        let not_before_unix_ms = now_unix_ms
            .checked_add(grace_period_ms)
            .ok_or(BlobMetadataError::InvalidGracePeriod)?;
        self.reserve_deletion_exact(reference, access, custody, not_before_unix_ms)
    }

    pub fn reserve_deletion_exact(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        not_before_unix_ms: u64,
    ) -> Result<BlobDeletionReservationV1, BlobMetadataError> {
        let _guard = self.lock()?;
        if not_before_unix_ms == 0
            || access.owner_id() != custody.owner_id()
            || reference.owner_id() != custody.owner_id()
        {
            return Err(BlobMetadataError::InvalidGracePeriod);
        }
        let mut record = self.required(reference)?;
        if !record.matches(reference, custody) {
            return Err(BlobMetadataError::ReservationMismatch);
        }
        if let BlobMetadataStateV1::DeleteReserved {
            not_before_unix_ms: existing,
        } = record.state()
        {
            return (existing == not_before_unix_ms)
                .then(|| BlobDeletionReservationV1::new(reference, existing))
                .ok_or(BlobMetadataError::ReservationMismatch);
        }
        if record.state() != BlobMetadataStateV1::Active {
            return Err(BlobMetadataError::ReservationMismatch);
        }
        record.reserve_deletion(not_before_unix_ms);
        self.write(&record)?;
        Ok(BlobDeletionReservationV1::new(
            reference,
            not_before_unix_ms,
        ))
    }

    pub fn deletion_is_due(
        &self,
        reservation: &BlobDeletionReservationV1,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
        now_unix_ms: u64,
    ) -> Result<(), BlobMetadataError> {
        let _guard = self.lock()?;
        let record = self.required(reference)?;
        matches_due_deletion(&record, reservation, reference, custody, now_unix_ms)
            .then_some(())
            .ok_or(BlobMetadataError::ReservationMismatch)
    }

    pub fn finalize_deletion(
        &self,
        reservation: &BlobDeletionReservationV1,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
        now_unix_ms: u64,
    ) -> Result<(), BlobMetadataError> {
        let _guard = self.lock()?;
        let record = self.required(reference)?;
        matches_due_deletion(&record, reservation, reference, custody, now_unix_ms)
            .then_some(())
            .ok_or(BlobMetadataError::ReservationMismatch)?;
        self.remove(reference)
    }

    pub fn reconcile_missing_deletions<F>(&self, present: F) -> Result<u64, BlobMetadataError>
    where
        F: Fn(&BlobRefV1) -> Result<bool, BlobMetadataError>,
    {
        let _guard = self.lock()?;
        let records = self.records()?;
        let mut removed = 0;
        for record in records {
            if matches!(record.state(), BlobMetadataStateV1::DeleteReserved { .. })
                && !present(record.reference())?
            {
                self.remove(record.reference())?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub(crate) fn due_deletions(
        &self,
        now_unix_ms: u64,
    ) -> Result<Vec<BlobDueDeletionV1>, BlobMetadataError> {
        let _guard = self.lock()?;
        Ok(self
            .records()?
            .iter()
            .filter_map(|record| record.due_deletion(now_unix_ms))
            .collect())
    }

    pub(crate) fn pending_writes(&self) -> Result<Vec<BlobMetadataRecordV1>, BlobMetadataError> {
        let _guard = self.lock()?;
        Ok(self
            .records()?
            .into_iter()
            .filter(BlobMetadataRecordV1::is_pending_write)
            .collect())
    }

    pub(crate) fn discard_pending_write(
        &self,
        pending: &BlobMetadataRecordV1,
    ) -> Result<(), BlobMetadataError> {
        let _guard = self.lock()?;
        let current = self.required(pending.reference())?;
        (current == *pending && current.is_pending_write())
            .then_some(())
            .ok_or(BlobMetadataError::ReservationMismatch)?;
        self.remove(pending.reference())
    }

    fn used_bytes(&self, custody: &BlobCustodyScopeV1) -> Result<u64, BlobMetadataError> {
        self.records()?
            .into_iter()
            .try_fold(0_u64, |total, record| {
                if record.belongs_to_quota(custody) && record.counts_toward_quota() {
                    total
                        .checked_add(record.reference().declared_size())
                        .ok_or(BlobMetadataError::QuotaExceeded)
                } else {
                    Ok(total)
                }
            })
    }

    fn required(&self, reference: &BlobRefV1) -> Result<BlobMetadataRecordV1, BlobMetadataError> {
        self.read(reference)?.ok_or(BlobMetadataError::NotFound)
    }

    fn read(
        &self,
        reference: &BlobRefV1,
    ) -> Result<Option<BlobMetadataRecordV1>, BlobMetadataError> {
        let path = self.path(reference);
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                root::validate_private_regular_file(&path).map_err(map_storage_error)?;
                codec::decode(&fs::read(path).map_err(|_| BlobMetadataError::Filesystem)?)
                    .filter(|record| record.reference() == reference)
                    .map(Some)
                    .ok_or(BlobMetadataError::MalformedRecord)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(BlobMetadataError::Filesystem),
        }
    }

    fn records(&self) -> Result<Vec<BlobMetadataRecordV1>, BlobMetadataError> {
        let mut records = Vec::new();
        for entry in fs::read_dir(&self.metadata_root).map_err(|_| BlobMetadataError::Filesystem)? {
            let path = entry.map_err(|_| BlobMetadataError::Filesystem)?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("meta") {
                return Err(BlobMetadataError::UnsafePath);
            }
            root::validate_private_regular_file(&path).map_err(map_storage_error)?;
            let record = codec::decode(&fs::read(path).map_err(|_| BlobMetadataError::Filesystem)?)
                .ok_or(BlobMetadataError::MalformedRecord)?;
            records.push(record);
        }
        Ok(records)
    }

    fn write(&self, record: &BlobMetadataRecordV1) -> Result<(), BlobMetadataError> {
        let target = self.path(record.reference());
        let staged = target.with_extension("staged");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&staged)
            .map_err(|_| BlobMetadataError::Filesystem)?;
        let result = self.write_staged(&mut file, &staged, &target, record);
        if result.is_err() {
            let _ = fs::remove_file(staged);
        }
        result
    }

    fn write_staged(
        &self,
        file: &mut File,
        staged: &Path,
        target: &Path,
        record: &BlobMetadataRecordV1,
    ) -> Result<(), BlobMetadataError> {
        file.write_all(&codec::encode(record))
            .and_then(|_| file.sync_all())
            .map_err(|_| BlobMetadataError::Filesystem)?;
        root::validate_private_regular_file(staged).map_err(map_storage_error)?;
        fs::rename(staged, target).map_err(|_| BlobMetadataError::Filesystem)?;
        self.sync_root()
    }

    fn remove(&self, reference: &BlobRefV1) -> Result<(), BlobMetadataError> {
        let path = self.path(reference);
        root::validate_private_regular_file(&path).map_err(map_storage_error)?;
        fs::remove_file(path).map_err(|_| BlobMetadataError::Filesystem)?;
        self.sync_root()
    }

    fn sync_root(&self) -> Result<(), BlobMetadataError> {
        File::open(&self.metadata_root)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| BlobMetadataError::Filesystem)
    }

    fn path(&self, reference: &BlobRefV1) -> PathBuf {
        let mut filename = String::with_capacity(37);
        for byte in reference.reference_id() {
            use std::fmt::Write as _;
            write!(&mut filename, "{byte:02x}").expect("writing to String cannot fail");
        }
        filename.push_str(METADATA_SUFFIX);
        self.metadata_root.join(filename)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ()>, BlobMetadataError> {
        self.operation_lock
            .lock()
            .map_err(|_| BlobMetadataError::Unavailable)
    }
}

fn matches_due_deletion(
    record: &BlobMetadataRecordV1,
    reservation: &BlobDeletionReservationV1,
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    now_unix_ms: u64,
) -> bool {
    matches!(
        record.state(),
        BlobMetadataStateV1::DeleteReserved { not_before_unix_ms }
            if not_before_unix_ms == reservation.not_before_unix_ms()
                && not_before_unix_ms <= now_unix_ms
    ) && reservation.matches(reference)
        && record.matches(reference, custody)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobMetadataError {
    AlreadyExists,
    FenceMismatch,
    Filesystem,
    InvalidGracePeriod,
    MalformedRecord,
    NotFound,
    QuotaExceeded,
    ReservationMismatch,
    Unavailable,
    UnsafePath,
}

const fn map_storage_error(error: BlobStorageError) -> BlobMetadataError {
    match error {
        BlobStorageError::UnsafePath => BlobMetadataError::UnsafePath,
        _ => BlobMetadataError::Filesystem,
    }
}

fn recover_staged_records(metadata_root: &Path) -> Result<(), BlobMetadataError> {
    let mut removed = false;
    for entry in fs::read_dir(metadata_root).map_err(|_| BlobMetadataError::Filesystem)? {
        let path = entry.map_err(|_| BlobMetadataError::Filesystem)?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("staged") {
            root::validate_private_regular_file(&path).map_err(map_storage_error)?;
            fs::remove_file(path).map_err(|_| BlobMetadataError::Filesystem)?;
            removed = true;
        }
    }
    if removed {
        File::open(metadata_root)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| BlobMetadataError::Filesystem)?;
    }
    Ok(())
}
