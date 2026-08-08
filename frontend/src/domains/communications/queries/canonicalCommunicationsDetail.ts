import type {
	AttachmentAnchorSummaryV1,
	ConversationSummaryV1,
	EvidenceSummaryV1,
	MessageReferenceSummaryV1,
	MessageSummaryV1,
	ObservedParticipantSummaryV1,
} from '../../../gen/makosh/communications/query/v1/query_pb'
import {
	getCanonicalConversation,
	getCanonicalMessage,
	listCanonicalConversationParticipants,
	listCanonicalMessageAttachmentAnchors,
	listCanonicalMessageEvidence,
	listCanonicalMessageReferences,
	type CanonicalCommunicationsPage,
} from './canonicalCommunicationsRead'

export type CanonicalCommunicationDetail = {
	message: MessageSummaryV1
	conversation: ConversationSummaryV1
	participants: CanonicalCommunicationsPage<ObservedParticipantSummaryV1>
	attachments: CanonicalCommunicationsPage<AttachmentAnchorSummaryV1>
	references: CanonicalCommunicationsPage<MessageReferenceSummaryV1>
	evidence: CanonicalCommunicationsPage<EvidenceSummaryV1>
}

export async function loadCanonicalCommunicationDetail(
	messageId: Uint8Array,
): Promise<CanonicalCommunicationDetail> {
	const message = await getCanonicalMessage(messageId)
	const [conversation, participants, attachments, references, evidence] = await Promise.all([
		getCanonicalConversation(message.conversationId),
		listCanonicalConversationParticipants(message.conversationId),
		listCanonicalMessageAttachmentAnchors(message.messageId),
		listCanonicalMessageReferences(message.messageId),
		listCanonicalMessageEvidence(message.messageId),
	])
	return { message, conversation, participants, attachments, references, evidence }
}
