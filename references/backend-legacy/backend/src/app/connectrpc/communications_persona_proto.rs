use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::CommunicationPersona;

use crate::domains::communications::personas::CommunicationPersona as DomainCommunicationPersona;

fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(super) fn from_domain(item: DomainCommunicationPersona) -> CommunicationPersona {
    CommunicationPersona {
        persona_id: item.persona_id,
        account_id: item.account_id,
        name: item.name,
        display_name: item.display_name,
        signature: item.signature,
        default_language: item.default_language,
        default_tone: item.default_tone,
        is_default: item.is_default,
        metadata_json: item.metadata.to_string(),
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}
