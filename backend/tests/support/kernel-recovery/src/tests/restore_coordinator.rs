use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::recovery::media::encryption::RecoveryMediaEncryptionKey;
use crate::recovery::media::format::{RecoveryMediaInventoryV1, RecoveryMediaProvenanceV1};
use crate::recovery::media::layout::PAYLOAD_DIRECTORY;
use crate::recovery::media::publish::{RecoveryMediaPublisher, RecoveryMediaSigner};
use crate::recovery::restore_coordinator::{WholeInstanceRestorePort, restore_verified_instance};
use crate::tests::common::{Signer, SigningKey, unique_target_root};

#[test]
fn coordinator_restores_verified_components_in_canonical_order() {
    let (root, public_key) = published_media(true, true);
    let key = encryption_key();
    let mut port = RecordingPort::default();
    restore_verified_instance(
        &root,
        "recovery-key",
        &public_key,
        root.parent().expect("workspace"),
        &key,
        &mut port,
    )
    .expect("restore");
    assert_eq!(
        port.calls,
        [
            "empty",
            "control",
            "vault",
            "storage",
            "blob",
            "events_topology",
            "outbox_inbox_replay",
            "fence"
        ]
    );
    cleanup_media(&root);
}

#[test]
fn coordinator_stops_before_later_components_after_failure() {
    let (root, public_key) = published_media(true, true);
    let key = encryption_key();
    let mut port = RecordingPort {
        fail_at: Some("storage"),
        ..Default::default()
    };
    assert!(
        restore_verified_instance(
            &root,
            "recovery-key",
            &public_key,
            root.parent().expect("workspace"),
            &key,
            &mut port,
        )
        .is_err()
    );
    assert_eq!(port.calls, ["empty", "control", "vault", "storage"]);
    cleanup_media(&root);
}

#[test]
fn coordinator_requires_an_empty_target_before_control_store_restore() {
    let (root, public_key) = published_media(false, false);
    let key = encryption_key();
    let mut port = RecordingPort {
        fail_at: Some("empty"),
        ..Default::default()
    };
    assert!(
        restore_verified_instance(
            &root,
            "recovery-key",
            &public_key,
            root.parent().expect("workspace"),
            &key,
            &mut port,
        )
        .is_err()
    );
    assert_eq!(port.calls, ["empty"]);
    cleanup_media(&root);
}

#[test]
fn coordinator_rejects_media_changed_after_publication() {
    let (root, public_key) = published_media(false, false);
    let key = encryption_key();
    fs::write(
        root.join(PAYLOAD_DIRECTORY).join("control-store/store.bin"),
        b"tampered-whole-instance-media",
    )
    .expect("tamper media");
    let mut port = RecordingPort::default();
    assert!(
        restore_verified_instance(
            &root,
            "recovery-key",
            &public_key,
            root.parent().expect("workspace"),
            &key,
            &mut port,
        )
        .is_err()
    );
    assert!(port.calls.is_empty());
    cleanup_media(&root);
}

#[derive(Default)]
struct RecordingPort {
    calls: Vec<&'static str>,
    fail_at: Option<&'static str>,
}

impl RecordingPort {
    fn call(&mut self, name: &'static str) -> Result<(), String> {
        self.calls.push(name);
        if self.fail_at == Some(name) {
            return Err("component failed".to_owned());
        }
        Ok(())
    }
}

impl WholeInstanceRestorePort for RecordingPort {
    fn verify_empty_target(&mut self) -> Result<(), String> {
        self.call("empty")
    }
    fn restore_control_store(&mut self, _source: &Path) -> Result<(), String> {
        self.call("control")
    }
    fn restore_vault(&mut self, _source: &Path) -> Result<(), String> {
        self.call("vault")
    }
    fn restore_storage(&mut self, _source: &Path) -> Result<(), String> {
        self.call("storage")
    }
    fn restore_blob(&mut self, _source: &Path) -> Result<(), String> {
        self.call("blob")
    }
    fn recreate_event_topology(
        &mut self,
        _source: &Path,
        _control_store_source: &Path,
    ) -> Result<(), String> {
        self.call("events_topology")
    }
    fn prepare_outbox_inbox_replay(
        &mut self,
        scheduler_source: Option<&Path>,
    ) -> Result<(), String> {
        assert!(scheduler_source.is_some());
        self.call("outbox_inbox_replay")
    }
    fn invalidate_stale_runtime_state(&mut self) -> Result<(), String> {
        self.call("fence")
    }
}

struct TestMediaSigner {
    key: SigningKey,
}

impl RecoveryMediaSigner for TestMediaSigner {
    fn key_id(&self) -> &str {
        "recovery-key"
    }

    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String> {
        let signature: p256::ecdsa::Signature = self.key.sign(manifest);
        Ok(signature.to_bytes().into())
    }
}

fn published_media(blob_enabled: bool, scheduler_enabled: bool) -> (PathBuf, [u8; 65]) {
    let parent = unique_target_root("makosh-whole-restore-parent");
    fs::create_dir(&parent).expect("media parent");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).expect("private parent");
    let destination = parent.join("published");
    let publisher = RecoveryMediaPublisher::create(&destination).expect("publisher");
    write_required_payload(publisher.payload_root());
    if blob_enabled {
        write_payload(publisher.payload_root(), "blob/object.bin");
    }
    if scheduler_enabled {
        write_payload(publisher.payload_root(), "scheduler/state.bin");
    }
    let signer = TestMediaSigner {
        key: SigningKey::from_bytes((&[11_u8; 32]).into()).expect("key"),
    };
    let public_key = signer
        .key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("public key");
    let encryption_key = encryption_key();
    let published = publisher
        .publish(
            provenance(),
            RecoveryMediaInventoryV1::new(blob_enabled, scheduler_enabled),
            &signer,
            &encryption_key,
        )
        .expect("publish media");
    (published, public_key)
}

fn write_required_payload(root: &Path) {
    for path in [
        "control-store/store.bin",
        "vault/snapshot.bin",
        "storage/postgres.dump",
        "event-hub/topology.bin",
    ] {
        write_payload(root, path);
    }
}

fn write_payload(root: &Path, path: &str) {
    let destination = root.join(path);
    fs::create_dir_all(destination.parent().expect("parent")).expect("component directory");
    fs::write(destination, path.as_bytes()).expect("component media");
}

fn provenance() -> RecoveryMediaProvenanceV1 {
    RecoveryMediaProvenanceV1::new(1, "a".repeat(40), [1; 32], [2; 32], [3; 32])
        .expect("provenance")
}

fn cleanup_media(root: &Path) {
    fs::remove_dir_all(root.parent().expect("media parent")).expect("cleanup");
}

fn encryption_key() -> RecoveryMediaEncryptionKey {
    RecoveryMediaEncryptionKey::new([29; 32])
}
