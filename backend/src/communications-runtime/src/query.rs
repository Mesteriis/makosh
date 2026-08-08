//! Communications runtime composition for provider-neutral evidence reads.

use makosh_communications_api::{
    CommunicationAccountSummaryV1, CommunicationAttachmentAnchorSummaryV1,
    CommunicationConversationSummaryV1, CommunicationMessageReferenceKindV1,
    CommunicationMessageReferenceSummaryV1, CommunicationMessageSummaryV1,
    CommunicationObservedParticipantSummaryV1, CommunicationSummary, CommunicationsClientError,
    GetCommunicationConversationV1, GetCommunicationEvidenceV1, GetCommunicationMessageV1,
    ListCommunicationAccountsV1, ListCommunicationConversationsV1, ListConversationMessagesV1,
    ListConversationParticipantsV1, ListMessageAttachmentAnchorsV1, ListMessageEvidenceV1,
    ListMessageReferencesV1,
};
use makosh_communications_persistence::{CanonicalReadPageV1, CommunicationsDurablePersistence};

use crate::canonical_read_cursor::{
    CanonicalReadCursorKindV1, decode_descending_cursor_v1, decode_reference_cursor_v1,
    encode_descending_cursor_v1, encode_reference_cursor_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalQueryPageV1<T> {
    pub items: Vec<T>,
    pub next_cursor: Vec<u8>,
}

pub async fn get_communication_evidence(
    persistence: &CommunicationsDurablePersistence,
    request: GetCommunicationEvidenceV1,
) -> Result<CommunicationSummary, CommunicationsClientError> {
    persistence
        .summary(request.evidence_id)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?
        .ok_or(CommunicationsClientError::UnknownCommunication)
}

pub async fn get_communication_conversation(
    persistence: &CommunicationsDurablePersistence,
    request: GetCommunicationConversationV1,
) -> Result<CommunicationConversationSummaryV1, CommunicationsClientError> {
    persistence
        .conversation(request.conversation_id)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?
        .ok_or(CommunicationsClientError::UnknownCommunication)
}

pub async fn get_communication_message(
    persistence: &CommunicationsDurablePersistence,
    request: GetCommunicationMessageV1,
) -> Result<CommunicationMessageSummaryV1, CommunicationsClientError> {
    persistence
        .canonical_message(request.message_id)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?
        .ok_or(CommunicationsClientError::UnknownCommunication)
}

pub async fn list_communication_accounts(
    persistence: &CommunicationsDurablePersistence,
    request: ListCommunicationAccountsV1,
) -> Result<CanonicalQueryPageV1<CommunicationAccountSummaryV1>, CommunicationsClientError> {
    let scope = [b"accounts".as_slice()];
    let after =
        decode_descending_cursor_v1(&request.cursor, CanonicalReadCursorKindV1::Accounts, &scope)
            .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let page = persistence
        .canonical_accounts_page(after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    Ok(descending_page(
        page,
        CanonicalReadCursorKindV1::Accounts,
        &scope,
        |item| (item.last_observed_at_unix_seconds, item.account_id.bytes()),
    ))
}

pub async fn list_communication_conversations(
    persistence: &CommunicationsDurablePersistence,
    request: ListCommunicationConversationsV1,
) -> Result<CanonicalQueryPageV1<CommunicationConversationSummaryV1>, CommunicationsClientError> {
    let account_scope = request
        .account_cursor
        .map(|value| value.bytes().to_vec())
        .unwrap_or_default();
    let scope = [account_scope.as_slice()];
    let after = decode_descending_cursor_v1(
        &request.cursor,
        CanonicalReadCursorKindV1::Conversations,
        &scope,
    )
    .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let page = persistence
        .canonical_conversations_page(request.account_cursor, after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    Ok(descending_page(
        page,
        CanonicalReadCursorKindV1::Conversations,
        &scope,
        |item| {
            (
                item.last_observed_at_unix_seconds,
                item.conversation_id.bytes(),
            )
        },
    ))
}

pub async fn list_conversation_messages(
    persistence: &CommunicationsDurablePersistence,
    request: ListConversationMessagesV1,
) -> Result<CanonicalQueryPageV1<CommunicationMessageSummaryV1>, CommunicationsClientError> {
    let conversation_id = request.conversation_id.bytes();
    let scope = [conversation_id.as_slice()];
    let after =
        decode_descending_cursor_v1(&request.cursor, CanonicalReadCursorKindV1::Messages, &scope)
            .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let page = persistence
        .canonical_messages_page(request.conversation_id, after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    Ok(descending_page(
        page,
        CanonicalReadCursorKindV1::Messages,
        &scope,
        |item| (item.last_observed_at_unix_seconds, item.message_id.bytes()),
    ))
}

pub async fn list_conversation_participants(
    persistence: &CommunicationsDurablePersistence,
    request: ListConversationParticipantsV1,
) -> Result<
    CanonicalQueryPageV1<CommunicationObservedParticipantSummaryV1>,
    CommunicationsClientError,
> {
    let conversation_id = request.conversation_id.bytes();
    let scope = [conversation_id.as_slice()];
    let after = decode_descending_cursor_v1(
        &request.cursor,
        CanonicalReadCursorKindV1::Participants,
        &scope,
    )
    .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let page = persistence
        .canonical_participants_page(request.conversation_id, after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    Ok(descending_page(
        page,
        CanonicalReadCursorKindV1::Participants,
        &scope,
        |item| {
            (
                item.last_observed_at_unix_seconds,
                item.participant_id.bytes(),
            )
        },
    ))
}

pub async fn list_message_attachment_anchors(
    persistence: &CommunicationsDurablePersistence,
    request: ListMessageAttachmentAnchorsV1,
) -> Result<CanonicalQueryPageV1<CommunicationAttachmentAnchorSummaryV1>, CommunicationsClientError>
{
    let message_id = request.message_id.bytes();
    let scope = [message_id.as_slice()];
    let after = decode_descending_cursor_v1(
        &request.cursor,
        CanonicalReadCursorKindV1::AttachmentAnchors,
        &scope,
    )
    .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let page = persistence
        .canonical_attachment_anchors_page(request.message_id, after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    Ok(descending_page(
        page,
        CanonicalReadCursorKindV1::AttachmentAnchors,
        &scope,
        |item| {
            (
                item.last_observed_at_unix_seconds,
                item.attachment_anchor_id.bytes(),
            )
        },
    ))
}

pub async fn list_message_references(
    persistence: &CommunicationsDurablePersistence,
    request: ListMessageReferencesV1,
) -> Result<CanonicalQueryPageV1<CommunicationMessageReferenceSummaryV1>, CommunicationsClientError>
{
    let message_id = request.message_id.bytes();
    let scope = [message_id.as_slice()];
    let after = decode_reference_cursor_v1(&request.cursor, &scope)
        .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let page = persistence
        .canonical_references_page(request.message_id, after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    let next_cursor = if page.has_more {
        page.items
            .last()
            .map(|item| {
                encode_reference_cursor_v1(
                    &scope,
                    item.summary.observed_at_unix_seconds,
                    reference_kind_value(item.summary.kind),
                    item.reference_id,
                )
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(CanonicalQueryPageV1 {
        items: page.items.into_iter().map(|item| item.summary).collect(),
        next_cursor,
    })
}

pub async fn list_message_evidence(
    persistence: &CommunicationsDurablePersistence,
    request: ListMessageEvidenceV1,
) -> Result<CanonicalQueryPageV1<CommunicationSummary>, CommunicationsClientError> {
    let message_id = request.message_id.bytes();
    let scope = [message_id.as_slice()];
    let after = decode_descending_cursor_v1(
        &request.cursor,
        CanonicalReadCursorKindV1::MessageEvidence,
        &scope,
    )
    .map_err(|_| CommunicationsClientError::InvalidCursor)?;
    let ids = persistence
        .canonical_message_evidence_page(request.message_id, after, request.limit)
        .await
        .map_err(|_| CommunicationsClientError::Unavailable)?;
    let mut items = Vec::with_capacity(ids.items.len());
    for evidence_id in ids.items {
        items.push(
            get_communication_evidence(persistence, GetCommunicationEvidenceV1 { evidence_id })
                .await?,
        );
    }
    let page = CanonicalReadPageV1 {
        items,
        has_more: ids.has_more,
    };
    Ok(descending_page(
        page,
        CanonicalReadCursorKindV1::MessageEvidence,
        &scope,
        |item| (item.observed_at_unix_seconds, item.evidence_id.bytes()),
    ))
}

fn descending_page<T>(
    page: CanonicalReadPageV1<T>,
    kind: CanonicalReadCursorKindV1,
    scope: &[&[u8]],
    anchor: impl Fn(&T) -> (i64, [u8; 16]),
) -> CanonicalQueryPageV1<T> {
    let next_cursor = if page.has_more {
        page.items
            .last()
            .map(|item| {
                let (observed_at_unix_seconds, canonical_id) = anchor(item);
                encode_descending_cursor_v1(kind, scope, observed_at_unix_seconds, canonical_id)
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    CanonicalQueryPageV1 {
        items: page.items,
        next_cursor,
    }
}

const fn reference_kind_value(kind: CommunicationMessageReferenceKindV1) -> i16 {
    match kind {
        CommunicationMessageReferenceKindV1::Reply => 1,
        CommunicationMessageReferenceKindV1::Forward => 2,
    }
}
