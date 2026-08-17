import { computed, reactive, ref } from 'vue'
import type {
	TelegramChatProjection,
	TelegramMessageProjection,
	TelegramParticipantProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import {
	loadTelegramChats,
	loadTelegramMessagePage,
	loadTelegramParticipants,
	readCachedTelegramChats,
	readCachedTelegramMessages,
	sendTelegramText,
} from '../api/telegramOperationalGateway'
import {
	openTelegramOperationalRealtime,
	type TelegramOperationalRealtimeBinding,
} from '../api/telegramOperationalRealtime'
import { replyToTelegramMessage } from '../api/telegramMessageCommandGateway'
import {
	activateTelegramMediaScope,
	deactivateTelegramMediaScopes,
} from '../api/telegramProviderMediaGateway'
import {
	buildTelegramChatRows,
	buildTelegramMessageRows,
	resolveTelegramSenderName,
	type TelegramOperationalPageModel,
	type TelegramOperationalStatus,
} from '../presentation/telegramOperationalPageModel'
import {
	getClientAccountLaneRegistry,
	type ClientAccountLane,
} from '../../../platform/gateway/clientAccountLane'

const CHAT_PAGE_SIZE = 50
const MAX_CHAT_LIMIT = 5_000

type TelegramChatHistoryState = {
	messages: readonly TelegramMessageProjection[]
	nextFromMessageId?: bigint
	hasOlderMessages: boolean
}

export function useTelegramOperationalPage(
	canSend: () => boolean,
	canReplay: () => boolean = () => false,
	senderPersonaNames: () => ReadonlyMap<string, string> = () => new Map(),
) {
	const accountId = ref('')
	const status = ref<TelegramOperationalStatus>('empty')
	const statusMessage = ref('Enter an admitted Telegram account ID to load its operational projection.')
	const chats = ref<readonly TelegramChatProjection[]>([])
	const messages = ref<readonly TelegramMessageProjection[]>([])
	const selectedChatId = ref('')
	const selectedMessageId = ref('')
	const selectedProviderMessageId = ref('')
	const replyToProviderMessageId = ref('')
	const draft = ref('')
	const sendPending = ref(false)
	const sendMessage = ref('')
	const historyPending = ref(false)
	const hasOlderMessages = ref(false)
	const chatPending = ref(false)
	const hasMoreChats = ref(false)
	let chatLimit = CHAT_PAGE_SIZE
	let chatSelectionGeneration = 0
	let nextFromMessageId: bigint | undefined
	const historyByChat = new Map<string, TelegramChatHistoryState>()
	const senderNamesByChat = reactive(new Map<string, ReadonlyMap<string, string>>())
	const realtimeStatus = ref<TelegramOperationalPageModel['realtimeStatus']>('disabled')
	let realtimeGeneration = 0
	let realtimeBinding: TelegramOperationalRealtimeBinding | undefined
	let realtimeLane: ClientAccountLane | undefined
	let realtimeAccountId = ''

	const model = computed<TelegramOperationalPageModel>(() => {
		const replyMessage = messages.value.find(message =>
			(message.providerMessageId || message.messageId) === replyToProviderMessageId.value)
		const selectedChat = chats.value.find(chat => chat.providerChatId === selectedChatId.value)
		const providerSenderNames = senderNamesByChat.get(historyKey(selectedChatId.value)) ?? new Map()
		const privateChatTitle = selectedChat?.kind === 'private' ? selectedChat.title : ''
		return {
		accountId: accountId.value,
		status: status.value,
		statusMessage: statusMessage.value,
		realtimeStatus: realtimeStatus.value,
		chats: buildTelegramChatRows(chats.value, selectedChatId.value),
			messages: buildTelegramMessageRows(
				messages.value,
				selectedProviderMessageId.value,
				senderPersonaNames(),
				providerSenderNames,
				privateChatTitle,
			),
		selectedChatId: selectedChatId.value,
		selectedChatTitle: chats.value.find((chat) => chat.providerChatId === selectedChatId.value)?.title || '',
		selectedChatAvatarProviderFileId: chats.value.find((chat) => chat.providerChatId === selectedChatId.value)?.avatarProviderFileId || '',
		selectedMessageId: selectedMessageId.value,
		selectedProviderMessageId: selectedProviderMessageId.value,
		replyToProviderMessageId: replyToProviderMessageId.value,
			replyToSender: replyMessage
				? resolveTelegramSenderName(
					replyMessage,
					senderPersonaNames(),
					providerSenderNames,
					privateChatTitle,
				)
			: '',
		replyToBody: replyMessage?.text || replyMessage?.media?.caption || '',
		draft: draft.value,
		sendPending: sendPending.value,
		sendMessage: sendMessage.value,
		canSend: canSend(),
		historyPending: historyPending.value,
			hasOlderMessages: hasOlderMessages.value,
			chatPending: chatPending.value,
			hasMoreChats: hasMoreChats.value,
		}
	})

	async function loadChats(): Promise<void> {
		chatPending.value = true
		chatLimit = CHAT_PAGE_SIZE
		status.value = 'loading'
		statusMessage.value = 'Loading Telegram projection…'
		sendMessage.value = ''
		try {
			const preferredChatId = selectedChatId.value
			chats.value = await loadTelegramChats(accountId.value, chatLimit)
			hasMoreChats.value = chats.value.length > 0 && chatLimit < MAX_CHAT_LIMIT
			status.value = chats.value.length === 0 ? 'empty' : 'ready'
			statusMessage.value = chats.value.length === 0
				? 'No cached Telegram chats are available for this account.'
				: ''
			const nextChat = chats.value.find(chat => chat.providerChatId === preferredChatId)
				?? chats.value[0]
			if (nextChat) {
				await selectChat(nextChat.providerChatId)
			} else {
				selectedChatId.value = ''
				selectedMessageId.value = ''
				selectedProviderMessageId.value = ''
				messages.value = []
			}
		} catch (error) {
			fail(error, 'Telegram projection is unavailable.')
		} finally {
			chatPending.value = false
		}
	}

	async function loadCachedProjection(): Promise<void> {
		if (!accountId.value) return
		chatPending.value = true
		status.value = 'loading'
		statusMessage.value = 'Loading cached Telegram projection…'
		try {
			const nextChats = await readCachedTelegramChats(accountId.value, chatLimit)
			chats.value = nextChats
			hasMoreChats.value = false
			const nextChat = nextChats.find(chat => chat.providerChatId === selectedChatId.value)
				?? nextChats[0]
			if (!nextChat) {
				selectedChatId.value = ''
				messages.value = []
				status.value = 'empty'
				statusMessage.value = 'No cached Telegram chats are available for this account.'
				return
			}
			await selectCachedChat(nextChat.providerChatId)
			status.value = 'ready'
			statusMessage.value = 'Showing cached Telegram chats while provider authorization is unavailable.'
		} catch (error) {
			fail(error, 'Cached Telegram projection is unavailable.')
		} finally {
			chatPending.value = false
		}
	}

	async function loadMoreChats(): Promise<void> {
		if (chatPending.value || !hasMoreChats.value || !accountId.value) return
		chatPending.value = true
		const nextLimit = Math.min(chatLimit + CHAT_PAGE_SIZE, MAX_CHAT_LIMIT)
		const previousLength = chats.value.length
		try {
			const nextChats = await loadTelegramChats(accountId.value, nextLimit)
			chats.value = nextChats
			chatLimit = nextLimit
			hasMoreChats.value = nextChats.length > previousLength && nextLimit < MAX_CHAT_LIMIT
		} catch (error) {
			statusMessage.value = error instanceof Error ? error.message : 'More Telegram chats are unavailable.'
		} finally {
			chatPending.value = false
		}
	}

	async function startRealtime(): Promise<void> {
		stopRealtime(false)
		if (!canReplay() || !accountId.value) return
		const generation = realtimeGeneration
		const selectedAccountId = accountId.value
		realtimeStatus.value = 'connecting'
		realtimeAccountId = selectedAccountId
		realtimeLane = getClientAccountLaneRegistry().get({
			provider: 'telegram',
			accountId: selectedAccountId,
		})
		realtimeBinding = openTelegramOperationalRealtime(selectedAccountId, {
			onProjectionChanged: latestSequence => {
				realtimeLane?.invalidate(latestSequence, async (_revision, signal) => {
					if (signal.aborted || generation !== realtimeGeneration) return
					await refreshCachedProjection(selectedAccountId, generation)
				})
		},
			onLive: () => {
				if (generation === realtimeGeneration) realtimeStatus.value = 'live'
		},
			onUnavailable: () => {
				if (generation !== realtimeGeneration) return
				realtimeStatus.value = 'recovering'
				realtimeLane?.recover(async (signal) => {
					if (signal.aborted || generation !== realtimeGeneration) return
					await refreshCachedProjection(selectedAccountId, generation)
				})
			},
		})
	}

	function stopRealtime(deactivateMedia = true): void {
		realtimeGeneration += 1
		realtimeBinding?.close()
		realtimeBinding = undefined
		if (realtimeAccountId) {
			getClientAccountLaneRegistry().close({ provider: 'telegram', accountId: realtimeAccountId })
		}
		realtimeLane = undefined
		realtimeAccountId = ''
		realtimeStatus.value = 'disabled'
		if (deactivateMedia) deactivateTelegramMediaScopes()
	}

	async function refreshCachedProjection(
		expectedAccountId = accountId.value,
		expectedRealtimeGeneration = realtimeGeneration,
	): Promise<void> {
		if (historyPending.value || sendPending.value) return
		const selectionGeneration = chatSelectionGeneration
		const nextChats = await readCachedTelegramChats(expectedAccountId, chatLimit)
		if (selectionGeneration !== chatSelectionGeneration
			|| expectedRealtimeGeneration !== realtimeGeneration
			|| expectedAccountId !== accountId.value) return
		if (nextChats.length === 0 && chats.value.length > 0) return
		chats.value = nextChats
		const nextChat = nextChats.find(chat => chat.providerChatId === selectedChatId.value)
			?? nextChats[0]
		if (!nextChat) {
			selectedChatId.value = ''
			messages.value = []
			status.value = 'empty'
			statusMessage.value = 'No Telegram chats are available for this account.'
			return
		}
		selectedChatId.value = nextChat.providerChatId
		const nextMessages = await readCachedTelegramMessages(expectedAccountId, nextChat.providerChatId)
		if (selectionGeneration !== chatSelectionGeneration
			|| expectedRealtimeGeneration !== realtimeGeneration
			|| expectedAccountId !== accountId.value) return
		const cachedHistory = historyByChat.get(historyKey(nextChat.providerChatId))
		const refreshedMessages = nextMessages.length === 0 && cachedHistory?.messages.length
			? cachedHistory.messages
			: nextMessages
		applyHistory(nextChat.providerChatId, {
			messages: refreshedMessages,
			nextFromMessageId: cachedHistory?.nextFromMessageId,
			hasOlderMessages: cachedHistory?.hasOlderMessages ?? false,
		})
		status.value = 'ready'
		statusMessage.value = ''
	}

	async function selectChat(providerChatId: string): Promise<void> {
		const selectionGeneration = ++chatSelectionGeneration
		const cachedHistory = historyByChat.get(historyKey(providerChatId))
		selectedChatId.value = providerChatId
		activateTelegramMediaScope(
			historyKey(providerChatId),
			`${accountId.value}:chat-list`,
		)
		selectedMessageId.value = ''
		selectedProviderMessageId.value = ''
		replyToProviderMessageId.value = ''
		status.value = 'ready'
		statusMessage.value = ''
		historyPending.value = true
		if (cachedHistory) {
			applyHistory(providerChatId, cachedHistory)
		} else {
			nextFromMessageId = undefined
			hasOlderMessages.value = false
			messages.value = []
			try {
				const cachedMessages = await readCachedTelegramMessages(accountId.value, providerChatId)
				if (selectionGeneration !== chatSelectionGeneration || selectedChatId.value !== providerChatId) return
				if (cachedMessages.length > 0) {
					applyHistory(providerChatId, {
						messages: cachedMessages,
						hasOlderMessages: false,
					})
				}
			} catch {
				// Provider refresh below remains authoritative when the offline cache is unavailable.
			}
		}
		try {
			const page = await loadTelegramMessagePage(accountId.value, providerChatId)
			if (selectionGeneration !== chatSelectionGeneration || selectedChatId.value !== providerChatId) return
			const nextHistory = page.messages.length === 0 && cachedHistory?.messages.length
				? cachedHistory
				: {
					messages: page.messages,
					nextFromMessageId: page.nextFromMessageId,
					hasOlderMessages: page.hasMore && page.nextFromMessageId !== undefined,
				}
			applyHistory(providerChatId, nextHistory)
			status.value = 'ready'
			statusMessage.value = messages.value.length === 0 ? 'No cached messages are available.' : ''
			historyPending.value = false
			loadSenderDirectoryInBackground(
				accountId.value,
				providerChatId,
				nextHistory.messages,
			)
		} catch (error) {
			if (selectionGeneration === chatSelectionGeneration) {
				if (cachedHistory) {
					status.value = 'ready'
					statusMessage.value = 'Telegram history refresh is unavailable; showing cached messages.'
				} else {
					fail(error, 'Telegram messages are unavailable.')
				}
			}
		} finally {
			if (selectionGeneration === chatSelectionGeneration) historyPending.value = false
		}
	}

	async function selectCachedChat(providerChatId: string): Promise<void> {
		const selectionGeneration = ++chatSelectionGeneration
		selectedChatId.value = providerChatId
		activateTelegramMediaScope(historyKey(providerChatId), `${accountId.value}:chat-list`)
		selectedMessageId.value = ''
		selectedProviderMessageId.value = ''
		replyToProviderMessageId.value = ''
		const cachedHistory = historyByChat.get(historyKey(providerChatId))
		const cachedMessages = cachedHistory?.messages
			?? await readCachedTelegramMessages(accountId.value, providerChatId)
		if (selectionGeneration !== chatSelectionGeneration || selectedChatId.value !== providerChatId) return
		applyHistory(providerChatId, {
			messages: cachedMessages,
			nextFromMessageId: cachedHistory?.nextFromMessageId,
			hasOlderMessages: cachedHistory?.hasOlderMessages ?? false,
		})
	}

	async function loadOlderMessages(): Promise<void> {
		if (historyPending.value || nextFromMessageId === undefined || !selectedChatId.value) return
		const providerChatId = selectedChatId.value
		const selectionGeneration = chatSelectionGeneration
		historyPending.value = true
		statusMessage.value = ''
		try {
			const page = await loadTelegramMessagePage(
				accountId.value,
				providerChatId,
				nextFromMessageId,
			)
			if (selectionGeneration !== chatSelectionGeneration || selectedChatId.value !== providerChatId) return
			const currentHistory = historyByChat.get(historyKey(providerChatId))
			const nextHistory = page.messages.length === 0 && currentHistory?.messages.length
				? currentHistory
				: {
					messages: mergeTelegramMessages(currentHistory?.messages ?? [], page.messages),
					nextFromMessageId: page.nextFromMessageId,
					hasOlderMessages: page.hasMore && page.nextFromMessageId !== undefined,
				}
			applyHistory(providerChatId, nextHistory)
		} catch (error) {
			if (selectionGeneration === chatSelectionGeneration) {
				statusMessage.value = error instanceof Error ? error.message : 'Older Telegram history is unavailable.'
			}
		} finally {
			if (selectionGeneration === chatSelectionGeneration) historyPending.value = false
		}
	}

	function selectMessage(messageId: string, providerMessageId: string): void {
		selectedMessageId.value = messageId
		selectedProviderMessageId.value = providerMessageId
	}

	function beginReply(): void {
		if (!selectedProviderMessageId.value) return
		replyToProviderMessageId.value = selectedProviderMessageId.value
	}

	function cancelReply(): void {
		replyToProviderMessageId.value = ''
	}

	async function send(): Promise<void> {
		if (!canSend()) {
			sendMessage.value = 'Telegram command capability is not admitted.'
			return
		}
		sendPending.value = true
		sendMessage.value = ''
		try {
			const operationId = crypto.randomUUID()
			const response = replyToProviderMessageId.value
				? await replyToTelegramMessage({
					accountId: accountId.value,
					providerChatId: selectedChatId.value,
					providerMessageId: replyToProviderMessageId.value,
					operationId,
				}, draft.value)
				: await sendTelegramText({
					accountId: accountId.value,
					providerChatId: selectedChatId.value,
					text: draft.value,
					operationId,
				})
			draft.value = ''
			replyToProviderMessageId.value = ''
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

	function suspendForAuthorization(authorizationState: string): void {
		status.value = 'empty'
		statusMessage.value = authorizationState === 'waiting_qr_scan'
			? 'Scan the Telegram QR code to load chats.'
			: authorizationState === 'waiting_password'
				? 'Enter the Telegram cloud password to load chats.'
				: 'Telegram authorization must be ready before chats can load.'
	}

	function historyKey(providerChatId: string): string {
		return `${accountId.value}:${providerChatId}`
	}

	function applyHistory(providerChatId: string, history: TelegramChatHistoryState): void {
		historyByChat.set(historyKey(providerChatId), history)
		if (selectedChatId.value !== providerChatId) return
		messages.value = history.messages
		nextFromMessageId = history.nextFromMessageId
		hasOlderMessages.value = history.hasOlderMessages
	}

	function loadSenderDirectoryInBackground(
		selectedAccountId: string,
		providerChatId: string,
		loadedMessages: readonly TelegramMessageProjection[],
	): void {
		if (!loadedMessages.some(message => senderNameNeedsResolution(message.senderDisplayName))) return
		const senderDirectoryKey = `${selectedAccountId}:${providerChatId}`
		void loadTelegramParticipants(selectedAccountId, providerChatId)
			.then((participants) => {
				if (accountId.value !== selectedAccountId) return
				senderNamesByChat.set(
					senderDirectoryKey,
					buildProviderSenderNameDirectory(participants),
				)
			})
			.catch(() => {
				// History remains usable when Telegram hides or can't return the
				// participant directory for this chat.
			})
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
		loadCachedProjection,
		loadMoreChats,
		loadOlderMessages,
		beginReply,
		cancelReply,
		startRealtime,
		stopRealtime,
		selectChat,
		selectMessage,
		send,
		suspendForAuthorization,
		updateAccountId,
		updateDraft,
	}
}

function mergeTelegramMessages(
	current: readonly TelegramMessageProjection[],
	incoming: readonly TelegramMessageProjection[],
): readonly TelegramMessageProjection[] {
	const merged = new Map(current.map(message => [message.messageId, message] as const))
	for (const message of incoming) merged.set(message.messageId, message)
	return [...merged.values()]
}

function buildProviderSenderNameDirectory(
	participants: readonly TelegramParticipantProjection[],
): ReadonlyMap<string, string> {
	const directory = new Map<string, string>()
	for (const participant of participants) {
		const displayName = participant.displayName?.trim()
			|| (participant.username?.trim() ? `@${participant.username.trim().replace(/^@/, '')}` : '')
		if (!displayName) continue
		const providerMemberId = participant.providerMemberId.trim()
		if (!providerMemberId) continue
		directory.set(providerMemberId, displayName)
		const separator = providerMemberId.indexOf(':')
		if (separator >= 0 && separator < providerMemberId.length - 1) {
			directory.set(providerMemberId.slice(separator + 1), displayName)
		}
	}
	return directory
}

function senderNameNeedsResolution(value?: string): boolean {
	const normalized = value?.trim() || ''
	return normalized === ''
		|| normalized === 'Telegram user'
		|| normalized === 'Telegram chat'
		|| normalized === 'Telegram participant'
}
