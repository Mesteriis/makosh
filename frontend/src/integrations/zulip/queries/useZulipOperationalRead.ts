import { computed, ref, shallowRef } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import type {
	ZulipAccountStatusV1,
	ZulipConversationV1,
	ZulipMessageV1,
	ZulipOperationalEventV1,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import {
	getZulipOperationalAccountStatus,
	listZulipOperationalConversations,
	listZulipOperationalEvents,
	listZulipOperationalMessages,
	searchZulipOperationalMessages,
} from '../api/zulipOperationalReadGateway'
import {
	buildZulipOperationalReadModel,
	type ZulipOperationalReadModel,
	type ZulipOperationalReadState,
} from '../presentation/zulipOperationalReadModel'
import { zulipOperationalQueryAccounts } from './zulipOperationalAccounts'

export function useZulipOperationalRead(input: {
	canQuery: () => boolean
	modules: () => readonly ClientModuleBootstrapV1[]
}) {
	const state = ref<ZulipOperationalReadState>('blocked')
	const statusMessage = ref('')
	const selectedAccountId = ref('')
	const selectedConversationId = ref('')
	const searchQuery = ref('')
	const accountStatus = shallowRef<ZulipAccountStatusV1>()
	const conversations = shallowRef<readonly ZulipConversationV1[]>([])
	const messages = shallowRef<readonly ZulipMessageV1[]>([])
	const events = shallowRef<readonly ZulipOperationalEventV1[]>([])
	const searchResults = shallowRef<readonly ZulipMessageV1[]>([])
	const conversationCursor = ref('')
	const messageCursor = ref('')
	const eventCursor = ref('')
	const searchCursor = ref('')
	let generation = 0

	const accounts = computed(() => zulipOperationalQueryAccounts(input.modules()))
	const model = computed<ZulipOperationalReadModel>(() => (
		buildZulipOperationalReadModel({
			canQuery: input.canQuery(),
			state: state.value,
			statusMessage: statusMessage.value,
			accounts: accounts.value,
			selectedAccountId: selectedAccountId.value,
			selectedConversationId: selectedConversationId.value,
			searchQuery: searchQuery.value,
			accountStatus: accountStatus.value,
			conversations: conversations.value,
			messages: messages.value,
			events: events.value,
			searchResults: searchResults.value,
			hasMoreConversations: Boolean(conversationCursor.value),
			hasMoreMessages: Boolean(messageCursor.value),
			hasMoreEvents: Boolean(eventCursor.value),
			hasMoreSearchResults: Boolean(searchCursor.value),
		})
	))

	async function reconcile(): Promise<void> {
		const available = accounts.value
		if (!input.canQuery()) {
			clear('Zulip operational query capability is not admitted.')
			return
		}
		if (available.length === 0) {
			clear('No admitted Zulip account is available in effective integration settings.')
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
		begin('Loading Zulip operational projection…')
		resetProjection()
		try {
			const [status, conversationPage, eventPage] = await Promise.all([
				getZulipOperationalAccountStatus(selectedAccountId.value),
				listZulipOperationalConversations({ accountId: selectedAccountId.value }),
				listZulipOperationalEvents({ accountId: selectedAccountId.value }),
			])
			if (!current(token)) return
			accountStatus.value = status
			conversations.value = conversationPage.item
			conversationCursor.value = conversationPage.nextCursor ?? ''
			events.value = eventPage.item
			eventCursor.value = eventPage.nextCursor ?? ''
			await loadConversation(
				conversationPage.item[0]?.providerConversationId ?? '',
				token,
			)
			if (current(token)) finishProjection()
		} catch (error) {
			fail(error, token, 'Zulip operational projection is unavailable.')
		}
	}

	async function selectAccount(accountId: string): Promise<void> {
		if (!accounts.value.some((account) => account.accountId === accountId)) return
		selectedAccountId.value = accountId
		await refresh()
	}

	async function selectConversation(providerConversationId: string): Promise<void> {
		if (!conversations.value.some(
			(conversation) => conversation.providerConversationId === providerConversationId,
		)) return
		const token = ++generation
		begin('Loading Zulip conversation…')
		try {
			await loadConversation(providerConversationId, token)
			if (current(token)) completeReady()
		} catch (error) {
			fail(error, token, 'Zulip conversation is unavailable.')
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
		begin('Searching Zulip messages…')
		try {
			const page = await searchZulipOperationalMessages({
				accountId: selectedAccountId.value,
				providerConversationId: selectedConversationId.value || undefined,
				searchQuery: query,
			})
			if (!current(token)) return
			searchResults.value = page.item
			searchCursor.value = page.nextCursor ?? ''
			completeReady()
		} catch (error) {
			fail(error, token, 'Zulip search is unavailable.')
		}
	}

	async function loadMoreConversations(): Promise<void> {
		const cursor = conversationCursor.value
		if (!cursor || !readyForQuery()) return
		await appendPage('Loading more Zulip conversations…', async (token) => {
			const page = await listZulipOperationalConversations({
				accountId: selectedAccountId.value,
				cursor,
			})
			if (!current(token)) return
			conversations.value = appendUnique(
				conversations.value,
				page.item,
				(conversation) => conversation.providerConversationId,
			)
			conversationCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreMessages(): Promise<void> {
		const cursor = messageCursor.value
		if (!cursor || !readyForQuery()) return
		await appendPage('Loading more Zulip messages…', async (token) => {
			const page = await listZulipOperationalMessages({
				accountId: selectedAccountId.value,
				providerConversationId: selectedConversationId.value || undefined,
				cursor,
			})
			if (!current(token)) return
			messages.value = appendUnique(
				messages.value,
				page.item,
				(message) => message.providerMessageId,
			)
			messageCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreEvents(): Promise<void> {
		const cursor = eventCursor.value
		if (!cursor || !readyForQuery()) return
		await appendPage('Loading more Zulip events…', async (token) => {
			const page = await listZulipOperationalEvents({
				accountId: selectedAccountId.value,
				cursor,
			})
			if (!current(token)) return
			events.value = appendUnique(
				events.value,
				page.item,
				(event) => `${event.providerEventId}:${event.providerMessageId}`,
			)
			eventCursor.value = page.nextCursor ?? ''
		})
	}

	async function loadMoreSearchResults(): Promise<void> {
		const cursor = searchCursor.value
		const query = searchQuery.value.trim()
		if (!cursor || !query || !readyForQuery()) return
		await appendPage('Loading more Zulip search results…', async (token) => {
			const page = await searchZulipOperationalMessages({
				accountId: selectedAccountId.value,
				providerConversationId: selectedConversationId.value || undefined,
				searchQuery: query,
				cursor,
			})
			if (!current(token)) return
			searchResults.value = appendUnique(
				searchResults.value,
				page.item,
				(message) => message.providerMessageId,
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

	async function loadConversation(
		providerConversationId: string,
		token: number,
	): Promise<void> {
		selectedConversationId.value = providerConversationId
		messages.value = []
		messageCursor.value = ''
		searchResults.value = []
		searchCursor.value = ''
		const page = await listZulipOperationalMessages({
			accountId: selectedAccountId.value,
			providerConversationId: providerConversationId || undefined,
		})
		if (!current(token)) return
		messages.value = page.item
		messageCursor.value = page.nextCursor ?? ''
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
			fail(error, token, 'Zulip operational page could not be extended.')
		}
	}

	function readyForQuery(): boolean {
		if (!input.canQuery()) {
			clear('Zulip operational query capability is not admitted.')
			return false
		}
		if (!selectedAccountId.value) {
			clear('Select an admitted Zulip account.')
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
			conversations.value.length === 0
			&& messages.value.length === 0
			&& events.value.length === 0
		) {
			state.value = 'empty'
			statusMessage.value = 'No Zulip operational records are available for this account.'
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
		selectedConversationId.value = ''
		searchQuery.value = ''
		accountStatus.value = undefined
		conversations.value = []
		messages.value = []
		events.value = []
		searchResults.value = []
		conversationCursor.value = ''
		messageCursor.value = ''
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
		loadMoreConversations,
		loadMoreEvents,
		loadMoreMessages,
		loadMoreSearchResults,
		reconcile,
		refresh,
		search,
		selectAccount,
		selectConversation,
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
