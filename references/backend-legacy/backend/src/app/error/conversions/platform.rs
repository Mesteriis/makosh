use super::super::types::ApiError;
use makosh_graph_api::GraphQueryError;

impl From<GraphQueryError> for ApiError {
    fn from(error: GraphQueryError) -> Self {
        tracing::error!(error = %error, "graph read port failed");
        Self::InvalidGraphQuery("graph query failed")
    }
}
use crate::domains::signal_hub::store::SignalHubError;
use crate::platform::audit::errors::ApiAuditError;
use crate::platform::settings::errors::SettingsError;
use crate::vault::errors::HostVaultError;
use makosh_events_api::EventEnvelopeError;
use makosh_events_postgres::errors::EventStoreError;

impl From<EventEnvelopeError> for ApiError {
    fn from(error: EventEnvelopeError) -> Self {
        Self::InvalidEnvelope(error)
    }
}

impl From<EventStoreError> for ApiError {
    fn from(error: EventStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<SettingsError> for ApiError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::SettingNotFound { .. } => Self::SettingNotFound,
            _ => Self::Settings(error),
        }
    }
}

impl From<SignalHubError> for ApiError {
    fn from(error: SignalHubError) -> Self {
        Self::SignalHub(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<HostVaultError> for ApiError {
    fn from(error: HostVaultError) -> Self {
        Self::HostVault(error)
    }
}
