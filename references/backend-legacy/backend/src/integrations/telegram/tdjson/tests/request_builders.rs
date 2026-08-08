use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;

use crate::integrations::telegram::client::models::qr_login::TelegramQrLoginStartRequest;

#[test]
fn tdlib_parameters_use_legacy_nested_shape_for_tdlib_1_8_runtime() {
    let request = TelegramQrLoginStartRequest {
        account_id: "telegram-qr".to_owned(),
        display_name: "Telegram QR".to_owned(),
        external_account_id: "qr-login:telegram-qr".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
        transcription_enabled: true,
    };

    let command = super::super::requests::set_tdlib_parameters_request(
        &request,
        Path::new("docker/data/telegram/telegram-qr"),
    )
    .expect("TDLib parameters");

    assert_eq!(command["@type"], "setTdlibParameters");
    assert_eq!(command["parameters"]["api_id"], 12345);
    assert_eq!(command["parameters"]["api_hash"], "telegram-api-hash");
    assert_eq!(
        command["parameters"]["database_encryption_key"],
        STANDARD.encode("telegram-session-key")
    );
    assert_eq!(command["database_encryption_key"], serde_json::Value::Null);
}

#[test]
fn tdlib_database_key_check_uses_same_base64_key_without_plaintext_secret() {
    let request = TelegramQrLoginStartRequest {
        account_id: "telegram-qr".to_owned(),
        display_name: "Telegram QR".to_owned(),
        external_account_id: "qr-login:telegram-qr".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
        transcription_enabled: true,
    };

    let command = super::super::requests::check_database_encryption_key_request(&request);

    assert_eq!(command["@type"], "checkDatabaseEncryptionKey");
    assert_eq!(
        command["encryption_key"],
        STANDARD.encode("telegram-session-key")
    );
    assert_ne!(command["encryption_key"], "telegram-session-key");
}

#[test]
fn tdlib_edit_chat_folder_remove_chat_request_preserves_shape_and_excludes_chat() {
    let command = super::super::folder_requests::tdlib_edit_chat_folder_remove_chat_request(
        7,
        222,
        &json!({
            "@type": "chatFolder",
            "name": {"@type": "chatFolderName", "text": "Projects", "animate_custom_emoji": false},
            "icon": {"@type": "chatFolderIcon", "name": "Custom"},
            "color_id": 3,
            "is_shareable": false,
            "pinned_chat_ids": [111, 222],
            "included_chat_ids": [222, 333],
            "excluded_chat_ids": [444],
            "exclude_muted": false,
            "exclude_read": true,
            "exclude_archived": false,
            "include_contacts": true,
            "include_non_contacts": false,
            "include_bots": false,
            "include_groups": true,
            "include_channels": true
        }),
        "makosh-folder-remove-1",
    )
    .expect("folder remove request");

    assert_eq!(command["@type"], "editChatFolder");
    assert_eq!(command["folder"]["pinned_chat_ids"], json!([111]));
    assert_eq!(command["folder"]["included_chat_ids"], json!([333]));
    assert_eq!(command["folder"]["excluded_chat_ids"], json!([444, 222]));
}
