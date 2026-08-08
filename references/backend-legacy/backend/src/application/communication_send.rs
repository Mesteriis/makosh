use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

use crate::domains::communications::command_service::{
    CommunicationCommandService, CommunicationCommandServiceError, CommunicationOutboxSendCommand,
};
use crate::platform::audit::errors::ApiAuditError;
use crate::platform::audit::models::NewApiAuditRecord;
use makosh_communications_api::email::OutgoingEmail;
use makosh_communications_postgres::errors::CommunicationIngestionError;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

#[derive(Clone)]
pub(crate) struct CommunicationSendDependencies {
    pool: sqlx::postgres::PgPool,
    audit_log: crate::platform::audit::store::ApiAuditLog,
}

impl CommunicationSendDependencies {
    pub(crate) fn new(
        pool: sqlx::postgres::PgPool,
        audit_log: crate::platform::audit::store::ApiAuditLog,
    ) -> Self {
        Self { pool, audit_log }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CommunicationSendRequest {
    pub(crate) account_id: String,
    pub(crate) to: Vec<String>,
    pub(crate) cc: Vec<String>,
    pub(crate) bcc: Vec<String>,
    pub(crate) subject: String,
    pub(crate) body_text: String,
    pub(crate) body_html: Option<String>,
    pub(crate) in_reply_to: Option<String>,
    pub(crate) references: Vec<String>,
    pub(crate) draft_id: Option<String>,
    pub(crate) scheduled_send_at: Option<DateTime<Utc>>,
    pub(crate) undo_send_seconds: Option<i64>,
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug)]
pub(crate) struct CommunicationSendResult {
    pub(crate) message_id: String,
    pub(crate) outbox_id: Option<String>,
    pub(crate) accepted: Vec<String>,
    pub(crate) accepted_recipients: Vec<String>,
    pub(crate) transport: String,
    pub(crate) status: String,
    pub(crate) scheduled_send_at: Option<DateTime<Utc>>,
    pub(crate) undo_deadline_at: Option<DateTime<Utc>>,
    pub(crate) failure_reason: Option<String>,
}

pub(crate) async fn send_email(
    deps: &CommunicationSendDependencies,
    req: CommunicationSendRequest,
) -> Result<CommunicationSendResult, CommunicationSendError> {
    let scheduled_send_at = req.scheduled_send_at;
    let undo_send_seconds = req.undo_send_seconds;
    let draft_id = req.draft_id.clone();
    let account = CommunicationProviderAccountStore::new(deps.pool.clone())
        .get(&req.account_id)
        .await?
        .ok_or(CommunicationSendError::ProviderAccountNotFound)?;
    let email = OutgoingEmail {
        from: account.external_account_id.clone(),
        message_id: None,
        to: req.to,
        cc: req.cc,
        bcc: req.bcc,
        subject: req.subject,
        body_text: req.body_text,
        body_html: req.body_html,
        in_reply_to: req.in_reply_to,
        references: req.references,
        attachments: Vec::new(),
    };

    if email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
        .all(|recipient| recipient.trim().is_empty())
    {
        return Err(CommunicationSendError::InvalidRequest(
            "at least one recipient is required",
        ));
    }
    if !req.metadata.is_object() {
        return Err(CommunicationSendError::InvalidRequest(
            "message metadata must be a JSON object",
        ));
    }

    let recipient_count = email.to.len() + email.cc.len() + email.bcc.len();
    let accepted_recipients = email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
        .cloned()
        .collect::<Vec<_>>();
    let item = CommunicationCommandService::new(deps.pool.clone())
        .enqueue_outbox_send(
            &account,
            &email,
            &CommunicationOutboxSendCommand {
                draft_id,
                scheduled_send_at,
                undo_send_seconds,
                metadata: req.metadata,
            },
        )
        .await?;

    deps.audit_log
        .record(&NewApiAuditRecord::communication_email_send(
            "makosh-frontend",
            &account.account_id,
            recipient_count,
        ))
        .await?;

    Ok(CommunicationSendResult {
        message_id: item.outbox_id.clone(),
        outbox_id: Some(item.outbox_id),
        accepted: accepted_recipients.clone(),
        accepted_recipients,
        transport: "outbox".to_owned(),
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at,
        undo_deadline_at: item.undo_deadline_at,
        failure_reason: None,
    })
}

#[derive(Debug, Error)]
pub(crate) enum CommunicationSendError {
    #[error("{0}")]
    InvalidRequest(&'static str),

    #[error("provider account was not found")]
    ProviderAccountNotFound,

    #[error(transparent)]
    CommunicationIngestion(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Command(#[from] CommunicationCommandServiceError),

    #[error(transparent)]
    Audit(#[from] ApiAuditError),
}
