use makosh_communications_api::evidence::{
    CommunicationEvidencePortError, CommunicationRawEvidenceCommandPort, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use serde_json::json;
use thiserror::Error;

use crate::domains::communications::sources::{
    FixtureEmailSourceError, parse_fixture_email_messages,
};

const EMAIL_MESSAGE_RECORD_KIND: &str = "email_message";

pub struct FixtureEmailImportRequest {
    pub account_id: String,
    pub import_batch_id: String,
    pub fixture_json: String,
}

impl FixtureEmailImportRequest {
    pub fn new(
        account_id: impl Into<String>,
        import_batch_id: impl Into<String>,
        fixture_json: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            import_batch_id: import_batch_id.into(),
            fixture_json: fixture_json.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureEmailImportReport {
    pub inserted_or_existing_records: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureEmailImportWithRecordsReport {
    pub inserted_or_existing_records: usize,
    pub raw_records: Vec<StoredRawCommunicationRecord>,
}

pub async fn import_fixture_email_messages(
    store: &dyn CommunicationRawEvidenceCommandPort,
    request: &FixtureEmailImportRequest,
) -> Result<FixtureEmailImportReport, FixtureEmailImportError> {
    let report = import_fixture_email_messages_with_records(store, request).await?;

    Ok(FixtureEmailImportReport {
        inserted_or_existing_records: report.inserted_or_existing_records,
    })
}

pub async fn import_fixture_email_messages_with_records(
    store: &dyn CommunicationRawEvidenceCommandPort,
    request: &FixtureEmailImportRequest,
) -> Result<FixtureEmailImportWithRecordsReport, FixtureEmailImportError> {
    let messages = parse_fixture_email_messages(&request.fixture_json)?;
    let mut inserted_or_existing_records = 0;
    let mut raw_records = Vec::new();

    for message in messages {
        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &request.account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
            ),
            &request.account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &request.import_batch_id,
            json!({
                "subject": message.subject,
                "from": message.from,
                "to": message.to,
                "body_text": message.body_text
            }),
        )
        .provenance(json!({"source": "fixture_email"}));

        if let Some(sent_at) = message.sent_at {
            raw_record = raw_record.occurred_at(sent_at);
        }

        raw_records.push(store.record_raw_source(&raw_record).await?);
        inserted_or_existing_records += 1;
    }

    Ok(FixtureEmailImportWithRecordsReport {
        inserted_or_existing_records,
        raw_records,
    })
}

fn raw_record_id(account_id: &str, record_kind: &str, provider_record_id: &str) -> String {
    let mut encoded = String::from("raw:v1:");
    append_raw_record_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, record_kind);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_raw_record_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

#[derive(Debug, Error)]
pub enum FixtureEmailImportError {
    #[error(transparent)]
    Source(#[from] FixtureEmailSourceError),

    #[error(transparent)]
    CommunicationEvidence(#[from] CommunicationEvidencePortError),
}
