#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_MAX_PATH_BYTES_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_MAX_REPORT_ENTRIES_V1,
};

mod join;
mod lifecycle;

pub use join::{
    ArchiveInspectionCanonicalSafetyFactV1, ArchiveInspectionCustodyDelegationIntentV1,
    ArchiveInspectionJoinDecisionV1, ArchiveInspectionRecordDecisionV1,
    ArchiveInspectionRejectionV1, ArchiveInspectionRequestV1, ArchiveInspectionSafetyStateV1,
    ArchiveInspectionScanCandidateV1, archive_inspection_rejection_evidence_id_v1,
    decide_archive_inspection_join_v1, decide_archive_inspection_safety_record_v1,
    decide_archive_scan_candidate_record_v1, validate_archive_inspection_request_v1,
};
pub use lifecycle::{
    ArchiveInspectionErrorV1, ArchiveInspectionStateV1, ArchiveInspectionStatusV1,
    ArchiveInspectionTransitionErrorV1, ArchiveInspectionTransitionV1,
    accepted_archive_inspection_status_v1, transition_archive_inspection_status_v1,
    validate_archive_inspection_status_v1,
};

pub const PACKAGE: &str = "makosh-attachment-archive-inspection-core";
pub const DEFAULT_MAX_ARCHIVE_BYTES_V1: u64 = 100 * 1024 * 1024;
pub const DEFAULT_MAX_TOTAL_UNCOMPRESSED_BYTES_V1: u64 = 1024 * 1024 * 1024;
pub const DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1: u64 = 256 * 1024 * 1024;
pub const DEFAULT_MAX_DEPTH_V1: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionLimitsV1 {
    max_archive_bytes: u64,
    max_total_uncompressed_bytes: u64,
    max_entry_uncompressed_bytes: u64,
    max_entries: usize,
    max_depth: usize,
    max_path_bytes: usize,
}

impl ArchiveInspectionLimitsV1 {
    pub fn new(
        max_archive_bytes: u64,
        max_total_uncompressed_bytes: u64,
        max_entry_uncompressed_bytes: u64,
        max_entries: usize,
        max_depth: usize,
        max_path_bytes: usize,
    ) -> Result<Self, ArchiveInspectionPolicyErrorV1> {
        let valid = max_archive_bytes > 0
            && max_archive_bytes <= DEFAULT_MAX_ARCHIVE_BYTES_V1
            && max_total_uncompressed_bytes > 0
            && max_total_uncompressed_bytes <= DEFAULT_MAX_TOTAL_UNCOMPRESSED_BYTES_V1
            && max_entry_uncompressed_bytes > 0
            && max_entry_uncompressed_bytes <= DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1
            && max_entry_uncompressed_bytes <= max_total_uncompressed_bytes
            && max_entries > 0
            && max_entries <= ATTACHMENT_ARCHIVE_INSPECTION_MAX_REPORT_ENTRIES_V1
            && max_depth > 0
            && max_depth <= DEFAULT_MAX_DEPTH_V1
            && max_path_bytes > 0
            && max_path_bytes <= ATTACHMENT_ARCHIVE_INSPECTION_MAX_PATH_BYTES_V1;
        valid
            .then_some(Self {
                max_archive_bytes,
                max_total_uncompressed_bytes,
                max_entry_uncompressed_bytes,
                max_entries,
                max_depth,
                max_path_bytes,
            })
            .ok_or(ArchiveInspectionPolicyErrorV1::InvalidLimits)
    }

    #[must_use]
    pub const fn max_archive_bytes(self) -> u64 {
        self.max_archive_bytes
    }

    #[must_use]
    pub const fn max_total_uncompressed_bytes(self) -> u64 {
        self.max_total_uncompressed_bytes
    }

    #[must_use]
    pub const fn max_entry_uncompressed_bytes(self) -> u64 {
        self.max_entry_uncompressed_bytes
    }

    #[must_use]
    pub const fn max_entries(self) -> usize {
        self.max_entries
    }

    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    #[must_use]
    pub const fn max_path_bytes(self) -> usize {
        self.max_path_bytes
    }
}

impl Default for ArchiveInspectionLimitsV1 {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_ARCHIVE_BYTES_V1,
            DEFAULT_MAX_TOTAL_UNCOMPRESSED_BYTES_V1,
            DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1,
            ATTACHMENT_ARCHIVE_INSPECTION_MAX_REPORT_ENTRIES_V1,
            DEFAULT_MAX_DEPTH_V1,
            ATTACHMENT_ARCHIVE_INSPECTION_MAX_PATH_BYTES_V1,
        )
        .expect("default archive inspection limits are valid")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionPolicyErrorV1 {
    InvalidLimits,
    ArchiveSizeExceeded,
    EntryCountExceeded,
    EntrySizeExceeded,
    TotalUncompressedSizeExceeded,
    UnsafeEntryPath,
    EntryPathTooLong,
    EntryDepthExceeded,
    DuplicateEntryPath,
    EncryptedEntry,
    NestedArchive,
    UnsupportedEntryType,
    MalformedArchive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveEntryKindV1 {
    File,
    Directory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawArchiveEntryMetadataV1 {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub is_directory: bool,
    pub encrypted: bool,
    pub unix_mode: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntryInspectionV1 {
    pub normalized_path: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub kind: ArchiveEntryKindV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionReportV1 {
    pub entry_count: usize,
    pub total_uncompressed_bytes: u64,
    pub entries: Vec<ArchiveEntryInspectionV1>,
}

pub fn inspect_zip_metadata_v1(
    archive_size: u64,
    raw_entries: Vec<RawArchiveEntryMetadataV1>,
    limits: ArchiveInspectionLimitsV1,
) -> Result<ArchiveInspectionReportV1, ArchiveInspectionPolicyErrorV1> {
    if archive_size > limits.max_archive_bytes() {
        return Err(ArchiveInspectionPolicyErrorV1::ArchiveSizeExceeded);
    }
    if raw_entries.len() > limits.max_entries() {
        return Err(ArchiveInspectionPolicyErrorV1::EntryCountExceeded);
    }

    let mut total_uncompressed_bytes = 0_u64;
    let mut normalized_paths = BTreeSet::new();
    let mut entries = Vec::with_capacity(raw_entries.len());
    for raw in raw_entries {
        if raw.encrypted {
            return Err(ArchiveInspectionPolicyErrorV1::EncryptedEntry);
        }
        let normalized_path = normalize_archive_entry_path_v1(&raw.name, limits)?;
        if is_nested_archive_path_v1(&normalized_path) {
            return Err(ArchiveInspectionPolicyErrorV1::NestedArchive);
        }
        if !normalized_paths.insert(normalized_path.clone()) {
            return Err(ArchiveInspectionPolicyErrorV1::DuplicateEntryPath);
        }
        if raw.uncompressed_size > limits.max_entry_uncompressed_bytes() {
            return Err(ArchiveInspectionPolicyErrorV1::EntrySizeExceeded);
        }
        total_uncompressed_bytes = total_uncompressed_bytes
            .checked_add(raw.uncompressed_size)
            .ok_or(ArchiveInspectionPolicyErrorV1::TotalUncompressedSizeExceeded)?;
        if total_uncompressed_bytes > limits.max_total_uncompressed_bytes() {
            return Err(ArchiveInspectionPolicyErrorV1::TotalUncompressedSizeExceeded);
        }
        let kind = archive_entry_kind_v1(raw.is_directory, raw.unix_mode)?;
        entries.push(ArchiveEntryInspectionV1 {
            normalized_path,
            compressed_size: raw.compressed_size,
            uncompressed_size: raw.uncompressed_size,
            kind,
        });
    }

    Ok(ArchiveInspectionReportV1 {
        entry_count: entries.len(),
        total_uncompressed_bytes,
        entries,
    })
}

fn normalize_archive_entry_path_v1(
    name: &str,
    limits: ArchiveInspectionLimitsV1,
) -> Result<String, ArchiveInspectionPolicyErrorV1> {
    let normalized = name.trim().replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.chars().any(char::is_control)
    {
        return Err(ArchiveInspectionPolicyErrorV1::UnsafeEntryPath);
    }

    let mut parts = Vec::new();
    for part in normalized.split('/') {
        let part = part.trim();
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            return Err(ArchiveInspectionPolicyErrorV1::UnsafeEntryPath);
        }
        parts.push(part);
    }
    if parts.is_empty() || parts.len() > limits.max_depth() {
        return Err(if parts.len() > limits.max_depth() {
            ArchiveInspectionPolicyErrorV1::EntryDepthExceeded
        } else {
            ArchiveInspectionPolicyErrorV1::UnsafeEntryPath
        });
    }
    let path = parts.join("/");
    if path.len() > limits.max_path_bytes() {
        return Err(ArchiveInspectionPolicyErrorV1::EntryPathTooLong);
    }
    Ok(path)
}

fn is_nested_archive_path_v1(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".zip") || lower.ends_with(".rar") || lower.ends_with(".7z")
}

fn archive_entry_kind_v1(
    is_directory: bool,
    unix_mode: Option<u32>,
) -> Result<ArchiveEntryKindV1, ArchiveInspectionPolicyErrorV1> {
    const UNIX_FILE_TYPE_MASK: u32 = 0o170000;
    const UNIX_REGULAR_FILE: u32 = 0o100000;
    const UNIX_DIRECTORY: u32 = 0o040000;

    let Some(mode) = unix_mode else {
        return Ok(if is_directory {
            ArchiveEntryKindV1::Directory
        } else {
            ArchiveEntryKindV1::File
        });
    };
    match mode & UNIX_FILE_TYPE_MASK {
        0 | UNIX_REGULAR_FILE if !is_directory => Ok(ArchiveEntryKindV1::File),
        0 | UNIX_DIRECTORY if is_directory => Ok(ArchiveEntryKindV1::Directory),
        _ => Err(ArchiveInspectionPolicyErrorV1::UnsupportedEntryType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bounded_regular_metadata_and_normalizes_separators() {
        let report = inspect_zip_metadata_v1(
            128,
            vec![
                entry("docs\\report.txt", 10, 20),
                RawArchiveEntryMetadataV1 {
                    name: "empty/".to_owned(),
                    compressed_size: 0,
                    uncompressed_size: 0,
                    is_directory: true,
                    encrypted: false,
                    unix_mode: Some(0o040755),
                },
            ],
            ArchiveInspectionLimitsV1::default(),
        )
        .expect("bounded archive");

        assert_eq!(report.entry_count, 2);
        assert_eq!(report.total_uncompressed_bytes, 20);
        assert_eq!(report.entries[0].normalized_path, "docs/report.txt");
        assert_eq!(report.entries[1].kind, ArchiveEntryKindV1::Directory);
    }

    #[test]
    fn rejects_traversal_drive_absolute_control_and_duplicate_paths() {
        let limits = ArchiveInspectionLimitsV1::default();
        for name in ["../secret", "/absolute", "C:\\secret", "safe/\0name"] {
            assert_eq!(
                inspect_zip_metadata_v1(1, vec![entry(name, 1, 1)], limits),
                Err(ArchiveInspectionPolicyErrorV1::UnsafeEntryPath),
            );
        }
        assert_eq!(
            inspect_zip_metadata_v1(1, vec![entry("a/./b", 1, 1), entry("a/b", 1, 1)], limits,),
            Err(ArchiveInspectionPolicyErrorV1::DuplicateEntryPath),
        );
    }

    #[test]
    fn rejects_nested_encrypted_special_deep_and_bomb_metadata() {
        let limits = ArchiveInspectionLimitsV1::default();
        assert_eq!(
            inspect_zip_metadata_v1(1, vec![entry("nested.zip", 1, 1)], limits),
            Err(ArchiveInspectionPolicyErrorV1::NestedArchive),
        );
        let mut encrypted = entry("safe.txt", 1, 1);
        encrypted.encrypted = true;
        assert_eq!(
            inspect_zip_metadata_v1(1, vec![encrypted], limits),
            Err(ArchiveInspectionPolicyErrorV1::EncryptedEntry),
        );
        let mut symlink = entry("link", 1, 1);
        symlink.unix_mode = Some(0o120777);
        assert_eq!(
            inspect_zip_metadata_v1(1, vec![symlink], limits),
            Err(ArchiveInspectionPolicyErrorV1::UnsupportedEntryType),
        );
        assert_eq!(
            inspect_zip_metadata_v1(1, vec![entry("a/b/c/d", 1, 1)], limits),
            Err(ArchiveInspectionPolicyErrorV1::EntryDepthExceeded),
        );
        assert_eq!(
            inspect_zip_metadata_v1(
                1,
                vec![entry(
                    "large.bin",
                    1,
                    DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1 + 1,
                )],
                limits,
            ),
            Err(ArchiveInspectionPolicyErrorV1::EntrySizeExceeded),
        );
    }

    #[test]
    fn limits_cannot_expand_hard_policy() {
        assert_eq!(
            ArchiveInspectionLimitsV1::new(
                DEFAULT_MAX_ARCHIVE_BYTES_V1 + 1,
                DEFAULT_MAX_TOTAL_UNCOMPRESSED_BYTES_V1,
                DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1,
                ATTACHMENT_ARCHIVE_INSPECTION_MAX_REPORT_ENTRIES_V1,
                DEFAULT_MAX_DEPTH_V1,
                ATTACHMENT_ARCHIVE_INSPECTION_MAX_PATH_BYTES_V1,
            ),
            Err(ArchiveInspectionPolicyErrorV1::InvalidLimits),
        );
    }

    fn entry(
        name: &str,
        compressed_size: u64,
        uncompressed_size: u64,
    ) -> RawArchiveEntryMetadataV1 {
        RawArchiveEntryMetadataV1 {
            name: name.to_owned(),
            compressed_size,
            uncompressed_size,
            is_directory: false,
            encrypted: false,
            unix_mode: Some(0o100644),
        }
    }
}
