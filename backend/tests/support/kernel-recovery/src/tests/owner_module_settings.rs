//! Public owner Settings proof, mutation authority and Gateway admission.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, StatusCode};
use makosh_gateway_protocol::v1::{
    CommitOwnerModuleSettingsRequestV1, CommitOwnerModuleSettingsResponseV1,
    CreateOwnerModuleSettingsTargetV1, ExportEffectiveOwnerModuleSettingsV1, OwnerSettingEntryV1,
    OwnerSettingValueV1, PrepareOwnerModuleSettingsRequestV1, PrepareOwnerModuleSettingsResponseV1,
    UpdateOwnerModuleSettingsV1, commit_owner_module_settings_response_v1, owner_setting_value_v1,
    prepare_owner_module_settings_request_v1,
};
use makosh_gateway_runtime::{
    OWNER_MODULE_SETTINGS_COMMIT_PATH, OWNER_MODULE_SETTINGS_PREPARE_PATH, OwnerBrowserPrincipalV1,
    OwnerModuleSettingsHandlerV1, OwnerModuleSettingsRouteErrorV1,
};
use makosh_runtime_protocol::SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID;

use super::common::*;
use crate::modules::settings::owner_gateway::KernelOwnerModuleSettingsHandlerV1;

const HUMAN_OWNER: &str = "owner-1";
const DEVICE: &str = "browser-1";
const INITIAL_DEVICE: &str = "owner-device";
const REGISTRATION: &str = "mail-registration";
const CAPABILITY: &str = "mail.settings";
const SESSION: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn owner_settings_requires_fresh_proof_and_preserves_schema_cas() {
    let fixture = OwnerSettingsFixture::new("makosh-owner-settings-direct");
    let handler = fixture.handler();
    let current_principal = principal(SESSION);
    let signing_key = super::browser_gateway_session::browser_signing_key();

    let committed = commit_direct(
        &handler,
        &current_principal,
        &signing_key,
        update_request([1; 16], 0, unsigned_value(1)),
    )
    .expect("commit owner Settings update");
    assert_eq!(committed.operation_id, [1; 16]);
    assert_eq!(
        fixture
            .store
            .settings_schema_binding(REGISTRATION)
            .expect("read settings binding")
            .expect("settings binding")
            .apply_state(),
        SettingsApplyState::PendingValidation
    );

    let invalid_signature = handler
        .prepare(
            &current_principal,
            update_request([2; 16], 1, unsigned_value(2)),
        )
        .expect("prepare invalid-signature challenge");
    let other_key = SigningKey::from_bytes((&[8_u8; 32]).into()).expect("other signing key");
    let wrong_signature: Signature = other_key.sign(&invalid_signature.challenge_bytes);
    assert_eq!(
        handler
            .commit(
                &current_principal,
                CommitOwnerModuleSettingsRequestV1 {
                    challenge_id: invalid_signature.challenge_id.clone(),
                    device_signature_raw: wrong_signature.to_bytes().to_vec(),
                },
            )
            .expect_err("invalid device proof must fail"),
        OwnerModuleSettingsRouteErrorV1::PermissionDenied
    );
    assert_eq!(
        handler
            .commit(
                &current_principal,
                CommitOwnerModuleSettingsRequestV1 {
                    challenge_id: invalid_signature.challenge_id,
                    device_signature_raw: wrong_signature.to_bytes().to_vec(),
                },
            )
            .expect_err("challenge must be single-use"),
        OwnerModuleSettingsRouteErrorV1::NotFound
    );

    let other_session =
        principal("1111111111111111111111111111111111111111111111111111111111111111");
    let prepared = handler
        .prepare(
            &current_principal,
            update_request([3; 16], 1, unsigned_value(2)),
        )
        .expect("prepare same-principal challenge");
    let signature: Signature = signing_key.sign(&prepared.challenge_bytes);
    assert_eq!(
        handler
            .commit(
                &other_session,
                CommitOwnerModuleSettingsRequestV1 {
                    challenge_id: prepared.challenge_id,
                    device_signature_raw: signature.to_bytes().to_vec(),
                },
            )
            .expect_err("different session must fail"),
        OwnerModuleSettingsRouteErrorV1::PermissionDenied
    );

    let wrong_type = commit_direct(
        &handler,
        &current_principal,
        &signing_key,
        update_request([4; 16], 1, string_value("not-a-number")),
    );
    assert_eq!(
        wrong_type.expect_err("schema mismatch must fail"),
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    );

    let stale = handler
        .prepare(
            &current_principal,
            update_request([5; 16], 1, unsigned_value(3)),
        )
        .expect("prepare stale CAS challenge");
    crate::modules::settings::mutation::commit_after_owner_authorization(
        &*fixture.store,
        REGISTRATION,
        1,
        &canonical_snapshot(2, 2).encode_to_vec(),
    )
    .expect("advance desired revision");
    let signature: Signature = signing_key.sign(&stale.challenge_bytes);
    assert_eq!(
        handler
            .commit(
                &current_principal,
                CommitOwnerModuleSettingsRequestV1 {
                    challenge_id: stale.challenge_id,
                    device_signature_raw: signature.to_bytes().to_vec(),
                },
            )
            .expect_err("stale desired revision must conflict"),
        OwnerModuleSettingsRouteErrorV1::Conflict
    );
}

#[test]
fn owner_settings_creates_idempotent_isolated_configuration_targets() {
    let fixture = OwnerSettingsFixture::new("makosh-owner-settings-targets");
    let handler = fixture.handler();
    let principal = principal(SESSION);
    let signing_key = super::browser_gateway_session::browser_signing_key();

    let first = commit_direct(
        &handler,
        &principal,
        &signing_key,
        create_target_request([9; 16]),
    )
    .expect("create first configuration target");
    let first_id = created_configuration_instance_id(&first);
    let repeated = commit_direct(
        &handler,
        &principal,
        &signing_key,
        create_target_request([9; 16]),
    )
    .expect("repeat idempotent target creation");
    assert_eq!(created_configuration_instance_id(&repeated), first_id);

    let second = commit_direct(
        &handler,
        &principal,
        &signing_key,
        create_target_request([10; 16]),
    )
    .expect("create second configuration target");
    let second_id = created_configuration_instance_id(&second);
    assert_ne!(first_id, second_id);
    assert_eq!(
        fixture
            .store
            .settings_configuration_targets(REGISTRATION)
            .expect("list configuration targets")
            .len(),
        3
    );

    commit_direct(
        &handler,
        &principal,
        &signing_key,
        update_target_request([11; 16], &first_id, 1, unsigned_value(7)),
    )
    .expect("update first configuration target");
    assert_eq!(
        fixture
            .store
            .settings_configuration_target(REGISTRATION, &first_id)
            .expect("read first target")
            .expect("first target")
            .desired_revision(),
        2
    );
    let repeated_after_update = commit_direct(
        &handler,
        &principal,
        &signing_key,
        create_target_request([9; 16]),
    )
    .expect("repeat target creation after settings update");
    assert_eq!(
        created_configuration_instance_id(&repeated_after_update),
        first_id
    );
    assert_eq!(
        match repeated_after_update.result.as_ref() {
            Some(commit_owner_module_settings_response_v1::Result::Created(created)) =>
                created.desired_revision,
            _ => panic!("configuration target creation receipt"),
        },
        2
    );
    assert_eq!(
        fixture
            .store
            .settings_configuration_target(REGISTRATION, &second_id)
            .expect("read second target")
            .expect("second target")
            .desired_revision(),
        1
    );
}

#[test]
fn owner_settings_denies_configuration_target_without_exact_catalog_grant() {
    let fixture = OwnerSettingsFixture::new_without_catalog(
        "makosh-owner-settings-target-without-catalog-grant",
    );
    let error = fixture
        .handler()
        .prepare(&principal(SESSION), create_target_request([12; 16]))
        .expect_err("catalog target creation must require its exact grant");
    assert_eq!(error, OwnerModuleSettingsRouteErrorV1::PermissionDenied);
}

#[test]
fn owner_settings_gateway_requires_authenticated_same_origin_and_denies_lan_mode() {
    let fixture = OwnerSettingsFixture::new("makosh-owner-settings-gateway");
    let signing_key = super::browser_gateway_session::browser_signing_key();
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        fixture.root.join("gateway-cert.der"),
        fixture.root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(&fixture.store),
        &fixture.data,
        fixture.supervisor.clone(),
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(1_024)
            .expect("test realtime source"),
        &configuration,
        None,
    )
    .expect("compose owner Settings Gateway");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let cookie = super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);

    let wrong_origin = runtime.block_on(router.route(settings_http_request(
        OWNER_MODULE_SETTINGS_PREPARE_PATH,
        update_request([6; 16], 0, unsigned_value(1)).encode_to_vec(),
        Some(&cookie),
        "https://evil.invalid",
    )));
    assert_eq!(wrong_origin.status(), StatusCode::FORBIDDEN);

    let prepared = runtime.block_on(router.route(settings_http_request(
        OWNER_MODULE_SETTINGS_PREPARE_PATH,
        update_request([7; 16], 0, unsigned_value(1)).encode_to_vec(),
        Some(&cookie),
        "https://hub.local",
    )));
    assert_eq!(prepared.status(), StatusCode::OK);
    let prepared = decode_body::<PrepareOwnerModuleSettingsResponseV1, _>(&runtime, prepared);
    let signature: Signature = signing_key.sign(&prepared.challenge_bytes);
    let committed = runtime.block_on(
        router.route(settings_http_request(
            OWNER_MODULE_SETTINGS_COMMIT_PATH,
            CommitOwnerModuleSettingsRequestV1 {
                challenge_id: prepared.challenge_id,
                device_signature_raw: signature.to_bytes().to_vec(),
            }
            .encode_to_vec(),
            Some(&cookie),
            "https://hub.local",
        )),
    );
    assert_eq!(committed.status(), StatusCode::OK);
    let committed = decode_body::<CommitOwnerModuleSettingsResponseV1, _>(&runtime, committed);
    assert_eq!(committed.operation_id, [7; 16]);

    let proof = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let loopback_configuration =
        crate::platform::gateway::BrowserGatewayConfigurationV1::new_loopback_development_proxy(
            "127.0.0.1:9444".parse().expect("loopback Gateway address"),
            "http://127.0.0.1:5173".to_owned(),
            "127.0.0.1".to_owned(),
            proof.to_owned(),
        )
        .expect("loopback development Gateway configuration");
    let loopback_router = crate::platform::gateway::gateway_service(
        Arc::clone(&fixture.store),
        &fixture.data,
        fixture.supervisor.clone(),
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(1_024)
            .expect("test realtime source"),
        &loopback_configuration,
        None,
    )
    .expect("compose loopback development Gateway");
    let mut loopback_prepare = settings_http_request(
        OWNER_MODULE_SETTINGS_PREPARE_PATH,
        update_request([8; 16], 1, unsigned_value(2)).encode_to_vec(),
        None,
        "http://127.0.0.1:5173",
    );
    loopback_prepare
        .headers_mut()
        .insert("host", "127.0.0.1:5173".parse().expect("host header"));
    loopback_prepare.headers_mut().insert(
        "x-makosh-development-proxy-proof",
        proof.parse().expect("proxy proof header"),
    );
    let prepared = runtime.block_on(loopback_router.route(loopback_prepare));
    assert_eq!(prepared.status(), StatusCode::OK);
    let prepared = decode_body::<PrepareOwnerModuleSettingsResponseV1, _>(&runtime, prepared);
    let signature: Signature = owner_signing_key().sign(&prepared.challenge_bytes);
    let mut loopback_commit = settings_http_request(
        OWNER_MODULE_SETTINGS_COMMIT_PATH,
        CommitOwnerModuleSettingsRequestV1 {
            challenge_id: prepared.challenge_id,
            device_signature_raw: signature.to_bytes().to_vec(),
        }
        .encode_to_vec(),
        None,
        "http://127.0.0.1:5173",
    );
    loopback_commit
        .headers_mut()
        .insert("host", "127.0.0.1:5173".parse().expect("host header"));
    loopback_commit.headers_mut().insert(
        "x-makosh-development-proxy-proof",
        proof.parse().expect("proxy proof header"),
    );
    let committed = runtime.block_on(loopback_router.route(loopback_commit));
    assert_eq!(committed.status(), StatusCode::OK);
    let committed = decode_body::<CommitOwnerModuleSettingsResponseV1, _>(&runtime, committed);
    assert_eq!(committed.operation_id, [8; 16]);

    assert_eq!(
        fixture
            .handler()
            .prepare(
                &principal("loopback-development"),
                update_request([9; 16], 2, unsigned_value(3)),
            )
            .expect_err("mismatched loopback device must fail"),
        OwnerModuleSettingsRouteErrorV1::PermissionDenied,
    );

    let lan_configuration =
        crate::platform::gateway::BrowserGatewayConfigurationV1::new_lan_development(
            "192.168.1.10:9443"
                .parse()
                .expect("private LAN Gateway address"),
            "http://192.168.1.10:9443".to_owned(),
            "192.168.1.10".to_owned(),
        )
        .expect("LAN Gateway configuration");
    let lan_router = crate::platform::gateway::gateway_service(
        Arc::clone(&fixture.store),
        &fixture.data,
        fixture.supervisor.clone(),
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(1_024)
            .expect("test realtime source"),
        &lan_configuration,
        None,
    )
    .expect("compose LAN development Gateway");
    let denied = runtime.block_on(lan_router.route(settings_http_request(
        OWNER_MODULE_SETTINGS_PREPARE_PATH,
        update_request([10; 16], 2, unsigned_value(3)).encode_to_vec(),
        None,
        "http://192.168.1.10:9443",
    )));
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
}

#[test]
fn owner_settings_export_returns_only_current_client_visible_values() {
    let fixture = OwnerSettingsFixture::new("makosh-owner-settings-export");
    make_export_snapshot_current(&fixture.store);
    let handler = fixture.handler();
    let current_principal = principal(SESSION);
    let signing_key = super::browser_gateway_session::browser_signing_key();

    let exported = commit_direct(
        &handler,
        &current_principal,
        &signing_key,
        export_request([9; 16], 1),
    )
    .expect("export current Settings");
    let Some(commit_owner_module_settings_response_v1::Result::Exported(exported)) =
        exported.result
    else {
        panic!("expected exported Settings receipt");
    };
    assert_eq!(exported.registration_id, REGISTRATION);
    assert_eq!(exported.schema_major, 1);
    assert_eq!(exported.schema_revision, 1);
    assert_eq!(exported.effective_revision, 1);
    assert_eq!(exported.values.len(), 1);
    assert_eq!(exported.values[0].setting_id, "mail.sync.window");

    assert_eq!(
        commit_direct(
            &handler,
            &current_principal,
            &signing_key,
            export_request([10; 16], 2),
        )
        .expect_err("stale export revision must fail"),
        OwnerModuleSettingsRouteErrorV1::Conflict
    );

    fixture
        .store
        .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
            registration_id: REGISTRATION.to_owned(),
            configuration_instance_id: REGISTRATION.to_owned(),
            expected_revision: 1,
            snapshot_bytes: canonical_snapshot(2, 2).encode_to_vec(),
        })
        .expect("move Settings out of current state");
    assert_eq!(
        commit_direct(
            &handler,
            &current_principal,
            &signing_key,
            export_request([11; 16], 1),
        )
        .expect_err("non-current Settings must not export"),
        OwnerModuleSettingsRouteErrorV1::Conflict
    );
}

struct OwnerSettingsFixture {
    root: std::path::PathBuf,
    data: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
}

impl OwnerSettingsFixture {
    fn new(prefix: &str) -> Self {
        Self::new_with_catalog_grant(prefix, true)
    }

    fn new_without_catalog(prefix: &str) -> Self {
        Self::new_with_catalog_grant(prefix, false)
    }

    fn new_with_catalog_grant(prefix: &str, catalog_granted: bool) -> Self {
        let root = unique_target_root(prefix);
        let data = root.join("kernel");
        let runtime = root.join("runtime");
        std::fs::create_dir_all(&data).expect("create data directory");
        std::fs::create_dir_all(&runtime).expect("create runtime directory");
        let store = Arc::new(
            SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
                .expect("create Control Store"),
        );
        admit_owner_browser_registration_and_schema(&store, catalog_granted);
        Self {
            root,
            data,
            store,
            supervisor: ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false))),
        }
    }

    fn handler(&self) -> KernelOwnerModuleSettingsHandlerV1 {
        KernelOwnerModuleSettingsHandlerV1::new(
            Arc::clone(&self.store),
            &self.data,
            &self.root.join("runtime"),
            self.supervisor.clone(),
        )
    }
}

impl Drop for OwnerSettingsFixture {
    fn drop(&mut self) {
        self.supervisor.shutdown().expect("stop managed runtimes");
        std::fs::remove_dir_all(&self.root).expect("remove owner Settings fixture");
    }
}

fn admit_owner_browser_registration_and_schema(
    store: &Arc<SqliteControlStore>,
    catalog_granted: bool,
) {
    let owner_key = owner_signing_key();
    let owner_public_key: [u8; 65] = owner_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("owner public key");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            HUMAN_OWNER,
            INITIAL_DEVICE,
            owner_public_key,
        ))
        .expect("claim owner");
    super::browser_gateway_session::admit_browser_test_device(store, HUMAN_OWNER);
    let mut capability_ids = vec![CAPABILITY.to_owned()];
    if catalog_granted {
        capability_ids.push(SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID.to_owned());
    }
    store
        .create_pending_registration(
            &ModuleRegistration::new(
                REGISTRATION,
                "integration.mail",
                "mail",
                [3; 32],
                ModuleRegistrationState::Pending,
                1,
            ),
            &capability_ids,
        )
        .expect("record Mail registration");
    store
        .approve_module_registration(REGISTRATION, &capability_ids)
        .expect("approve Mail registration");
    let schema = settings_schema();
    let bytes = schema.encode_to_vec();
    store
        .admit_settings_schema(
            &SettingsSchemaBinding::new(
                makosh_kernel_control_store::SettingsSchemaBindingInputV1 {
                    registration_id: REGISTRATION.to_owned(),
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
        .expect("admit Mail settings schema");
}

fn settings_schema() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![
            SettingDefinitionV1 {
                setting_id: "mail.internal.hidden".to_owned(),
                capability_id: String::new(),
                value_type: SettingValueTypeV1::String as i32,
                mutation_authority: SettingMutationAuthorityV1::KernelManaged as i32,
                target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
                apply_mode: SettingApplyModeV1::RestartModule as i32,
                client_visibility: SettingClientVisibilityV1::Hidden as i32,
                fresh_owner_proof_required: false,
                kernel_controller_id: "mail.controller".to_owned(),
                display_name: "Internal".to_owned(),
                default_value: None,
                optional: false,
            },
            SettingDefinitionV1 {
                setting_id: "mail.sync.window".to_owned(),
                capability_id: String::new(),
                value_type: SettingValueTypeV1::UnsignedInteger as i32,
                mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
                target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
                apply_mode: SettingApplyModeV1::RestartModule as i32,
                client_visibility: SettingClientVisibilityV1::Editable as i32,
                fresh_owner_proof_required: true,
                kernel_controller_id: String::new(),
                display_name: "Sync window".to_owned(),
                default_value: None,
                optional: false,
            },
        ],
    }
}

fn canonical_snapshot(revision: u64, value: u64) -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: REGISTRATION.to_owned(),
        revision,
        values: vec![SettingsValueEntryV1 {
            setting_id: "mail.sync.window".to_owned(),
            value: Some(SettingValueV1 {
                value: Some(
                    makosh_runtime_protocol::v1::setting_value_v1::Value::UnsignedIntegerValue(
                        value,
                    ),
                ),
            }),
        }],
    }
}

fn export_snapshot(revision: u64, value: u64) -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: REGISTRATION.to_owned(),
        revision,
        values: vec![
            SettingsValueEntryV1 {
                setting_id: "mail.internal.hidden".to_owned(),
                value: Some(SettingValueV1 {
                    value: Some(
                        makosh_runtime_protocol::v1::setting_value_v1::Value::StringValue(
                            "private".to_owned(),
                        ),
                    ),
                }),
            },
            canonical_snapshot(revision, value)
                .values
                .into_iter()
                .next()
                .expect("visible Settings value"),
        ],
    }
}

fn make_export_snapshot_current(store: &SqliteControlStore) {
    store
        .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
            registration_id: REGISTRATION.to_owned(),
            configuration_instance_id: REGISTRATION.to_owned(),
            expected_revision: 0,
            snapshot_bytes: export_snapshot(1, 1).encode_to_vec(),
        })
        .expect("commit export Settings");
    store
        .transition_settings_apply_state(REGISTRATION, 1, SettingsApplyState::PendingApply, None)
        .expect("accept export Settings validation");
    store
        .transition_settings_apply_state(REGISTRATION, 1, SettingsApplyState::Applying, None)
        .expect("start export Settings apply");
    store
        .confirm_effective_settings_revision(REGISTRATION, 1)
        .expect("confirm export Settings");
}

fn principal(session: &str) -> OwnerBrowserPrincipalV1 {
    OwnerBrowserPrincipalV1::new(HUMAN_OWNER, DEVICE, session).expect("browser principal")
}

fn owner_signing_key() -> SigningKey {
    SigningKey::from_bytes((&[11_u8; 32]).into()).expect("owner signing key")
}

fn update_request(
    operation_id: [u8; 16],
    expected_desired_revision: u64,
    value: OwnerSettingValueV1,
) -> PrepareOwnerModuleSettingsRequestV1 {
    update_target_request(operation_id, REGISTRATION, expected_desired_revision, value)
}

fn update_target_request(
    operation_id: [u8; 16],
    configuration_instance_id: &str,
    expected_desired_revision: u64,
    value: OwnerSettingValueV1,
) -> PrepareOwnerModuleSettingsRequestV1 {
    PrepareOwnerModuleSettingsRequestV1 {
        operation_id: operation_id.to_vec(),
        operation: Some(
            prepare_owner_module_settings_request_v1::Operation::UpdateDesired(
                UpdateOwnerModuleSettingsV1 {
                    registration_id: REGISTRATION.to_owned(),
                    configuration_instance_id: configuration_instance_id.to_owned(),
                    expected_desired_revision,
                    values: vec![OwnerSettingEntryV1 {
                        setting_id: "mail.sync.window".to_owned(),
                        value: Some(value),
                    }],
                },
            ),
        ),
    }
}

fn create_target_request(operation_id: [u8; 16]) -> PrepareOwnerModuleSettingsRequestV1 {
    PrepareOwnerModuleSettingsRequestV1 {
        operation_id: operation_id.to_vec(),
        operation: Some(
            prepare_owner_module_settings_request_v1::Operation::CreateConfigurationTarget(
                CreateOwnerModuleSettingsTargetV1 {
                    registration_id: REGISTRATION.to_owned(),
                },
            ),
        ),
    }
}

fn created_configuration_instance_id(response: &CommitOwnerModuleSettingsResponseV1) -> String {
    match response.result.as_ref() {
        Some(commit_owner_module_settings_response_v1::Result::Created(created)) => {
            created.configuration_instance_id.clone()
        }
        _ => panic!("configuration target creation receipt"),
    }
}

fn export_request(
    operation_id: [u8; 16],
    expected_effective_revision: u64,
) -> PrepareOwnerModuleSettingsRequestV1 {
    PrepareOwnerModuleSettingsRequestV1 {
        operation_id: operation_id.to_vec(),
        operation: Some(
            prepare_owner_module_settings_request_v1::Operation::ExportEffective(
                ExportEffectiveOwnerModuleSettingsV1 {
                    registration_id: REGISTRATION.to_owned(),
                    configuration_instance_id: REGISTRATION.to_owned(),
                    expected_effective_revision,
                },
            ),
        ),
    }
}

fn unsigned_value(value: u64) -> OwnerSettingValueV1 {
    OwnerSettingValueV1 {
        value: Some(owner_setting_value_v1::Value::UnsignedIntegerValue(value)),
    }
}

fn string_value(value: &str) -> OwnerSettingValueV1 {
    OwnerSettingValueV1 {
        value: Some(owner_setting_value_v1::Value::StringValue(value.to_owned())),
    }
}

fn commit_direct(
    handler: &KernelOwnerModuleSettingsHandlerV1,
    principal: &OwnerBrowserPrincipalV1,
    signing_key: &SigningKey,
    request: PrepareOwnerModuleSettingsRequestV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let prepared = handler.prepare(principal, request)?;
    let signature: Signature = signing_key.sign(&prepared.challenge_bytes);
    handler.commit(
        principal,
        CommitOwnerModuleSettingsRequestV1 {
            challenge_id: prepared.challenge_id,
            device_signature_raw: signature.to_bytes().to_vec(),
        },
    )
}

fn settings_http_request(
    path: &str,
    payload: Vec<u8>,
    cookie: Option<&str>,
    origin: &str,
) -> Request<Full<Bytes>> {
    let mut request = Request::post(path)
        .header("content-type", "application/proto")
        .header("connect-protocol-version", "1")
        .header("origin", origin);
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    request
        .body(Full::new(Bytes::from(payload)))
        .expect("owner Settings request")
}

fn decode_body<T, B>(runtime: &tokio::runtime::Runtime, response: hyper::Response<B>) -> T
where
    T: Message + Default,
    B: hyper::body::Body<Data = Bytes>,
    B::Error: std::fmt::Debug,
{
    let body = runtime
        .block_on(response.into_body().collect())
        .expect("collect owner Settings response")
        .to_bytes();
    T::decode(body).expect("decode owner Settings response")
}
