use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::recovery::media::encryption::RecoveryMediaEncryptionKey;
use crate::recovery::media::format::{
    RecoveryMediaComponentV1, RecoveryMediaEntryV1, RecoveryMediaInclusionV1,
    RecoveryMediaInventoryV1, RecoveryMediaManifestV1, RecoveryMediaProvenanceV1,
};
use crate::recovery::media::layout::{MANIFEST_FILE, PAYLOAD_DIRECTORY, SIGNATURE_FILE};
use crate::recovery::media::publish::{RecoveryMediaPublisher, RecoveryMediaSigner};
use crate::recovery::media::signature::SignedRecoveryMediaManifestV1;
use crate::recovery::media::verification::{
    open_verified_recovery_media, verify_inventory, verify_published_recovery_media,
    verify_signed_inventory,
};
use crate::tests::common::{Signer, SigningKey, unique_target_root};

#[test]
fn recovery_media_requires_an_exact_regular_file_inventory() {
    let root = unique_target_root("makosh-recovery-media");
    fs::create_dir_all(root.join("vault")).expect("media root");
    let entry = write_entry(
        &root,
        RecoveryMediaComponentV1::Vault,
        RecoveryMediaInclusionV1::Required,
        "vault/snapshot.bin",
        b"verified-vault-snapshot",
    );
    assert!(verify_inventory(&root, std::slice::from_ref(&entry)).is_ok());

    fs::write(root.join("extra.bin"), b"unexpected").expect("extra file");
    assert!(verify_inventory(&root, &[entry]).is_err());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn recovery_media_rejects_path_escape_digest_drift_and_wrong_classification() {
    assert!(
        RecoveryMediaEntryV1::new(
            RecoveryMediaComponentV1::Vault,
            RecoveryMediaInclusionV1::Required,
            "../escape".to_owned(),
            1,
            [0; 32]
        )
        .is_err()
    );
    assert!(
        RecoveryMediaEntryV1::new(
            RecoveryMediaComponentV1::Blob,
            RecoveryMediaInclusionV1::Required,
            "blob/object.bin".to_owned(),
            1,
            [0; 32]
        )
        .is_err()
    );
    let root = unique_target_root("makosh-recovery-media-digest");
    fs::create_dir_all(root.join("control-store")).expect("media root");
    fs::write(root.join("control-store/store.bin"), b"changed").expect("control store");
    let entry = RecoveryMediaEntryV1::new(
        RecoveryMediaComponentV1::ControlStore,
        RecoveryMediaInclusionV1::Required,
        "control-store/store.bin".to_owned(),
        7,
        [0; 32],
    )
    .expect("entry");
    assert!(verify_inventory(&root, &[entry]).is_err());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn recovery_media_rejects_symlinked_manifest_entry() {
    let root = unique_target_root("makosh-recovery-media-symlink");
    fs::create_dir_all(root.join("vault")).expect("media root");
    let external = unique_target_root("makosh-recovery-media-external");
    fs::write(&external, b"external").expect("external file");
    std::os::unix::fs::symlink(&external, root.join("vault/snapshot.bin")).expect("media symlink");
    let entry = RecoveryMediaEntryV1::new(
        RecoveryMediaComponentV1::Vault,
        RecoveryMediaInclusionV1::Required,
        "vault/snapshot.bin".to_owned(),
        8,
        Sha256::digest(b"external").into(),
    )
    .expect("entry");
    assert!(verify_inventory(&root, &[entry]).is_err());
    fs::remove_dir_all(root).expect("cleanup media");
    fs::remove_file(external).expect("cleanup external");
}

#[test]
fn recovery_media_requires_the_pinned_manifest_signature() {
    let key = SigningKey::from_bytes((&[7_u8; 32]).into()).expect("signing key");
    let root = unique_target_root("makosh-recovery-media-signed");
    fs::create_dir_all(&root).expect("media root");
    let raw = RecoveryMediaManifestV1::encode(
        provenance(),
        RecoveryMediaInventoryV1::new(false, false),
        required_entries(&root),
    )
    .expect("manifest");
    let signature: p256::ecdsa::Signature = key.sign(&raw);
    let manifest = SignedRecoveryMediaManifestV1::new(
        "recovery-media-2026".to_owned(),
        raw,
        signature.to_bytes().into(),
    )
    .expect("signed manifest");
    let public_key = key.verifying_key().to_sec1_point(false);
    assert!(
        verify_signed_inventory(
            &root,
            &manifest,
            "recovery-media-2026",
            public_key.as_bytes()
        )
        .is_ok()
    );
    assert!(
        verify_signed_inventory(&root, &manifest, "different-key", public_key.as_bytes()).is_err()
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn recovery_media_binds_conditional_components_to_the_signed_plan() {
    let root = unique_target_root("makosh-recovery-media-plan");
    fs::create_dir_all(&root).expect("media root");
    let mut entries = required_entries(&root);
    entries.push(write_entry(
        &root,
        RecoveryMediaComponentV1::Blob,
        RecoveryMediaInclusionV1::Conditional,
        "blob/object.bin",
        b"encrypted-blob",
    ));
    entries.sort_by(|left, right| left.canonical_order().cmp(&right.canonical_order()));
    assert!(
        RecoveryMediaManifestV1::encode(
            provenance(),
            RecoveryMediaInventoryV1::new(false, false),
            entries.clone()
        )
        .is_err()
    );
    assert!(
        RecoveryMediaManifestV1::encode(
            provenance(),
            RecoveryMediaInventoryV1::new(true, false),
            entries
        )
        .is_ok()
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn recovery_media_publisher_creates_one_private_verified_directory() {
    let parent = private_parent("makosh-recovery-publish");
    let destination = parent.join("published");
    let publisher = RecoveryMediaPublisher::create(&destination).expect("publisher");
    let _ = required_entries(publisher.payload_root());
    let signer = TestMediaSigner::new("media-key", 13);
    let public_key = signer.key.verifying_key().to_sec1_point(false);
    let encryption_key = encryption_key();
    let published = publisher
        .publish(
            provenance(),
            RecoveryMediaInventoryV1::new(false, false),
            &signer,
            &encryption_key,
        )
        .expect("publish");
    assert_eq!(published, destination);
    assert!(
        verify_published_recovery_media(&published, "media-key", public_key.as_bytes()).is_ok()
    );
    assert_private_directory(&published);
    assert_private_directory(&published.join(PAYLOAD_DIRECTORY));
    assert_private_file(&published.join(MANIFEST_FILE));
    assert_private_file(&published.join(SIGNATURE_FILE));
    fs::remove_dir_all(parent).expect("cleanup");
}

#[test]
fn recovery_media_publisher_rechecks_payload_and_removes_failed_staging() {
    let parent = private_parent("makosh-recovery-publish-race");
    let destination = parent.join("published");
    let publisher = RecoveryMediaPublisher::create(&destination).expect("publisher");
    let _ = required_entries(publisher.payload_root());
    let signer = MutatingMediaSigner {
        signer: TestMediaSigner::new("media-key", 17),
        target: publisher
            .payload_root()
            .parent()
            .expect("staging")
            .join("payload/control-store/store.bin"),
    };
    let encryption_key = encryption_key();
    assert!(
        publisher
            .publish(
                provenance(),
                RecoveryMediaInventoryV1::new(false, false),
                &signer,
                &encryption_key,
            )
            .is_err()
    );
    assert!(!destination.exists());
    assert_eq!(fs::read_dir(&parent).expect("parent").count(), 0);
    fs::remove_dir(parent).expect("cleanup");
}

#[test]
fn recovery_media_encrypts_large_payloads_and_rejects_the_wrong_recovery_key() {
    let parent = private_parent("makosh-recovery-encryption");
    let destination = parent.join("published");
    let publisher = RecoveryMediaPublisher::create(&destination).expect("publisher");
    let mut plaintext = vec![0x5a; 1024 * 1024 + 37];
    write_required_payload_with_control_store(publisher.payload_root(), &plaintext);
    let signer = TestMediaSigner::new("media-key", 23);
    let public_key = signer.key.verifying_key().to_sec1_point(false);
    let key = encryption_key();
    let published = publisher
        .publish(
            provenance(),
            RecoveryMediaInventoryV1::new(false, false),
            &signer,
            &key,
        )
        .expect("publish encrypted media");
    assert_payload_is_encrypted(&published, &plaintext);
    assert_wrong_key_is_rejected(&published, public_key.as_bytes(), &parent);
    let decrypted = decrypt_payload(&published, public_key.as_bytes(), &parent, &key);
    assert_eq!(
        fs::read(decrypted.root().join("control-store/store.bin")).expect("read plaintext"),
        plaintext
    );
    plaintext.fill(0);
    drop(decrypted);
    assert_eq!(
        fs::read_dir(&parent)
            .expect("parent")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".makosh-recovery-decrypt-")
            })
            .count(),
        0
    );
    fs::remove_dir_all(parent).expect("cleanup");
}

fn assert_payload_is_encrypted(published: &Path, plaintext: &[u8]) {
    let ciphertext = fs::read(
        published
            .join(PAYLOAD_DIRECTORY)
            .join("control-store/store.bin"),
    )
    .expect("read ciphertext");
    assert_ne!(ciphertext, plaintext);
    assert!(
        !ciphertext
            .windows(64)
            .any(|window| window == &plaintext[..64])
    );
}

fn assert_wrong_key_is_rejected(published: &Path, public_key: &[u8], parent: &Path) {
    let wrong_key = RecoveryMediaEncryptionKey::new([31; 32]);
    assert!(
        open_verified_recovery_media(published, "media-key", public_key, parent, &wrong_key)
            .is_err()
    );
}

fn decrypt_payload(
    published: &Path,
    public_key: &[u8],
    parent: &Path,
    key: &RecoveryMediaEncryptionKey,
) -> crate::recovery::media::encryption::DecryptedRecoveryPayload {
    open_verified_recovery_media(published, "media-key", public_key, parent, key)
        .expect("decrypt verified media")
        .1
}

#[test]
fn recovery_media_rejects_invalid_signed_provenance() {
    assert!(RecoveryMediaProvenanceV1::new(0, "a".repeat(40), [1; 32], [2; 32], [3; 32]).is_err());
    assert!(
        RecoveryMediaProvenanceV1::new(1, "not-a-commit".to_owned(), [1; 32], [2; 32], [3; 32])
            .is_err()
    );
}

fn required_entries(root: &Path) -> Vec<RecoveryMediaEntryV1> {
    [
        (
            RecoveryMediaComponentV1::ControlStore,
            "control-store/store.bin",
        ),
        (RecoveryMediaComponentV1::Vault, "vault/snapshot.bin"),
        (RecoveryMediaComponentV1::Storage, "storage/postgres.dump"),
        (RecoveryMediaComponentV1::EventHub, "event-hub/topology.bin"),
    ]
    .into_iter()
    .map(|(component, path)| {
        write_entry(
            root,
            component,
            RecoveryMediaInclusionV1::Required,
            path,
            path.as_bytes(),
        )
    })
    .collect()
}

fn write_required_payload_with_control_store(root: &Path, control_store: &[u8]) {
    for (path, bytes) in [
        ("control-store/store.bin", control_store),
        ("vault/snapshot.bin", b"vault".as_slice()),
        ("storage/postgres.dump", b"storage".as_slice()),
        ("event-hub/topology.bin", b"events".as_slice()),
    ] {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().expect("parent")).expect("component directory");
        fs::write(destination, bytes).expect("component media");
    }
}

fn write_entry(
    root: &Path,
    component: RecoveryMediaComponentV1,
    inclusion: RecoveryMediaInclusionV1,
    path: &str,
    bytes: &[u8],
) -> RecoveryMediaEntryV1 {
    let destination = root.join(path);
    fs::create_dir_all(destination.parent().expect("parent")).expect("component directory");
    fs::write(&destination, bytes).expect("component media");
    RecoveryMediaEntryV1::new(
        component,
        inclusion,
        path.to_owned(),
        bytes.len() as u64,
        Sha256::digest(bytes).into(),
    )
    .expect("entry")
}

struct TestMediaSigner {
    key_id: &'static str,
    key: SigningKey,
}

impl TestMediaSigner {
    fn new(key_id: &'static str, key_byte: u8) -> Self {
        Self {
            key_id,
            key: SigningKey::from_bytes((&[key_byte; 32]).into()).expect("key"),
        }
    }
}

impl RecoveryMediaSigner for TestMediaSigner {
    fn key_id(&self) -> &str {
        self.key_id
    }

    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String> {
        let signature: p256::ecdsa::Signature = self.key.sign(manifest);
        Ok(signature.to_bytes().into())
    }
}

struct MutatingMediaSigner {
    signer: TestMediaSigner,
    target: std::path::PathBuf,
}

impl RecoveryMediaSigner for MutatingMediaSigner {
    fn key_id(&self) -> &str {
        self.signer.key_id()
    }

    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String> {
        fs::write(&self.target, b"changed-after-first-inventory").expect("mutate payload");
        self.signer.sign(manifest)
    }
}

fn private_parent(name: &str) -> std::path::PathBuf {
    let parent = unique_target_root(name);
    fs::create_dir(&parent).expect("parent");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).expect("private parent");
    parent
}

fn assert_private_directory(path: &Path) {
    let metadata = fs::symlink_metadata(path).expect("directory metadata");
    assert!(metadata.is_dir());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
    assert_eq!(metadata.uid(), current_uid());
}

fn assert_private_file(path: &Path) {
    let metadata = fs::symlink_metadata(path).expect("file metadata");
    assert!(metadata.is_file());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    assert_eq!(metadata.uid(), current_uid());
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}

fn provenance() -> RecoveryMediaProvenanceV1 {
    RecoveryMediaProvenanceV1::new(1, "a".repeat(40), [1; 32], [2; 32], [3; 32])
        .expect("provenance")
}

fn encryption_key() -> RecoveryMediaEncryptionKey {
    RecoveryMediaEncryptionKey::new([29; 32])
}
