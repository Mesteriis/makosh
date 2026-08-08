use makosh_communications_api::accounts::ProviderAccount;

use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn telegram_account_record(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        self.telegram_provider_account(account_id).await
    }

    pub(in crate::integrations::telegram::client) async fn telegram_provider_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        let provider_account = self
            .provider_account_store()
            .get(account_id)
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{account_id}` is not configured"
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                provider_account.account_id
            )));
        }
        Ok(provider_account)
    }
}
