//! Generated Communications metadata-query port.

use hermes_communications_api::{
    CommunicationConversationIdV1, CommunicationMessageIdV1, CommunicationObservationIdV1,
    CommunicationSourceCursorV1, CommunicationsClientError, GetCommunicationConversationV1,
    GetCommunicationEvidenceV1, GetCommunicationMessageV1, ListCommunicationAccountsV1,
    ListCommunicationConversationsV1, ListConversationMessagesV1, ListConversationParticipantsV1,
    ListMessageAttachmentAnchorsV1, ListMessageEvidenceV1, ListMessageReferencesV1,
    query_wire::{
        CommunicationsQueryRequestV1, CommunicationsQueryResponseV1,
        communications_query_request_v1::Operation,
        communications_query_response_v1::Result as QueryResult,
    },
};
use hermes_communications_persistence::CommunicationsDurablePersistence;
use hermes_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;
use std::os::unix::net::UnixStream;

use crate::{
    query::{
        get_communication_conversation, get_communication_evidence, get_communication_message,
        list_communication_accounts, list_communication_conversations, list_conversation_messages,
        list_conversation_participants, list_message_attachment_anchors, list_message_evidence,
        list_message_references,
    },
    search_access::CommunicationsSearchAccessV1,
    search_query::{CommunicationsSearchQueryErrorV1, search_communications_v1},
};

const PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsQueryPortErrorV1 {
    Protocol,
    Unavailable,
}

pub async fn handle_query_request_v1(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsQueryPortErrorV1> {
    let request = CommunicationsQueryRequestV1::decode(bytes)
        .map_err(|_| CommunicationsQueryPortErrorV1::Protocol)?;
    if request.protocol_major != PROTOCOL_MAJOR {
        return Err(CommunicationsQueryPortErrorV1::Protocol);
    }
    let result = match request
        .operation
        .ok_or(CommunicationsQueryPortErrorV1::Protocol)?
    {
        Operation::ListAccounts(request) => {
            let page = list_communication_accounts(
                persistence,
                ListCommunicationAccountsV1 {
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListAccounts(
                hermes_communications_api::query_wire::ListAccountsResponseV1 {
                    accounts: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::ListConversations(request) => {
            let page = list_communication_conversations(
                persistence,
                ListCommunicationConversationsV1 {
                    account_cursor: optional_source_cursor(&request.account_cursor_sha256)?,
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListConversations(
                hermes_communications_api::query_wire::ListConversationsResponseV1 {
                    conversations: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::GetConversation(request) => QueryResult::GetConversation(
            hermes_communications_api::query_wire::GetConversationResponseV1 {
                conversation: Some(
                    (&get_communication_conversation(
                        persistence,
                        GetCommunicationConversationV1 {
                            conversation_id: CommunicationConversationIdV1::new(id16(
                                &request.conversation_id,
                            )?),
                        },
                    )
                    .await
                    .map_err(map_client_error)?)
                        .into(),
                ),
            },
        ),
        Operation::GetMessage(request) => QueryResult::GetMessage(
            hermes_communications_api::query_wire::GetMessageResponseV1 {
                message: Some(
                    (&get_communication_message(
                        persistence,
                        GetCommunicationMessageV1 {
                            message_id: CommunicationMessageIdV1::new(id16(&request.message_id)?),
                        },
                    )
                    .await
                    .map_err(map_client_error)?)
                        .into(),
                ),
            },
        ),
        Operation::ListConversationMessages(request) => {
            let page = list_conversation_messages(
                persistence,
                ListConversationMessagesV1 {
                    conversation_id: CommunicationConversationIdV1::new(id16(
                        &request.conversation_id,
                    )?),
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListConversationMessages(
                hermes_communications_api::query_wire::ListConversationMessagesResponseV1 {
                    messages: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::ListConversationParticipants(request) => {
            let page = list_conversation_participants(
                persistence,
                ListConversationParticipantsV1 {
                    conversation_id: CommunicationConversationIdV1::new(id16(
                        &request.conversation_id,
                    )?),
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListConversationParticipants(
                hermes_communications_api::query_wire::ListConversationParticipantsResponseV1 {
                    participants: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::ListMessageAttachmentAnchors(request) => {
            let page = list_message_attachment_anchors(
                persistence,
                ListMessageAttachmentAnchorsV1 {
                    message_id: CommunicationMessageIdV1::new(id16(&request.message_id)?),
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListMessageAttachmentAnchors(
                hermes_communications_api::query_wire::ListMessageAttachmentAnchorsResponseV1 {
                    anchors: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::ListMessageReferences(request) => {
            let page = list_message_references(
                persistence,
                ListMessageReferencesV1 {
                    message_id: CommunicationMessageIdV1::new(id16(&request.message_id)?),
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListMessageReferences(
                hermes_communications_api::query_wire::ListMessageReferencesResponseV1 {
                    references: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::ListMessageEvidence(request) => {
            let page = list_message_evidence(
                persistence,
                ListMessageEvidenceV1 {
                    message_id: CommunicationMessageIdV1::new(id16(&request.message_id)?),
                    limit: page_limit(request.limit)?,
                    cursor: request.cursor,
                },
            )
            .await
            .map_err(map_client_error)?;
            QueryResult::ListMessageEvidence(
                hermes_communications_api::query_wire::ListMessageEvidenceResponseV1 {
                    evidence: page.items.iter().map(Into::into).collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::SearchCommunications(request) => {
            let page = search_communications_v1(
                persistence,
                search_access,
                control_channel,
                dispatcher,
                &request.query,
                &request.cursor,
                page_limit(request.limit)?,
            )
            .await
            .map_err(map_search_error)?;
            QueryResult::SearchCommunications(
                hermes_communications_api::query_wire::SearchCommunicationsResponseV1 {
                    hits: page
                        .items
                        .into_iter()
                        .map(|hit| {
                            hermes_communications_api::query_wire::CommunicationSearchHitV1 {
                                evidence_id: hit.evidence_id.bytes().to_vec(),
                                message_id: hit.message_id.bytes().to_vec(),
                                conversation_id: hit.conversation_id.bytes().to_vec(),
                                observed_at_unix_seconds: hit.observed_at_unix_seconds,
                                matched_token_count: u32::from(hit.matched_token_count),
                            }
                        })
                        .collect(),
                    next_cursor: page.next_cursor,
                },
            )
        }
        Operation::GetEvidence(request) => {
            let evidence_id = CommunicationObservationIdV1::new(id16(&request.evidence_id)?);
            let evidence =
                get_communication_evidence(persistence, GetCommunicationEvidenceV1 { evidence_id })
                    .await
                    .map_err(map_client_error)?;
            let message_id = persistence
                .canonical_message_id_for_evidence(evidence_id)
                .await
                .map_err(|_| CommunicationsQueryPortErrorV1::Unavailable)?
                .map_or_else(Vec::new, |message_id| message_id.bytes().to_vec());
            QueryResult::GetEvidence(
                hermes_communications_api::query_wire::GetEvidenceResponseV1 {
                    evidence: Some((&evidence).into()),
                    message_id,
                },
            )
        }
    };
    Ok(CommunicationsQueryResponseV1 {
        result: Some(result),
        error_code: String::new(),
    }
    .encode_to_vec())
}

fn page_limit(value: u32) -> Result<u16, CommunicationsQueryPortErrorV1> {
    u16::try_from(value)
        .ok()
        .filter(|value| (1..=100).contains(value))
        .ok_or(CommunicationsQueryPortErrorV1::Protocol)
}

const fn map_client_error(error: CommunicationsClientError) -> CommunicationsQueryPortErrorV1 {
    match error {
        CommunicationsClientError::InvalidCursor
        | CommunicationsClientError::DraftValidationFailed
        | CommunicationsClientError::DuplicateObservation => {
            CommunicationsQueryPortErrorV1::Protocol
        }
        CommunicationsClientError::UnknownCommunication
        | CommunicationsClientError::Unavailable => CommunicationsQueryPortErrorV1::Unavailable,
    }
}

const fn map_search_error(
    error: CommunicationsSearchQueryErrorV1,
) -> CommunicationsQueryPortErrorV1 {
    match error {
        CommunicationsSearchQueryErrorV1::InvalidQuery
        | CommunicationsSearchQueryErrorV1::InvalidCursor => {
            CommunicationsQueryPortErrorV1::Protocol
        }
        CommunicationsSearchQueryErrorV1::Unavailable => {
            CommunicationsQueryPortErrorV1::Unavailable
        }
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsQueryPortErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsQueryPortErrorV1::Protocol)
}

fn optional_source_cursor(
    value: &[u8],
) -> Result<Option<CommunicationSourceCursorV1>, CommunicationsQueryPortErrorV1> {
    if value.is_empty() {
        return Ok(None);
    }
    let cursor: [u8; 32] = value
        .try_into()
        .map_err(|_| CommunicationsQueryPortErrorV1::Protocol)?;
    Ok(Some(CommunicationSourceCursorV1::new(cursor)))
}

#[cfg(test)]
mod tests {
    use super::page_limit;

    #[test]
    fn canonical_read_page_limit_is_bounded() {
        assert_eq!(page_limit(1), Ok(1));
        assert_eq!(page_limit(100), Ok(100));
        assert!(page_limit(0).is_err());
        assert!(page_limit(101).is_err());
    }
}
