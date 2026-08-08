use crate::domains::communications::folders::CommunicationFolder;
use crate::domains::communications::saved_searches::CommunicationSavedSearch;
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    CommunicationFolder as ProtoCommunicationFolder,
    CommunicationSavedSearch as ProtoCommunicationSavedSearch,
};
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
pub(super) fn saved_search(item: CommunicationSavedSearch) -> ProtoCommunicationSavedSearch {
    ProtoCommunicationSavedSearch {
        saved_search_id: item.saved_search_id,
        name: item.name,
        description: item.description,
        account_id: item.account_id,
        query: item.query,
        workflow_state: item.workflow_state.map(|state| state.as_str().to_owned()),
        local_state: item.local_state.as_str().to_owned(),
        channel_kind: item.channel_kind,
        is_smart_folder: item.is_smart_folder,
        sort_order: item.sort_order,
        message_count: item.message_count,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}
pub(super) fn folder(item: CommunicationFolder) -> ProtoCommunicationFolder {
    ProtoCommunicationFolder {
        folder_id: item.folder_id,
        account_id: item.account_id,
        name: item.name,
        description: item.description,
        color: item.color,
        sort_order: item.sort_order,
        message_count: item.message_count,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}
