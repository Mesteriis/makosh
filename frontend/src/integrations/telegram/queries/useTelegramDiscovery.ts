import { computed, ref } from 'vue'

import type {
	TelegramChatProjection,
	TelegramHistoryPageProjection,
	TelegramMessageProjection,
	TelegramOperationResponse,
} from '../../../gen/makosh/telegram/v1/client_pb'
import {
	listTelegramOperations,
	loadTelegramChatContext,
	loadTelegramHistory,
	searchTelegramChats,
	searchTelegramMessages,
} from '../api/telegramDiscoveryGateway'
import type { TelegramChatContext } from '../api/telegramDiscoveryGateway'
import {
	buildTelegramContextView,
	buildTelegramDiscoveryResults,
	buildTelegramHistoryRows,
	buildTelegramOperationRows,
} from '../presentation/telegramDiscoveryModel'
import type { TelegramDiscoveryModel } from '../presentation/telegramDiscoveryModel'

export function useTelegramDiscovery(input: {
	accountId: () => string
	canQuery: () => boolean
	selectedChatId: () => string
	senderPersonaNames?: () => ReadonlyMap<string, string>
}) {
	const query = ref('')
	const chats = ref<readonly TelegramChatProjection[]>([])
	const messages = ref<readonly TelegramMessageProjection[]>([])
	const history = ref<TelegramHistoryPageProjection | null>(null)
	const context = ref<TelegramChatContext | null>(null)
	const operations = ref<readonly TelegramOperationResponse[]>([])
	const statusMessage = ref('')
	const pending = ref(false)

	const model = computed<TelegramDiscoveryModel>(() => {
		const contextView = buildTelegramContextView(context.value)
		return {
			query: query.value,
			statusMessage: statusMessage.value,
			pending: pending.value,
			canQuery: input.canQuery(),
			results: buildTelegramDiscoveryResults({
				chats: chats.value,
				messages: messages.value,
				personaNames: input.senderPersonaNames?.(),
			}),
			history: buildTelegramHistoryRows(
				history.value?.item || [],
				input.senderPersonaNames?.(),
			),
			participants: contextView.participants,
			topics: contextView.topics,
			folders: contextView.folders,
			operations: buildTelegramOperationRows(operations.value),
			chatState: contextView.chatState,
		}
	})

	async function search(): Promise<void> {
		await runQuery(async () => {
			const accountId = requireIdentifier('account ID', input.accountId())
			const selectedChatId = input.selectedChatId()
			const [nextChats, nextMessages] = await Promise.all([
				searchTelegramChats(accountId, query.value),
				searchTelegramMessages(accountId, selectedChatId, query.value),
			])
			chats.value = nextChats
			messages.value = nextMessages
			statusMessage.value = `${nextChats.length + nextMessages.length} Telegram search results.`
		})
	}

	async function refreshChatContext(): Promise<void> {
		await runQuery(async () => {
			const accountId = requireIdentifier('account ID', input.accountId())
			const chatId = requireIdentifier('chat ID', input.selectedChatId())
			const [nextHistory, nextContext, nextOperations] = await Promise.all([
				loadTelegramHistory(accountId, chatId),
				loadTelegramChatContext(accountId, chatId),
				listTelegramOperations(accountId),
			])
			history.value = nextHistory
			context.value = nextContext
			operations.value = nextOperations
			statusMessage.value = nextHistory.hasMore
				? 'Telegram context loaded; older provider history is available.'
				: 'Telegram context loaded.'
		})
	}

	function updateQuery(value: string): void {
		query.value = value
	}

	async function runQuery(action: () => Promise<void>): Promise<void> {
		if (!input.canQuery()) {
			statusMessage.value = 'Telegram query capability is not admitted.'
			return
		}
		pending.value = true
		statusMessage.value = ''
		try {
			await action()
		} catch (error) {
			statusMessage.value = error instanceof Error ? error.message : 'Telegram query failed.'
		} finally {
			pending.value = false
		}
	}

	return {
		model,
		refreshChatContext,
		search,
		updateQuery,
	}
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
