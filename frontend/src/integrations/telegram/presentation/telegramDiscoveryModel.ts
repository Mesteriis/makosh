import type {
	TelegramChatProjection,
	TelegramMessageObservationProjection,
	TelegramMessageProjection,
	TelegramOperationResponse,
	TelegramParticipantProjection,
	TelegramTopicProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import type { TelegramChatContext } from '../api/telegramDiscoveryGateway'
import { resolveTelegramSenderName } from './telegramOperationalPageModel'

export type TelegramDiscoveryResultRow = {
	id: string
	title: string
	detail: string
	kind: 'chat' | 'message'
	providerChatId: string
	providerMessageId: string
}

export type TelegramDiscoveryDetailRow = {
	id: string
	title: string
	detail: string
}

export type TelegramDiscoveryModel = {
	query: string
	statusMessage: string
	pending: boolean
	canQuery: boolean
	results: readonly TelegramDiscoveryResultRow[]
	history: readonly TelegramDiscoveryDetailRow[]
	participants: readonly TelegramDiscoveryDetailRow[]
	topics: readonly TelegramDiscoveryDetailRow[]
	folders: readonly TelegramDiscoveryDetailRow[]
	operations: readonly TelegramDiscoveryDetailRow[]
	chatState: readonly string[]
}

export function buildTelegramDiscoveryResults(input: {
	chats: readonly TelegramChatProjection[]
	messages: readonly TelegramMessageProjection[]
	personaNames?: ReadonlyMap<string, string>
}): readonly TelegramDiscoveryResultRow[] {
	const personaNames = input.personaNames ?? new Map<string, string>()
	return [
		...input.chats.map((chat) => ({
			id: chat.providerChatId,
			title: chat.title || chat.username || chat.providerChatId,
			detail: chat.username ? `@${chat.username} · ${chat.kind}` : chat.kind,
			kind: 'chat' as const,
			providerChatId: chat.providerChatId,
			providerMessageId: '',
		})),
		...input.messages.map((message) => ({
			id: message.messageId,
			title: resolveTelegramSenderName(message, personaNames),
			detail: telegramMessageSummary(message),
			kind: 'message' as const,
			providerChatId: message.providerChatId,
			providerMessageId: message.providerMessageId || message.messageId,
		})),
	]
}

export function buildTelegramHistoryRows(
	items: readonly TelegramMessageObservationProjection[],
	personaNames: ReadonlyMap<string, string> = new Map(),
): readonly TelegramDiscoveryDetailRow[] {
	return items.map((item) => ({
		id: item.providerMessageId,
		title: resolveTelegramSenderName(item, personaNames),
		detail: telegramMessageSummary(item),
	}))
}

function telegramMessageSummary(message: {
	text?: string
	media?: { caption?: string; filename?: string; kind?: string }
}): string {
	if (!message.media) return message.text?.trim() || 'Message'
	return message.media.caption?.trim()
		|| message.media.filename?.trim()
		|| readableMediaKind(message.media.kind)
}

function readableMediaKind(kind?: string): string {
	const normalized = kind?.trim().replaceAll('_', ' ') || ''
	return normalized ? normalized.charAt(0).toUpperCase() + normalized.slice(1) : 'Attachment'
}

export function buildTelegramParticipantRows(
	items: readonly TelegramParticipantProjection[],
): readonly TelegramDiscoveryDetailRow[] {
	return items.map((item) => ({
		id: item.providerMemberId,
		title: item.displayName || item.username || 'Telegram participant',
		detail: [item.role, item.status, item.isOwner ? 'owner' : '', item.isAdmin ? 'admin' : '']
			.filter(Boolean)
			.join(' · '),
	}))
}

export function buildTelegramTopicRows(
	items: readonly TelegramTopicProjection[],
): readonly TelegramDiscoveryDetailRow[] {
	return items.map((item) => ({
		id: item.providerTopicId,
		title: `${item.iconEmoji || '•'} ${item.title}`,
		detail: [
			`${item.unreadCount} unread`,
			item.isPinned ? 'pinned' : '',
			item.isClosed ? 'closed' : 'open',
		].filter(Boolean).join(' · '),
	}))
}

export function buildTelegramOperationRows(
	items: readonly TelegramOperationResponse[],
): readonly TelegramDiscoveryDetailRow[] {
	return items.map((item) => ({
		id: item.operationId,
		title: item.commandKind || 'provider operation',
		detail: [
			item.state || 'unknown',
			item.reconciliation,
			`${item.retryCount}/${item.maxRetries} retries`,
		].filter(Boolean).join(' · '),
	}))
}

export function buildTelegramContextView(context: TelegramChatContext | null): {
	chatState: readonly string[]
	folders: readonly TelegramDiscoveryDetailRow[]
	participants: readonly TelegramDiscoveryDetailRow[]
	topics: readonly TelegramDiscoveryDetailRow[]
} {
	if (!context) {
		return { chatState: [], folders: [], participants: [], topics: [] }
	}
	const state = context.state
	const operational = context.operationalState
	return {
		chatState: [
			state?.unreadCount === undefined ? '' : `${state.unreadCount} unread`,
			state?.unreadMentionCount === undefined ? '' : `${state.unreadMentionCount} mentions`,
			state?.isMarkedAsUnread ? 'marked unread' : '',
			operational?.isArchived ? 'archived' : 'active',
			operational?.isPinned ? 'pinned' : '',
			operational?.isMuted ? 'muted' : '',
		].filter(Boolean),
		folders: context.folders.map((folder) => ({
			id: folder.providerFolderId.toString(),
			title: folder.title || `Folder ${folder.providerFolderId}`,
			detail: `${folder.includedChatId.length} included · ${folder.pinnedChatId.length} pinned`,
		})),
		participants: buildTelegramParticipantRows(context.participants),
		topics: buildTelegramTopicRows(context.topics),
	}
}
