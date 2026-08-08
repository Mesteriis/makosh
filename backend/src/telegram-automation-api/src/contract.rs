//! Exact Telegram automation client capability and route identities.

pub const TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/makosh.telegram.automation.v1.bin"
));
pub const TELEGRAM_AUTOMATION_CONTRACT_MAJOR: u32 = 1;
pub const TELEGRAM_AUTOMATION_CONTRACT_REVISION: u32 = 1;
pub const TELEGRAM_AUTOMATION_MODULE_ID: &str = "makosh-telegram-runtime";
pub const TELEGRAM_AUTOMATION_OWNER_ID: &str = "telegram";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TelegramAutomationContractV1 {
    Query,
    Command,
}

impl TelegramAutomationContractV1 {
    pub const ALL: [Self; 2] = [Self::Query, Self::Command];

    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Query => "telegram.automation.query.v1",
            Self::Command => "telegram.automation.command.v1",
        }
    }

    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::Query => "/makosh.telegram.automation.v1.TelegramAutomationQueryService/Query",
            Self::Command => {
                "/makosh.telegram.automation.v1.TelegramAutomationCommandService/Execute"
            }
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
    fn automation_contracts_have_unique_exact_route_identities() {
        assert_eq!(
            TelegramAutomationContractV1::ALL
                .map(TelegramAutomationContractV1::capability_id)
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            TelegramAutomationContractV1::ALL.len()
        );
        assert_eq!(
            TelegramAutomationContractV1::ALL
                .map(TelegramAutomationContractV1::connect_path)
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            TelegramAutomationContractV1::ALL.len()
        );
        assert!(!TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1.is_empty());
        assert_eq!(
            TelegramAutomationContractV1::from_contract_name("telegram.client"),
            None
        );
    }
}
