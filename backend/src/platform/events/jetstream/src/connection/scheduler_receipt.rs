//! Scheduler-specific adapter from an authorized pull consumer to its public
//! receipt-delivery contract. It carries only opaque envelope bytes and ACK.

use futures_util::StreamExt;
use makosh_scheduler_protocol::{
    SchedulerReceiptDeliveryErrorV1, SchedulerReceiptDeliveryPortV1, SchedulerReceiptDeliveryV1,
};

use super::{RuntimeJetStreamConnection, RuntimeSubscribePermitV1};

pub struct RuntimeSchedulerReceiptPortV1<'a> {
    connection: &'a RuntimeJetStreamConnection,
    permit: RuntimeSubscribePermitV1,
}

impl<'a> RuntimeSchedulerReceiptPortV1<'a> {
    #[must_use]
    pub const fn new(
        connection: &'a RuntimeJetStreamConnection,
        permit: RuntimeSubscribePermitV1,
    ) -> Self {
        Self { connection, permit }
    }
}

impl SchedulerReceiptDeliveryPortV1 for RuntimeSchedulerReceiptPortV1<'_> {
    type Delivery = RuntimeSchedulerReceiptDeliveryV1;

    async fn receive(&mut self) -> Result<Self::Delivery, SchedulerReceiptDeliveryErrorV1> {
        let consumer = self
            .connection
            .open_pull_consumer(&self.permit)
            .await
            .map_err(|_| SchedulerReceiptDeliveryErrorV1::Unavailable)?;
        let mut messages = consumer
            .fetch()
            .max_messages(1)
            .messages()
            .await
            .map_err(|_| SchedulerReceiptDeliveryErrorV1::Unavailable)?;
        messages
            .next()
            .await
            .ok_or(SchedulerReceiptDeliveryErrorV1::Unavailable)?
            .map(RuntimeSchedulerReceiptDeliveryV1::new)
            .map_err(|_| SchedulerReceiptDeliveryErrorV1::Unavailable)
    }
}

pub struct RuntimeSchedulerReceiptDeliveryV1 {
    message: async_nats::jetstream::Message,
}

impl RuntimeSchedulerReceiptDeliveryV1 {
    const fn new(message: async_nats::jetstream::Message) -> Self {
        Self { message }
    }
}

impl SchedulerReceiptDeliveryV1 for RuntimeSchedulerReceiptDeliveryV1 {
    fn exact_bytes(&self) -> &[u8] {
        self.message.payload.as_ref()
    }

    async fn acknowledge(self) -> Result<(), SchedulerReceiptDeliveryErrorV1> {
        self.message
            .ack()
            .await
            .map_err(|_| SchedulerReceiptDeliveryErrorV1::Unavailable)
    }
}
