use super::{ProviderAccount, WhatsappWebError, WhatsappWebStore};
use chrono::Utc;
use makosh_communications_api::accounts::ProviderAccountMutationOrigin;
use serde_json::json;

impl WhatsappWebStore {
    pub(super) async fn update_account_runtime_kind(
        &self,
        account_id: &str,
        runtime_kind: &str,
        actor: &str,
    ) -> Result<ProviderAccount, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let mut config = account.config.clone();
        let Some(object) = config.as_object_mut() else {
            return Err(WhatsappWebError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        object.insert("runtime".to_owned(), json!(runtime_kind));
        self.provider_account_store()
            .update_config_with_origin(
                &account.account_id,
                &config,
                ProviderAccountMutationOrigin::LocalRuntime,
                actor,
                "runtime_kind_update",
            )
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{}` is not configured",
                    account.account_id
                ))
            })
    }

    pub(super) async fn update_account_lifecycle_state(
        &self,
        account_id: &str,
        lifecycle_state: &str,
        actor: &str,
    ) -> Result<ProviderAccount, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let mut config = account.config.clone();
        let Some(object) = config.as_object_mut() else {
            return Err(WhatsappWebError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        let now = Utc::now();
        object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
        object.insert("lifecycle_updated_at".to_owned(), json!(now));
        if let Some(key) = match lifecycle_state {
            "created" => Some("created_at_runtime"),
            "linked" => Some("linked_at"),
            "revoked" => Some("revoked_at"),
            "removed" => Some("removed_at"),
            _ => None,
        } {
            object.insert(key.to_owned(), json!(now));
        }
        self.provider_account_store()
            .update_config_with_origin(
                &account.account_id,
                &config,
                ProviderAccountMutationOrigin::LocalRuntime,
                actor,
                lifecycle_state,
            )
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{}` is not configured",
                    account.account_id
                ))
            })
    }
}
