use std::os::unix::fs::{PermissionsExt, symlink};

use makosh_storage_postgres::{InitdbPasswordFileV1, PostgresAdapterErrorV1};
use tempfile::TempDir;
use zeroize::Zeroizing;

#[test]
fn initdb_password_file_is_private_one_shot_and_removed_explicitly() {
    let runtime = private_runtime_directory();
    let password = Zeroizing::new(b"fenced-platform-password".to_vec());
    let file = InitdbPasswordFileV1::create(runtime.path(), &password).expect("password file");
    let path = file.path().to_path_buf();

    assert_eq!(
        path.metadata()
            .expect("password metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::read(&path).expect("password bytes"),
        b"fenced-platform-password\n"
    );
    file.remove().expect("explicit removal");
    assert!(!path.exists());
}

#[test]
fn initdb_password_file_rejects_untrusted_directory_and_password_bytes() {
    let runtime = TempDir::new().expect("runtime directory");
    let password = Zeroizing::new(b"fenced-platform-password".to_vec());
    assert!(matches!(
        InitdbPasswordFileV1::create(runtime.path(), &password),
        Err(PostgresAdapterErrorV1::Bootstrap)
    ));

    let private = private_runtime_directory();
    let invalid_password = Zeroizing::new(b"contains\nnewline".to_vec());
    assert!(matches!(
        InitdbPasswordFileV1::create(private.path(), &invalid_password),
        Err(PostgresAdapterErrorV1::Bootstrap)
    ));
}

#[test]
fn initdb_password_file_rejects_a_symlinked_runtime_directory() {
    let target = private_runtime_directory();
    let parent = TempDir::new().expect("parent directory");
    let alias = parent.path().join("runtime-alias");
    symlink(target.path(), &alias).expect("symlink runtime directory");
    let password = Zeroizing::new(b"fenced-platform-password".to_vec());

    assert!(matches!(
        InitdbPasswordFileV1::create(&alias, &password),
        Err(PostgresAdapterErrorV1::Bootstrap)
    ));
}

fn private_runtime_directory() -> TempDir {
    let directory = TempDir::new().expect("runtime directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private runtime directory");
    directory
}
