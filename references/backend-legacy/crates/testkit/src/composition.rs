//! Explicit backend composition for integration tests.

use axum::Router;
use makosh_hub_backend::app::router::build_router_with_database;

use crate::app::TEST_API_SECRET;
use crate::context::TestContext;

pub fn router_for_context(context: &TestContext) -> Router {
    build_router_with_database(context.app_config(TEST_API_SECRET), context.database())
}
