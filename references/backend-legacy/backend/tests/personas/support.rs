#![allow(dead_code)]

use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_events_postgres::consumers::EventConsumerConfig;
use makosh_events_postgres::consumers::EventConsumerRunner;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::workflows::persona_derived_evidence::{
    PERSONA_DERIVED_EVIDENCE_CONSUMER, project_persona_derived_evidence_event,
};
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn live_personas_pool(_test_name: &str) -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let pool = test_context.pool().clone();
    // This helper intentionally returns only PgPool. Keep the TestContext alive
    // for the test process so an owned PostgreSQL testcontainer is not dropped
    // while the returned pool is still in use.
    Box::leak(Box::new(test_context));
    Some(pool)
}

pub async fn live_personas_store(test_name: &str) -> Option<PersonaProjectionStore> {
    let pool = live_personas_pool(test_name).await?;
    Some(PersonaProjectionStore::new(pool))
}

pub fn disconnected_personas_store() -> PersonaProjectionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    PersonaProjectionStore::new(pool)
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn run_persona_derived_evidence_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSONA_DERIVED_EVIDENCE_CONSUMER),
    );
    runner
        .process_next_batch(|event| project_persona_derived_evidence_event(pool.clone(), event))
        .await
        .expect("persona derived evidence consumer");
}
