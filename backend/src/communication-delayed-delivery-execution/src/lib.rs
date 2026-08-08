#![forbid(unsafe_code)]

mod cleanup;
mod delivery_port;
mod ports;
mod worker;

pub use delivery_port::{
    DelayedDeliveryIntentRequestV1, DelayedDeliveryIntentResponseV1, DeliveryIntentRequestErrorV1,
    DeliveryIntentRequestPortV1, decode_delivery_intent_response_v1,
};
pub use ports::{
    BodyCleanupErrorV1, BodyCleanupPortV1, BodyCleanupReasonV1, BodyReadErrorV1, BodyReadPortV1,
    ClaimDueExecutionOutcomeV1, ClaimDueExecutionV1, CleanupStorePortV1,
    DelayedDeliveryBodyCleanupJobV1, DelayedDeliveryBodyReceiptV1, DelayedDeliveryDurableMessageV1,
    DelayedDeliveryExecutionClaimV1, DelayedDeliveryRuntimePortV1, ExecutionStoreErrorV1,
    ExecutionStorePortV1, MarkDeliveryAcceptedV1, MarkDeliveryFailedV1, SchedulerExecutionFenceV1,
    SchedulerReceiptErrorV1, SchedulerReceiptFactoryPortV1, SchedulerTerminalOutcomeV1,
};
pub use worker::{
    DelayedDeliveryExecutionOutcomeV1, DelayedDeliveryWorkerErrorV1, execute_due_delivery_v1,
};

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-execution";
pub use cleanup::{
    DelayedDeliveryCleanupErrorV1, DelayedDeliveryCleanupOutcomeV1, process_body_cleanup_once_v1,
};
