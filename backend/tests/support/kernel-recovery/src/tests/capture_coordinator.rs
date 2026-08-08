use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::recovery::capture_coordinator::{WholeInstanceCapturePort, capture_verified_instance};
use crate::recovery::media::encryption::RecoveryMediaEncryptionKey;
use crate::recovery::media::format::{RecoveryMediaInventoryV1, RecoveryMediaProvenanceV1};
use crate::recovery::media::publish::RecoveryMediaSigner;
use crate::recovery::media::verification::verify_published_recovery_media;
use crate::tests::common::{Signer, SigningKey, unique_target_root};

#[test]
fn capture_coordinator_publishes_every_enabled_component_in_canonical_order() {
    let parent = private_parent("makosh-whole-capture");
    let destination = parent.join("published");
    let signer = TestSigner::new();
    let public_key = signer.key.verifying_key().to_sec1_point(false);
    let encryption_key = encryption_key();
    let mut port = RecordingCapturePort::default();
    let published = capture_verified_instance(
        &destination,
        provenance(),
        RecoveryMediaInventoryV1::new(true, true),
        &signer,
        &encryption_key,
        &mut port,
    )
    .expect("capture");
    assert_eq!(
        port.calls,
        [
            "quiesced",
            "control",
            "vault",
            "storage",
            "blob",
            "events",
            "scheduler"
        ]
    );
    assert!(
        verify_published_recovery_media(&published, "capture-key", public_key.as_bytes()).is_ok()
    );
    fs::remove_dir_all(parent).expect("cleanup");
}

#[test]
fn capture_coordinator_omits_disabled_conditional_components() {
    let parent = private_parent("makosh-whole-capture-required");
    let destination = parent.join("published");
    let signer = TestSigner::new();
    let encryption_key = encryption_key();
    let mut port = RecordingCapturePort::default();
    capture_verified_instance(
        &destination,
        provenance(),
        RecoveryMediaInventoryV1::new(false, false),
        &signer,
        &encryption_key,
        &mut port,
    )
    .expect("capture");
    assert_eq!(
        port.calls,
        ["quiesced", "control", "vault", "storage", "events"]
    );
    fs::remove_dir_all(parent).expect("cleanup");
}

#[test]
fn capture_coordinator_removes_staging_after_component_failure() {
    let parent = private_parent("makosh-whole-capture-failure");
    let destination = parent.join("published");
    let signer = TestSigner::new();
    let encryption_key = encryption_key();
    let mut port = RecordingCapturePort {
        fail_at: Some("storage"),
        ..Default::default()
    };
    assert!(
        capture_verified_instance(
            &destination,
            provenance(),
            RecoveryMediaInventoryV1::new(true, true),
            &signer,
            &encryption_key,
            &mut port,
        )
        .is_err()
    );
    assert_eq!(port.calls, ["quiesced", "control", "vault", "storage"]);
    assert_eq!(fs::read_dir(&parent).expect("parent").count(), 0);
    fs::remove_dir(parent).expect("cleanup");
}

#[derive(Default)]
struct RecordingCapturePort {
    calls: Vec<&'static str>,
    fail_at: Option<&'static str>,
}

impl RecordingCapturePort {
    fn capture(&mut self, name: &'static str, destination: &Path) -> Result<(), String> {
        self.calls.push(name);
        if self.fail_at == Some(name) {
            return Err("component capture failed".to_owned());
        }
        fs::write(destination.join("snapshot.bin"), name.as_bytes())
            .map_err(|error| error.to_string())
    }
}

impl WholeInstanceCapturePort for RecordingCapturePort {
    fn verify_quiesced(&mut self) -> Result<(), String> {
        self.calls.push("quiesced");
        (self.fail_at != Some("quiesced"))
            .then_some(())
            .ok_or_else(|| "instance is not quiesced".to_owned())
    }
    fn capture_control_store(&mut self, destination: &Path) -> Result<(), String> {
        self.capture("control", destination)
    }
    fn capture_vault(&mut self, destination: &Path) -> Result<(), String> {
        self.capture("vault", destination)
    }
    fn capture_storage(&mut self, destination: &Path) -> Result<(), String> {
        self.capture("storage", destination)
    }
    fn capture_blob(&mut self, destination: &Path) -> Result<(), String> {
        self.capture("blob", destination)
    }
    fn capture_event_topology(&mut self, destination: &Path) -> Result<(), String> {
        self.capture("events", destination)
    }
    fn capture_scheduler(&mut self, destination: &Path) -> Result<(), String> {
        self.capture("scheduler", destination)
    }
}

struct TestSigner {
    key: SigningKey,
}

impl TestSigner {
    fn new() -> Self {
        Self {
            key: SigningKey::from_bytes((&[19_u8; 32]).into()).expect("key"),
        }
    }
}

impl RecoveryMediaSigner for TestSigner {
    fn key_id(&self) -> &str {
        "capture-key"
    }

    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String> {
        let signature: p256::ecdsa::Signature = self.key.sign(manifest);
        Ok(signature.to_bytes().into())
    }
}

fn private_parent(name: &str) -> PathBuf {
    let parent = unique_target_root(name);
    fs::create_dir(&parent).expect("parent");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).expect("private parent");
    parent
}

fn provenance() -> RecoveryMediaProvenanceV1 {
    RecoveryMediaProvenanceV1::new(7, "b".repeat(40), [1; 32], [2; 32], [3; 32])
        .expect("provenance")
}

fn encryption_key() -> RecoveryMediaEncryptionKey {
    RecoveryMediaEncryptionKey::new([29; 32])
}
