use super::super::*;
use super::calendar::upsert_apple_icloud_calendar_account;
use super::models::{EmailAccountSetupApiResponse, ImapAccountSetupApiRequest};
use crate::app::handlers::communications::account_support::*;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};
use makosh_communications_api::accounts::CommunicationProviderKind;

pub(crate) async fn post_imap_account_setup(
    State(state): State<AppState>,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let setup_request = request.into_setup_request()?;
    let service = account_setup_service(&state)?;
    require_unlocked_host_vault(&state)?;
    let icloud_calendar_account =
        (setup_request.provider_kind == CommunicationProviderKind::Icloud).then(|| {
            (
                setup_request.account_id.clone(),
                setup_request.display_name.clone(),
                setup_request.external_account_id.clone(),
            )
        });
    let result = service.setup_imap_account(setup_request).await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.secret_ref)).await?;
    if let Some((mail_account_id, display_name, external_account_id)) = icloud_calendar_account {
        upsert_apple_icloud_calendar_account(
            &state,
            &mail_account_id,
            &display_name,
            &external_account_id,
            &result.secret_ref,
        )
        .await?;
    }

    Ok(Json(result.into()))
}
