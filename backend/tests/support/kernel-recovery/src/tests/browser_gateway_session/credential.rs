use super::*;

#[test]
fn browser_credential_is_resolved_only_while_its_device_is_active() {
    let (root, store, cose_public_key) = active_browser_credential_fixture();
    assert_active_browser_credential(&store, &cose_public_key);
    assert_browser_authentication_route(&store);
    store
        .revoke_browser_device("browser-1", 1)
        .expect("revoke browser");
    assert_browser_credential_is_inactive(&store);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn active_browser_credential_fixture() -> (std::path::PathBuf, Arc<SqliteControlStore>, Vec<u8>) {
    let root = unique_target_root("makosh-browser-gateway-session");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-browser", 1)
            .expect("create store"),
    );
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let cose_public_key = valid_browser_cose_key();
    store
        .admit_browser_device(
            &BrowserDeviceEnrollmentV1::new(
                makosh_kernel_control_store::BrowserDeviceEnrollmentInputV1 {
                    owner_id: "owner-1".to_owned(),
                    device_id: "browser-1".to_owned(),
                    credential_id: vec![1],
                    cose_public_key: cose_public_key.clone(),
                    browser_key_public_key: valid_browser_local_key(),
                    rp_id: "hub.local".to_owned(),
                    sign_count: 0,
                    backup_eligible: true,
                    backup_state: true,
                },
            )
            .expect("valid browser enrollment"),
            1,
        )
        .expect("admit browser");
    (root, store, cose_public_key)
}

fn assert_active_browser_credential(store: &Arc<SqliteControlStore>, cose_public_key: &[u8]) {
    let authority = browser_authority(Arc::clone(store));
    let principal = authority
        .active_browser_device_by_credential(&[1])
        .expect("active browser credential");
    assert_eq!(principal.owner_id(), "owner-1");
    let credential = authority
        .active_browser_credential(&[1])
        .expect("active browser credential material");
    assert_eq!(credential.credential_id(), [1]);
    assert_eq!(credential.cose_public_key(), cose_public_key);
    assert!(credential.backup_eligible());
    assert!(credential.backup_state());
}

fn assert_browser_authentication_route(store: &Arc<SqliteControlStore>) {
    let verifier =
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("verifier");
    let service = BrowserGatewaySessionService::new(
        browser_authority(Arc::clone(store)),
        verifier,
        "https://hub.local",
    )
    .expect("browser session service");
    let router = BrowserAuthenticationRouter::new(service);
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let response =
        runtime.block_on(router.route(begin_authentication_request("https://hub.local")));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .expect("cache control"),
        "no-store"
    );
    assert!(
        response
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
    let body = runtime
        .block_on(response.into_body().collect())
        .expect("response body")
        .to_bytes();
    assert!(
        std::str::from_utf8(&body)
            .expect("response JSON")
            .contains("authentication_id")
    );
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Full::new(Bytes::new()))
                .expect("unrelated gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let response =
        runtime.block_on(router.route(begin_authentication_request("https://other.local")));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

fn assert_browser_credential_is_inactive(store: &Arc<SqliteControlStore>) {
    let authority = browser_authority(Arc::clone(store));
    assert!(authority.active_browser_device_by_credential(&[1]).is_err());
    assert!(authority.active_browser_credential(&[1]).is_err());
}
