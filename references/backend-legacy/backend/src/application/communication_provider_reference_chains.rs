use std::collections::HashSet;

use crate::application::communication_provider_reference_mapping::{
    map_forward_reference, map_reply_reference,
};
use crate::application::communication_provider_reference_reads::list_reference_summaries;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::message_references::{
    TelegramForwardChainResponse, TelegramForwardRef, TelegramReplyChainResponse, TelegramReplyRef,
};
use makosh_communications_api::canonical::CanonicalMessageReadPort;

const MAX_DEPTH: usize = 16;
const MAX_EDGES: usize = 128;

async fn reply_refs_by_target(
    reads: &dyn CanonicalMessageReadPort,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    reads
        .list_reply_references_by_target(message_id)
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))
        .and_then(map_reply_reference)
}

async fn reply_refs_by_source(
    reads: &dyn CanonicalMessageReadPort,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    reads
        .list_reply_references_by_source(message_id)
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))
        .and_then(map_reply_reference)
}

pub(crate) async fn reply_chain(
    reads: &dyn CanonicalMessageReadPort,
    message_id: &str,
) -> Result<TelegramReplyChainResponse, TelegramError> {
    let mut replies = Vec::new();
    let mut reply_to = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));
    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }
        for mut item in reply_refs_by_target(reads, &current_id).await? {
            let next_id = item.source_message_id.clone();
            if !visited.insert(next_id.clone()) {
                continue;
            }
            item.reply_depth = (depth + 1) as i32;
            replies.push(item);
            if replies.len() >= MAX_EDGES {
                break;
            }
            queue.push_back((next_id, depth + 1));
        }
    }
    visited.clear();
    queue.clear();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));
    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }
        for mut item in reply_refs_by_source(reads, &current_id).await? {
            let next_id = item.target_message_id.clone();
            if !visited.insert(next_id.clone()) {
                continue;
            }
            item.reply_depth = (depth + 1) as i32;
            reply_to.push(item);
            if reply_to.len() >= MAX_EDGES {
                break;
            }
            queue.push_back((next_id, depth + 1));
        }
    }
    let summary_ids = replies
        .iter()
        .chain(reply_to.iter())
        .flat_map(|item| {
            [
                item.source_message_id.clone(),
                item.target_message_id.clone(),
            ]
        })
        .collect();
    let summaries = list_reference_summaries(reads, summary_ids).await?;
    for item in replies.iter_mut().chain(reply_to.iter_mut()) {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
        item.target_message_summary = summaries.get(&item.target_message_id).cloned();
    }
    Ok(TelegramReplyChainResponse {
        message_id: message_id.to_owned(),
        replies,
        reply_to,
    })
}

pub(crate) async fn forward_chain(
    reads: &dyn CanonicalMessageReadPort,
    message_id: &str,
) -> Result<TelegramForwardChainResponse, TelegramError> {
    let mut forwards: Vec<TelegramForwardRef> = Vec::new();
    let mut visited = HashSet::new();
    let mut current_id = Some(message_id.to_owned());
    let mut depth = 0usize;
    while let Some(source_message_id) = current_id {
        if depth >= MAX_DEPTH || !visited.insert(source_message_id.clone()) {
            break;
        }
        let rows = reads
            .list_forward_references_by_source(&source_message_id)
            .await
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;
        let Some(mut item) = map_forward_reference(rows)?.into_iter().next() else {
            break;
        };
        item.forward_depth = (depth + 1) as i32;
        current_id = item
            .metadata
            .get("target_message_id")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);
        forwards.push(item);
        if forwards.len() >= MAX_EDGES {
            break;
        }
        depth += 1;
    }
    let summary_ids = forwards
        .iter()
        .map(|item| item.source_message_id.clone())
        .collect();
    let summaries = list_reference_summaries(reads, summary_ids).await?;
    for item in &mut forwards {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
    }
    Ok(TelegramForwardChainResponse {
        message_id: message_id.to_owned(),
        forwards,
    })
}
