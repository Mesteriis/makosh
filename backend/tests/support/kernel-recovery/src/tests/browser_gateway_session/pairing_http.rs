use super::*;

#[test]
fn browser_pairing_http_exposes_only_an_owner_approved_ceremony_and_keeps_it_after_origin_reject() {
    let fixture = pairing_http_fixture();
    assert_unavailable_pairing_is_rejected(&fixture);
    assert_pairing_options_are_origin_bound(&fixture);
    assert_wrong_origin_keeps_pairing_available(&fixture);
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

struct PairingHttpFixture {
    root: std::path::PathBuf,
    pairing_id: String,
    router: GatewayApplicationRouter<ControlStoreBrowserAuthority, ClosedReplaySource>,
    runtime: tokio::runtime::Runtime,
}

fn pairing_http_fixture() -> PairingHttpFixture {
    let root = unique_target_root("makosh-browser-pairing-http");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store =
        Arc::new(SqliteControlStore::create(&path, "instance-browser", 1).expect("create store"));
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let authority = browser_authority(Arc::clone(&store));
    let pairings = Arc::new(std::sync::Mutex::new(BrowserPairingManager::default()));
    let ceremony = pairings
        .lock()
        .expect("pairing lock")
        .begin_webauthn(
            &authority,
            &BrowserWebauthnVerifier::new("hub.local", "https://hub.local")
                .expect("pairing verifier"),
            OwnerPairingApprovalV1::new("owner-1", "desktop-1").expect("pairing approval"),
            1_000,
        )
        .expect("owner-approved pairing");
    let pairing_id = ceremony.pairing().pairing_id().to_owned();
    let session = BrowserGatewaySessionService::new(
        authority.clone(),
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("session verifier"),
        "https://hub.local",
    )
    .expect("browser session service");
    let router = GatewayApplicationRouter::new(true, Arc::new(session), ClosedReplaySource)
        .with_browser_pairing(BrowserPairingRouter::new(
            Arc::clone(&pairings),
            authority,
            BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("route verifier"),
            "https://hub.local",
        ));
    PairingHttpFixture {
        root,
        pairing_id,
        router,
        runtime: tokio::runtime::Runtime::new().expect("test runtime"),
    }
}

fn assert_unavailable_pairing_is_rejected(fixture: &PairingHttpFixture) {
    let unavailable = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration",
                    "f".repeat(64)
                ))
                .body(Full::new(Bytes::new()))
                .expect("unavailable pairing request"),
        ),
    );
    assert_eq!(unavailable.status(), StatusCode::UNAUTHORIZED);
}

fn assert_pairing_options_are_origin_bound(fixture: &PairingHttpFixture) {
    let options = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration",
                    fixture.pairing_id
                ))
                .body(Full::new(Bytes::new()))
                .expect("pairing options request"),
        ),
    );
    assert_eq!(options.status(), StatusCode::OK);
    assert_eq!(
        options.headers().get("cache-control"),
        Some(&"no-store".parse().unwrap())
    );
    let options_body = fixture
        .runtime
        .block_on(options.into_body().collect())
        .expect("options body")
        .to_bytes();
    let options: serde_json::Value = serde_json::from_slice(&options_body).expect("options JSON");
    assert_eq!(options["public_key"]["publicKey"]["rp"]["id"], "hub.local");
}

fn assert_wrong_origin_keeps_pairing_available(fixture: &PairingHttpFixture) {
    let rejected = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration/finish",
                    fixture.pairing_id
                ))
                .header("content-type", "application/json")
                .header("origin", "https://other.local")
                .body(Full::new(Bytes::from_static(b"{}")))
                .expect("wrong-origin registration request"),
        ),
    );
    assert_eq!(rejected.status(), StatusCode::FORBIDDEN);
    let still_available = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/browser/v1/pairing/{}/registration",
                    fixture.pairing_id
                ))
                .body(Full::new(Bytes::new()))
                .expect("pairing options retry"),
        ),
    );
    assert_eq!(still_available.status(), StatusCode::OK);
}
