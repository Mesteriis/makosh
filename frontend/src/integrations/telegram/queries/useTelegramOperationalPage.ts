import { computed, ref } from 'vue'
import type {
	TelegramChatProjection,
	TelegramMessageProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import {
	listCachedTelegramChats,
	listCachedTelegramMessages,
	sendTelegramText,
} from '../api/telegramOperationalGateway'
import {
	buildTelegramChatRows,
	buildTelegramMessageRows,
	type TelegramOperationalPageModel,
	type TelegramOperationalStatus,
} from '../presentation/telegramOperationalPageModel'

export function useTelegramOperationalPage(canSend: () => boolean) {
	const accountId = ref('')
	const status = ref<TelegramOperationalStatus>('empty')
	const statusMessage = ref('Enter an admitted Telegram account ID to load its operational projection.')
	const chats = ref<readonly TelegramChatProjection[]>([])
	const messages = ref<readonly TelegramMessageProjection[]>([])
	const selectedChatId = ref('')
	const selectedMessageId = ref('')
	const selectedProviderMessageId = ref('')
	const draft = ref('')
	const sendPending = ref(false)
	const sendMessage = ref('')

	const model = computed<TelegramOperationalPageModel>(() => ({
		accountId: accountId.value,
		status: status.value,
		statusMessage: statusMessage.value,
		chats: buildTelegramChatRows(chats.value, selectedChatId.value),
		messages: buildTelegramMessageRows(messages.value, selectedProviderMessageId.value),
		selectedChatId: selectedChatId.value,
		selectedChatTitle: chats.value.find((chat) => chat.providerChatId === selectedChatId.value)?.title || '',
		selectedMessageId: selectedMessageId.value,
		selectedProviderMessageId: selectedProviderMessageId.value,
		draft: draft.value,
		sendPending: sendPending.value,
		sendMessage: sendMessage.value,
		canSend: canSend(),
	}))

	async function loadChats(): Promise<void> {
		status.value = 'loading'
		statusMessage.value = 'Loading Telegram projection…'
		sendMessage.value = ''
		try {
			chats.value = await listCachedTelegramChats(accountId.value)
			status.value = chats.value.length === 0 ? 'empty' : 'ready'
			statusMessage.value = chats.value.length === 0
				? 'No cached Telegram chats are available for this account.'
				: ''
			if (chats.value[0]) {
				await selectChat(chats.value[0].providerChatId)
			} else {
				selectedChatId.value = ''
				selectedMessageId.value = ''
				selectedProviderMessageId.value = ''
				messages.value = []
			}
		} catch (error) {
			fail(error, 'Telegram projection is unavailable.')
		}
	}

	async function selectChat(providerChatId: string): Promise<void> {
		selectedChatId.value = providerChatId
		selectedMessageId.value = ''
		selectedProviderMessageId.value = ''
		status.value = 'loading'
		statusMessage.value = 'Loading Telegram messages…'
		try {
			messages.value = await listCachedTelegramMessages(accountId.value, providerChatId)
			status.value = 'ready'
			statusMessage.value = messages.value.length === 0 ? 'No cached messages are available.' : ''
		} catch (error) {
			fail(error, 'Telegram messages are unavailable.')
		}
	}

	function selectMessage(messageId: string, providerMessageId: string): void {
		selectedMessageId.value = messageId
		selectedProviderMessageId.value = providerMessageId
	}

	async function send(): Promise<void> {
		if (!canSend()) {
			sendMessage.value = 'Telegram command capability is not admitted.'
			return
		}
		sendPending.value = true
		sendMessage.value = ''
		try {
			const response = await sendTelegramText({
				accountId: accountId.value,
				providerChatId: selectedChatId.value,
				text: draft.value,
				operationId: crypto.randomUUID(),
			})
			draft.value = ''
			sendMessage.value = `Operation ${response.operationId} is ${response.state || 'accepted'}.`
			await selectChat(selectedChatId.value)
		} catch (error) {
			sendMessage.value = error instanceof Error ? error.message : 'Telegram send failed.'
		} finally {
			sendPending.value = false
		}
	}

	function updateAccountId(value: string): void {
		accountId.value = value
	}

	function updateDraft(value: string): void {
		draft.value = value
	}

	function fail(error: unknown, fallback: string): void {
		status.value = 'error'
		statusMessage.value = error instanceof Error ? error.message : fallback
	}

	return {
		model,
		loadChats,
		selectChat,
		selectMessage,
		send,
		updateAccountId,
		updateDraft,
	}
}
