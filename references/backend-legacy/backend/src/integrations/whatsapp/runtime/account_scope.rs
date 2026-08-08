use makosh_communications_api::accounts::ProviderAccountLookupPort;
use serde_json::Value;

use super::WhatsappWebError;

pub(super) async fn live_whatsapp_account_ids(
    account_lookup: &dyn ProviderAccountLookupPort,
    account_id: Option<&str>,
) -> Result<Vec<String>, WhatsappWebError> {
    let accounts = account_lookup
        .list()
        .await
        .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
    Ok(accounts
        .into_iter()
        .filter(|account| {
            account_id.is_none_or(|id| id == account.account_id)
                && account.provider_kind.as_str() == "whatsapp_web"
                && account
                    .config
                    .get("runtime")
                    .and_then(Value::as_str)
                    .is_some_and(|runtime| runtime != "fixture" && !runtime.is_empty())
                && account
                    .config
                    .get("lifecycle_state")
                    .and_then(Value::as_str)
                    .is_some_and(|state| {
                        matches!(state, "linked" | "available" | "syncing" | "degraded")
                    })
        })
        .map(|account| account.account_id)
        .collect())
}
