//! Vault-facing capability contracts.
//!
//! Providers never depend on this crate. Composition resolves a lease through
//! this port, then passes the provider-owned capability value to a runtime.

use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use makosh_provider_api::{CredentialLease, ProviderId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CredentialLeaseRequest {
    pub provider_id: ProviderId,
    pub account_id: String,
    pub purpose: String,
    pub minimum_epoch: u64,
    pub required_until: DateTime<Utc>,
}

impl CredentialLeaseRequest {
    pub fn new(
        provider_id: ProviderId,
        account_id: impl Into<String>,
        purpose: impl Into<String>,
        minimum_epoch: u64,
        required_until: DateTime<Utc>,
    ) -> Result<Self, VaultApiError> {
        Ok(Self {
            provider_id,
            account_id: required_identifier(account_id.into(), "account id")?,
            purpose: required_identifier(purpose.into(), "credential purpose")?,
            minimum_epoch,
            required_until,
        })
    }
}

pub type CredentialLeaseFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CredentialLeasePortError>> + Send + 'a>>;

/// Issues and revokes secret capability values without disclosing vault storage
/// or master-key details to application and provider layers.
pub trait CredentialLeasePort: Send + Sync {
    fn issue<'a>(
        &'a self,
        request: &'a CredentialLeaseRequest,
    ) -> CredentialLeaseFuture<'a, CredentialLease>;

    fn revoke<'a>(
        &'a self,
        provider_id: &'a ProviderId,
        account_id: &'a str,
        epoch: u64,
    ) -> CredentialLeaseFuture<'a, bool>;
}

#[derive(Debug, Error)]
#[error("credential lease port error: {code}")]
pub struct CredentialLeasePortError {
    code: String,
}

impl CredentialLeasePortError {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

fn required_identifier(value: String, kind: &'static str) -> Result<String, VaultApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(VaultApiError::EmptyField(kind));
    }
    Ok(value.to_owned())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum VaultApiError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use makosh_provider_api::ProviderId;

    use super::{CredentialLeaseRequest, VaultApiError};

    #[test]
    fn credential_lease_request_requires_account_and_purpose() {
        let provider_id = ProviderId::parse("zulip").expect("provider id");
        let required_until = Utc.timestamp_opt(100, 0).single().expect("timestamp");

        assert_eq!(
            CredentialLeaseRequest::new(provider_id.clone(), " ", "command", 0, required_until),
            Err(VaultApiError::EmptyField("account id"))
        );
        assert_eq!(
            CredentialLeaseRequest::new(provider_id, "account-1", " ", 0, required_until),
            Err(VaultApiError::EmptyField("credential purpose"))
        );
    }
}
