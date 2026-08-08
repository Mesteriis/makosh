use super::common::*;
use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
    PlatformEventsAuthorityConfigurationV1,
};
use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, EventsAuthorityRuntimeControlRequestV1,
    EventsAuthorityRuntimeControlResponseV1, EventsAuthorityRuntimeStateV1,
    EventsAuthorityRuntimeStatusV1, GetEventsAuthorityRuntimeStatusRequestV1,
    GetVaultRuntimeStatusRequestV1, ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedVaultRuntimeControlRequestV1, ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1,
    VaultRuntimeStatusV1, events_authority_runtime_control_response_v1::Result as AuthorityResult,
    managed_runtime_control_request_v1::Operation as ManagedOperation,
    managed_runtime_control_response_v1::Result as ManagedResult,
    managed_vault_runtime_control_request_v1::Operation as VaultOperation,
    managed_vault_runtime_control_response_v1::Result as VaultResult,
};

use crate::platform::events::authority::{binding, launch};
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};

const ACCOUNT_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const AUTHORITY_ARTIFACT_ID: &str = "platform.events-authority";

#[test]
fn kernel_starts_events_authority_as_a_fenced_managed_child() {
    let fixture = AuthorityLaunchFixture::new();
    let store = fixture.store();
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));

    fixture.start_vault(&supervisor);
    binding::bind_installed_release(&store, fixture.release.kernel())
        .expect("bind signed Events authority release");
    store
        .record_platform_events_authority_configuration(
            &PlatformEventsAuthorityConfigurationV1::new(1, ACCOUNT_KEY, 4),
        )
        .expect("record Events authority configuration");
    store
        .record_platform_event_hub_topology(&event_hub_topology())
        .expect("record Event Hub topology");

    assert_eq!(fixture.start_authority(&supervisor, &store), 1);
    assert!(
        supervisor
            .is_active("events_authority")
            .expect("read Events authority worker")
    );
    let record = store
        .platform_managed_process_launch("events_authority")
        .expect("read Events authority launch")
        .expect("Events authority launch record");
    assert_eq!(record.runtime_generation(), 1);
    assert_eq!(record.grant_epoch(), store.snapshot().grant_epoch());

    supervisor.shutdown().expect("stop managed children");
}

fn event_hub_topology() -> PlatformEventHubTopologyV1 {
    let stream_budgets = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ]
    .into_iter()
    .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
    .collect();
    PlatformEventHubTopologyV1::new(1, "nats://127.0.0.1:4222", "event_hub", 1, stream_budgets)
}

struct AuthorityLaunchFixture {
    root: std::path::PathBuf,
    release: InstalledSignedBundle,
    vault_child: std::path::PathBuf,
}

impl AuthorityLaunchFixture {
    fn new() -> Self {
        let root = unique_target_root("makosh-events-authority-managed-launch");
        std::fs::create_dir_all(&root).expect("create fixture directory");
        let authority_descriptor = authority_descriptor();
        let authority_schema = authority_schema();
        let authority_child =
            write_authority_child(&root, &authority_descriptor, &authority_schema);
        let release = InstalledSignedBundle::install(
            &root,
            &[SignedRuntimeArtifact::new(
                AUTHORITY_ARTIFACT_ID,
                authority_child,
                authority_descriptor,
            )
            .with_settings_schema(authority_schema)],
        )
        .expect("install signed Events authority release");
        let vault_child = write_vault_child(&root);
        Self {
            root,
            release,
            vault_child,
        }
    }

    fn store(&self) -> SqliteControlStore {
        let store = SqliteControlStore::create(&self.root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store");
        store
            .record_platform_managed_process_binding(&PlatformManagedProcessBinding::new(
                "vault",
                1,
                "fixture",
                "platform.vault",
                [1; 32],
                [2; 32],
                None,
            ))
            .expect("record Vault binding");
        store
            .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
                "vault", 1, 1, 3, 1,
            ))
            .expect("record Vault launch");
        store
    }

    fn start_vault(&self, supervisor: &ManagedRuntimeSupervisor) {
        let descriptor = vault_descriptor();
        let descriptor_bytes = descriptor.encode_to_vec();
        let digest: [u8; 32] =
            Sha256::digest(std::fs::read(&self.vault_child).expect("read Vault fixture child"))
                .into();
        let staged = staged_native_artifact::stage(
            &self.vault_child,
            &self.root.join("vault-launch"),
            "vault-fixture",
            &digest,
        )
        .expect("stage Vault fixture child");
        supervisor
            .start(
                "vault".to_owned(),
                staged,
                ManagedRuntimeExpectation::new(
                    "vault",
                    "vault",
                    "vault",
                    3,
                    1,
                    Sha256::digest(descriptor_bytes).into(),
                    None,
                ),
                ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                    .expect("Vault fixture execution policy"),
            )
            .expect("start Vault fixture child");
    }

    fn start_authority(
        &self,
        supervisor: &ManagedRuntimeSupervisor,
        store: &SqliteControlStore,
    ) -> u64 {
        launch::start_from_kernel(
            supervisor,
            store,
            self.release.kernel(),
            &self.root.join("runtime"),
        )
        .expect("start Events authority")
    }
}

impl Drop for AuthorityLaunchFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn authority_schema() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        ..Default::default()
    }
    .encode_to_vec()
}

fn authority_descriptor() -> Vec<u8> {
    let schema = authority_schema();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "events".to_owned(),
        owner_id: "events".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "events-authority-test".to_owned(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: schema.len() as u64,
            sha256: Sha256::digest(schema).to_vec(),
        }),
        ..Default::default()
    }
    .encode_to_vec()
}

fn vault_descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".to_owned(),
        owner_id: "vault".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "vault-fixture".to_owned(),
        ..Default::default()
    }
}

fn write_authority_child(
    root: &std::path::Path,
    descriptor: &[u8],
    schema: &[u8],
) -> std::path::PathBuf {
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Describe(
            DescribeManagedRuntimeRequestV1 {
                descriptor_bytes: descriptor.to_vec(),
                settings_schema_bytes: schema.to_vec(),
            },
        )),
    };
    let ready = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
            registration_id: "events_authority".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        })),
    };
    let response = EventsAuthorityRuntimeControlResponseV1 {
        result: Some(AuthorityResult::Status(EventsAuthorityRuntimeStatusV1 {
            state: EventsAuthorityRuntimeStateV1::Ready as i32,
            runtime_generation: 1,
            grant_epoch: 1,
            vault_runtime_generation: 3,
            signer_credential_revision: 4,
            blocker_code: String::new(),
        })),
        error_code: String::new(),
    };
    write_child_script(
        root.join("events-authority-child.sh"),
        &[
            describe.encode_to_vec(),
            ready.encode_to_vec(),
            response.encode_to_vec(),
        ],
        authority_status_request_length(),
        describe_response_length("events_authority", 1, 1),
    )
}

fn authority_status_request_length() -> usize {
    EventsAuthorityRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::events_authority_runtime_control_request_v1::Operation::GetStatus(
                GetEventsAuthorityRuntimeStatusRequestV1 {},
            ),
        ),
    }
    .encode_to_vec()
    .len()
    + 1
}

fn write_vault_child(root: &std::path::Path) -> std::path::PathBuf {
    let descriptor = vault_descriptor().encode_to_vec();
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Describe(
            DescribeManagedRuntimeRequestV1 {
                descriptor_bytes: descriptor,
                settings_schema_bytes: Vec::new(),
            },
        )),
    };
    let response = ManagedVaultRuntimeControlResponseV1 {
        result: Some(VaultResult::Status(VaultRuntimeStatusV1 {
            state: VaultRuntimeStateV1::Ready as i32,
            vault_runtime_generation: 3,
            hpke_public_key_x25519: vec![9; 32],
            blocker_code: String::new(),
        })),
        error_code: String::new(),
    };
    let request = ManagedVaultRuntimeControlRequestV1 {
        operation: Some(VaultOperation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
    };
    write_child_script(
        root.join("vault-child.sh"),
        &[describe.encode_to_vec(), response.encode_to_vec()],
        request.encode_to_vec().len() + 1,
        describe_response_length("vault", 3, 1),
    )
}

fn write_child_script(
    path: std::path::PathBuf,
    frames: &[Vec<u8>],
    request_length: usize,
    describe_response_length: usize,
) -> std::path::PathBuf {
    let describe = frame(&frames[0]);
    let response = frame(frames.last().expect("child response"));
    let middle = frames
        .get(1)
        .filter(|_| frames.len() == 3)
        .map(|value| format!("printf '{}' >&0\n", shell_binary_literal(&frame(value))))
        .unwrap_or_default();
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count={describe_response_length} of=/dev/null 2>/dev/null\n{middle}dd bs=1 count={request_length} of=/dev/null 2>/dev/null\nprintf '{}' >&0\nsleep 30\n",
            shell_binary_literal(&describe),
            shell_binary_literal(&response),
        ),
    )
    .expect("write fixture child");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("make fixture child executable");
    path
}

fn describe_response_length(registration_id: &str, generation: u64, grant_epoch: u64) -> usize {
    let response = ManagedRuntimeControlResponseV1 {
        result: Some(ManagedResult::Describe(DescribeManagedRuntimeResponseV1 {
            registration_id: registration_id.to_owned(),
            runtime_generation: generation,
            grant_epoch,
        })),
        error_code: String::new(),
    };
    frame(&response.encode_to_vec()).len()
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() < 128, "fixture frame stays single-byte");
    [vec![bytes.len() as u8], bytes.to_vec()].concat()
}
