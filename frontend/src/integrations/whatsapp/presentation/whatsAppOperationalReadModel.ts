import type {
	WhatsAppOperationalRuntimeStatusV1,
} from '../../../gen/makosh/whatsapp/operational/v1/client_pb'
import type {
	WhatsAppDialog,
	WhatsAppMessage,
	WhatsAppParticipant,
	WhatsAppProviderEventV1,
} from '../../../gen/makosh/whatsapp/v1/client_pb'
import type { WhatsAppOperationalAccount } from '../queries/whatsAppOperationalAccounts'

export type WhatsAppOperationalReadState = 'blocked' | 'empty' | 'error' | 'loading' | 'ready'

export type WhatsAppOperationalReadModel = {
	canQuery: boolean
	state: WhatsAppOperationalReadState
	statusMessage: string
	accounts: readonly WhatsAppAccountOption[]
	selectedAccountId: string
	selectedChatId: string
	searchQuery: string
	runtime: WhatsAppRuntimeCard | null
	dialogs: readonly WhatsAppDialogRow[]
	messages: readonly WhatsAppMessageRow[]
	participants: readonly WhatsAppParticipantRow[]
	events: readonly WhatsAppEventRow[]
	searchResults: readonly WhatsAppMessageRow[]
	hasMoreDialogs: boolean
	hasMoreMessages: boolean
	hasMoreParticipants: boolean
	hasMoreEvents: boolean
	hasMoreSearchResults: boolean
}

export type WhatsAppAccountOption = {
	id: string
	label: string
}

export type WhatsAppRuntimeCard = {
	state: string
	projectionState: string
	latestSequence: string
}

export type WhatsAppDialogRow = {
	id: string
	title: string
	meta: string
	flags: string
	selected: boolean
}

export type WhatsAppMessageRow = {
	id: string
	chatId: string
	sender: string
	text: string
	meta: string
	deliveryState: string
}

export type WhatsAppParticipantRow = {
	id: string
	displayName: string
	meta: string
	isSelf: boolean
}

export type WhatsAppEventRow = {
	id: string
	kind: string
	summary: string
}

export function buildWhatsAppOperationalReadModel(input: {
	canQuery: boolean
	state: WhatsAppOperationalReadState
	statusMessage: string
	accounts: readonly WhatsAppOperationalAccount[]
	selectedAccountId: string
	selectedChatId: string
	searchQuery: string
	runtime: WhatsAppOperationalRuntimeStatusV1 | undefined
	dialogs: readonly WhatsAppDialog[]
	messages: readonly WhatsAppMessage[]
	participants: readonly WhatsAppParticipant[]
	events: readonly WhatsAppProviderEventV1[]
	searchResults: readonly WhatsAppMessage[]
	hasMoreDialogs: boolean
	hasMoreMessages: boolean
	hasMoreParticipants: boolean
	hasMoreEvents: boolean
	hasMoreSearchResults: boolean
}): WhatsAppOperationalReadModel {
	return {
		canQuery: input.canQuery,
		state: input.state,
		statusMessage: input.statusMessage,
		accounts: input.accounts.map(({ accountId }) => ({ id: accountId, label: accountId })),
		selectedAccountId: input.selectedAccountId,
		selectedChatId: input.selectedChatId,
		searchQuery: input.searchQuery,
		runtime: input.runtime ? {
			state: input.runtime.runtimeState || 'unknown',
			projectionState: input.runtime.projectionReady ? 'Ready' : 'Resync required',
			latestSequence: `${input.runtime.latestEventSequence}`,
		} : null,
		dialogs: input.dialogs.map((dialog) => buildDialogRow(
			dialog,
			dialog.providerChatId === input.selectedChatId,
		)),
		messages: input.messages.map(buildMessageRow),
		participants: input.participants.map(buildParticipantRow),
		events: input.events.map((event, index) => buildEventRow(event, index)),
		searchResults: input.searchResults.map(buildMessageRow),
		hasMoreDialogs: input.hasMoreDialogs,
		hasMoreMessages: input.hasMoreMessages,
		hasMoreParticipants: input.hasMoreParticipants,
		hasMoreEvents: input.hasMoreEvents,
		hasMoreSearchResults: input.hasMoreSearchResults,
	}
}

export function buildDialogRow(
	dialog: WhatsAppDialog,
	selected: boolean,
): WhatsAppDialogRow {
	const flags = [
		dialog.isPinned ? 'Pinned' : '',
		dialog.isMuted ? 'Muted' : '',
		dialog.isArchived ? 'Archived' : '',
		dialog.isUnread ? 'Unread' : '',
	].filter(Boolean)
	return {
		id: dialog.providerChatId,
		title: dialog.title || dialog.providerChatId,
		meta: `${dialog.kind || 'chat'} · ${dialog.unreadCount ?? 0n} unread · ${formatUnixSeconds(dialog.observedAtUnixSeconds)}`,
		flags: flags.join(' · ') || 'Active',
		selected,
	}
}

export function buildMessageRow(message: WhatsAppMessage): WhatsAppMessageRow {
	return {
		id: message.providerMessageId,
		chatId: message.providerChatId,
		sender: message.senderDisplayName || message.senderId || 'Unknown sender',
		text: message.text || 'No text content.',
		meta: formatUnixSeconds(message.occurredAtUnixSeconds),
		deliveryState: message.deliveryState || 'unknown',
	}
}

export function buildParticipantRow(
	participant: WhatsAppParticipant,
): WhatsAppParticipantRow {
	return {
		id: participant.providerIdentityId,
		displayName: participant.displayName || participant.providerIdentityId,
		meta: `${participant.role || 'participant'} · ${participant.status || 'unknown'} · ${formatUnixSeconds(participant.observedAtUnixSeconds)}`,
		isSelf: participant.isSelf,
	}
}

export function buildEventRow(
	event: WhatsAppProviderEventV1,
	index: number,
): WhatsAppEventRow {
	return {
		id: `${event.event.case ?? 'unknown'}-${index}`,
		kind: providerEventLabel(event),
		summary: providerEventSummary(event),
	}
}

export function providerEventLabel(event: WhatsAppProviderEventV1): string {
	switch (event.event.case) {
		case 'runtimeStateChanged': return 'Runtime state'
		case 'messageObserved': return 'Message observed'
		case 'messageEdited': return 'Message edited'
		case 'messageDeleted': return 'Message deleted'
		case 'receiptChanged': return 'Receipt changed'
		case 'reactionChanged': return 'Reaction changed'
		case 'dialogObserved': return 'Dialog observed'
		case 'participantObserved': return 'Participant observed'
		case 'participantRemoved': return 'Participant removed'
		case 'presenceChanged': return 'Presence changed'
		case 'callObserved': return 'Call observed'
		case 'statusObserved': return 'Status observed'
		case 'statusViewObserved': return 'Status view observed'
		case 'statusDeleted': return 'Status deleted'
		case 'mediaObserved': return 'Media observed'
		case 'sessionStateChanged': return 'Session state'
		case 'commandResultObserved': return 'Command result'
		default: return 'Unknown event'
	}
}

function providerEventSummary(event: WhatsAppProviderEventV1): string {
	switch (event.event.case) {
		case 'runtimeStateChanged':
			return event.event.value.state || 'Runtime state changed.'
		case 'messageObserved':
			return `${event.event.value.senderDisplayName || event.event.value.senderId || 'Unknown sender'} · ${event.event.value.text || 'No text content.'}`
		case 'dialogObserved':
			return event.event.value.title || event.event.value.providerChatId
		case 'participantObserved':
			return event.event.value.displayName || event.event.value.providerIdentityId
		default:
			return providerEventLabel(event)
	}
}

function formatUnixSeconds(value: bigint): string {
	if (value <= 0n || value > 8_640_000_000_000n) return 'Not recorded'
	const date = new Date(Number(value) * 1_000)
	if (Number.isNaN(date.getTime())) return 'Not recorded'
	return date.toISOString().replace('T', ' ').replace('.000Z', ' UTC')
}
