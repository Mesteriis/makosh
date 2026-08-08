use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use super::store::{
    SignalHubError, SignalHubStore, SignalProfile, SignalProfileCreate, SignalProfileSummary,
    SignalProfileUpdate,
};
use crate::platform::settings::store::ApplicationSettingsStore;
use makosh_events_postgres::store::EventStore;

const ACTIVE_PROFILE_SETTING_KEY: &str = "signal_hub.active_profile";

#[derive(Clone)]
pub struct SignalHubProfileService {
    signal_store: SignalHubStore,
    settings_store: ApplicationSettingsStore,
    event_store: EventStore,
}

impl SignalHubProfileService {
    pub fn new(
        signal_store: SignalHubStore,
        settings_store: ApplicationSettingsStore,
        event_store: EventStore,
    ) -> Self {
        Self {
            signal_store,
            settings_store,
            event_store,
        }
    }

    pub async fn list_profiles(&self) -> Result<Vec<SignalProfileSummary>, SignalHubError> {
        let active_profile_code = self.active_profile_code().await?;
        let profiles = self.signal_store.list_profiles().await?;
        Ok(profiles
            .into_iter()
            .map(|profile| SignalProfileSummary {
                id: profile.id,
                code: profile.code.clone(),
                display_name: profile.display_name,
                description: profile.description,
                policy_count: profile.source_policies.len(),
                source_policies: profile.source_policies,
                is_system: profile.is_system,
                is_active: active_profile_code.as_deref() == Some(profile.code.as_str()),
                created_at: profile.created_at,
                updated_at: profile.updated_at,
            })
            .collect())
    }

    pub async fn create_profile(
        &self,
        request: &SignalProfileCreate,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile = self.signal_store.create_profile(request).await?;
        self.append_profile_event("signal.profile.created", &profile)
            .await?;
        self.summary_for_code(&profile.code).await
    }

    pub async fn update_profile(
        &self,
        request: &SignalProfileUpdate,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile = self.signal_store.update_profile(request).await?;
        self.append_profile_event("signal.profile.updated", &profile)
            .await?;
        self.summary_for_code(&profile.code).await
    }

    pub async fn remove_profile(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile_code = profile_code.trim();
        if profile_code.is_empty() {
            return Err(SignalHubError::EmptyField("profile_code"));
        }

        self.settings_store.repair_declared_settings().await?;
        let active_profile_code = self.active_profile_code().await?;
        let removed = self.signal_store.delete_profile(profile_code).await?;

        if active_profile_code.as_deref() == Some(removed.code.as_str()) {
            self.settings_store
                .update_setting_value(
                    ACTIVE_PROFILE_SETTING_KEY,
                    &json!("production"),
                    "makosh-frontend",
                )
                .await?;
        }

        self.append_profile_event("signal.profile.removed", &removed)
            .await?;

        Ok(SignalProfileSummary {
            id: removed.id,
            code: removed.code,
            display_name: removed.display_name,
            description: removed.description,
            policy_count: removed.source_policies.len(),
            source_policies: removed.source_policies,
            is_system: removed.is_system,
            is_active: false,
            created_at: removed.created_at,
            updated_at: removed.updated_at,
        })
    }

    pub async fn apply_profile(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile_code = profile_code.trim();
        if profile_code.is_empty() {
            return Err(SignalHubError::EmptyField("profile_code"));
        }

        let profile = self
            .signal_store
            .profile_by_code(profile_code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(profile_code.to_owned()))?;

        self.settings_store.repair_declared_settings().await?;
        self.signal_store.expire_managed_profile_policies().await?;
        for policy in &profile.source_policies {
            self.signal_store
                .create_profile_managed_policy(&profile.code, policy)
                .await?;
        }
        self.settings_store
            .update_setting_value(
                ACTIVE_PROFILE_SETTING_KEY,
                &json!(profile.code),
                "makosh-frontend",
            )
            .await?;
        self.append_profile_event("signal.profile.applied", &profile)
            .await?;

        self.summary_for_code(&profile.code).await
    }

    async fn active_profile_code(&self) -> Result<Option<String>, SignalHubError> {
        self.settings_store.repair_declared_settings().await?;
        Ok(self
            .settings_store
            .setting(ACTIVE_PROFILE_SETTING_KEY)
            .await?
            .and_then(|setting| setting.value.as_str().map(ToOwned::to_owned)))
    }

    async fn append_profile_event(
        &self,
        event_type: &str,
        profile: &SignalProfile,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_{}_{}_{}",
                event_type.replace('.', "_"),
                profile.code,
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
            event_type,
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": "system",
                "source_id": profile.code,
            }),
            json!({
                "kind": "signal_profile",
                "entity_id": profile.code,
                "profile_code": profile.code,
            }),
        )
        .payload(json!({
            "profile_code": profile.code,
            "policy_count": profile.source_policies.len(),
        }))
        .provenance(json!({
            "source": "signal_hub_profile_service",
            "profile_code": profile.code,
        }))
        .build()?;
        self.event_store.append_for_dispatch(&event).await?;
        Ok(())
    }

    async fn summary_for_code(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        self.list_profiles()
            .await?
            .into_iter()
            .find(|item| item.code == profile_code)
            .ok_or_else(|| SignalHubError::ProfileNotFound(profile_code.to_owned()))
    }
}
