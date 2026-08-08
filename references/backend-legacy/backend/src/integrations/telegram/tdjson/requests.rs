use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::TelegramQrLoginStartRequest;

use super::identifiers::safe_path_segment;

pub(crate) fn set_tdlib_parameters_request(
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<Value, TelegramError> {
    let api_id = request.required_api_id()?;
    let api_hash = request.required_api_hash()?;
    let database_directory = database_directory.to_string_lossy().into_owned();
    let files_directory = Path::new(&database_directory)
        .join("files")
        .to_string_lossy()
        .into_owned();

    let parameters = json!({
        "use_test_dc": false,
        "database_directory": database_directory,
        "files_directory": files_directory,
        "database_encryption_key": tdlib_database_encryption_key(request),
        "use_file_database": true,
        "use_chat_info_database": true,
        "use_message_database": true,
        "use_secret_chats": false,
        "api_id": api_id,
        "api_hash": api_hash,
        "system_language_code": "en",
        "device_model": "Макошь",
        "system_version": std::env::consts::OS,
        "application_version": env!("CARGO_PKG_VERSION"),
        "enable_storage_optimizer": true,
        "ignore_file_names": false
    });

    Ok(json!({
        "@type": "setTdlibParameters",
        "parameters": parameters,
        "@extra": "makosh-set-tdlib-parameters"
    }))
}

pub(crate) fn tdlib_database_directory(request: &TelegramQrLoginStartRequest) -> PathBuf {
    request
        .tdlib_data_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("docker/data/telegram").join(safe_path_segment(&request.account_id))
        })
}

pub(crate) fn check_database_encryption_key_request(
    request: &TelegramQrLoginStartRequest,
) -> Value {
    json!({
        "@type": "checkDatabaseEncryptionKey",
        "encryption_key": tdlib_database_encryption_key(request),
        "@extra": "makosh-check-database-encryption-key"
    })
}

fn tdlib_database_encryption_key(request: &TelegramQrLoginStartRequest) -> String {
    request
        .session_encryption_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| STANDARD.encode(value.as_bytes()))
        .unwrap_or_default()
}
