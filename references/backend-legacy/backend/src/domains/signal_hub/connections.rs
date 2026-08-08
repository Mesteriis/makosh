use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::Value;
use serde_json::json;
use uuid::Uuid;

use super::store::{
    SignalConnection, SignalConnectionCreate, SignalConnectionUpdate, SignalHubError,
    SignalHubStore,
};
use makosh_events_postgres::store::EventStore;
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};

#[derive(Clone)]
pub struct SignalHubConnectionService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubConnectionService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn create_connection(
        &self,
        request: &SignalConnectionCreate,
    ) -> Result<SignalConnection, SignalHubError> {
        let connection = self.signal_store.create_connection(request).await?;
        let sync = self.reconcile_operator_status(&connection).await?;
        self.append_connection_event(
            "signal.connection.created",
            &connection,
            sync.cleared_count,
            sync.applied_mode,
        )
        .await?;
        Ok(connection)
    }

    pub async fn update_connection(
        &self,
        request: &SignalConnectionUpdate,
    ) -> Result<SignalConnection, SignalHubError> {
        let connection = self.signal_store.update_connection(request).await?;
        let sync = self.reconcile_operator_status(&connection).await?;
        self.append_connection_event(
            "signal.connection.updated",
            &connection,
            sync.cleared_count,
            sync.applied_mode,
        )
        .await?;
        Ok(connection)
    }

    pub async fn remove_connection(
        &self,
        connection_id: &str,
    ) -> Result<SignalConnection, SignalHubError> {
        let connection = self.signal_store.remove_connection(connection_id).await?;
        let sync = self.reconcile_operator_status(&connection).await?;
        self.append_connection_event(
            "signal.connection.removed",
            &connection,
            sync.cleared_count,
            sync.applied_mode,
        )
        .await?;
        Ok(connection)
    }

    pub async fn upsert_account_connection(
        &self,
        source_code: &str,
        account_id: &str,
        display_name: &str,
        status: &str,
        settings: Value,
        secret_ref: Option<String>,
    ) -> Result<SignalConnection, SignalHubError> {
        if let Some(existing) = self
            .signal_store
            .find_connection_by_account(source_code, account_id)
            .await?
        {
            return self
                .update_connection(&SignalConnectionUpdate {
                    id: existing.id,
                    display_name: Some(display_name.to_owned()),
                    status: Some(status.to_owned()),
                    profile: None,
                    settings: Some(settings),
                    secret_ref,
                })
                .await;
        }

        self.create_connection(&SignalConnectionCreate {
            source_code: source_code.to_owned(),
            display_name: display_name.to_owned(),
            status: status.to_owned(),
            profile: None,
            settings,
            secret_ref,
        })
        .await
    }

    pub async fn remove_account_connection(
        &self,
        source_code: &str,
        account_id: &str,
    ) -> Result<Option<SignalConnection>, SignalHubError> {
        let Some(existing) = self
            .signal_store
            .find_connection_by_account(source_code, account_id)
            .await?
        else {
            return Ok(None);
        };

        self.remove_connection(&existing.id).await.map(Some)
    }

    async fn reconcile_operator_status(
        &self,
        connection: &SignalConnection,
    ) -> Result<ConnectionPolicySync, SignalHubError> {
        let selector = SignalPolicy {
            scope: SignalPolicyScope::Connection,
            source_code: Some(connection.source_code.clone()),
            connection_id: Some(connection.id.clone()),
            event_pattern: None,
            mode: SignalPolicyMode::Enabled,
            reason: format!("connection status {}", connection.status),
            expires_at: None,
        };
        let cleared_count = self
            .signal_store
            .expire_matching_policies(
                &selector,
                &[
                    SignalPolicyMode::Disabled,
                    SignalPolicyMode::Paused,
                    SignalPolicyMode::Muted,
                ],
            )
            .await?;
        let applied_mode = connection_operator_mode(connection.status.as_str());
        if let Some(mode) = applied_mode.clone() {
            self.signal_store
                .create_policy(&SignalPolicy { mode, ..selector })
                .await?;
        }

        Ok(ConnectionPolicySync {
            cleared_count,
            applied_mode,
        })
    }

    async fn append_connection_event(
        &self,
        event_type: &str,
        connection: &SignalConnection,
        cleared_count: u64,
        applied_mode: Option<SignalPolicyMode>,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_{}_{}_{}",
                event_type.replace('.', "_"),
                connection.id,
                Uuid::now_v7()
            ),
            event_type,
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": connection.source_code,
                "source_id": connection.id,
            }),
            json!({
                "kind": "signal_connection",
                "entity_id": connection.id,
                "source_code": connection.source_code,
                "connection_id": connection.id,
            }),
        )
        .payload(json!({
            "display_name": connection.display_name,
            "status": connection.status,
            "profile": connection.profile,
            "cleared_operator_policy_count": cleared_count,
            "applied_operator_policy_mode": applied_mode.as_ref().map(SignalPolicyMode::as_str),
        }))
        .provenance(json!({
            "source": "signal_hub_connection_service",
            "source_code": connection.source_code,
            "connection_id": connection.id,
        }))
        .correlation_id(&connection.id)
        .build()?;
        self.event_store
            .append_for_dispatch_idempotent(&event)
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
struct ConnectionPolicySync {
    cleared_count: u64,
    applied_mode: Option<SignalPolicyMode>,
}

fn connection_operator_mode(status: &str) -> Option<SignalPolicyMode> {
    match status.trim() {
        "disabled" => Some(SignalPolicyMode::Disabled),
        "paused" => Some(SignalPolicyMode::Paused),
        "muted" => Some(SignalPolicyMode::Muted),
        _ => None,
    }
}
