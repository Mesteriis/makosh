//! Crash-aware composition of encrypted Blob bytes and technical quota metadata.

use std::path::Path;

use makosh_blob_protocol::{
    BlobAccessFenceV1, BlobCustodyScopeV1, BlobQuotaGrantV1, BlobRangeV1, BlobRefV1,
};
use sha2::{Digest, Sha256};

use crate::lease::BlobKeyLeaseV1;
use crate::metadata::{BlobDeletionReservationV1, BlobMetadataError, BlobMetadataLedger};
use crate::release::{
    BlobCustodyReleaseErrorV1, BlobCustodyReleaseLedgerV1, BlobCustodyReleaseOutcomeV1,
    BlobCustodyReleaseRequestV1,
};

use super::{BlobStorageError, EncryptedBlobStore};

/// Resolves one current deletion key or explicitly defers the removal.
pub trait BlobDeletionLeaseResolverV1 {
    fn resolve_deletion_lease(
        &mut self,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
        now_unix_ms: u64,
    ) -> Result<BlobDeletionAuthorizationV1, BlobDeletionLeaseErrorV1>;
}

pub struct BlobDeletionAuthorizationV1 {
    access: BlobAccessFenceV1,
    lease: BlobKeyLeaseV1,
}

impl BlobDeletionAuthorizationV1 {
    #[must_use]
    pub fn new(access: BlobAccessFenceV1, lease: BlobKeyLeaseV1) -> Self {
        Self { access, lease }
    }
}

/// A revoked or unavailable key never authorizes an implicit Blob deletion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobDeletionLeaseErrorV1 {
    Revoked,
    Unavailable,
}

/// Sanitized result of one scheduled deletion pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlobGarbageCollectionReportV1 {
    deleted: u64,
    deferred: u64,
}

impl BlobGarbageCollectionReportV1 {
    #[must_use]
    pub const fn deleted(&self) -> u64 {
        self.deleted
    }

    #[must_use]
    pub const fn deferred(&self) -> u64 {
        self.deferred
    }
}

/// The only Blob write lifecycle that applies aggregate quota reservations.
pub struct BlobContentLifecycleStore {
    encrypted: EncryptedBlobStore,
    metadata: BlobMetadataLedger,
}

impl BlobContentLifecycleStore {
    pub fn open(data_dir: &Path, maximum_blob_bytes: u64) -> Result<Self, BlobLifecycleError> {
        let store = Self {
            encrypted: EncryptedBlobStore::open(data_dir, maximum_blob_bytes)
                .map_err(BlobLifecycleError::Storage)?,
            metadata: BlobMetadataLedger::open(data_dir).map_err(BlobLifecycleError::Metadata)?,
        };
        store.recover_uncommitted_writes()?;
        Ok(store)
    }

    pub fn write_new(
        &self,
        request: BlobContentWriteRequestV1<'_>,
    ) -> Result<(), BlobLifecycleError> {
        let BlobContentWriteRequestV1 {
            reference,
            access,
            custody,
            quota,
            lease,
            plaintext,
            now_unix_ms,
        } = request;
        let reservation = self
            .metadata
            .reserve_write(reference, access, custody, quota)
            .map_err(BlobLifecycleError::Metadata)?;
        match self
            .encrypted
            .write_new(reference, access, custody, lease, plaintext, now_unix_ms)
        {
            Ok(()) => self
                .metadata
                .commit_write(&reservation, reference, custody)
                .map_err(BlobLifecycleError::Metadata),
            Err(error) => {
                let _ = self
                    .metadata
                    .abandon_write(&reservation, reference, custody);
                Err(BlobLifecycleError::Storage(error))
            }
        }
    }

    /// Accepts an exact retry only when the existing encrypted object matches
    /// the full receipt. Ordinary writes remain create-only through `write_new`.
    pub fn write_receipt_bound(
        &self,
        request: BlobContentWriteRequestV1<'_>,
        expected_plaintext_sha256: &[u8; 32],
    ) -> Result<(), BlobLifecycleError> {
        if Sha256::digest(request.plaintext).as_slice() != expected_plaintext_sha256 {
            return Err(BlobLifecycleError::Integrity);
        }
        match self.write_new(request) {
            Ok(()) => Ok(()),
            Err(BlobLifecycleError::Metadata(BlobMetadataError::AlreadyExists))
            | Err(BlobLifecycleError::Storage(BlobStorageError::AlreadyExists)) => {
                let full = BlobRangeV1::new(
                    0,
                    request.reference.declared_size(),
                    request.reference.declared_size(),
                )
                .map_err(|_| BlobLifecycleError::Integrity)?;
                let existing = self.read_range(
                    request.reference,
                    request.access,
                    request.custody,
                    request.lease,
                    full,
                    request.now_unix_ms,
                )?;
                (Sha256::digest(existing).as_slice() == expected_plaintext_sha256)
                    .then_some(())
                    .ok_or(BlobLifecycleError::Integrity)
            }
            Err(error) => Err(error),
        }
    }

    pub fn read_range(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        lease: &BlobKeyLeaseV1,
        range: BlobRangeV1,
        now_unix_ms: u64,
    ) -> Result<Vec<u8>, BlobLifecycleError> {
        self.encrypted
            .read_range(reference, access, custody, lease, range, now_unix_ms)
            .map_err(BlobLifecycleError::Storage)
    }

    /// Re-encrypts verified source content into a receiver-owned reference.
    /// Plaintext remains inside Blob runtime and the source object is retained.
    pub fn custody_transfer(
        &self,
        request: BlobCustodyTransferRequestV1<'_>,
    ) -> Result<(), BlobLifecycleError> {
        let BlobCustodyTransferRequestV1 {
            source_reference,
            source_access,
            source_custody,
            source_lease,
            target_reference,
            target_access,
            target_custody,
            target_quota,
            target_lease,
            expected_plaintext_sha256,
            now_unix_ms,
        } = request;
        let full_source = BlobRangeV1::new(
            0,
            source_reference.declared_size(),
            source_reference.declared_size(),
        )
        .map_err(|_| BlobLifecycleError::Integrity)?;
        let plaintext = self.read_range(
            source_reference,
            source_access,
            source_custody,
            source_lease,
            full_source,
            now_unix_ms,
        )?;
        if Sha256::digest(&plaintext).as_slice() != expected_plaintext_sha256 {
            return Err(BlobLifecycleError::Integrity);
        }
        self.write_receipt_bound(
            BlobContentWriteRequestV1 {
                reference: target_reference,
                access: target_access,
                custody: target_custody,
                quota: target_quota,
                lease: target_lease,
                plaintext: &plaintext,
                now_unix_ms,
            },
            expected_plaintext_sha256,
        )
    }

    pub fn reserve_deletion(
        &self,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        now_unix_ms: u64,
        grace_period_ms: u64,
    ) -> Result<BlobDeletionReservationV1, BlobLifecycleError> {
        self.metadata
            .reserve_deletion(reference, access, custody, now_unix_ms, grace_period_ms)
            .map_err(BlobLifecycleError::Metadata)
    }

    pub fn reserve_custody_release(
        &self,
        ledger: &BlobCustodyReleaseLedgerV1,
        request: BlobCustodyReleaseRequestV1<'_>,
    ) -> Result<BlobCustodyReleaseOutcomeV1, BlobCustodyReleaseErrorV1> {
        ledger.reserve(&self.metadata, request)
    }

    pub fn delete_due(
        &self,
        reservation: &BlobDeletionReservationV1,
        reference: &BlobRefV1,
        access: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        lease: &BlobKeyLeaseV1,
        now_unix_ms: u64,
    ) -> Result<(), BlobLifecycleError> {
        self.metadata
            .deletion_is_due(reservation, reference, custody, now_unix_ms)
            .map_err(BlobLifecycleError::Metadata)?;
        self.encrypted
            .delete(reference, access, custody, lease, now_unix_ms)
            .map_err(BlobLifecycleError::Storage)?;
        self.metadata
            .finalize_deletion(reservation, reference, custody, now_unix_ms)
            .map_err(BlobLifecycleError::Metadata)
    }

    pub fn reconcile_missing_deletions(&self) -> Result<u64, BlobLifecycleError> {
        self.metadata
            .reconcile_missing_deletions(|reference| {
                self.encrypted
                    .exists(reference)
                    .map_err(|_| BlobMetadataError::Filesystem)
            })
            .map_err(BlobLifecycleError::Metadata)
    }

    pub fn collect_due_deletions<R>(
        &self,
        resolver: &mut R,
        now_unix_ms: u64,
    ) -> Result<BlobGarbageCollectionReportV1, BlobLifecycleError>
    where
        R: BlobDeletionLeaseResolverV1,
    {
        let mut report = BlobGarbageCollectionReportV1 {
            deleted: 0,
            deferred: 0,
        };
        for due in self
            .metadata
            .due_deletions(now_unix_ms)
            .map_err(BlobLifecycleError::Metadata)?
        {
            match resolver.resolve_deletion_lease(due.reference(), due.custody(), now_unix_ms) {
                Ok(authorization) => {
                    self.delete_due(
                        due.reservation(),
                        due.reference(),
                        &authorization.access,
                        due.custody(),
                        &authorization.lease,
                        now_unix_ms,
                    )?;
                    report.deleted += 1;
                }
                Err(BlobDeletionLeaseErrorV1::Revoked | BlobDeletionLeaseErrorV1::Unavailable) => {
                    report.deferred += 1;
                }
            }
        }
        Ok(report)
    }

    fn recover_uncommitted_writes(&self) -> Result<(), BlobLifecycleError> {
        for pending in self
            .metadata
            .pending_writes()
            .map_err(BlobLifecycleError::Metadata)?
        {
            self.encrypted
                .discard_uncommitted(pending.reference())
                .map_err(BlobLifecycleError::Storage)?;
            self.metadata
                .discard_pending_write(&pending)
                .map_err(BlobLifecycleError::Metadata)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct BlobContentWriteRequestV1<'a> {
    pub reference: &'a BlobRefV1,
    pub access: &'a BlobAccessFenceV1,
    pub custody: &'a BlobCustodyScopeV1,
    pub quota: &'a BlobQuotaGrantV1,
    pub lease: &'a BlobKeyLeaseV1,
    pub plaintext: &'a [u8],
    pub now_unix_ms: u64,
}

pub struct BlobCustodyTransferRequestV1<'a> {
    pub source_reference: &'a BlobRefV1,
    pub source_access: &'a BlobAccessFenceV1,
    pub source_custody: &'a BlobCustodyScopeV1,
    pub source_lease: &'a BlobKeyLeaseV1,
    pub target_reference: &'a BlobRefV1,
    pub target_access: &'a BlobAccessFenceV1,
    pub target_custody: &'a BlobCustodyScopeV1,
    pub target_quota: &'a BlobQuotaGrantV1,
    pub target_lease: &'a BlobKeyLeaseV1,
    pub expected_plaintext_sha256: &'a [u8; 32],
    pub now_unix_ms: u64,
}

#[derive(Debug)]
pub enum BlobLifecycleError {
    Integrity,
    Metadata(BlobMetadataError),
    Storage(BlobStorageError),
}
