use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use chrono::Utc;
pub(crate) use makosh_communications_api::evidence::NewIngestionCheckpoint;
pub(crate) use makosh_communications_postgres::store::CommunicationIngestionStore;
pub(crate) use makosh_hub_backend::domains::communications::credentials::{
    ProviderCredentialError, ProviderCredentialReader,
};

pub(crate) use makosh_hub_backend::platform::secrets::errors::SecretResolutionError;
pub(crate) use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretStoreKind,
};
pub(crate) use makosh_hub_backend::platform::secrets::resolver::{
    InMemorySecretResolver, SecretResolver,
};
pub(crate) use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
pub(crate) use makosh_hub_backend::platform::storage::database::Database;
pub(crate) use serde_json::json;

pub(crate) async fn test_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}

pub(crate) async fn connect_database(test_name: &str) -> Option<Database> {
    let database_url = test_database_url(test_name).await?;
    Some(
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    )
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
