use serde::{Deserialize, Serialize};

use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppProviderRuntimeShape, WhatsAppRuntimeStatus,
};

use super::whatsapp_capability_catalog::{
    is_whatsapp_provider_write_capability, is_whatsapp_runtime_observe_capability,
    provider_shape_summary_reason, provider_shape_summary_status, whatsapp_capability_rows,
};

// ---------------------------------------------------------------------------
// WhatsApp capability model (unchanged)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WhatsAppCapabilityState {
    Available,
    Blocked,
    Degraded,
    Planned,
    Unsupported,
}

impl WhatsAppCapabilityState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Planned => "planned",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WhatsAppActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    SecretAccess,
}

impl WhatsAppActionClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::SecretAccess => "secret_access",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WhatsappProviderShapeStatus {
    pub(crate) provider_shape: String,
    pub(crate) status: String,
    pub(crate) reason: String,
}

impl WhatsappProviderShapeStatus {
    pub(crate) fn new(
        provider_shape: WhatsAppProviderRuntimeShape,
        status: WhatsAppCapabilityState,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            provider_shape: provider_shape.as_str().to_owned(),
            status: status.as_str().to_owned(),
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WhatsappCapabilityAccountScope {
    pub(crate) account_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: String,
    pub(crate) runtime_kind: String,
    pub(crate) lifecycle_state: String,
    pub(crate) live_runtime_available: bool,
    pub(crate) live_send_available: bool,
    pub(crate) media_download_available: bool,
    pub(crate) media_upload_available: bool,
}

impl WhatsappCapabilityAccountScope {
    fn from_runtime_status(status: &WhatsAppRuntimeStatus) -> Self {
        Self {
            account_id: status.account_id.clone(),
            provider_kind: status.provider_kind.clone(),
            provider_shape: status.provider_shape.clone(),
            runtime_kind: status.runtime_kind.clone(),
            lifecycle_state: status.status.clone(),
            live_runtime_available: status.live_runtime_available,
            live_send_available: status.live_send_available,
            media_download_available: status.media_download_available,
            media_upload_available: status.media_upload_available,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: String,
    pub(crate) provider_shapes: Vec<WhatsappProviderShapeStatus>,
    pub(crate) account_scope: Option<WhatsappCapabilityAccountScope>,
    pub(crate) capabilities: Vec<WhatsappCapabilityStatus>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl WhatsappCapabilitiesResponse {
    pub(crate) fn current(runtime_shape: WhatsAppProviderRuntimeShape) -> Self {
        Self::build(runtime_shape, None)
    }

    pub(crate) fn current_for_account(status: &WhatsAppRuntimeStatus) -> Self {
        Self::build(WhatsAppProviderRuntimeShape::WebCompanion, Some(status))
    }

    fn build(
        runtime_shape: WhatsAppProviderRuntimeShape,
        status: Option<&WhatsAppRuntimeStatus>,
    ) -> Self {
        let account_scope = status.map(WhatsappCapabilityAccountScope::from_runtime_status);
        let runtime_mode = status
            .map(|item| item.runtime_kind.clone())
            .unwrap_or_else(|| "fixture".to_owned());
        let mut response = Self {
            version: "2.0",
            runtime_mode,
            provider_shapes: vec![WhatsappProviderShapeStatus::new(
                WhatsAppProviderRuntimeShape::WebCompanion,
                provider_shape_summary_status(
                    runtime_shape,
                    WhatsAppProviderRuntimeShape::WebCompanion,
                ),
                provider_shape_summary_reason(
                    runtime_shape,
                    WhatsAppProviderRuntimeShape::WebCompanion,
                ),
            )],
            account_scope,
            capabilities: whatsapp_capability_rows(),
            planned_features: vec![
                "live_runtime_execution",
                "live_media_transfer_progress",
                "live_presence_feed",
                "live_call_feed",
                "live_status_feed",
                "manual_smoke_test_checklist",
            ],
            unsupported_features: vec![
                "hidden_web_scraping",
                "bulk_messaging",
                "auto_messaging",
                "auto_dialing",
                "whatsapp_data_fine_tuning",
                "external_headless_browser_automation",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let _provider_shape = scope.provider_shape.as_str();

        for capability in &mut self.capabilities {
            match capability.capability.as_str() {
                "runtime.fixture" if runtime_kind != "fixture" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "This account does not use the fixture-only WhatsApp runtime.".to_owned();
                }
                "sessions.restore" if matches!(lifecycle_state, "created" | "link_required") => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account has not completed WhatsApp session linking yet.".to_owned();
                }
                "sessions.restore" if lifecycle_state == "revoked" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account was revoked and must be relinked before restore.".to_owned();
                }
                "sessions.restore" if lifecycle_state == "removed" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account was removed from Макошь runtime control.".to_owned();
                }
                "auth.qr_link_start" if lifecycle_state == "pair_code_pending" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "Pair-code linking is already pending for this account.".to_owned();
                }
                "auth.pair_code_link_start" if lifecycle_state == "qr_pending" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "QR linking is already pending for this account.".to_owned();
                }
                "media.download"
                    if runtime_kind != "fixture" && !scope.media_download_available =>
                {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Runtime `{runtime_kind}` is not live-enabled yet for WhatsApp media download."
                    );
                }
                capability_name
                    if matches!(capability_name, "media.upload_send" | "media.voice_send")
                        && runtime_kind != "fixture"
                        && !scope.media_upload_available =>
                {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Runtime `{runtime_kind}` is not live-enabled yet for WhatsApp media upload."
                    );
                }
                capability_name if is_whatsapp_provider_write_capability(capability_name) => {
                    if lifecycle_state == "removed" {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason =
                            "Removed accounts cannot execute WhatsApp provider commands."
                                .to_owned();
                    } else if matches!(
                        lifecycle_state,
                        "created"
                            | "link_required"
                            | "qr_pending"
                            | "pair_code_pending"
                            | "revoked"
                    ) {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason = format!(
                            "WhatsApp provider commands are blocked while account state is `{lifecycle_state}`."
                        );
                    } else if runtime_kind != "fixture" && !scope.live_send_available {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason = format!(
                            "Runtime `{runtime_kind}` is not live-enabled yet for WhatsApp provider execution."
                        );
                    }
                }
                capability_name if is_whatsapp_runtime_observe_capability(capability_name) => {
                    if lifecycle_state == "removed" {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason =
                            "Removed accounts do not project runtime evidence anymore.".to_owned();
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilityStatus {
    pub(crate) capability: String,
    pub(crate) category: String,
    pub(crate) status: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_required: bool,
    pub(crate) closure_gate: bool,
    pub(crate) reason: String,
}

impl WhatsappCapabilityStatus {
    pub(crate) fn new(
        capability: &str,
        category: &str,
        status: WhatsAppCapabilityState,
        action_class: WhatsAppActionClass,
        reason: &str,
        confirmation_required: bool,
        closure_gate: bool,
    ) -> Self {
        Self {
            capability: capability.to_owned(),
            category: category.to_owned(),
            status: status.as_str().to_owned(),
            action_class: action_class.as_str().to_owned(),
            confirmation_required,
            closure_gate,
            reason: reason.to_owned(),
        }
    }
}
