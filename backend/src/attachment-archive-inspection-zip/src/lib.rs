#![forbid(unsafe_code)]

use std::io::Cursor;

use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionLimitsV1, ArchiveInspectionPolicyErrorV1, ArchiveInspectionReportV1,
    RawArchiveEntryMetadataV1, inspect_zip_metadata_v1,
};
use zip::ZipArchive;

pub const PACKAGE: &str = "makosh-attachment-archive-inspection-zip";

pub fn inspect_zip_bytes_v1(
    bytes: &[u8],
    limits: ArchiveInspectionLimitsV1,
) -> Result<ArchiveInspectionReportV1, ArchiveInspectionPolicyErrorV1> {
    let archive_size = u64::try_from(bytes.len())
        .map_err(|_| ArchiveInspectionPolicyErrorV1::ArchiveSizeExceeded)?;
    if archive_size > limits.max_archive_bytes() {
        return Err(ArchiveInspectionPolicyErrorV1::ArchiveSizeExceeded);
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| ArchiveInspectionPolicyErrorV1::MalformedArchive)?;
    if archive.len() > limits.max_entries() {
        return Err(ArchiveInspectionPolicyErrorV1::EntryCountExceeded);
    }

    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|_| ArchiveInspectionPolicyErrorV1::MalformedArchive)?;
        entries.push(RawArchiveEntryMetadataV1 {
            name: file.name().to_owned(),
            compressed_size: file.compressed_size(),
            uncompressed_size: file.size(),
            is_directory: file.is_dir(),
            encrypted: file.encrypted(),
            unix_mode: file.unix_mode(),
        });
    }
    inspect_zip_metadata_v1(archive_size, entries, limits)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use zip::{ZipWriter, write::SimpleFileOptions};

    #[test]
    fn reads_real_zip_central_directory_without_extracting_entries() {
        let bytes = zip_bytes(&[("docs/report.txt", b"hello")]);
        let report =
            inspect_zip_bytes_v1(&bytes, ArchiveInspectionLimitsV1::default()).expect("real ZIP");

        assert_eq!(report.entry_count, 1);
        assert_eq!(report.total_uncompressed_bytes, 5);
        assert_eq!(report.entries[0].normalized_path, "docs/report.txt");
    }

    #[test]
    fn real_zip_nested_path_and_malformed_input_fail_closed() {
        let nested = zip_bytes(&[("../escape.txt", b"no")]);
        assert_eq!(
            inspect_zip_bytes_v1(&nested, ArchiveInspectionLimitsV1::default()),
            Err(ArchiveInspectionPolicyErrorV1::UnsafeEntryPath),
        );
        assert_eq!(
            inspect_zip_bytes_v1(b"not a zip", ArchiveInspectionLimitsV1::default()),
            Err(ArchiveInspectionPolicyErrorV1::MalformedArchive),
        );
    }

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .expect("entry");
            writer.write_all(bytes).expect("entry bytes");
        }
        writer.finish().expect("ZIP").into_inner()
    }
}
