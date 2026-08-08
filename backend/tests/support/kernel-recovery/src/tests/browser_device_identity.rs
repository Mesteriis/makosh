use super::common::*;
use makosh_kernel_control_store::{
    BrowserDeviceEnrollmentInputV1, BrowserDeviceEnrollmentV1, BrowserDeviceStateV1,
};

#[test]
fn browser_device_identity_is_admitted_and_revoked_with_an_epoch_fence() {
    let root = unique_target_root("makosh-browser-device-identity");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store = SqliteControlStore::create(&path, "instance-browser", 1).expect("create store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let enrollment = browser_enrollment("owner-1", "browser-1", 1);

    let admitted = store
        .admit_browser_device(&enrollment, 1)
        .expect("admit browser identity");
    assert_eq!(admitted.state(), BrowserDeviceStateV1::Active);
    assert_eq!(admitted.identity_epoch(), 1);
    assert!(matches!(
        store
            .admit_browser_device(&enrollment, 1)
            .expect_err("duplicate browser credential"),
        StoreError::BrowserDeviceAlreadyExists
    ));
    assert!(matches!(
        store
            .revoke_browser_device("browser-1", 2)
            .expect_err("stale epoch"),
        StoreError::BrowserDeviceIdentityEpochConflict
    ));

    let updated = store
        .revoke_browser_device("browser-1", 1)
        .expect("revoke browser identity");
    assert_eq!(updated.identity_epoch(), 2);
    assert_eq!(
        store.current_identity_epoch().expect("read current epoch"),
        2
    );
    let revoked = store
        .browser_device_identity("browser-1")
        .expect("read browser identity")
        .expect("browser identity exists");
    assert_eq!(revoked.state(), BrowserDeviceStateV1::Revoked);
    assert_eq!(revoked.identity_epoch(), 2);

    let reopened = SqliteControlStore::open(&path).expect("reopen store");
    assert_eq!(reopened.snapshot().identity_epoch(), 2);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn browser_device_identity_rejects_foreign_owner_and_malformed_registration() {
    let root = unique_target_root("makosh-browser-device-owner");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store = SqliteControlStore::create(&path, "instance-browser", 1).expect("create store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    assert!(
        BrowserDeviceEnrollmentV1::new(BrowserDeviceEnrollmentInputV1 {
            owner_id: "owner-1".to_owned(),
            device_id: "browser-1".to_owned(),
            credential_id: vec![1; 16],
            cose_public_key: vec![2; 16],
            browser_key_public_key: vec![4; 65],
            rp_id: "invalid".to_owned(),
            sign_count: 0,
            backup_eligible: false,
            backup_state: false,
        })
        .is_err()
    );
    assert!(matches!(
        store
            .admit_browser_device(&browser_enrollment("owner-2", "browser-1", 1), 1)
            .expect_err("foreign owner"),
        StoreError::BrowserDeviceOwnerMismatch
    ));
    let short_credential = BrowserDeviceEnrollmentV1::new(BrowserDeviceEnrollmentInputV1 {
        owner_id: "owner-1".to_owned(),
        device_id: "browser-2".to_owned(),
        credential_id: vec![9],
        cose_public_key: vec![3; 16],
        browser_key_public_key: vec![4; 65],
        rp_id: "hub.local".to_owned(),
        sign_count: 0,
        backup_eligible: false,
        backup_state: false,
    })
    .expect("WebAuthn credential identifiers may be short");
    store
        .admit_browser_device(&short_credential, 1)
        .expect("admit short credential identifier");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn browser_assertion_counter_is_durable_and_never_regresses_after_initial_zero() {
    let root = unique_target_root("makosh-browser-device-counter");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store = SqliteControlStore::create(&path, "instance-browser", 1).expect("create store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let enrollment = browser_enrollment("owner-1", "browser-1", 7);
    store
        .admit_browser_device(&enrollment, 1)
        .expect("admit browser identity");
    let zero = store
        .record_verified_browser_assertion(enrollment.credential_id(), 0, false, false, 1)
        .expect("zero-counter authenticator remains valid");
    assert_eq!(zero.enrollment().sign_count(), 0);
    let advanced = store
        .record_verified_browser_assertion(enrollment.credential_id(), 7, true, true, 1)
        .expect("advance browser assertion counter");
    assert_eq!(advanced.enrollment().sign_count(), 7);
    assert!(advanced.enrollment().backup_eligible());
    assert!(advanced.enrollment().backup_state());
    assert!(matches!(
        store
            .record_verified_browser_assertion(enrollment.credential_id(), 7, false, false, 1)
            .expect_err("replayed assertion"),
        StoreError::BrowserDeviceCounterConflict
    ));
    assert!(matches!(
        store
            .record_verified_browser_assertion(enrollment.credential_id(), 8, false, false, 2)
            .expect_err("stale identity epoch"),
        StoreError::BrowserDeviceIdentityEpochConflict
    ));
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn browser_enrollment(owner_id: &str, device_id: &str, marker: u8) -> BrowserDeviceEnrollmentV1 {
    BrowserDeviceEnrollmentV1::new(BrowserDeviceEnrollmentInputV1 {
        owner_id: owner_id.to_owned(),
        device_id: device_id.to_owned(),
        credential_id: vec![marker; 16],
        cose_public_key: vec![marker.wrapping_add(1); 16],
        browser_key_public_key: vec![4; 65],
        rp_id: "hub.local".to_owned(),
        sign_count: 0,
        backup_eligible: false,
        backup_state: false,
    })
    .expect("valid browser enrollment")
}
