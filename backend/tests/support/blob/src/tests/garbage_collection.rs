use std::fs;
use std::os::unix::fs::PermissionsExt;

use makosh_blob_protocol::{
    BlobAccessFenceV1, BlobBackupClassV1, BlobCustodyScopeV1, BlobQuotaGrantV1, BlobRefV1,
};
use makosh_blob_runtime::lease::BlobKeyLeaseV1;
use makosh_blob_runtime::storage::{
    BlobContentLifecycleStore, BlobContentWriteRequestV1, BlobDeletionAuthorizationV1,
    BlobDeletionLeaseErrorV1, BlobDeletionLeaseResolverV1,
};
use zeroize::Zeroizing;

#[test]
fn scheduled_gc_deletes_only_due_owner_marked_references() {
    let directory = private_directory();
    let store = BlobContentLifecycleStore::open(directory.path(), 1024).expect("lifecycle store");
    let reference = reference(1);
    let access = access();
    let lease = lease(&reference, access.clone());
    store
        .write_new(BlobContentWriteRequestV1 {
            reference: &reference,
            access: &access,
            custody: &custody(),
            quota: &quota(),
            lease: &lease,
            plaintext: b"private bytes",
            now_unix_ms: 2,
        })
        .expect("write encrypted Blob");
    store
        .reserve_deletion(&reference, &access, &custody(), 10, 5)
        .expect("owner marks Blob eligible");

    let mut resolver = OneLeaseResolver::new(lease);
    let report = store
        .collect_due_deletions(&mut resolver, 15)
        .expect("collect due encrypted Blob");
    assert_eq!(report.deleted(), 1);
    assert_eq!(report.deferred(), 0);
    assert!(!content_path(directory.path(), &reference).exists());
}

#[test]
fn revoked_or_unavailable_gc_key_defers_deletion_without_touching_ciphertext() {
    let directory = private_directory();
    let store = BlobContentLifecycleStore::open(directory.path(), 1024).expect("lifecycle store");
    let reference = reference(2);
    let access = access();
    let lease = lease(&reference, access.clone());
    store
        .write_new(BlobContentWriteRequestV1 {
            reference: &reference,
            access: &access,
            custody: &custody(),
            quota: &quota(),
            lease: &lease,
            plaintext: b"private bytes",
            now_unix_ms: 2,
        })
        .expect("write encrypted Blob");
    store
        .reserve_deletion(&reference, &access, &custody(), 10, 5)
        .expect("owner marks Blob eligible");

    let report = store
        .collect_due_deletions(&mut DeniedResolver, 15)
        .expect("revoked key defers deletion");
    assert_eq!(report.deleted(), 0);
    assert_eq!(report.deferred(), 1);
    assert!(content_path(directory.path(), &reference).is_file());

    let report = store
        .collect_due_deletions(&mut OneLeaseResolver::new(lease), 15)
        .expect("fresh current key deletes Blob");
    assert_eq!(report.deleted(), 1);
    assert!(!content_path(directory.path(), &reference).exists());
}

struct OneLeaseResolver(Option<BlobKeyLeaseV1>);

impl OneLeaseResolver {
    fn new(lease: BlobKeyLeaseV1) -> Self {
        Self(Some(lease))
    }
}

impl BlobDeletionLeaseResolverV1 for OneLeaseResolver {
    fn resolve_deletion_lease(
        &mut self,
        _: &BlobRefV1,
        _: &BlobCustodyScopeV1,
        _: u64,
    ) -> Result<BlobDeletionAuthorizationV1, BlobDeletionLeaseErrorV1> {
        self.0
            .take()
            .map(|lease| BlobDeletionAuthorizationV1::new(access(), lease))
            .ok_or(BlobDeletionLeaseErrorV1::Unavailable)
    }
}

struct DeniedResolver;

impl BlobDeletionLeaseResolverV1 for DeniedResolver {
    fn resolve_deletion_lease(
        &mut self,
        _: &BlobRefV1,
        _: &BlobCustodyScopeV1,
        _: u64,
    ) -> Result<BlobDeletionAuthorizationV1, BlobDeletionLeaseErrorV1> {
        Err(BlobDeletionLeaseErrorV1::Revoked)
    }
}

fn reference(value: u8) -> BlobRefV1 {
    BlobRefV1::new(
        [value; 16],
        "owner_notes",
        13,
        Some(2_000),
        BlobBackupClassV1::Required,
    )
    .expect("Blob reference")
}

fn access() -> BlobAccessFenceV1 {
    BlobAccessFenceV1::new(
        "owner_notes",
        "registration_notes",
        "attachments",
        "notes_runtime",
        3,
        4,
    )
    .expect("Blob access fence")
}

fn quota() -> BlobQuotaGrantV1 {
    BlobQuotaGrantV1::new(
        "owner_notes",
        "registration_notes",
        "attachments",
        4,
        20,
        custody(),
    )
    .expect("Blob quota grant")
}

fn custody() -> BlobCustodyScopeV1 {
    BlobCustodyScopeV1::new("owner_notes", "attachments").expect("Blob custody")
}

fn lease(reference: &BlobRefV1, access: BlobAccessFenceV1) -> BlobKeyLeaseV1 {
    BlobKeyLeaseV1::from_vault_response(
        reference,
        access,
        custody(),
        1,
        1_500,
        1,
        Zeroizing::new(vec![9; 64]),
    )
    .expect("Blob key lease")
}

fn private_directory() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary root");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private temporary root");
    directory
}

fn content_path(root: &std::path::Path, reference: &BlobRefV1) -> std::path::PathBuf {
    let id = reference
        .reference_id()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    root.join("content").join(format!("{id}.blob"))
}
