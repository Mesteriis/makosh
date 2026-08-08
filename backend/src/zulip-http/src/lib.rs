//! Zulip REST protocol boundary. Credentials never implement `Debug`.

mod command;
mod event_queue;
mod history;
mod wire;

use std::fmt::{Debug, Formatter};

use makosh_zulip_api::{
    ZulipAccountV1, ZulipCommandV1, ZulipEventQueueV1, ZulipHistoryPageV1, ZulipPolledEventV1,
};
use zeroize::Zeroizing;

pub use command::ZulipHttpRequestV1;
pub use wire::{ZulipHttpErrorV1, ZulipHttpResponseV1};

pub const PACKAGE: &str = "makosh-zulip-http";

#[derive(Clone, Eq, PartialEq)]
pub struct ZulipHttpConfigV1 {
    pub account: ZulipAccountV1,
    pub api_key: Zeroizing<String>,
}

impl Debug for ZulipHttpConfigV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ZulipHttpConfigV1")
            .field("account", &"<redacted>")
            .field("api_key", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipHttpConfigErrorV1 {
    InvalidRealm,
    EmptyCredential,
}

impl ZulipHttpConfigV1 {
    pub fn new(account: ZulipAccountV1, api_key: String) -> Result<Self, ZulipHttpConfigErrorV1> {
        if !valid_realm_url(&account.realm_url) {
            return Err(ZulipHttpConfigErrorV1::InvalidRealm);
        }
        if api_key.trim().is_empty() {
            return Err(ZulipHttpConfigErrorV1::EmptyCredential);
        }
        Ok(Self {
            account,
            api_key: Zeroizing::new(api_key),
        })
    }
}

pub async fn execute_command(
    config: &ZulipHttpConfigV1,
    command: &ZulipCommandV1,
) -> Result<ZulipHttpResponseV1, ZulipHttpErrorV1> {
    let request = command::request_for_command(config, command)?;
    wire::execute(config, request).await
}

pub async fn upload_file(
    config: &ZulipHttpConfigV1,
    filename: &str,
    bytes: &[u8],
) -> Result<String, ZulipHttpErrorV1> {
    let (_, value) = wire::execute_value(
        config,
        command::request_for_upload(config, filename, bytes)?,
    )
    .await?;
    value
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .filter(|uri| is_same_realm_upload_url(&config.account.realm_url, uri))
        .map(str::to_owned)
        .ok_or(ZulipHttpErrorV1::Protocol)
}

pub async fn download_user_upload(
    config: &ZulipHttpConfigV1,
    upload_path: &str,
) -> Result<(Vec<u8>, Option<String>), ZulipHttpErrorV1> {
    wire::execute_binary(
        config,
        command::request_for_user_upload_download(config, upload_path)?,
    )
    .await
}

pub async fn register_event_queue(
    config: &ZulipHttpConfigV1,
) -> Result<ZulipEventQueueV1, ZulipHttpErrorV1> {
    event_queue::register(config).await
}

pub async fn poll_event_queue(
    config: &ZulipHttpConfigV1,
    queue: &ZulipEventQueueV1,
) -> Result<Vec<ZulipPolledEventV1>, ZulipHttpErrorV1> {
    event_queue::poll(config, queue).await
}

pub async fn fetch_message_history_page(
    config: &ZulipHttpConfigV1,
    before_provider_message_id: Option<&str>,
    limit: u32,
) -> Result<ZulipHistoryPageV1, ZulipHttpErrorV1> {
    history::fetch_page(config, before_provider_message_id, limit).await
}

#[must_use]
pub fn is_same_realm_upload_url(realm_url: &str, upload_url: &str) -> bool {
    realm_url.starts_with("https://")
        && upload_url.starts_with(realm_url)
        && upload_url[realm_url.len()..].starts_with("user_uploads/")
}

fn valid_realm_url(realm_url: &str) -> bool {
    realm_url.starts_with("https://")
        && !realm_url.contains(['?', '#', '@', '\r', '\n', '\0'])
        && realm_url["https://".len()..]
            .split('/')
            .next()
            .is_some_and(|authority| !authority.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_zulip_api::{ZulipCommandV1, ZulipReactionOperationV1, ZulipReactionV1};

    fn config() -> ZulipHttpConfigV1 {
        ZulipHttpConfigV1::new(
            ZulipAccountV1 {
                account_id: "a".into(),
                realm_url: "https://zulip.test/".into(),
                account_email: "account@zulip.test".into(),
            },
            "secret".into(),
        )
        .expect("config")
    }

    #[test]
    fn redacts_api_key_and_rejects_other_realm_uploads() {
        let diagnostic = format!("{:?}", config());
        for private_value in ["secret", "https://zulip.test/", "bot@zulip.test"] {
            assert!(!diagnostic.contains(private_value));
        }
        assert!(is_same_realm_upload_url(
            "https://zulip.test/",
            "https://zulip.test/user_uploads/a"
        ));
        assert!(!is_same_realm_upload_url(
            "https://zulip.test/",
            "https://evil.test/user_uploads/a"
        ));
    }

    #[test]
    fn compiles_reaction_identity_without_leaking_credentials() {
        let request = command::request_for_command(
            &config(),
            &ZulipCommandV1::Reaction {
                operation_id: "op".into(),
                account_id: "a".into(),
                provider_message_id: "42".into(),
                reaction: ZulipReactionV1 {
                    emoji_name: "+1".into(),
                    emoji_code: Some("1f44d".into()),
                    reaction_type: Some("unicode_emoji".into()),
                },
                operation: ZulipReactionOperationV1::Add,
            },
        )
        .expect("request");
        assert_eq!(request.method, "POST");
        assert!(request.path.ends_with("/api/v1/messages/42/reactions"));
        assert!(request.form_body.contains("emoji_code=1f44d"));
    }

    #[test]
    fn compiles_bounded_multipart_upload_without_putting_data_in_the_url() {
        let request = command::request_for_upload(&config(), "report.txt", b"private bytes")
            .expect("upload request");
        assert_eq!(request.method, "POST");
        assert!(request.path.ends_with("/api/v1/user_uploads"));
        assert!(request.content_type.starts_with("multipart/form-data"));
        assert!(
            request
                .body
                .windows(b"private bytes".len())
                .any(|window| window == b"private bytes")
        );
        assert!(!request.path.contains("private"));
    }

    #[test]
    fn accepts_only_relative_user_upload_paths_for_binary_download() {
        let request = command::request_for_user_upload_download(&config(), "/user_uploads/a/file")
            .expect("download request");
        assert_eq!(request.path, "/user_uploads/a/file");
        assert!(
            command::request_for_user_upload_download(
                &config(),
                "https://evil.test/user_uploads/a"
            )
            .is_err()
        );
        assert!(
            command::request_for_user_upload_download(&config(), "/user_uploads/a?token=x")
                .is_err()
        );
    }
}
