//! Exact platform credential scopes accepted during Vault initialization.

use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::SecretRecordScope;
use zeroize::Zeroizing;

const STORAGE_OWNER: &str = "storage";
const STORAGE_INSTANCE_ID: &str = "storage_main";
const INITIAL_STORAGE_GENERATION: u64 = 1;
const EVENT_HUB_OWNER: &str = "kernel";
const EVENT_HUB_INSTANCE_ID: &str = "event_hub_main";
const EVENTS_OWNER: &str = "events";
const EVENTS_AUTHORITY_INSTANCE_ID: &str = "events_authority_runtime";

const PGBOUNCER_ADMIN_PURPOSE: &str = "storage.control.pgbouncer.admin";
const POSTGRES_ADMIN_PURPOSE: &str = "storage.control.postgres.admin";
const EVENT_HUB_NATS_PURPOSE: &str = "events.nats.event_hub.credential";
const EVENT_ACCOUNT_SIGNER_PURPOSE: &str = "events.nats.account_signer";

type ScopedPlatformCredentialV1 = (SecretRecordScope, Zeroizing<Vec<u8>>);

pub(super) struct PlatformCredentialSeeds {
    pgbouncer_admin: Zeroizing<Vec<u8>>,
    postgres_admin: Zeroizing<Vec<u8>>,
    event_hub_nats: Option<Zeroizing<Vec<u8>>>,
    event_account_signer: Option<Zeroizing<Vec<u8>>>,
}

impl PlatformCredentialSeeds {
    pub(super) fn new(
        pgbouncer_admin: Vec<u8>,
        postgres_admin: Vec<u8>,
        event_hub_nats: Option<Vec<u8>>,
        event_account_signer: Option<Vec<u8>>,
    ) -> Result<Self, String> {
        let seeds = Self {
            pgbouncer_admin: Zeroizing::new(pgbouncer_admin),
            postgres_admin: Zeroizing::new(postgres_admin),
            event_hub_nats: event_hub_nats.map(Zeroizing::new),
            event_account_signer: event_account_signer.map(Zeroizing::new),
        };
        seeds.validate()?;
        Ok(seeds)
    }

    pub(super) fn scoped_credentials(self) -> Result<Vec<ScopedPlatformCredentialV1>, String> {
        let mut credentials = vec![
            (scope(PGBOUNCER_ADMIN_PURPOSE)?, self.pgbouncer_admin),
            (scope(POSTGRES_ADMIN_PURPOSE)?, self.postgres_admin),
        ];
        if let Some(event_hub_nats) = self.event_hub_nats {
            credentials.push((event_hub_scope()?, event_hub_nats));
        }
        if let Some(event_account_signer) = self.event_account_signer {
            credentials.push((event_account_signer_scope()?, event_account_signer));
        }
        Ok(credentials)
    }

    fn validate(&self) -> Result<(), String> {
        valid_credential(&self.pgbouncer_admin)
            .then_some(())
            .ok_or_else(|| "Vault platform credential import is invalid".to_owned())?;
        valid_credential(&self.postgres_admin)
            .then_some(())
            .ok_or_else(|| "Vault platform credential import is invalid".to_owned())?;
        self.event_hub_nats
            .as_deref()
            .map_or(Ok(()), |credential| {
                valid_credential(credential)
                    .then_some(())
                    .ok_or_else(|| "Vault platform credential import is invalid".to_owned())
            })?;
        self.event_account_signer
            .as_deref()
            .map_or(Ok(()), |credential| {
                valid_credential(credential)
                    .then_some(())
                    .ok_or_else(|| "Vault platform credential import is invalid".to_owned())
            })
    }
}

fn scope(purpose_id: &str) -> Result<SecretRecordScope, String> {
    let purpose = VaultPurposeRequestV1::new(
        purpose_id.to_owned(),
        STORAGE_INSTANCE_ID.to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        600,
    )
    .map_err(|_| "Vault platform credential import is invalid".to_owned())?;
    SecretRecordScope::new(
        STORAGE_OWNER.to_owned(),
        &purpose,
        SecretClassV1::PlatformCredential,
        INITIAL_STORAGE_GENERATION,
    )
    .map_err(|_| "Vault platform credential import is invalid".to_owned())
}

fn event_hub_scope() -> Result<SecretRecordScope, String> {
    let purpose = VaultPurposeRequestV1::new(
        EVENT_HUB_NATS_PURPOSE.to_owned(),
        EVENT_HUB_INSTANCE_ID.to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        600,
    )
    .map_err(|_| "Vault platform credential import is invalid".to_owned())?;
    SecretRecordScope::new(
        EVENT_HUB_OWNER.to_owned(),
        &purpose,
        SecretClassV1::PlatformCredential,
        INITIAL_STORAGE_GENERATION,
    )
    .map_err(|_| "Vault platform credential import is invalid".to_owned())
}

fn event_account_signer_scope() -> Result<SecretRecordScope, String> {
    let purpose = VaultPurposeRequestV1::new(
        EVENT_ACCOUNT_SIGNER_PURPOSE.to_owned(),
        EVENTS_AUTHORITY_INSTANCE_ID.to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        600,
    )
    .map_err(|_| "Vault platform credential import is invalid".to_owned())?;
    SecretRecordScope::new(
        EVENTS_OWNER.to_owned(),
        &purpose,
        SecretClassV1::PlatformCredential,
        INITIAL_STORAGE_GENERATION,
    )
    .map_err(|_| "Vault platform credential import is invalid".to_owned())
}

fn valid_credential(value: &[u8]) -> bool {
    !value.is_empty() && value.len() <= 65_536 && !value.contains(&0)
}
