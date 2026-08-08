use super::*;

use makosh_gateway_protocol::v1::{
    BrowserGatewayAccessModeV1, BrowserSessionStatusResponseV1, ClientBootstrapResponseV1,
    ClientSettingsApplyStateV1, ClientSurfaceAvailabilityStateV1, ClientSurfaceIdV1,
    ClientSystemComponentIdV1, ClientSystemComponentStateV1,
    client_setting_value_v1::Value as GatewayValue,
};
use makosh_kernel_control_store::{
    ModuleRegistration, ModuleRegistrationState, SettingsApplyState, SettingsDesiredSnapshot,
    SettingsInitialSnapshot, SettingsSchemaBinding,
};
use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingValueV1, SettingsSchemaV1, SettingsSnapshotV1,
    SettingsValueEntryV1, setting_value_v1::Value,
};
use prost::Message;
use sha2::{Digest, Sha256};

const SESSION_STATUS_PATH: &str = "/makosh.gateway.v1.BrowserSessionService/GetStatus";
const BOOTSTRAP_PATH: &str = "/makosh.gateway.v1.ClientBootstrapService/GetBootstrap";

#[test]
fn browser_connect_session_status_is_cookie_only_and_owner_neutral() {
    let fixture = authentication_http_fixture();
    let unauthenticated = fixture
        .runtime
        .block_on(fixture.router.route(session_status_request(None)));
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauthenticated.headers().get("connect-error-code"),
        Some(&"unauthenticated".parse().expect("header value"))
    );

    let (authentication_id, challenge, browser_key_challenge) =
        begin_browser_authentication(&fixture);
    let cookie = finish_browser_authentication(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    let response = fixture
        .runtime
        .block_on(fixture.router.route(session_status_request(Some(&cookie))));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type"),
        Some(&"application/proto".parse().unwrap())
    );
    assert_eq!(
        response.headers().get("cache-control"),
        Some(&"no-store".parse().unwrap())
    );
    assert!(response.headers().get("set-cookie").is_none());
    let body = fixture
        .runtime
        .block_on(response.into_body().collect())
        .expect("Connect response body")
        .to_bytes();
    let status = BrowserSessionStatusResponseV1::decode(body).expect("typed Connect response");
    assert_eq!(status.major, 1);
    assert!(status.session_expires_at_unix_millis > 0);
    assert_eq!(
        status.access_mode,
        BrowserGatewayAccessModeV1::Paired as i32
    );
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

#[test]
fn lan_development_mode_bypasses_cookie_only_for_direct_same_origin_requests() {
    let root = unique_target_root("makosh-browser-gateway-lan-development");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-development", 1)
            .expect("create store"),
    );
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let service = BrowserGatewaySessionService::new_lan_development(
        browser_authority(Arc::clone(&store)),
        "http://192.168.1.10:9443",
        "owner-1",
        "desktop-1",
    )
    .expect("development session service");
    let router = GatewayApplicationRouter::new(true, Arc::new(service), ClosedReplaySource)
        .with_lan_development_policy("http://192.168.1.10:9443")
        .expect("development policy");
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");

    let status_response = runtime.block_on(router.route(session_status_request_with_admission(
        None,
        Some("192.168.1.10:9443"),
        None,
    )));
    assert_eq!(status_response.status(), StatusCode::OK);
    let body = runtime
        .block_on(status_response.into_body().collect())
        .expect("status body")
        .to_bytes();
    let status = BrowserSessionStatusResponseV1::decode(body).expect("typed status");
    assert_eq!(
        status.access_mode,
        BrowserGatewayAccessModeV1::LanDevelopment as i32
    );

    assert_eq!(
        runtime
            .block_on(router.route(session_status_request_with_admission(None, None, None)))
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        runtime
            .block_on(router.route(session_status_request_with_admission(
                None,
                Some("192.168.1.10:9443"),
                Some(("x-forwarded-for", "203.0.113.4")),
            )))
            .status(),
        StatusCode::FORBIDDEN
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn loopback_development_mode_requires_the_exact_private_proxy_hop() {
    const PROOF: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let root = unique_target_root("makosh-browser-gateway-loopback-development");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-development", 1)
            .expect("create store"),
    );
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    let service = BrowserGatewaySessionService::new_loopback_development(
        browser_authority(Arc::clone(&store)),
        "http://127.0.0.1:5173",
        "owner-1",
        "desktop-1",
    )
    .expect("loopback development session service");
    let router = GatewayApplicationRouter::new(true, Arc::new(service), ClosedReplaySource)
        .with_loopback_development_proxy_policy("http://127.0.0.1:5173", PROOF)
        .expect("loopback development proxy policy");
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let admitted_headers = [
        ("host", "127.0.0.1:5173"),
        ("origin", "http://127.0.0.1:5173"),
        ("x-makosh-development-proxy-proof", PROOF),
    ];

    let status_response = runtime
        .block_on(router.route(session_status_request_with_headers(None, &admitted_headers)));
    assert_eq!(status_response.status(), StatusCode::OK);
    let body = runtime
        .block_on(status_response.into_body().collect())
        .expect("status body")
        .to_bytes();
    let status = BrowserSessionStatusResponseV1::decode(body).expect("typed status");
    assert_eq!(
        status.access_mode,
        BrowserGatewayAccessModeV1::LocalDevelopment as i32
    );

    for rejected_headers in [
        vec![
            ("host", "127.0.0.1:5173"),
            ("origin", "http://127.0.0.1:5173"),
        ],
        vec![
            ("host", "127.0.0.1:5173"),
            ("origin", "http://127.0.0.1:5173"),
            ("x-makosh-development-proxy-proof", "bad"),
        ],
        vec![
            ("host", "127.0.0.1:9444"),
            ("origin", "http://127.0.0.1:5173"),
            ("x-makosh-development-proxy-proof", PROOF),
        ],
        vec![
            ("host", "127.0.0.1:5173"),
            ("origin", "http://127.0.0.1:9444"),
            ("x-makosh-development-proxy-proof", PROOF),
        ],
        vec![
            ("host", "127.0.0.1:5173"),
            ("origin", "http://127.0.0.1:5173"),
            ("x-makosh-development-proxy-proof", PROOF),
            ("x-forwarded-for", "203.0.113.4"),
        ],
    ] {
        assert_eq!(
            runtime
                .block_on(
                    router.route(session_status_request_with_headers(None, &rejected_headers,))
                )
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn browser_bootstrap_contains_the_logical_owners_approved_module_composition() {
    let fixture = authentication_http_fixture();
    admit_bootstrap_modules(&fixture);
    let (authentication_id, challenge, browser_key_challenge) =
        begin_browser_authentication(&fixture);
    let cookie = finish_browser_authentication(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    let bootstrap = read_bootstrap(&fixture, Some(&cookie));
    assert_current_owner_bootstrap(&bootstrap);

    advance_visible_settings_revision(&fixture);
    let pending = read_bootstrap(&fixture, Some(&cookie));
    assert_pending_owner_bootstrap(&pending);
    assert_bootstrap_requires_session(&fixture);
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

fn assert_current_owner_bootstrap(bootstrap: &ClientBootstrapResponseV1) {
    assert_eq!(bootstrap.modules.len(), 3);
    let module = bootstrap
        .modules
        .iter()
        .find(|module| module.module_id == "module-visible")
        .expect("visible module");
    assert_eq!(module.module_id, "module-visible");
    assert!(module.sections_enabled);
    assert_eq!(
        module.capability_ids,
        ["client.surface.personas.v1", "sections.read"]
    );
    let settings = module.settings.as_ref().expect("settings package");
    assert_eq!(settings.desired_revision, 1);
    assert_eq!(settings.effective_revision, 1);
    assert_eq!(settings.values.len(), 1);
    assert_eq!(settings.values[0].setting_id, "visible.flag");
    assert_eq!(
        settings.values[0]
            .value
            .as_ref()
            .expect("typed value")
            .value,
        Some(GatewayValue::BooleanValue(true))
    );
    assert_current_surface_availability(bootstrap);
    assert_eq!(bootstrap.system_status.len(), 15);
    assert!(bootstrap.system_status.iter().any(|status| {
        status.component_id == ClientSystemComponentIdV1::Kernel as i32
            && status.state == ClientSystemComponentStateV1::Healthy as i32
    }));
    assert!(bootstrap.system_status.iter().any(|status| {
        status.component_id == ClientSystemComponentIdV1::Sse as i32
            && status.state == ClientSystemComponentStateV1::Unavailable as i32
            && status.sanitized_reason_code == "client_realtime_owner_not_admitted"
    }));
}

fn assert_current_surface_availability(bootstrap: &ClientBootstrapResponseV1) {
    assert_eq!(bootstrap.surfaces.len(), 13);
    let catalog_module = bootstrap
        .modules
        .iter()
        .find(|module| module.module_id == "module-catalog")
        .expect("catalog module");
    assert!(
        catalog_module.sections_enabled,
        "current account-scoped target must enable catalog-backed sections; capabilities={:?}, settings={:?}",
        catalog_module.capability_ids, catalog_module.settings
    );
    assert_eq!(catalog_module.settings_targets.len(), 2);
    let current_target = catalog_module
        .settings_targets
        .iter()
        .find(|target| target.configuration_instance_id == "configuration-current")
        .expect("current client-safe configuration target");
    assert_eq!(current_target.desired_revision, 1);
    assert_eq!(current_target.effective_revision, 1);
    assert_eq!(current_target.values.len(), 1);
    assert_eq!(current_target.values[0].setting_id, "visible.flag");
    let settings_surface = bootstrap
        .surfaces
        .iter()
        .find(|surface| surface.surface_id == ClientSurfaceIdV1::Settings as i32)
        .expect("Settings surface");
    assert_eq!(
        settings_surface.state,
        ClientSurfaceAvailabilityStateV1::Available as i32
    );
    assert!(settings_surface.sanitized_reason_code.is_empty());
    assert_eq!(settings_surface.supported_client_contract_major, 1);
    let personas_surface = bootstrap
        .surfaces
        .iter()
        .find(|surface| surface.surface_id == ClientSurfaceIdV1::Personas as i32)
        .expect("Personas surface");
    assert_eq!(
        personas_surface.state,
        ClientSurfaceAvailabilityStateV1::Available as i32
    );
    assert!(personas_surface.sanitized_reason_code.is_empty());
    let mail_surface = bootstrap
        .surfaces
        .iter()
        .find(|surface| surface.surface_id == ClientSurfaceIdV1::Mail as i32)
        .expect("Mail surface");
    assert_eq!(
        mail_surface.state,
        ClientSurfaceAvailabilityStateV1::Available as i32
    );
    assert!(mail_surface.sanitized_reason_code.is_empty());
    for surface in bootstrap.surfaces.iter().filter(|surface| {
        !matches!(
            surface.surface_id,
            value if value == ClientSurfaceIdV1::Settings as i32
                || value == ClientSurfaceIdV1::Personas as i32
                || value == ClientSurfaceIdV1::Mail as i32
        )
    }) {
        assert_eq!(
            surface.state,
            ClientSurfaceAvailabilityStateV1::NotAdmitted as i32
        );
        assert_eq!(surface.sanitized_reason_code, "surface_not_admitted");
        assert_eq!(surface.supported_client_contract_major, 1);
    }
}

fn advance_visible_settings_revision(fixture: &AuthenticationHttpFixture) {
    fixture
        .store
        .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
            registration_id: "registration-visible".to_owned(),
            configuration_instance_id: "registration-visible".to_owned(),
            expected_revision: 1,
            snapshot_bytes: current_snapshot("registration-visible", 2).encode_to_vec(),
        })
        .expect("advance desired settings");
}

fn assert_pending_owner_bootstrap(pending: &ClientBootstrapResponseV1) {
    let module = pending
        .modules
        .iter()
        .find(|module| module.module_id == "module-visible")
        .expect("visible module");
    assert!(!module.sections_enabled);
    assert!(
        module
            .settings
            .as_ref()
            .expect("settings package")
            .values
            .is_empty()
    );
    let pending_personas_surface = pending
        .surfaces
        .iter()
        .find(|surface| surface.surface_id == ClientSurfaceIdV1::Personas as i32)
        .expect("pending Personas surface");
    assert_eq!(
        pending_personas_surface.state,
        ClientSurfaceAvailabilityStateV1::Blocked as i32
    );
    assert_eq!(
        pending_personas_surface.sanitized_reason_code,
        "surface_settings_not_current"
    );
    let catalog_module = pending
        .modules
        .iter()
        .find(|module| module.module_id == "module-catalog")
        .expect("catalog module");
    assert!(catalog_module.sections_enabled);
    let settings = catalog_module.settings.as_ref().expect("settings package");
    assert_eq!(
        settings.apply_state,
        ClientSettingsApplyStateV1::BlockedConfig as i32
    );
    assert!(settings.values.is_empty());
    let mail_surface = pending
        .surfaces
        .iter()
        .find(|surface| surface.surface_id == ClientSurfaceIdV1::Mail as i32)
        .expect("Mail surface");
    assert_eq!(
        mail_surface.state,
        ClientSurfaceAvailabilityStateV1::Available as i32
    );
    assert!(mail_surface.sanitized_reason_code.is_empty());
}

fn assert_bootstrap_requires_session(fixture: &AuthenticationHttpFixture) {
    assert_eq!(
        read_bootstrap_status(fixture, None),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        read_bootstrap_status(fixture, Some("__Host-makosh-session=invalid")),
        StatusCode::UNAUTHORIZED
    );
}

fn session_status_request(cookie: Option<&str>) -> Request<Full<Bytes>> {
    session_status_request_with_admission(cookie, None, None)
}

fn session_status_request_with_admission(
    cookie: Option<&str>,
    host: Option<&str>,
    extra_header: Option<(&str, &str)>,
) -> Request<Full<Bytes>> {
    let mut headers = Vec::new();
    if let Some(host) = host {
        headers.push(("host", host));
    }
    if let Some(extra_header) = extra_header {
        headers.push(extra_header);
    }
    session_status_request_with_headers(cookie, &headers)
}

fn session_status_request_with_headers(
    cookie: Option<&str>,
    headers: &[(&str, &str)],
) -> Request<Full<Bytes>> {
    let mut request = Request::builder()
        .method("POST")
        .uri(SESSION_STATUS_PATH)
        .header("content-type", "application/proto")
        .header("connect-protocol-version", "1");
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    for &(name, value) in headers {
        request = request.header(name, value);
    }
    request
        .body(Full::new(Bytes::new()))
        .expect("Connect status request")
}

fn admit_bootstrap_modules(fixture: &AuthenticationHttpFixture) {
    admit_owner_scoped_modules(fixture);
    admit_current_visible_settings(fixture);
}

fn admit_owner_scoped_modules(fixture: &AuthenticationHttpFixture) {
    for (registration_id, module_id, owner_id) in [
        ("registration-visible", "module-visible", "owner-1"),
        ("registration-other", "module-other", "owner-2"),
        ("registration-catalog", "module-catalog", "owner-1"),
    ] {
        let capability_ids = match registration_id {
            "registration-visible" => vec![
                "client.surface.personas.v1".to_owned(),
                "sections.read".to_owned(),
            ],
            "registration-catalog" => vec![
                "mail.delivery.query.v1".to_owned(),
                makosh_runtime_protocol::SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID.to_owned(),
            ],
            _ => vec!["sections.read".to_owned()],
        };
        fixture
            .store
            .create_pending_registration(
                &ModuleRegistration::new(
                    registration_id,
                    module_id,
                    owner_id,
                    [9; 32],
                    ModuleRegistrationState::Pending,
                    1,
                ),
                &capability_ids,
            )
            .expect("create module registration");
        fixture
            .store
            .approve_module_registration(registration_id, &capability_ids)
            .expect("approve module registration");
    }
}

fn admit_current_visible_settings(fixture: &AuthenticationHttpFixture) {
    let schema = visible_schema();
    let bytes = schema.encode_to_vec();
    fixture
        .store
        .admit_settings_schema(
            &SettingsSchemaBinding::new(
                makosh_kernel_control_store::SettingsSchemaBindingInputV1 {
                    registration_id: "registration-visible".to_owned(),
                    schema_major: 1,
                    schema_revision: 1,
                    schema_sha256: Sha256::digest(&bytes).into(),
                    desired_revision: 0,
                    effective_revision: 0,
                    apply_state: SettingsApplyState::Current,
                    sanitized_reason_code: None,
                },
            ),
            &bytes,
        )
        .expect("admit settings schema");
    fixture
        .store
        .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
            registration_id: "registration-visible".to_owned(),
            configuration_instance_id: "registration-visible".to_owned(),
            expected_revision: 0,
            snapshot_bytes: current_snapshot("registration-visible", 1).encode_to_vec(),
        })
        .expect("commit desired settings");
    fixture
        .store
        .transition_settings_apply_state(
            "registration-visible",
            1,
            SettingsApplyState::PendingApply,
            None,
        )
        .expect("settings pending apply");
    fixture
        .store
        .transition_settings_apply_state(
            "registration-visible",
            1,
            SettingsApplyState::Applying,
            None,
        )
        .expect("settings applying");
    fixture
        .store
        .confirm_effective_settings_revision("registration-visible", 1)
        .expect("settings current");
    admit_current_catalog_target(fixture);
}

fn admit_current_catalog_target(fixture: &AuthenticationHttpFixture) {
    let bytes = visible_schema().encode_to_vec();
    fixture
        .store
        .admit_settings_schema(
            &SettingsSchemaBinding::new(
                makosh_kernel_control_store::SettingsSchemaBindingInputV1 {
                    registration_id: "registration-catalog".to_owned(),
                    schema_major: 1,
                    schema_revision: 1,
                    schema_sha256: Sha256::digest(&bytes).into(),
                    desired_revision: 0,
                    effective_revision: 0,
                    apply_state: SettingsApplyState::Current,
                    sanitized_reason_code: None,
                },
            ),
            &bytes,
        )
        .expect("admit catalog settings schema");
    fixture
        .store
        .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
            registration_id: "registration-catalog".to_owned(),
            configuration_instance_id: "registration-catalog".to_owned(),
            created_operation_id: None,
            snapshot_bytes: vec![0],
            complete: false,
        })
        .expect("block compatibility target");
    fixture
        .store
        .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
            registration_id: "registration-catalog".to_owned(),
            configuration_instance_id: "configuration-current".to_owned(),
            created_operation_id: Some([7; 16]),
            snapshot_bytes: current_snapshot("configuration-current", 1).encode_to_vec(),
            complete: true,
        })
        .expect("materialize catalog target");
    for state in [
        SettingsApplyState::PendingApply,
        SettingsApplyState::Applying,
    ] {
        fixture
            .store
            .transition_settings_apply_state_for_target(
                "registration-catalog",
                "configuration-current",
                1,
                state,
                None,
            )
            .expect("advance catalog target");
    }
    fixture
        .store
        .confirm_effective_settings_revision_for_target(
            "registration-catalog",
            "configuration-current",
            1,
        )
        .expect("catalog target current");
}

fn visible_schema() -> SettingsSchemaV1 {
    let definition = |setting_id: &str, visibility| SettingDefinitionV1 {
        setting_id: setting_id.to_owned(),
        capability_id: String::new(),
        value_type: SettingValueTypeV1::Boolean as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
        apply_mode: SettingApplyModeV1::HotReload as i32,
        client_visibility: visibility as i32,
        fresh_owner_proof_required: false,
        kernel_controller_id: String::new(),
        display_name: setting_id.to_owned(),
        default_value: None,
        optional: false,
    };
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![
            definition("hidden.flag", SettingClientVisibilityV1::Hidden),
            definition("visible.flag", SettingClientVisibilityV1::ReadOnly),
        ],
    }
}

fn current_snapshot(registration_id: &str, revision: u64) -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: registration_id.to_owned(),
        revision,
        values: vec![
            SettingsValueEntryV1 {
                setting_id: "hidden.flag".to_owned(),
                value: Some(SettingValueV1 {
                    value: Some(Value::BooleanValue(false)),
                }),
            },
            SettingsValueEntryV1 {
                setting_id: "visible.flag".to_owned(),
                value: Some(SettingValueV1 {
                    value: Some(Value::BooleanValue(true)),
                }),
            },
        ],
    }
}

fn read_bootstrap(
    fixture: &AuthenticationHttpFixture,
    cookie: Option<&str>,
) -> ClientBootstrapResponseV1 {
    let response = fixture
        .runtime
        .block_on(fixture.router.route(bootstrap_request(cookie)));
    assert_eq!(response.status(), StatusCode::OK);
    let body = fixture
        .runtime
        .block_on(response.into_body().collect())
        .expect("bootstrap response body")
        .to_bytes();
    ClientBootstrapResponseV1::decode(body).expect("typed bootstrap response")
}

fn read_bootstrap_status(fixture: &AuthenticationHttpFixture, cookie: Option<&str>) -> StatusCode {
    fixture
        .runtime
        .block_on(fixture.router.route(bootstrap_request(cookie)))
        .status()
}

fn bootstrap_request(cookie: Option<&str>) -> Request<Full<Bytes>> {
    let mut request = Request::builder()
        .method("POST")
        .uri(BOOTSTRAP_PATH)
        .header("content-type", "application/proto")
        .header("connect-protocol-version", "1");
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    request
        .body(Full::new(Bytes::new()))
        .expect("Connect bootstrap request")
}
