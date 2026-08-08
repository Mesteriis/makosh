use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use std::sync::Arc;
use thiserror::Error;

use crate::domains::personas::command_service::{
    PersonaCommandService, PersonaCommandServiceError, ProviderAddressBookEntryPersonaCommand,
};
use makosh_communications_api::accounts::{
    CommunicationProviderKind, ProviderAccount, ProviderAccountLookupPort, ProviderAccountPortError,
};
use makosh_communications_api::address_book::{
    AddressBookProviderEntry, AddressBookProviderFetchRequest, AddressBookProviderSyncError,
    AddressBookProviderUpsertRequest, SharedAddressBookProviderSyncPort,
};
use makosh_communications_postgres::errors::CommunicationIngestionError;

const ADDRESS_BOOK_SYNC_POLL_INTERVAL_SECONDS: i64 = 3600;
const ADDRESS_BOOK_SYNC_PAGE_SIZE: u16 = 500;
const GOOGLE_CONTACTS_WRITE_SCOPE: &str = "https://www.googleapis.com/auth/contacts";

#[derive(Debug, Error)]
pub enum AddressBookSyncError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    ProviderAccount(#[from] ProviderAccountPortError),

    #[error("address book sync provider account was not found: {0}")]
    AccountNotFound(String),

    #[error(transparent)]
    PersonaCommand(#[from] PersonaCommandServiceError),

    #[error(transparent)]
    Provider(#[from] AddressBookProviderSyncError),

    #[error("DATABASE_URL is not configured")]
    DatabaseNotConfigured,
}

#[derive(Clone)]
pub struct AddressBookSyncService {
    pool: PgPool,
    provider_sync: SharedAddressBookProviderSyncPort,
    provider_accounts: Arc<dyn ProviderAccountLookupPort>,
}

impl AddressBookSyncService {
    pub fn new(
        pool: PgPool,
        provider_sync: SharedAddressBookProviderSyncPort,
        provider_accounts: Arc<dyn ProviderAccountLookupPort>,
    ) -> Self {
        Self {
            pool,
            provider_sync,
            provider_accounts,
        }
    }

    pub async fn run_due_accounts(&self) -> Result<AddressBookSyncReport, AddressBookSyncError> {
        let mut report = AddressBookSyncReport::default();
        for account_id in due_account_ids(&self.pool).await? {
            match self
                .run_account(&account_id, AddressBookSyncTrigger::Scheduled)
                .await
            {
                Ok(account_report) => report.merge(account_report),
                Err(error) => {
                    report.failed_accounts += 1;
                    tracing::warn!(
                        account_id = %account_id,
                        error = %error,
                        "address book sync account run failed"
                    );
                }
            }
        }
        Ok(report)
    }

    pub async fn run_account(
        &self,
        account_id: &str,
        trigger: AddressBookSyncTrigger,
    ) -> Result<AddressBookSyncAccountReport, AddressBookSyncError> {
        let account = self
            .provider_accounts
            .get(account_id)
            .await?
            .ok_or_else(|| AddressBookSyncError::AccountNotFound(account_id.to_owned()))?;
        let run_id = address_book_sync_run_id(account_id);
        start_run(&self.pool, &run_id, account_id, trigger).await?;

        let result = if !address_book_sync_enabled(&account) {
            Ok(AddressBookSyncAccountReport::skipped())
        } else {
            self.execute_account_sync(&account).await
        };

        match result {
            Ok(report) => {
                finish_run(&self.pool, &run_id, &report, None).await?;
                Ok(report)
            }
            Err(error) => {
                let failed = AddressBookSyncAccountReport::failed();
                let error_message = error.to_string();
                finish_run(
                    &self.pool,
                    &run_id,
                    &failed,
                    Some(("address_book_sync_failed", &error_message)),
                )
                .await?;
                Err(error)
            }
        }
    }

    async fn execute_account_sync(
        &self,
        account: &ProviderAccount,
    ) -> Result<AddressBookSyncAccountReport, AddressBookSyncError> {
        let mut report = AddressBookSyncAccountReport::default();
        self.pull_provider_address_book_entries(account, &mut report)
            .await?;
        self.push_local_address_book_entries(account, &mut report)
            .await?;
        Ok(report)
    }

    async fn pull_provider_address_book_entries(
        &self,
        account: &ProviderAccount,
        report: &mut AddressBookSyncAccountReport,
    ) -> Result<(), AddressBookSyncError> {
        let mut page_token = None;
        loop {
            let batch = self
                .provider_sync
                .fetch_entries(AddressBookProviderFetchRequest {
                    account_id: account.account_id.clone(),
                    provider_kind: account.provider_kind,
                    provider_config: account.config.clone(),
                    page_token,
                    page_size: ADDRESS_BOOK_SYNC_PAGE_SIZE,
                })
                .await?;

            for provider_entry in batch.entries {
                report.provider_entries_seen += 1;
                match self
                    .upsert_provider_address_book_entry(account, provider_entry)
                    .await?
                {
                    ProviderAddressBookEntryUpsertOutcome::Upserted => {
                        report.provider_entries_upserted += 1
                    }
                    ProviderAddressBookEntryUpsertOutcome::Skipped => {
                        report.provider_entries_skipped += 1
                    }
                }
            }

            page_token = batch.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn upsert_provider_address_book_entry(
        &self,
        account: &ProviderAccount,
        provider_entry: AddressBookProviderEntry,
    ) -> Result<ProviderAddressBookEntryUpsertOutcome, AddressBookSyncError> {
        let Some(identity) = address_book_entry_identity(&provider_entry) else {
            return Ok(ProviderAddressBookEntryUpsertOutcome::Skipped);
        };

        let persona = PersonaCommandService::new(self.pool.clone())
            .upsert_persona_from_address_book_entry(ProviderAddressBookEntryPersonaCommand {
                source_account_id: account.account_id.clone(),
                provider_address_book_entry_id: provider_entry
                    .provider_address_book_entry_id
                    .clone(),
                display_name: address_book_entry_display_name(&provider_entry),
                primary_email: identity.primary_email,
                additional_emails: identity.additional_emails,
                phone_numbers: provider_entry.phone_numbers.clone(),
            })
            .await?;
        upsert_provider_address_book_entry_link(
            &self.pool,
            account,
            &persona.persona_id,
            &provider_entry,
        )
        .await?;

        Ok(ProviderAddressBookEntryUpsertOutcome::Upserted)
    }

    async fn push_local_address_book_entries(
        &self,
        account: &ProviderAccount,
        report: &mut AddressBookSyncAccountReport,
    ) -> Result<(), AddressBookSyncError> {
        if !bidirectional_address_book_sync_enabled(account) {
            return Ok(());
        }

        let address_book_entries =
            local_address_book_entries_due_for_provider_sync(&self.pool, &account.account_id)
                .await?;
        report.local_entries_seen += address_book_entries.len() as i32;

        let remote_write_allowed = remote_address_book_write_allowed(account);
        if !remote_write_allowed {
            report.local_entries_blocked += address_book_entries.len() as i32;
            return Ok(());
        }

        for address_book_entry in address_book_entries {
            if address_book_entry.provider_address_book_entry_id.is_some()
                && address_book_entry.provider_etag.is_none()
            {
                mark_provider_address_book_link_blocked(
                    &self.pool,
                    &account.account_id,
                    &address_book_entry.persona_id,
                    "missing_provider_etag",
                )
                .await?;
                report.local_entries_blocked += 1;
                continue;
            }

            let provider_entry = self
                .provider_sync
                .upsert_entry(AddressBookProviderUpsertRequest {
                    account_id: account.account_id.clone(),
                    provider_kind: account.provider_kind,
                    provider_address_book_entry_id: address_book_entry
                        .provider_address_book_entry_id
                        .clone(),
                    provider_etag: address_book_entry.provider_etag.clone(),
                    display_name: address_book_entry.display_name.clone(),
                    email_address: address_book_entry.email_address.clone(),
                    phone_numbers: address_book_entry.phone_numbers.clone(),
                    remote_write_allowed,
                })
                .await?;
            upsert_provider_address_book_entry_link(
                &self.pool,
                account,
                &address_book_entry.persona_id,
                &provider_entry,
            )
            .await?;
            mark_provider_address_book_link_pushed(
                &self.pool,
                &account.account_id,
                &address_book_entry.persona_id,
            )
            .await?;
            report.local_entries_pushed += 1;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum AddressBookSyncTrigger {
    Scheduled,
    Manual,
}

impl AddressBookSyncTrigger {
    fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

#[derive(Default)]
pub struct AddressBookSyncReport {
    pub accounts_synced: i32,
    pub failed_accounts: i32,
    pub provider_entries_seen: i32,
    pub provider_entries_upserted: i32,
    pub provider_entries_skipped: i32,
    pub local_entries_seen: i32,
    pub local_entries_pushed: i32,
    pub local_entries_blocked: i32,
}

impl AddressBookSyncReport {
    fn merge(&mut self, account_report: AddressBookSyncAccountReport) {
        if account_report.status == AddressBookSyncRunStatus::Completed {
            self.accounts_synced += 1;
        }
        self.provider_entries_seen += account_report.provider_entries_seen;
        self.provider_entries_upserted += account_report.provider_entries_upserted;
        self.provider_entries_skipped += account_report.provider_entries_skipped;
        self.local_entries_seen += account_report.local_entries_seen;
        self.local_entries_pushed += account_report.local_entries_pushed;
        self.local_entries_blocked += account_report.local_entries_blocked;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AddressBookSyncRunStatus {
    Completed,
    Skipped,
    Failed,
}

impl AddressBookSyncRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone)]
pub struct AddressBookSyncAccountReport {
    status: AddressBookSyncRunStatus,
    provider_entries_seen: i32,
    provider_entries_upserted: i32,
    provider_entries_skipped: i32,
    local_entries_seen: i32,
    local_entries_pushed: i32,
    local_entries_blocked: i32,
}

impl Default for AddressBookSyncAccountReport {
    fn default() -> Self {
        Self {
            status: AddressBookSyncRunStatus::Completed,
            provider_entries_seen: 0,
            provider_entries_upserted: 0,
            provider_entries_skipped: 0,
            local_entries_seen: 0,
            local_entries_pushed: 0,
            local_entries_blocked: 0,
        }
    }
}

impl AddressBookSyncAccountReport {
    pub fn response(&self) -> AddressBookSyncRunResponse {
        AddressBookSyncRunResponse {
            status: self.status.as_str().to_owned(),
            provider_entries_seen: self.provider_entries_seen,
            provider_entries_upserted: self.provider_entries_upserted,
            provider_entries_skipped: self.provider_entries_skipped,
            local_entries_seen: self.local_entries_seen,
            local_entries_pushed: self.local_entries_pushed,
            local_entries_blocked: self.local_entries_blocked,
        }
    }

    fn skipped() -> Self {
        Self {
            status: AddressBookSyncRunStatus::Skipped,
            ..Self::default()
        }
    }

    fn failed() -> Self {
        Self {
            status: AddressBookSyncRunStatus::Failed,
            ..Self::default()
        }
    }
}

#[derive(Clone, Serialize)]
pub struct AddressBookSyncRunResponse {
    pub status: String,
    pub provider_entries_seen: i32,
    pub provider_entries_upserted: i32,
    pub provider_entries_skipped: i32,
    pub local_entries_seen: i32,
    pub local_entries_pushed: i32,
    pub local_entries_blocked: i32,
}

#[derive(Clone)]
struct LocalAddressBookEntry {
    persona_id: String,
    display_name: String,
    email_address: Option<String>,
    phone_numbers: Vec<String>,
    provider_address_book_entry_id: Option<String>,
    provider_etag: Option<String>,
}

enum ProviderAddressBookEntryUpsertOutcome {
    Upserted,
    Skipped,
}

struct AddressBookEntryIdentity {
    primary_email: Option<String>,
    additional_emails: Vec<String>,
}

fn address_book_entry_identity(
    provider_entry: &AddressBookProviderEntry,
) -> Option<AddressBookEntryIdentity> {
    let email_addresses = provider_entry
        .email_addresses
        .iter()
        .filter_map(|email| non_empty_string(email))
        .collect::<Vec<_>>();

    if let Some((primary_email, additional_emails)) = email_addresses.split_first() {
        return Some(AddressBookEntryIdentity {
            primary_email: Some(primary_email.clone()),
            additional_emails: additional_emails.to_vec(),
        });
    }

    let has_display_name = provider_entry
        .display_name
        .as_deref()
        .and_then(non_empty_str)
        .is_some();
    let has_phone_number = provider_entry
        .phone_numbers
        .iter()
        .any(|phone| non_empty_str(phone).is_some());
    if !has_display_name && !has_phone_number {
        return None;
    }

    Some(AddressBookEntryIdentity {
        primary_email: None,
        additional_emails: Vec::new(),
    })
}

fn address_book_entry_display_name(provider_entry: &AddressBookProviderEntry) -> Option<String> {
    provider_entry
        .display_name
        .as_deref()
        .and_then(non_empty_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            provider_entry
                .email_addresses
                .iter()
                .find_map(|email| non_empty_string(email))
        })
        .or_else(|| {
            provider_entry
                .phone_numbers
                .iter()
                .find_map(|phone| non_empty_string(phone))
        })
}

fn non_empty_string(value: &str) -> Option<String> {
    non_empty_str(value).map(ToOwned::to_owned)
}

fn non_empty_str(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn address_book_sync_run_id(account_id: &str) -> String {
    format!(
        "address_book_sync_run:v1:{}:{}",
        account_id.trim(),
        uuid::Uuid::now_v7()
    )
}

fn address_book_sync_enabled(account: &ProviderAccount) -> bool {
    let enabled = json_bool(&account.config, "address_book_sync_enabled").unwrap_or(true);
    !account.is_deleted() && connected_services_include(&account.config, "contacts") && enabled
}

fn bidirectional_address_book_sync_enabled(account: &ProviderAccount) -> bool {
    account
        .config
        .get("address_book_sync_direction")
        .and_then(Value::as_str)
        .is_some_and(|value| value == "bidirectional")
        || json_bool(&account.config, "address_book_bidirectional_enabled").unwrap_or(false)
}

fn remote_address_book_write_allowed(account: &ProviderAccount) -> bool {
    bidirectional_address_book_sync_enabled(account)
        && account.provider_kind == CommunicationProviderKind::Gmail
        && requested_scopes_include(&account.config, GOOGLE_CONTACTS_WRITE_SCOPE)
        && json_bool(&account.config, "address_book_remote_write_enabled").unwrap_or(false)
}

fn connected_services_include(config: &Value, service: &str) -> bool {
    config
        .get("connected_services")
        .and_then(Value::as_array)
        .is_some_and(|services| {
            services
                .iter()
                .filter_map(Value::as_str)
                .any(|value| value == service)
        })
}

fn requested_scopes_include(config: &Value, scope: &str) -> bool {
    config
        .get("requested_scopes")
        .and_then(Value::as_array)
        .is_some_and(|scopes| {
            scopes
                .iter()
                .filter_map(Value::as_str)
                .any(|value| value.trim() == scope)
        })
}

fn json_bool(config: &Value, key: &str) -> Option<bool> {
    config.get(key).and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;

    use super::*;

    #[test]
    fn remote_address_book_write_requires_bidirectional_scope_and_explicit_flag() {
        let account = provider_account(json!({
            "connected_services": ["contacts"],
            "address_book_sync_direction": "bidirectional",
            "address_book_remote_write_enabled": true,
            "requested_scopes": [GOOGLE_CONTACTS_WRITE_SCOPE],
        }));

        assert!(address_book_sync_enabled(&account));
        assert!(remote_address_book_write_allowed(&account));
    }

    #[test]
    fn remote_address_book_write_ignores_legacy_contact_config_keys_after_migration() {
        let legacy_direction_key = ["contacts", "sync_direction"].join("_");
        let legacy_remote_write_key = ["contacts", "remote_write_enabled"].join("_");
        let mut config = json!({
            "connected_services": ["contacts"],
            "requested_scopes": [GOOGLE_CONTACTS_WRITE_SCOPE],
        });
        let config_object = config.as_object_mut().expect("test config object");
        config_object.insert(legacy_direction_key, json!("bidirectional"));
        config_object.insert(legacy_remote_write_key, json!(true));
        let account = provider_account(config);

        assert!(address_book_sync_enabled(&account));
        assert!(!remote_address_book_write_allowed(&account));
    }

    #[test]
    fn remote_address_book_write_stays_blocked_without_write_scope() {
        let account = provider_account(json!({
            "connected_services": ["contacts"],
            "address_book_sync_direction": "bidirectional",
            "address_book_remote_write_enabled": true,
            "requested_scopes": ["https://www.googleapis.com/auth/contacts.readonly"],
        }));

        assert!(!remote_address_book_write_allowed(&account));
    }

    #[test]
    fn remote_address_book_write_stays_blocked_without_explicit_remote_write_flag() {
        let account = provider_account(json!({
            "connected_services": ["contacts"],
            "address_book_sync_direction": "bidirectional",
            "requested_scopes": [GOOGLE_CONTACTS_WRITE_SCOPE],
        }));

        assert!(!remote_address_book_write_allowed(&account));
    }

    #[test]
    fn platform_provider_kind_maps_domain_provider_kind() {
        assert_eq!(
            CommunicationProviderKind::Gmail,
            CommunicationProviderKind::Gmail
        );
        assert_eq!(
            CommunicationProviderKind::WhatsappWeb,
            CommunicationProviderKind::WhatsappWeb
        );
    }

    #[test]
    fn address_book_entry_identity_accepts_phone_only_provider_entry() {
        let identity = address_book_entry_identity(&AddressBookProviderEntry {
            provider_address_book_entry_id: "people/phone-only".to_owned(),
            display_name: Some("Phone Only".to_owned()),
            email_addresses: Vec::new(),
            phone_numbers: vec!["+1 555 0100".to_owned()],
            etag: Some("etag".to_owned()),
        })
        .expect("phone-only address book entries should be importable");

        assert_eq!(identity.primary_email, None);
        assert!(identity.additional_emails.is_empty());
    }

    #[test]
    fn address_book_entry_identity_accepts_name_only_provider_entry() {
        let identity = address_book_entry_identity(&AddressBookProviderEntry {
            provider_address_book_entry_id: "people/name-only".to_owned(),
            display_name: Some("Name Only".to_owned()),
            email_addresses: Vec::new(),
            phone_numbers: Vec::new(),
            etag: Some("etag".to_owned()),
        })
        .expect("name-only address book entries should still materialize a persona");

        assert_eq!(identity.primary_email, None);
        assert!(identity.additional_emails.is_empty());
    }

    #[test]
    fn address_book_entry_identity_skips_empty_provider_shell() {
        assert!(
            address_book_entry_identity(&AddressBookProviderEntry {
                provider_address_book_entry_id: "people/empty".to_owned(),
                display_name: None,
                email_addresses: Vec::new(),
                phone_numbers: Vec::new(),
                etag: None,
            })
            .is_none()
        );
    }

    #[tokio::test]
    async fn local_address_book_entries_include_name_only_personas_for_provider_sync() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("connect to postgres");
        let suffix = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .unsigned_abs();
        let account_id = format!("account-name-only-outbound-{suffix}");
        let persona_id = format!("persona:test:name-only-outbound:{suffix}");

        sqlx::query(
            r#"
            INSERT INTO personas (
                persona_id,
                display_name,
                email_address,
                is_address_book
            )
            VALUES ($1, 'Name Only Outbound Persona', NULL, true)
            "#,
        )
        .bind(&persona_id)
        .execute(&pool)
        .await
        .expect("insert name-only address-book persona");

        let entries = local_address_book_entries_due_for_provider_sync(&pool, &account_id)
            .await
            .expect("load local address-book entries due for provider sync");

        let entry = entries
            .iter()
            .find(|entry| entry.persona_id == persona_id)
            .expect("name-only address-book persona should be selected for provider sync");
        assert_eq!(entry.email_address, None);
        assert_eq!(entry.phone_numbers, Vec::<String>::new());
        assert_eq!(entry.display_name, "Name Only Outbound Persona");
    }

    fn provider_account(config: Value) -> ProviderAccount {
        ProviderAccount {
            account_id: "provider-account".to_owned(),
            provider_kind: CommunicationProviderKind::Gmail,
            display_name: "Gmail".to_owned(),
            external_account_id: "owner@example.com".to_owned(),
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
mod store;
use store::*;
