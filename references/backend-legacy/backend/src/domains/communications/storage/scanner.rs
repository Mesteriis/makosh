use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::Cursor;
use zip::ZipArchive;

use super::errors::{AttachmentSafetyScanError, CommunicationStorageError};
use super::validation::validate_non_empty;
use crate::platform::attachment_scanning::{ClamAvClient, ClamAvVerdict};

const MAX_OOXML_INSPECTION_BYTES: usize = 50 * 1024 * 1024;
const MAX_OOXML_INSPECTION_ENTRIES: usize = 10_000;
const MAX_LEGACY_OLE_DIRECTORY_SECTORS: usize = 4_096;
const LEGACY_OLE_MAGIC: &[u8] = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1";
const LEGACY_OLE_END_OF_CHAIN: u32 = 0xFFFF_FFFE;
const LEGACY_OLE_FREE_SECTOR: u32 = 0xFFFF_FFFF;
const LEGACY_OLE_FAT_SECTOR: u32 = 0xFFFF_FFFD;
const RAR4_MAGIC: &[u8] = b"Rar!\x1a\x07\x00";
const RAR5_MAGIC: &[u8] = b"Rar!\x1a\x07\x01\x00";
const SEVEN_Z_MAGIC: &[u8] = b"7z\xbc\xaf'\x1c";

#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentSafetyScanReport {
    pub status: AttachmentSafetyScanStatus,
    pub engine: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub metadata: Value,
}

impl AttachmentSafetyScanReport {
    pub fn not_scanned() -> Self {
        Self {
            status: AttachmentSafetyScanStatus::NotScanned,
            engine: None,
            checked_at: None,
            summary: None,
            metadata: json!({}),
        }
    }

    pub(crate) fn validate(&self) -> Result<Self, CommunicationStorageError> {
        let engine = self
            .engine
            .as_deref()
            .map(|value| validate_non_empty("scan_engine", value))
            .transpose()?;
        let summary = self
            .summary
            .as_deref()
            .map(|value| validate_non_empty("scan_summary", value))
            .transpose()?;
        if !self.metadata.is_object() {
            return Err(CommunicationStorageError::NonObjectJson("scan_metadata"));
        }

        if self.status == AttachmentSafetyScanStatus::NotScanned
            && (engine.is_some() || self.checked_at.is_some() || summary.is_some())
        {
            return Err(CommunicationStorageError::InvalidNotScannedReport);
        }

        Ok(Self {
            status: self.status,
            engine,
            checked_at: self.checked_at,
            summary,
            metadata: self.metadata.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyScanStatus {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl AttachmentSafetyScanStatus {
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

impl TryFrom<&str> for AttachmentSafetyScanStatus {
    type Error = CommunicationStorageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_scanned" => Ok(Self::NotScanned),
            "clean" => Ok(Self::Clean),
            "suspicious" => Ok(Self::Suspicious),
            "malicious" => Ok(Self::Malicious),
            "failed" => Ok(Self::Failed),
            other => Err(CommunicationStorageError::InvalidScanStatus(
                other.to_owned(),
            )),
        }
    }
}

pub struct AttachmentSafetyScanRequest<'a> {
    pub provider_attachment_id: &'a str,
    pub filename: Option<&'a str>,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub sha256: &'a str,
    pub storage_kind: &'a str,
    pub storage_path: &'a str,
    pub bytes: &'a [u8],
}

pub trait AttachmentSafetyScanner {
    fn scan(
        &self,
        request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError>;
}

/// Applies deterministic prefiltering first, then uses the local ClamAV verdict only when the
/// prefilter found no reason to keep the attachment quarantined. Scanner failures deliberately
/// retain `not_scanned`, so callers can retry without ever treating an unavailable verdict as
/// clean.
pub async fn scan_attachment_with_clamav(
    request: &AttachmentSafetyScanRequest<'_>,
    clamav: Option<&ClamAvClient>,
) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
    let heuristic = HeuristicAttachmentSafetyScanner.scan(request)?;
    if heuristic.status != AttachmentSafetyScanStatus::NotScanned {
        return Ok(heuristic);
    }
    let Some(clamav) = clamav else {
        return Ok(heuristic);
    };

    match clamav.scan(request.bytes).await {
        Ok(ClamAvVerdict::Clean) => Ok(AttachmentSafetyScanReport {
            status: AttachmentSafetyScanStatus::Clean,
            engine: Some("clamav_clamd".to_owned()),
            checked_at: Some(Utc::now()),
            summary: Some("ClamAV found no known malware".to_owned()),
            metadata: json!({ "verdict": "clean" }),
        }),
        Ok(ClamAvVerdict::Malicious { signature }) => Ok(AttachmentSafetyScanReport {
            status: AttachmentSafetyScanStatus::Malicious,
            engine: Some("clamav_clamd".to_owned()),
            checked_at: Some(Utc::now()),
            summary: Some("ClamAV detected malware".to_owned()),
            metadata: json!({ "verdict": "malicious", "signature": signature }),
        }),
        Err(error) => {
            tracing::warn!(error = %error, "ClamAV attachment scan failed; attachment remains quarantined");
            Ok(heuristic)
        }
    }
}

pub async fn scan_attachment_with_configured_clamav(
    request: &AttachmentSafetyScanRequest<'_>,
) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
    let clamav = match ClamAvClient::from_env() {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!(error = %error, "ClamAV attachment scanner configuration is invalid");
            None
        }
    };
    scan_attachment_with_clamav(request, clamav.as_ref()).await
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicAttachmentSafetyScanner;

impl AttachmentSafetyScanner for HeuristicAttachmentSafetyScanner {
    fn scan(
        &self,
        request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        let extension = normalized_extension(request.filename);
        let content_type = normalized_content_type(request.content_type);
        let mut reasons = Vec::new();
        let mut status = AttachmentSafetyScanStatus::NotScanned;
        let mut legacy_ole_inspection = None;

        if has_executable_magic(request.bytes) {
            status = AttachmentSafetyScanStatus::Malicious;
            reasons.push("executable_magic");
        }

        if let Some(extension) = extension.as_deref() {
            if is_active_content_extension(extension) {
                status = AttachmentSafetyScanStatus::Malicious;
                reasons.push("active_content_extension");
            } else if is_macro_document_extension(extension) {
                status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
                reasons.push("macro_enabled_document_extension");
            }
        }

        if let Some(extension) = extension.as_deref()
            && is_mime_extension_mismatch(&content_type, extension)
        {
            status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
            reasons.push("mime_extension_mismatch");
        }

        if has_legacy_ole_magic(request.bytes) {
            status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
            match inspect_legacy_ole_container(request.bytes) {
                LegacyOleInspection::MacroMarkers => {
                    reasons.push("legacy_ole_vba_storage");
                    legacy_ole_inspection = Some("vba_markers");
                }
                LegacyOleInspection::NoMacroMarkers => {
                    reasons.push("legacy_ole_compound_document");
                    legacy_ole_inspection = Some("no_vba_markers");
                }
                LegacyOleInspection::Unreadable => {
                    reasons.push("legacy_ole_container_unreadable");
                    legacy_ole_inspection = Some("unreadable");
                }
                LegacyOleInspection::TooLarge => {
                    reasons.push("legacy_ole_container_exceeds_inspection_limit");
                    legacy_ole_inspection = Some("too_large");
                }
            }
        }

        if is_uninspected_archive(extension.as_deref(), &content_type, request.bytes) {
            status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
            reasons.push("uninspected_rar_or_7z_archive");
        }

        if is_ooxml_document(extension.as_deref(), &content_type) {
            match inspect_ooxml_container(request.bytes) {
                OoxmlInspection::MacroPayload => {
                    status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
                    reasons.push("ooxml_macro_payload");
                }
                OoxmlInspection::Unreadable => {
                    status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
                    reasons.push("ooxml_container_unreadable");
                }
                OoxmlInspection::TooLarge => {
                    status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
                    reasons.push("ooxml_container_exceeds_inspection_limit");
                }
                OoxmlInspection::NoMacroPayload => {}
            }
        }

        if status == AttachmentSafetyScanStatus::NotScanned {
            return Ok(AttachmentSafetyScanReport::not_scanned());
        }

        Ok(AttachmentSafetyScanReport {
            status,
            engine: Some("makosh_heuristic_v1".to_owned()),
            checked_at: Some(Utc::now()),
            summary: Some(scan_summary(status).to_owned()),
            metadata: json!({
                "reasons": reasons,
                "content_type": content_type,
                "filename_extension": extension,
                "size_bytes": request.size_bytes,
                "legacy_ole_inspection": legacy_ole_inspection,
            }),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopAttachmentSafetyScanner;

impl AttachmentSafetyScanner for NoopAttachmentSafetyScanner {
    fn scan(
        &self,
        _request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        Ok(AttachmentSafetyScanReport::not_scanned())
    }
}

fn max_scan_status(
    current: AttachmentSafetyScanStatus,
    candidate: AttachmentSafetyScanStatus,
) -> AttachmentSafetyScanStatus {
    if scan_status_rank(candidate) > scan_status_rank(current) {
        candidate
    } else {
        current
    }
}

fn scan_status_rank(status: AttachmentSafetyScanStatus) -> u8 {
    match status {
        AttachmentSafetyScanStatus::NotScanned => 0,
        AttachmentSafetyScanStatus::Clean => 1,
        AttachmentSafetyScanStatus::Suspicious => 2,
        AttachmentSafetyScanStatus::Failed => 3,
        AttachmentSafetyScanStatus::Malicious => 4,
    }
}

fn scan_summary(status: AttachmentSafetyScanStatus) -> &'static str {
    match status {
        AttachmentSafetyScanStatus::Malicious => "Executable payload detected",
        AttachmentSafetyScanStatus::Suspicious => "Attachment metadata requires safety review",
        AttachmentSafetyScanStatus::Failed => "Attachment safety scan failed",
        AttachmentSafetyScanStatus::Clean | AttachmentSafetyScanStatus::NotScanned => {
            "Attachment was not scanned by a safety backend"
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};
    use std::time::Duration;

    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    fn request<'a>(
        filename: Option<&'a str>,
        content_type: &'a str,
        bytes: &'a [u8],
    ) -> AttachmentSafetyScanRequest<'a> {
        AttachmentSafetyScanRequest {
            provider_attachment_id: "part-1",
            filename,
            content_type,
            size_bytes: bytes.len() as i64,
            sha256: "sha256:fixture",
            storage_kind: "local_fs",
            storage_path: "aa/bb/blob",
            bytes,
        }
    }

    async fn fake_clamd(response: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake ClamAV");
        let address = listener.local_addr().expect("listener address").to_string();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept scan");
            let mut command = [0_u8; 10];
            socket.read_exact(&mut command).await.expect("read command");
            assert_eq!(&command, b"zINSTREAM\0");
            loop {
                let length = socket.read_u32().await.expect("read frame") as usize;
                if length == 0 {
                    break;
                }
                let mut chunk = vec![0_u8; length];
                socket.read_exact(&mut chunk).await.expect("read body");
            }
            socket.write_all(response).await.expect("write verdict");
        });
        address
    }

    #[tokio::test]
    async fn local_clamav_clean_verdict_transitions_unmatched_attachment_to_clean() {
        let address = fake_clamd(b"stream: OK\0").await;
        let client = ClamAvClient::new(address, Duration::from_secs(1)).expect("client");

        let report = scan_attachment_with_clamav(
            &request(Some("invoice.txt"), "text/plain", b"safe attachment"),
            Some(&client),
        )
        .await
        .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Clean);
        assert_eq!(report.engine.as_deref(), Some("clamav_clamd"));
        assert_eq!(report.metadata, json!({ "verdict": "clean" }));
    }

    #[tokio::test]
    async fn unavailable_clamav_retains_unmatched_attachment_quarantined() {
        let client = ClamAvClient::new("127.0.0.1:1", Duration::from_millis(20)).expect("client");

        let report = scan_attachment_with_clamav(
            &request(Some("invoice.txt"), "text/plain", b"safe attachment"),
            Some(&client),
        )
        .await
        .expect("scan report");

        assert_eq!(report, AttachmentSafetyScanReport::not_scanned());
    }

    #[test]
    fn heuristic_scanner_leaves_unmatched_attachments_not_scanned() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(Some("invoice.txt"), "text/plain", b"hello"))
            .expect("scan report");

        assert_eq!(report, AttachmentSafetyScanReport::not_scanned());
    }

    #[test]
    fn heuristic_scanner_marks_executable_payloads_malicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("invoice.pdf"),
                "application/pdf",
                b"MZ\x90\x00fake portable executable",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Malicious);
        assert_eq!(report.engine.as_deref(), Some("makosh_heuristic_v1"));
        assert!(report.checked_at.is_some());
        assert_eq!(
            report.summary.as_deref(),
            Some("Executable payload detected")
        );
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["executable_magic"])
        );
    }

    #[test]
    fn heuristic_scanner_marks_mime_filename_mismatch_suspicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("invoice.pdf"),
                "application/zip",
                b"PK\x03\x04fake zip",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(report.engine.as_deref(), Some("makosh_heuristic_v1"));
        assert!(report.checked_at.is_some());
        assert_eq!(
            report.summary.as_deref(),
            Some("Attachment metadata requires safety review")
        );
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["mime_extension_mismatch"])
        );
    }

    #[test]
    fn heuristic_scanner_marks_ooxml_macro_payloads_suspicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let bytes = zip_bytes(&[
            ("[Content_Types].xml", b"types" as &[u8]),
            ("word/vbaProject.bin", b"macro payload"),
        ]);

        let report = scanner
            .scan(&request(
                Some("report.docx"),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                &bytes,
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["ooxml_macro_payload"])
        );
    }

    #[test]
    fn heuristic_scanner_marks_legacy_ole_documents_suspicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("legacy.doc"),
                "application/msword",
                b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1legacy",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["legacy_ole_container_unreadable"])
        );
        assert_eq!(report.metadata["legacy_ole_inspection"], "unreadable");
    }

    #[test]
    fn heuristic_scanner_detects_vba_storage_in_bounded_legacy_ole_directory() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let bytes = legacy_ole_bytes(&["Root Entry", "VBA"]);
        let report = scanner
            .scan(&request(Some("macro.doc"), "application/msword", &bytes))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["legacy_ole_vba_storage"])
        );
        assert_eq!(report.metadata["legacy_ole_inspection"], "vba_markers");
    }

    #[test]
    fn heuristic_scanner_records_legacy_ole_without_macro_markers() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let bytes = legacy_ole_bytes(&["Root Entry", "WordDocument"]);
        let report = scanner
            .scan(&request(Some("legacy.doc"), "application/msword", &bytes))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["legacy_ole_compound_document"])
        );
        assert_eq!(report.metadata["legacy_ole_inspection"], "no_vba_markers");
    }

    #[test]
    fn heuristic_scanner_quarantines_uninspected_rar_and_7z_archives() {
        let scanner = HeuristicAttachmentSafetyScanner;

        let rar = scanner
            .scan(&request(
                Some("archive.rar"),
                "application/x-rar-compressed",
                b"Rar!\x1a\x07\x00fixture",
            ))
            .expect("RAR scan report");
        assert_eq!(rar.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(
            rar.metadata["reasons"],
            serde_json::json!(["uninspected_rar_or_7z_archive"])
        );

        let seven_z = scanner
            .scan(&request(
                Some("archive.bin"),
                "application/octet-stream",
                b"7z\xbc\xaf'\x1cfixture",
            ))
            .expect("7z scan report");
        assert_eq!(seven_z.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(
            seven_z.metadata["reasons"],
            serde_json::json!(["uninspected_rar_or_7z_archive"])
        );
    }

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, bytes) in entries {
            writer.start_file(*name, options).expect("ZIP entry");
            writer.write_all(bytes).expect("ZIP entry bytes");
        }
        writer.finish().expect("finish ZIP").into_inner()
    }

    fn legacy_ole_bytes(directory_names: &[&str]) -> Vec<u8> {
        const SECTOR_SIZE: usize = 512;
        let mut bytes = vec![0_u8; 512 + SECTOR_SIZE * 2];
        bytes[..8].copy_from_slice(LEGACY_OLE_MAGIC);
        write_u16(&mut bytes, 28, 0xFFFE);
        write_u16(&mut bytes, 30, 9);
        write_u16(&mut bytes, 32, 6);
        write_u16(&mut bytes, 26, 3);
        write_u32(&mut bytes, 44, 1);
        write_u32(&mut bytes, 48, 1);
        write_u32(&mut bytes, 56, 4_096);
        write_u32(&mut bytes, 60, LEGACY_OLE_FREE_SECTOR);
        write_u32(&mut bytes, 68, LEGACY_OLE_FREE_SECTOR);
        for index in 0..109 {
            write_u32(&mut bytes, 76 + index * 4, LEGACY_OLE_FREE_SECTOR);
        }
        write_u32(&mut bytes, 76, 0);

        let fat_offset = 512;
        for index in 0..SECTOR_SIZE / 4 {
            write_u32(&mut bytes, fat_offset + index * 4, LEGACY_OLE_FREE_SECTOR);
        }
        write_u32(&mut bytes, fat_offset, LEGACY_OLE_FAT_SECTOR);
        write_u32(&mut bytes, fat_offset + 4, LEGACY_OLE_END_OF_CHAIN);

        let directory_offset = 512 + SECTOR_SIZE;
        for (index, name) in directory_names.iter().enumerate() {
            let entry_offset = directory_offset + index * 128;
            write_legacy_ole_directory_name(&mut bytes, entry_offset, name);
            bytes[entry_offset + 66] = if index == 0 { 5 } else { 1 };
        }
        bytes
    }

    fn write_legacy_ole_directory_name(bytes: &mut [u8], offset: usize, name: &str) {
        let mut code_units: Vec<u16> = name.encode_utf16().collect();
        code_units.push(0);
        for (index, code_unit) in code_units.iter().enumerate() {
            write_u16(bytes, offset + index * 2, *code_unit);
        }
        write_u16(bytes, offset + 64, (code_units.len() * 2) as u16);
    }

    fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
mod policy;
use policy::*;
