use std::os::unix::net::UnixStream;

use makosh_communication_delayed_delivery_execution::{
    DelayedDeliveryCleanupErrorV1, DelayedDeliveryCleanupOutcomeV1, process_body_cleanup_once_v1,
};
use makosh_communication_delayed_delivery_persistence::CommunicationDelayedDeliveryPersistenceV1;
use makosh_communication_delayed_delivery_runtime_adapters::ManagedDelayedDeliveryRuntimePortV1;
use makosh_communication_delayed_delivery_store_adapters::DelayedDeliveryExecutionStoreAdapterV1;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};

use crate::COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1;

pub(crate) async fn process_pending_body_cleanup_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    now_unix_millis: u64,
) -> Result<DelayedDeliveryCleanupOutcomeV1, DelayedDeliveryCleanupErrorV1> {
    let mut store = DelayedDeliveryExecutionStoreAdapterV1::new(persistence.clone());
    let mut cleanup_port = ManagedDelayedDeliveryRuntimePortV1::new(
        channel,
        dispatcher,
        COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1,
    )
    .map_err(|_| DelayedDeliveryCleanupErrorV1::InvalidInput)?;
    process_body_cleanup_once_v1(
        &mut store,
        &mut cleanup_port,
        logical_owner_id,
        now_unix_millis,
    )
    .await
}
