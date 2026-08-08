use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use postgres::{Config, IsolationLevel, NoTls};
use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::catalog::{validate_catalog, validate_google_oauth, validate_legacy_configuration};
use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};
use crate::legacy_configuration::{
    LegacyDatabaseSourceConfigurationV1, parse_database_configuration, parse_provider_configuration,
};
use crate::model::{
    GoogleOauthClientV1, LegacyProviderCandidateKindV1, LegacyProviderConfigurationV1,
    LegacyProviderRecoveryCountsV1, RECOVERY_SCHEMA_REVISION, RecoveryBundleFileV1,
    RecoveryBundleManifestV1, RecoveryCatalogCandidateV1, RecoveryCatalogConfigurationV1,
    RecoveryCatalogSecretBindingV1, RecoveryCatalogV1,
};
use crate::private_files::{
    CATALOG_FILE, GOOGLE_OAUTH_FILE, MANIFEST_DATA_FILES, MANIFEST_FILE,
    PROVIDER_CONFIGURATION_FILE, VAULT_FILE, set_private_permissions, sha256_hex,
};

const MAX_SOURCE_FILE_BYTES: u64 = 1024 * 1024;
const EXPECTED_CATALOG_ROWS: u16 = 5;
const PROVIDER_KINDS: [&str; 4] = ["gmail", "icloud", "telegram_bot", "telegram_user"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyProviderRecoveryPreparationInputV1 {
    pub database_host: IpAddr,
    pub database_port: u16,
    pub database_environment_file: PathBuf,
    pub provider_environment_file: PathBuf,
    pub legacy_vault_root: PathBuf,
    pub legacy_vault_master_key_file: PathBuf,
    pub output_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyProviderRecoveryPreparationReceiptV1 {
    pub schema_revision: u16,
    pub bundle_fingerprint_sha256: String,
    pub counts: LegacyProviderRecoveryCountsV1,
}

pub fn prepare_bundle(
    input: &LegacyProviderRecoveryPreparationInputV1,
) -> LegacyProviderRecoveryResultV1<LegacyProviderRecoveryPreparationReceiptV1> {
    validate_input(input)?;
    let database_environment = read_source_file(&input.database_environment_file)?;
    let provider_environment = read_source_file(&input.provider_environment_file)?;
    let legacy_master_key_bytes =
        Zeroizing::new(read_source_file(&input.legacy_vault_master_key_file)?);
    let legacy_master_key = crate::legacy_vault::decode_master_key_file(&legacy_master_key_bytes)?;
    let database = parse_database_configuration(&database_environment)?;
    let provider = parse_provider_configuration(&provider_environment)?;
    let google = read_google_oauth_client(&provider.google_oauth_client_path)?;
    validate_google_oauth(&google)?;
    let catalog = read_catalog(input, &database)?;
    validate_catalog(&catalog)?;
    let provider_configuration = LegacyProviderConfigurationV1 {
        schema_revision: RECOVERY_SCHEMA_REVISION,
        telegram_api_id: provider.telegram_api_id,
        telegram_api_hash: provider.telegram_api_hash,
    };
    validate_legacy_configuration(&provider_configuration)?;
    validate_cross_source_configuration(&catalog, &google)?;

    let mut output = PrivateOutputDirectoryV1::create(&input.output_root)?;
    output.write_json(CATALOG_FILE, &catalog)?;
    output.write_json(PROVIDER_CONFIGURATION_FILE, &provider_configuration)?;
    output.write_json(GOOGLE_OAUTH_FILE, &google)?;
    let mut normalized_master_key = Zeroizing::new(legacy_master_key_bytes.trim_ascii().to_vec());
    normalized_master_key.push(b'\n');
    output.write_bytes(
        crate::private_files::VAULT_MASTER_KEY_FILE,
        &normalized_master_key,
    )?;
    snapshot_vault(
        &input.legacy_vault_root.join(VAULT_FILE),
        &output.path().join(VAULT_FILE),
    )?;
    set_private_permissions(&output.path().join(VAULT_FILE), 0o600);
    validate_vault_bindings(&output.path().join(VAULT_FILE), &catalog)?;
    validate_icloud_decryption(
        &output.path().join(VAULT_FILE),
        &catalog,
        &legacy_master_key,
    )?;

    let manifest = build_manifest(output.path(), &catalog)?;
    let manifest_bytes = private_json(&manifest)?;
    output.write_bytes(MANIFEST_FILE, &manifest_bytes)?;
    let fingerprint = sha256_hex(&manifest_bytes);
    output.commit()?;
    Ok(LegacyProviderRecoveryPreparationReceiptV1 {
        schema_revision: RECOVERY_SCHEMA_REVISION,
        bundle_fingerprint_sha256: fingerprint,
        counts: catalog.counts,
    })
}

fn validate_input(
    input: &LegacyProviderRecoveryPreparationInputV1,
) -> LegacyProviderRecoveryResultV1<()> {
    if input.database_host != IpAddr::V4(Ipv4Addr::LOCALHOST)
        || input.database_port == 0
        || !input.database_environment_file.is_absolute()
        || !input.provider_environment_file.is_absolute()
        || !input.legacy_vault_root.is_absolute()
        || !input.legacy_vault_master_key_file.is_absolute()
        || !input.output_root.is_absolute()
        || input.output_root.exists()
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidArguments);
    }
    Ok(())
}

fn validate_icloud_decryption(
    vault_path: &Path,
    catalog: &RecoveryCatalogV1,
    master_key: &[u8; 32],
) -> LegacyProviderRecoveryResultV1<()> {
    let candidate = catalog
        .candidates
        .iter()
        .find(|candidate| candidate.kind == LegacyProviderCandidateKindV1::Icloud)
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?;
    let secret = crate::legacy_vault::decrypt_candidate_secret(vault_path, candidate, master_key)?;
    if secret.is_empty() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
    }
    Ok(())
}

fn read_catalog(
    input: &LegacyProviderRecoveryPreparationInputV1,
    database: &LegacyDatabaseSourceConfigurationV1,
) -> LegacyProviderRecoveryResultV1<RecoveryCatalogV1> {
    let mut configuration = Config::new();
    configuration
        .host(&input.database_host.to_string())
        .port(input.database_port)
        .dbname(&database.database)
        .user(&database.username)
        .password(database.password.as_bytes());
    let mut client = configuration
        .connect(NoTls)
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    let mut transaction = client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    let account_rows = transaction
        .query(
            r#"
            SELECT account_id, provider_kind, display_name, external_account_id, config
            FROM communication_provider_accounts
            WHERE provider_kind = ANY($1)
            ORDER BY provider_kind, account_id
            "#,
            &[&PROVIDER_KINDS.as_slice()],
        )
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    let secret_rows = transaction
        .query(
            r#"
            SELECT binding.account_id, binding.secret_purpose, binding.secret_ref,
                   secret.secret_kind, secret.store_kind
            FROM communication_provider_account_secret_refs binding
            JOIN secret_references secret ON secret.secret_ref = binding.secret_ref
            JOIN communication_provider_accounts account ON account.account_id = binding.account_id
            WHERE account.provider_kind = ANY($1)
            ORDER BY binding.account_id, binding.secret_purpose
            "#,
            &[&PROVIDER_KINDS.as_slice()],
        )
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    transaction
        .commit()
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;

    let bindings = secret_rows
        .into_iter()
        .map(|row| {
            let account_id: String = row.get(0);
            let binding = RecoveryCatalogSecretBindingV1 {
                purpose: row.get(1),
                secret_ref: row.get(2),
                secret_kind: row.get(3),
                store_kind: row.get(4),
            };
            (account_id, binding)
        })
        .collect::<Vec<_>>();
    assemble_catalog(account_rows, bindings)
}

fn assemble_catalog(
    rows: Vec<postgres::Row>,
    bindings: Vec<(String, RecoveryCatalogSecretBindingV1)>,
) -> LegacyProviderRecoveryResultV1<RecoveryCatalogV1> {
    let mut by_account = BTreeMap::<String, Vec<RecoveryCatalogSecretBindingV1>>::new();
    for (account_id, binding) in bindings {
        by_account.entry(account_id).or_default().push(binding);
    }
    let source_generation = random_hex(16)?;
    let mut candidates = Vec::new();
    let mut counts = LegacyProviderRecoveryCountsV1 {
        gmail_active: 0,
        icloud_active: 0,
        telegram_user_active: 0,
        gmail_deleted: 0,
    };
    for row in rows {
        let account_id: String = row.get(0);
        let provider_kind: String = row.get(1);
        let display_name: String = row.get(2);
        let external_account_id: String = row.get(3);
        let configuration: Value = row.get(4);
        let deleted = configuration
            .as_object()
            .is_some_and(|object| object.contains_key("deleted_at"));
        if deleted {
            if provider_kind != "gmail"
                || by_account
                    .remove(&account_id)
                    .is_some_and(|bindings| !bindings.is_empty())
            {
                return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
            }
            counts.gmail_deleted = counts.gmail_deleted.saturating_add(1);
            continue;
        }
        let kind = match provider_kind.as_str() {
            "gmail" => {
                counts.gmail_active = counts.gmail_active.saturating_add(1);
                LegacyProviderCandidateKindV1::Gmail
            }
            "icloud" => {
                counts.icloud_active = counts.icloud_active.saturating_add(1);
                LegacyProviderCandidateKindV1::Icloud
            }
            "telegram_user" => {
                counts.telegram_user_active = counts.telegram_user_active.saturating_add(1);
                LegacyProviderCandidateKindV1::TelegramUser
            }
            "telegram_bot" => return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog),
            _ => return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog),
        };
        let binding = exactly_one(
            by_account
                .remove(&account_id)
                .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?,
        )?;
        candidates.push(RecoveryCatalogCandidateV1 {
            kind,
            source_account_digest_sha256: source_account_digest(
                &source_generation,
                kind,
                &account_id,
            ),
            account_id,
            display_name,
            external_account_id,
            configuration: extract_configuration(kind, &configuration)?,
            legacy_secret: binding,
        });
    }
    if !by_account.is_empty() || !counts.is_exact() || candidates.len() != 3 {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    candidates.sort_by_key(|candidate| candidate.kind);
    Ok(RecoveryCatalogV1 {
        schema_revision: RECOVERY_SCHEMA_REVISION,
        source_generation,
        counts,
        candidates,
    })
}

fn extract_configuration(
    kind: LegacyProviderCandidateKindV1,
    value: &Value,
) -> LegacyProviderRecoveryResultV1<RecoveryCatalogConfigurationV1> {
    let object = value
        .as_object()
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?;
    let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    match kind {
        LegacyProviderCandidateKindV1::Gmail => {
            require_exact_keys(
                &keys,
                &[
                    "api",
                    "auth",
                    "connected_services",
                    "gmail_send_enabled",
                    "history_stream_id",
                    "oauth_client_id",
                    "requested_scopes",
                ],
            )?;
            Ok(RecoveryCatalogConfigurationV1::Gmail {
                oauth_client_id: json_string(object, "oauth_client_id")?,
            })
        }
        LegacyProviderCandidateKindV1::Icloud => {
            require_exact_keys(
                &keys,
                &[
                    "address_book_sync_enabled",
                    "address_book_sync_unsupported_reason",
                    "connected_services",
                    "contacts_sync_enabled",
                    "contacts_sync_unsupported_reason",
                    "host",
                    "mailbox",
                    "mailboxes",
                    "port",
                    "sync_all_mailboxes",
                    "tls",
                    "username",
                ],
            )?;
            let port = object
                .get("port")
                .and_then(Value::as_u64)
                .and_then(|value| u16::try_from(value).ok())
                .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?;
            let tls = object
                .get("tls")
                .and_then(Value::as_bool)
                .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?;
            Ok(RecoveryCatalogConfigurationV1::Icloud {
                imap_host: json_string(object, "host")?,
                imap_port: port,
                tls,
                username: json_string(object, "username")?,
            })
        }
        LegacyProviderCandidateKindV1::TelegramUser => {
            require_exact_keys(
                &keys,
                &["runtime", "tdlib_data_path", "transcription_enabled"],
            )?;
            if json_string(object, "runtime")? != "tdlib_qr_authorized" {
                return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
            }
            Ok(RecoveryCatalogConfigurationV1::TelegramUser)
        }
    }
}

fn validate_cross_source_configuration(
    catalog: &RecoveryCatalogV1,
    google: &GoogleOauthClientV1,
) -> LegacyProviderRecoveryResultV1<()> {
    let gmail = catalog
        .candidates
        .iter()
        .find(|candidate| candidate.kind == LegacyProviderCandidateKindV1::Gmail)
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?;
    match &gmail.configuration {
        RecoveryCatalogConfigurationV1::Gmail { oauth_client_id }
            if oauth_client_id == &google.client_id =>
        {
            Ok(())
        }
        _ => Err(LegacyProviderRecoveryErrorV1::InvalidConfiguration),
    }
}

fn read_google_oauth_client(path: &Path) -> LegacyProviderRecoveryResultV1<GoogleOauthClientV1> {
    let source: GoogleOauthSourceDocumentV1 = serde_json::from_slice(&read_source_file(path)?)
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidConfiguration)?;
    Ok(GoogleOauthClientV1 {
        schema_revision: RECOVERY_SCHEMA_REVISION,
        client_id: source.installed.client_id,
        redirect_uris: source.installed.redirect_uris,
    })
}

fn snapshot_vault(source: &Path, destination: &Path) -> LegacyProviderRecoveryResultV1<()> {
    require_private_source_file(source)?;
    let destination_path = destination.to_path_buf();
    let source = Connection::open_with_flags(
        source,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    let mut destination = Connection::open(destination)
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    let backup = Backup::new(&source, &mut destination)
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    backup
        .run_to_completion(32, std::time::Duration::from_millis(10), None)
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    drop(backup);
    destination
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode = DELETE;")
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    drop(destination);
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", destination_path.to_string_lossy()));
        if sidecar.exists() {
            fs::remove_file(sidecar).map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        }
    }
    Ok(())
}

fn validate_vault_bindings(
    vault_path: &Path,
    catalog: &RecoveryCatalogV1,
) -> LegacyProviderRecoveryResultV1<()> {
    let connection = Connection::open_with_flags(
        vault_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    for candidate in &catalog.candidates {
        let count: u32 = connection
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM vault_entries entry
                JOIN account_secret_manifest manifest
                  ON manifest.secret_ref = entry.secret_ref
                WHERE entry.secret_ref = ?1
                  AND (?5 = 'gmail' OR entry.account_id = ?2)
                  AND entry.purpose = ?3
                  AND manifest.secret_kind = ?4
                  AND manifest.store_kind = 'host_vault'
                "#,
                (
                    &candidate.legacy_secret.secret_ref,
                    &candidate.account_id,
                    &candidate.legacy_secret.purpose,
                    &candidate.legacy_secret.secret_kind,
                    candidate.kind.as_str(),
                ),
                |row| row.get(0),
            )
            .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
        if count != 1 {
            return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
        }
    }
    Ok(())
}

fn build_manifest(
    output_root: &Path,
    catalog: &RecoveryCatalogV1,
) -> LegacyProviderRecoveryResultV1<RecoveryBundleManifestV1> {
    let files = MANIFEST_DATA_FILES
        .iter()
        .map(|relative_path| {
            let bytes = fs::read(output_root.join(relative_path))
                .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
            Ok(RecoveryBundleFileV1 {
                relative_path: (*relative_path).to_owned(),
                size_bytes: bytes.len() as u64,
                sha256: sha256_hex(&bytes),
            })
        })
        .collect::<LegacyProviderRecoveryResultV1<_>>()?;
    let created_at_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?
        .as_secs();
    Ok(RecoveryBundleManifestV1 {
        schema_revision: RECOVERY_SCHEMA_REVISION,
        created_at_unix_seconds,
        source_generation: catalog.source_generation.clone(),
        files,
        catalog_row_count: EXPECTED_CATALOG_ROWS,
        counts: catalog.counts.clone(),
    })
}

fn read_source_file(path: &Path) -> LegacyProviderRecoveryResultV1<Vec<u8>> {
    require_private_source_file(path)?;
    let metadata = fs::metadata(path).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidSource)?;
    if metadata.len() == 0 || metadata.len() > MAX_SOURCE_FILE_BYTES {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSource);
    }
    fs::read(path).map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)
}

fn require_private_source_file(path: &Path) -> LegacyProviderRecoveryResultV1<()> {
    if !path.is_absolute() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSource);
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidSource)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSource);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(LegacyProviderRecoveryErrorV1::InvalidSource);
        }
    }
    Ok(())
}

fn private_json<T: Serialize>(value: &T) -> LegacyProviderRecoveryResultV1<Zeroizing<Vec<u8>>> {
    let mut bytes = Zeroizing::new(
        serde_json::to_vec(value)
            .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidConfiguration)?,
    );
    bytes.push(b'\n');
    Ok(bytes)
}

fn random_hex(bytes: usize) -> LegacyProviderRecoveryResultV1<String> {
    let mut random = vec![0_u8; bytes];
    getrandom::getrandom(&mut random)
        .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
    Ok(random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

fn source_account_digest(
    generation: &str,
    kind: LegacyProviderCandidateKindV1,
    account_id: &str,
) -> String {
    let mut hash = Sha256::new();
    hash.update(generation.as_bytes());
    hash.update([0]);
    hash.update(kind.as_str().as_bytes());
    hash.update([0]);
    hash.update(account_id.as_bytes());
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn exactly_one<T>(values: Vec<T>) -> LegacyProviderRecoveryResultV1<T> {
    let mut values = values.into_iter();
    let value = values
        .next()
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)?;
    if values.next().is_some() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    Ok(value)
}

fn json_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> LegacyProviderRecoveryResultV1<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)
}

fn require_exact_keys(
    actual: &BTreeSet<&str>,
    expected: &[&str],
) -> LegacyProviderRecoveryResultV1<()> {
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != &expected {
        return Err(LegacyProviderRecoveryErrorV1::InvalidCatalog);
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoogleOauthSourceDocumentV1 {
    installed: GoogleOauthInstalledSourceV1,
}

#[derive(Deserialize)]
struct GoogleOauthInstalledSourceV1 {
    client_id: String,
    redirect_uris: Vec<String>,
}

struct PrivateOutputDirectoryV1 {
    temporary_root: PathBuf,
    output_root: PathBuf,
    committed: bool,
}

impl PrivateOutputDirectoryV1 {
    fn create(output_root: &Path) -> LegacyProviderRecoveryResultV1<Self> {
        let parent = output_root
            .parent()
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?;
        let temporary_root = parent.join(format!(".makosh-recovery-{}", random_hex(8)?));
        fs::create_dir(&temporary_root)
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        set_private_permissions(&temporary_root, 0o700);
        Ok(Self {
            temporary_root,
            output_root: output_root.to_owned(),
            committed: false,
        })
    }

    fn path(&self) -> &Path {
        &self.temporary_root
    }

    fn write_json<T: Serialize>(
        &mut self,
        relative_path: &str,
        value: &T,
    ) -> LegacyProviderRecoveryResultV1<()> {
        let bytes = private_json(value)?;
        self.write_bytes(relative_path, &bytes)
    }

    fn write_bytes(
        &mut self,
        relative_path: &str,
        bytes: &[u8],
    ) -> LegacyProviderRecoveryResultV1<()> {
        let path = self.temporary_root.join(relative_path);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(path)
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)
    }

    fn commit(&mut self) -> LegacyProviderRecoveryResultV1<()> {
        fs::rename(&self.temporary_root, &self.output_root)
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        self.committed = true;
        if let Some(parent) = self.output_root.parent() {
            File::open(parent)
                .and_then(|file| file.sync_all())
                .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        }
        Ok(())
    }
}

impl Drop for PrivateOutputDirectoryV1 {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.temporary_root);
        }
    }
}
