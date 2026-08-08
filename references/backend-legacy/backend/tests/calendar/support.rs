use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_hub_backend::platform::storage::database::Database;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn live_pool() -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub fn disconnected_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool")
}
