use std::fs;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use super::HostVault;
use super::constants::MASTER_KEY_LEN;
use super::crypto::decode_master_key;
use super::errors::HostVaultError;
use super::files::write_secure_file;

const SERVICE_NAME: &str = "makosh";
const KEYCHAIN_USER: &str = "host-vault-master-key";

impl HostVault {
    pub(super) fn has_stored_master_key(&self) -> Result<bool, HostVaultError> {
        if self.dev_mode {
            return Ok(self.dev_key_path.exists());
        }
        #[cfg(target_os = "macos")]
        {
            Ok(keyring_entry()?.get_password().is_ok())
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(HostVaultError::UnsupportedPlatform)
        }
    }

    pub(super) fn store_master_key(
        &self,
        master_key: &[u8; MASTER_KEY_LEN],
    ) -> Result<(), HostVaultError> {
        let encoded = BASE64_STANDARD.encode(master_key);
        if self.dev_mode {
            write_secure_file(&self.dev_key_path, encoded.as_bytes())?;
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            keyring_entry()?.set_password(&encoded)?;
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(HostVaultError::UnsupportedPlatform)
        }
    }

    pub(super) fn load_master_key(&self) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let encoded = if self.dev_mode {
            fs::read_to_string(&self.dev_key_path)?
        } else {
            #[cfg(target_os = "macos")]
            {
                keyring_entry()?.get_password()?
            }
            #[cfg(not(target_os = "macos"))]
            {
                return Err(HostVaultError::UnsupportedPlatform);
            }
        };
        decode_master_key(&encoded)
    }
}

#[cfg(target_os = "macos")]
fn keyring_entry() -> Result<keyring::Entry, HostVaultError> {
    Ok(keyring::Entry::new(SERVICE_NAME, KEYCHAIN_USER)?)
}
