use makosh_communications_api::accounts::CommunicationProviderKind;

use crate::errors::DevEmailSyncError;

pub(super) const DEFAULT_IMAP_PORT: u16 = 993;

const DEFAULT_ICLOUD_IMAP_HOST: &str = "imap.mail.me.com";

pub(super) fn parse_provider_kind(
    value: &str,
) -> Result<CommunicationProviderKind, DevEmailSyncError> {
    let provider_kind = CommunicationProviderKind::try_from(value.trim())
        .map_err(|_| DevEmailSyncError::InvalidProviderKind(value.to_owned()))?;
    match provider_kind {
        CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap => Ok(provider_kind),
        CommunicationProviderKind::Gmail
        | CommunicationProviderKind::TelegramUser
        | CommunicationProviderKind::TelegramBot
        | CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud
        | CommunicationProviderKind::ZulipBot
        | CommunicationProviderKind::ZoomUser
        | CommunicationProviderKind::ZoomServerToServer
        | CommunicationProviderKind::YandexTelemostUser => {
            Err(DevEmailSyncError::UnsupportedProviderForDevSync)
        }
    }
}

pub(super) fn default_host(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::Icloud => DEFAULT_ICLOUD_IMAP_HOST,
        CommunicationProviderKind::Imap => "localhost",
        CommunicationProviderKind::Gmail
        | CommunicationProviderKind::TelegramUser
        | CommunicationProviderKind::TelegramBot
        | CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud
        | CommunicationProviderKind::ZulipBot
        | CommunicationProviderKind::ZoomUser
        | CommunicationProviderKind::ZoomServerToServer
        | CommunicationProviderKind::YandexTelemostUser => "",
    }
}
