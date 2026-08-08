use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::{SecretReferenceError, SecretResolutionError};
use super::validation::{validate_non_empty, validate_object};
use makosh_communications_api::accounts::ProviderAccountSecretBinding;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    OauthToken,
    AppPassword,
    Password,
    ApiToken,
    PrivateKey,
    Other,
}

impl SecretKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::AppPassword => "app_password",
            Self::Password => "password",
            Self::ApiToken => "api_token",
            Self::PrivateKey => "private_key",
            Self::Other => "other",
        }
    }
}

impl makosh_communications_api::accounts::SecretKindTag for SecretKind {
    fn secret_kind_tag(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<&str> for SecretKind {
    type Error = SecretReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "app_password" => Ok(Self::AppPassword),
            "password" => Ok(Self::Password),
            "api_token" => Ok(Self::ApiToken),
            "private_key" => Ok(Self::PrivateKey),
            "other" => Ok(Self::Other),
            other => Err(SecretReferenceError::UnsupportedSecretKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretStoreKind {
    OsKeychain,
    EncryptedVault,
    DatabaseEncryptedVault,
    HostVault,
    ExternalVault,
    TestDouble,
}

impl SecretStoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
            Self::DatabaseEncryptedVault => "database_encrypted_vault",
            Self::HostVault => "host_vault",
            Self::ExternalVault => "external_vault",
            Self::TestDouble => "test_double",
        }
    }
}

impl TryFrom<&str> for SecretStoreKind {
    type Error = SecretReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "os_keychain" => Ok(Self::OsKeychain),
            "encrypted_vault" => Ok(Self::EncryptedVault),
            "database_encrypted_vault" => Ok(Self::DatabaseEncryptedVault),
            "host_vault" => Ok(Self::HostVault),
            "external_vault" => Ok(Self::ExternalVault),
            "test_double" => Ok(Self::TestDouble),
            other => Err(SecretReferenceError::UnsupportedStoreKind(other.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SecretReference {
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
    pub label: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub binding: ProviderAccountSecretBinding,
    pub reference: SecretReference,
    pub secret: ResolvedSecret,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ResolvedSecret {
    value: String,
}

impl ResolvedSecret {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretResolutionError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SecretResolutionError::EmptySecretValue);
        }

        Ok(Self { value })
    }

    pub fn expose_for_runtime(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for ResolvedSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedSecret")
            .field("value", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewSecretReference {
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
    pub label: String,
    pub metadata: Value,
}

impl NewSecretReference {
    pub fn new(
        secret_ref: impl Into<String>,
        secret_kind: SecretKind,
        store_kind: SecretStoreKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            secret_ref: secret_ref.into(),
            secret_kind,
            store_kind,
            label: label.into(),
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(super) fn validate(&self) -> Result<(), SecretReferenceError> {
        validate_non_empty("secret_ref", &self.secret_ref)?;
        validate_non_empty("label", &self.label)?;
        validate_object("metadata", &self.metadata)
    }
}
