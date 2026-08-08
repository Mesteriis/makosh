use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self, snapshots::TelegramTdlibChatMemberSnapshot};
use makosh_provider_telegram::tdlib::chats::{
    get_basic_group, get_basic_group_full_info, get_supergroup_administrators,
    get_supergroup_members,
};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

const TDLIB_SUPERGROUP_MEMBER_PAGE_LIMIT: i32 = 100;

pub(super) fn actor_get_supergroup_members(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    actor_get_supergroup_members_with_filter(
        client,
        supergroup_id,
        limit,
        "recent",
        get_supergroup_members,
    )
}

pub(super) fn actor_get_supergroup_administrators(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    actor_get_supergroup_members_with_filter(
        client,
        supergroup_id,
        limit,
        "administrators",
        get_supergroup_administrators,
    )
}

fn actor_get_supergroup_members_with_filter(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
    filter_name: &str,
    request_builder: fn(i64, i32, i32, &str) -> serde_json::Value,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let target_limit = limit.clamp(1, 1000);
    let mut offset = 0_i32;
    let mut items = Vec::new();
    let mut seen_member_ids = std::collections::HashSet::new();

    while items.len() < target_limit as usize {
        let remaining = target_limit - items.len() as i32;
        let page_limit = remaining.clamp(1, TDLIB_SUPERGROUP_MEMBER_PAGE_LIMIT);
        let extra = format!("makosh-supergroup-members-{filter_name}-{supergroup_id}-{offset}");
        client.send_json(&request_builder(supergroup_id, offset, page_limit, &extra))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        let page_items = tdjson::parsing::participants::parse_tdlib_chat_member_list(&response)?;
        if page_items.is_empty() {
            break;
        }
        let page_len = page_items.len();
        for item in page_items {
            if seen_member_ids.insert(item.provider_member_id.clone()) {
                items.push(item);
            }
        }
        if page_len < page_limit as usize {
            break;
        }
        offset += page_len as i32;
    }

    Ok(items)
}

pub(super) fn actor_get_basic_group_members(
    client: &TdJsonClient,
    basic_group_id: i64,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let group_extra = format!("makosh-basic-group-{basic_group_id}");
    client.send_json(&get_basic_group(basic_group_id, &group_extra))?;
    let group_response = receive_tdlib_extra(client, &group_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&group_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    let full_info_extra = format!("makosh-basic-group-full-info-{basic_group_id}");
    client.send_json(&get_basic_group_full_info(basic_group_id, &full_info_extra))?;
    let full_info_response = receive_tdlib_extra(client, &full_info_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&full_info_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    tdjson::parsing::participants::parse_tdlib_basic_group_member_list(&full_info_response)
}
