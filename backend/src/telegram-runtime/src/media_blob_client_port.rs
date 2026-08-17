//! Exact `client_blob` authorization for TDLib files admitted into Blob storage.

use makosh_runtime_protocol::v1::{
    ModuleClientBlobAuthorizationV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_telegram_api::wire::FileIdQuery;
use makosh_telegram_persistence::TelegramDurablePersistence;
use prost::Message;

use crate::admission::telegram_media_read_contract_reference_v1;

const MODULE_CLIENT_PROTOCOL_MAJOR_V1: u32 = 1;

pub(crate) async fn try_handle(
    bytes: &[u8],
    owns_account: impl FnOnce(&str) -> bool,
    durable: &TelegramDurablePersistence,
) -> Result<Option<Vec<u8>>, MediaBlobClientPortErrorV1> {
    let request =
        ModuleClientRequestV1::decode(bytes).map_err(|_| MediaBlobClientPortErrorV1::Protocol)?;
    if request.contract.as_ref() != Some(&telegram_media_read_contract_reference_v1()) {
        return Ok(None);
    }
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR_V1
        || request.module_id != makosh_telegram_api::client_contract::TELEGRAM_MODULE_ID
        || request.owner_id != makosh_telegram_api::client_contract::TELEGRAM_OWNER_ID
        || request.request_id == 0
        || request.request_payload.is_empty()
        || request.logical_owner_id.trim().is_empty()
    {
        return Err(MediaBlobClientPortErrorV1::Protocol);
    }
    let query = FileIdQuery::decode(request.request_payload.as_slice())
        .map_err(|_| MediaBlobClientPortErrorV1::Protocol)?;
    if !owns_account(&query.account_id) || query.provider_file_id.trim().is_empty() {
        return Ok(Some(module_error(request.request_id, "NOT_FOUND")));
    }
    let file = durable
        .file(&query.account_id, &query.provider_file_id)
        .await
        .map_err(|_| MediaBlobClientPortErrorV1::Unavailable)?;
    let Some(file) = file else {
        return Ok(Some(module_error(request.request_id, "NOT_FOUND")));
    };
    let (Some(reference_id), Some(plaintext_sha256), Some(backup_class)) = (
        file.blob_reference_id,
        file.blob_plaintext_sha256,
        file.blob_backup_class,
    ) else {
        return Ok(Some(module_error(request.request_id, "NOT_FOUND")));
    };
    let declared_size = file.size_bytes.or(file.downloaded_size_bytes).unwrap_or(0);
    if !file.is_downloaded
        || reference_id.len() != 16
        || plaintext_sha256.len() != 32
        || backup_class == 0
        || declared_size == 0
        || declared_size > crate::admission::TELEGRAM_MEDIA_CLIENT_MAX_BYTES_V1
    {
        return Ok(Some(module_error(request.request_id, "NOT_FOUND")));
    }
    Ok(Some(module_response(
        request.request_id,
        ModuleClientBlobAuthorizationV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
            reference_id,
            declared_size,
            expected_plaintext_sha256: plaintext_sha256,
            backup_class,
        }
        .encode_to_vec(),
    )))
}

fn module_response(request_id: u64, response_payload: Vec<u8>) -> Vec<u8> {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec()
}

fn module_error(request_id: u64, error_code: &str) -> Vec<u8> {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
    .encode_to_vec()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MediaBlobClientPortErrorV1 {
    Protocol,
    Unavailable,
}
