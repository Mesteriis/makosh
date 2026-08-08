use crate::domains::communications::archive_inspection::ArchiveInspectionReport;
use makosh_connectrpc_contracts::makosh::communications::v1::{
    ArchiveInspectionEntry as ProtoArchiveInspectionEntry,
    ArchiveInspectionReport as ProtoArchiveInspectionReport,
};
pub(super) fn inspection_report(report: ArchiveInspectionReport) -> ProtoArchiveInspectionReport {
    ProtoArchiveInspectionReport {
        archive_kind: report.archive_kind,
        entry_count: report.entry_count as u32,
        total_uncompressed_bytes: report.total_uncompressed_bytes,
        has_nested_archive: report.has_nested_archive,
        entries: report
            .entries
            .into_iter()
            .map(|entry| ProtoArchiveInspectionEntry {
                name: entry.name,
                normalized_path: entry.normalized_path,
                compressed_size: entry.compressed_size,
                uncompressed_size: entry.uncompressed_size,
                is_dir: entry.is_dir,
                is_nested_archive: entry.is_nested_archive,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}
