import type {
	TelegramChatFolderProjection,
	TelegramChatOperationalStateProjection,
	TelegramChatPositionProjection,
	TelegramChatProjection,
	TelegramChatStateProjection,
	TelegramHistoryPageProjection,
	TelegramMessageProjection,
	TelegramOperationResponse,
	TelegramParticipantProjection,
	TelegramTopicProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

const PAGE_LIMIT = 100

export type TelegramChatContext = {
	state?: TelegramChatStateProjection
	operationalState?: TelegramChatOperationalStateProjection
	positions: readonly TelegramChatPositionProjection[]
	folders: readonly TelegramChatFolderProjection[]
	participants: readonly TelegramParticipantProjection[]
	topics: readonly TelegramTopicProjection[]
}

export async function searchTelegramChats(
	accountId: string,
	query: string,
): Promise<readonly TelegramChatProjection[]> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'searchChats',
			value: {
				accountId: requireIdentifier('account ID', accountId),
				query: requireQuery(query),
				limit: PAGE_LIMIT,
			},
		},
	})
	if (response.response.case !== 'chats') {
		throw new Error('Telegram chat search result is unavailable')
	}
	return response.response.value.chat
}

export async function searchTelegramMessages(
	accountId: string,
	providerChatId: string,
	query: string,
): Promise<readonly TelegramMessageProjection[]> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'searchMessages',
			value: {
				accountId: requireIdentifier('account ID', accountId),
				providerChatId: optionalIdentifier(providerChatId),
				query: requireQuery(query),
				limit: PAGE_LIMIT,
			},
		},
	})
	if (response.response.case !== 'cachedMessages') {
		throw new Error('Telegram message search result is unavailable')
	}
	return response.response.value.item
}

export async function loadTelegramHistory(
	accountId: string,
	providerChatId: string,
): Promise<TelegramHistoryPageProjection> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'loadHistory',
			value: {
				accountId: requireIdentifier('account ID', accountId),
				providerChatId: requireIdentifier('chat ID', providerChatId),
				mode: 'older',
				limit: PAGE_LIMIT,
			},
		},
	})
	if (response.response.case !== 'historyPage') {
		throw new Error('Telegram provider history is unavailable')
	}
	if (!response.response.value.page) {
		throw new Error('Telegram provider history page is unavailable')
	}
	return response.response.value.page
}

export async function loadTelegramChatContext(
	accountId: string,
	providerChatId: string,
): Promise<TelegramChatContext> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const normalizedChatId = requireIdentifier('chat ID', providerChatId)
	const client = getTelegramOperationalConnectClient()
	const [state, operationalState, positions, participants, topics] = await Promise.all([
		client.executeQuery({
			query: {
				case: 'chatState',
				value: {
					accountId: normalizedAccountId,
					providerChatId: normalizedChatId,
				},
			},
		}),
		client.executeQuery({
			query: {
				case: 'chatOperationalState',
				value: {
					accountId: normalizedAccountId,
					providerChatId: normalizedChatId,
				},
			},
		}),
		client.executeQuery({
			query: {
				case: 'chatPositions',
				value: {
					accountId: normalizedAccountId,
					providerChatId: normalizedChatId,
				},
			},
		}),
		client.executeQuery({
			query: {
				case: 'listParticipants',
				value: {
					accountId: normalizedAccountId,
					providerChatId: normalizedChatId,
					filter: 'recent',
					offset: 0,
					limit: PAGE_LIMIT,
				},
			},
		}),
		client.executeQuery({
			query: {
				case: 'listTopics',
				value: {
					accountId: normalizedAccountId,
					providerChatId: normalizedChatId,
					limit: PAGE_LIMIT,
				},
			},
		}),
	])
	if (
		state.response.case !== 'chatState'
		|| operationalState.response.case !== 'chatOperationalState'
		|| positions.response.case !== 'chatPositions'
		|| participants.response.case !== 'participants'
		|| topics.response.case !== 'topics'
	) {
		throw new Error('Telegram chat context is incomplete')
	}
	const folderIds = positions.response.value.position.flatMap((position) =>
		position.providerFolderId === undefined ? [] : [position.providerFolderId]
	)
	const folders = folderIds.length === 0
		? []
		: await loadTelegramFolders(normalizedAccountId, folderIds)
	return {
		state: state.response.value.state,
		operationalState: operationalState.response.value.state,
		positions: positions.response.value.position,
		folders,
		participants: participants.response.value.item,
		topics: topics.response.value.topic,
	}
}

export async function listTelegramOperations(
	accountId: string,
): Promise<readonly TelegramOperationResponse[]> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'operations',
			value: {
				accountId: requireIdentifier('account ID', accountId),
				limit: PAGE_LIMIT,
			},
		},
	})
	if (response.response.case !== 'operations') {
		throw new Error('Telegram operation history is unavailable')
	}
	return response.response.value.operation
}

async function loadTelegramFolders(
	accountId: string,
	providerFolderIds: readonly bigint[],
): Promise<readonly TelegramChatFolderProjection[]> {
	const response = await getTelegramOperationalConnectClient().executeQuery({
		query: {
			case: 'chatFolders',
			value: {
				accountId,
				providerFolderId: [...new Set(providerFolderIds)],
			},
		},
	})
	if (response.response.case !== 'chatFolders') {
		throw new Error('Telegram folder projection is unavailable')
	}
	return response.response.value.folder
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}

function optionalIdentifier(value: string): string | undefined {
	const normalized = value.trim()
	return normalized || undefined
}

function requireQuery(value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError('Telegram search query is required')
	}
	return normalized
}
