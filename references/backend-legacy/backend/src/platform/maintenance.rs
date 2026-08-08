use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::config::app_config::AppConfig;
use crate::platform::storage::database::Database;

const DEV_LOG_ROOT: &str = ".local/dev-logs";
const BACKUPS_ROOT: &str = "backups";
const TELEGRAM_DATA_ROOT: &str = "docker/data/telegram";
const WHATSAPP_DATA_ROOT: &str = "docker/data/whatsapp";
const VAULT_BACKUP_SCRIPT: &str = "scripts/vault-backup.sh";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceOverview {
    pub generated_at: DateTime<Utc>,
    pub inventory: Vec<MaintenanceInventoryItem>,
    pub backups: Vec<MaintenanceBackupItem>,
    pub actions: Vec<MaintenanceActionDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceInventoryItem {
    pub id: String,
    pub label: String,
    pub description: String,
    pub kind: String,
    pub path_label: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub file_count: Option<u64>,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceBackupItem {
    pub id: String,
    pub label: String,
    pub created_at: Option<String>,
    pub path_label: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub has_database_dump: bool,
    pub has_vault_snapshot: bool,
    pub has_storage_snapshot: bool,
    pub manifest_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceActionDescriptor {
    pub id: String,
    pub label: String,
    pub description: String,
    pub icon: String,
    pub destructive: bool,
    pub enabled: bool,
    pub requires_confirmation: bool,
    pub confirmation_phrase: Option<String>,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct MaintenanceActionRequest {
    pub confirmation: Option<String>,
    pub backup_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceActionResponse {
    pub action_id: String,
    pub status: String,
    pub message: String,
    pub completed_at: DateTime<Utc>,
    pub backup: Option<MaintenanceBackupItem>,
}

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceError {
    #[error("unknown maintenance action")]
    UnknownAction,
    #[error("confirmation phrase is required: {0}")]
    ConfirmationRequired(&'static str),
    #[error("{0}")]
    Unsupported(String),
    #[error("maintenance I/O operation failed")]
    Io(#[from] io::Error),
    #[error("maintenance command failed")]
    CommandFailed,
}

impl MaintenanceError {
    pub fn public_message(&self) -> String {
        match self {
            Self::UnknownAction => "Unknown maintenance action".to_owned(),
            Self::ConfirmationRequired(phrase) => {
                format!("Type {phrase} to confirm this maintenance action")
            }
            Self::Unsupported(message) => message.clone(),
            Self::Io(_) => "Maintenance file operation failed".to_owned(),
            Self::CommandFailed => "Maintenance command failed; inspect backend logs".to_owned(),
        }
    }
}

pub async fn build_maintenance_overview(
    config: &AppConfig,
    database: &Database,
) -> MaintenanceOverview {
    let repo = repository_root();
    let database_size = match database.size_bytes().await {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(error = %error, "failed to inspect database size for maintenance overview");
            None
        }
    };
    let inventory = vec![
        database_inventory_item(database_size),
        path_inventory_item(
            &repo,
            "mail_blobs",
            "Mail blob store",
            "Raw .eml payloads and attachment blobs referenced by communications metadata.",
            "storage",
            DEFAULT_MAIL_SYNC_BLOB_ROOT,
        ),
        path_inventory_item(
            &repo,
            "provider_runtime_data",
            "Provider runtime data",
            "Telegram and WhatsApp local runtime state under docker/data.",
            "storage",
            "docker/data",
        ),
        path_inventory_item(
            &repo,
            "dev_logs",
            "Development logs",
            "Local dev watcher and backend log files.",
            "logs",
            DEV_LOG_ROOT,
        ),
        path_inventory_item(
            &repo,
            "backups",
            "Backups",
            "Local backup manifests and copied snapshots.",
            "backup",
            BACKUPS_ROOT,
        ),
        vault_inventory_item(config.vault_home()),
    ];

    let backups = list_backups(&repo);
    let actions = maintenance_actions(&repo);

    MaintenanceOverview {
        generated_at: Utc::now(),
        inventory,
        backups,
        actions,
    }
}

pub fn run_maintenance_action(
    action_id: &str,
    request: MaintenanceActionRequest,
) -> Result<MaintenanceActionResponse, MaintenanceError> {
    match action_id {
        "clean_dev_logs" => {
            require_confirmation(&request, "CLEAN LOGS")?;
            clean_dev_logs()
        }
        "backup_database" => {
            require_confirmation(&request, "BACKUP DB")?;
            backup_database_and_vault()
        }
        "backup_storage" => {
            require_confirmation(&request, "BACKUP STORAGE")?;
            backup_local_storage()
        }
        "restore_database" => Err(MaintenanceError::Unsupported(
            "Database and vault restore is CLI-only while the backend is running. Stop Макошь and run make vault-restore.".to_owned(),
        )),
        "restore_storage" => Err(MaintenanceError::Unsupported(
            "Storage restore is intentionally disabled in the live API until a stopped-process restore workflow exists.".to_owned(),
        )),
        _ => Err(MaintenanceError::UnknownAction),
    }
}

fn require_confirmation(
    request: &MaintenanceActionRequest,
    phrase: &'static str,
) -> Result<(), MaintenanceError> {
    if request.confirmation.as_deref() == Some(phrase) {
        return Ok(());
    }
    Err(MaintenanceError::ConfirmationRequired(phrase))
}

fn database_inventory_item(size_bytes: Option<u64>) -> MaintenanceInventoryItem {
    MaintenanceInventoryItem {
        id: "database".to_owned(),
        label: "PostgreSQL database".to_owned(),
        description: "Logical application state stored in PostgreSQL.".to_owned(),
        kind: "database".to_owned(),
        path_label: "configured database".to_owned(),
        exists: size_bytes.is_some(),
        size_bytes,
        file_count: None,
        status: if size_bytes.is_some() {
            "ok".to_owned()
        } else {
            "unavailable".to_owned()
        },
        detail: if size_bytes.is_some() {
            "pg_database_size(current_database())".to_owned()
        } else {
            "Database is not configured or size inspection failed".to_owned()
        },
    }
}

fn path_inventory_item(
    repo: &Path,
    id: &str,
    label: &str,
    description: &str,
    kind: &str,
    relative_path: &str,
) -> MaintenanceInventoryItem {
    let summary = summarize_path(&repo.join(relative_path));
    inventory_from_summary(id, label, description, kind, relative_path, summary)
}

fn vault_inventory_item(vault_home: &Path) -> MaintenanceInventoryItem {
    inventory_from_summary(
        "host_vault",
        "Host vault",
        "Encrypted local provider credentials and recovery material.",
        "vault",
        "host vault home",
        summarize_path(vault_home),
    )
}

fn inventory_from_summary(
    id: &str,
    label: &str,
    description: &str,
    kind: &str,
    path_label: &str,
    summary: PathSummary,
) -> MaintenanceInventoryItem {
    MaintenanceInventoryItem {
        id: id.to_owned(),
        label: label.to_owned(),
        description: description.to_owned(),
        kind: kind.to_owned(),
        path_label: path_label.to_owned(),
        exists: summary.exists,
        size_bytes: summary.size_bytes,
        file_count: summary.file_count,
        status: summary.status,
        detail: summary.detail,
    }
}

fn maintenance_actions(repo: &Path) -> Vec<MaintenanceActionDescriptor> {
    let backup_script_exists = repo.join(VAULT_BACKUP_SCRIPT).is_file();
    vec![
        MaintenanceActionDescriptor {
            id: "clean_dev_logs".to_owned(),
            label: "Clean dev logs".to_owned(),
            description: "Remove local development logs under .local/dev-logs.".to_owned(),
            icon: "tabler:trash".to_owned(),
            destructive: true,
            enabled: true,
            requires_confirmation: true,
            confirmation_phrase: Some("CLEAN LOGS".to_owned()),
            disabled_reason: None,
        },
        MaintenanceActionDescriptor {
            id: "backup_database".to_owned(),
            label: "Backup DB + vault".to_owned(),
            description: "Run the repository backup script for PostgreSQL and host vault data."
                .to_owned(),
            icon: "tabler:database-export".to_owned(),
            destructive: false,
            enabled: backup_script_exists,
            requires_confirmation: true,
            confirmation_phrase: Some("BACKUP DB".to_owned()),
            disabled_reason: if backup_script_exists {
                None
            } else {
                Some("Backup script was not found".to_owned())
            },
        },
        MaintenanceActionDescriptor {
            id: "backup_storage".to_owned(),
            label: "Backup storage".to_owned(),
            description:
                "Copy local mail and provider runtime storage into a timestamped backup folder."
                    .to_owned(),
            icon: "tabler:archive".to_owned(),
            destructive: false,
            enabled: true,
            requires_confirmation: true,
            confirmation_phrase: Some("BACKUP STORAGE".to_owned()),
            disabled_reason: None,
        },
        MaintenanceActionDescriptor {
            id: "restore_database".to_owned(),
            label: "Restore DB + vault".to_owned(),
            description: "Restore is available through the stopped-process CLI workflow."
                .to_owned(),
            icon: "tabler:database-import".to_owned(),
            destructive: true,
            enabled: false,
            requires_confirmation: true,
            confirmation_phrase: Some("RESTORE".to_owned()),
            disabled_reason: Some("Stop Макошь and run make vault-restore".to_owned()),
        },
        MaintenanceActionDescriptor {
            id: "restore_storage".to_owned(),
            label: "Restore storage".to_owned(),
            description:
                "Storage restore is blocked until Макошь has a stopped-process restore workflow."
                    .to_owned(),
            icon: "tabler:archive-off".to_owned(),
            destructive: true,
            enabled: false,
            requires_confirmation: true,
            confirmation_phrase: Some("RESTORE STORAGE".to_owned()),
            disabled_reason: Some("Live storage restore is not enabled".to_owned()),
        },
    ]
}

fn clean_dev_logs() -> Result<MaintenanceActionResponse, MaintenanceError> {
    let repo = repository_root();
    let log_root = repo.join(DEV_LOG_ROOT);
    if log_root.exists() {
        fs::remove_dir_all(&log_root)?;
    }
    fs::create_dir_all(&log_root)?;
    Ok(action_response(
        "clean_dev_logs",
        "completed",
        "Development logs were cleaned.",
        None,
    ))
}

fn backup_database_and_vault() -> Result<MaintenanceActionResponse, MaintenanceError> {
    let repo = repository_root();
    let output = Command::new(repo.join(VAULT_BACKUP_SCRIPT))
        .current_dir(&repo)
        .output()?;
    if !output.status.success() {
        tracing::warn!(
            status = ?output.status.code(),
            stderr = %String::from_utf8_lossy(&output.stderr),
            "database and vault backup command failed"
        );
        return Err(MaintenanceError::CommandFailed);
    }
    let backup = list_backups(&repo).into_iter().next();
    Ok(action_response(
        "backup_database",
        "completed",
        "Database and vault backup finished.",
        backup,
    ))
}

fn backup_local_storage() -> Result<MaintenanceActionResponse, MaintenanceError> {
    let repo = repository_root();
    let backup_dir = timestamped_backup_dir(&repo, "storage")?;
    let storage_dir = backup_dir.join("storage");
    fs::create_dir_all(&storage_dir)?;

    let mut copied_roots = Vec::new();
    for relative_root in [
        DEFAULT_MAIL_SYNC_BLOB_ROOT,
        TELEGRAM_DATA_ROOT,
        WHATSAPP_DATA_ROOT,
    ] {
        let source = repo.join(relative_root);
        if !source.exists() {
            continue;
        }
        copy_dir_contents(&source, &storage_dir.join(relative_root))?;
        copied_roots.push(relative_root);
    }

    let manifest = json!({
        "created_at": Utc::now().to_rfc3339(),
        "kind": "local_storage",
        "backup_dir": relative_label(&repo, &backup_dir),
        "storage": {
            "relative_path": "storage",
            "copied_roots": copied_roots
        }
    });
    fs::write(
        backup_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)
            .map_err(|error| io::Error::other(error.to_string()))?,
    )?;

    let backup = backup_item_from_dir(&repo, &backup_dir);
    Ok(action_response(
        "backup_storage",
        "completed",
        "Local storage backup finished.",
        backup,
    ))
}

fn action_response(
    action_id: &str,
    status: &str,
    message: &str,
    backup: Option<MaintenanceBackupItem>,
) -> MaintenanceActionResponse {
    MaintenanceActionResponse {
        action_id: action_id.to_owned(),
        status: status.to_owned(),
        message: message.to_owned(),
        completed_at: Utc::now(),
        backup,
    }
}

fn list_backups(repo: &Path) -> Vec<MaintenanceBackupItem> {
    let root = repo.join(BACKUPS_ROOT);
    let Ok(day_entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut backups = Vec::new();
    for day_entry in day_entries.flatten() {
        let Ok(file_type) = day_entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let Ok(backup_entries) = fs::read_dir(day_entry.path()) else {
            continue;
        };
        for backup_entry in backup_entries.flatten() {
            let Ok(backup_type) = backup_entry.file_type() else {
                continue;
            };
            if !backup_type.is_dir() {
                continue;
            }
            if let Some(item) = backup_item_from_dir(repo, &backup_entry.path()) {
                backups.push(item);
            }
        }
    }
    backups.sort_by(|left, right| right.id.cmp(&left.id));
    backups.truncate(20);
    backups
}

fn backup_item_from_dir(repo: &Path, backup_dir: &Path) -> Option<MaintenanceBackupItem> {
    let summary = summarize_path(backup_dir);
    if !summary.exists {
        return None;
    }
    let manifest_path = backup_dir.join("manifest.json");
    let manifest_present = manifest_path.is_file();
    let created_at = read_manifest_created_at(&manifest_path);
    let id = relative_label(repo, backup_dir);
    Some(MaintenanceBackupItem {
        label: backup_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("backup")
            .to_owned(),
        path_label: id.clone(),
        id,
        created_at,
        size_bytes: summary.size_bytes.unwrap_or(0),
        file_count: summary.file_count.unwrap_or(0),
        has_database_dump: backup_dir.join("postgres.sql").is_file(),
        has_vault_snapshot: backup_dir.join("vault").is_dir(),
        has_storage_snapshot: backup_dir.join("storage").is_dir(),
        manifest_present,
    })
}

fn read_manifest_created_at(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let value = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    value
        .get("created_at")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

#[derive(Debug, Clone)]
struct PathSummary {
    exists: bool,
    size_bytes: Option<u64>,
    file_count: Option<u64>,
    status: String,
    detail: String,
}

fn summarize_path(path: &Path) -> PathSummary {
    if !path.exists() {
        return PathSummary {
            exists: false,
            size_bytes: Some(0),
            file_count: Some(0),
            status: "missing".to_owned(),
            detail: "Path does not exist yet".to_owned(),
        };
    }
    match summarize_existing_path(path) {
        Ok((size_bytes, file_count)) => PathSummary {
            exists: true,
            size_bytes: Some(size_bytes),
            file_count: Some(file_count),
            status: "ok".to_owned(),
            detail: "Filesystem path inspected without following symlinks".to_owned(),
        },
        Err(error) => {
            tracing::warn!(error = %error, "maintenance path inspection failed");
            PathSummary {
                exists: true,
                size_bytes: None,
                file_count: None,
                status: "unavailable".to_owned(),
                detail: "Path exists but could not be inspected".to_owned(),
            }
        }
    }
}

fn summarize_existing_path(path: &Path) -> io::Result<(u64, u64)> {
    let mut size_bytes = 0_u64;
    let mut file_count = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            file_count += 1;
            size_bytes = size_bytes.saturating_add(metadata.len());
            continue;
        }
        if metadata.is_file() {
            file_count += 1;
            size_bytes = size_bytes.saturating_add(metadata.len());
            continue;
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(&current)? {
                stack.push(entry?.path());
            }
        }
    }
    Ok((size_bytes, file_count))
}

fn copy_dir_contents(source: &Path, destination: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_file_type = entry.file_type()?;
        if entry_file_type.is_symlink() {
            continue;
        }
        copy_dir_contents(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn timestamped_backup_dir(repo: &Path, suffix: &str) -> io::Result<PathBuf> {
    let now = Utc::now();
    let day = now.format("%Y-%m-%d").to_string();
    let stamp = now.format("%Y%m%dT%H%M%SZ").to_string();
    let backup_dir = repo
        .join(BACKUPS_ROOT)
        .join(day)
        .join(format!("{stamp}-{suffix}"));
    fs::create_dir_all(&backup_dir)?;
    Ok(backup_dir)
}

fn repository_root() -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in current.ancestors() {
        if ancestor.join("backend/Cargo.toml").is_file() && ancestor.join("scripts").is_dir() {
            return ancestor.to_path_buf();
        }
    }
    current
}

fn relative_label(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(path_to_forward_slash)
        .unwrap_or_else(|| {
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("path")
                .to_owned()
        })
}

fn path_to_forward_slash(path: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_str()?.to_owned()),
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintenance_restore_actions_are_disabled() {
        let actions = maintenance_actions(&repository_root());
        let restore = actions
            .iter()
            .find(|action| action.id == "restore_database")
            .expect("restore action");

        assert!(!restore.enabled);
        assert!(restore.destructive);
        assert_eq!(restore.confirmation_phrase.as_deref(), Some("RESTORE"));
    }

    #[test]
    fn maintenance_action_requires_exact_confirmation_phrase() {
        let request = MaintenanceActionRequest {
            confirmation: Some("clean logs".to_owned()),
            backup_id: None,
        };

        let error = require_confirmation(&request, "CLEAN LOGS").expect_err("confirmation error");
        assert_eq!(
            error.public_message(),
            "Type CLEAN LOGS to confirm this maintenance action"
        );
    }
}
