use super::*;

#[test]
fn browser_gateway_authority_persists_the_verified_assertion_counter() {
    let root = unique_target_root("makosh-browser-gateway-assertion");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store =
        Arc::new(SqliteControlStore::create(&path, "instance-browser", 1).expect("create store"));
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let enrollment = BrowserDeviceEnrollmentV1::new(
        makosh_kernel_control_store::BrowserDeviceEnrollmentInputV1 {
            owner_id: "owner-1".to_owned(),
            device_id: "browser-1".to_owned(),
            credential_id: vec![1],
            cose_public_key: vec![2; 16],
            browser_key_public_key: vec![4; 65],
            rp_id: "hub.local".to_owned(),
            sign_count: 0,
            backup_eligible: false,
            backup_state: false,
        },
    )
    .expect("valid browser enrollment");
    store
        .admit_browser_device(&enrollment, 1)
        .expect("admit browser");
    let authority = browser_authority(Arc::clone(&store));
    authority
        .accept_verified_browser_assertion(&[1], 3, false, false)
        .expect("persist verified assertion");
    let identity = store
        .browser_device_identity("browser-1")
        .expect("read browser device")
        .expect("browser device exists");
    assert_eq!(identity.enrollment().sign_count(), 3);
    assert!(
        authority
            .accept_verified_browser_assertion(&[1], 3, false, false)
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}
