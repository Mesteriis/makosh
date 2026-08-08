use std::path::Path;

use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};

pub(crate) fn read_secret_key(path: &Path) -> Result<[u8; 32], String> {
    let bytes = read_secure_file(path, SecureReadPolicy::owner_private(32))
        .map_err(|_| "recovery key file is unavailable".to_owned())?;
    bytes
        .try_into()
        .map_err(|_| "recovery key file has an invalid length".to_owned())
}
