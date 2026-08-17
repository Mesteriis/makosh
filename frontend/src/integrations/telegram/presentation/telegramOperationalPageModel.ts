import type {
	TelegramChatProjection,
	TelegramMessageProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { providerSourceIdentityKey } from '../../../shared/identity/providerSourceIdentity'

export type TelegramOperationalStatus = 'empty' | 'error' | 'loading' | 'ready'
export type TelegramOperationalRealtimeStatus = 'disabled' | 'connecting' | 'live' | 'recovering'

export type TelegramOperationalChatRow = {
	id: string
	title: string
	detail: string
	selected: boolean
	avatarProviderFileId: string
}

export type TelegramOperationalMessageRow = {
	id: string
	providerMessageId: string
	sender: string
	body: string
	meta: string
	outgoing: boolean
	selected: boolean
	media?: {
		kind: string
		filename: string
		providerFileId: string
		previewProviderFileId: string
		previewInlineData: Uint8Array
		contentType: string
		previewContentType: string
		renderKind: 'animation' | 'audio' | 'file' | 'image' | 'tgs' | 'video'
	}
}

export type TelegramOperationalPageModel = {
	accountId: string
	status: TelegramOperationalStatus
	statusMessage: string
	realtimeStatus: TelegramOperationalRealtimeStatus
	chats: readonly TelegramOperationalChatRow[]
	messages: readonly TelegramOperationalMessageRow[]
	selectedChatId: string
	selectedChatTitle: string
	selectedChatAvatarProviderFileId: string
	selectedMessageId: string
	selectedProviderMessageId: string
	replyToProviderMessageId: string
	replyToSender: string
	replyToBody: string
	draft: string
	sendPending: boolean
	sendMessage: string
	canSend: boolean
	historyPending: boolean
	hasOlderMessages: boolean
	chatPending: boolean
	hasMoreChats: boolean
}

export function buildTelegramChatRows(
	chats: readonly TelegramChatProjection[],
	selectedChatId: string,
): readonly TelegramOperationalChatRow[] {
	return chats.map((chat) => ({
		id: chat.providerChatId,
		title: chat.title || chat.username || 'Untitled Telegram chat',
		detail: chat.username ? `@${chat.username} · ${chat.kind}` : chat.kind || 'Telegram chat',
		selected: chat.providerChatId === selectedChatId,
		avatarProviderFileId: chat.avatarProviderFileId || '',
	}))
}

export function buildTelegramMessageRows(
	messages: readonly TelegramMessageProjection[],
	selectedProviderMessageId = '',
	personaNames: ReadonlyMap<string, string> = new Map(),
	providerSenderNames: ReadonlyMap<string, string> = new Map(),
	privateChatTitle = '',
): readonly TelegramOperationalMessageRow[] {
	return [...messages].sort((left, right) => {
		if (left.observedAtUnixSeconds !== right.observedAtUnixSeconds) {
			return left.observedAtUnixSeconds < right.observedAtUnixSeconds ? -1 : 1
		}
		return left.providerMessageId.localeCompare(right.providerMessageId)
	}).map((message) => {
		const outgoing = message.deliveryState !== '' && message.deliveryState !== 'received'
		const mediaCaption = message.media ? normalizeMediaCaption(message.media.caption) : ''
		const mediaFilename = message.media ? normalizeMediaCaption(message.media.filename) || mediaCaption : ''
		return {
			id: message.messageId,
			providerMessageId: message.providerMessageId || message.messageId,
			sender: resolveTelegramSenderName(
				message,
				personaNames,
				providerSenderNames,
				privateChatTitle,
			),
			body: message.media
				? mediaCaption
				: message.text?.trim() || '',
			meta: `${formatObservedAt(message.observedAtUnixSeconds)} · ${message.deliveryState || 'observed'}`,
			outgoing,
			selected: (message.providerMessageId || message.messageId) === selectedProviderMessageId,
			media: message.media ? {
				kind: message.media.kind || 'attachment',
				filename: mediaFilename || message.media.kind || 'Attachment',
				providerFileId: message.media.providerFileId || '',
				previewProviderFileId: message.media.previewProviderFileId || '',
				previewInlineData: message.media.previewInlineData || new Uint8Array(),
				contentType: message.media.contentType || '',
				previewContentType: message.media.previewContentType || 'image/jpeg',
				renderKind: mediaRenderKind(message.media.kind, message.media.contentType),
			} : undefined,
		}
	})
}

function normalizeMediaCaption(value: string | undefined): string {
	const normalized = value?.trim() || ''
	if (!normalized) return ''
	if (normalized.startsWith('[') && normalized.endsWith(']')) return ''
	return normalized
}

export function resolveTelegramSenderName(
	message: Pick<TelegramMessageProjection, 'senderDisplayName' | 'senderId' | 'senderSourceIdentity'>
		& Partial<Pick<TelegramMessageProjection, 'deliveryState'>>,
	personaNames: ReadonlyMap<string, string>,
	providerSenderNames: ReadonlyMap<string, string> = new Map(),
	privateChatTitle = '',
): string {
	const sourceKey = providerSourceIdentityKey(message.senderSourceIdentity)
	const providerName = providerSenderNames.get(message.senderId?.trim() || '')?.trim()
	const tdlibName = message.senderDisplayName?.trim()
	return (sourceKey ? personaNames.get(sourceKey)?.trim() : undefined)
		|| providerName
		|| (tdlibName && !isGenericTelegramSenderName(tdlibName) ? tdlibName : undefined)
		|| (message.deliveryState && message.deliveryState !== 'received' ? 'You' : undefined)
		|| privateChatTitle.trim()
		|| 'Telegram user'
}

function isGenericTelegramSenderName(value: string): boolean {
	return value === 'Telegram user'
		|| value === 'Telegram chat'
		|| value === 'Telegram participant'
}

function mediaRenderKind(
	kind: string,
	contentType?: string,
): 'animation' | 'audio' | 'file' | 'image' | 'tgs' | 'video' {
	const normalizedKind = kind.trim().toLowerCase()
	const normalizedType = contentType?.trim().toLowerCase() || ''
	if (normalizedType === 'application/x-tgsticker') return 'tgs'
	if (normalizedKind === 'photo' || normalizedType.startsWith('image/')) return 'image'
	if (normalizedKind === 'animation') return 'animation'
	if (normalizedKind === 'video' || normalizedType.startsWith('video/')) return 'video'
	if (
		normalizedKind === 'audio'
		|| normalizedKind === 'voicenote'
		|| normalizedKind === 'voice_note'
		|| normalizedType.startsWith('audio/')
	) return 'audio'
	return 'file'
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
