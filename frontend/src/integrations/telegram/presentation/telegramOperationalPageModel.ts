import type {
	TelegramChatProjection,
	TelegramMessageProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'

export type TelegramOperationalStatus = 'empty' | 'error' | 'loading' | 'ready'

export type TelegramOperationalChatRow = {
	id: string
	title: string
	detail: string
	selected: boolean
}

export type TelegramOperationalMessageRow = {
	id: string
	providerMessageId: string
	sender: string
	body: string
	meta: string
	outgoing: boolean
	selected: boolean
}

export type TelegramOperationalPageModel = {
	accountId: string
	status: TelegramOperationalStatus
	statusMessage: string
	chats: readonly TelegramOperationalChatRow[]
	messages: readonly TelegramOperationalMessageRow[]
	selectedChatId: string
	selectedChatTitle: string
	selectedMessageId: string
	selectedProviderMessageId: string
	draft: string
	sendPending: boolean
	sendMessage: string
	canSend: boolean
}

export function buildTelegramChatRows(
	chats: readonly TelegramChatProjection[],
	selectedChatId: string,
): readonly TelegramOperationalChatRow[] {
	return chats.map((chat) => ({
		id: chat.providerChatId,
		title: chat.title || chat.username || `Chat ${chat.providerChatId}`,
		detail: chat.username ? `@${chat.username} · ${chat.kind}` : chat.kind || 'Telegram chat',
		selected: chat.providerChatId === selectedChatId,
	}))
}

export function buildTelegramMessageRows(
	messages: readonly TelegramMessageProjection[],
	selectedProviderMessageId = '',
): readonly TelegramOperationalMessageRow[] {
	return messages.map((message) => ({
		id: message.messageId,
		providerMessageId: message.providerMessageId || message.messageId,
		sender: message.senderDisplayName || message.senderId || 'Unknown sender',
		body: message.text || message.media?.caption || `[${message.media?.kind || 'message'}]`,
		meta: `${formatObservedAt(message.observedAtUnixSeconds)} · ${message.deliveryState || 'observed'}`,
		outgoing: message.deliveryState !== '' && message.deliveryState !== 'received',
		selected: (message.providerMessageId || message.messageId) === selectedProviderMessageId,
	}))
}

function formatObservedAt(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds) || milliseconds <= 0) {
		return 'Unknown time'
	}
	return new Intl.DateTimeFormat('en', {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(new Date(milliseconds))
}
