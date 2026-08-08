use chrono::{DateTime, NaiveTime, Utc};
use makosh_communications_api::content_egress::AccountContentEgressPermissions;
use makosh_communications_api::sensitive_forwarding::{
    NewSensitiveForwardingPolicy, SensitiveForwardingDispatchReport, SensitiveForwardingRequest,
    SensitiveForwardingSuppression, StoredSensitiveForwardingPolicy,
};
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::store::MessageProjectionStore;
use crate::domains::communications::outbox::{
    CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStatus,
    NewCommunicationOutboxItem, enqueue_in_transaction,
};
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SensitiveForwardingOutcome {
    Queued(Box<CommunicationOutboxItem>),
    AlreadyDispatched,
    Suppressed(SensitiveForwardingSuppression),
}

#[derive(Clone, Debug)]
struct SensitiveForwardingPolicy {
    policy_id: String,
    source_account_id: String,
    delivery_account_id: String,
    name: String,
    fixed_recipients: Vec<String>,
    minimum_severity: String,
    subject_template: String,
    body_template: String,
    max_sends_per_hour: i32,
    quiet_hours: Value,
    enabled: bool,
    include_message_body: bool,
    include_attachments: bool,
    expires_at: Option<DateTime<Utc>>,
}

const MAX_SENSITIVE_FORWARDING_ATTACHMENTS: usize = 25;
const MAX_SENSITIVE_FORWARDING_ATTACHMENT_BYTES: i64 = 50 * 1024 * 1024;

#[derive(Clone, Debug)]
struct ForwardableAttachment {
    attachment_id: String,
    blob_id: String,
    filename: Option<String>,
    content_type: String,
    size_bytes: i64,
    sha256: String,
    scan_engine: Option<String>,
    scan_checked_at: Option<DateTime<Utc>>,
    scan_summary: Option<String>,
    scan_metadata: Value,
}

#[derive(Clone, Debug, Default)]
struct AttachmentForwardingPlan {
    attachments: Vec<ForwardableAttachment>,
    unsafe_withheld: usize,
    delivery_limit_withheld: usize,
}

impl AttachmentForwardingPlan {
    fn copied_count(&self) -> usize {
        self.attachments.len()
    }

    fn attachments_transferred(&self) -> bool {
        !self.attachments.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QuietHours {
    start: NaiveTime,
    end: NaiveTime,
}

#[derive(Clone)]
pub struct SensitiveForwardingPgStore {
    pool: PgPool,
}

pub type SensitiveForwardingPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, SensitiveForwardingError>> + Send + 'a>>;

pub trait SensitiveForwardingCommandPort: Send + Sync {
    fn content_egress_permissions<'a>(
        &'a self,
        account_id: &'a str,
    ) -> SensitiveForwardingPortFuture<'a, AccountContentEgressPermissions>;

    fn enqueue_for_message<'a>(
        &'a self,
        source_account_id: &'a str,
        message_id: &'a str,
        severity: &'a str,
        now: DateTime<Utc>,
    ) -> SensitiveForwardingPortFuture<'a, SensitiveForwardingDispatchReport>;
}

impl SensitiveForwardingPgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn content_egress_permissions(
        &self,
        account_id: &str,
    ) -> Result<AccountContentEgressPermissions, SensitiveForwardingError> {
        if account_id.trim().is_empty() {
            return Err(SensitiveForwardingError::Invalid);
        }
        let config = sqlx::query_scalar::<_, Value>(
            "SELECT config FROM communication_provider_accounts WHERE account_id = $1",
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(SensitiveForwardingError::AccountNotFound)?;
        Ok(AccountContentEgressPermissions::from_account_config(
            &config,
        ))
    }

    pub async fn upsert_policy(
        &self,
        policy: &NewSensitiveForwardingPolicy,
    ) -> Result<(), SensitiveForwardingError> {
        validate_sensitive_forwarding_policy(policy)?;
        sqlx::query(
            r#"
            INSERT INTO mail_sensitive_forwarding_policies (
                policy_id, source_account_id, delivery_account_id, name, enabled,
                include_message_body, include_attachments,
                fixed_recipients, minimum_severity, subject_template, body_template,
                max_sends_per_hour, quiet_hours, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (policy_id)
            DO UPDATE SET
                source_account_id = EXCLUDED.source_account_id,
                delivery_account_id = EXCLUDED.delivery_account_id,
                name = EXCLUDED.name,
                enabled = EXCLUDED.enabled,
                include_message_body = EXCLUDED.include_message_body,
                include_attachments = EXCLUDED.include_attachments,
                fixed_recipients = EXCLUDED.fixed_recipients,
                minimum_severity = EXCLUDED.minimum_severity,
                subject_template = EXCLUDED.subject_template,
                body_template = EXCLUDED.body_template,
                max_sends_per_hour = EXCLUDED.max_sends_per_hour,
                quiet_hours = EXCLUDED.quiet_hours,
                expires_at = EXCLUDED.expires_at,
                updated_at = now()
            "#,
        )
        .bind(policy.policy_id.trim())
        .bind(policy.source_account_id.trim())
        .bind(policy.delivery_account_id.trim())
        .bind(policy.name.trim())
        .bind(policy.enabled)
        .bind(policy.include_message_body)
        .bind(policy.include_attachments)
        .bind(json!(policy.fixed_recipients))
        .bind(&policy.minimum_severity)
        .bind(policy.subject_template.trim())
        .bind(policy.body_template.trim())
        .bind(policy.max_sends_per_hour)
        .bind(&policy.quiet_hours)
        .bind(policy.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn policies_for_source_account(
        &self,
        source_account_id: &str,
    ) -> Result<Vec<StoredSensitiveForwardingPolicy>, SensitiveForwardingError> {
        if source_account_id.trim().is_empty() {
            return Err(SensitiveForwardingError::Invalid);
        }
        let rows = sqlx::query(
            r#"
            SELECT
                policy_id,
                source_account_id,
                delivery_account_id,
                name,
                enabled,
                include_message_body,
                include_attachments,
                fixed_recipients,
                minimum_severity,
                subject_template,
                body_template,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                updated_at
            FROM mail_sensitive_forwarding_policies
            WHERE source_account_id = $1
            ORDER BY updated_at DESC, policy_id ASC
            "#,
        )
        .bind(source_account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(stored_policy_from_row).collect()
    }

    pub async fn delete_policy(
        &self,
        source_account_id: &str,
        policy_id: &str,
    ) -> Result<bool, SensitiveForwardingError> {
        if source_account_id.trim().is_empty() || policy_id.trim().is_empty() {
            return Err(SensitiveForwardingError::Invalid);
        }
        let result = sqlx::query(
            r#"
            DELETE FROM mail_sensitive_forwarding_policies
            WHERE policy_id = $1
              AND source_account_id = $2
            "#,
        )
        .bind(policy_id.trim())
        .bind(source_account_id.trim())
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// The policy may include a bounded source body only with a separate source-account egress
    /// grant. Attachments are never copied by this path.
    pub async fn enqueue_notification(
        &self,
        request: &SensitiveForwardingRequest,
        now: DateTime<Utc>,
    ) -> Result<SensitiveForwardingOutcome, SensitiveForwardingError> {
        validate_sensitive_forwarding_request(request)?;
        let mut transaction = self.pool.begin().await?;
        let policy = policy_for_update(&mut transaction, &request.policy_id).await?;
        if policy.source_account_id != request.source_account_id.trim() {
            return Err(SensitiveForwardingError::SourceAccountMismatch);
        }

        if let Some(suppression) = policy_suppression(&policy, request, now)? {
            transaction.commit().await?;
            return Ok(SensitiveForwardingOutcome::Suppressed(suppression));
        }

        let already_dispatched = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM mail_sensitive_forwarding_dispatches
                WHERE policy_id = $1
                  AND message_id = $2
            )
            "#,
        )
        .bind(&policy.policy_id)
        .bind(request.message_id.trim())
        .fetch_one(&mut *transaction)
        .await?;
        if already_dispatched {
            transaction.commit().await?;
            return Ok(SensitiveForwardingOutcome::AlreadyDispatched);
        }

        let sends_in_last_hour = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM mail_sensitive_forwarding_dispatches
            WHERE policy_id = $1
              AND created_at >= $2 - INTERVAL '1 hour'
            "#,
        )
        .bind(&policy.policy_id)
        .bind(now)
        .fetch_one(&mut *transaction)
        .await?;
        if sends_in_last_hour >= i64::from(policy.max_sends_per_hour) {
            transaction.commit().await?;
            return Ok(SensitiveForwardingOutcome::Suppressed(
                SensitiveForwardingSuppression::RateLimited,
            ));
        }

        let source_message = MessageProjectionStore::new(self.pool.clone())
            .message(request.message_id.trim())
            .await?
            .ok_or(SensitiveForwardingError::MessageNotFound)?;
        if source_message.account_id != policy.source_account_id {
            return Err(SensitiveForwardingError::SourceAccountMismatch);
        }
        let permissions = self
            .content_egress_permissions(&policy.source_account_id)
            .await?;
        let body_transferred = policy.include_message_body && permissions.body;
        let attachment_transfer_permitted = policy.include_attachments && permissions.attachments;
        let attachment_plan = if attachment_transfer_permitted {
            plan_forwardable_attachments(&mut transaction, request.message_id.trim()).await?
        } else {
            AttachmentForwardingPlan::default()
        };
        let outbox_id = format!("sensitive-forward:{}", request.dispatch_id.trim());
        let outbox = enqueue_in_transaction(
            &mut transaction,
            &NewCommunicationOutboxItem {
                outbox_id: outbox_id.clone(),
                account_id: policy.delivery_account_id.clone(),
                draft_id: None,
                to_recipients: policy.fixed_recipients.clone(),
                cc_recipients: Vec::new(),
                bcc_recipients: Vec::new(),
                subject: render_template(&policy.subject_template, request, ""),
                body_text: render_notification_body(
                    &policy.body_template,
                    request,
                    &source_message,
                    body_transferred,
                    attachment_transfer_permitted,
                    &attachment_plan,
                ),
                body_html: None,
                status: CommunicationOutboxStatus::Queued,
                scheduled_send_at: None,
                undo_deadline_at: None,
                metadata: json!({
                    "automation": {
                        "kind": "sensitive_forwarding",
                        "policy_id": policy.policy_id,
                        "message_id": request.message_id,
                        "severity": request.severity,
                    },
                    "attachments": {
                        "copied": attachment_plan.attachments_transferred(),
                        "clean_copied_count": attachment_plan.copied_count(),
                        "unsafe_content_withheld": request.has_unsafe_attachments || attachment_plan.unsafe_withheld > 0,
                        "delivery_limit_withheld_count": attachment_plan.delivery_limit_withheld,
                    },
                    "content_transfer": {
                        "body": body_transferred,
                        "attachments": attachment_plan.attachments_transferred(),
                    },
                }),
            },
        )
        .await?;
        copy_forwardable_attachments_to_outbox(
            &mut transaction,
            &outbox_id,
            &policy.delivery_account_id,
            request.message_id.trim(),
            &policy.policy_id,
            &attachment_plan,
        )
        .await?;

        let result = sqlx::query(
            r#"
            INSERT INTO mail_sensitive_forwarding_dispatches (
                dispatch_id, policy_id, message_id, outbox_id
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (policy_id, message_id) DO NOTHING
            "#,
        )
        .bind(request.dispatch_id.trim())
        .bind(&policy.policy_id)
        .bind(request.message_id.trim())
        .bind(&outbox_id)
        .execute(&mut *transaction)
        .await?;
        if result.rows_affected() == 0 {
            transaction.rollback().await?;
            return Ok(SensitiveForwardingOutcome::AlreadyDispatched);
        }

        let event = sensitive_forwarding_queued_event(
            &policy,
            request,
            &outbox,
            body_transferred,
            &attachment_plan,
            now,
        )?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(SensitiveForwardingOutcome::Queued(Box::new(outbox)))
    }

    pub async fn enqueue_for_message(
        &self,
        source_account_id: &str,
        message_id: &str,
        severity: &str,
        now: DateTime<Utc>,
    ) -> Result<SensitiveForwardingDispatchReport, SensitiveForwardingError> {
        if source_account_id.trim().is_empty()
            || message_id.trim().is_empty()
            || severity_rank(severity).is_none()
        {
            return Err(SensitiveForwardingError::Invalid);
        }
        let has_unsafe_attachments = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM communication_attachments
                WHERE message_id = $1
                  AND scan_status <> 'clean'
            )
            "#,
        )
        .bind(message_id.trim())
        .fetch_one(&self.pool)
        .await?;
        let policy_ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT policy_id
            FROM mail_sensitive_forwarding_policies
            WHERE source_account_id = $1
            ORDER BY policy_id
            "#,
        )
        .bind(source_account_id.trim())
        .fetch_all(&self.pool)
        .await?;
        let mut report = SensitiveForwardingDispatchReport::default();
        for policy_id in policy_ids {
            let request = SensitiveForwardingRequest {
                dispatch_id: format!("sensitive-forward:{}:{}", policy_id, message_id.trim()),
                policy_id,
                source_account_id: source_account_id.trim().to_owned(),
                message_id: message_id.trim().to_owned(),
                severity: severity.trim().to_owned(),
                has_unsafe_attachments,
            };
            match self.enqueue_notification(&request, now).await? {
                SensitiveForwardingOutcome::Queued(_) => report.queued += 1,
                SensitiveForwardingOutcome::AlreadyDispatched => report.already_dispatched += 1,
                SensitiveForwardingOutcome::Suppressed(_) => report.suppressed += 1,
            }
        }
        Ok(report)
    }
}

impl SensitiveForwardingCommandPort for SensitiveForwardingPgStore {
    fn content_egress_permissions<'a>(
        &'a self,
        account_id: &'a str,
    ) -> SensitiveForwardingPortFuture<'a, AccountContentEgressPermissions> {
        Box::pin(async move { self.content_egress_permissions(account_id).await })
    }

    fn enqueue_for_message<'a>(
        &'a self,
        source_account_id: &'a str,
        message_id: &'a str,
        severity: &'a str,
        now: DateTime<Utc>,
    ) -> SensitiveForwardingPortFuture<'a, SensitiveForwardingDispatchReport> {
        Box::pin(async move {
            self.enqueue_for_message(source_account_id, message_id, severity, now)
                .await
        })
    }
}

async fn policy_for_update(
    transaction: &mut Transaction<'_, sqlx::Postgres>,
    policy_id: &str,
) -> Result<SensitiveForwardingPolicy, SensitiveForwardingError> {
    let row = sqlx::query(
        r#"
        SELECT
            policy_id,
            source_account_id,
            delivery_account_id,
            name,
            fixed_recipients,
            minimum_severity,
            subject_template,
            body_template,
            max_sends_per_hour,
            quiet_hours,
            enabled,
            include_message_body,
            include_attachments,
            expires_at
        FROM mail_sensitive_forwarding_policies
        WHERE policy_id = $1
        FOR UPDATE
        "#,
    )
    .bind(policy_id.trim())
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(SensitiveForwardingError::PolicyNotFound)?;
    let fixed_recipients: Vec<String> = serde_json::from_value(row.try_get("fixed_recipients")?)?;
    let policy = SensitiveForwardingPolicy {
        policy_id: row.try_get("policy_id")?,
        source_account_id: row.try_get("source_account_id")?,
        delivery_account_id: row.try_get("delivery_account_id")?,
        name: row.try_get("name")?,
        fixed_recipients,
        minimum_severity: row.try_get("minimum_severity")?,
        subject_template: row.try_get("subject_template")?,
        body_template: row.try_get("body_template")?,
        max_sends_per_hour: row.try_get("max_sends_per_hour")?,
        quiet_hours: row.try_get("quiet_hours")?,
        enabled: row.try_get("enabled")?,
        include_message_body: row.try_get("include_message_body")?,
        include_attachments: row.try_get("include_attachments")?,
        expires_at: row.try_get("expires_at")?,
    };
    validate_sensitive_forwarding_policy(&NewSensitiveForwardingPolicy {
        policy_id: policy.policy_id.clone(),
        source_account_id: policy.source_account_id.clone(),
        delivery_account_id: policy.delivery_account_id.clone(),
        name: policy.name.clone(),
        enabled: policy.enabled,
        include_message_body: policy.include_message_body,
        include_attachments: policy.include_attachments,
        fixed_recipients: policy.fixed_recipients.clone(),
        minimum_severity: policy.minimum_severity.clone(),
        subject_template: policy.subject_template.clone(),
        body_template: policy.body_template.clone(),
        max_sends_per_hour: policy.max_sends_per_hour,
        quiet_hours: policy.quiet_hours.clone(),
        expires_at: policy.expires_at,
    })?;
    Ok(policy)
}

fn sensitive_forwarding_queued_event(
    policy: &SensitiveForwardingPolicy,
    request: &SensitiveForwardingRequest,
    outbox: &CommunicationOutboxItem,
    body_transferred: bool,
    attachment_plan: &AttachmentForwardingPlan,
    now: DateTime<Utc>,
) -> Result<NewEventEnvelope, SensitiveForwardingError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "communication_intelligence_sensitive_forwarding_queued:{}",
            request.dispatch_id.trim()
        ),
        "communication.intelligence.sensitive_forwarding_queued.v1",
        now,
        json!({ "kind": "mail_sensitive_forwarding" }),
        json!({
            "kind": "communication_message",
            "id": request.message_id,
            "account_id": request.source_account_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-mail-automation" }))
    .payload(json!({
        "policy_id": policy.policy_id,
        "outbox_id": outbox.outbox_id,
        "severity": request.severity,
        "body_transferred": body_transferred,
        "attachment_count": attachment_plan.copied_count(),
        "unsafe_attachments_withheld": request.has_unsafe_attachments || attachment_plan.unsafe_withheld > 0,
        "delivery_limit_withheld_attachment_count": attachment_plan.delivery_limit_withheld,
    }))
    .provenance(json!({
        "source_kind": "sensitive_forwarding_policy",
        "source_id": policy.policy_id,
    }))
    .causation_id(request.dispatch_id.clone())
    .correlation_id(request.message_id.clone())
    .build()
    .map_err(EventStoreError::from)?)
}

#[derive(Debug, Error)]
pub enum SensitiveForwardingError {
    #[error("invalid sensitive forwarding policy or request")]
    Invalid,
    #[error("sensitive forwarding policy was not found")]
    PolicyNotFound,
    #[error("communication provider account was not found")]
    AccountNotFound,
    #[error("source communication message was not found")]
    MessageNotFound,
    #[error("sensitive forwarding policy does not own the source account")]
    SourceAccountMismatch,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Message(#[from] MessageProjectionError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Outbox(#[from] CommunicationOutboxError),
    #[error(transparent)]
    Event(#[from] EventStoreError),
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use makosh_backend_testkit::context::TestContext;
    use serde_json::json;

    use super::{
        AccountContentEgressPermissions, NewSensitiveForwardingPolicy, QuietHours,
        SensitiveForwardingDispatchReport, SensitiveForwardingOutcome, SensitiveForwardingPgStore,
        SensitiveForwardingRequest, SensitiveForwardingSuppression, parse_quiet_hours,
        policy_suppression, validate_sensitive_forwarding_policy,
    };
    use crate::domains::communications::messages::models::NewProjectedMessage;
    use crate::domains::communications::messages::store::MessageProjectionStore;
    use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
    use makosh_communications_api::evidence::NewRawCommunicationRecord;
    use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
    use makosh_communications_postgres::store::CommunicationIngestionStore;

    fn policy() -> NewSensitiveForwardingPolicy {
        NewSensitiveForwardingPolicy {
            policy_id: "policy:test".to_owned(),
            source_account_id: "source".to_owned(),
            delivery_account_id: "delivery".to_owned(),
            name: "Sensitive forward".to_owned(),
            enabled: false,
            include_message_body: false,
            include_attachments: false,
            fixed_recipients: vec!["owner@example.test".to_owned()],
            minimum_severity: "high".to_owned(),
            subject_template: "Sensitive {{severity}} message".to_owned(),
            body_template: "Review {{message_id}}. {{attachment_notice}}".to_owned(),
            max_sends_per_hour: 1,
            quiet_hours: json!({}),
            expires_at: None,
        }
    }

    #[test]
    fn policy_requires_fixed_recipients_valid_severity_and_unambiguous_quiet_hours() {
        assert!(validate_sensitive_forwarding_policy(&policy()).is_ok());
        let mut no_recipients = policy();
        no_recipients.fixed_recipients.clear();
        assert!(validate_sensitive_forwarding_policy(&no_recipients).is_err());
        let mut invalid_severity = policy();
        invalid_severity.minimum_severity = "urgent".to_owned();
        assert!(validate_sensitive_forwarding_policy(&invalid_severity).is_err());
        let mut invalid_quiet_hours = policy();
        invalid_quiet_hours.quiet_hours = json!({ "start": "09:00", "end": "09:00" });
        assert!(validate_sensitive_forwarding_policy(&invalid_quiet_hours).is_err());
        assert!(parse_quiet_hours(&json!({ "start": "09:00", "end": "17:00" })).is_ok());
    }

    #[test]
    fn content_egress_defaults_to_deny_and_requires_explicit_flags() {
        assert_eq!(
            AccountContentEgressPermissions::from_account_config(&json!({})),
            AccountContentEgressPermissions::default()
        );
        assert_eq!(
            AccountContentEgressPermissions::from_account_config(&json!({
                "content_egress": { "body": true, "attachments": false, "extracted_text": true }
            })),
            AccountContentEgressPermissions {
                body: true,
                attachments: false,
                extracted_text: true
            }
        );
    }

    #[test]
    fn quiet_hours_handles_cross_midnight_in_utc() {
        let hours = QuietHours {
            start: "22:00".parse().expect("time"),
            end: "07:00".parse().expect("time"),
        };
        assert!(hours.contains(Utc.with_ymd_and_hms(2026, 7, 11, 23, 0, 0).unwrap()));
        assert!(hours.contains(Utc.with_ymd_and_hms(2026, 7, 12, 6, 0, 0).unwrap()));
        assert!(!hours.contains(Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap()));
    }

    #[test]
    fn policy_suppresses_below_threshold_before_egress() {
        let policy = super::SensitiveForwardingPolicy {
            policy_id: "policy:test".to_owned(),
            source_account_id: "source".to_owned(),
            delivery_account_id: "delivery".to_owned(),
            name: "test policy".to_owned(),
            fixed_recipients: vec!["owner@example.test".to_owned()],
            minimum_severity: "high".to_owned(),
            subject_template: "subject".to_owned(),
            body_template: "body".to_owned(),
            max_sends_per_hour: 1,
            quiet_hours: json!({}),
            enabled: true,
            include_message_body: false,
            include_attachments: false,
            expires_at: None,
        };
        let request = SensitiveForwardingRequest {
            dispatch_id: "dispatch".to_owned(),
            policy_id: "policy:test".to_owned(),
            source_account_id: "source".to_owned(),
            message_id: "message".to_owned(),
            severity: "medium".to_owned(),
            has_unsafe_attachments: false,
        };
        assert_eq!(
            policy_suppression(&policy, &request, Utc::now()).expect("policy evaluation"),
            Some(SensitiveForwardingSuppression::BelowMinimumSeverity)
        );
    }

    #[tokio::test]
    async fn enqueue_notification_is_idempotent_and_withholds_source_content() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let accounts = CommunicationProviderAccountStore::new(pool.clone());
        for (account_id, external_account_id) in [
            ("sensitive-source", "source@example.test"),
            ("sensitive-delivery", "delivery@example.test"),
        ] {
            accounts
                .upsert(&NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::Gmail,
                    account_id,
                    external_account_id,
                ))
                .await
                .expect("provider account");
        }
        let raw = CommunicationIngestionStore::new(pool.clone())
            .record_raw_source(&NewRawCommunicationRecord::new(
                "sensitive-raw",
                "sensitive-source",
                "email_message",
                "provider-message",
                "source-fingerprint",
                "test-import",
                json!({ "kind": "mail" }),
            ))
            .await
            .expect("raw source");
        let raw_record_id = raw.raw_record_id.clone();
        let message = MessageProjectionStore::new(pool.clone())
            .upsert_email_message(&NewProjectedMessage {
                message_id: "ignored-by-canonicalization".to_owned(),
                raw_record_id: raw_record_id.clone(),
                account_id: "sensitive-source".to_owned(),
                provider_record_id: "provider-message".to_owned(),
                subject: "Private source subject".to_owned(),
                sender: "sender@example.test".to_owned(),
                recipients: vec!["source@example.test".to_owned()],
                body_text: "private source body must not be forwarded".to_owned(),
                occurred_at: None,
                channel_kind: "email".to_owned(),
                conversation_id: None,
                sender_display_name: None,
                delivery_state: "received".to_owned(),
                message_metadata: json!({ "transport": "gmail" }),
            })
            .await
            .expect("projected message");
        let message_id = message.message_id.clone();
        let clean_sha256 = format!("sha256:{}", "a".repeat(64));
        let unsafe_sha256 = format!("sha256:{}", "b".repeat(64));
        for (blob_id, storage_path, sha256) in [
            (
                "sensitive-clean-blob",
                "tests/sensitive-clean",
                clean_sha256.as_str(),
            ),
            (
                "sensitive-unsafe-blob",
                "tests/sensitive-unsafe",
                unsafe_sha256.as_str(),
            ),
        ] {
            sqlx::query(
                r#"
                INSERT INTO communication_mail_blobs (
                    blob_id, storage_kind, storage_path, sha256, size_bytes, content_type
                ) VALUES ($1, 'local_fs', $2, $3, 16, 'text/plain')
                "#,
            )
            .bind(blob_id)
            .bind(storage_path)
            .bind(sha256)
            .execute(&pool)
            .await
            .expect("source attachment blob");
        }
        for (attachment_id, blob_id, provider_attachment_id, sha256, scan_status) in [
            (
                "sensitive-clean-attachment",
                "sensitive-clean-blob",
                "clean-source-attachment",
                clean_sha256.as_str(),
                "clean",
            ),
            (
                "sensitive-unsafe-attachment",
                "sensitive-unsafe-blob",
                "unsafe-source-attachment",
                unsafe_sha256.as_str(),
                "suspicious",
            ),
        ] {
            sqlx::query(
                r#"
                INSERT INTO communication_attachments (
                    attachment_id, message_id, raw_record_id, blob_id, provider_attachment_id,
                    filename, content_type, size_bytes, sha256, disposition,
                    scan_status, scan_engine, scan_checked_at, scan_metadata
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    'report.txt', 'text/plain', 16, $6, 'attachment',
                    $7, 'test-scanner', now(), '{}'::jsonb
                )
                "#,
            )
            .bind(attachment_id)
            .bind(&message_id)
            .bind(&raw_record_id)
            .bind(blob_id)
            .bind(provider_attachment_id)
            .bind(sha256)
            .bind(scan_status)
            .execute(&pool)
            .await
            .expect("source attachment");
        }
        let store = SensitiveForwardingPgStore::new(pool.clone());
        let mut configured_policy = policy();
        configured_policy.enabled = true;
        configured_policy.source_account_id = "sensitive-source".to_owned();
        configured_policy.delivery_account_id = "sensitive-delivery".to_owned();
        store
            .upsert_policy(&configured_policy)
            .await
            .expect("forwarding policy");
        let request = SensitiveForwardingRequest {
            dispatch_id: "sensitive-dispatch-1".to_owned(),
            policy_id: configured_policy.policy_id.clone(),
            source_account_id: "sensitive-source".to_owned(),
            message_id: message_id.clone(),
            severity: "high".to_owned(),
            has_unsafe_attachments: true,
        };

        let SensitiveForwardingOutcome::Queued(outbox) = store
            .enqueue_notification(&request, Utc::now())
            .await
            .expect("queue notification")
        else {
            panic!("notification should be queued");
        };
        assert_eq!(outbox.to_recipients, vec!["owner@example.test"]);
        assert!(!outbox.body_text.contains("private source body"));
        assert!(outbox.body_text.contains("Attachments were withheld"));

        let duplicate = SensitiveForwardingRequest {
            dispatch_id: "sensitive-dispatch-2".to_owned(),
            ..request
        };
        assert_eq!(
            store
                .enqueue_notification(&duplicate, Utc::now())
                .await
                .expect("duplicate request"),
            SensitiveForwardingOutcome::AlreadyDispatched
        );
        let outbox_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM communication_outbox WHERE account_id = 'sensitive-delivery'",
        )
        .fetch_one(&pool)
        .await
        .expect("outbox count");
        let dispatch_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM mail_sensitive_forwarding_dispatches WHERE policy_id = 'policy:test'",
        )
        .fetch_one(&pool)
        .await
        .expect("dispatch count");
        assert_eq!(outbox_count, 1);
        assert_eq!(dispatch_count, 1);

        let mut body_policy = configured_policy.clone();
        body_policy.policy_id = "policy:body-transfer".to_owned();
        body_policy.include_message_body = true;
        store
            .upsert_policy(&body_policy)
            .await
            .expect("body transfer policy");
        let denied_request = SensitiveForwardingRequest {
            dispatch_id: "sensitive-body-denied".to_owned(),
            policy_id: body_policy.policy_id.clone(),
            source_account_id: "sensitive-source".to_owned(),
            message_id: message_id.clone(),
            severity: "high".to_owned(),
            has_unsafe_attachments: false,
        };
        let SensitiveForwardingOutcome::Queued(denied_outbox) = store
            .enqueue_notification(&denied_request, Utc::now())
            .await
            .expect("queue body transfer without egress permission")
        else {
            panic!("body transfer notification should be queued");
        };
        assert!(!denied_outbox.body_text.contains("private source body"));
        assert_eq!(denied_outbox.metadata["content_transfer"]["body"], false);

        accounts
            .update_config(
                "sensitive-source",
                &json!({ "content_egress": { "body": true } }),
            )
            .await
            .expect("enable source body content egress");
        body_policy.policy_id = "policy:body-transfer-approved".to_owned();
        store
            .upsert_policy(&body_policy)
            .await
            .expect("approved body transfer policy");
        let approved_request = SensitiveForwardingRequest {
            dispatch_id: "sensitive-body-approved".to_owned(),
            policy_id: body_policy.policy_id,
            source_account_id: "sensitive-source".to_owned(),
            message_id: message_id.clone(),
            severity: "high".to_owned(),
            has_unsafe_attachments: false,
        };
        let SensitiveForwardingOutcome::Queued(approved_outbox) = store
            .enqueue_notification(&approved_request, Utc::now())
            .await
            .expect("queue approved body transfer")
        else {
            panic!("approved body transfer should be queued");
        };
        assert!(approved_outbox.body_text.contains("private source body"));
        assert_eq!(approved_outbox.metadata["content_transfer"]["body"], true);
        assert_eq!(
            approved_outbox.metadata["content_transfer"]["attachments"],
            false
        );

        accounts
            .update_config(
                "sensitive-source",
                &json!({ "content_egress": { "body": true, "attachments": true } }),
            )
            .await
            .expect("enable source attachment content egress");
        let mut attachment_policy = configured_policy.clone();
        attachment_policy.policy_id = "policy:attachment-transfer-approved".to_owned();
        attachment_policy.include_attachments = true;
        store
            .upsert_policy(&attachment_policy)
            .await
            .expect("approved attachment transfer policy");
        let attachment_request = SensitiveForwardingRequest {
            dispatch_id: "sensitive-attachment-approved".to_owned(),
            policy_id: attachment_policy.policy_id,
            source_account_id: "sensitive-source".to_owned(),
            message_id,
            severity: "high".to_owned(),
            has_unsafe_attachments: true,
        };
        let SensitiveForwardingOutcome::Queued(attachment_outbox) = store
            .enqueue_notification(&attachment_request, Utc::now())
            .await
            .expect("queue approved attachment transfer")
        else {
            panic!("approved attachment transfer should be queued");
        };
        assert_eq!(
            attachment_outbox.metadata["content_transfer"]["attachments"],
            true
        );
        assert_eq!(
            attachment_outbox.metadata["attachments"]["clean_copied_count"],
            1
        );
        assert!(
            attachment_outbox
                .body_text
                .contains("1 clean attachment(s) included")
        );
        assert!(
            attachment_outbox
                .body_text
                .contains("1 unsafe or unverified attachment(s) withheld")
        );
        let copied_attachment_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM communication_outbox_attachments WHERE outbox_id = $1",
        )
        .bind(&attachment_outbox.outbox_id)
        .fetch_one(&pool)
        .await
        .expect("forwarded attachment link count");
        assert_eq!(copied_attachment_count, 1);
        let forwarded_source_attachment_id: String = sqlx::query_scalar(
            r#"
            SELECT imported.metadata -> 'source' ->> 'attachment_id'
            FROM communication_outbox_attachments link
            JOIN communication_attachment_imports imported ON imported.attachment_id = link.attachment_id
            WHERE link.outbox_id = $1
            "#,
        )
        .bind(&attachment_outbox.outbox_id)
        .fetch_one(&pool)
        .await
        .expect("forwarded attachment provenance");
        assert_eq!(forwarded_source_attachment_id, "sensitive-clean-attachment");
    }

    #[tokio::test]
    async fn account_command_uses_fixed_policy_recipients_and_is_repeat_safe() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let accounts = CommunicationProviderAccountStore::new(pool.clone());
        for (account_id, external_account_id) in [
            ("forward-source", "source@example.test"),
            ("forward-delivery", "delivery@example.test"),
        ] {
            accounts
                .upsert(&NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::Gmail,
                    account_id,
                    external_account_id,
                ))
                .await
                .expect("provider account");
        }
        let raw = CommunicationIngestionStore::new(pool.clone())
            .record_raw_source(&NewRawCommunicationRecord::new(
                "forward-raw",
                "forward-source",
                "email_message",
                "provider-message",
                "source-fingerprint",
                "test-import",
                json!({ "kind": "mail" }),
            ))
            .await
            .expect("raw source");
        let message = MessageProjectionStore::new(pool.clone())
            .upsert_email_message(&NewProjectedMessage {
                message_id: "ignored-by-canonicalization".to_owned(),
                raw_record_id: raw.raw_record_id,
                account_id: "forward-source".to_owned(),
                provider_record_id: "provider-message".to_owned(),
                subject: "Private source subject".to_owned(),
                sender: "sender@example.test".to_owned(),
                recipients: vec!["source@example.test".to_owned()],
                body_text: "private source body must not be forwarded".to_owned(),
                occurred_at: None,
                channel_kind: "email".to_owned(),
                conversation_id: None,
                sender_display_name: None,
                delivery_state: "received".to_owned(),
                message_metadata: json!({ "transport": "gmail" }),
            })
            .await
            .expect("projected message");
        let store = SensitiveForwardingPgStore::new(pool.clone());
        let mut configured_policy = policy();
        configured_policy.enabled = true;
        configured_policy.source_account_id = "forward-source".to_owned();
        configured_policy.delivery_account_id = "forward-delivery".to_owned();
        store
            .upsert_policy(&configured_policy)
            .await
            .expect("forwarding policy");

        assert_eq!(
            store
                .enqueue_for_message("forward-source", &message.message_id, "high", Utc::now(),)
                .await
                .expect("first policy dispatch"),
            SensitiveForwardingDispatchReport {
                queued: 1,
                already_dispatched: 0,
                suppressed: 0,
            }
        );
        assert_eq!(
            store
                .enqueue_for_message("forward-source", &message.message_id, "high", Utc::now(),)
                .await
                .expect("second policy dispatch"),
            SensitiveForwardingDispatchReport {
                queued: 0,
                already_dispatched: 1,
                suppressed: 0,
            }
        );
    }
}
mod attachments;
use attachments::*;
mod policy;
use policy::*;
