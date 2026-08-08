use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use makosh_hub_backend::platform::config::app_config::AppConfig;
use tempfile::TempDir;

static RETAINED_VAULTS: OnceLock<Mutex<Vec<TestVault>>> = OnceLock::new();

fn retained_vaults() -> &'static Mutex<Vec<TestVault>> {
    RETAINED_VAULTS.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Debug)]
pub struct TestVault {
    _dir: TempDir,
    vault_home: PathBuf,
    dev_key_path: PathBuf,
}

impl TestVault {
    pub fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("makosh-test-vault-")
            .tempdir()
            .expect("create temporary test vault directory");

        let vault_home = dir.path().join("vault");
        let dev_key_path = dir.path().join("dev").join("master.key");

        Self {
            _dir: dir,
            vault_home,
            dev_key_path,
        }
    }

    pub fn vault_home(&self) -> &Path {
        &self.vault_home
    }

    pub fn dev_key_path(&self) -> &Path {
        &self.dev_key_path
    }

    pub fn vault_database_path(&self) -> PathBuf {
        self.vault_home.join("vault.db")
    }

    pub fn apply_to_config(&self, config: AppConfig) -> AppConfig {
        config.with_test_dev_vault_paths(self.vault_home.clone(), self.dev_key_path.clone())
    }
}

impl Default for TestVault {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_test_vault() -> TestVault {
    TestVault::new()
}

pub fn retain_test_vault_and_apply(config: AppConfig) -> AppConfig {
    let vault = TestVault::new();
    let config = vault.apply_to_config(config);

    retained_vaults()
        .lock()
        .expect("test vault retention lock")
        .push(vault);

    config
}
