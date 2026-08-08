pub const WHATSAPP_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.whatsapp.v1.bin"));
pub const WHATSAPP_OPERATIONAL_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/makosh.whatsapp.operational.v1.bin"
));
pub const WHATSAPP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/makosh.whatsapp.operational.realtime.v1.bin"
));
pub const WHATSAPP_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const WHATSAPP_CLIENT_CONTRACT_REVISION: u32 = 1;
pub const WHATSAPP_MODULE_ID: &str = "makosh-whatsapp-runtime";
pub const WHATSAPP_OWNER_ID: &str = "whatsapp";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppClientContractV1 {
    Command,
    Query,
    OperationalQuery,
    OperationalRealtime,
}

impl WhatsAppClientContractV1 {
    pub const ALL: [Self; 4] = [
        Self::Command,
        Self::Query,
        Self::OperationalQuery,
        Self::OperationalRealtime,
    ];

    #[must_use]
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Command => "whatsapp.command.v1",
            Self::Query => "whatsapp.query.v1",
            Self::OperationalQuery => "whatsapp.operational.query.v1",
            Self::OperationalRealtime => "whatsapp.operational.realtime.v1",
        }
    }

    #[must_use]
    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    #[must_use]
    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::Command => "/makosh.whatsapp.v1.WhatsAppCommandService/ExecuteCommand",
            Self::Query => "/makosh.whatsapp.v1.WhatsAppQueryService/GetOperationStatus",
            Self::OperationalQuery => {
                "/makosh.whatsapp.operational.v1.WhatsAppOperationalQueryService/Query"
            }
            Self::OperationalRealtime => {
                "/makosh.whatsapp.operational.realtime.v1.WhatsAppOperationalRealtimeService/Replay"
            }
        }
    }

    #[must_use]
    pub const fn descriptor_set(self) -> &'static [u8] {
        match self {
            Self::Command | Self::Query => WHATSAPP_DESCRIPTOR_SET_V1,
            Self::OperationalQuery => WHATSAPP_OPERATIONAL_DESCRIPTOR_SET_V1,
            Self::OperationalRealtime => WHATSAPP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1,
        }
    }

    #[must_use]
    pub fn from_contract_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|contract| contract.contract_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn client_contracts_have_unique_capabilities_names_and_routes() {
        assert!(!WHATSAPP_DESCRIPTOR_SET_V1.is_empty());
        assert!(!WHATSAPP_OPERATIONAL_DESCRIPTOR_SET_V1.is_empty());
        assert!(!WHATSAPP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1.is_empty());
        assert_ne!(
            WHATSAPP_DESCRIPTOR_SET_V1,
            WHATSAPP_OPERATIONAL_DESCRIPTOR_SET_V1
        );
        assert_ne!(
            WHATSAPP_OPERATIONAL_DESCRIPTOR_SET_V1,
            WHATSAPP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1
        );
        assert_eq!(
            WhatsAppClientContractV1::ALL
                .into_iter()
                .map(WhatsAppClientContractV1::capability_id)
                .collect::<BTreeSet<_>>()
                .len(),
            WhatsAppClientContractV1::ALL.len()
        );
        assert_eq!(
            WhatsAppClientContractV1::ALL
                .into_iter()
                .map(WhatsAppClientContractV1::connect_path)
                .collect::<BTreeSet<_>>()
                .len(),
            WhatsAppClientContractV1::ALL.len()
        );
    }

    #[test]
    fn umbrella_contract_is_not_a_route_identity() {
        assert_eq!(
            WhatsAppClientContractV1::from_contract_name("whatsapp.client"),
            None
        );
    }
}
