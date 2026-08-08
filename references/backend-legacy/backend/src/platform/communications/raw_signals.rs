use makosh_events_api::NewEventEnvelope;
use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};

use makosh_communications_api::evidence::StoredRawCommunicationRecord;

use makosh_observations_postgres::store::observation_captured_event_id;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRawSignalSource {
    Mail,
    Telegram,
    Whatsapp,
}

impl CommunicationRawSignalSource {
    fn source_code(self) -> &'static str {
        match self {
            Self::Mail => "mail",
            Self::Telegram => "telegram",
            Self::Whatsapp => "whatsapp",
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Mail => "signal.raw.mail.message.observed",
            Self::Telegram => "signal.raw.telegram.message.observed",
            Self::Whatsapp => "signal.raw.whatsapp.message.observed",
        }
    }

    fn event_id_prefix(self) -> &'static str {
        match self {
            Self::Mail => "mail",
            Self::Telegram => "telegram",
            Self::Whatsapp => "whatsapp",
        }
    }
}

pub fn build_communication_raw_signal_event(
    source: CommunicationRawSignalSource,
    raw_record: &StoredRawCommunicationRecord,
    raw_blob_root: Option<&Path>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source_code = source.source_code();
    let mut provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });
    if let Some(root) = raw_blob_root.and_then(Path::to_str) {
        provenance["blob_root"] = json!(root);
    }

    NewEventEnvelope::builder(
        raw_signal_event_id(source, &raw_record.raw_record_id),
        source.event_type(),
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": source_code,
            "source_id": raw_record.raw_record_id,
            "account_id": raw_record.account_id,
        }),
        json!({
            "kind": "communication_raw_record",
            "source_code": source_code,
            "raw_record_id": raw_record.raw_record_id,
            "account_id": raw_record.account_id,
            "provider_record_id": raw_record.provider_record_id,
            "record_kind": raw_record.record_kind,
        }),
    )
    .payload(raw_record.payload.clone())
    .provenance(provenance)
    .causation_id(observation_captured_event_id(&raw_record.observation_id))
    .correlation_id(raw_record.observation_id.clone())
    .build()
}

fn raw_signal_event_id(source: CommunicationRawSignalSource, raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!(
        "evt_signal_raw_{}_{:x}",
        source.event_id_prefix(),
        hasher.finalize()
    )
}
