use chrono::Utc;
use makosh_communications_api::accounts::ProviderAccountMutationOrigin;
use serde_json::json;

use super::super::errors::TelegramError;
use super::super::identifiers::{
    telegram_account_from_provider_account, telegram_account_lifecycle_state,
};
use super::super::models::accounts::TelegramAccount;
use super::super::store::TelegramStore;
use super::super::validation::validate_object;
use super::super::{TELEGRAM_ACCOUNT_LOGGED_OUT, TELEGRAM_ACCOUNT_REMOVED};

impl TelegramStore {
    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<Vec<TelegramAccount>, TelegramError> {
        let accounts = self
            .provider_account_store()
            .list()
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        Ok(accounts
            .into_iter()
            .filter(|account| account.provider_kind.is_telegram())
            .map(telegram_account_from_provider_account)
            .filter(|account| {
                include_removed || account.lifecycle_state != TELEGRAM_ACCOUNT_REMOVED
            })
            .collect())
    }

    pub async fn logout_account(&self, account_id: &str) -> Result<TelegramAccount, TelegramError> {
        self.update_account_lifecycle(account_id, TELEGRAM_ACCOUNT_LOGGED_OUT)
            .await
    }

    pub async fn remove_account(&self, account_id: &str) -> Result<TelegramAccount, TelegramError> {
        self.update_account_lifecycle(account_id, TELEGRAM_ACCOUNT_REMOVED)
            .await
    }

    async fn update_account_lifecycle(
        &self,
        account_id: &str,
        lifecycle_state: &'static str,
    ) -> Result<TelegramAccount, TelegramError> {
        let account = self.telegram_provider_account(account_id).await?;
        let current_state = telegram_account_lifecycle_state(&account);
        if current_state == TELEGRAM_ACCOUNT_REMOVED && lifecycle_state != TELEGRAM_ACCOUNT_REMOVED
        {
            return Err(TelegramError::InvalidRequest(format!(
                "Telegram account `{}` is removed",
                account.account_id
            )));
        }

        let mut config = account.config.clone();
        validate_object("config", &config)?;
        let Some(config_object) = config.as_object_mut() else {
            return Err(TelegramError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        let now = Utc::now();
        config_object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
        config_object.insert("lifecycle_updated_at".to_owned(), json!(now));
        match lifecycle_state {
            TELEGRAM_ACCOUNT_LOGGED_OUT => {
                config_object.insert("logged_out_at".to_owned(), json!(now));
            }
            TELEGRAM_ACCOUNT_REMOVED => {
                config_object.insert("removed_at".to_owned(), json!(now));
            }
            _ => {}
        }

        let updated = self
            .provider_account_store()
            .update_config_with_origin(
                &account.account_id,
                &config,
                ProviderAccountMutationOrigin::LocalRuntime,
                "telegram.accounts.lifecycle.update",
                lifecycle_state,
            )
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{}` is not configured",
                    account.account_id
                ))
            })?;

        Ok(telegram_account_from_provider_account(updated))
    }
}
