use axum::Json;
use axum::extract::State;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::{Deserialize, Serialize};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;

const MAX_EML_IMPORT_BYTES: usize = 20 * 1024 * 1024;

#[derive(Deserialize)]
pub(crate) struct EmlImportRequest {
    pub(super) account_id: String,
    pub(super) eml_base64: String,
}

#[derive(Serialize)]
pub(crate) struct EmlImportResponse {
    pub(super) message_id: String,
    pub(super) raw_record_id: String,
    pub(super) attachment_count: usize,
}

#[derive(Deserialize)]
pub(crate) struct MboxImportRequest {
    pub(super) account_id: String,
    pub(super) mbox_base64: String,
}

#[derive(Serialize)]
pub(crate) struct MboxImportResponse {
    pub(super) imported_count: usize,
    pub(super) message_ids: Vec<String>,
    pub(super) failed_count: usize,
    pub(super) failures: Vec<MboxImportFailureResponse>,
}

#[derive(Serialize)]
pub(crate) struct MboxImportFailureResponse {
    pub(super) message_index: usize,
    pub(super) reason: &'static str,
}

pub(crate) async fn post_v1_import_eml(
    State(state): State<AppState>,
    Json(request): Json<EmlImportRequest>,
) -> Result<Json<EmlImportResponse>, ApiError> {
    let account_id = request.account_id.trim();
    if account_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ));
    }
    let raw_eml = BASE64_STANDARD
        .decode(request.eml_base64.trim())
        .map_err(|_| ApiError::InvalidCommunicationQuery("eml_base64 is invalid"))?;
    if raw_eml.is_empty() || raw_eml.len() > MAX_EML_IMPORT_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "EML import exceeds allowed size",
        ));
    }

    let service = eml_import_service(&state)?;
    let imported = service
        .import_eml(account_id, &raw_eml)
        .await
        .map_err(eml_import_api_error)?;
    Ok(Json(EmlImportResponse {
        message_id: imported.message.message_id,
        raw_record_id: imported.raw_record.raw_record_id,
        attachment_count: imported.attachment_count,
    }))
}

pub(crate) async fn post_v1_import_mbox(
    State(state): State<AppState>,
    Json(request): Json<MboxImportRequest>,
) -> Result<Json<MboxImportResponse>, ApiError> {
    let account_id = request.account_id.trim();
    if account_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ));
    }
    let raw_mbox = BASE64_STANDARD
        .decode(request.mbox_base64.trim())
        .map_err(|_| ApiError::InvalidCommunicationQuery("mbox_base64 is invalid"))?;
    if raw_mbox.is_empty() || raw_mbox.len() > MAX_EML_IMPORT_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "MBOX import exceeds allowed size",
        ));
    }

    let imported = eml_import_service(&state)?
        .import_mbox(account_id, &raw_mbox)
        .await
        .map_err(eml_import_api_error)?;
    Ok(Json(MboxImportResponse {
        imported_count: imported.imported.len(),
        message_ids: imported
            .imported
            .into_iter()
            .map(|result| result.message.message_id)
            .collect(),
        failed_count: imported.failures.len(),
        failures: imported
            .failures
            .into_iter()
            .map(|failure| MboxImportFailureResponse {
                message_index: failure.message_index,
                reason: failure.reason,
            })
            .collect(),
    }))
}

fn eml_import_service(
    state: &AppState,
) -> Result<crate::application::eml_import::EmlImportService, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(crate::application::eml_import::EmlImportService::new(
        makosh_communications_postgres::store::CommunicationIngestionStore::new(pool.clone()),
        makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        ),
        crate::domains::communications::messages::store::MessageProjectionStore::new(pool.clone()),
        crate::domains::communications::storage::store::CommunicationStorageStore::new(pool),
        crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore::new(
            crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT,
        ),
    ))
}

fn eml_import_api_error(error: crate::application::eml_import::EmlImportError) -> ApiError {
    match error {
        crate::application::eml_import::EmlImportError::AccountNotFound => ApiError::NotFound,
        crate::application::eml_import::EmlImportError::UnsupportedAccountKind
        | crate::application::eml_import::EmlImportError::Rfc822(_)
        | crate::application::eml_import::EmlImportError::Mbox(_) => {
            ApiError::InvalidCommunicationQuery("mail import payload is invalid")
        }
        crate::application::eml_import::EmlImportError::Ingestion(error) => {
            ApiError::CommunicationIngestion(error)
        }
        crate::application::eml_import::EmlImportError::MessageProjection(error) => {
            ApiError::Messages(error)
        }
        crate::application::eml_import::EmlImportError::Storage(error) => {
            ApiError::CommunicationStorage(error)
        }
        crate::application::eml_import::EmlImportError::Scan(error) => {
            ApiError::FailedPrecondition(error.to_string())
        }
    }
}
