import type {
	AccountSummaryV1,
	AttachmentAnchorSummaryV1,
	ConversationSummaryV1,
	EvidenceSummaryV1,
	MessageReferenceSummaryV1,
	MessageSummaryV1,
	ObservedParticipantSummaryV1,
} from '../../../gen/makosh/communications/query/v1/query_pb'
import { getCommunicationsQueryConnectClient } from '../../../platform/connect/communicationsQueryClient'

const MAX_PAGE_LIMIT = 100
const MAX_CURSOR_BYTES = 64
const CANONICAL_ID_BYTES = 16

export type CanonicalCommunicationsPage<T> = {
	items: readonly T[]
	nextCursor: Uint8Array
}

export async function listCanonicalCommunicationAccounts(
	limit = 50,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<AccountSummaryV1>> {
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: { case: 'listAccounts', value: { limit, cursor } },
	})
	if (response.errorCode || response.result.case !== 'listAccounts') {
		throw new Error('Canonical communication accounts are unavailable')
	}
	return page(response.result.value.accounts, response.result.value.nextCursor)
}

export async function listCanonicalConversations(
	accountCursorSha256: Uint8Array,
	limit = 100,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<ConversationSummaryV1>> {
	assertDigest('account cursor', accountCursorSha256)
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listConversations',
			value: { accountCursorSha256, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'listConversations') {
		throw new Error('Canonical conversations are unavailable')
	}
	return page(response.result.value.conversations, response.result.value.nextCursor)
}

export async function getCanonicalConversation(
	conversationId: Uint8Array,
): Promise<ConversationSummaryV1> {
	assertIdentifier('conversation ID', conversationId)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: { case: 'getConversation', value: { conversationId } },
	})
	if (
		response.errorCode
		|| response.result.case !== 'getConversation'
		|| !response.result.value.conversation
	) {
		throw new Error('Canonical conversation is unavailable')
	}
	return response.result.value.conversation
}

export async function getCanonicalMessage(messageId: Uint8Array): Promise<MessageSummaryV1> {
	assertIdentifier('message ID', messageId)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: { case: 'getMessage', value: { messageId } },
	})
	if (
		response.errorCode
		|| response.result.case !== 'getMessage'
		|| !response.result.value.message
	) {
		throw new Error('Canonical message is unavailable')
	}
	return response.result.value.message
}

export async function resolveCanonicalMessageIdForEvidence(
	evidenceId: Uint8Array,
): Promise<Uint8Array> {
	assertIdentifier('evidence ID', evidenceId)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: { case: 'getEvidence', value: { evidenceId } },
	})
	if (
		response.errorCode
		|| response.result.case !== 'getEvidence'
		|| response.result.value.messageId.byteLength !== CANONICAL_ID_BYTES
	) {
		throw new Error('Canonical message for evidence is unavailable')
	}
	return new Uint8Array(response.result.value.messageId)
}

export async function listCanonicalConversationMessages(
	conversationId: Uint8Array,
	limit = 100,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<MessageSummaryV1>> {
	assertIdentifier('conversation ID', conversationId)
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listConversationMessages',
			value: { conversationId, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'listConversationMessages') {
		throw new Error('Canonical communication messages are unavailable')
	}
	return page(response.result.value.messages, response.result.value.nextCursor)
}

export async function listCanonicalConversationParticipants(
	conversationId: Uint8Array,
	limit = 100,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<ObservedParticipantSummaryV1>> {
	assertIdentifier('conversation ID', conversationId)
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listConversationParticipants',
			value: { conversationId, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'listConversationParticipants') {
		throw new Error('Canonical conversation participants are unavailable')
	}
	return page(response.result.value.participants, response.result.value.nextCursor)
}

export async function listCanonicalMessageAttachmentAnchors(
	messageId: Uint8Array,
	limit = 100,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<AttachmentAnchorSummaryV1>> {
	assertIdentifier('message ID', messageId)
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listMessageAttachmentAnchors',
			value: { messageId, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'listMessageAttachmentAnchors') {
		throw new Error('Canonical message attachment anchors are unavailable')
	}
	return page(response.result.value.anchors, response.result.value.nextCursor)
}

export async function listCanonicalMessageReferences(
	messageId: Uint8Array,
	limit = 100,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<MessageReferenceSummaryV1>> {
	assertIdentifier('message ID', messageId)
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listMessageReferences',
			value: { messageId, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'listMessageReferences') {
		throw new Error('Canonical message references are unavailable')
	}
	return page(response.result.value.references, response.result.value.nextCursor)
}

export async function listCanonicalMessageEvidence(
	messageId: Uint8Array,
	limit = 100,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<EvidenceSummaryV1>> {
	assertIdentifier('message ID', messageId)
	assertPage(limit, cursor)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listMessageEvidence',
			value: { messageId, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'listMessageEvidence') {
		throw new Error('Canonical message evidence is unavailable')
	}
	return page(response.result.value.evidence, response.result.value.nextCursor)
}

function page<T>(items: readonly T[], nextCursor: Uint8Array): CanonicalCommunicationsPage<T> {
	assertCursor(nextCursor)
	return { items, nextCursor }
}

function assertPage(limit: number, cursor: Uint8Array): void {
	if (!Number.isInteger(limit) || limit < 1 || limit > MAX_PAGE_LIMIT) {
		throw new RangeError(`Canonical Communications page limit must be between 1 and ${MAX_PAGE_LIMIT}`)
	}
	assertCursor(cursor)
}

function assertCursor(cursor: Uint8Array): void {
	if (cursor.byteLength > MAX_CURSOR_BYTES) {
		throw new RangeError(`Canonical Communications cursor must not exceed ${MAX_CURSOR_BYTES} bytes`)
	}
}

function assertDigest(label: string, value: Uint8Array): void {
	if (value.byteLength !== 32) {
		throw new RangeError(`Canonical Communications ${label} must be a SHA-256 digest`)
	}
}

function assertIdentifier(label: string, value: Uint8Array): void {
	if (value.byteLength !== CANONICAL_ID_BYTES) {
		throw new RangeError(`Canonical Communications ${label} must be ${CANONICAL_ID_BYTES} bytes`)
	}
}
