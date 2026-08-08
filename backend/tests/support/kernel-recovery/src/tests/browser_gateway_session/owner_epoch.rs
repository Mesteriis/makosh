use super::*;

#[test]
fn browser_pairing_binds_its_owner_epoch_fence_to_the_webauthn_ceremony() {
    let root = unique_target_root("makosh-browser-webauthn-pairing");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-browser", 1)
            .expect("create store"),
    );
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let authority = browser_authority(Arc::clone(&store));
    let verifier =
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("verifier");
    let ceremony = BrowserPairingManager::default()
        .begin_webauthn(
            &authority,
            &verifier,
            OwnerPairingApprovalV1::new("owner-1", "desktop-1").expect("approval"),
            1_000,
        )
        .expect("begin pairing");
    assert_eq!(ceremony.pairing().owner_id(), "owner-1");
    assert_eq!(ceremony.pairing().rp_id(), "hub.local");
    assert_eq!(ceremony.options().public_key.rp.id, "hub.local");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}
