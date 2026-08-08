use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountCommandPort, ProviderAccountSecretPurpose,
};
use makosh_communications_api::accounts::{ProviderAccount, ProviderSecretBindingCommandPort};
use makosh_events_api::NewEventEnvelope;
use makosh_provider_telemost::models::{
    TelemostCohost, YandexTelemostCohostPage, YandexTelemostConference,
    YandexTelemostConferencePatchRequest, YandexTelemostConferenceRequest,
};
use makosh_provider_telemost::protocol::{
    YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR,
    sanitize_yandex_telemost_payload, validate_api_base_url, validate_json_object,
    validate_required, yandex_telemost_oauth_token_secret_ref,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::secrets::models::{SecretKind, SecretStoreKind};
use crate::platform::settings::store::ApplicationSettingsStore;
use crate::vault::HostVault;
use makosh_events_api::EventLogQuery;
use makosh_events_postgres::store::EventStore;

use super::errors::YandexTelemostError;
mod auth;
mod policy;
mod retention;
use super::models::{
    YANDEX_TELEMOST_LIVE_RUNTIME_KIND, YANDEX_TELEMOST_RUNTIME_KIND, YandexTelemostAccount,
    YandexTelemostAccountListResponse, YandexTelemostAccountSetupRequest,
    YandexTelemostAccountSetupResponse, YandexTelemostCapabilityState,
    YandexTelemostRetentionCleanupItem, YandexTelemostRetentionCleanupRequest,
    YandexTelemostRetentionCleanupResponse, YandexTelemostRuntimeStatus, telemost_provider_kind,
    yandex_telemost_capabilities, yandex_telemost_default_config,
};
use auth::store_oauth_token;
use policy::{
    client_for_account, merge_metadata, provider_payload, runtime_status_from_account,
    validate_account_setup_request, validate_conference_patch_request, validate_conference_request,
};
use retention::{
    LocalRecordingRetentionPolicy, local_recording_retention_policy_from_days,
    local_recording_retention_policy_from_manifest, record_retention_cleanup_in_manifest,
    remove_local_file_if_exists, retention_cleanup_candidate_from_event,
};

const YANDEX_TELEMOST_RECORDING_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.yandex_telemost_recording_retention_days";
const YANDEX_TELEMOST_SPEAKER_TIMELINE_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.yandex_telemost_speaker_timeline_retention_days";

#[derive(Clone)]
pub struct YandexTelemostHttpClient {
    http: reqwest::Client,
    base_url: String,
}

impl YandexTelemostHttpClient {
    pub fn new(base_url: Option<&str>) -> Result<Self, YandexTelemostError> {
        Ok(Self {
            http: reqwest::Client::new(),
            base_url: validate_api_base_url(base_url)?,
        })
    }

    pub async fn create_conference(
        &self,
        oauth_token: &str,
        request: &YandexTelemostConferenceRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        validate_conference_request(request)?;
        let payload = provider_payload(request)?;
        let response = self
            .http
            .post(format!("{}/conferences", self.base_url))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn get_conference(
        &self,
        oauth_token: &str,
        conference_id: &str,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        let response = self
            .http
            .get(format!("{}/conferences/{}", self.base_url, conference_id))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn update_conference(
        &self,
        oauth_token: &str,
        conference_id: &str,
        request: &YandexTelemostConferencePatchRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        validate_conference_patch_request(request)?;
        let payload = provider_payload(request)?;
        let response = self
            .http
            .patch(format!("{}/conferences/{}", self.base_url, conference_id))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn list_cohosts(
        &self,
        oauth_token: &str,
        conference_id: &str,
        offset: Option<u32>,
        limit: Option<u16>,
    ) -> Result<YandexTelemostCohostPage, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        let mut request = self
            .http
            .get(format!(
                "{}/conferences/{}/cohosts",
                self.base_url, conference_id
            ))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json");
        if let Some(offset) = offset {
            request = request.query(&[("offset", offset)]);
        }
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<YandexTelemostCohostPage>().await?)
    }
}

#[derive(Clone)]
pub struct YandexTelemostStore {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
}

impl YandexTelemostStore {
    pub fn new(
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            provider_account_store,
            provider_secret_binding_store,
            event_store,
            event_bus,
        }
    }

    pub async fn setup_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &YandexTelemostAccountSetupRequest,
    ) -> Result<YandexTelemostAccountSetupResponse, YandexTelemostError> {
        validate_account_setup_request(request)?;
        let account_id = validate_required("account_id", &request.account_id)?;
        let display_name = validate_required("display_name", &request.display_name)?;
        let external_account_id =
            validate_required("external_account_id", &request.external_account_id)?;
        let api_base_url = validate_api_base_url(request.api_base_url.as_deref())?;
        let token_secret_ref = self
            .store_or_register_oauth_token(secret_store, vault, &account_id, request)
            .await?;
        let config = merge_metadata(
            yandex_telemost_default_config(Some(&token_secret_ref), &api_base_url),
            &request.metadata,
        );
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                account_id.clone(),
                telemost_provider_kind().as_str().to_owned(),
                display_name,
                external_account_id,
                config,
            )
            .await?;
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
                &token_secret_ref,
            ))
            .await?;
        self.publish_account_configured_event(&account, &token_secret_ref)
            .await?;
        self.publish_authorization_event(&account, &token_secret_ref)
            .await?;
        Ok(YandexTelemostAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<YandexTelemostAccountListResponse, YandexTelemostError> {
        let mut items = self
            .provider_account_store
            .list()
            .await?
            .into_iter()
            .filter(|account| account.provider_kind.is_yandex_telemost())
            .map(YandexTelemostAccount::from)
            .filter(|account| include_removed || account.lifecycle_state != "removed")
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(YandexTelemostAccountListResponse { items })
    }

    pub async fn runtime_status(
        &self,
        account_id: &str,
    ) -> Result<YandexTelemostRuntimeStatus, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let authorized = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
            )
            .await?
            .is_some();
        let status = runtime_status_from_account(account.into(), authorized);
        self.publish_runtime_status_event(&status).await?;
        Ok(status)
    }

    pub async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &YandexTelemostRetentionCleanupRequest,
    ) -> Result<YandexTelemostRetentionCleanupResponse, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        request.validate()?;
        let checked_at = Utc::now();
        let mut response = YandexTelemostRetentionCleanupResponse {
            account_id: account.account_id.clone(),
            checked_at,
            audio_files_removed: 0,
            speaker_hint_files_removed: 0,
            bundles_cleaned: 0,
            items: Vec::new(),
        };
        let events = self
            .event_store
            .list_matching(
                EventLogQuery::default()
                    .event_type(yandex_telemost_event_types::LOCAL_RECORDING_COMPLETED)
                    .limit(1000),
            )
            .await?;

        for event in events {
            if response.items.len() >= request.limit() as usize {
                break;
            }
            let Some(candidate) = retention_cleanup_candidate_from_event(
                &event.event.payload,
                event.event.occurred_at,
            ) else {
                continue;
            };
            if candidate.account_id != account.account_id {
                continue;
            }
            let policy = self
                .resolved_local_recording_retention_policy(
                    &candidate.manifest_path,
                    event.event.occurred_at,
                )
                .await?;
            let now = Utc::now();
            let audio_expired = request.remove_audio
                && policy
                    .audio_expires_at
                    .is_some_and(|expires| expires <= now);
            let speaker_expired = request.remove_speaker_hints
                && policy
                    .speaker_hints_expires_at
                    .is_some_and(|expires| expires <= now);
            if !audio_expired && !speaker_expired {
                continue;
            }

            let removed_at = Utc::now();
            let audio_removed = if audio_expired {
                remove_local_file_if_exists(&candidate.audio_path)?
            } else {
                false
            };
            let speaker_jsonl_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.speaker_jsonl_path)?
            } else {
                false
            };
            let speaker_txt_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.speaker_txt_path)?
            } else {
                false
            };
            let speaker_hints_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.bundle_root.join("speaker-hints.jsonl"))?
            } else {
                false
            };
            if !audio_removed
                && !speaker_jsonl_removed
                && !speaker_txt_removed
                && !speaker_hints_removed
            {
                continue;
            }

            record_retention_cleanup_in_manifest(
                &candidate.manifest_path,
                &policy,
                audio_removed,
                speaker_jsonl_removed || speaker_txt_removed || speaker_hints_removed,
                removed_at,
            )?;
            if audio_removed {
                response.audio_files_removed += 1;
            }
            response.speaker_hint_files_removed += [
                speaker_jsonl_removed,
                speaker_txt_removed,
                speaker_hints_removed,
            ]
            .into_iter()
            .filter(|removed| *removed)
            .count();
            response.bundles_cleaned += 1;
            response.items.push(YandexTelemostRetentionCleanupItem {
                bundle_id: candidate.bundle_id,
                conference_id: candidate.conference_id,
                bundle_root: candidate.bundle_root.to_string_lossy().into_owned(),
                audio_removed,
                speaker_jsonl_removed,
                speaker_txt_removed,
                speaker_hints_removed,
                audio_expires_at: policy.audio_expires_at,
                speaker_hints_expires_at: policy.speaker_hints_expires_at,
                removed_at,
            });
        }

        self.publish_retention_cleanup_completed_event(&response)
            .await?;
        Ok(response)
    }

    pub async fn create_conference(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        request: &YandexTelemostConferenceRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let conference = client_for_account(&account)?
            .create_conference(&token, request)
            .await?;
        self.publish_conference_event(
            yandex_telemost_event_types::CONFERENCE_CREATED,
            &account,
            &conference,
            request.metadata.clone(),
            "create_conference",
        )
        .await?;
        Ok(conference)
    }

    pub async fn get_conference(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        conference_id: &str,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let conference = client_for_account(&account)?
            .get_conference(&token, conference_id)
            .await?;
        self.publish_conference_event(
            yandex_telemost_event_types::CONFERENCE_OBSERVED,
            &account,
            &conference,
            json!({ "source": "explicit_read" }),
            "get_conference",
        )
        .await?;
        Ok(conference)
    }

    pub async fn update_conference(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        conference_id: &str,
        request: &YandexTelemostConferencePatchRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let conference = client_for_account(&account)?
            .update_conference(&token, conference_id, request)
            .await?;
        self.publish_conference_event(
            yandex_telemost_event_types::CONFERENCE_UPDATED,
            &account,
            &conference,
            request.metadata.clone(),
            "update_conference",
        )
        .await?;
        Ok(conference)
    }

    pub async fn list_cohosts(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        conference_id: &str,
        offset: Option<u32>,
        limit: Option<u16>,
    ) -> Result<YandexTelemostCohostPage, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let token = self
            .resolve_oauth_token(secret_store, vault, &account.account_id)
            .await?;
        let page = client_for_account(&account)?
            .list_cohosts(&token, conference_id, offset, limit)
            .await?;
        self.publish_cohosts_observed_event(&account, conference_id, &page)
            .await?;
        Ok(page)
    }

    async fn store_or_register_oauth_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
        request: &YandexTelemostAccountSetupRequest,
    ) -> Result<String, YandexTelemostError> {
        if let Some(token) = request
            .oauth_token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let secret_ref = request
                .oauth_token_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| yandex_telemost_oauth_token_secret_ref(account_id));
            store_oauth_token(
                secret_store,
                vault,
                account_id,
                &secret_ref,
                token,
                &request.metadata,
            )
            .await?;
            return Ok(secret_ref);
        }
        let secret_ref = request
            .oauth_token_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| YandexTelemostError::InvalidRequest("oauth_token or oauth_token_ref must be provided for live Yandex Telemost API calls".to_owned()))?;
        let reference = secret_store
            .secret_reference(secret_ref)
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost token secret reference `{secret_ref}` was not found"
                ))
            })?;
        if reference.secret_kind != SecretKind::OauthToken
            || reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "Yandex Telemost token secret reference `{secret_ref}` must be oauth_token in host_vault"
            )));
        }
        let _ = vault.read_secret(secret_ref)?;
        Ok(secret_ref.to_owned())
    }

    async fn telemost_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, YandexTelemostError> {
        let account_id = validate_required("account_id", account_id)?;
        let account = self
            .provider_account_store
            .get(&account_id)
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost account `{account_id}` was not found"
                ))
            })?;
        if !account.provider_kind.is_yandex_telemost() {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "provider account `{account_id}` is not a Yandex Telemost account"
            )));
        }
        Ok(account)
    }

    async fn resolve_oauth_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<String, YandexTelemostError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
            )
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost account `{account_id}` has no oauth token binding"
                ))
            })?;
        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(format!(
                    "Yandex Telemost token secret reference `{}` was not found",
                    binding.secret_ref
                ))
            })?;
        if reference.secret_kind != SecretKind::OauthToken
            || reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(YandexTelemostError::InvalidRequest(format!(
                "Yandex Telemost token secret reference `{}` must be oauth_token in host_vault",
                reference.secret_ref
            )));
        }
        Ok(vault.read_secret(&reference.secret_ref)?)
    }

    async fn publish_account_configured_event(
        &self,
        account: &ProviderAccount,
        token_secret_ref: &str,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-account-{}-{}", account.account_id, Uuid::new_v4()),
            yandex_telemost_event_types::ACCOUNT_CONFIGURED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": account.account_id }),
        )
        .payload(json!({
            "account_id": account.account_id,
            "provider_kind": account.provider_kind.as_str(),
            "token_secret_ref": token_secret_ref,
            "secret_material": "excluded"
        }))
        .provenance(json!({ "origin": "local_setup" }))
        .correlation_id(format!("yandex-telemost:{}", account.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_authorization_event(
        &self,
        account: &ProviderAccount,
        token_secret_ref: &str,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-authorization-{}-{}", account.account_id, Uuid::new_v4()),
            yandex_telemost_event_types::AUTHORIZATION_COMPLETED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": account.account_id }),
        )
        .payload(json!({
            "account_id": account.account_id,
            "provider_kind": account.provider_kind.as_str(),
            "token_secret_ref": token_secret_ref,
            "secret_material": "excluded"
        }))
        .provenance(json!({ "origin": "local_setup" }))
        .correlation_id(format!("yandex-telemost:{}", account.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_runtime_status_event(
        &self,
        status: &YandexTelemostRuntimeStatus,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-runtime-{}-{}", status.account_id, Uuid::new_v4()),
            yandex_telemost_event_types::RUNTIME_STATUS_CHANGED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": status.account_id }),
        )
        .payload(sanitize_yandex_telemost_payload(json!({ "status": status })))
        .provenance(json!({ "origin": "runtime_status_check" }))
        .correlation_id(format!("yandex-telemost:{}", status.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_conference_event(
        &self,
        event_type: &str,
        account: &ProviderAccount,
        conference: &YandexTelemostConference,
        metadata: Value,
        operation: &'static str,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!(
                "yandex-telemost-conference-{}-{}-{}",
                account.account_id,
                conference.id,
                Uuid::new_v4()
            ),
            event_type,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "telemost_conference", "entity_id": conference.id }),
        )
        .payload(sanitize_yandex_telemost_payload(json!({
            "account_id": account.account_id,
            "provider_kind": account.provider_kind.as_str(),
            "conference": conference,
            "metadata": metadata,
            "operation": operation,
        })))
        .provenance(json!({
            "origin": "yandex_telemost_api",
            "api_base_url": account.config.get("api_base_url").and_then(Value::as_str).unwrap_or(YANDEX_TELEMOST_API_BASE_URL)
        }))
        .correlation_id(format!("yandex-telemost:{}:{}", account.account_id, conference.id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_cohosts_observed_event(
        &self,
        account: &ProviderAccount,
        conference_id: &str,
        page: &YandexTelemostCohostPage,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!("yandex-telemost-cohosts-{}-{}-{}", account.account_id, conference_id, Uuid::new_v4()),
            yandex_telemost_event_types::COHOSTS_OBSERVED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "telemost_conference", "entity_id": conference_id }),
        )
        .payload(sanitize_yandex_telemost_payload(json!({
            "account_id": account.account_id,
            "conference_id": conference_id,
            "cohosts": page.cohosts,
        })))
        .provenance(json!({ "origin": "yandex_telemost_api" }))
        .correlation_id(format!("yandex-telemost:{}:{}", account.account_id, conference_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn publish_retention_cleanup_completed_event(
        &self,
        result: &YandexTelemostRetentionCleanupResponse,
    ) -> Result<(), YandexTelemostError> {
        let event = NewEventEnvelope::builder(
            format!(
                "yandex-telemost-retention-cleanup-{}-{}",
                result.account_id,
                Uuid::new_v4()
            ),
            yandex_telemost_event_types::RETENTION_CLEANUP_COMPLETED,
            result.checked_at,
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "provider_account", "entity_id": result.account_id }),
        )
        .payload(serde_json::to_value(result)?)
        .provenance(json!({ "origin": "local_retention_cleanup" }))
        .correlation_id(format!("yandex-telemost-retention:{}", result.account_id))
        .build()?;
        self.append_and_broadcast(&event).await
    }

    async fn resolved_local_recording_retention_policy(
        &self,
        manifest_path: &Path,
        observed_at: DateTime<Utc>,
    ) -> Result<LocalRecordingRetentionPolicy, YandexTelemostError> {
        if let Ok(content) = fs::read_to_string(manifest_path)
            && let Ok(manifest) = serde_json::from_str::<Value>(&content)
            && let Some(policy) = local_recording_retention_policy_from_manifest(&manifest)
        {
            return Ok(policy);
        }

        let settings = ApplicationSettingsStore::new(self.event_store.pool().clone());
        settings.repair_declared_settings().await?;
        let recording_retention_days = settings
            .setting(YANDEX_TELEMOST_RECORDING_RETENTION_DAYS_SETTING_KEY)
            .await?
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0)
            .max(0);
        let speaker_hint_retention_days = settings
            .setting(YANDEX_TELEMOST_SPEAKER_TIMELINE_RETENTION_DAYS_SETTING_KEY)
            .await?
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0)
            .max(0);
        Ok(local_recording_retention_policy_from_days(
            observed_at,
            recording_retention_days,
            speaker_hint_retention_days,
        ))
    }

    async fn append_and_broadcast(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<(), YandexTelemostError> {
        if self
            .event_store
            .append_for_dispatch_idempotent(event)
            .await?
            .is_some()
        {
            self.event_bus.broadcast(event.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_payload_drops_local_metadata() {
        let request = YandexTelemostConferenceRequest {
            waiting_room_level: Some("all".to_owned()),
            live_stream: None,
            cohosts: vec![],
            is_auto_summarization_enabled: Some(true),
            metadata: json!({ "local": true }),
        };
        let payload = provider_payload(&request).unwrap();
        assert!(payload.get("metadata").is_none());
        assert_eq!(payload["waiting_room_level"], "all");
    }
}
