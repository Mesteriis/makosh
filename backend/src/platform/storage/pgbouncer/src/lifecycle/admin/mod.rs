//! Authenticated PgBouncer administration and fenced lifecycle adaptation.

mod adapter;
mod catalog;
mod connection;
mod credential;
mod endpoint;
mod error;
mod readiness;
mod reload;

pub use adapter::{PgBouncerAdminPortV1, PgBouncerPoolFenceAdapterV1};
pub use catalog::{database_is_configured, user_is_configured};
#[allow(unused_imports)]
pub use connection::TokioPostgresPgBouncerAdminPortV1;
#[allow(unused_imports)]
pub use credential::PgBouncerAdminCredentialV1;
#[allow(unused_imports)]
pub use endpoint::PgBouncerAdminEndpointV1;
#[allow(unused_imports)]
pub use error::PgBouncerAdminConnectionErrorV1;
pub use readiness::verify_admin_connection;
pub use reload::reload_configuration;
