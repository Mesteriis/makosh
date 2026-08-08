use crate::domains::communications::blockers::ArchitectureBlocker;
use makosh_connectrpc_contracts::makosh::communications::v1::CommunicationArchitectureBlocker;

pub(super) fn from_domain(item: ArchitectureBlocker) -> CommunicationArchitectureBlocker {
    CommunicationArchitectureBlocker {
        section: item.section,
        feature: item.feature,
        reason: item.reason,
        resolution: item.resolution,
        ..Default::default()
    }
}
