use super::common::*;
use std::sync::atomic::{AtomicU64, Ordering};

static REGISTRATION_FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn module_registration_has_explicit_state_transitions_and_grant_epoch_fencing() {
    let fixture = RegistrationFixture::new();
    fixture.assert_initial_approval();
    fixture.assert_runtime_bindings();
    fixture.assert_settings_applied();
    fixture.assert_blocked_settings();
    fixture.assert_registration_fences();
}

#[test]
fn initial_settings_materialization_is_atomic_exact_and_idempotent() {
    let fixture = RegistrationFixture::new();
    let schema =
        SettingsSchemaBinding::new(makosh_kernel_control_store::SettingsSchemaBindingInputV1 {
            registration_id: "registration-1".to_owned(),
            schema_major: 1,
            schema_revision: 1,
            schema_sha256: [8; 32],
            desired_revision: 0,
            effective_revision: 0,
            apply_state: SettingsApplyState::Current,
            sanitized_reason_code: None,
        });
    fixture
        .store
        .register_settings_schema(&schema)
        .expect("bind settings schema");
    let initial = SettingsInitialSnapshot {
        registration_id: "registration-1".to_owned(),
        configuration_instance_id: "registration-1".to_owned(),
        created_operation_id: None,
        snapshot_bytes: vec![1, 2, 3],
        complete: true,
    };

    assert_eq!(
        fixture
            .store
            .materialize_initial_settings_snapshot(&initial)
            .expect("materialize initial settings"),
        1,
    );
    assert_eq!(
        fixture
            .store
            .materialize_initial_settings_snapshot(&initial)
            .expect("replay initial settings"),
        1,
    );
    assert_eq!(
        fixture
            .store
            .desired_settings_snapshot("registration-1")
            .expect("read initial snapshot"),
        Some((1, vec![1, 2, 3])),
    );
    let new_target = SettingsInitialSnapshot {
        registration_id: "registration-1".to_owned(),
        configuration_instance_id: "cfg-second".to_owned(),
        created_operation_id: Some([7; 16]),
        snapshot_bytes: vec![4, 5, 6],
        complete: true,
    };
    assert_eq!(
        fixture
            .store
            .materialize_initial_settings_snapshot(&new_target)
            .expect("materialize complete new target"),
        1,
    );
    let new_target = fixture
        .store
        .settings_configuration_target("registration-1", "cfg-second")
        .expect("read complete new target")
        .expect("complete new target");
    assert_eq!(new_target.desired_revision(), 1);
    assert_eq!(new_target.effective_revision(), 0);
    assert_eq!(
        new_target.apply_state(),
        SettingsApplyState::PendingValidation
    );
    let current = fixture.settings_binding();
    assert_eq!(current.desired_revision(), 1);
    assert_eq!(current.effective_revision(), 1);
    assert_eq!(current.apply_state(), SettingsApplyState::Current);
    assert!(
        fixture
            .store
            .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
                snapshot_bytes: vec![9],
                ..initial
            })
            .is_err()
    );
}

#[test]
fn incomplete_initial_settings_are_atomically_blocked_without_effective_revision() {
    let path = std::env::temp_dir().join(format!(
        "makosh-control-store-incomplete-settings-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let store = SqliteControlStore::create(&path, "instance-incomplete", 1)
        .expect("create incomplete settings store");
    let registration = ModuleRegistration::new(
        "registration-incomplete",
        "module-incomplete",
        "owner-incomplete",
        [9; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &[])
        .expect("create incomplete registration");
    store
        .transition_module_registration(
            "registration-incomplete",
            ModuleRegistrationState::Approved,
        )
        .expect("approve incomplete registration");
    store
        .register_settings_schema(&SettingsSchemaBinding::new(
            makosh_kernel_control_store::SettingsSchemaBindingInputV1 {
                registration_id: "registration-incomplete".to_owned(),
                schema_major: 1,
                schema_revision: 1,
                schema_sha256: [6; 32],
                desired_revision: 0,
                effective_revision: 0,
                apply_state: SettingsApplyState::Current,
                sanitized_reason_code: None,
            },
        ))
        .expect("bind incomplete schema");
    let initial = SettingsInitialSnapshot {
        registration_id: "registration-incomplete".to_owned(),
        configuration_instance_id: "registration-incomplete".to_owned(),
        created_operation_id: None,
        snapshot_bytes: vec![4, 5, 6],
        complete: false,
    };

    assert_eq!(
        store
            .materialize_initial_settings_snapshot(&initial)
            .expect("materialize incomplete settings"),
        1
    );
    assert_eq!(
        store
            .materialize_initial_settings_snapshot(&initial)
            .expect("replay incomplete settings"),
        1
    );
    let binding = store
        .settings_schema_binding("registration-incomplete")
        .expect("read incomplete binding")
        .expect("incomplete binding");
    assert_eq!(binding.desired_revision(), 1);
    assert_eq!(binding.effective_revision(), 0);
    assert_eq!(binding.apply_state(), SettingsApplyState::BlockedConfig);
    assert_eq!(
        binding.sanitized_reason_code(),
        Some("required_settings_missing")
    );
    let _ = std::fs::remove_file(path);
}

struct RegistrationFixture {
    path: std::path::PathBuf,
    store: SqliteControlStore,
}

impl RegistrationFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "makosh-control-store-registration-{}-{}.sqlite",
            std::process::id(),
            REGISTRATION_FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);
        let store = SqliteControlStore::create(&path, "instance-1", 1).expect("create store");
        let registration = ModuleRegistration::new(
            "registration-1",
            "module-1",
            "owner-1",
            [7; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        store
            .create_pending_registration(
                &registration,
                &["capability.read".to_owned(), "capability.write".to_owned()],
            )
            .expect("create pending registration");
        assert_eq!(
            store
                .module_registration("registration-1")
                .expect("read registration"),
            Some(registration)
        );
        let grants = store
            .approve_module_registration("registration-1", &["capability.read".to_owned()])
            .expect("approve capability subset");
        assert_eq!(grants.grant_epoch(), 2);
        assert_eq!(grants.capability_ids(), ["capability.read"]);
        Self { path, store }
    }

    fn assert_runtime_bindings(&self) {
        self.assert_platform_process_fencing();
        let binding = BundledManagedLaunchBinding::new(
            "registration-1",
            1,
            "makosh-desktop",
            "runtime.mail",
            [10; 32],
            [7; 32],
            Some([11; 32]),
        );
        self.store
            .record_bundled_managed_launch_binding(&binding)
            .expect("record bundled launch binding");
        assert_eq!(
            self.store
                .effective_bundled_managed_launch_binding("registration-1")
                .expect("read bundled launch binding"),
            Some(binding.clone())
        );
        let launch = ManagedLaunchRecord::new("registration-1", "runtime-1", 1, 1, 1, 2);
        self.store
            .record_managed_launch(&launch)
            .expect("record managed launch");
        assert_eq!(
            self.store
                .effective_managed_launch_record("registration-1")
                .expect("read managed launch record"),
            Some(launch.clone())
        );
        assert!(self.store.record_managed_launch(&launch).is_err());
        let attestation =
            ExternalRuntimeAttestation::new("registration-1", "compose-runtime-1", 1, 2, [9; 32]);
        self.store
            .attest_external_runtime(&attestation)
            .expect("attest external runtime");
        assert_eq!(
            self.store
                .effective_external_runtime_attestation("registration-1")
                .expect("effective external runtime"),
            Some(attestation.clone())
        );
        assert!(self.store.attest_external_runtime(&attestation).is_err());
    }

    fn assert_platform_process_fencing(&self) {
        let binding = PlatformManagedProcessBinding::new(
            "vault",
            1,
            "makosh-desktop",
            "runtime.vault",
            [12; 32],
            [13; 32],
            None,
        );
        self.store
            .record_platform_managed_process_binding(&binding)
            .expect("record platform process binding");
        assert_eq!(
            self.store
                .platform_managed_process_binding("vault")
                .expect("read platform process binding"),
            Some(binding.clone())
        );
        let launch = PlatformManagedProcessLaunch::new("vault", 1, 1, 1, 2);
        self.store
            .record_platform_managed_process_launch(&launch)
            .expect("record platform process launch");
        assert_eq!(
            self.store
                .platform_managed_process_launch("vault")
                .expect("read platform process launch"),
            Some(launch.clone())
        );
        assert!(
            self.store
                .record_platform_managed_process_launch(&launch)
                .is_err()
        );
        assert!(
            ManagedRuntimeExpectation::from_platform_fenced_launch(
                "vault", "vault", &binding, &launch
            )
            .is_ok()
        );
        assert!(
            ManagedRuntimeExpectation::from_platform_fenced_launch(
                "other", "vault", &binding, &launch
            )
            .is_err()
        );
    }

    fn assert_initial_approval(&self) {
        let approved = self
            .store
            .module_registration("registration-1")
            .expect("read approved registration")
            .expect("registration exists");
        assert_eq!(approved.grant_epoch(), 2);
        assert_eq!(
            self.store
                .module_grant_snapshot("registration-1")
                .expect("effective grants")
                .expect("registration snapshot")
                .effective_grants()
                .expect("approved grants")
                .capability_ids(),
            ["capability.read"]
        );
    }

    fn assert_settings_applied(&self) {
        let schema =
            SettingsSchemaBinding::new(makosh_kernel_control_store::SettingsSchemaBindingInputV1 {
                registration_id: "registration-1".to_owned(),
                schema_major: 1,
                schema_revision: 1,
                schema_sha256: [8; 32],
                desired_revision: 0,
                effective_revision: 0,
                apply_state: SettingsApplyState::Current,
                sanitized_reason_code: None,
            });
        self.store
            .register_settings_schema(&schema)
            .expect("bind settings schema");
        let revision = self
            .store
            .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
                registration_id: "registration-1".into(),
                configuration_instance_id: "registration-1".into(),
                expected_revision: 0,
                snapshot_bytes: vec![1, 2, 3],
            })
            .expect("commit desired settings");
        assert_eq!(revision, 1);
        assert_eq!(
            self.store
                .desired_settings_snapshot("registration-1")
                .expect("read desired snapshot"),
            Some((1, vec![1, 2, 3]))
        );
        let pending = self.settings_binding();
        assert_eq!(pending.desired_revision(), 1);
        assert_eq!(pending.apply_state(), SettingsApplyState::PendingValidation);
        assert!(
            self.store
                .transition_settings_apply_state(
                    "registration-1",
                    1,
                    SettingsApplyState::Applying,
                    None,
                )
                .is_err()
        );
        self.transition_settings(SettingsApplyState::PendingApply, "validation accepted");
        self.transition_settings(SettingsApplyState::Applying, "start apply");
        self.store
            .confirm_effective_settings_revision("registration-1", 1)
            .expect("confirm applied settings");
        let current = self.settings_binding();
        assert_eq!(current.effective_revision(), 1);
        assert_eq!(current.apply_state(), SettingsApplyState::Current);
    }

    fn assert_blocked_settings(&self) {
        let blocked_revision = self
            .store
            .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
                registration_id: "registration-1".into(),
                configuration_instance_id: "registration-1".into(),
                expected_revision: 1,
                snapshot_bytes: vec![4],
            })
            .expect("commit correction");
        assert!(
            self.store
                .transition_settings_apply_state(
                    "registration-1",
                    blocked_revision,
                    SettingsApplyState::BlockedConfig,
                    Some("invalid reason with spaces"),
                )
                .is_err()
        );
        self.store
            .transition_settings_apply_state(
                "registration-1",
                blocked_revision,
                SettingsApplyState::BlockedConfig,
                Some("validation.invalid_interval"),
            )
            .expect("record sanitized validation failure");
        let blocked = self.settings_binding();
        assert_eq!(blocked.apply_state(), SettingsApplyState::BlockedConfig);
        assert_eq!(
            blocked.sanitized_reason_code(),
            Some("validation.invalid_interval")
        );
        assert!(
            self.store
                .confirm_effective_settings_revision("registration-1", blocked_revision)
                .is_err()
        );
        assert!(
            self.store
                .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
                    registration_id: "registration-1".into(),
                    configuration_instance_id: "registration-1".into(),
                    expected_revision: 0,
                    snapshot_bytes: vec![4],
                })
                .is_err()
        );
        assert!(
            self.store
                .approve_module_registration(
                    "registration-1",
                    &["capability.unrequested".to_owned()]
                )
                .is_err()
        );
    }

    fn assert_registration_fences(&self) {
        let suspended = self
            .store
            .transition_module_registration("registration-1", ModuleRegistrationState::Suspended)
            .expect("suspend registration");
        assert_eq!(suspended.grant_epoch(), 3);
        let reapproved = self
            .store
            .approve_module_registration("registration-1", &["capability.write".to_owned()])
            .expect("replace approved capability subset");
        assert_eq!(reapproved.grant_epoch(), 4);
        assert_eq!(reapproved.capability_ids(), ["capability.write"]);
        assert!(
            self.store
                .effective_external_runtime_attestation("registration-1")
                .expect("stale external runtime")
                .is_none()
        );
        let suspended = self
            .store
            .transition_module_registration("registration-1", ModuleRegistrationState::Suspended)
            .expect("suspend reapproved registration");
        assert_eq!(suspended.grant_epoch(), 5);
        assert!(
            self.store
                .transition_module_registration("registration-1", ModuleRegistrationState::Pending,)
                .is_err()
        );
        let revoked = self
            .store
            .transition_module_registration("registration-1", ModuleRegistrationState::Revoked)
            .expect("revoke registration");
        assert_eq!(revoked.grant_epoch(), 6);
        assert!(
            self.store
                .transition_module_registration("registration-1", ModuleRegistrationState::Approved)
                .is_err()
        );
    }

    fn transition_settings(&self, state: SettingsApplyState, message: &str) {
        self.store
            .transition_settings_apply_state("registration-1", 1, state, None)
            .expect(message);
    }

    fn settings_binding(&self) -> SettingsSchemaBinding {
        self.store
            .settings_schema_binding("registration-1")
            .expect("read settings binding")
            .expect("settings binding")
    }
}

impl Drop for RegistrationFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn exports_a_consistent_control_store_with_a_checksum() {
    let directory = std::env::temp_dir().join(format!(
        "makosh-control-store-export-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir(&directory).expect("create temporary directory");
    let source = directory.join("source.sqlite");
    let destination = directory.join("export.sqlite");

    let store = SqliteControlStore::create(&source, "instance-1", 1).expect("create store");
    let export = store.export_to(&destination).expect("export store");

    assert_eq!(export.instance_id(), "instance-1");
    assert_eq!(export.generation(), 1);
    assert_eq!(export.sha256_hex().len(), 64);
    assert!(SqliteControlStore::open(&destination).is_ok());

    std::fs::remove_dir_all(directory).expect("remove temporary directory");
}

#[test]
fn v1_envelope_preserves_the_major_version_field_number() {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        ..Default::default()
    };

    assert_eq!(envelope.encode_to_vec(), vec![0x08, 0x01]);
}

#[test]
fn envelope_validation_requires_a_complete_typed_header_and_kind() {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: vec![1; 16],
        correlation_id: vec![1; 16],
        partition_key: b"owner:communications".to_vec(),
        contract: Some(ContractRefV1 {
            owner: "communications".into(),
            name: "message".into(),
            major: 1,
            revision: 1,
            schema_sha256: vec![2; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "mail".into(),
            runtime_instance_id: vec![3; 16],
            runtime_generation: 1,
        }),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: vec![4],
        }),
        recorded_at: Some(prost_types::Timestamp {
            seconds: 1,
            nanos: 0,
        }),
        semantics: Some(
            makosh_events_protocol::v1::durable_envelope_v1::Semantics::Event(EventMetadataV1 {
                occurred_at: Some(prost_types::Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
            }),
        ),
        ..Default::default()
    };
    assert!(decode_envelope_v1(&envelope.encode_to_vec()).is_ok());
    let mut missing_kind = envelope;
    missing_kind.semantics = None;
    assert!(decode_envelope_v1(&missing_kind.encode_to_vec()).is_err());
}

#[test]
fn recovery_only_has_a_stable_runtime_state_discriminant() {
    assert_eq!(KernelStateV1::RecoveryOnly as i32, 3);
}

#[test]
fn recovery_status_request_is_an_empty_typed_message() {
    assert!(
        GetRecoveryStatusRequestV1::default()
            .encode_to_vec()
            .is_empty()
    );
}

#[test]
fn recovery_control_envelope_preserves_the_status_operation_variant() {
    let request = RecoveryControlRequestV1 {
        operation: Some(
            makosh_gateway_protocol::v1::recovery_control_request_v1::Operation::GetRecoveryStatus(
                GetRecoveryStatusRequestV1 {},
            ),
        ),
    };

    let decoded = RecoveryControlRequestV1::decode(request.encode_to_vec().as_slice())
        .expect("decode recovery request");
    assert!(matches!(
        decoded.operation,
        Some(
            makosh_gateway_protocol::v1::recovery_control_request_v1::Operation::GetRecoveryStatus(
                _
            )
        )
    ));
}
