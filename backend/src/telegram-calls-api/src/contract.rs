pub const TELEGRAM_CALLS_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.telegram.calls.v1.bin"));
pub const TELEGRAM_CALLS_CONTRACT_MAJOR: u32 = 1;
pub const TELEGRAM_CALLS_CONTRACT_REVISION: u32 = 1;
pub const TELEGRAM_CALLS_MODULE_ID: &str = "makosh-telegram-runtime";
pub const TELEGRAM_CALLS_OWNER_ID: &str = "telegram";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallsContractV1 {
    Query,
    Command,
    Realtime,
}

impl TelegramCallsContractV1 {
    pub const ALL: [Self; 3] = [Self::Query, Self::Command, Self::Realtime];

    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Query => "telegram.calls.query.v1",
            Self::Command => "telegram.calls.command.v1",
            Self::Realtime => "telegram.calls.realtime.v1",
        }
    }

    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::Query => "/makosh.telegram.calls.v1.TelegramCallsQueryService/Query",
            Self::Command => "/makosh.telegram.calls.v1.TelegramCallsCommandService/Execute",
            Self::Realtime => "/makosh.telegram.calls.v1.TelegramCallsRealtimeService/Replay",
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
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn calls_contracts_have_unique_exact_route_identities() {
        let capabilities = TelegramCallsContractV1::ALL
            .into_iter()
            .map(TelegramCallsContractV1::capability_id)
            .collect::<BTreeSet<_>>();
        let paths = TelegramCallsContractV1::ALL
            .into_iter()
            .map(TelegramCallsContractV1::connect_path)
            .collect::<BTreeSet<_>>();

        assert_eq!(capabilities.len(), TelegramCallsContractV1::ALL.len());
        assert_eq!(paths.len(), TelegramCallsContractV1::ALL.len());
        assert!(TELEGRAM_CALLS_DESCRIPTOR_SET_V1.len() > 32);
        assert_eq!(
            TelegramCallsContractV1::from_contract_name("telegram.calls.query.v1"),
            Some(TelegramCallsContractV1::Query)
        );
        assert_eq!(
            TelegramCallsContractV1::from_contract_name("telegram.calls"),
            None
        );
    }
}
