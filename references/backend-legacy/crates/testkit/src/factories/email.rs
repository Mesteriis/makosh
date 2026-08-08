use chrono::Utc;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct EmailFactory<'a> {
    pool: &'a PgPool,
    account_id: Option<String>,
    subject: String,
    from_address: String,
    body_text: String,
}

impl<'a> EmailFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            account_id: None,
            subject: "Test Email Subject".into(),
            from_address: "test@example.com".into(),
            body_text: "This is a test email body for integration testing.".into(),
        }
    }

    pub fn with_account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from_address = from.into();
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body_text = body.into();
        self
    }

    /// Create a provider account and a raw communication record for an email.
    /// Returns the raw record.
    pub async fn create(
        self,
    ) -> Result<
        (NewProviderAccount, NewRawCommunicationRecord),
        makosh_communications_postgres::errors::CommunicationIngestionError,
    > {
        let store = CommunicationIngestionStore::new(self.pool.clone());

        let account_id = self
            .account_id
            .unwrap_or_else(|| format!("acct:{}", Uuid::new_v4()));
        let account = NewProviderAccount {
            account_id: account_id.clone(),
            provider_kind: CommunicationProviderKind::Gmail,
            display_name: "Test Gmail Account".into(),
            external_account_id: format!("gm-{}", Uuid::new_v4()),
            config: serde_json::json!({"email": self.from_address}),
        };
        store.upsert_provider_account(&account).await?;

        let record_id = format!("rec:{}", Uuid::new_v4());
        let raw = NewRawCommunicationRecord {
            raw_record_id: record_id.clone(),
            account_id,
            record_kind: "email".into(),
            provider_record_id: format!("msg-{}", Uuid::new_v4()),
            source_fingerprint: format!("fp-{}", Uuid::new_v4()),
            import_batch_id: format!("batch-{}", Uuid::new_v4()),
            occurred_at: Some(Utc::now()),
            payload: serde_json::json!({
                "subject": self.subject,
                "from": self.from_address,
                "to": ["recipient@example.com"],
                "body_text": self.body_text,
            }),
            provenance: serde_json::json!({
                "source": "EmailFactory",
                "provider": "gmail",
            }),
        };

        Ok((account, raw))
    }
}
