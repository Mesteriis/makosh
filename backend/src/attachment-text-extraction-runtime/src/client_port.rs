use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1, ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
    wire::{
        AttachmentTextExtractionErrorCodeV1 as WireError,
        AttachmentTextExtractionStateV1 as WireState, AttachmentTextFormatV1 as WireFormat,
        GetAttachmentTextExtractionRequestV1, GetAttachmentTextExtractionResponseV1,
        ReadAttachmentTextRequestV1, ReadAttachmentTextResponseV1,
        StartAttachmentTextExtractionRequestV1, StartAttachmentTextExtractionResponseV1,
    },
};
use makosh_attachment_text_extraction_core::{
    AttachmentTextExtractionErrorV1, AttachmentTextExtractionStateV1, AttachmentTextFormatV1,
    visible_attachment_text_v1,
};
use makosh_attachment_text_extraction_persistence::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    CreateAttachmentTextExtractionRunOutcomeV1, CreateAttachmentTextExtractionRunV1,
    PersistedAttachmentTextExtractionRunV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

use crate::contracts::{command_contract_v1, content_contract_v1, query_contract_v1};

pub(crate) enum ClientDispatchV1 {
    Response(ModuleClientResponseV1),
    ReadText { request_id: u64, run_id: [u8; 16] },
}

pub(crate) async fn dispatch_client_request_v1(
    persistence: &AttachmentTextExtractionPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ClientDispatchV1 {
    let response_request_id = request.request_id;
    if request.protocol_major != 1
        || request.module_id != ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1
        || request.owner_id != ATTACHMENT_TEXT_EXTRACTION_OWNER_V1
        || request.logical_owner_id != logical_owner_id
    {
        return ClientDispatchV1::Response(rejected(response_request_id));
    }
    if request.contract.as_ref() == Some(&command_contract_v1()) {
        let payload = start_payload(
            persistence,
            logical_owner_id,
            &request.request_payload,
            now_unix_millis,
        )
        .await;
        return ClientDispatchV1::Response(success(response_request_id, payload));
    }
    if request.contract.as_ref() == Some(&query_contract_v1()) {
        let payload = get_payload(persistence, logical_owner_id, &request.request_payload).await;
        return ClientDispatchV1::Response(success(response_request_id, payload));
    }
    if request.contract.as_ref() == Some(&content_contract_v1()) {
        let Ok(payload) = ReadAttachmentTextRequestV1::decode(request.request_payload.as_slice())
        else {
            return ClientDispatchV1::Response(success(
                response_request_id,
                read_error(Vec::new(), WireError::InvalidRequest),
            ));
        };
        if payload.protocol_major != ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1 {
            return ClientDispatchV1::Response(success(
                response_request_id,
                read_error(payload.run_id, WireError::InvalidRequest),
            ));
        }
        let Some(run_id) = valid_id16(&payload.run_id) else {
            return ClientDispatchV1::Response(success(
                response_request_id,
                read_error(payload.run_id, WireError::InvalidRequest),
            ));
        };
        return ClientDispatchV1::ReadText {
            request_id: response_request_id,
            run_id,
        };
    }
    ClientDispatchV1::Response(rejected(response_request_id))
}

pub(crate) fn read_text_response_v1(
    request_id: u64,
    run_id: [u8; 16],
    text_utf8: &[u8],
    extracted_size_bytes: u64,
) -> ModuleClientResponseV1 {
    let (visible, visible_truncated) = visible_attachment_text_v1(text_utf8);
    success(
        request_id,
        ReadAttachmentTextResponseV1 {
            run_id: run_id.to_vec(),
            text_utf8: visible.to_vec(),
            extracted_size_bytes,
            visible_truncated,
            error: WireError::Unspecified as i32,
        }
        .encode_to_vec(),
    )
}

pub(crate) fn read_text_error_response_v1(
    request_id: u64,
    run_id: [u8; 16],
    error: WireError,
) -> ModuleClientResponseV1 {
    success(request_id, read_error(run_id.to_vec(), error))
}

async fn start_payload(
    persistence: &AttachmentTextExtractionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartAttachmentTextExtractionRequestV1::decode(payload) else {
        return start_error(WireError::InvalidRequest);
    };
    let Some(operation_id) = valid_id16(&request.operation_id) else {
        return start_error(WireError::InvalidRequest);
    };
    let Some(attachment_anchor_id) = valid_id16(&request.attachment_anchor_id) else {
        return start_error(WireError::InvalidRequest);
    };
    if request.protocol_major != ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1
        || logical_owner_id.is_empty()
        || now_unix_millis <= 0
    {
        return start_error(WireError::InvalidRequest);
    }
    match persistence
        .create_run(&CreateAttachmentTextExtractionRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            operation_id,
            attachment_anchor_id,
            created_at_unix_millis: now_unix_millis,
        })
        .await
    {
        Ok(CreateAttachmentTextExtractionRunOutcomeV1::Created(run))
        | Ok(CreateAttachmentTextExtractionRunOutcomeV1::Replayed(run)) => {
            StartAttachmentTextExtractionResponseV1 {
                run_id: run.request.run_id.to_vec(),
                state: wire_state(run.status.state) as i32,
                error: wire_error(run.status.error) as i32,
            }
            .encode_to_vec()
        }
        Ok(CreateAttachmentTextExtractionRunOutcomeV1::OperationCollision)
        | Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput) => {
            start_error(WireError::InvalidRequest)
        }
        Err(_) => start_error(WireError::Unavailable),
    }
}

async fn get_payload(
    persistence: &AttachmentTextExtractionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetAttachmentTextExtractionRequestV1::decode(payload) else {
        return get_error(Vec::new(), WireError::InvalidRequest);
    };
    let Some(run_id) = valid_id16(&request.run_id) else {
        return get_error(request.run_id, WireError::InvalidRequest);
    };
    if request.protocol_major != ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1 {
        return get_error(request.run_id, WireError::InvalidRequest);
    }
    match persistence.find_run(logical_owner_id, run_id).await {
        Ok(Some(run)) => get_response(run),
        Ok(None) => get_error(request.run_id, WireError::NotFound),
        Err(_) => get_error(request.run_id, WireError::Unavailable),
    }
}

fn get_response(run: PersistedAttachmentTextExtractionRunV1) -> Vec<u8> {
    GetAttachmentTextExtractionResponseV1 {
        run_id: run.request.run_id.to_vec(),
        attachment_anchor_id: run.request.attachment_anchor_id.to_vec(),
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        format: run
            .status
            .format
            .map_or(WireFormat::Unspecified, wire_format) as i32,
        extracted_size_bytes: run.status.extracted_size_bytes,
        extraction_truncated: run.status.extraction_truncated,
        error: wire_error(run.status.error) as i32,
    }
    .encode_to_vec()
}

pub(crate) const fn wire_state(value: AttachmentTextExtractionStateV1) -> WireState {
    match value {
        AttachmentTextExtractionStateV1::Accepted => WireState::Accepted,
        AttachmentTextExtractionStateV1::AwaitingEvidence => WireState::AwaitingEvidence,
        AttachmentTextExtractionStateV1::Extracting => WireState::Extracting,
        AttachmentTextExtractionStateV1::Ready => WireState::Ready,
        AttachmentTextExtractionStateV1::Unsupported => WireState::Unsupported,
        AttachmentTextExtractionStateV1::Rejected => WireState::Rejected,
    }
}

pub(crate) const fn wire_format(value: AttachmentTextFormatV1) -> WireFormat {
    match value {
        AttachmentTextFormatV1::PlainUtf8 => WireFormat::PlainUtf8,
        AttachmentTextFormatV1::Pdf => WireFormat::Pdf,
        AttachmentTextFormatV1::Docx => WireFormat::Docx,
        AttachmentTextFormatV1::Ocr => WireFormat::Ocr,
    }
}

pub(crate) const fn wire_error(value: Option<AttachmentTextExtractionErrorV1>) -> WireError {
    match value {
        None => WireError::Unspecified,
        Some(AttachmentTextExtractionErrorV1::NotSafe) => WireError::NotSafe,
        Some(AttachmentTextExtractionErrorV1::Unsupported) => WireError::Unsupported,
        Some(AttachmentTextExtractionErrorV1::SourceTooLarge) => WireError::SourceTooLarge,
        Some(AttachmentTextExtractionErrorV1::InvalidContent) => WireError::InvalidContent,
        Some(AttachmentTextExtractionErrorV1::ParserUnavailable) => WireError::ParserUnavailable,
        Some(AttachmentTextExtractionErrorV1::ParserFailed) => WireError::ParserFailed,
        Some(AttachmentTextExtractionErrorV1::CustodyRejected) => WireError::CustodyRejected,
        Some(AttachmentTextExtractionErrorV1::Unavailable) => WireError::Unavailable,
    }
}

fn valid_id16(value: &[u8]) -> Option<[u8; 16]> {
    let value: [u8; 16] = value.try_into().ok()?;
    value.iter().any(|byte| *byte != 0).then_some(value)
}

fn success(request_id: u64, response_payload: Vec<u8>) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id,
        response_payload,
        error_code: String::new(),
    }
}

fn rejected(request_id: u64) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id,
        response_payload: Vec::new(),
        error_code: "REJECTED".to_owned(),
    }
}

fn start_error(error: WireError) -> Vec<u8> {
    StartAttachmentTextExtractionResponseV1 {
        run_id: Vec::new(),
        state: WireState::Unspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetAttachmentTextExtractionResponseV1 {
        run_id,
        attachment_anchor_id: Vec::new(),
        state: WireState::Unspecified as i32,
        state_revision: 0,
        format: WireFormat::Unspecified as i32,
        extracted_size_bytes: 0,
        extraction_truncated: false,
        error: error as i32,
    }
    .encode_to_vec()
}

fn read_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    ReadAttachmentTextResponseV1 {
        run_id,
        text_utf8: Vec::new(),
        extracted_size_bytes: 0,
        visible_truncated: false,
        error: error as i32,
    }
    .encode_to_vec()
}
