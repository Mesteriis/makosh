import { computed, ref, shallowRef } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import type { WhatsAppOperationalRuntimeStatusV1 } from '../../../gen/makosh/whatsapp/operational/v1/client_pb'
import type {
	WhatsAppDialog,
	WhatsAppMessage,
	WhatsAppParticipant,
	WhatsAppProviderEventV1,
} from '../../../gen/makosh/whatsapp/v1/client_pb'
import {
	getWhatsAppOperationalRuntimeStatus,
	listWhatsAppOperationalDialogs,
	listWhatsAppOperationalEvents,
	listWhatsAppOperationalMessages,
	listWhatsAppOperationalParticipants,
	searchWhatsAppOperationalMessages,
} from '../api/whatsAppOperationalReadGateway'
import {
	buildWhatsAppOperationalReadModel,
	type WhatsAppOperationalReadModel,
	type WhatsAppOperationalReadState,
} from '../presentation/whatsAppOperationalReadModel'
import { whatsAppOperationalQueryAccounts } from './whatsAppOperationalAccounts'

export function useWhatsAppOperationalRead(input: {
	canQuery: () => boolean
	modules: () => readonly ClientModuleBootstrapV1[]
}) {
	const state = ref<WhatsAppOperationalReadState>('blocked')
	const statusMessage = ref('')
	const selectedAccountId = ref('')
	const selectedChatId = ref('')
	const searchQuery = ref('')
	const runtime = shallowRef<WhatsAppOperationalRuntimeStatusV1>()
	const dialogs = shallowRef<readonly WhatsAppDialog[]>([])
	const messages = shallowRef<readonly WhatsAppMessage[]>([])
	const participants = shallowRef<readonly WhatsAppParticipant[]>([])
	const events = shallowRef<readonly WhatsAppProviderEventV1[]>([])
	const searchResults = shallowRef<readonly WhatsAppMessage[]>([])
	const dialogCursor = ref('')
	const messageCursor = ref('')
	const participantCursor = ref('')
	const eventCursor = ref('')
	const searchCursor = ref('')
	let generation = 0

	const accounts = computed(() => whatsAppOperationalQueryAccounts(input.modules()))
	const model = computed<WhatsAppOperationalReadModel>(() => (
		buildWhatsAppOperationalReadModel({
			canQuery: input.canQuery(),
			state: state.value,
			statusMessage: statusMessage.value,
			accounts: accounts.value,
			selectedAccountId: selectedAccountId.value,
			selectedChatId: selectedChatId.value,
			searchQuery: searchQuery.value,
			runtime: runtime.value,
			dialogs: dialogs.value,
			messages: messages.value,
			participants: participants.value,
			events: events.value,
			searchResults: searchResults.value,
			hasMoreDialogs: Boolean(dialogCursor.value),
			hasMoreMessages: Boolean(messageCursor.value),
			hasMoreParticipants: Boolean(participantCursor.value),
			hasMoreEvents: Boolean(eventCursor.value),
			hasMoreSearchResults: Boolean(searchCursor.value),
		})
	))

	async function reconcile(): Promise<void> {
		const available = accounts.value
		if (!input.canQuery()) {
			clear('WhatsApp operational query capability is not admitted.')
			return
		}
		if (available.length === 0) {
			clear('No admitted WhatsApp account is available in effective integration settings.')
			state.value = 'empty'
			return
		}
		if (!available.some((account) => account.accountId === selectedAccountId.value)) {
			selectedAccountId.value = available[0]!.accountId
		}
		await refresh()
	}

	async function refresh(): Promise<void> {
		if (!readyForQuery()) return
		const token = ++generation
		begin('Loading WhatsApp operational projection…')
		resetProjection()
		try {
			const [runtimeStatus, dialogPage, eventPage] = await Promise.all([
				getWhatsAppOperationalRuntimeStatus(selectedAccountId.value),
				listWhatsAppOperationalDialogs({ accountId: selectedAccountId.value }),
				listWhatsAppOperationalEvents({ accountId: selectedAccountId.value }),
			])
			if (!current(token)) return
			runtime.value = runtimeStatus
			dialogs.value = dialogPage.item
			dialogCursor.value = dialogPage.nextCursor ?? ''
			events.value = eventPage.item
			eventCursor.value = eventPage.nextCursor ?? ''
			const initialDialog = dialogPage.item[0]
			await loadConversation(initialDialog?.providerChatId ?? '', token)
			if (!current(token)) return
			finishProjection()
		} catch (error) {
			fail(error, token, 'WhatsApp operational projection is unavailable.')
		}
	}

	async function selectAccount(accountId: string): Promise<void> {
		if (!accounts.value.some((account) => account.accountId === accountId)) return
		selectedAccountId.value = accountId
		await refresh()
	}

	async function selectDialog(providerChatId: string): Promise<void> {
		if (!dialogs.value.some((dialog) => dialog.providerChatId === providerChatId)) return
		const token = ++generation
		begin('Loading WhatsApp conversation…')
		try {
			await loadConversation(providerChatId, token)
			if (current(token)) completeReady()
		} catch (error) {
			fail(error, token, 'WhatsApp conversation is unavailable.')
		}
	}

	async function search(): Promise<void> {
		const query = searchQuery.value.trim()
		if (!readyForQuery()) return
		if (!query) {
			searchResults.value = []
			searchCursor.value = ''
			return
		}
		const token = ++generation
		begin('Searching WhatsApp messages…')
		try {
			const page = await searchWhatsAppOperationalMessages({
				accountId: selectedAccountId.value,
				providerChatId: selectedChatId.value || undefined,
				searchQuery: query,
			})
			if (!current(token)) return
			searchResults.value = page.item
			searchCursor.value = page.nextCursor ?? ''
			completeReady()
		} catch (error) {
			fail(error, token, 'WhatsApp search is unavailable.')
		}
	}

	async function loadMoreDialogs(): Promise<void> {
		const cursor = dialogCursor.value
		if (!cursor || !readyForQuery()) return
		await appendPage('Loading more WhatsApp dialogs…', async (token) => {
			const page = await listWhatsAppOperationalDialogs({
				accountId: selectedAccountId.value,
				cursor,
			})
			if (!current(token)) return
			dialogs.value = appendUnique(
				dialogs.value,
				page.item,
				(dialog) => dialog.providerChatId,
			)
			dialogCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreMessages(): Promise<void> {
		const cursor = messageCursor.value
		if (!cursor || !readyForQuery()) return
		await appendPage('Loading more WhatsApp messages…', async (token) => {
			const page = await listWhatsAppOperationalMessages({
				accountId: selectedAccountId.value,
				providerChatId: selectedChatId.value || undefined,
				cursor,
			})
			if (!current(token)) return
			messages.value = appendUnique(
				messages.value,
				page.item,
				(message) => `${message.providerChatId}:${message.providerMessageId}`,
			)
			messageCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreParticipants(): Promise<void> {
		const cursor = participantCursor.value
		if (!cursor || !selectedChatId.value || !readyForQuery()) return
		await appendPage('Loading more WhatsApp participants…', async (token) => {
			const page = await listWhatsAppOperationalParticipants({
				accountId: selectedAccountId.value,
				providerChatId: selectedChatId.value,
				cursor,
			})
			if (!current(token)) return
			participants.value = appendUnique(
				participants.value,
				page.item,
				(participant) => participant.providerIdentityId,
			)
			participantCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreEvents(): Promise<void> {
		const cursor = eventCursor.value
		if (!cursor || !readyForQuery()) return
		await appendPage('Loading more WhatsApp events…', async (token) => {
			const page = await listWhatsAppOperationalEvents({
				accountId: selectedAccountId.value,
				cursor,
			})
			if (!current(token)) return
			events.value = [...events.value, ...page.item]
			eventCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreSearchResults(): Promise<void> {
		const cursor = searchCursor.value
		const query = searchQuery.value.trim()
		if (!cursor || !query || !readyForQuery()) return
		await appendPage('Loading more WhatsApp search results…', async (token) => {
			const page = await searchWhatsAppOperationalMessages({
				accountId: selectedAccountId.value,
				providerChatId: selectedChatId.value || undefined,
				searchQuery: query,
				cursor,
			})
			if (!current(token)) return
			searchResults.value = appendUnique(
				searchResults.value,
				page.item,
				(message) => `${message.providerChatId}:${message.providerMessageId}`,
			)
			searchCursor.value = page.nextCursor ?? ''
		})
	}

	function updateSearchQuery(value: string): void {
		searchQuery.value = value
		if (value.trim()) return
		searchResults.value = []
		searchCursor.value = ''
	}

	async function loadConversation(providerChatId: string, token: number): Promise<void> {
		selectedChatId.value = providerChatId
		messages.value = []
		participants.value = []
		messageCursor.value = ''
		participantCursor.value = ''
		searchResults.value = []
		searchCursor.value = ''
		const messageRequest = listWhatsAppOperationalMessages({
			accountId: selectedAccountId.value,
			providerChatId: providerChatId || undefined,
		})
		const participantRequest = providerChatId
			? listWhatsAppOperationalParticipants({
				accountId: selectedAccountId.value,
				providerChatId,
			})
			: Promise.resolve(undefined)
		const [messagePage, participantPage] = await Promise.all([
			messageRequest,
			participantRequest,
		])
		if (!current(token)) return
		messages.value = messagePage.item
		messageCursor.value = messagePage.nextCursor ?? ''
		participants.value = participantPage?.item ?? []
		participantCursor.value = participantPage?.nextCursor ?? ''
	}

	async function appendPage(
		message: string,
		work: (token: number) => Promise<void>,
	): Promise<void> {
		const token = ++generation
		begin(message)
		try {
			await work(token)
			if (current(token)) completeReady()
		} catch (error) {
			fail(error, token, 'WhatsApp operational page could not be extended.')
		}
	}

	function readyForQuery(): boolean {
		if (!input.canQuery()) {
			clear('WhatsApp operational query capability is not admitted.')
			return false
		}
		if (!selectedAccountId.value) {
			clear('Select an admitted WhatsApp account.')
			state.value = 'empty'
			return false
		}
		return true
	}

	function begin(message: string): void {
		state.value = 'loading'
		statusMessage.value = message
	}

	function completeReady(): void {
		state.value = 'ready'
		statusMessage.value = ''
	}

	function finishProjection(): void {
		if (
			dialogs.value.length === 0
			&& messages.value.length === 0
			&& events.value.length === 0
		) {
			state.value = 'empty'
			statusMessage.value = 'No WhatsApp operational records are available for this account.'
			return
		}
		completeReady()
	}

	function clear(message: string): void {
		generation += 1
		selectedAccountId.value = ''
		resetProjection()
		state.value = 'blocked'
		statusMessage.value = message
	}

	function resetProjection(): void {
		selectedChatId.value = ''
		searchQuery.value = ''
		runtime.value = undefined
		dialogs.value = []
		messages.value = []
		participants.value = []
		events.value = []
		searchResults.value = []
		dialogCursor.value = ''
		messageCursor.value = ''
		participantCursor.value = ''
		eventCursor.value = ''
		searchCursor.value = ''
	}

	function fail(error: unknown, token: number, fallback: string): void {
		if (!current(token)) return
		state.value = 'error'
		statusMessage.value = error instanceof Error ? error.message : fallback
	}

	function current(token: number): boolean {
		return token === generation
	}

	return {
		model,
		loadMoreDialogs,
		loadMoreEvents,
		loadMoreMessages,
		loadMoreParticipants,
		loadMoreSearchResults,
		reconcile,
		refresh,
		search,
		selectAccount,
		selectDialog,
		updateSearchQuery,
	}
}

function appendUnique<T>(
	current: readonly T[],
	next: readonly T[],
	key: (value: T) => string,
): readonly T[] {
	const existing = new Set(current.map(key))
	return [...current, ...next.filter((value) => !existing.has(key(value)))]
}
