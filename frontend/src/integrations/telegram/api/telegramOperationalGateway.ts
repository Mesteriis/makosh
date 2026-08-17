import { create } from '@bufbuild/protobuf'
import type {
	TelegramChatProjection,
	TelegramMessageObservationProjection,
	TelegramMessageProjection,
	TelegramOperationResponse,
	TelegramParticipantProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { TelegramMessageProjectionSchema } from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'
import { withTelegramOperationalRuntimeV1 } from './telegramOperationalRuntimeRetry'

const MAX_CHAT_LIMIT = 5_000
const DEFAULT_CHAT_LIMIT = 50
const MESSAGE_PAGE_LIMIT = 100
const MESSAGE_CACHE_LIMIT = 500

export type TelegramOperationalMessagePage = {
	messages: readonly TelegramMessageProjection[]
	nextFromMessageId?: bigint
	hasMore: boolean
}

export async function loadTelegramChats(
	accountId: string,
	limit = DEFAULT_CHAT_LIMIT,
): Promise<readonly TelegramChatProjection[]> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const normalizedLimit = requireChatLimit(limit)
	const response = await withTelegramOperationalRuntimeV1(() =>
		getTelegramOperationalConnectClient().executeQuery({
			query: {
				case: 'loadChats',
				value: { accountId: normalizedAccountId, limit: normalizedLimit },
			},
		}),
		'interactive',
		normalizedAccountId,
	)
	if (response.response.case !== 'chats') {
		throw new Error('Telegram chat projection is unavailable')
	}
	return response.response.value.chat
}

export async function readCachedTelegramChats(
	accountId: string,
	limit = DEFAULT_CHAT_LIMIT,
): Promise<readonly TelegramChatProjection[]> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const normalizedLimit = requireChatLimit(limit)
	const response = await withTelegramOperationalRuntimeV1(() =>
		getTelegramOperationalConnectClient().executeQuery({
			query: {
				case: 'cachedChats',
				value: { accountId: normalizedAccountId, limit: normalizedLimit },
			},
		}),
		'interactive',
		normalizedAccountId,
	)
	if (response.response.case !== 'chats') {
		throw new Error('Telegram chat projection is unavailable')
	}
	return response.response.value.chat
}

export async function loadTelegramMessages(
	accountId: string,
	providerChatId: string,
): Promise<readonly TelegramMessageProjection[]> {
	return (await loadTelegramMessagePage(accountId, providerChatId)).messages
}

export async function loadTelegramMessagePage(
	accountId: string,
	providerChatId: string,
	fromMessageId?: bigint,
): Promise<TelegramOperationalMessagePage> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const normalizedProviderChatId = requireIdentifier('chat ID', providerChatId)
	const history = await withTelegramOperationalRuntimeV1(() =>
		getTelegramOperationalConnectClient().executeQuery({
			query: {
				case: 'loadHistory',
				value: {
					accountId: normalizedAccountId,
					providerChatId: normalizedProviderChatId,
					fromMessageId,
					mode: fromMessageId === undefined ? 'latest' : 'older',
					limit: MESSAGE_PAGE_LIMIT,
				},
			},
		}),
		'interactive',
		normalizedAccountId,
	)
	if (history.response.case !== 'historyPage' || !history.response.value.page) {
		throw new Error('Telegram history projection is unavailable')
	}
	const cachedMessages = await readCachedTelegramMessages(normalizedAccountId, normalizedProviderChatId)
	const freshMessagesByProviderId = new Map(
		history.response.value.page.item.map(message => [message.providerMessageId, message] as const),
	)
	const mergedMessages = new Map(
		cachedMessages.map(message => [message.providerMessageId, message] as const),
	)
	for (const observation of history.response.value.page.item) {
		const cached = mergedMessages.get(observation.providerMessageId)
		mergedMessages.set(
			observation.providerMessageId,
			cached
				? mergeFreshObservation(cached, observation)
				: projectFreshObservation(observation),
		)
	}
	return {
		messages: [...mergedMessages.values()].map((message) => {
			const fresh = freshMessagesByProviderId.get(message.providerMessageId)
			if (!fresh) return message
			return mergeFreshObservation(message, fresh)
		}),
		nextFromMessageId: history.response.value.page.nextFromMessageId,
		hasMore: history.response.value.page.hasMore,
	}
}

function projectFreshObservation(
	observation: TelegramMessageObservationProjection,
): TelegramMessageProjection {
	return create(TelegramMessageProjectionSchema, {
		messageId: `telegram:${observation.accountId}:${observation.providerChatId}:${observation.providerMessageId}`,
		accountId: observation.accountId,
		providerChatId: observation.providerChatId,
		providerMessageId: observation.providerMessageId,
		providerTopicId: observation.providerTopicId,
		senderId: observation.senderId,
		senderDisplayName: observation.senderDisplayName,
		text: observation.text,
		media: observation.media,
		references: observation.references,
		observedAtUnixSeconds: observation.observedAtUnixSeconds,
		deliveryState: 'received',
		senderSourceIdentity: observation.senderSourceIdentity,
	})
}

function mergeFreshObservation(
	cached: TelegramMessageProjection,
	observation: TelegramMessageObservationProjection,
): TelegramMessageProjection {
	return create(TelegramMessageProjectionSchema, {
		...cached,
		senderId: observation.senderId || cached.senderId,
		senderDisplayName: observation.senderDisplayName || cached.senderDisplayName,
		senderSourceIdentity: observation.senderSourceIdentity || cached.senderSourceIdentity,
		text: observation.text || cached.text,
		media: observation.media || cached.media,
		references: observation.references || cached.references,
		observedAtUnixSeconds: observation.observedAtUnixSeconds || cached.observedAtUnixSeconds,
	})
}

function hasRenderableMessageContent(message: TelegramMessageProjection): boolean {
	return Boolean(message.text?.trim() || message.media)
}

export async function readCachedTelegramMessages(
	accountId: string,
	providerChatId: string,
): Promise<readonly TelegramMessageProjection[]> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const response = await withTelegramOperationalRuntimeV1(() =>
		getTelegramOperationalConnectClient().executeQuery({
			query: {
				case: 'cachedMessages',
				value: {
					accountId: normalizedAccountId,
					providerChatId: requireIdentifier('chat ID', providerChatId),
					limit: MESSAGE_CACHE_LIMIT,
				},
			},
		}),
		'interactive',
		normalizedAccountId,
	)
	if (response.response.case !== 'cachedMessages') {
		throw new Error('Telegram message projection is unavailable')
	}
	return response.response.value.item.filter(hasRenderableMessageContent)
}

export async function loadTelegramParticipants(
	accountId: string,
	providerChatId: string,
): Promise<readonly TelegramParticipantProjection[]> {
	const response = await withTelegramOperationalRuntimeV1(() =>
		getTelegramOperationalConnectClient().executeQuery({
			query: {
				case: 'listParticipants',
				value: {
					accountId: requireIdentifier('account ID', accountId),
					providerChatId: requireIdentifier('chat ID', providerChatId),
					filter: 'recent',
					offset: 0,
					limit: MESSAGE_PAGE_LIMIT,
				},
			},
		}),
		'enrichment',
		accountId,
	)
	if (response.response.case !== 'participants') {
		throw new Error('Telegram participant directory is unavailable')
	}
	return response.response.value.item
}

export async function sendTelegramText(input: {
	accountId: string
	providerChatId: string
	text: string
	operationId: string
}): Promise<TelegramOperationResponse> {
	const text = input.text.trim()
	if (!text) {
		throw new RangeError('Telegram message text is required')
	}
	return withTelegramOperationalRuntimeV1(() => getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'sendText',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				providerChatId: requireIdentifier('chat ID', input.providerChatId),
				text,
				operationId: requireIdentifier('operation ID', input.operationId),
			},
		},
	}), 'interactive', input.accountId)
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`Telegram ${label} is required`)
	}
	return normalized
}

function requireChatLimit(limit: number): number {
	if (!Number.isSafeInteger(limit) || limit < 1 || limit > MAX_CHAT_LIMIT) {
		throw new RangeError(`Telegram chat limit must be 1-${MAX_CHAT_LIMIT}`)
	}
	return limit
}
