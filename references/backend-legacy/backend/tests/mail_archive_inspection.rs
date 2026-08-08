use std::io::{Cursor, Write};

use makosh_hub_backend::domains::communications::archive_inspection::{
    ArchiveInspectionError, ArchiveInspectionLimits, archive_inspection_cache_metadata,
    cached_archive_inspection_report, inspect_zip_bytes,
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }

    writer.finish().unwrap().into_inner()
}

#[test]
fn inspects_safe_zip_metadata_without_extracting() {
    let bytes = zip_bytes(&[("docs/readme.txt", b"hello"), ("invoice.pdf", b"pdf bytes")]);

    let report = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap();

    assert_eq!(report.entry_count, 2);
    assert_eq!(report.total_uncompressed_bytes, 14);
    assert_eq!(report.entries[0].normalized_path, "docs/readme.txt");
    assert_eq!(report.entries[0].uncompressed_size, 5);
    assert!(!report.has_nested_archive);
}

#[test]
fn cached_report_is_bound_to_the_inspected_blob_hash() {
    let bytes = zip_bytes(&[("docs/readme.txt", b"hello")]);
    let report = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap();
    let metadata = serde_json::json!({
        "archive_inspection": archive_inspection_cache_metadata(
            "sha256:current-blob",
            &report,
        )
    });

    assert_eq!(
        cached_archive_inspection_report(&metadata, "sha256:current-blob"),
        Some(report)
    );
    assert_eq!(
        cached_archive_inspection_report(&metadata, "sha256:replaced-blob"),
        None
    );
}

#[test]
fn rejects_zip_entries_with_path_traversal() {
    let bytes = zip_bytes(&[("../secret.txt", b"secret")]);

    let err = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::UnsafeEntryPath { entry_name }
            if entry_name == "../secret.txt"
    ));
}

#[test]
fn rejects_zip_bombs_by_uncompressed_size_limit() {
    let bytes = zip_bytes(&[("large.txt", b"12345")]);
    let limits = ArchiveInspectionLimits {
        max_uncompressed_bytes: 4,
        ..ArchiveInspectionLimits::default()
    };

    let err = inspect_zip_bytes(&bytes, limits).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::UncompressedSizeExceeded { total, limit }
            if total == 5 && limit == 4
    ));
}

#[test]
fn rejects_password_protected_zip_entries_without_extracting() {
    let mut bytes = zip_bytes(&[("protected.txt", b"secret")]);
    mark_first_zip_entry_encrypted(&mut bytes);

    let err = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap_err();

    assert!(
        matches!(err, ArchiveInspectionError::EncryptedArchive),
        "unexpected archive inspection error: {err:?}"
    );
}

#[test]
fn rejects_nested_archives_until_recursive_sandbox_inspection_exists() {
    let nested_zip = zip_bytes(&[("nested.txt", b"nested")]);
    let bytes = zip_bytes(&[("bundle/nested.zip", nested_zip.as_slice())]);

    let err = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::NestedArchiveNotAllowed { entry_name }
            if entry_name == "bundle/nested.zip"
    ));
}

fn mark_first_zip_entry_encrypted(bytes: &mut [u8]) {
    // Bit 0 in both headers denotes traditional ZIP entry encryption.
    set_encryption_flag(bytes, b"PK\x03\x04", 6);
    set_encryption_flag(bytes, b"PK\x01\x02", 8);
}

fn set_encryption_flag(bytes: &mut [u8], signature: &[u8; 4], flag_offset: usize) {
    let offset = bytes
        .windows(signature.len())
        .position(|window| window == signature)
        .expect("ZIP header signature");
    bytes[offset + flag_offset] |= 1;
}
