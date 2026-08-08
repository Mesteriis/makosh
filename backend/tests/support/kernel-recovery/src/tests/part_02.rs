use super::common::*;
use std::path::{Path, PathBuf};

#[test]
fn staged_native_artifact_creates_a_private_verified_execution_copy() {
    let fixture_name = format!(
        "makosh-staged-artifact-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let root = std::env::temp_dir().join(fixture_name);
    let source = root.join("signed-runtime");
    let launch_directory = root.join("runtime/launch");
    let bytes = b"makosh verified staged runtime";
    std::fs::create_dir_all(&root).expect("create fixture root");
    std::fs::write(&source, bytes).expect("write source artifact");
    let expected_digest: [u8; 32] = Sha256::digest(bytes).into();

    let staged =
        staged_native_artifact::stage(&source, &launch_directory, "runtime.mail", &expected_digest)
            .expect("stage verified runtime");

    assert_ne!(staged.path(), source);
    assert_eq!(
        std::fs::read(staged.path()).expect("read staged runtime"),
        bytes
    );
    assert_eq!(
        std::fs::metadata(staged.path())
            .expect("staged metadata")
            .permissions()
            .mode()
            & 0o777,
        0o500
    );
    staged.remove().expect("remove staged runtime");
    assert!(!launch_directory.join("runtime.mail").exists());
    std::fs::remove_dir_all(root).expect("remove staged fixture");
}

#[test]
fn staged_native_artifact_removes_a_copy_with_the_wrong_digest() {
    let root = std::env::temp_dir().join(format!(
        "makosh-staged-artifact-reject-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let source = root.join("signed-runtime");
    let launch_directory = root.join("runtime/launch");
    std::fs::create_dir_all(&root).expect("create fixture root");
    std::fs::write(&source, b"actual runtime").expect("write source artifact");

    assert_eq!(
        staged_native_artifact::stage(&source, &launch_directory, "runtime.mail", &[0; 32])
            .expect_err("wrong digest"),
        "staged artifact digest does not match manifest"
    );
    assert!(!launch_directory.join("runtime.mail").exists());
    std::fs::remove_dir_all(root).expect("remove rejected fixture");
}

#[test]
fn bounded_managed_child_execution_retries_only_within_its_explicit_budget() {
    let root = std::env::temp_dir().join(format!(
        "makosh-managed-child-execution-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let executable = root.join("child.sh");
    let state = root.join("first-attempt");
    std::fs::create_dir_all(&root).expect("create fixture root");
    std::fs::write(
        &executable,
        "#!/bin/sh\nif [ -e \"$1\" ]; then exit 0; fi\n: > \"$1\"\nexit 1\n",
    )
    .expect("write child");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o500))
        .expect("make child executable");
    let policy =
        ManagedChildExecutionPolicy::new(2, Duration::from_secs(2)).expect("execution policy");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&executable).expect("read executable for staging")).into();
    let staged =
        staged_native_artifact::stage(&executable, &root.join("launch"), "managed-child", &digest)
            .expect("stage executable");
    let result =
        bounded_managed_child_execution::run(&staged, &[state.display().to_string()], &policy)
            .expect("second attempt succeeds");

    assert_eq!(result.attempts(), 2);
    assert_eq!(result.exit_code(), 0);
    std::fs::remove_dir_all(root).expect("remove execution fixture");
}

#[test]
fn bounded_managed_child_execution_terminates_a_stalled_child() {
    let root = std::env::temp_dir().join(format!(
        "makosh-managed-child-timeout-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let executable = root.join("child.sh");
    std::fs::create_dir_all(&root).expect("create fixture root");
    std::fs::write(&executable, "#!/bin/sh\nwhile :; do :; done\n").expect("write child");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o500))
        .expect("make child executable");
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(1)).expect("execution policy");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&executable).expect("read executable for staging")).into();
    let staged =
        staged_native_artifact::stage(&executable, &root.join("launch"), "managed-child", &digest)
            .expect("stage executable");

    assert_eq!(
        bounded_managed_child_execution::run(&staged, &[], &policy).expect_err("timeout"),
        "managed child exceeded its bounded runtime"
    );
    std::fs::remove_dir_all(root).expect("remove timeout fixture");
}

#[test]
fn creates_and_reopens_a_trustworthy_control_store() {
    let path = std::env::temp_dir().join(format!(
        "makosh-control-store-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    let created = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
    assert_eq!(created.snapshot().health(), StoreHealth::Trustworthy);

    let reopened = SqliteControlStore::open(&path).expect("open store");
    assert_eq!(reopened.snapshot().instance_id(), "instance-1");
    assert_eq!(reopened.snapshot().generation(), 1);

    std::fs::remove_file(path).expect("remove temporary store");
}

#[test]
fn recovery_fences_advance_monotonically() {
    let path = std::env::temp_dir().join(format!(
        "makosh-control-store-fences-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");

    let advanced = store.advance_recovery_fences().expect("advance fences");
    assert_eq!(advanced.generation(), 2);
    assert_eq!(advanced.identity_epoch(), 2);
    assert_eq!(advanced.grant_epoch(), 2);

    let reopened = SqliteControlStore::open(&path).expect("open store");
    assert_eq!(reopened.snapshot().generation(), 2);
    assert_eq!(reopened.snapshot().identity_epoch(), 2);
    assert_eq!(reopened.snapshot().grant_epoch(), 2);
    std::fs::remove_file(path).expect("remove temporary store");
}

#[test]
fn staged_restore_suspends_authority_and_replaces_every_fence() {
    let root = std::env::temp_dir().join(format!(
        "makosh-staged-control-store-restore-{}",
        std::process::id()
    ));
    let (source, staged) = prepare_staged_restore_fixture(&root);
    let prepared = StagedControlStoreRestore::prepare(
        &source,
        &staged,
        "11111111111111111111111111111111",
        RecoveryFences::new(13, 9, 11),
    )
    .expect("prepare fenced restore");
    assert_eq!(prepared.snapshot().generation(), 13);
    assert_staged_authority_is_suspended(&staged);
    std::fs::remove_dir_all(root).expect("remove restore fixture");
}

fn prepare_staged_restore_fixture(root: &Path) -> (PathBuf, PathBuf) {
    let source = root.join("source.sqlite");
    let staged = root.join("staged.sqlite");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir(root).expect("create restore fixture");
    let store = SqliteControlStore::create(&source, "11111111111111111111111111111111", 1)
        .expect("create source store");
    let registration = ModuleRegistration::new(
        "registration-restore",
        "module-restore",
        "owner-restore",
        [7; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &["capability.read".to_owned()])
        .expect("register source module");
    store
        .approve_module_registration("registration-restore", &["capability.read".to_owned()])
        .expect("approve source module");
    store
        .attest_external_runtime(&ExternalRuntimeAttestation::new(
            "registration-restore",
            "runtime-restore",
            1,
            2,
            [8; 32],
        ))
        .expect("attest source runtime");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            "registration-restore",
            1,
            "distribution-restore",
            "artifact-restore",
            [9; 32],
            [7; 32],
            None,
        ))
        .expect("bind source launch");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            "registration-restore",
            "runtime-restore",
            1,
            1,
            1,
            2,
        ))
        .expect("record source launch");
    (source, staged)
}

fn assert_staged_authority_is_suspended(staged: &Path) {
    let restored = SqliteControlStore::open(staged).expect("open staged restore");
    let restored_registration = restored
        .module_registration("registration-restore")
        .expect("read restored registration")
        .expect("restored registration");
    assert_eq!(
        restored_registration.state(),
        ModuleRegistrationState::Suspended
    );
    assert_eq!(restored_registration.grant_epoch(), 11);
    assert!(
        restored
            .module_grant_snapshot("registration-restore")
            .expect("read restored grants")
            .and_then(|snapshot| snapshot.effective_grants().cloned())
            .is_none()
    );
    assert!(
        restored
            .effective_external_runtime_attestation("registration-restore")
            .expect("read restored attestation")
            .is_none()
    );
    assert!(
        restored
            .effective_managed_launch_record("registration-restore")
            .expect("read restored launch")
            .is_none()
    );
}

#[test]
fn initial_owner_claim_is_atomic_and_keeps_only_the_public_key() {
    let path = std::env::temp_dir().join(format!(
        "makosh-control-store-owner-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
    let public_key = [
        0x04, 0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6, 0xe5, 0x63, 0xa4,
        0x40, 0xf2, 0x77, 0x03, 0x7d, 0x81, 0x2d, 0xeb, 0x33, 0xa0, 0xf4, 0xa1, 0x39, 0x45, 0xd8,
        0x98, 0xc2, 0x96, 0x4f, 0xe3, 0x42, 0xe2, 0xfe, 0x1a, 0x7f, 0x9b, 0x8e, 0xe7, 0xeb, 0x4a,
        0x7c, 0x0f, 0x9e, 0x16, 0x2b, 0xce, 0x33, 0x57, 0x6b, 0x31, 0x5e, 0xce, 0xcb, 0xb6, 0x40,
        0x68, 0x37, 0xbf, 0x51, 0xf5,
    ];
    let first = InitialOwnerIdentity::new("owner-1", "device-1", public_key);
    store
        .claim_initial_owner(&first)
        .expect("claim first owner");
    assert_eq!(
        store.initial_owner_identity().expect("read owner"),
        Some(first.clone())
    );
    assert!(
        store
            .claim_initial_owner(&InitialOwnerIdentity::new(
                "owner-2", "device-2", public_key
            ))
            .is_err()
    );
    std::fs::remove_file(path).expect("remove temporary store");
}

#[test]
fn server_bootstrap_pairing_claims_the_initial_owner_once_and_rejects_replay() {
    let path = std::env::temp_dir().join(format!(
        "makosh-control-store-server-pairing-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
    let signing_key = SigningKey::from_bytes((&[29_u8; 32]).into()).expect("test key");
    let public_key_sec1: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed P-256 key");
    let pairing = ServerBootstrapPairing::new([1; 32], [2; 32], [3; 32], 2_000);
    store
        .begin_server_bootstrap_pairing(&pairing, 1_000)
        .expect("begin pairing");
    let identity = InitialOwnerIdentity::new("owner-1", "device-1", public_key_sec1);
    assert!(matches!(
        store
            .claim_initial_owner_from_server_bootstrap_pairing(&identity, &[9; 32], 1_001)
            .expect_err("wrong token"),
        StoreError::ServerBootstrapPairingTokenRejected
    ));
    store
        .claim_initial_owner_from_server_bootstrap_pairing(&identity, &[1; 32], 1_001)
        .expect("claim paired owner");
    assert_eq!(
        store.initial_owner_identity().expect("read owner"),
        Some(identity)
    );
    assert!(matches!(
        store
            .claim_initial_owner_from_server_bootstrap_pairing(
                &InitialOwnerIdentity::new("owner-2", "device-2", public_key_sec1),
                &[1; 32],
                1_001,
            )
            .expect_err("replayed pairing"),
        StoreError::InitialOwnerAlreadyClaimed
    ));
    std::fs::remove_file(path).expect("remove temporary store");
}

#[test]
fn server_bootstrap_pairing_expires_before_an_owner_claim() {
    let path = std::env::temp_dir().join(format!(
        "makosh-control-store-server-pairing-expiry-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
    let signing_key = SigningKey::from_bytes((&[31_u8; 32]).into()).expect("test key");
    let public_key_sec1: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed P-256 key");
    let pairing = ServerBootstrapPairing::new([4; 32], [5; 32], [6; 32], 2_000);
    store
        .begin_server_bootstrap_pairing(&pairing, 1_000)
        .expect("begin pairing");
    assert!(matches!(
        store
            .claim_initial_owner_from_server_bootstrap_pairing(
                &InitialOwnerIdentity::new("owner-1", "device-1", public_key_sec1),
                &[4; 32],
                2_000,
            )
            .expect_err("expired pairing"),
        StoreError::ServerBootstrapPairingExpired
    ));
    assert!(
        store
            .initial_owner_identity()
            .expect("read owner")
            .is_none()
    );
    std::fs::remove_file(path).expect("remove temporary store");
}
