//! One-time platform credential seeding before the Vault runtime is launched.

mod credentials;
mod file_source;

use std::path::Path;

use makosh_vault_store_sqlcipher::VaultStore;

use self::credentials::PlatformCredentialSeeds;
use self::file_source::FilePlatformCredentialSource;

pub(crate) fn import_platform_credentials(
    store: &VaultStore,
    directory: Option<&Path>,
) -> Result<(), String> {
    let Some(directory) = directory else {
        return Ok(());
    };
    let seeds = FilePlatformCredentialSource::new(directory).read()?;
    store_platform_credentials(store, seeds)
}

fn store_platform_credentials(
    store: &VaultStore,
    seeds: PlatformCredentialSeeds,
) -> Result<(), String> {
    store
        .store_secrets_atomically(seeds.scoped_credentials()?)
        .map_err(|_| "Vault platform credential import failed".to_owned())?;
    Ok(())
}
