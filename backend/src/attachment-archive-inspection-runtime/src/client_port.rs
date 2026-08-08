use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1, ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
    wire::{
        ArchiveEntryKindV1 as WireEntryKind, ArchiveEntryV1,
        ArchiveInspectionErrorCodeV1 as WireError, ArchiveInspectionReportV1 as WireReport,
        ArchiveInspectionStateV1 as WireState, ArchiveKindV1 as WireArchiveKind,
        GetArchiveInspectionRequestV1, GetArchiveInspectionResponseV1,
        StartArchiveInspectionRequestV1, StartArchiveInspectionResponseV1,
    },
};
use makosh_attachment_archive_inspection_core::{
    ArchiveEntryKindV1, ArchiveInspectionErrorV1, ArchiveInspectionReportV1,
    ArchiveInspectionStateV1,
};
use makosh_attachment_archive_inspection_persistence::{
    ArchiveInspectionPersistenceErrorV1, AttachmentArchiveInspectionPersistenceV1,
    CreateArchiveInspectionRunOutcomeV1, CreateArchiveInspectionRunV1,
    PersistedArchiveInspectionRunV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

use crate::contracts::{
    archive_inspection_command_contract_v1, archive_inspection_query_contract_v1,
};

pub(crate) async fn dispatch_archive_inspection_client_request_v1(
    persistence: &AttachmentArchiveInspectionPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let valid_identity = request.protocol_major == 1
        && request.module_id == ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1
        && request.owner_id == ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1;
    let (response_payload, accepted_route) = if valid_identity {
        if request.contract.as_ref() == Some(&archive_inspection_command_contract_v1()) {
            (
                start_archive_inspection_payload_v1(
                    persistence,
                    logical_owner_id,
                    &request.request_payload,
                    now_unix_millis,
                )
                .await,
                true,
            )
        } else if request.contract.as_ref() == Some(&archive_inspection_query_contract_v1()) {
            (
                get_archive_inspection_payload_v1(
                    persistence,
                    logical_owner_id,
                    &request.request_payload,
                )
                .await,
                true,
            )
        } else {
            (Vec::new(), false)
        }
    } else {
        (Vec::new(), false)
    };
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id: request.request_id,
        response_payload,
        error_code: if accepted_route {
            String::new()
        } else {
            "REJECTED".to_owned()
        },
    }
}

async fn start_archive_inspection_payload_v1(
    persistence: &AttachmentArchiveInspectionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartArchiveInspectionRequestV1::decode(payload) else {
        return start_error(Vec::new(), WireError::InvalidRequest);
    };
    let response_operation_id = request.operation_id.clone();
    let Some(create) = create_run(logical_owner_id, &request, now_unix_millis) else {
        return start_error(response_operation_id, WireError::InvalidRequest);
    };
    match persistence.create_run(&create).await {
        Ok(CreateArchiveInspectionRunOutcomeV1::Created(run))
        | Ok(CreateArchiveInspectionRunOutcomeV1::Replayed(run)) => {
            StartArchiveInspectionResponseV1 {
                run_id: run.request.run_id.to_vec(),
                state: wire_state(run.status.state) as i32,
                error: wire_error(run.status.error) as i32,
            }
            .encode_to_vec()
        }
        Ok(CreateArchiveInspectionRunOutcomeV1::OperationCollision)
        | Err(ArchiveInspectionPersistenceErrorV1::InvalidInput) => {
            start_error(response_operation_id, WireError::InvalidRequest)
        }
        Err(_) => start_error(response_operation_id, WireError::Unavailable),
    }
}

async fn get_archive_inspection_payload_v1(
    persistence: &AttachmentArchiveInspectionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetArchiveInspectionRequestV1::decode(payload) else {
        return get_error(Vec::new(), WireError::InvalidRequest);
    };
    let response_run_id = request.run_id.clone();
    let Some(run_id) = valid_id16(&request.run_id) else {
        return get_error(response_run_id, WireError::InvalidRequest);
    };
    if request.protocol_major != ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1 {
        return get_error(response_run_id, WireError::InvalidRequest);
    }
    match persistence.load_run(logical_owner_id, run_id).await {
        Ok(Some(run)) => {
            get_response(run).unwrap_or_else(|| get_error(response_run_id, WireError::Unavailable))
        }
        Ok(None) => get_error(response_run_id, WireError::NotFound),
        Err(_) => get_error(response_run_id, WireError::Unavailable),
    }
}

fn create_run(
    logical_owner_id: &str,
    request: &StartArchiveInspectionRequestV1,
    now_unix_millis: i64,
) -> Option<CreateArchiveInspectionRunV1> {
    if request.protocol_major != ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1
        || logical_owner_id.is_empty()
        || now_unix_millis <= 0
    {
        return None;
    }
    Some(CreateArchiveInspectionRunV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id: valid_id16(&request.operation_id)?,
        attachment_anchor_id: valid_id16(&request.attachment_anchor_id)?,
        created_at_unix_millis: now_unix_millis,
    })
}

fn get_response(run: PersistedArchiveInspectionRunV1) -> Option<Vec<u8>> {
    let report = match run.status.report {
        Some(report) => Some(wire_report(report)?),
        None => None,
    };
    Some(
        GetArchiveInspectionResponseV1 {
            run_id: run.request.run_id.to_vec(),
            attachment_anchor_id: run.request.attachment_anchor_id.to_vec(),
            state: wire_state(run.status.state) as i32,
            state_revision: run.status.state_revision,
            report,
            error: wire_error(run.status.error) as i32,
        }
        .encode_to_vec(),
    )
}

fn wire_report(report: ArchiveInspectionReportV1) -> Option<WireReport> {
    let entry_count = u32::try_from(report.entry_count).ok()?;
    let entries = report
        .entries
        .into_iter()
        .map(|entry| ArchiveEntryV1 {
            normalized_path_utf8: entry.normalized_path.into_bytes(),
            compressed_size: entry.compressed_size,
            uncompressed_size: entry.uncompressed_size,
            entry_kind: match entry.kind {
                ArchiveEntryKindV1::File => WireEntryKind::File as i32,
                ArchiveEntryKindV1::Directory => WireEntryKind::Directory as i32,
            },
        })
        .collect();
    Some(WireReport {
        archive_kind: WireArchiveKind::Zip as i32,
        entry_count,
        total_uncompressed_bytes: report.total_uncompressed_bytes,
        entries,
    })
}

pub(crate) const fn wire_state(value: ArchiveInspectionStateV1) -> WireState {
    match value {
        ArchiveInspectionStateV1::Accepted => WireState::Accepted,
        ArchiveInspectionStateV1::AwaitingEvidence => WireState::AwaitingEvidence,
        ArchiveInspectionStateV1::Inspecting => WireState::Inspecting,
        ArchiveInspectionStateV1::Ready => WireState::Ready,
        ArchiveInspectionStateV1::Rejected => WireState::Rejected,
    }
}

pub(crate) const fn wire_error(value: Option<ArchiveInspectionErrorV1>) -> WireError {
    match value {
        None => WireError::Unspecified,
        Some(ArchiveInspectionErrorV1::NotSafe) => WireError::NotSafe,
        Some(ArchiveInspectionErrorV1::NotZip) => WireError::NotZip,
        Some(ArchiveInspectionErrorV1::PolicyRejected) => WireError::PolicyRejected,
        Some(ArchiveInspectionErrorV1::CorruptArchive) => WireError::CorruptArchive,
        Some(ArchiveInspectionErrorV1::Unavailable) => WireError::Unavailable,
    }
}

fn valid_id16(value: &[u8]) -> Option<[u8; 16]> {
    let value: [u8; 16] = value.try_into().ok()?;
    value.iter().any(|byte| *byte != 0).then_some(value)
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartArchiveInspectionResponseV1 {
        run_id,
        state: WireState::Unspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetArchiveInspectionResponseV1 {
        run_id,
        attachment_anchor_id: Vec::new(),
        state: WireState::Unspecified as i32,
        state_revision: 0,
        report: None,
        error: error as i32,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_request_is_owner_local_and_requires_exact_ids() {
        let request = StartArchiveInspectionRequestV1 {
            protocol_major: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1,
            operation_id: vec![1; 16],
            attachment_anchor_id: vec![2; 16],
        };
        let create = create_run("owner-1", &request, 1_000).expect("create");
        assert_eq!(create.logical_owner_id, "owner-1");
        assert_eq!(create.operation_id, [1; 16]);
        assert_eq!(create.attachment_anchor_id, [2; 16]);
        assert!(create_run("", &request, 1_000).is_none());
        assert!(create_run("owner-1", &request, 0).is_none());
    }

    #[test]
    fn wire_report_contains_only_bounded_derived_metadata() {
        let report = wire_report(ArchiveInspectionReportV1 {
            entry_count: 1,
            total_uncompressed_bytes: 3,
            entries: vec![
                makosh_attachment_archive_inspection_core::ArchiveEntryInspectionV1 {
                    normalized_path: "folder/file.txt".to_owned(),
                    compressed_size: 2,
                    uncompressed_size: 3,
                    kind: ArchiveEntryKindV1::File,
                },
            ],
        })
        .expect("report");
        assert_eq!(report.archive_kind, WireArchiveKind::Zip as i32);
        assert_eq!(report.entries[0].normalized_path_utf8, b"folder/file.txt");
    }
}
