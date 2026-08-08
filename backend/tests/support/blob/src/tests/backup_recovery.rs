use std::fs;
use std::os::unix::fs::PermissionsExt;

use makosh_blob_runtime::recovery::{
    export_backup_offline, restore_backup_offline, verify_backup_offline,
};

#[test]
fn blob_backup_restores_exact_ciphertext_inventory_only_into_an_empty_target() {
    let root = tempfile::tempdir().expect("temporary root");
    private_directory(root.path());
    let source = root.path().join("source");
    let backup = root.path().join("backup");
    let restored = root.path().join("restored");
    create_blob_root(&source);
    write_private(
        &source
            .join("content")
            .join("07070707070707070707070707070707.blob"),
        b"ciphertext-only-content",
    );
    write_private(
        &source
            .join("metadata")
            .join("07070707070707070707070707070707.meta"),
        b"ciphertext-metadata",
    );

    let manifest = export_backup_offline(&source, &backup).expect("export Blob backup");
    assert_eq!(manifest.entries().len(), 2);
    assert!(verify_backup_offline(&backup).is_ok());
    restore_backup_offline(&backup, &restored).expect("restore into empty target");
    assert_eq!(
        fs::read(restored.join("content/07070707070707070707070707070707.blob"))
            .expect("restored content"),
        b"ciphertext-only-content"
    );
    assert!(
        fs::read_dir(root.path())
            .expect("read recovery parent")
            .all(|entry| !entry
                .expect("read recovery entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".blob-restore-"))
    );

    write_private(&restored.join("unexpected"), b"occupied");
    assert!(restore_backup_offline(&backup, &restored).is_err());
    write_private(&backup.join("unexpected"), b"unexpected");
    assert!(verify_backup_offline(&backup).is_err());
    fs::remove_file(backup.join("unexpected")).expect("remove unexpected backup file");
    write_private(
        &backup
            .join("content")
            .join("07070707070707070707070707070707.blob"),
        b"tampered-ciphertext",
    );
    assert!(verify_backup_offline(&backup).is_err());
}

fn create_blob_root(path: &std::path::Path) {
    fs::create_dir(path).expect("source root");
    private_directory(path);
    for child in ["content", "metadata"] {
        let child = path.join(child);
        fs::create_dir(&child).expect("Blob child root");
        private_directory(&child);
    }
}

fn write_private(path: &std::path::Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("private fixture");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture mode");
}

fn private_directory(path: &std::path::Path) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private directory mode");
}
