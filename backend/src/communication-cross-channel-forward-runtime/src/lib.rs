mod admission;
mod blob_transfer;
mod client_port;
mod client_realtime;
mod contracts;
mod custody_cleanup;
mod delivery_results;
mod event_outbox;
mod managed_runtime;
mod source_prepare;
mod source_results;

pub use blob_transfer::{
    CrossChannelForwardBlobMaterializationV1, CrossChannelForwardBlobPortV1,
    CrossChannelForwardBlobTransferErrorV1, ManagedCrossChannelForwardBlobPortV1,
};
pub use client_port::{
    get_cross_channel_forward_status_payload_v1, start_cross_channel_forward_payload_v1,
};
pub use custody_cleanup::{
    CrossChannelForwardCustodyCleanupErrorV1, CrossChannelForwardCustodyReleasePortV1,
    ManagedCrossChannelForwardCustodyReleasePortV1, process_cross_channel_custody_cleanup_once_v1,
};
pub use delivery_results::{
    CrossChannelForwardDeliveryResultErrorV1, consume_delivery_rejected_once_v1,
    consume_delivery_submitted_once_v1,
};
pub use event_outbox::{CrossChannelForwardEventRelayErrorV1, relay_event_outbox_once_v1};
pub use managed_runtime::{
    CrossChannelForwardManagedRuntimeErrorV1, CrossChannelForwardManagedRuntimeV1,
    CrossChannelForwardRuntimeAdmissionV1,
};
pub use source_prepare::{CrossChannelForwardSourcePrepareErrorV1, enqueue_source_prepare_once_v1};
pub use source_results::{
    CrossChannelForwardSourceConsumerContextV1, CrossChannelForwardSourceResultErrorV1,
    consume_source_prepared_once_v1, consume_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-cross-channel-forward-runtime";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1: &str =
    "communication_cross_channel_forward.blob.v1";
pub use admission::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1,
    communication_cross_channel_forward_module_descriptor_v1,
    communication_cross_channel_forward_settings_schema_bytes_v1,
    communication_cross_channel_forward_settings_schema_v1,
};
