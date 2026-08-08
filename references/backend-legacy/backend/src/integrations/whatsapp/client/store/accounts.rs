use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::json;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebSession, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCompanionRuntime, WhatsappWebLinkState,
};
use makosh_provider_whatsapp::ids::whatsapp_web_session_id;

impl WhatsappWebStore {
    pub async fn setup_fixture_account(
        &self,
        request: &WhatsappWebAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        if request.provider_kind != CommunicationProviderKind::WhatsappWeb {
            return Err(WhatsappWebError::InvalidRequest(
                "provider_kind must be a WhatsApp provider".to_owned(),
            ));
        }
        let provider_shape = normalize_fixture_provider_shape(
            request.provider_kind,
            request.provider_shape.as_deref(),
        )?;
        let session_mode = fixture_session_mode(request.provider_kind);
        let setup_semantics = fixture_setup_semantics(request.provider_kind);

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "provider_shape": provider_shape,
            "local_state_path": request.local_state_path,
            "device_name": request.device_name,
            "lifecycle_state": "created",
            "setup_semantics": setup_semantics,
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name: request.device_name.clone(),
                companion_runtime: fixture_companion_runtime(request.provider_kind),
                link_state: WhatsappWebLinkState::Fixture,
                local_state_path: request.local_state_path.clone(),
                last_sync_at: None,
                metadata: json!({
                    "runtime": "fixture",
                    "provider_shape": provider_shape,
                    "setup_semantics": setup_semantics,
                    "session_mode": session_mode,
                }),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            session,
        })
    }

    pub async fn setup_live_blocked_account(
        &self,
        request: &WhatsappLiveAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        let provider_shape = normalize_provider_shape(&request.provider_shape)?;
        validate_live_provider_kind(request.provider_kind, provider_shape)?;
        let device_name = default_live_device_name(provider_shape, request.device_name.clone());
        let local_state_path = default_live_local_state_path(
            provider_shape,
            &request.account_id,
            request.local_state_path.clone(),
        );

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "live_blocked",
            "provider_shape": provider_shape,
            "local_state_path": local_state_path,
            "device_name": device_name,
            "lifecycle_state": "created",
            "setup_semantics": live_setup_semantics(provider_shape),
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name,
                companion_runtime: live_companion_runtime(provider_shape),
                link_state: WhatsappWebLinkState::Blocked,
                local_state_path,
                last_sync_at: None,
                metadata: json!({
                    "runtime": "live_blocked",
                    "provider_shape": provider_shape,
                    "setup_semantics": live_setup_semantics(provider_shape),
                    "session_mode": live_session_mode(provider_shape),
                }),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "live_blocked".to_owned(),
            session,
        })
    }
}

fn normalize_provider_shape(input: &str) -> Result<&'static str, WhatsappWebError> {
    let normalized = input.trim();
    if normalized == "whatsapp_web_companion" {
        Ok("whatsapp_web_companion")
    } else {
        Err(WhatsappWebError::InvalidRequest(format!(
            "unsupported WhatsApp provider_shape `{input}`; only whatsapp_web_companion is available"
        )))
    }
}

fn normalize_fixture_provider_shape(
    provider_kind: CommunicationProviderKind,
    requested_shape: Option<&str>,
) -> Result<&'static str, WhatsappWebError> {
    match requested_shape {
        Some(input) => {
            let normalized = normalize_provider_shape(input)?;
            validate_live_provider_kind(provider_kind, normalized)?;
            Ok(normalized)
        }
        None => Ok(fixture_provider_shape(provider_kind)),
    }
}

fn validate_live_provider_kind(
    provider_kind: CommunicationProviderKind,
    provider_shape: &str,
) -> Result<(), WhatsappWebError> {
    let expected_kind = CommunicationProviderKind::WhatsappWeb;
    if provider_kind != expected_kind {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "provider_kind `{}` is invalid for provider_shape `{provider_shape}`; expected `{}`",
            provider_kind.as_str(),
            expected_kind.as_str(),
        )));
    }
    Ok(())
}

fn default_live_device_name(provider_shape: &str, request_value: Option<String>) -> String {
    request_value.unwrap_or_else(|| format!("{provider_shape} hidden WebView runtime"))
}

fn default_live_local_state_path(
    _provider_shape: &str,
    account_id: &str,
    request_value: Option<String>,
) -> String {
    request_value.unwrap_or_else(|| format!("docker/data/whatsapp/webview/{account_id}"))
}

fn live_setup_semantics(_provider_shape: &str) -> &'static str {
    "hidden_webview_runtime"
}

fn live_session_mode(_provider_shape: &str) -> &'static str {
    "device_session"
}

fn live_companion_runtime(_provider_shape: &str) -> WhatsappWebCompanionRuntime {
    WhatsappWebCompanionRuntime::Blocked
}

fn fixture_provider_shape(_provider_kind: CommunicationProviderKind) -> &'static str {
    "whatsapp_web_companion"
}

fn fixture_setup_semantics(_provider_kind: CommunicationProviderKind) -> &'static str {
    "hidden_webview_runtime"
}

fn fixture_session_mode(_provider_kind: CommunicationProviderKind) -> &'static str {
    "device_session"
}

fn fixture_companion_runtime(
    _provider_kind: CommunicationProviderKind,
) -> WhatsappWebCompanionRuntime {
    WhatsappWebCompanionRuntime::Fixture
}
