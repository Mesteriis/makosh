use connectrpc::{ConnectError, ErrorCode};

use super::communications::message_connect_error;
use super::communications::storage_connect_error;

pub(super) fn raw_evidence(
    error: makosh_communications_postgres::errors::CommunicationIngestionError,
) -> ConnectError {
    tracing::error!(error = %error, "communication raw evidence query failed");
    ConnectError::new(
        ErrorCode::Internal,
        "communication raw evidence query failed",
    )
}

pub(super) fn export(
    error: crate::domains::communications::export::CommunicationExportError,
) -> ConnectError {
    match error {
        crate::domains::communications::export::CommunicationExportError::NotFound => {
            ConnectError::new(ErrorCode::NotFound, error.to_string())
        }
        crate::domains::communications::export::CommunicationExportError::MessageProjection(
            err,
        ) => message_connect_error(err),
        crate::domains::communications::export::CommunicationExportError::CommunicationStorage(
            err,
        ) => storage_connect_error(err),
    }
}
