import { ZulipCredentialBindingStateV1 } from '../../../gen/makosh/zulip/account/v1/client_pb'
import {
	ZulipConversationKindV1,
	ZulipHistoryStateV1,
	ZulipOperationalEventKindV1,
	type ZulipAccountStatusV1,
	type ZulipConversationV1,
	type ZulipMessageV1,
	type ZulipOperationalEventV1,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import type { ZulipOperationalAccount } from '../queries/zulipOperationalAccounts'

export type ZulipOperationalReadState = 'blocked' | 'empty' | 'error' | 'loading' | 'ready'

export type ZulipOperationalReadModel = {
	canQuery: boolean
	state: ZulipOperationalReadState
	statusMessage: string
	accounts: readonly ZulipAccountOption[]
	selectedAccountId: string
	selectedConversationId: string
	searchQuery: string
	accountStatus: ZulipAccountStatusCard | null
	conversations: readonly ZulipConversationRow[]
	messages: readonly ZulipMessageRow[]
	events: readonly ZulipEventRow[]
	searchResults: readonly ZulipMessageRow[]
	hasMoreConversations: boolean
	hasMoreMessages: boolean
	hasMoreEvents: boolean
	hasMoreSearchResults: boolean
}

export type ZulipAccountOption = {
	id: string
	label: string
}

export type ZulipAccountStatusCard = {
	projectionState: string
	historyState: string
	credentialState: string
	oldestMessageId: string
	lastProviderEventId: string
	latestSequence: string
	credentialRevision: string
	bindingRevision: string
	runtimeGeneration: string
}

export type ZulipConversationRow = {
	id: string
	title: string
	kind: string
	meta: string
	selected: boolean
}

export type ZulipMessageRow = {
	id: string
	conversationId: string
	sender: string
	content: string
	meta: string
	attachments: string
	reactions: string
	deleted: boolean
}

export type ZulipEventRow = {
	id: string
	kind: string
	summary: string
	meta: string
}

export function buildZulipOperationalReadModel(input: {
	canQuery: boolean
	state: ZulipOperationalReadState
	statusMessage: string
	accounts: readonly ZulipOperationalAccount[]
	selectedAccountId: string
	selectedConversationId: string
	searchQuery: string
	accountStatus: ZulipAccountStatusV1 | undefined
	conversations: readonly ZulipConversationV1[]
	messages: readonly ZulipMessageV1[]
	events: readonly ZulipOperationalEventV1[]
	searchResults: readonly ZulipMessageV1[]
	hasMoreConversations: boolean
	hasMoreMessages: boolean
	hasMoreEvents: boolean
	hasMoreSearchResults: boolean
}): ZulipOperationalReadModel {
	return {
		canQuery: input.canQuery,
		state: input.state,
		statusMessage: input.statusMessage,
		accounts: input.accounts.map(({ accountId }) => ({ id: accountId, label: accountId })),
		selectedAccountId: input.selectedAccountId,
		selectedConversationId: input.selectedConversationId,
		searchQuery: input.searchQuery,
		accountStatus: input.accountStatus
			? buildAccountStatusCard(input.accountStatus)
			: null,
		conversations: input.conversations.map((conversation) => (
			buildConversationRow(
				conversation,
				conversation.providerConversationId === input.selectedConversationId,
			)
		)),
		messages: input.messages.map(buildMessageRow),
		events: input.events.map(buildEventRow),
		searchResults: input.searchResults.map(buildMessageRow),
		hasMoreConversations: input.hasMoreConversations,
		hasMoreMessages: input.hasMoreMessages,
		hasMoreEvents: input.hasMoreEvents,
		hasMoreSearchResults: input.hasMoreSearchResults,
	}
}

export function buildAccountStatusCard(
	status: ZulipAccountStatusV1,
): ZulipAccountStatusCard {
	return {
		projectionState: status.projectionReady ? 'Ready' : 'Partial',
		historyState: historyStateLabel(status.historyState),
		credentialState: credentialStateLabel(status.credentialState),
		oldestMessageId: status.oldestProviderMessageId ?? 'Not available',
		lastProviderEventId: optionalBigInt(status.lastProviderEventId),
		latestSequence: `${status.latestEventSequence}`,
		credentialRevision: optionalBigInt(status.credentialRevision),
		bindingRevision: `${status.bindingRevision}`,
		runtimeGeneration: optionalBigInt(status.appliedRuntimeGeneration),
	}
}

export function buildConversationRow(
	conversation: ZulipConversationV1,
	selected: boolean,
): ZulipConversationRow {
	const isStream = conversation.kind
		=== ZulipConversationKindV1.ZULIP_CONVERSATION_KIND_STREAM_TOPIC
	return {
		id: conversation.providerConversationId,
		title: isStream
			? [conversation.streamName, conversation.topic].filter(Boolean).join(' / ')
				|| conversation.providerConversationId
			: conversation.directRecipientId ?? conversation.providerConversationId,
		kind: isStream ? 'Stream topic' : 'Direct',
		meta: `Latest sequence ${conversation.latestEventSequence}`,
		selected,
	}
}

export function buildMessageRow(message: ZulipMessageV1): ZulipMessageRow {
	return {
		id: message.providerMessageId,
		conversationId: message.providerConversationId,
		sender: message.isOutgoing ? 'You' : message.senderId,
		content: message.deleted ? 'Message deleted.' : message.content ?? 'No content.',
		meta: [
			formatUnixSeconds(message.sentAtUnixSeconds),
			message.editedAtUnixSeconds !== undefined ? 'Edited' : '',
			`sequence ${message.lastEventSequence}`,
		].filter(Boolean).join(' · '),
		attachments: message.attachment
			.map((attachment) => attachment.filename ?? attachment.providerAttachmentId)
			.join(', '),
		reactions: message.reaction
			.map((reaction) => `${reaction.emojiName} · ${reaction.actorId}`)
			.join(', '),
		deleted: message.deleted,
	}
}

export function buildEventRow(event: ZulipOperationalEventV1): ZulipEventRow {
	return {
		id: `${event.providerEventId}:${event.providerMessageId}`,
		kind: eventKindLabel(event.kind),
		summary: event.content
			?? event.topic
			?? event.reaction?.emojiName
			?? event.providerMessageId,
		meta: [
			event.actorId,
			formatUnixSeconds(event.observedAtUnixSeconds),
		].filter(Boolean).join(' · '),
	}
}

export function eventKindLabel(kind: ZulipOperationalEventKindV1): string {
	switch (kind) {
		case ZulipOperationalEventKindV1.ZULIP_OPERATIONAL_EVENT_KIND_MESSAGE_UPSERTED:
			return 'Message upserted'
		case ZulipOperationalEventKindV1.ZULIP_OPERATIONAL_EVENT_KIND_MESSAGE_UPDATED:
			return 'Message updated'
		case ZulipOperationalEventKindV1.ZULIP_OPERATIONAL_EVENT_KIND_MESSAGE_DELETED:
			return 'Message deleted'
		case ZulipOperationalEventKindV1.ZULIP_OPERATIONAL_EVENT_KIND_REACTION_ADDED:
			return 'Reaction added'
		case ZulipOperationalEventKindV1.ZULIP_OPERATIONAL_EVENT_KIND_REACTION_REMOVED:
			return 'Reaction removed'
		default:
			return 'Unknown event'
	}
}

function historyStateLabel(state: ZulipHistoryStateV1): string {
	switch (state) {
		case ZulipHistoryStateV1.ZULIP_HISTORY_STATE_NOT_STARTED: return 'Not started'
		case ZulipHistoryStateV1.ZULIP_HISTORY_STATE_SYNCING: return 'Syncing'
		case ZulipHistoryStateV1.ZULIP_HISTORY_STATE_READY: return 'Ready'
		case ZulipHistoryStateV1.ZULIP_HISTORY_STATE_DEGRADED: return 'Degraded'
		default: return 'Unknown'
	}
}

function credentialStateLabel(state: ZulipCredentialBindingStateV1): string {
	switch (state) {
		case ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_UNCONFIGURED:
			return 'Unconfigured'
		case ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_PENDING_RESTART:
			return 'Pending restart'
		case ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_ACTIVE:
			return 'Active'
		case ZulipCredentialBindingStateV1.ZULIP_CREDENTIAL_BINDING_STATE_RETIRED:
			return 'Retired'
		default:
			return 'Unknown'
	}
}

function optionalBigInt(value: bigint | undefined): string {
	return value === undefined ? 'Not available' : `${value}`
}

function formatUnixSeconds(value: bigint | undefined): string {
	if (value === undefined || value <= 0n || value > 8_640_000_000_000n) {
		return 'Not recorded'
	}
	const date = new Date(Number(value) * 1_000)
	return Number.isNaN(date.getTime())
		? 'Not recorded'
		: date.toISOString().replace('T', ' ').replace('.000Z', ' UTC')
}
