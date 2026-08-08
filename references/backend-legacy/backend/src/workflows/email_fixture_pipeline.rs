use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{ProviderAccountCommandPort, ProviderAccountPortError};
use makosh_communications_api::evidence::{
    CommunicationEvidencePortError, CommunicationRawEvidenceCommandPort,
};
use serde::Serialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::import::{
    FixtureEmailImportError, FixtureEmailImportRequest, import_fixture_email_messages_with_records,
};
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::provider_observation_projection::{
    CommunicationSignalProjectionError, project_accepted_signal_if_runtime_allows,
};
use crate::domains::graph::ports::{GraphProjectionPort, GraphProjectionPortError};
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::api::participants::upsert_personas_from_message_participants;
use crate::domains::personas::ports::PersonaProjectionPort;
use crate::domains::signal_hub::mail::dispatch_mail_raw_signal;
use crate::domains::signal_hub::store::SignalHubError;
use makosh_graph_api::GraphSummary;

use crate::workflows::graph_projection::errors::GraphProjectionError;
use crate::workflows::graph_projection::models::GraphProjectionReport;
use crate::workflows::graph_projection::service::GraphProjectionService;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailFixturePipelineRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub import_batch_id: String,
    pub fixture_json: String,
}

impl EmailFixturePipelineRequest {
    pub fn new(
        account_id: impl Into<String>,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        provider_kind: CommunicationProviderKind,
        import_batch_id: impl Into<String>,
        fixture_json: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            provider_kind,
            import_batch_id: import_batch_id.into(),
            fixture_json: fixture_json.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailFixtureImportPipelineReport {
    pub account_id: String,
    pub import_batch_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub imported_records: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EmailFixtureProjectionPipelineReport {
    pub account_id: String,
    pub import_batch_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub imported_records: usize,
    pub projected_messages: usize,
    pub upserted_personas: usize,
    pub graph_projection: GraphProjectionReport,
    pub graph_summary: GraphSummary,
    pub total_graph_nodes: i64,
    pub total_graph_edges: i64,
}

pub async fn import_fixture_email_messages_for_dev(
    provider_accounts: &dyn ProviderAccountCommandPort,
    communication_evidence: &dyn CommunicationRawEvidenceCommandPort,
    request: &EmailFixturePipelineRequest,
) -> Result<EmailFixtureImportPipelineReport, EmailFixturePipelineError> {
    upsert_fixture_provider_account(provider_accounts, request).await?;
    let import_report = import_fixture_email_messages_with_records(
        communication_evidence,
        &FixtureEmailImportRequest::new(
            &request.account_id,
            &request.import_batch_id,
            &request.fixture_json,
        ),
    )
    .await?;

    Ok(EmailFixtureImportPipelineReport {
        account_id: request.account_id.clone(),
        import_batch_id: request.import_batch_id.clone(),
        provider_kind: request.provider_kind,
        imported_records: import_report.inserted_or_existing_records,
    })
}

pub async fn project_fixture_email_messages(
    pool: PgPool,
    provider_accounts: &dyn ProviderAccountCommandPort,
    communication_evidence: &dyn CommunicationRawEvidenceCommandPort,
    request: &EmailFixturePipelineRequest,
) -> Result<EmailFixtureProjectionPipelineReport, EmailFixturePipelineError> {
    upsert_fixture_provider_account(provider_accounts, request).await?;
    let import_report = import_fixture_email_messages_with_records(
        communication_evidence,
        &FixtureEmailImportRequest::new(
            &request.account_id,
            &request.import_batch_id,
            &request.fixture_json,
        ),
    )
    .await?;

    let person_store = PersonaProjectionPort::new(pool.clone());
    let mut projected_messages = 0;
    let mut participants = Vec::new();
    for raw_record in &import_report.raw_records {
        let Some(accepted_event) = dispatch_mail_raw_signal(pool.clone(), raw_record, None).await?
        else {
            continue;
        };
        let Some(message) =
            project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event).await?
        else {
            continue;
        };
        participants.push(message.sender.clone());
        participants.extend(message.recipients.clone());
        projected_messages += 1;
    }
    let personas = upsert_personas_from_message_participants(&person_store, &participants).await?;

    let graph_projection = GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await?;
    let graph_summary = GraphProjectionPort::new(pool).summary().await?;
    let total_graph_nodes = graph_summary
        .node_counts
        .iter()
        .map(|count| count.count)
        .sum();
    let total_graph_edges = graph_summary
        .edge_counts
        .iter()
        .map(|count| count.count)
        .sum();

    Ok(EmailFixtureProjectionPipelineReport {
        account_id: request.account_id.clone(),
        import_batch_id: request.import_batch_id.clone(),
        provider_kind: request.provider_kind,
        imported_records: import_report.inserted_or_existing_records,
        projected_messages,
        upserted_personas: personas.len(),
        graph_projection,
        graph_summary,
        total_graph_nodes,
        total_graph_edges,
    })
}

async fn upsert_fixture_provider_account(
    provider_accounts: &dyn ProviderAccountCommandPort,
    request: &EmailFixturePipelineRequest,
) -> Result<(), ProviderAccountPortError> {
    let account = NewProviderAccount::new(
        &request.account_id,
        request.provider_kind,
        &request.display_name,
        &request.external_account_id,
    )
    .config(provider_config(request.provider_kind));
    provider_accounts.upsert(&account).await?;
    Ok(())
}

fn provider_config(provider_kind: CommunicationProviderKind) -> serde_json::Value {
    match provider_kind {
        CommunicationProviderKind::Gmail => json!({"history_stream_id": "gmail:fixture"}),
        CommunicationProviderKind::Icloud => {
            json!({"host": "imap.mail.me.com", "port": 993, "tls": true, "mailbox": "INBOX"})
        }
        CommunicationProviderKind::Imap => {
            json!({"host": "localhost", "port": 993, "tls": true, "mailbox": "INBOX"})
        }
        CommunicationProviderKind::TelegramUser
        | CommunicationProviderKind::TelegramBot
        | CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud
        | CommunicationProviderKind::ZulipBot
        | CommunicationProviderKind::ZoomUser
        | CommunicationProviderKind::ZoomServerToServer
        | CommunicationProviderKind::YandexTelemostUser => json!({}),
    }
}

#[derive(Debug, Error)]
pub enum EmailFixturePipelineError {
    #[error(transparent)]
    CommunicationEvidence(#[from] CommunicationEvidencePortError),

    #[error(transparent)]
    ProviderAccount(#[from] ProviderAccountPortError),

    #[error(transparent)]
    Import(#[from] FixtureEmailImportError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),

    #[error(transparent)]
    Persona(#[from] PersonaProjectionError),

    #[error(transparent)]
    GraphProjection(#[from] GraphProjectionError),

    #[error(transparent)]
    GraphProjectionPort(#[from] GraphProjectionPortError),
}
