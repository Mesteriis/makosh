import type {
	TelegramChatProjection,
	TelegramMessageProjection,
	TelegramOperationResponse,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

const PAGE_LIMIT = 100

export async function listCachedTelegramChats(
	accountId: string,
): Promise<readonly TelegramChatProjection[]> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'cachedChats',
			value: { accountId: normalizedAccountId, limit: PAGE_LIMIT },
		},
	})
	if (response.response.case !== 'chats') {
		throw new Error('Telegram chat projection is unavailable')
	}
	return response.response.value.chat
}

export async function listCachedTelegramMessages(
	accountId: string,
	providerChatId: string,
): Promise<readonly TelegramMessageProjection[]> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'cachedMessages',
			value: {
				accountId: requireIdentifier('account ID', accountId),
				providerChatId: requireIdentifier('chat ID', providerChatId),
				limit: PAGE_LIMIT,
			},
		},
	})
	if (response.response.case !== 'cachedMessages') {
		throw new Error('Telegram message projection is unavailable')
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
	return getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'sendText',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				providerChatId: requireIdentifier('chat ID', input.providerChatId),
				text,
				operationId: requireIdentifier('operation ID', input.operationId),
			},
		},
	})
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`Telegram ${label} is required`)
	}
	return normalized
}
