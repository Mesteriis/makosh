use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

use makosh_secure_file::{SecureReadPolicy, read};

use super::common::unique_target_root;

#[test]
fn private_read_is_fd_bound_bounded_and_never_follows_a_symlink() {
    let root = unique_target_root("makosh-secure-file");
    fs::create_dir_all(&root).expect("create fixture directory");
    let source = root.join("source");
    fs::write(&source, b"private bytes").expect("write fixture");
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).expect("private mode");

    assert_eq!(
        read(&source, SecureReadPolicy::owner_private(64)).expect("secure read"),
        b"private bytes",
    );
    assert!(read(&source, SecureReadPolicy::owner_private(4)).is_err());

    let link = root.join("link");
    symlink(&source, &link).expect("create symlink");
    assert!(read(&link, SecureReadPolicy::owner_private(64)).is_err());
    fs::remove_dir_all(root).expect("remove fixture directory");
}
