//! Connection factory with bounded endpoint validation and no ambient credentials.

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use makosh_events_protocol::RuntimeNatsJwtCredentialV1;
use nats_jwt::KeyPair;

use super::{
    EventHubJetStreamConnection, NatsPasswordCredentialV1, RuntimeJetStreamConnection,
    RuntimeNatsIdentity,
};
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Factory for connections with no process-global client or ambient credentials.
pub struct JetStreamClient;

impl JetStreamClient {
    pub async fn connect_runtime(
        endpoint: &str,
        identity: RuntimeNatsIdentity,
        credential: NatsPasswordCredentialV1,
    ) -> Result<RuntimeJetStreamConnection, String> {
        let (username, password) = credential.credentials();
        let context = connect_context(endpoint, username, password).await?;
        Ok(RuntimeJetStreamConnection::new(context, identity))
    }

    pub async fn connect_event_hub(
        endpoint: &str,
        credential: NatsPasswordCredentialV1,
    ) -> Result<EventHubJetStreamConnection, String> {
        let (username, password) = credential.credentials();
        let context = connect_context(endpoint, username, password).await?;
        Ok(EventHubJetStreamConnection::new(context))
    }

    pub async fn connect_runtime_with_jwt(
        endpoint: &str,
        identity: RuntimeNatsIdentity,
        credential: RuntimeNatsJwtCredentialV1,
    ) -> Result<RuntimeJetStreamConnection, String> {
        validate_endpoint(endpoint)?;
        let options = runtime_jwt_connection_options(credential)?;
        let context = connect_with_options(endpoint, options).await?;
        Ok(RuntimeJetStreamConnection::new(context, identity))
    }
}

fn runtime_jwt_connection_options(
    credential: RuntimeNatsJwtCredentialV1,
) -> Result<async_nats::ConnectOptions, String> {
    let (jwt, seed, expires_at_unix_seconds) = credential.into_connection_material();
    let now_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "NATS clock is unavailable".to_owned())?
        .as_secs();
    if now_unix_seconds >= expires_at_unix_seconds {
        return Err("NATS JWT credential expired".to_owned());
    }
    let user_key = Arc::new(
        KeyPair::from_seed(seed.as_str())
            .map_err(|_| "NATS JWT credential is unavailable".to_owned())?,
    );
    Ok(async_nats::ConnectOptions::with_jwt(
        jwt.to_string(),
        move |nonce| {
            let user_key = Arc::clone(&user_key);
            async move { user_key.sign(&nonce).map_err(async_nats::AuthError::new) }
        },
    ))
}

async fn connect_context(
    endpoint: &str,
    username: &str,
    password: &str,
) -> Result<async_nats::jetstream::Context, String> {
    validate_endpoint(endpoint)?;
    let options = async_nats::ConnectOptions::new()
        .user_and_password(username.to_owned(), password.to_owned());
    connect_with_options(endpoint, options).await
}

async fn connect_with_options(
    endpoint: &str,
    options: async_nats::ConnectOptions,
) -> Result<async_nats::jetstream::Context, String> {
    let client = tokio::time::timeout(
        CONNECT_TIMEOUT,
        options
            .connection_timeout(CONNECT_TIMEOUT)
            .connect(endpoint),
    )
    .await
    .map_err(|_| "NATS connection timed out".to_owned())?
    .map_err(|_| "NATS connection is unavailable".to_owned())?;
    let mut context = async_nats::jetstream::new(client);
    context.set_timeout(REQUEST_TIMEOUT);
    Ok(context)
}

fn validate_endpoint(endpoint: &str) -> Result<(), String> {
    (endpoint.starts_with("nats://") && !endpoint.contains(['@', '?', '#', ' ']))
        .then_some(())
        .ok_or_else(|| "NATS endpoint is invalid".to_owned())
}
