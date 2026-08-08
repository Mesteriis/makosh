//! Desired Event Hub topology derived from the approved catalog.

mod managed_runtime;
pub(crate) mod plan;
mod scheduler_dispatches;
mod scheduler_receipts;
pub(crate) mod subject;

pub(crate) use managed_runtime::{
    managed_runtime_consumer_bindings, managed_runtime_publish_subjects,
};
pub use plan::{EventConsumerPlanV1, EventPublisherPermitPlanV1, EventTopologyPlanV1};
#[allow(unused_imports)] // Re-exported for topology contract tests.
pub(crate) use scheduler_dispatches::SchedulerDispatchTopologyErrorV1;
pub(crate) use scheduler_dispatches::scheduler_dispatch_bindings;
#[allow(unused_imports)] // Re-exported for topology contract tests.
pub(crate) use scheduler_receipts::SchedulerReceiptTopologyErrorV1;
pub(crate) use scheduler_receipts::scheduler_receipt_bindings;

use super::catalog::EventCatalogContractV1;

pub fn plan(
    contracts: &[EventCatalogContractV1],
    configuration: &makosh_kernel_control_store::PlatformEventHubTopologyV1,
) -> Result<EventTopologyPlanV1, String> {
    EventTopologyPlanV1::from_contracts(contracts, configuration)
}
