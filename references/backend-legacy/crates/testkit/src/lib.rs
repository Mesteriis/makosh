//! Backend-specific test infrastructure for Макошь.
//!
//! Provides programmatic container management, isolated test databases,
//! and entity factories for integration testing.
//!
//! # Usage
//!
//! ```ignore
//! use makosh_backend_testkit::context::TestContext;
//! use makosh_backend_testkit::factories::task::TaskFactory;
//!
//! #[tokio::test]
//! async fn my_integration_test() {
//!     let ctx = TestContext::new().await;
//!     let task = TaskFactory::new(ctx.pool())
//!         .with_title("Review Q1 report")
//!         .create()
//!         .await
//!         .unwrap();
//!     assert_eq!(task.title, "Review Q1 report");
//! }
//! ```

pub mod app;
pub mod composition;
pub mod containers;
pub mod context;
pub mod factories;
pub mod vault;
