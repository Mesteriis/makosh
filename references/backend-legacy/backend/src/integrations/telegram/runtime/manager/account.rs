use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::identifiers::ensure_telegram_account_active;
use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::accounts::ProviderAccountLookupPort;

use super::super::status::load_telegram_account;

pub(in crate::integrations::telegram::runtime::manager) async fn load_active_account(
    provider_account_store: &dyn ProviderAccountLookupPort,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account = load_telegram_account(provider_account_store, account_id).await?;
    ensure_telegram_account_active(&account)?;
    Ok(account)
}
