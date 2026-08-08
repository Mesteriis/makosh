import type {
	TelegramChatProjection,
	TelegramMessageObservationProjection,
	TelegramMessageProjection,
	TelegramOperationResponse,
	TelegramParticipantProjection,
	TelegramTopicProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import type { TelegramChatContext } from '../api/telegramDiscoveryGateway'

export type TelegramDiscoveryResultRow = {
	id: string
	title: string
	detail: string
	kind: 'chat' | 'message'
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
}): readonly TelegramDiscoveryResultRow[] {
	return [
		...input.chats.map((chat) => ({
			id: chat.providerChatId,
			title: chat.title || chat.username || chat.providerChatId,
			detail: chat.username ? `@${chat.username} · ${chat.kind}` : chat.kind,
			kind: 'chat' as const,
		})),
		...input.messages.map((message) => ({
			id: message.messageId,
			title: message.senderDisplayName || message.senderId || 'Unknown sender',
			detail: message.text || message.media?.caption || `[${message.media?.kind || 'message'}]`,
			kind: 'message' as const,
		})),
	]
}

export function buildTelegramHistoryRows(
	items: readonly TelegramMessageObservationProjection[],
): readonly TelegramDiscoveryDetailRow[] {
	return items.map((item) => ({
		id: item.providerMessageId,
		title: item.senderDisplayName || item.senderId || 'Unknown sender',
		detail: item.text || item.media?.caption || `[${item.media?.kind || 'message'}]`,
	}))
}

export function buildTelegramParticipantRows(
	items: readonly TelegramParticipantProjection[],
): readonly TelegramDiscoveryDetailRow[] {
	return items.map((item) => ({
		id: item.providerMemberId,
		title: item.displayName || item.username || item.providerMemberId,
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
