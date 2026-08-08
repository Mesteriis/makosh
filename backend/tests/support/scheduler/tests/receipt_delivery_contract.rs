//! Fail-closed behavior for the Scheduler receipt-consumption boundary.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use makosh_scheduler_persistence::{
    SchedulerPostgresStoreV1, SchedulerReceiptConsumeErrorV1, SchedulerReceiptConsumerV1,
};
use makosh_scheduler_protocol::{
    SchedulerReceiptDeliveryErrorV1, SchedulerReceiptDeliveryPortV1, SchedulerReceiptDeliveryV1,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn malformed_receipt_is_not_acknowledged() {
    let acknowledged = Arc::new(AtomicBool::new(false));
    let port = FakeReceiptPortV1::new(vec![0xff], Arc::clone(&acknowledged));
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://scheduler@localhost/scheduler")
        .expect("lazy PostgreSQL pool");
    let store = SchedulerPostgresStoreV1::new(pool);
    let mut consumer = SchedulerReceiptConsumerV1::new(port, &store);

    assert_eq!(
        consumer.consume_one().await,
        Err(SchedulerReceiptConsumeErrorV1::InvalidReceipt)
    );
    assert!(!acknowledged.load(Ordering::Relaxed));
}

struct FakeReceiptPortV1 {
    delivery: Option<FakeReceiptDeliveryV1>,
}

impl FakeReceiptPortV1 {
    fn new(bytes: Vec<u8>, acknowledged: Arc<AtomicBool>) -> Self {
        Self {
            delivery: Some(FakeReceiptDeliveryV1 {
                bytes,
                acknowledged,
            }),
        }
    }
}

impl SchedulerReceiptDeliveryPortV1 for FakeReceiptPortV1 {
    type Delivery = FakeReceiptDeliveryV1;

    async fn receive(&mut self) -> Result<Self::Delivery, SchedulerReceiptDeliveryErrorV1> {
        self.delivery
            .take()
            .ok_or(SchedulerReceiptDeliveryErrorV1::Unavailable)
    }
}

struct FakeReceiptDeliveryV1 {
    bytes: Vec<u8>,
    acknowledged: Arc<AtomicBool>,
}

impl SchedulerReceiptDeliveryV1 for FakeReceiptDeliveryV1 {
    fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    async fn acknowledge(self) -> Result<(), SchedulerReceiptDeliveryErrorV1> {
        self.acknowledged.store(true, Ordering::Relaxed);
        Ok(())
    }
}
