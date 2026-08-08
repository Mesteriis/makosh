# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `073-source-backend-part-053`
- Group / Группа: `backend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/src/platform/storage/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/storage/errors.rs`
- Size bytes / Размер в байтах: `481`
- Included characters / Включено символов: `481`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::EventStoreError;
use crate::platform::settings::SettingsError;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to connect to PostgreSQL")]
    Connect(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error("{0}")]
    Invalid(String),
}
```

### `backend/src/platform/storage/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/storage/mod.rs`
- Size bytes / Размер в байтах: `530`
- Included characters / Включено символов: `530`
- Truncated / Обрезано: `no`

```rust
mod communication_media;
mod database;
mod errors;
mod models;

pub use communication_media::{
    ImportedAttachmentRecord, ImportedAttachmentRemovalResult, ImportedAttachmentStoragePort,
    ImportedAttachmentUpsert, LocalBlobRecord, SafetyScanReport, SafetyScanRequest,
    SafetyScanStatus, StoredBlobRecord, delete_local_blob, new_attachment_import_id,
    put_local_blob, scan_attachment,
};
pub use database::Database;
pub use errors::StorageError;
pub use models::{DatabaseReadiness, MigrationReadiness, ReadinessStatus};
```

### `backend/src/platform/storage/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/storage/models.rs`
- Size bytes / Размер в байтах: `1821`
- Included characters / Включено символов: `1821`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatabaseReadiness {
    status: ReadinessStatus,
    message: &'static str,
}

impl DatabaseReadiness {
    pub(crate) fn ok() -> Self {
        Self {
            status: ReadinessStatus::Ok,
            message: "database is reachable",
        }
    }

    pub(crate) fn not_configured() -> Self {
        Self {
            status: ReadinessStatus::NotConfigured,
            message: "DATABASE_URL is not configured",
        }
    }

    pub(crate) fn unavailable(message: &'static str) -> Self {
        Self {
            status: ReadinessStatus::Unavailable,
            message,
        }
    }

    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn message(&self) -> &str {
        self.message
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MigrationReadiness {
    status: ReadinessStatus,
    message: &'static str,
}

impl MigrationReadiness {
    pub(crate) fn ok() -> Self {
        Self {
            status: ReadinessStatus::Ok,
            message: "required database migrations are applied",
        }
    }

    pub(crate) fn not_configured() -> Self {
        Self {
            status: ReadinessStatus::NotConfigured,
            message: "DATABASE_URL is not configured",
        }
    }

    pub(crate) fn unavailable(message: &'static str) -> Self {
        Self {
            status: ReadinessStatus::Unavailable,
            message,
        }
    }

    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn message(&self) -> &str {
        self.message
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    Ok,
    NotConfigured,
    Unavailable,
}
```

### `backend/src/vault/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/constants.rs`
- Size bytes / Размер в байтах: `138`
- Included characters / Включено символов: `138`
- Truncated / Обрезано: `no`

```rust
pub(super) const VAULT_VERSION: u16 = 1;
pub(super) const MIN_ENTROPY_EVENTS: usize = 2_000;
pub(super) const MASTER_KEY_LEN: usize = 32;
```

### `backend/src/vault/crypto.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/crypto.rs`
- Size bytes / Размер в байтах: `3252`
- Included characters / Включено символов: `3252`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use hkdf::Hkdf;
use sha2::{Digest, Sha512};

use super::constants::{MASTER_KEY_LEN, MIN_ENTROPY_EVENTS, VAULT_VERSION};
use super::errors::HostVaultError;
use super::models::{EntropyEvent, SecretEntryContext};

pub(super) fn derive_master_key(
    os_random: &[u8],
    entropy: &[EntropyEvent],
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let entropy_json = serde_json::to_vec(entropy)?;
    let mut hasher = Sha512::new();
    hasher.update(os_random);
    hasher.update(&entropy_json);
    hasher.update(
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_be_bytes(),
    );
    let digest = hasher.finalize();
    let hkdf = Hkdf::<sha2::Sha256>::new(None, &digest);
    let mut key = [0_u8; MASTER_KEY_LEN];
    hkdf.expand(b"makosh-host-vault:master:v1", &mut key)
        .map_err(|_| HostVaultError::Crypto)?;
    Ok(key)
}

pub(super) fn derive_domain_key(
    master_key: &[u8; MASTER_KEY_LEN],
    label: &[u8],
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let hkdf = Hkdf::<sha2::Sha256>::new(None, master_key);
    let mut key = [0_u8; MASTER_KEY_LEN];
    let mut info = b"makosh-host-vault:v1:".to_vec();
    info.extend_from_slice(label);
    hkdf.expand(&info, &mut key)
        .map_err(|_| HostVaultError::Crypto)?;
    Ok(key)
}

pub(super) fn entry_aad(secret_ref: &str, context: SecretEntryContext<'_>) -> String {
    format!(
        "v={VAULT_VERSION};ref={};kind={};account_id={};purpose={};secret_kind={}",
        secret_ref.trim(),
        context.entry_kind.trim(),
        context.account_id.trim(),
        context.purpose.trim(),
        context.secret_kind.trim()
    )
}

pub(super) fn recovery_phrase(master_key: &[u8; MASTER_KEY_LEN]) -> Result<String, HostVaultError> {
    Ok(master_key
        .chunks(2)
        .map(|chunk| format!("{:02x}{:02x}", chunk[0], chunk[1]))
        .collect::<Vec<_>>()
        .join(" "))
}

pub(super) fn master_key_from_recovery_phrase(
    phrase: &str,
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let compact = phrase.split_whitespace().collect::<String>();
    if compact.len() != MASTER_KEY_LEN * 2 {
        return Err(HostVaultError::InvalidRecoveryPhrase);
    }
    let mut key = [0_u8; MASTER_KEY_LEN];
    for index in 0..MASTER_KEY_LEN {
        let byte = u8::from_str_radix(&compact[index * 2..index * 2 + 2], 16)
            .map_err(|_| HostVaultError::InvalidRecoveryPhrase)?;
        key[index] = byte;
    }
    Ok(key)
}

pub(super) fn decode_master_key(encoded: &str) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let decoded = BASE64_STANDARD
        .decode(encoded.trim())
        .map_err(|_| HostVaultError::InvalidEncoding)?;
    decoded
        .try_into()
        .map_err(|_| HostVaultError::InvalidEncoding)
}

pub(super) fn entropy_progress(events: usize) -> u8 {
    ((events.min(MIN_ENTROPY_EVENTS) * 100) / MIN_ENTROPY_EVENTS) as u8
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), HostVaultError> {
    if value.trim().is_empty() {
        return Err(HostVaultError::EmptyField(field));
    }
    Ok(())
}
```

### `backend/src/vault/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/errors.rs`
- Size bytes / Размер в байтах: `3490`
- Included characters / Включено символов: `3490`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::secrets::SecretResolutionError;

#[derive(Debug, Error)]
pub enum HostVaultError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Keyring(#[from] keyring::Error),

    #[error("host vault is not initialized")]
    Uninitialized,

    #[error("host vault is already initialized")]
    AlreadyInitialized,

    #[error("host vault is locked")]
    Locked,

    #[error("host vault state is poisoned")]
    StatePoisoned,

    #[error("insufficient vault entropy: collected {collected}, required {required}")]
    InsufficientEntropy { collected: usize, required: usize },

    #[error("entropy batch must not be empty")]
    EmptyEntropyBatch,

    #[error("host vault cryptographic operation failed")]
    Crypto,

    #[error("host vault random generation failed")]
    Random,

    #[error("invalid host vault encoding")]
    InvalidEncoding,

    #[error("invalid host vault recovery phrase")]
    InvalidRecoveryPhrase,

    #[error("unsupported host vault version: {0}")]
    UnsupportedVaultVersion(u16),

    #[error("secret was not found in host vault: {secret_ref}")]
    MissingSecret { secret_ref: String },

    #[error("host vault dev mode is forbidden in release builds")]
    DevModeForbiddenInRelease,

    #[error("host vault release runtime is macOS-only")]
    UnsupportedPlatform,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl HostVaultError {
    fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid host vault key or corrupted encrypted payload".to_owned(),
            Self::InvalidEncoding => "invalid host vault encoding".to_owned(),
            Self::InvalidRecoveryPhrase => "invalid host vault recovery phrase".to_owned(),
            Self::Locked => "host vault is locked".to_owned(),
            Self::Uninitialized => "host vault is not initialized".to_owned(),
            Self::MissingSecret { secret_ref } => format!("secret was not found: {secret_ref}"),
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::InsufficientEntropy {
                collected,
                required,
            } => {
                format!("insufficient entropy: collected {collected}, required {required}")
            }
            Self::AlreadyInitialized => "host vault is already initialized".to_owned(),
            Self::EmptyEntropyBatch => "entropy batch must not be empty".to_owned(),
            Self::UnsupportedVaultVersion(_) => "unsupported host vault version".to_owned(),
            Self::DevModeForbiddenInRelease => {
                "host vault dev mode is forbidden in release".to_owned()
            }
            Self::UnsupportedPlatform => "host vault release runtime is macOS-only".to_owned(),
            Self::Io(_) | Self::Sqlite(_) | Self::Json(_) | Self::StatePoisoned | Self::Random => {
                "host vault operation failed".to_owned()
            }
            #[cfg(target_os = "macos")]
            Self::Keyring(_) => "macOS Keychain operation failed".to_owned(),
        }
    }
}

pub(super) fn host_secret_store_failure(error: HostVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}
```

### `backend/src/vault/files.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/files.rs`
- Size bytes / Размер в байтах: `1408`
- Included characters / Включено символов: `1408`
- Truncated / Обрезано: `no`

```rust
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::errors::HostVaultError;

pub(super) fn write_secure_file(path: &Path, bytes: &[u8]) -> Result<(), HostVaultError> {
    if let Some(parent) = path.parent() {
        ensure_secure_dir(parent)?;
    }
    let temp_path = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    ensure_secure_file(&temp_path)?;
    fs::rename(&temp_path, path)?;
    ensure_secure_file(path)?;
    Ok(())
}

pub(super) fn ensure_secure_dir(path: &Path) -> Result<(), HostVaultError> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub(super) fn ensure_secure_file(path: &Path) -> Result<(), HostVaultError> {
    if !path.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub(super) fn guard_release_dev_mode(dev_mode: bool) -> Result<(), HostVaultError> {
    if dev_mode && !cfg!(debug_assertions) {
        return Err(HostVaultError::DevModeForbiddenInRelease);
    }
    Ok(())
}
```

### `backend/src/vault/key_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/key_store.rs`
- Size bytes / Размер в байтах: `2030`
- Included characters / Включено символов: `2030`
- Truncated / Обрезано: `no`

```rust
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
```

### `backend/src/vault/lifecycle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/lifecycle.rs`
- Size bytes / Размер в байтах: `5274`
- Included characters / Включено символов: `5274`
- Truncated / Обрезано: `no`

```rust
use std::sync::{Arc, Mutex};

use zeroize::Zeroize;

use super::HostVault;
use super::constants::{MASTER_KEY_LEN, MIN_ENTROPY_EVENTS, VAULT_VERSION};
use super::crypto::{derive_domain_key, derive_master_key, entropy_progress};
use super::errors::HostVaultError;
use super::files::{ensure_secure_dir, guard_release_dev_mode};
use super::models::{
    EntropyEvent, HostVaultConfig, HostVaultState, SessionKey, VaultMode, VaultStatus,
};

impl HostVault {
    pub fn new(config: HostVaultConfig) -> Result<Self, HostVaultError> {
        guard_release_dev_mode(config.dev_mode)?;
        ensure_secure_dir(&config.home)?;
        if let Some(parent) = config.dev_key_path.parent() {
            ensure_secure_dir(parent)?;
        }

        let vault = Self {
            home: config.home,
            dev_mode: config.dev_mode,
            dev_key_path: config.dev_key_path,
            state: Arc::new(Mutex::new(HostVaultState::Locked)),
            entropy: Arc::new(Mutex::new(Vec::new())),
        };
        vault.initialize_database()?;
        Ok(vault)
    }

    pub fn status(&self) -> Result<VaultStatus, HostVaultError> {
        let initialized = self.has_stored_master_key()?;
        let state = if !initialized {
            VaultMode::Uninitialized
        } else if self.is_unlocked()? {
            VaultMode::Unlocked
        } else {
            VaultMode::Locked
        };
        let entropy_events = self
            .entropy
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?
            .len();

        Ok(VaultStatus {
            state,
            needs_entropy: !initialized && entropy_events < MIN_ENTROPY_EVENTS,
            needs_biometric: initialized && !self.dev_mode,
            needs_recovery: !initialized,
            version: VAULT_VERSION,
            recoverable: self.recovery_file_path().exists(),
            entropy_progress: entropy_progress(entropy_events),
        })
    }

    pub fn collect_entropy(
        &self,
        events: Vec<EntropyEvent>,
    ) -> Result<VaultStatus, HostVaultError> {
        if events.is_empty() {
            return Err(HostVaultError::EmptyEntropyBatch);
        }
        let mut entropy = self
            .entropy
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        entropy.extend(events);
        drop(entropy);
        self.status()
    }

    pub fn create(&self) -> Result<VaultStatus, HostVaultError> {
        if self.has_stored_master_key()? {
            return Err(HostVaultError::AlreadyInitialized);
        }
        let entropy = self
            .entropy
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        if entropy.len() < MIN_ENTROPY_EVENTS {
            return Err(HostVaultError::InsufficientEntropy {
                collected: entropy.len(),
                required: MIN_ENTROPY_EVENTS,
            });
        }

        let mut os_random = [0_u8; 64];
        getrandom::getrandom(&mut os_random).map_err(|_| HostVaultError::Random)?;
        let mut master_key = derive_master_key(&os_random, &entropy)?;
        drop(entropy);

        self.store_master_key(&master_key)?;
        self.set_unlocked(SessionKey::new(master_key))?;
        master_key.zeroize();
        self.write_vault_check()?;
        self.status()
    }

    pub fn unlock(&self) -> Result<VaultStatus, HostVaultError> {
        if !self.has_stored_master_key()? {
            return Err(HostVaultError::Uninitialized);
        }
        let master_key = self.load_master_key()?;
        self.set_unlocked(SessionKey::new(master_key))?;
        self.read_vault_check()?;
        self.status()
    }

    pub fn unlock_existing(&self) -> Result<VaultStatus, HostVaultError> {
        if !self.has_stored_master_key()? {
            return self.status();
        }
        self.unlock()
    }

    pub fn lock(&self) -> Result<VaultStatus, HostVaultError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        *state = HostVaultState::Locked;
        drop(state);
        self.status()
    }

    pub(super) fn domain_key(&self, label: &[u8]) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let key = self.current_master_key()?;
        derive_domain_key(&key, label)
    }

    pub(super) fn current_master_key(&self) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
        let state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        match &*state {
            HostVaultState::Unlocked(key) => Ok(key.bytes),
            HostVaultState::Locked => Err(HostVaultError::Locked),
        }
    }

    pub(super) fn set_unlocked(&self, key: SessionKey) -> Result<(), HostVaultError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        *state = HostVaultState::Unlocked(key);
        Ok(())
    }

    pub(super) fn is_unlocked(&self) -> Result<bool, HostVaultError> {
        let state = self
            .state
            .lock()
            .map_err(|_| HostVaultError::StatePoisoned)?;
        Ok(matches!(*state, HostVaultState::Unlocked(_)))
    }
}
```

### `backend/src/vault/manifest.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/manifest.rs`
- Size bytes / Размер в байтах: `3677`
- Included characters / Включено символов: `3677`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use rusqlite::params;

use super::HostVault;
use super::crypto::validate_non_empty;
use super::errors::HostVaultError;
use super::models::{HostVaultManifestEntry, SecretEntryContext};

impl HostVault {
    pub fn account_secret_manifest(&self) -> Result<Vec<HostVaultManifestEntry>, HostVaultError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            r#"
            SELECT secret_ref, entry_kind, account_id, purpose, secret_kind, store_kind, label, metadata, updated_at
            FROM account_secret_manifest
            ORDER BY account_id ASC, purpose ASC, secret_ref ASC
            "#,
        )?;
        let mut rows = statement.query([])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let metadata: String = row.get("metadata")?;
            entries.push(HostVaultManifestEntry {
                secret_ref: row.get("secret_ref")?,
                entry_kind: row.get("entry_kind")?,
                account_id: row.get("account_id")?,
                purpose: row.get("purpose")?,
                secret_kind: row.get("secret_kind")?,
                store_kind: row.get("store_kind")?,
                label: row.get("label")?,
                metadata: serde_json::from_str(&metadata)?,
                updated_at: row.get("updated_at")?,
            });
        }
        Ok(entries)
    }

    pub fn upsert_account_secret_manifest_entry(
        &self,
        secret_ref: &str,
        context: SecretEntryContext<'_>,
    ) -> Result<(), HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        validate_non_empty("entry_kind", context.entry_kind)?;
        validate_non_empty("account_id", context.account_id)?;
        validate_non_empty("purpose", context.purpose)?;
        self.upsert_manifest_entry(secret_ref, context)
    }

    pub(super) fn upsert_manifest_entry(
        &self,
        secret_ref: &str,
        context: SecretEntryContext<'_>,
    ) -> Result<(), HostVaultError> {
        let metadata = serde_json::to_string(&context.metadata)?;
        let now = Utc::now().to_rfc3339();
        self.connection()?.execute(
            r#"
            INSERT INTO account_secret_manifest (
                secret_ref, entry_kind, account_id, purpose, secret_kind, store_kind, label, metadata, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, 'host_vault', ?6, ?7, ?8)
            ON CONFLICT(secret_ref)
            DO UPDATE SET
                entry_kind = excluded.entry_kind,
                account_id = excluded.account_id,
                purpose = excluded.purpose,
                secret_kind = excluded.secret_kind,
                store_kind = excluded.store_kind,
                label = excluded.label,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at
            "#,
            params![
                secret_ref.trim(),
                context.entry_kind.trim(),
                context.account_id.trim(),
                context.purpose.trim(),
                context.secret_kind,
                context.label.trim(),
                metadata,
                now
            ],
        )?;
        Ok(())
    }

    pub(super) fn delete_manifest_entry(&self, secret_ref: &str) -> Result<bool, HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let deleted = self.connection()?.execute(
            r#"
            DELETE FROM account_secret_manifest
            WHERE secret_ref = ?1
            "#,
            params![secret_ref.trim()],
        )?;
        Ok(deleted > 0)
    }
}
```

### `backend/src/vault/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/mod.rs`
- Size bytes / Размер в байтах: `675`
- Included characters / Включено символов: `675`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod crypto;
mod errors;
mod files;
mod key_store;
mod lifecycle;
mod manifest;
mod models;
mod paths;
mod recovery;
mod secrets;
mod storage;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use errors::HostVaultError;
use models::HostVaultState;
pub use models::{
    EntropyEvent, HostVaultConfig, HostVaultManifestEntry, RecoveryExportResponse,
    SecretEntryContext, VaultMode, VaultStatus,
};
pub use paths::{default_dev_key_path, default_vault_home};

#[derive(Clone)]
pub struct HostVault {
    home: PathBuf,
    dev_mode: bool,
    dev_key_path: PathBuf,
    state: Arc<Mutex<HostVaultState>>,
    entropy: Arc<Mutex<Vec<EntropyEvent>>>,
}
```

### `backend/src/vault/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/models.rs`
- Size bytes / Размер в байтах: `2346`
- Included characters / Включено символов: `2346`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::constants::MASTER_KEY_LEN;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostVaultConfig {
    pub home: PathBuf,
    pub dev_mode: bool,
    pub dev_key_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultMode {
    Uninitialized,
    Locked,
    Unlocked,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VaultStatus {
    pub state: VaultMode,
    pub needs_entropy: bool,
    pub needs_biometric: bool,
    pub needs_recovery: bool,
    pub version: u16,
    pub recoverable: bool,
    pub entropy_progress: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntropyEvent {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub timestamp_ms: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub interval_ms: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SecretEntryContext<'a> {
    pub entry_kind: &'a str,
    pub account_id: &'a str,
    pub purpose: &'a str,
    pub secret_kind: &'a str,
    pub label: &'a str,
    pub metadata: &'a serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryExportResponse {
    pub path: PathBuf,
    pub recovery_phrase: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HostVaultManifestEntry {
    pub secret_ref: String,
    pub entry_kind: String,
    pub account_id: String,
    pub purpose: String,
    pub secret_kind: String,
    pub store_kind: String,
    pub label: String,
    pub metadata: serde_json::Value,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct RecoveryFile {
    pub(super) version: u16,
    pub(super) nonce: String,
    pub(super) ciphertext: String,
}

#[derive(Debug)]
pub(super) struct StoredVaultEntry {
    pub(super) version: u16,
    pub(super) nonce: String,
    pub(super) ciphertext: String,
    pub(super) aad: String,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub(super) struct SessionKey {
    pub(super) bytes: [u8; MASTER_KEY_LEN],
}

impl SessionKey {
    pub(super) fn new(bytes: [u8; MASTER_KEY_LEN]) -> Self {
        Self { bytes }
    }
}

pub(super) enum HostVaultState {
    Locked,
    Unlocked(SessionKey),
}
```

### `backend/src/vault/paths.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/paths.rs`
- Size bytes / Размер в байтах: `255`
- Included characters / Включено символов: `255`
- Truncated / Обрезано: `no`

```rust
use std::path::{Path, PathBuf};

pub fn default_vault_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".makosh").join("vault")
}

pub fn default_dev_key_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".makosh").join("dev").join("master.key")
}
```

### `backend/src/vault/recovery.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/recovery.rs`
- Size bytes / Размер в байтах: `1901`
- Included characters / Включено символов: `1901`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305};

use super::HostVault;
use super::constants::VAULT_VERSION;
use super::crypto::{
    derive_domain_key, master_key_from_recovery_phrase, recovery_phrase, validate_non_empty,
};
use super::errors::HostVaultError;
use super::files::write_secure_file;
use super::models::{RecoveryExportResponse, RecoveryFile, SessionKey, VaultStatus};

impl HostVault {
    pub fn export_recovery(&self) -> Result<RecoveryExportResponse, HostVaultError> {
        let key = self.current_master_key()?;
        let phrase = recovery_phrase(&key)?;
        let recovery_key = derive_domain_key(&key, b"recovery")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&recovery_key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, key.as_slice())
            .map_err(|_| HostVaultError::Crypto)?;
        let file = RecoveryFile {
            version: VAULT_VERSION,
            nonce: BASE64_STANDARD.encode(nonce.as_slice()),
            ciphertext: BASE64_STANDARD.encode(ciphertext),
        };
        let path = self.recovery_file_path();
        write_secure_file(&path, &serde_json::to_vec_pretty(&file)?)?;
        Ok(RecoveryExportResponse {
            path,
            recovery_phrase: phrase,
        })
    }

    pub fn import_recovery(&self, recovery_phrase: &str) -> Result<VaultStatus, HostVaultError> {
        validate_non_empty("recovery_phrase", recovery_phrase)?;
        let master_key = master_key_from_recovery_phrase(recovery_phrase)?;
        self.store_master_key(&master_key)?;
        self.set_unlocked(SessionKey::new(master_key))?;
        self.write_vault_check()?;
        self.status()
    }
}
```

### `backend/src/vault/secrets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/secrets.rs`
- Size bytes / Размер в байтах: `5914`
- Included characters / Включено символов: `5914`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};

use crate::platform::secrets::{
    ResolvedSecret, SecretReference, SecretResolutionError, SecretResolutionFuture, SecretResolver,
    SecretStoreKind,
};

use super::HostVault;
use super::constants::VAULT_VERSION;
use super::crypto::{entry_aad, validate_non_empty};
use super::errors::{HostVaultError, host_secret_store_failure};
use super::models::{SecretEntryContext, StoredVaultEntry};

impl HostVault {
    pub fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
        context: SecretEntryContext<'_>,
    ) -> Result<(), HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        validate_non_empty("secret value", value)?;
        validate_non_empty("entry_kind", context.entry_kind)?;
        validate_non_empty("account_id", context.account_id)?;
        validate_non_empty("purpose", context.purpose)?;

        let key = self.domain_key(b"encryption")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = entry_aad(secret_ref, context);
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: value.as_bytes(),
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        let now = Utc::now().to_rfc3339();
        let connection = self.connection()?;
        connection.execute(
            r#"
            INSERT INTO vault_entries (
                secret_ref, entry_kind, account_id, purpose, version, nonce, ciphertext, aad, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(secret_ref)
            DO UPDATE SET
                entry_kind = excluded.entry_kind,
                account_id = excluded.account_id,
                purpose = excluded.purpose,
                version = excluded.version,
                nonce = excluded.nonce,
                ciphertext = excluded.ciphertext,
                aad = excluded.aad,
                updated_at = excluded.updated_at
            "#,
            params![
                secret_ref.trim(),
                context.entry_kind.trim(),
                context.account_id.trim(),
                context.purpose.trim(),
                VAULT_VERSION,
                BASE64_STANDARD.encode(nonce.as_slice()),
                BASE64_STANDARD.encode(ciphertext),
                aad,
                now,
                now
            ],
        )?;
        self.upsert_manifest_entry(secret_ref, context)?;
        Ok(())
    }

    pub fn resolve_host_secret(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::HostVault {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }
        let value = self
            .read_secret(&reference.secret_ref)
            .map_err(host_secret_store_failure)?;
        ResolvedSecret::new(value)
    }

    pub fn read_secret(&self, secret_ref: &str) -> Result<String, HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let connection = self.connection()?;
        let row = connection
            .query_row(
                r#"
                SELECT version, nonce, ciphertext, aad
                FROM vault_entries
                WHERE secret_ref = ?1
                "#,
                params![secret_ref.trim()],
                |row| {
                    Ok(StoredVaultEntry {
                        version: row.get(0)?,
                        nonce: row.get(1)?,
                        ciphertext: row.get(2)?,
                        aad: row.get(3)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| HostVaultError::MissingSecret {
                secret_ref: secret_ref.trim().to_owned(),
            })?;
        if row.version != VAULT_VERSION {
            return Err(HostVaultError::UnsupportedVaultVersion(row.version));
        }
        let key = self.domain_key(b"encryption")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = BASE64_STANDARD
            .decode(row.nonce)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let ciphertext = BASE64_STANDARD
            .decode(row.ciphertext)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let plaintext = cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: row.aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;

        String::from_utf8(plaintext).map_err(|_| HostVaultError::InvalidEncoding)
    }

    pub fn delete_secret(&self, secret_ref: &str) -> Result<bool, HostVaultError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let deleted = self.connection()?.execute(
            r#"
            DELETE FROM vault_entries
            WHERE secret_ref = ?1
            "#,
            params![secret_ref.trim()],
        )?;
        self.delete_manifest_entry(secret_ref)?;
        Ok(deleted > 0)
    }
}

impl SecretResolver for HostVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_host_secret(reference)))
    }
}
```

### `backend/src/vault/storage.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/vault/storage.rs`
- Size bytes / Размер в байтах: `5233`
- Included characters / Включено символов: `5233`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use super::HostVault;
use super::constants::VAULT_VERSION;
use super::errors::HostVaultError;
use super::files::ensure_secure_file;
use super::models::StoredVaultEntry;

impl HostVault {
    pub(super) fn initialize_database(&self) -> Result<(), HostVaultError> {
        let connection = self.connection()?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS vault_entries (
                secret_ref TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                account_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                version INTEGER NOT NULL,
                nonce TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                aad TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS account_secret_manifest (
                secret_ref TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                account_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                secret_kind TEXT NOT NULL,
                store_kind TEXT NOT NULL,
                label TEXT NOT NULL,
                metadata TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vault_check (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version INTEGER NOT NULL,
                nonce TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                aad TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        ensure_secure_file(&self.database_path())?;
        Ok(())
    }

    pub(super) fn connection(&self) -> Result<Connection, HostVaultError> {
        Ok(Connection::open(self.database_path())?)
    }

    pub(super) fn database_path(&self) -> PathBuf {
        self.home.join("vault.db")
    }

    pub(super) fn recovery_file_path(&self) -> PathBuf {
        self.home.join("makosh-recovery.key")
    }

    pub(super) fn write_vault_check(&self) -> Result<(), HostVaultError> {
        let key = self.domain_key(b"integrity")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = format!("vault_check:v{VAULT_VERSION}");
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: b"makosh-host-vault",
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        self.connection()?.execute(
            r#"
            INSERT INTO vault_check (id, version, nonce, ciphertext, aad, updated_at)
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id)
            DO UPDATE SET
                version = excluded.version,
                nonce = excluded.nonce,
                ciphertext = excluded.ciphertext,
                aad = excluded.aad,
                updated_at = excluded.updated_at
            "#,
            params![
                VAULT_VERSION,
                BASE64_STANDARD.encode(nonce.as_slice()),
                BASE64_STANDARD.encode(ciphertext),
                aad,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub(super) fn read_vault_check(&self) -> Result<(), HostVaultError> {
        let Some(row) = self
            .connection()?
            .query_row(
                "SELECT version, nonce, ciphertext, aad FROM vault_check WHERE id = 1",
                [],
                |row| {
                    Ok(StoredVaultEntry {
                        version: row.get(0)?,
                        nonce: row.get(1)?,
                        ciphertext: row.get(2)?,
                        aad: row.get(3)?,
                    })
                },
            )
            .optional()?
        else {
            return Ok(());
        };
        if row.version != VAULT_VERSION {
            return Err(HostVaultError::UnsupportedVaultVersion(row.version));
        }
        let key = self.domain_key(b"integrity")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = BASE64_STANDARD
            .decode(row.nonce)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let ciphertext = BASE64_STANDARD
            .decode(row.ciphertext)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: row.aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        Ok(())
    }
}
```

### `backend/src/workflows/consistency_review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/consistency_review.rs`
- Size bytes / Размер в байтах: `7654`
- Included characters / Включено символов: `7654`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxError, ReviewInboxPort, ReviewItemKind,
    ReviewItemStatus,
};
use crate::engines::consistency::evidence::link_consistency_entity_in_transaction;
use crate::engines::consistency::{
    ConsistencyError, ContradictionObservation, ContradictionObservationPort,
    ContradictionReviewState,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationPortError,
};

#[derive(Clone)]
pub struct ContradictionReviewService {
    pool: PgPool,
}

impl ContradictionReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        resolution: Option<&str>,
    ) -> Result<ContradictionObservation, ContradictionReviewServiceError> {
        let review_observation = ObservationPort::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "contradiction_observation_id": observation_id,
                        "review_state": review_state.as_str(),
                        "resolution": resolution,
                        "operation": "contradiction_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("contradiction://{observation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "consistency.review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let observation = ContradictionObservationPort::new(self.pool.clone())
            .set_review_state_with_observation(
                observation_id,
                review_state,
                "makosh-frontend",
                resolution,
                Some(&review_observation.observation_id),
                None,
            )
            .await?;

        sync_contradiction_review_item(&self.pool, &observation).await?;

        Ok(observation)
    }
}

pub async fn sync_contradiction_review_item(
    pool: &PgPool,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    let mut transaction = pool.begin().await?;
    sync_contradiction_review_item_in_transaction(&mut transaction, contradiction).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn sync_contradiction_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    ensure_contradiction_review_item_in_transaction(transaction, contradiction).await?;
    sync_contradiction_review_state_in_transaction(transaction, contradiction).await?;
    Ok(())
}

async fn ensure_contradiction_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    let evidence_observation =
        capture_evidence_observation_in_transaction(transaction, contradiction).await?;
    let item = NewReviewItem::new(
        ReviewItemKind::ContradictionCandidate,
        contradiction.conflict_type.clone(),
        contradiction_summary(contradiction),
        contradiction.confidence,
    )
    .metadata(json!({
        "mirrored_from": "contradictions",
        "contradiction_observation_id": contradiction.observation_id,
        "severity": contradiction.severity.as_str(),
        "old_source_kind": contradiction.old_source_kind.as_str(),
        "old_source_id": contradiction.old_source_id,
        "new_source_kind": contradiction.new_source_kind.as_str(),
        "new_source_id": contradiction.new_source_id,
    }));
    let evidence = NewReviewItemEvidence::new(evidence_observation.observation_id)
        .role("primary")
        .metadata(json!({
            "mirrored_from": "contradictions",
            "contradiction_observation_id": contradiction.observation_id,
        }));
    let _ = ReviewInboxPort::create_with_evidence_in_transaction(transaction, &item, &[evidence])
        .await?;
    Ok(())
}

async fn capture_evidence_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<crate::platform::observations::Observation, ObservationPortError> {
    ObservationPort::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "CONTRADICTION_OBSERVATION",
            ObservationOriginKind::LocalRuntime,
            contradiction.created_at,
            json!({
                "contradiction_observation_id": contradiction.observation_id,
                "conflict_type": contradiction.conflict_type,
                "old_claim": contradiction.old_claim,
                "new_claim": contradiction.new_claim,
                "severity": contradiction.severity.as_str(),
                "review_state": contradiction.review_state.as_str(),
                "affected_entities": contradiction.affected_entities,
            }),
            format!("contradiction://{}", contradiction.observation_id),
        )
        .confidence(contradiction.confidence)
        .provenance(json!({
            "engine": "consistency",
            "pipeline": "contradiction_observations",
        })),
    )
    .await
}

async fn sync_contradiction_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    let review_item = ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::ContradictionCandidate,
        &json!({
            "contradiction_observation_id": contradiction.observation_id,
        }),
    )
    .await?
    .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(contradiction.observation_id.clone()))?;

    let status = match contradiction.review_state {
        ContradictionReviewState::Suggested => ReviewItemStatus::New,
        ContradictionReviewState::UserConfirmed => ReviewItemStatus::Approved,
        ContradictionReviewState::UserRejected => ReviewItemStatus::Dismissed,
    };

    let _ = ReviewInboxPort::transition_status_in_transaction(
        transaction,
        &review_item.review_item_id,
        status,
    )
    .await?;
    Ok(())
}

fn contradiction_summary(contradiction: &ContradictionObservation) -> String {
    format!(
        "{} -> {}",
        contradiction.old_claim.trim(),
        contradiction.new_claim.trim()
    )
}

#[derive(Debug, Error)]
pub enum ContradictionReviewWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Consistency(#[from] ConsistencyError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Observation(#[from] ObservationPortError),
}

#[derive(Debug, Error)]
pub enum ContradictionReviewServiceError {
    #[error(transparent)]
    Consistency(#[from] ConsistencyError),

    #[error(transparent)]
    Observation(#[from] ObservationPortError),

    #[error(transparent)]
    ReviewWorkflow(#[from] ContradictionReviewWorkflowError),
}
```

### `backend/src/workflows/email_fixture_pipeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_fixture_pipeline.rs`
- Size bytes / Размер в байтах: `7743`
- Included characters / Включено символов: `7743`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::core::CommunicationProviderAccountPort;
use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionPort, EmailProviderKind, NewProviderAccount,
};
use crate::domains::communications::import::{
    FixtureEmailImportError, FixtureEmailImportRequest, import_fixture_email_messages_with_records,
};
use crate::domains::communications::messages::{
    CommunicationSignalProjectionError, MessageProjectionError,
    project_accepted_signal_if_runtime_allows,
};
use crate::domains::graph::core::{GraphProjectionPort, GraphProjectionPortError, GraphSummary};
use crate::domains::persons::api::{
    PersonProjectionError, PersonProjectionPort, upsert_persons_from_message_participants,
};
use crate::domains::signal_hub::{SignalHubError, dispatch_mail_raw_signal};
use crate::workflows::graph_projection::{
    GraphProjectionError, GraphProjectionReport, GraphProjectionService,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailFixturePipelineRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub provider_kind: EmailProviderKind,
    pub import_batch_id: String,
    pub fixture_json: String,
}

impl EmailFixturePipelineRequest {
    pub fn new(
        account_id: impl Into<String>,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        provider_kind: EmailProviderKind,
        import_batch_id: impl Into<String>,
        fixture_json: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            provider_kind,
            import_batch_id: import_batch_id.into(),
            fixture_json: fixture_json.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailFixtureImportPipelineReport {
    pub account_id: String,
    pub import_batch_id: String,
    pub provider_kind: EmailProviderKind,
    pub imported_records: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EmailFixtureProjectionPipelineReport {
    pub account_id: String,
    pub import_batch_id: String,
    pub provider_kind: EmailProviderKind,
    pub imported_records: usize,
    pub projected_messages: usize,
    pub upserted_persons: usize,
    pub graph_projection: GraphProjectionReport,
    pub graph_summary: GraphSummary,
    pub total_graph_nodes: i64,
    pub total_graph_edges: i64,
}

pub async fn import_fixture_email_messages_for_dev(
    pool: PgPool,
    request: &EmailFixturePipelineRequest,
) -> Result<EmailFixtureImportPipelineReport, EmailFixturePipelineError> {
    let communication_store = CommunicationIngestionPort::new(pool.clone());
    upsert_fixture_provider_account(&pool, request).await?;
    let import_report = import_fixture_email_messages_with_records(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &request.account_id,
            &request.import_batch_id,
            &request.fixture_json,
        ),
    )
    .await?;

    Ok(EmailFixtureImportPipelineReport {
        account_id: request.account_id.clone(),
        import_batch_id: request.import_batch_id.clone(),
        provider_kind: request.provider_kind,
        imported_records: import_report.inserted_or_existing_records,
    })
}

pub async fn project_fixture_email_messages(
    pool: PgPool,
    request: &EmailFixturePipelineRequest,
) -> Result<EmailFixtureProjectionPipelineReport, EmailFixturePipelineError> {
    let communication_store = CommunicationIngestionPort::new(pool.clone());
    upsert_fixture_provider_account(&pool, request).await?;
    let import_report = import_fixture_email_messages_with_records(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &request.account_id,
            &request.import_batch_id,
            &request.fixture_json,
        ),
    )
    .await?;

    let person_store = PersonProjectionPort::new(pool.clone());
    let mut projected_messages = 0;
    let mut participants = Vec::new();
    for raw_record in &import_report.raw_records {
        let Some(accepted_event) = dispatch_mail_raw_signal(pool.clone(), raw_record, None).await?
        else {
            continue;
        };
        let Some(message) =
            project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event).await?
        else {
            continue;
        };
        participants.push(message.sender.clone());
        participants.extend(message.recipients.clone());
        projected_messages += 1;
    }
    let persons = upsert_persons_from_message_participants(&person_store, &participants).await?;

    let graph_projection = GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await?;
    let graph_summary = GraphProjectionPort::new(pool).summary().await?;
    let total_graph_nodes = graph_summary
        .node_counts
        .iter()
        .map(|count| count.count)
        .sum();
    let total_graph_edges = graph_summary
        .edge_counts
        .iter()
        .map(|count| count.count)
        .sum();

    Ok(EmailFixtureProjectionPipelineReport {
        account_id: request.account_id.clone(),
        import_batch_id: request.import_batch_id.clone(),
        provider_kind: request.provider_kind,
        imported_records: import_report.inserted_or_existing_records,
        projected_messages,
        upserted_persons: persons.len(),
        graph_projection,
        graph_summary,
        total_graph_nodes,
        total_graph_edges,
    })
}

async fn upsert_fixture_provider_account(
    pool: &PgPool,
    request: &EmailFixturePipelineRequest,
) -> Result<(), CommunicationIngestionError> {
    let account = NewProviderAccount::new(
        &request.account_id,
        request.provider_kind,
        &request.display_name,
        &request.external_account_id,
    )
    .config(provider_config(request.provider_kind));
    CommunicationProviderAccountPort::new(pool.clone())
        .upsert(&account)
        .await?;
    Ok(())
}

fn provider_config(provider_kind: EmailProviderKind) -> serde_json::Value {
    match provider_kind {
        EmailProviderKind::Gmail => json!({"history_stream_id": "gmail:fixture"}),
        EmailProviderKind::Icloud => {
            json!({"host": "imap.mail.me.com", "port": 993, "tls": true, "mailbox": "INBOX"})
        }
        EmailProviderKind::Imap => {
            json!({"host": "localhost", "port": 993, "tls": true, "mailbox": "INBOX"})
        }
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb
        | EmailProviderKind::WhatsappBusinessCloud
        | EmailProviderKind::ZoomUser
        | EmailProviderKind::ZoomServerToServer
        | EmailProviderKind::YandexTelemostUser => json!({}),
    }
}

#[derive(Debug, Error)]
pub enum EmailFixturePipelineError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Import(#[from] FixtureEmailImportError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),

    #[error(transparent)]
    Contact(#[from] PersonProjectionError),

    #[error(transparent)]
    GraphProjection(#[from] GraphProjectionError),

    #[error(transparent)]
    GraphProjectionPort(#[from] GraphProjectionPortError),
}
```

### `backend/src/workflows/email_intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence.rs`
- Size bytes / Размер в байтах: `305`
- Included characters / Включено символов: `305`
- Truncated / Обрезано: `no`

```rust
mod categories;
mod errors;
mod heuristics;
mod models;
mod prompt;
mod service;

#[cfg(test)]
mod tests;

pub use categories::EmailCategory;
pub use errors::EmailIntelligenceError;
pub use models::{EmailAnalysis, EmailKnowledgeCandidate, EmailSummaryContract};
pub use service::EmailIntelligenceService;
```

### `backend/src/workflows/email_intelligence/categories.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/categories.rs`
- Size bytes / Размер в байтах: `1935`
- Included characters / Включено символов: `1935`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EmailCategory {
    Critical,
    Important,
    Personal,
    Work,
    Finance,
    Legal,
    Notification,
    Newsletter,
    Marketing,
    Spam,
    Scam,
    Phishing,
    Suspicious,
}

impl EmailCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailCategory::Critical => "critical",
            EmailCategory::Important => "important",
            EmailCategory::Personal => "personal",
            EmailCategory::Work => "work",
            EmailCategory::Finance => "finance",
            EmailCategory::Legal => "legal",
            EmailCategory::Notification => "notification",
            EmailCategory::Newsletter => "newsletter",
            EmailCategory::Marketing => "marketing",
            EmailCategory::Spam => "spam",
            EmailCategory::Scam => "scam",
            EmailCategory::Phishing => "phishing",
            EmailCategory::Suspicious => "suspicious",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "critical" => Some(EmailCategory::Critical),
            "important" => Some(EmailCategory::Important),
            "personal" => Some(EmailCategory::Personal),
            "work" => Some(EmailCategory::Work),
            "finance" => Some(EmailCategory::Finance),
            "legal" => Some(EmailCategory::Legal),
            "notification" => Some(EmailCategory::Notification),
            "newsletter" => Some(EmailCategory::Newsletter),
            "marketing" => Some(EmailCategory::Marketing),
            "spam" => Some(EmailCategory::Spam),
            "scam" => Some(EmailCategory::Scam),
            "phishing" => Some(EmailCategory::Phishing),
            "suspicious" => Some(EmailCategory::Suspicious),
            _ => None,
        }
    }
}
```

### `backend/src/workflows/email_intelligence/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/errors.rs`
- Size bytes / Размер в байтах: `430`
- Included characters / Включено символов: `430`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::communications::messages::MessageProjectionError;
use crate::platform::ai_runtime::AiRuntimePortError;

#[derive(Debug, Error)]
pub enum EmailIntelligenceError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error("failed to parse AI response: {0}")]
    ParseError(String),
}
```

### `backend/src/workflows/email_intelligence/heuristics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/heuristics.rs`
- Size bytes / Размер в байтах: `10417`
- Included characters / Включено символов: `10417`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::ProjectedMessage;
use crate::workflows::email_intelligence::models::{EmailKnowledgeCandidate, EmailSummaryContract};

const URGENT_WORDS: &[&str] = &[
    "urgent",
    "asap",
    "deadline",
    "immediately",
    "critical",
    "action required",
];
const FINANCE_WORDS: &[&str] = &[
    "invoice",
    "payment",
    "factura",
    "bill",
    "amount due",
    "receipt",
    "tax",
];
const LEGAL_WORDS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "legal",
    "liability",
    "confidential",
    "attorney",
];
const ATTACHMENT_WORDS: &[&str] = &["attached", "attachment", "see attached", "please find"];
const JUNK_WORDS: &[&str] = &[
    "unsubscribe",
    "opt out",
    "this email was sent",
    "if you no longer wish",
];
const ACTION_WORDS: &[&str] = &[
    "action required",
    "please",
    "review",
    "respond",
    "reply",
    "confirm",
    "send",
    "approve",
    "sign",
];
const RISK_WORDS: &[&str] = &[
    "risk",
    "blocked",
    "blocker",
    "issue",
    "problem",
    "phishing",
    "scam",
    "verify your account",
    "click here",
];
const DEADLINE_WORDS: &[&str] = &[
    "deadline",
    "due",
    "by ",
    "before",
    "today",
    "tomorrow",
    "eod",
    "friday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
];
const EVENT_WORDS: &[&str] = &[
    "meeting",
    "call",
    "demo",
    "appointment",
    "interview",
    "workshop",
    "webinar",
];
const DOCUMENT_WORDS: &[&str] = &[
    "attachment",
    "attached",
    "document",
    "file",
    "pdf",
    "invoice",
    "receipt",
    "msa",
    "sow",
];
const AGREEMENT_WORDS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "msa",
    "sow",
    "terms",
    "liability",
    "confidential",
];

pub(super) fn heuristic_score(message: &ProjectedMessage) -> i16 {
    let mut score: i16 = 30;
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if contains_any(&subject_lower, URGENT_WORDS) {
        score = score.saturating_add(15);
    }
    if contains_any(&body_lower, FINANCE_WORDS) || contains_any(&subject_lower, FINANCE_WORDS) {
        score = score.saturating_add(20);
    }
    if contains_any(&body_lower, LEGAL_WORDS) || contains_any(&subject_lower, LEGAL_WORDS) {
        score = score.saturating_add(25);
    }

    score_question_sign(&mut score, &body_lower);
    score_attachment_language(&mut score, &body_lower);
    score_junk_language(&mut score, &body_lower);

    if message.body_text.len() < 50 {
        score = score.saturating_sub(10);
    }

    score.clamp(0, 100)
}

pub(super) fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if body_lower.contains("invoice")
        || body_lower.contains("factura")
        || body_lower.contains("payment")
    {
        return Some("finance".to_owned());
    }
    if body_lower.contains("contract")
        || body_lower.contains("nda")
        || body_lower.contains("agreement")
    {
        return Some("legal".to_owned());
    }
    if body_lower.contains("unsubscribe") || body_lower.contains("newsletter") {
        return Some("marketing".to_owned());
    }
    if subject_lower.contains("notification") || body_lower.contains("notification") {
        return Some("notification".to_owned());
    }
    if body_lower.contains("click here")
        && (body_lower.contains("account") || body_lower.contains("verify"))
    {
        return Some("suspicious".to_owned());
    }

    None
}

pub(super) fn structured_summary(message: &ProjectedMessage) -> EmailSummaryContract {
    let mut key_points = Vec::new();
    push_unique_bounded(&mut key_points, cleaned_phrase(&message.subject), 5);

    let phrases = message_phrases(message);
    for phrase in &phrases {
        if key_points.len() >= 5 {
            break;
        }
        let lower = phrase.to_lowercase();
        if !contains_any(&lower, ACTION_WORDS) {
            push_unique_bounded(&mut key_points, Some(phrase.clone()), 5);
        }
    }

    let mut action_items = Vec::new();
    let mut risks = Vec::new();
    let mut deadlines = Vec::new();

    for phrase in phrases {
        let lower = phrase.to_lowercase();
        if contains_any(&lower, ACTION_WORDS) {
            push_unique_bounded(&mut action_items, Some(phrase.clone()), 5);
        }
        if contains_any(&lower, RISK_WORDS) {
            push_unique_bounded(&mut risks, Some(phrase.clone()), 5);
        }
        if contains_any(&lower, DEADLINE_WORDS) {
            push_unique_bounded(&mut deadlines, Some(phrase), 5);
        }
    }

    let candidate_phrases = phrases_for_candidates(message);
    EmailSummaryContract {
        key_points,
        action_items,
        risks,
        deadlines,
        event_candidates: knowledge_candidates(&candidate_phrases, EVENT_WORDS, 5),
        persona_candidates: persona_candidates(message),
        organization_candidates: organization_candidates(message),
        document_candidates: knowledge_candidates(&candidate_phrases, DOCUMENT_WORDS, 5),
        agreement_candidates: knowledge_candidates(&candidate_phrases, AGREEMENT_WORDS, 5),
    }
}

fn phrases_for_candidates(message: &ProjectedMessage) -> Vec<String> {
    let mut phrases = Vec::new();
    push_unique_bounded(&mut phrases, cleaned_phrase(&message.subject), 30);
    for phrase in message_phrases(message) {
        push_unique_bounded(&mut phrases, Some(phrase), 30);
    }
    phrases
}

fn knowledge_candidates(
    phrases: &[String],
    words: &[&str],
    limit: usize,
) -> Vec<EmailKnowledgeCandidate> {
    let mut candidates = Vec::new();
    for phrase in phrases {
        let lower = phrase.to_lowercase();
        if contains_any(&lower, words) {
            push_candidate_bounded(&mut candidates, phrase.clone(), phrase.clone(), limit);
        }
    }
    candidates
}

fn persona_candidates(message: &ProjectedMessage) -> Vec<EmailKnowledgeCandidate> {
    let mut candidates = Vec::new();
    push_persona_candidate(
        &mut candidates,
        message.sender_display_name.as_deref(),
        &message.sender,
    );
    push_persona_candidate(
        &mut candidates,
        Some(message.sender.as_str()),
        &message.sender,
    );
    for line in message.body_text.lines().take(20) {
        let trimmed = line.trim();
        if let Some((label, email)) = email_identity(trimmed) {
            push_candidate_bounded(&mut candidates, label, email, 5);
        }
    }
    candidates
}

fn organization_candidates(message: &ProjectedMessage) -> Vec<EmailKnowledgeCandidate> {
    let mut candidates = Vec::new();
    let mut values: Vec<&str> = Vec::with_capacity(message.recipients.len() + 16);
    values.push(&message.sender);
    values.extend(message.recipients.iter().map(String::as_str));
    values.extend(message.body_text.split_whitespace().take(80));
    for value in values {
        if let Some(domain) = email_domain(value) {
            push_candidate_bounded(&mut candidates, domain.clone(), value.to_owned(), 5);
        }
    }
    candidates
}

fn score_question_sign(score: &mut i16, body_lower: &str) {
    if body_lower.contains('?') {
        *score = score.saturating_add(10);
    }
}

fn score_attachment_language(score: &mut i16, body_lower: &str) {
    if contains_any(body_lower, ATTACHMENT_WORDS) {
        *score = score.saturating_add(10);
    }
}

fn score_junk_language(score: &mut i16, body_lower: &str) {
    if contains_any(body_lower, JUNK_WORDS) {
        *score = score.saturating_sub(20);
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn message_phrases(message: &ProjectedMessage) -> Vec<String> {
    message
        .body_text
        .split(['.', '\n', '!', '?'])
        .filter_map(cleaned_phrase)
        .take(30)
        .collect()
}

fn cleaned_phrase(value: &str) -> Option<String> {
    let phrase = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_matches(|c: char| matches!(c, '"' | '\'' | ':' | ';' | ',' | '-' | ' '))
        .to_owned();
    (!phrase.is_empty()).then_some(phrase)
}

fn push_unique_bounded(target: &mut Vec<String>, value: Option<String>, limit: usize) {
    let Some(value) = value else {
        return;
    };
    if target.len() >= limit || target.iter().any(|existing| existing == &value) {
        return;
    }
    target.push(value);
}

fn push_persona_candidate(
    target: &mut Vec<EmailKnowledgeCandidate>,
    label: Option<&str>,
    evidence: &str,
) {
    let Some(label) = label.and_then(cleaned_phrase) else {
        return;
    };
    if label.contains('@') && label.len() > 120 {
        return;
    }
    push_candidate_bounded(target, label, evidence.to_owned(), 5);
}

fn email_identity(value: &str) -> Option<(String, String)> {
    let email = value
        .split_whitespace()
        .find(|part| part.contains('@'))
        .map(|part| {
            part.trim_matches(|c| matches!(c, '<' | '>' | ',' | ';'))
                .to_owned()
        })?;
    let label = value
        .split('<')
        .next()
        .and_then(cleaned_phrase)
        .filter(|name| !name.eq_ignore_ascii_case("from"))
        .unwrap_or_else(|| email.clone());
    Some((label, email))
}

fn email_domain(value: &str) -> Option<String> {
    let email = value
        .trim_matches(|c| matches!(c, '<' | '>' | ',' | ';' | ')' | '(' | '"' | '\''))
        .split('@')
        .nth(1)?
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-')
        .to_lowercase();
    if email.is_empty() || email.ends_with(".local") {
        None
    } else {
        Some(email)
    }
}

fn push_candidate_bounded(
    target: &mut Vec<EmailKnowledgeCandidate>,
    title: String,
    evidence: String,
    limit: usize,
) {
    let title = title.trim().to_owned();
    let evidence = evidence.trim().to_owned();
    if title.is_empty()
        || target.len() >= limit
        || target
            .iter()
            .any(|candidate| candidate.title.eq_ignore_ascii_case(&title))
    {
        return;
    }
    target.push(EmailKnowledgeCandidate { title, evidence });
}
```

### `backend/src/workflows/email_intelligence/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/models.rs`
- Size bytes / Размер в байтах: `1839`
- Included characters / Включено символов: `1839`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailAnalysis {
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    #[serde(default)]
    pub action_items: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub deadlines: Vec<String>,
    #[serde(default)]
    pub event_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub persona_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub organization_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub document_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub agreement_candidates: Vec<EmailKnowledgeCandidate>,
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub suggested_action: Option<String>,
    pub extracted_deadline: Option<String>,
    pub language: Option<String>,
    pub model: String,
    pub prompt_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailKnowledgeCandidate {
    pub title: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailSummaryContract {
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub risks: Vec<String>,
    pub deadlines: Vec<String>,
    #[serde(default)]
    pub event_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub persona_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub organization_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub document_candidates: Vec<EmailKnowledgeCandidate>,
    #[serde(default)]
    pub agreement_candidates: Vec<EmailKnowledgeCandidate>,
}
```

### `backend/src/workflows/email_intelligence/prompt.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/prompt.rs`
- Size bytes / Размер в байтах: `1941`
- Included characters / Включено символов: `1941`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::ProjectedMessage;

pub(super) const EMAIL_INTELLIGENCE_PROMPT_VERSION: &str =
    "v3-email-intelligence-mail-knowledge-candidates-2026-06-15";

pub(super) fn build_email_analysis_prompt(message: &ProjectedMessage) -> String {
    let body = if message.body_text.len() <= 2000 {
        &message.body_text
    } else {
        &message.body_text[..2000.min(message.body_text.len())]
    };

    format!(
        "You are an email intelligence assistant. Analyze this email and respond with a JSON object containing:\n\
- category: one of [critical, important, personal, work, finance, legal, notification, newsletter, marketing, spam, scam, phishing, suspicious]\n\
- summary: 1-2 sentence TL;DR\n\
- key_points: array of up to 5 short evidence-backed key points\n\
- action_items: array of up to 5 requested or implied actions\n\
- risks: array of up to 5 risks, blockers, scams, suspicious details or delivery concerns\n\
- deadlines: array of up to 5 deadlines or time constraints\n\
- event_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- persona_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- organization_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- document_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- agreement_candidates: array of up to 5 mail-derived candidate objects with title and evidence\n\
- importance_score: integer 0-100\n\
- is_spam: boolean\n\
- is_phishing: boolean\n\
- suggested_action: what the recipient should do, or null\n\
- extracted_deadline: any deadline mentioned, or null\n\
- language: the language code (e.g., \"en\", \"es\", \"ru\"), or null\n\
\n\
From: {}\n\
Subject: {}\n\
Body:\n\
{}\n\
\n\
Respond with ONLY the JSON object.",
        message.sender, message.subject, body
    )
}
```

### `backend/src/workflows/email_intelligence/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/service.rs`
- Size bytes / Размер в байтах: `5108`
- Included characters / Включено символов: `5108`
- Truncated / Обрезано: `no`

````rust
use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, ProjectedMessage, WorkflowState,
};
use crate::platform::ai_runtime::SharedAiRuntimePort;
use crate::workflows::email_intelligence::errors::EmailIntelligenceError;
use crate::workflows::email_intelligence::heuristics;
use crate::workflows::email_intelligence::models::{EmailAnalysis, EmailSummaryContract};
use crate::workflows::email_intelligence::prompt::{
    EMAIL_INTELLIGENCE_PROMPT_VERSION, build_email_analysis_prompt,
};

#[derive(Clone)]
pub struct EmailIntelligenceService {
    runtime: Option<SharedAiRuntimePort>,
}

impl EmailIntelligenceService {
    pub fn new(runtime: Option<SharedAiRuntimePort>) -> Self {
        Self { runtime }
    }

    pub async fn analyze_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Option<EmailAnalysis>, EmailIntelligenceError> {
        let Some(ref runtime) = self.runtime else {
            return Ok(None);
        };

        let prompt = build_email_analysis_prompt(message);
        let result = runtime.chat(&prompt).await?;
        let mut analysis: EmailAnalysis =
            serde_json::from_str(clean_json_response(&result.content))
                .map_err(|e| EmailIntelligenceError::ParseError(e.to_string()))?;

        analysis.model = result.model;
        analysis.prompt_version = EMAIL_INTELLIGENCE_PROMPT_VERSION.to_owned();

        Ok(Some(analysis))
    }

    pub async fn analyze_and_persist(
        &self,
        store: &CommunicationMessageProjectionPort,
        message: &ProjectedMessage,
    ) -> Result<bool, EmailIntelligenceError> {
        let Some(analysis) = self.analyze_message(message).await? else {
            return Ok(false);
        };

        let workflow_hint = if analysis.is_spam || analysis.is_phishing {
            Some(WorkflowState::Spam)
        } else if analysis.importance_score >= 80 {
            Some(WorkflowState::NeedsAction)
        } else {
            None
        };

        store
            .set_ai_analysis(
                &message.message_id,
                Some(&analysis.category),
                Some(&analysis.summary),
                Some(analysis.importance_score),
            )
            .await?;
        let summary_contract = analysis_summary_contract(&analysis, message);
        let mut metadata = message.message_metadata.clone();
        metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract)
            .map_err(|error| EmailIntelligenceError::ParseError(error.to_string()))?;
        store
            .set_message_metadata(&message.message_id, &metadata)
            .await?;

        if let Some(state) = workflow_hint {
            let _ = store
                .transition_workflow_state(&message.message_id, state)
                .await;
        }

        Ok(true)
    }

    pub fn heuristic_score(message: &ProjectedMessage) -> i16 {
        heuristics::heuristic_score(message)
    }

    pub fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
        heuristics::heuristic_category(message)
    }

    pub fn heuristic_structured_summary(message: &ProjectedMessage) -> EmailSummaryContract {
        heuristics::structured_summary(message)
    }
}

fn clean_json_response(content: &str) -> &str {
    content
        .trim()
        .strip_prefix("```json")
        .and_then(|value| value.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(content.trim())
}

fn analysis_summary_contract(
    analysis: &EmailAnalysis,
    message: &ProjectedMessage,
) -> EmailSummaryContract {
    let fallback = EmailIntelligenceService::heuristic_structured_summary(message);
    EmailSummaryContract {
        key_points: non_empty_or(analysis.key_points.clone(), fallback.key_points),
        action_items: non_empty_or(analysis.action_items.clone(), fallback.action_items),
        risks: non_empty_or(analysis.risks.clone(), fallback.risks),
        deadlines: non_empty_or(analysis.deadlines.clone(), fallback.deadlines),
        event_candidates: non_empty_candidates_or(
            analysis.event_candidates.clone(),
            fallback.event_candidates,
        ),
        persona_candidates: non_empty_candidates_or(
            analysis.persona_candidates.clone(),
            fallback.persona_candidates,
        ),
        organization_candidates: non_empty_candidates_or(
            analysis.organization_candidates.clone(),
            fallback.organization_candidates,
        ),
        document_candidates: non_empty_candidates_or(
            analysis.document_candidates.clone(),
            fallback.document_candidates,
        ),
        agreement_candidates: non_empty_candidates_or(
            analysis.agreement_candidates.clone(),
            fallback.agreement_candidates,
        ),
    }
}

fn non_empty_or(values: Vec<String>, fallback: Vec<String>) -> Vec<String> {
    if values.is_empty() { fallback } else { values }
}

fn non_empty_candidates_or<T>(values: Vec<T>, fallback: Vec<T>) -> Vec<T> {
    if values.is_empty() { fallback } else { values }
}
````
