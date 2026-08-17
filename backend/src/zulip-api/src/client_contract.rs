pub const ZULIP_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.zulip.v1.bin"));
pub const ZULIP_ACCOUNT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.zulip.account.v1.bin"));
pub const ZULIP_OPERATIONAL_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.zulip.operational.v1.bin"));
pub const ZULIP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/makosh.zulip.operational.realtime.v1.bin"
));
pub const ZULIP_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const ZULIP_CLIENT_CONTRACT_REVISION: u32 = 2;
pub const ZULIP_MODULE_ID: &str = "makosh-zulip-runtime";
pub const ZULIP_OWNER_ID: &str = "zulip";
pub const ZULIP_OPERATIONAL_SHARED_REALTIME_CAPABILITY_ID_V1: &str =
    "zulip.operational.realtime.shared.v1";
pub const ZULIP_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1: &str =
    "zulip.operational.projection_changed.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipClientContractV1 {
    AccountLifecycle,
    Command,
    Query,
    OperationalQuery,
    OperationalRealtime,
}

impl ZulipClientContractV1 {
    pub const ALL: [Self; 5] = [
        Self::AccountLifecycle,
        Self::Command,
        Self::Query,
        Self::OperationalQuery,
        Self::OperationalRealtime,
    ];

    #[must_use]
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::AccountLifecycle => "zulip.account.lifecycle.v1",
            Self::Command => "zulip.command.v1",
            Self::Query => "zulip.query.v1",
            Self::OperationalQuery => "zulip.operational.query.v1",
            Self::OperationalRealtime => "zulip.operational.realtime.v1",
        }
    }

    #[must_use]
    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    #[must_use]
    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::AccountLifecycle => "/makosh.zulip.account.v1.ZulipAccountLifecycleService/Apply",
            Self::Command => "/makosh.zulip.v1.ZulipCommandService/ExecuteCommand",
            Self::Query => "/makosh.zulip.v1.ZulipQueryService/GetOperationStatus",
            Self::OperationalQuery => {
                "/makosh.zulip.operational.v1.ZulipOperationalQueryService/Query"
            }
            Self::OperationalRealtime => {
                "/makosh.zulip.operational.realtime.v1.ZulipOperationalRealtimeService/Replay"
            }
        }
    }

    #[must_use]
    pub const fn descriptor_set(self) -> &'static [u8] {
        match self {
            Self::AccountLifecycle => ZULIP_ACCOUNT_DESCRIPTOR_SET_V1,
            Self::Command | Self::Query => ZULIP_CLIENT_DESCRIPTOR_SET_V1,
            Self::OperationalQuery => ZULIP_OPERATIONAL_DESCRIPTOR_SET_V1,
            Self::OperationalRealtime => ZULIP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1,
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
        assert!(!ZULIP_CLIENT_DESCRIPTOR_SET_V1.is_empty());
        assert!(!ZULIP_ACCOUNT_DESCRIPTOR_SET_V1.is_empty());
        assert!(!ZULIP_OPERATIONAL_DESCRIPTOR_SET_V1.is_empty());
        assert!(!ZULIP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1.is_empty());
        assert_ne!(
            ZULIP_CLIENT_DESCRIPTOR_SET_V1,
            ZULIP_OPERATIONAL_DESCRIPTOR_SET_V1
        );
        assert_ne!(
            ZULIP_OPERATIONAL_DESCRIPTOR_SET_V1,
            ZULIP_OPERATIONAL_REALTIME_DESCRIPTOR_SET_V1
        );
        assert_eq!(
            ZulipClientContractV1::ALL
                .into_iter()
                .map(ZulipClientContractV1::capability_id)
                .collect::<BTreeSet<_>>()
                .len(),
            ZulipClientContractV1::ALL.len()
        );
        assert_eq!(
            ZulipClientContractV1::ALL
                .into_iter()
                .map(ZulipClientContractV1::connect_path)
                .collect::<BTreeSet<_>>()
                .len(),
            ZulipClientContractV1::ALL.len()
        );
    }

    #[test]
    fn umbrella_contract_is_not_a_route_identity() {
        assert_eq!(
            ZulipClientContractV1::from_contract_name("zulip.client"),
            None
        );
    }
}
