use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1,
    ModuleEventRouteRequestV1, ModuleRegistration, ModuleRegistrationState,
    PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
    ReconcileEventsTopologyResponseV1,
    events_authority_runtime_control_request_v1::Operation as EventsOperation,
    events_authority_runtime_control_response_v1::Result as EventsResult,
};
use p256::ecdsa::signature::Signer;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::control_store::lifecycle::bootstrap_control_store;
use crate::distribution::staged_artifact::{StagedNativeArtifact, stage};
use crate::infrastructure::filesystem::resolve_runtime_directory;
use crate::recovery::capture_coordinator::capture_verified_instance;
use crate::recovery::media::encryption::RecoveryMediaEncryptionKey;
use crate::recovery::media::format::{RecoveryMediaInventoryV1, RecoveryMediaProvenanceV1};
use crate::recovery::media::publish::RecoveryMediaSigner;
use crate::recovery::media::verification::open_verified_recovery_media;
use crate::recovery::process_port::{
    PostgresRecoveryCommandV1, ProcessWholeInstanceCapturePort, ProcessWholeInstanceRestorePort,
    RecoveryComponentExecutables,
};
use crate::recovery::restore_coordinator::restore_verified_instance;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

use super::common::{SigningKey, unique_target_root};

#[test]
fn process_ports_round_trip_staged_components_and_exact_inventory() {
    let fixture = Fixture::new();
    let executable = fixture.stage_component();
    let signer = TestSigner::new();
    let public_key = signer.key.verifying_key().to_sec1_point(false);
    let encryption_key = RecoveryMediaEncryptionKey::new([29; 32]);
    let published = capture_media(&fixture, &executable, &signer, &encryption_key);
    assert_media_inventory(&fixture, &published, public_key.as_bytes(), &encryption_key);
    restore_media(
        &fixture,
        &executable,
        &published,
        public_key.as_bytes(),
        &encryption_key,
    );
}

fn capture_media(
    fixture: &Fixture,
    executable: &StagedNativeArtifact,
    signer: &TestSigner,
    encryption_key: &RecoveryMediaEncryptionKey,
) -> PathBuf {
    let store = bootstrap_control_store(
        &fixture.data,
        &fixture.data.join("kernel-control-store.sqlite"),
    )
    .expect("bootstrap Control Store");
    install_event_topology(&store);
    let mut port = ProcessWholeInstanceCapturePort::open(
        fixture.data.clone(),
        executables(executable),
        postgres(fixture.root.join("password")),
    )
    .expect("open stopped-instance capture port");
    fixture.private_file("password", b"test-password");
    let published = capture_verified_instance(
        &fixture.media,
        provenance(),
        RecoveryMediaInventoryV1::new(true, true),
        signer,
        encryption_key,
        &mut port,
    )
    .expect("capture whole instance");
    drop(port);
    published
}

fn assert_media_inventory(
    fixture: &Fixture,
    published: &Path,
    public_key: &[u8],
    encryption_key: &RecoveryMediaEncryptionKey,
) {
    let (_, payload) = open_verified_recovery_media(
        published,
        "process-port-key",
        public_key,
        &fixture.root,
        encryption_key,
    )
    .expect("open captured media");
    for path in [
        "control-store/control-store.sqlite",
        "control-store/.makosh-installation-anchor-v1",
        "control-store/.makosh-recovery-fence-v1",
        "vault/snapshot/data.bin",
        "storage/postgres.dump",
        "blob/snapshot/data.bin",
        "event-hub/topology.pb",
        "scheduler/storage-bundle.pb",
    ] {
        assert!(payload.root().join(path).is_file(), "missing {path}");
    }
}

fn restore_media(
    fixture: &Fixture,
    executable: &StagedNativeArtifact,
    published: &Path,
    public_key: &[u8],
    encryption_key: &RecoveryMediaEncryptionKey,
) {
    let relay = TopologyRelay;
    let recovery_key = fixture.private_file("vault-recovery-key", b"test recovery key");
    let mut port = ProcessWholeInstanceRestorePort::open(
        fixture.restore.clone(),
        executables(executable),
        postgres(fixture.root.join("password")),
        recovery_key,
        &relay,
    )
    .expect("open restore port");
    restore_verified_instance(
        published,
        "process-port-key",
        public_key,
        &fixture.root,
        encryption_key,
        &mut port,
    )
    .expect("restore whole instance");
    drop(port);
    let restored = crate::control_store::lifecycle::open_validated_control_store(
        &fixture.restore.join("kernel-control-store.sqlite"),
    )
    .expect("open restored Control Store");
    assert!(
        restored
            .approved_module_grant_snapshots()
            .expect("restored grants")
            .is_empty()
    );
    assert!(fixture.restore.join("vault/restored.bin").is_file());
    assert!(fixture.restore.join("blob/restored.bin").is_file());
}

fn executables(executable: &StagedNativeArtifact) -> RecoveryComponentExecutables<'_> {
    RecoveryComponentExecutables {
        vault: executable,
        storage: executable,
        blob: Some(executable),
        scheduler: Some(executable),
    }
}

fn postgres(password_file: PathBuf) -> PostgresRecoveryCommandV1 {
    PostgresRecoveryCommandV1 {
        pg_dump: PathBuf::from("/usr/bin/true"),
        pg_restore: PathBuf::from("/usr/bin/true"),
        psql: PathBuf::from("/usr/bin/true"),
        host: "127.0.0.1".to_owned(),
        port: 5432,
        database: "makosh".to_owned(),
        username: "recovery".to_owned(),
        ssl_mode: "disable".to_owned(),
        password_file,
    }
}

struct TopologyRelay;

impl ManagedRuntimeRelay for TopologyRelay {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        assert_eq!(registration_id, "events_authority");
        let request = EventsAuthorityRuntimeControlRequestV1::decode(payload.as_slice())
            .expect("topology request");
        let Some(EventsOperation::ReconcileTopology(topology)) = request.operation else {
            panic!("expected topology reconciliation");
        };
        Ok(EventsAuthorityRuntimeControlResponseV1 {
            result: Some(EventsResult::TopologyReconciled(
                ReconcileEventsTopologyResponseV1 {
                    topology_revision: topology.topology_revision,
                    stream_count: topology.streams.len() as u32,
                    consumer_count: topology.consumers.len() as u32,
                },
            )),
            error_code: String::new(),
        }
        .encode_to_vec())
    }
}

fn install_event_topology(store: &SqliteControlStore) {
    let registration = ModuleRegistration::new(
        "registration_notes",
        "module_notes",
        "owner_notes",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    let capability = "events.publish".to_owned();
    let route = ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_id: capability.clone(),
        envelope_kind: ModuleEventEnvelopeKindV1::Event,
        contract_owner: "owner_notes".to_owned(),
        contract_name: "changed".to_owned(),
        contract_major: 1,
        contract_revision: 1,
        contract_schema_sha256: [7; 32],
        direction: ModuleEventRouteDirectionV1::Publish,
        max_in_flight: 32,
        delivery_policy: None,
    });
    store
        .create_pending_registration_with_requests(
            &registration,
            std::slice::from_ref(&capability),
            &[],
            &[route],
            &[],
        )
        .expect("persist event publisher");
    store
        .approve_module_registration(registration.registration_id(), &[capability])
        .expect("approve event publisher");
    let budgets = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ]
    .into_iter()
    .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
    .collect();
    store
        .record_platform_event_hub_topology(&PlatformEventHubTopologyV1::new(
            1,
            "nats://127.0.0.1:4222",
            "event_hub",
            1,
            budgets,
        ))
        .expect("record Event Hub topology");
}

struct TestSigner {
    key: SigningKey,
}

impl TestSigner {
    fn new() -> Self {
        Self {
            key: SigningKey::from_bytes((&[23_u8; 32]).into()).expect("signing key"),
        }
    }
}

impl RecoveryMediaSigner for TestSigner {
    fn key_id(&self) -> &str {
        "process-port-key"
    }

    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String> {
        let signature: p256::ecdsa::Signature = self.key.sign(manifest);
        Ok(signature.to_bytes().into())
    }
}

struct Fixture {
    root: PathBuf,
    data: PathBuf,
    media: PathBuf,
    restore: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = unique_target_root("makosh-process-capture-port");
        let data = root.join("data");
        let restore = root.join("restore");
        fs::create_dir_all(&data).expect("create data directory");
        fs::create_dir(&restore).expect("create restore directory");
        for directory in [&root, &data, &restore] {
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
                .expect("private fixture directory");
        }
        Self {
            media: root.join("media"),
            root,
            data,
            restore,
        }
    }

    fn stage_component(&self) -> StagedNativeArtifact {
        let script = self.root.join("component.sh");
        let bytes = b"#!/bin/sh\nset -eu\ncommand_name=$1\nshift\ndestination=\ndata_dir=\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --destination|--output) destination=$2; shift 2 ;;\n    --data-dir) data_dir=$2; shift 2 ;;\n    *) shift ;;\n  esac\ndone\ncase \"$command_name\" in\n  export-backup)\n    case \"$destination\" in\n      */snapshot) mkdir \"$destination\"; printf component > \"$destination/data.bin\" ;;\n      *) printf postgres > \"$destination\" ;;\n    esac\n    ;;\n  export-recovery-bundle) printf scheduler > \"$destination\" ;;\n  restore-backup) if [ -n \"$data_dir\" ]; then printf restored > \"$data_dir/restored.bin\"; fi ;;\n  prepare-event-replay) exit 0 ;;\n  *) exit 64 ;;\nesac\n";
        fs::write(&script, bytes).expect("write component script");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700))
            .expect("executable component script");
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        stage(&script, &self.root.join("launch"), "component", &digest)
            .expect("stage verified component")
    }

    fn private_file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, bytes).expect("write private file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private file");
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Ok(canonical) = self.data.canonicalize()
            && let Ok(runtime) = resolve_runtime_directory(&canonical)
        {
            let _ = fs::remove_dir_all(runtime);
        }
        if let Ok(canonical) = self.restore.canonicalize()
            && let Ok(runtime) = resolve_runtime_directory(&canonical)
        {
            let _ = fs::remove_dir_all(runtime);
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn provenance() -> RecoveryMediaProvenanceV1 {
    RecoveryMediaProvenanceV1::new(9, "c".repeat(40), [1; 32], [2; 32], [3; 32])
        .expect("provenance")
}
