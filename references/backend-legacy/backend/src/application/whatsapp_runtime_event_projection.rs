use chrono::{DateTime, Utc};
use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::accounts::ProviderAccountMutationOrigin;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_events_api::StoredEventEnvelope;
use serde_json::{Map, Value, json};
use sqlx::postgres::PgPool;

use crate::domains::signal_hub::connections::SignalHubConnectionService;
use crate::domains::signal_hub::store::{SignalHubError, SignalHubStore};
use crate::integrations::whatsapp::client::store::WhatsappWebStore;
use crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_communications_postgres::store::CommunicationIngestionStore;

use makosh_events_postgres::store::EventStore;

use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use crate::vault::errors::HostVaultError;

pub(crate) const WHATSAPP_RUNTIME_EVENT_CONSUMER: &str = "whatsapp_runtime_event_projection";

pub(crate) async fn sync_whatsapp_runtime_signal_connection_for_pool(
    pool: &PgPool,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
    secret_ref: Option<String>,
) -> Result<(), SignalHubError> {
    let signal_store = SignalHubStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), EventStore::new(pool.clone()));
    signal_store.restore_system_sources().await?;
    let settings = merged_whatsapp_runtime_connection_settings(
        signal_store
            .find_connection_by_account("whatsapp", &account.account_id)
            .await?
            .as_ref()
            .map(|connection| &connection.settings),
        account,
        status,
    );
    connection_service
        .upsert_account_connection(
            "whatsapp",
            &account.account_id,
            &account.display_name,
            whatsapp_runtime_signal_status(status),
            settings,
            secret_ref,
        )
        .await?;
    Ok(())
}

pub(crate) async fn project_whatsapp_runtime_event(
    pool: PgPool,
    vault: HostVault,
    event: StoredEventEnvelope,
) -> Result<(), String> {
    if event.event.event_type != "signal.accepted.whatsapp.runtime_event" {
        return Ok(());
    }

    let raw_record_id = required_subject_str(&event.event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("WhatsApp runtime event raw record `{raw_record_id}` not found"))?;

    let decision = reconcile_decision_from_payload(&raw_record.payload);
    let Some(decision) = decision else {
        return Ok(());
    };

    let account_store = CommunicationProviderAccountStore::new(pool.clone());
    let binding_store = CommunicationProviderSecretBindingStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool.clone());
    let runtime =
        crate::application::provider_runtime_factories::whatsapp_provider_runtime(pool.clone());

    let Some(current_account) = account_store
        .get(&raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    if !current_account.provider_kind.is_whatsapp() {
        return Ok(());
    }

    if decision.clear_restorable_session {
        clear_whatsapp_restorable_session(
            &binding_store,
            &secret_store,
            &vault,
            &raw_record.account_id,
        )
        .await?;
    }

    if let Some(account_lifecycle_state) = decision.account_lifecycle_state {
        update_whatsapp_account_lifecycle_state(
            &account_store,
            &current_account,
            account_lifecycle_state,
            raw_record.occurred_at.unwrap_or(raw_record.captured_at),
        )
        .await?;
    }

    if let Some(session_link_state) = decision.session_link_state {
        update_whatsapp_session_projection_state(
            &pool,
            &raw_record.account_id,
            session_link_state,
            raw_record.occurred_at.unwrap_or(raw_record.captured_at),
        )
        .await?;
    }

    let status = runtime
        .runtime_status(&secret_store, &vault, &raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?;

    if status.status == "removed" {
        remove_whatsapp_signal_connection(&pool, &raw_record.account_id).await?;
    } else {
        let account = account_store
            .get(&raw_record.account_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                format!(
                    "WhatsApp account `{}` disappeared during runtime-event reconciliation",
                    raw_record.account_id
                )
            })?;
        sync_whatsapp_runtime_signal_connection_for_pool(
            &pool,
            &account,
            &status,
            status.session_secret_ref.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuntimeEventReconcileDecision {
    account_lifecycle_state: Option<&'static str>,
    session_link_state: Option<&'static str>,
    clear_restorable_session: bool,
}

fn reconcile_decision_from_payload(payload: &Value) -> Option<RuntimeEventReconcileDecision> {
    let lifecycle_state = payload.get("lifecycle_state").and_then(Value::as_str);
    let runtime_status = payload.get("runtime_status").and_then(Value::as_str);
    let effective_state = lifecycle_state.or(runtime_status)?.trim();
    if effective_state.is_empty() {
        return None;
    }
    reconcile_decision_from_effective_state(effective_state)
}

fn reconcile_decision_from_effective_state(
    effective_state: &str,
) -> Option<RuntimeEventReconcileDecision> {
    match effective_state {
        "qr_pending" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("qr_pending"),
            session_link_state: Some("qr_pending"),
            clear_restorable_session: false,
        }),
        "pair_code_pending" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("pair_code_pending"),
            session_link_state: Some("pair_code_pending"),
            clear_restorable_session: false,
        }),
        "linked" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("linked"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "available" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("available"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "syncing" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("syncing"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "degraded" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("degraded"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "link_required" | "created" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("created"),
            session_link_state: Some("link_required"),
            clear_restorable_session: true,
        }),
        "revoked" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("revoked"),
            session_link_state: Some("revoked"),
            clear_restorable_session: true,
        }),
        "removed" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("removed"),
            session_link_state: Some("removed"),
            clear_restorable_session: true,
        }),
        _ => None,
    }
}

async fn update_whatsapp_account_lifecycle_state(
    account_store: &CommunicationProviderAccountStore,
    account: &makosh_communications_api::accounts::ProviderAccount,
    lifecycle_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), String> {
    let mut config = account.config.clone();
    let config_object = config
        .as_object_mut()
        .ok_or_else(|| "WhatsApp provider account config must be an object".to_owned())?;
    config_object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
    match lifecycle_state {
        "created" => {
            config_object.remove("revoked_at");
            config_object.remove("removed_at");
        }
        "revoked" => {
            config_object.insert("revoked_at".to_owned(), json!(observed_at));
            config_object.remove("removed_at");
        }
        "removed" => {
            config_object.insert("removed_at".to_owned(), json!(observed_at));
        }
        _ => {
            config_object.remove("revoked_at");
            config_object.remove("removed_at");
        }
    }
    account_store
        .update_config_with_origin(
            &account.account_id,
            &config,
            ProviderAccountMutationOrigin::LocalRuntime,
            "application.whatsapp_runtime_event_projection",
            "runtime_event_reconcile",
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn update_whatsapp_session_projection_state(
    pool: &PgPool,
    account_id: &str,
    link_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), String> {
    WhatsappWebStore::update_projection_link_state(pool, account_id, link_state, observed_at)
        .await
        .map_err(|error| error.to_string())
}

async fn clear_whatsapp_restorable_session(
    binding_store: &CommunicationProviderSecretBindingStore,
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
    account_id: &str,
) -> Result<(), String> {
    let Some(binding) = binding_store
        .get_for_account(
            account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
        )
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };

    binding_store
        .unbind_for_account(
            account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
        )
        .await
        .map_err(|error| error.to_string())?;
    secret_store
        .delete_secret_reference(&binding.secret_ref)
        .await
        .map_err(|error| error.to_string())?;
    match vault.delete_secret(&binding.secret_ref) {
        Ok(_) => {}
        Err(HostVaultError::MissingSecret { .. }) => {}
        Err(error) => return Err(error.to_string()),
    }
    Ok(())
}

async fn remove_whatsapp_signal_connection(pool: &PgPool, account_id: &str) -> Result<(), String> {
    let signal_store = crate::domains::signal_hub::store::SignalHubStore::new(pool.clone());
    let connection_service =
        crate::domains::signal_hub::connections::SignalHubConnectionService::new(
            signal_store,
            makosh_events_postgres::store::EventStore::new(pool.clone()),
        );
    connection_service
        .remove_account_connection("whatsapp", account_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn whatsapp_runtime_signal_status(status: &WhatsAppRuntimeStatus) -> &'static str {
    match status.status.as_str() {
        "removed" => "removed",
        "revoked" | "link_required" | "created" | "blocked" => "awaiting_user_action",
        "qr_pending" | "pair_code_pending" | "syncing" | "degraded" => "connecting",
        "available" | "linked" => "connected",
        _ => {
            if status.session_restore_available {
                "connected"
            } else {
                "awaiting_user_action"
            }
        }
    }
}

fn merged_whatsapp_runtime_connection_settings(
    current: Option<&Value>,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
) -> Value {
    let mut settings = current
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_else(Map::new);
    settings.insert("account_id".to_owned(), json!(account.account_id));
    settings.insert(
        "provider_kind".to_owned(),
        json!(account.provider_kind.as_str()),
    );
    settings.insert(
        "external_account_id".to_owned(),
        json!(account.external_account_id),
    );
    copy_config_field(&mut settings, &account.config, "runtime");
    copy_config_field(&mut settings, &account.config, "lifecycle_state");
    copy_config_field(&mut settings, &account.config, "auth_state");
    settings.insert(
        "whatsapp_runtime_status".to_owned(),
        json!(status.status.as_str()),
    );
    settings.insert(
        "whatsapp_provider_shape".to_owned(),
        json!(status.provider_shape.as_str()),
    );
    settings.insert(
        "whatsapp_runtime_kind".to_owned(),
        json!(status.runtime_kind.as_str()),
    );
    settings.insert(
        "whatsapp_session_restore_available".to_owned(),
        json!(status.session_restore_available),
    );
    settings.insert(
        "whatsapp_live_runtime_available".to_owned(),
        json!(status.live_runtime_available),
    );
    settings.insert(
        "whatsapp_runtime_blockers".to_owned(),
        json!(status.runtime_blockers),
    );
    settings.insert("whatsapp_last_error".to_owned(), json!(status.last_error));
    Value::Object(settings)
}

fn copy_config_field(settings: &mut Map<String, Value>, config: &Value, field: &str) {
    if let Some(value) = config.get(field) {
        settings.insert(field.to_owned(), value.clone());
    }
}

fn required_subject_str<'a>(subject: &'a Value, field: &str) -> Result<&'a str, String> {
    subject
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("event subject field `{field}` is required"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_decision_maps_pending_states() {
        assert_eq!(
            reconcile_decision_from_effective_state("qr_pending"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("qr_pending"),
                session_link_state: Some("qr_pending"),
                clear_restorable_session: false,
            })
        );
        assert_eq!(
            reconcile_decision_from_effective_state("pair_code_pending"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("pair_code_pending"),
                session_link_state: Some("pair_code_pending"),
                clear_restorable_session: false,
            })
        );
    }

    #[test]
    fn reconcile_decision_maps_restorable_and_terminal_states() {
        assert_eq!(
            reconcile_decision_from_effective_state("available"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("available"),
                session_link_state: Some("linked"),
                clear_restorable_session: false,
            })
        );
        assert_eq!(
            reconcile_decision_from_effective_state("syncing"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("syncing"),
                session_link_state: Some("linked"),
                clear_restorable_session: false,
            })
        );
        assert_eq!(
            reconcile_decision_from_effective_state("degraded"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("degraded"),
                session_link_state: Some("linked"),
                clear_restorable_session: false,
            })
        );
        assert_eq!(
            reconcile_decision_from_effective_state("link_required"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("created"),
                session_link_state: Some("link_required"),
                clear_restorable_session: true,
            })
        );
        assert_eq!(
            reconcile_decision_from_effective_state("revoked"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("revoked"),
                session_link_state: Some("revoked"),
                clear_restorable_session: true,
            })
        );
        assert_eq!(
            reconcile_decision_from_effective_state("removed"),
            Some(RuntimeEventReconcileDecision {
                account_lifecycle_state: Some("removed"),
                session_link_state: Some("removed"),
                clear_restorable_session: true,
            })
        );
    }

    #[test]
    fn reconcile_decision_ignores_non_lifecycle_runtime_events() {
        assert_eq!(reconcile_decision_from_effective_state("started"), None);
        assert_eq!(
            reconcile_decision_from_payload(&json!({
                "runtime_event_kind": "media.upload.progress",
                "runtime_status": "started",
                "lifecycle_state": "started",
            })),
            None
        );
    }
}
