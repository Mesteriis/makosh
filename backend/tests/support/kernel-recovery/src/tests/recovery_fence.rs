use std::path::PathBuf;

use makosh_kernel_control_store::RecoveryFences;
use makosh_kernel_control_store_sqlite::{SqliteControlStore, StagedControlStoreRestore};

use crate::recovery::fence as recovery_fence;

const INSTANCE_ID: &str = "11111111111111111111111111111111";

#[test]
fn reserved_fence_without_database_replacement_fails_closed_and_advances_retry() {
    let fixture = RestoreFenceFixture::new("reserved-only");
    let staged = fixture.stage(RecoveryFences::new(2, 2, 2));
    recovery_fence::reserve(
        &fixture.root,
        INSTANCE_ID,
        RecoveryFences::new(2, 2, 2),
        *staged.sha256(),
    )
    .expect("reserve fence");

    let current = SqliteControlStore::open(&fixture.destination).expect("open old destination");
    assert!(recovery_fence::verify_or_finalize(
        &fixture.root,
        &fixture.destination,
        current.snapshot(),
    )
    .is_err());
    let record = recovery_fence::read(&fixture.root).expect("read reserved fence");
    let source = SqliteControlStore::open(&fixture.source).expect("open source");
    let retry = recovery_fence::next(&record, Some(current.snapshot()), source.snapshot())
        .expect("calculate retry fences");
    assert_eq!(retry, RecoveryFences::new(3, 3, 3));
}

#[test]
fn reserved_fence_with_matching_database_is_committed_after_restart() {
    let fixture = RestoreFenceFixture::new("renamed-before-commit");
    let staged = fixture.stage(RecoveryFences::new(2, 2, 2));
    recovery_fence::reserve(
        &fixture.root,
        INSTANCE_ID,
        RecoveryFences::new(2, 2, 2),
        *staged.sha256(),
    )
    .expect("reserve fence");
    std::fs::rename(staged.path(), &fixture.destination).expect("replace destination");

    let restored = SqliteControlStore::open(&fixture.destination).expect("open restored store");
    recovery_fence::verify_or_finalize(&fixture.root, &fixture.destination, restored.snapshot())
        .expect("finalize matching reservation");
    recovery_fence::verify_or_finalize(&fixture.root, &fixture.destination, restored.snapshot())
        .expect("accept committed fence");
}

#[test]
fn reserved_fence_rejects_a_replaced_database_with_the_wrong_digest() {
    let fixture = RestoreFenceFixture::new("digest-mismatch");
    let staged = fixture.stage(RecoveryFences::new(2, 2, 2));
    recovery_fence::reserve(
        &fixture.root,
        INSTANCE_ID,
        RecoveryFences::new(2, 2, 2),
        [7; 32],
    )
    .expect("reserve mismatched fence");
    std::fs::rename(staged.path(), &fixture.destination).expect("replace destination");

    let restored = SqliteControlStore::open(&fixture.destination).expect("open restored store");
    let error = recovery_fence::verify_or_finalize(
        &fixture.root,
        &fixture.destination,
        restored.snapshot(),
    )
    .expect_err("digest mismatch must remain recovery-only");
    assert_eq!(
        error,
        "reserved recovery fence has no matching staged store"
    );
}

#[test]
fn next_fences_use_the_highest_external_current_or_backup_value() {
    let fixture = RestoreFenceFixture::new("high-water");
    let mut current = SqliteControlStore::open(&fixture.destination).expect("open current");
    for _ in 0..4 {
        current.advance_recovery_fences().expect("advance current");
        drop(current);
        current = SqliteControlStore::open(&fixture.destination).expect("reopen current");
    }
    recovery_fence::initialize(&fixture.root, INSTANCE_ID, RecoveryFences::new(7, 6, 4))
        .expect("raise external high-water");

    let record = recovery_fence::read(&fixture.root).expect("read external fence");
    let source = SqliteControlStore::open(&fixture.source).expect("open older backup");
    let fences = recovery_fence::next(&record, Some(current.snapshot()), source.snapshot())
        .expect("calculate high-water fences");
    assert_eq!(fences, RecoveryFences::new(8, 7, 6));
}

#[test]
fn repeated_restore_reserves_the_next_generation() {
    let fixture = RestoreFenceFixture::new("repeated");
    install_and_finalize(&fixture, RecoveryFences::new(2, 2, 2));
    let current = SqliteControlStore::open(&fixture.destination).expect("open first restore");
    let source = SqliteControlStore::open(&fixture.source).expect("open backup");
    let record = recovery_fence::read(&fixture.root).expect("read first fence");
    let next = recovery_fence::next(&record, Some(current.snapshot()), source.snapshot())
        .expect("calculate repeated restore fences");
    assert_eq!(next, RecoveryFences::new(3, 3, 3));

    install_and_finalize(&fixture, next);
    let repeated = SqliteControlStore::open(&fixture.destination).expect("open repeated restore");
    assert_eq!(repeated.snapshot().generation(), 3);
}

fn install_and_finalize(fixture: &RestoreFenceFixture, fences: RecoveryFences) {
    let staged = fixture.stage(fences);
    recovery_fence::reserve(&fixture.root, INSTANCE_ID, fences, *staged.sha256())
        .expect("reserve restore");
    std::fs::rename(staged.path(), &fixture.destination).expect("replace destination");
    let restored = SqliteControlStore::open(&fixture.destination).expect("open restored store");
    recovery_fence::verify_or_finalize(&fixture.root, &fixture.destination, restored.snapshot())
        .expect("finalize restore");
}

struct RestoreFenceFixture {
    root: PathBuf,
    source: PathBuf,
    destination: PathBuf,
    staged: PathBuf,
}

impl RestoreFenceFixture {
    fn new(name: &str) -> Self {
        let root = fixture_root(name);
        let source = root.join("source.sqlite");
        let destination = root.join("destination.sqlite");
        let staged = root.join("staged.sqlite");
        std::fs::create_dir(&root).expect("create fixture root");
        SqliteControlStore::create(&source, INSTANCE_ID, 1).expect("create source");
        SqliteControlStore::create(&destination, INSTANCE_ID, 1).expect("create destination");
        recovery_fence::initialize(&root, INSTANCE_ID, RecoveryFences::new(1, 1, 1))
            .expect("initialize recovery fence");
        Self {
            root,
            source,
            destination,
            staged,
        }
    }

    fn stage(&self, fences: RecoveryFences) -> StagedControlStoreRestore {
        StagedControlStoreRestore::prepare(&self.source, &self.staged, INSTANCE_ID, fences)
            .expect("prepare staged restore")
    }
}

impl Drop for RestoreFenceFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_root(name: &str) -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "makosh-recovery-fence-{name}-{}-{suffix}",
        std::process::id()
    ))
}
