use std::future::Future;
use std::path::{Component, Path};
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::errors::StorageError;

const LOCAL_FS_STORAGE_KIND: &str = "local_fs";
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalBlobRecord {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredBlobRecord {
    pub blob_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyScanStatus {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl SafetyScanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotScanned => "not_scanned",
            Self::Clean => "clean",
            Self::Suspicious => "suspicious",
            Self::Malicious => "malicious",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SafetyScanReport {
    pub status: SafetyScanStatus,
    pub engine: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub metadata: Value,
}

impl SafetyScanReport {
    pub fn not_scanned() -> Self {
        Self {
            status: SafetyScanStatus::NotScanned,
            engine: None,
            checked_at: None,
            summary: None,
            metadata: json!({}),
        }
    }
}

pub struct SafetyScanRequest<'a> {
    pub filename: Option<&'a str>,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedAttachmentRecord {
    pub attachment_id: String,
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source_kind: String,
    pub imported_by: String,
    pub scan_status: SafetyScanStatus,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub scan_metadata: Value,
    pub metadata: Value,
    pub storage_kind: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedAttachmentRemovalResult {
    pub imported_attachment: ImportedAttachmentRecord,
    pub blob_metadata_removed: bool,
}

#[derive(Clone, Debug)]
pub struct ImportedAttachmentUpsert {
    pub attachment_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source_kind: String,
    pub imported_by: String,
    pub scan_report: SafetyScanReport,
    pub metadata: Value,
}

pub trait ImportedAttachmentStoragePort: Send + Sync {
    fn upsert_blob_record<'a>(
        &'a self,
        blob: &'a LocalBlobRecord,
        content_type: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<StoredBlobRecord, StorageError>> + Send + 'a>>;

    fn upsert_imported_attachment_record<'a>(
        &'a self,
        import: &'a ImportedAttachmentUpsert,
    ) -> Pin<Box<dyn Future<Output = Result<ImportedAttachmentRecord, StorageError>> + Send + 'a>>;

    fn list_imported_attachment_records<'a>(
        &'a self,
        account_id: &'a str,
        source_kind: &'a str,
        limit: i64,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ImportedAttachmentRecord>, StorageError>> + Send + 'a>,
    >;

    fn list_expired_imported_attachment_records<'a>(
        &'a self,
        account_id: &'a str,
        source_kind: &'a str,
        limit: i64,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ImportedAttachmentRecord>, StorageError>> + Send + 'a>,
    >;

    fn remove_imported_attachment_record<'a>(
        &'a self,
        attachment_id: &'a str,
        account_id: &'a str,
        source_kind: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ImportedAttachmentRemovalResult>, StorageError>>
                + Send
                + 'a,
        >,
    >;
}

pub fn new_attachment_import_id(seed: &str) -> String {
    format!("att-import:v1:{}:{}", seed.len(), seed)
}

pub async fn put_local_blob(root: &str, bytes: &[u8]) -> Result<LocalBlobRecord, StorageError> {
    let size_bytes = i64::try_from(bytes.len())
        .map_err(|_| StorageError::Invalid("blob too large".to_owned()))?;
    let digest_hex = sha256_hex(bytes);
    let storage_path = format!("sha256/{}/{}.blob", &digest_hex[..2], digest_hex);
    let absolute_path = Path::new(root).join(&storage_path);

    if let Some(parent) = absolute_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    if !path_exists(&absolute_path).await? {
        let temp_path = absolute_path.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        tokio::fs::write(&temp_path, bytes).await?;
        tokio::fs::rename(&temp_path, &absolute_path).await?;
    }

    let actual_size = i64::try_from(tokio::fs::metadata(&absolute_path).await?.len())
        .map_err(|_| StorageError::Invalid("blob too large".to_owned()))?;
    if actual_size != size_bytes {
        return Err(StorageError::Invalid(format!(
            "blob size mismatch for {}: expected {}, actual {}",
            absolute_path.display(),
            size_bytes,
            actual_size
        )));
    }

    Ok(LocalBlobRecord {
        storage_kind: LOCAL_FS_STORAGE_KIND.to_owned(),
        storage_path,
        sha256: format!("{SHA256_PREFIX}{digest_hex}"),
        size_bytes,
    })
}

pub async fn delete_local_blob(root: &str, storage_path: &str) -> Result<bool, StorageError> {
    let storage_path = validate_storage_path(storage_path)?;
    let absolute_path = Path::new(root).join(&storage_path);
    match tokio::fs::remove_file(&absolute_path).await {
        Ok(()) => {
            prune_empty_parent_dirs(Path::new(root), &absolute_path).await?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub fn scan_attachment(request: &SafetyScanRequest<'_>) -> Result<SafetyScanReport, StorageError> {
    let extension = normalized_extension(request.filename);
    let content_type = normalized_content_type(request.content_type);
    let mut reasons = Vec::new();
    let mut status = SafetyScanStatus::NotScanned;

    if has_executable_magic(request.bytes) {
        status = SafetyScanStatus::Malicious;
        reasons.push("executable_magic");
    }

    if let Some(extension) = extension.as_deref() {
        if is_active_content_extension(extension) {
            status = SafetyScanStatus::Malicious;
            reasons.push("active_content_extension");
        } else if is_macro_document_extension(extension) {
            status = max_scan_status(status, SafetyScanStatus::Suspicious);
            reasons.push("macro_enabled_document_extension");
        }
    }

    if let Some(extension) = extension.as_deref()
        && is_mime_extension_mismatch(&content_type, extension)
    {
        status = max_scan_status(status, SafetyScanStatus::Suspicious);
        reasons.push("mime_extension_mismatch");
    }

    if status == SafetyScanStatus::NotScanned {
        return Ok(SafetyScanReport::not_scanned());
    }

    Ok(SafetyScanReport {
        status,
        engine: Some("makosh_heuristic_v1".to_owned()),
        checked_at: Some(Utc::now()),
        summary: Some(scan_summary(status).to_owned()),
        metadata: json!({
            "reasons": reasons,
            "content_type": content_type,
            "filename_extension": extension,
            "size_bytes": request.size_bytes,
        }),
    })
}

fn validate_storage_path(value: &str) -> Result<String, StorageError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(StorageError::Invalid(
            "storage_path must not be empty".to_owned(),
        ));
    }
    let path = Path::new(value.as_str());
    if path.is_absolute() || value.contains('\\') {
        return Err(StorageError::Invalid(format!(
            "storage_path must be relative and stay inside mail blob root: {value}"
        )));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(StorageError::Invalid(format!(
                    "storage_path must be relative and stay inside mail blob root: {value}"
                )));
            }
        }
    }

    Ok(value)
}

async fn path_exists(path: &Path) -> Result<bool, std::io::Error> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

async fn prune_empty_parent_dirs(root: &Path, path: &Path) -> Result<(), std::io::Error> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == root {
            break;
        }
        match tokio::fs::remove_dir(dir).await {
            Ok(()) => current = dir.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(hex_char(byte >> 4));
        encoded.push(hex_char(byte & 0x0f));
    }
    encoded
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}

fn normalized_extension(filename: Option<&str>) -> Option<String> {
    let filename = filename?.trim();
    let basename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    let (_, extension) = basename.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    (!extension.is_empty()).then_some(extension)
}

fn normalized_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase()
}

fn has_executable_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ") || bytes.starts_with(b"\x7fELF")
}

fn is_active_content_extension(extension: &str) -> bool {
    matches!(
        extension,
        "app"
            | "bat"
            | "cmd"
            | "com"
            | "dll"
            | "dmg"
            | "exe"
            | "hta"
            | "jar"
            | "jse"
            | "js"
            | "msi"
            | "ps1"
            | "scr"
            | "vbe"
            | "vbs"
            | "wsf"
    )
}

fn is_macro_document_extension(extension: &str) -> bool {
    matches!(
        extension,
        "docm" | "dotm" | "xlsm" | "xltm" | "pptm" | "potm"
    )
}

fn is_mime_extension_mismatch(content_type: &str, extension: &str) -> bool {
    let expected = expected_extensions_for_content_type(content_type);
    !expected.is_empty() && !expected.contains(&extension)
}

fn expected_extensions_for_content_type(content_type: &str) -> &'static [&'static str] {
    match content_type {
        "application/pdf" => &["pdf"],
        "application/zip" => &["zip"],
        "image/jpeg" => &["jpg", "jpeg"],
        "image/png" => &["png"],
        "text/csv" => &["csv"],
        "text/plain" => &["txt", "text", "log", "csv"],
        _ => &[],
    }
}

fn max_scan_status(current: SafetyScanStatus, candidate: SafetyScanStatus) -> SafetyScanStatus {
    if scan_status_rank(candidate) > scan_status_rank(current) {
        candidate
    } else {
        current
    }
}

fn scan_status_rank(status: SafetyScanStatus) -> u8 {
    match status {
        SafetyScanStatus::NotScanned => 0,
        SafetyScanStatus::Clean => 1,
        SafetyScanStatus::Suspicious => 2,
        SafetyScanStatus::Failed => 3,
        SafetyScanStatus::Malicious => 4,
    }
}

fn scan_summary(status: SafetyScanStatus) -> &'static str {
    match status {
        SafetyScanStatus::Malicious => "Executable payload detected",
        SafetyScanStatus::Suspicious => "Attachment metadata requires safety review",
        SafetyScanStatus::Failed => "Attachment safety scan failed",
        SafetyScanStatus::Clean | SafetyScanStatus::NotScanned => {
            "Attachment was not scanned by a safety backend"
        }
    }
}
