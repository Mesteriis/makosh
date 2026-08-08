//! Stable Telegram client capability, contract and Connect route identities.

pub const TELEGRAM_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.telegram.v1.bin"));
pub const TELEGRAM_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const TELEGRAM_CLIENT_CONTRACT_REVISION: u32 = 6;
pub const TELEGRAM_MODULE_ID: &str = "makosh-telegram-runtime";
pub const TELEGRAM_OWNER_ID: &str = "telegram";
pub const TELEGRAM_AUTHORIZATION_REALTIME_CAPABILITY_ID_V1: &str =
    "telegram.authorization.realtime.v1";
pub const TELEGRAM_AUTHORIZATION_STATUS_CHANGED_CONTRACT_NAME_V1: &str =
    "telegram.authorization.status_changed.v1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TelegramClientContractV1 {
    Authorization,
    Lifecycle,
    Command,
    Query,
    Realtime,
    Reconfiguration,
}

impl TelegramClientContractV1 {
    pub const ALL: [Self; 6] = [
        Self::Authorization,
        Self::Lifecycle,
        Self::Command,
        Self::Query,
        Self::Realtime,
        Self::Reconfiguration,
    ];

    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Authorization => "telegram.authorization.v1",
            Self::Lifecycle => "telegram.lifecycle.v1",
            Self::Command => "telegram.command.v1",
            Self::Query => "telegram.query.v1",
            Self::Realtime => "telegram.realtime.v1",
            Self::Reconfiguration => "telegram.reconfiguration.v1",
        }
    }

    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::Authorization => "/makosh.telegram.v1.TelegramAuthorizationService/Authorize",
            Self::Lifecycle => "/makosh.telegram.v1.TelegramLifecycleService/Execute",
            Self::Command => "/makosh.telegram.v1.TelegramOperationalService/ExecuteCommand",
            Self::Query => "/makosh.telegram.v1.TelegramOperationalService/ExecuteQuery",
            Self::Realtime => "/makosh.telegram.v1.TelegramRealtimeService/Replay",
            Self::Reconfiguration => "/makosh.telegram.v1.TelegramReconfigurationService/Execute",
        }
    }

    pub fn from_contract_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.contract_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contracts_have_unique_capabilities_names_and_routes() {
        let capabilities = TelegramClientContractV1::ALL
            .map(TelegramClientContractV1::capability_id)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let names = TelegramClientContractV1::ALL
            .map(TelegramClientContractV1::contract_name)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let routes = TelegramClientContractV1::ALL
            .map(TelegramClientContractV1::connect_path)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(capabilities.len(), TelegramClientContractV1::ALL.len());
        assert_eq!(names.len(), TelegramClientContractV1::ALL.len());
        assert_eq!(routes.len(), TelegramClientContractV1::ALL.len());
        assert!(!TELEGRAM_CLIENT_DESCRIPTOR_SET_V1.is_empty());
    }

    #[test]
    fn umbrella_contract_is_not_a_route_identity() {
        assert_eq!(
            TelegramClientContractV1::from_contract_name("telegram.client"),
            None
        );
    }
}
