import { computed, ref, shallowRef } from 'vue'

import type {
	AccountSummaryV1,
	CommunicationSearchHitV1,
	ConversationSummaryV1,
	MessageSummaryV1,
} from '../../../gen/makosh/communications/query/v1/query_pb'
import {
	buildCanonicalAccountRows,
	buildCanonicalConversationRows,
	buildCanonicalMessageRows,
	buildCanonicalSearchRows,
	bytesKey,
	type CanonicalCommunicationsPageModel,
	type CanonicalCommunicationsPageStatus,
	type CanonicalCommunicationsSearchStatus,
} from '../presentation/canonicalCommunicationsPageModel'
import {
	listCanonicalCommunicationAccounts,
	listCanonicalConversationMessages,
	listCanonicalConversations,
} from './canonicalCommunicationsRead'
import { searchCanonicalCommunications } from './canonicalCommunicationsSearch'

export function useCanonicalCommunicationsPage() {
	const accounts = ref<readonly AccountSummaryV1[]>([])
	const conversations = ref<readonly ConversationSummaryV1[]>([])
	const messages = ref<readonly MessageSummaryV1[]>([])
	const searchResults = ref<readonly CommunicationSearchHitV1[]>([])
	const accountCursor = shallowRef<Uint8Array>(new Uint8Array())
	const conversationCursor = shallowRef<Uint8Array>(new Uint8Array())
	const messageCursor = shallowRef<Uint8Array>(new Uint8Array())
	const searchCursor = shallowRef<Uint8Array>(new Uint8Array())
	const selectedAccountKey = ref('')
	const selectedConversationKey = ref('')
	const selectedMessageKey = ref('')
	const searchText = ref('')
	const activeSearchQuery = ref('')
	const status = ref<CanonicalCommunicationsPageStatus>('loading')
	const statusMessage = ref('Loading canonical evidence…')
	const searchStatus = ref<CanonicalCommunicationsSearchStatus>('idle')
	const searchMessage = ref('Search uses exact tokens in the owner-local derived index.')
	const loadingMore = ref(false)
	let accountGeneration = 0
	let conversationGeneration = 0
	let messageGeneration = 0
	let searchGeneration = 0

	const model = computed<CanonicalCommunicationsPageModel>(() => ({
		status: status.value,
		statusMessage: statusMessage.value,
		accounts: buildCanonicalAccountRows(accounts.value, selectedAccountKey.value),
		conversations: buildCanonicalConversationRows(
			conversations.value,
			selectedConversationKey.value,
		),
		messages: buildCanonicalMessageRows(messages.value, selectedMessageKey.value),
		searchText: searchText.value,
		searchStatus: searchStatus.value,
		searchMessage: searchMessage.value,
		searchResults: buildCanonicalSearchRows(searchResults.value, selectedMessageKey.value),
		hasMoreAccounts: accountCursor.value.byteLength > 0,
		hasMoreConversations: conversationCursor.value.byteLength > 0,
		hasMoreMessages: messageCursor.value.byteLength > 0,
		hasMoreSearchResults: searchCursor.value.byteLength > 0,
		loadingMore: loadingMore.value,
	}))

	async function load(): Promise<void> {
		const generation = ++accountGeneration
		status.value = 'loading'
		statusMessage.value = 'Loading canonical evidence…'
		try {
			const page = await listCanonicalCommunicationAccounts()
			if (generation !== accountGeneration) return
			accounts.value = page.items
			accountCursor.value = page.nextCursor
			if (page.items.length === 0) {
				clearConversationState()
				status.value = 'empty'
				statusMessage.value = 'No canonical communication evidence has been observed yet.'
				return
			}
			await selectAccount(bytesKey(page.items[0]!.accountId))
		} catch {
			if (generation !== accountGeneration) return
			clearAllState()
			status.value = 'error'
			statusMessage.value = 'Canonical Communications is temporarily unavailable.'
		}
	}

	async function selectAccount(accountKey: string): Promise<void> {
		const account = accounts.value.find((candidate) => bytesKey(candidate.accountId) === accountKey)
		if (!account) return
		selectedAccountKey.value = accountKey
		clearConversationState()
		status.value = 'loading'
		statusMessage.value = 'Loading canonical conversations…'
		const generation = ++conversationGeneration
		try {
			const page = await listCanonicalConversations(account.accountCursorSha256)
			if (generation !== conversationGeneration) return
			conversations.value = page.items
			conversationCursor.value = page.nextCursor
			status.value = 'ready'
			statusMessage.value = page.items.length === 0
				? 'This source has no canonical conversations yet.'
				: ''
			if (page.items[0]) {
				await selectConversation(bytesKey(page.items[0].conversationId))
			}
		} catch {
			if (generation !== conversationGeneration) return
			conversations.value = []
			conversationCursor.value = new Uint8Array()
			clearMessageState()
			status.value = 'error'
			statusMessage.value = 'Canonical conversations are temporarily unavailable.'
		}
	}

	async function selectConversation(conversationKey: string): Promise<void> {
		const conversation = conversations.value.find(
			(candidate) => bytesKey(candidate.conversationId) === conversationKey,
		)
		if (!conversation) return
		selectedConversationKey.value = conversationKey
		clearMessageState()
		const generation = ++messageGeneration
		try {
			const page = await listCanonicalConversationMessages(conversation.conversationId)
			if (generation !== messageGeneration) return
			messages.value = page.items
			messageCursor.value = page.nextCursor
			status.value = 'ready'
			statusMessage.value = page.items.length === 0
				? 'This conversation has no canonical messages yet.'
				: ''
		} catch {
			if (generation !== messageGeneration) return
			clearMessageState()
			status.value = 'error'
			statusMessage.value = 'Canonical messages are temporarily unavailable.'
		}
	}

	async function search(): Promise<void> {
		const query = searchText.value.trim()
		if (!query) {
			searchStatus.value = 'idle'
			searchResults.value = []
			searchCursor.value = new Uint8Array()
			activeSearchQuery.value = ''
			searchMessage.value = 'Enter at least one exact token.'
			return
		}
		const generation = ++searchGeneration
		searchStatus.value = 'loading'
		searchMessage.value = 'Searching canonical evidence…'
		try {
			const page = await searchCanonicalCommunications(query)
			if (generation !== searchGeneration) return
			activeSearchQuery.value = query
			searchResults.value = page.items
			searchCursor.value = page.nextCursor
			searchStatus.value = 'ready'
			searchMessage.value = page.items.length === 0
				? 'No canonical evidence matched those exact tokens.'
				: ''
		} catch {
			if (generation !== searchGeneration) return
			searchResults.value = []
			searchCursor.value = new Uint8Array()
			searchStatus.value = 'error'
			searchMessage.value = 'Canonical search is temporarily unavailable.'
		}
	}

	async function loadMoreAccounts(): Promise<void> {
		if (accountCursor.value.byteLength === 0) return
		const generation = accountGeneration
		await appendPage(async () => {
			const page = await listCanonicalCommunicationAccounts(50, accountCursor.value)
			if (generation !== accountGeneration) return
			accounts.value = appendUnique(accounts.value, page.items, (item) => bytesKey(item.accountId))
			accountCursor.value = page.nextCursor
		})
	}

	async function loadMoreConversations(): Promise<void> {
		const account = accounts.value.find(
			(candidate) => bytesKey(candidate.accountId) === selectedAccountKey.value,
		)
		if (!account || conversationCursor.value.byteLength === 0) return
		const generation = conversationGeneration
		await appendPage(async () => {
			const page = await listCanonicalConversations(
				account.accountCursorSha256,
				100,
				conversationCursor.value,
			)
			if (generation !== conversationGeneration) return
			conversations.value = appendUnique(
				conversations.value,
				page.items,
				(item) => bytesKey(item.conversationId),
			)
			conversationCursor.value = page.nextCursor
		})
	}

	async function loadMoreMessages(): Promise<void> {
		const conversation = conversations.value.find(
			(candidate) => bytesKey(candidate.conversationId) === selectedConversationKey.value,
		)
		if (!conversation || messageCursor.value.byteLength === 0) return
		const generation = messageGeneration
		await appendPage(async () => {
			const page = await listCanonicalConversationMessages(
				conversation.conversationId,
				100,
				messageCursor.value,
			)
			if (generation !== messageGeneration) return
			messages.value = appendUnique(messages.value, page.items, (item) => bytesKey(item.messageId))
			messageCursor.value = page.nextCursor
		})
	}

	async function loadMoreSearchResults(): Promise<void> {
		if (!activeSearchQuery.value || searchCursor.value.byteLength === 0) return
		const generation = searchGeneration
		await appendPage(async () => {
			const page = await searchCanonicalCommunications(
				activeSearchQuery.value,
				20,
				searchCursor.value,
			)
			if (generation !== searchGeneration) return
			searchResults.value = appendUnique(
				searchResults.value,
				page.items,
				(item) => bytesKey(item.evidenceId),
			)
			searchCursor.value = page.nextCursor
		})
	}

	function selectMessage(messageKey: string): Uint8Array | undefined {
		const message = messages.value.find((candidate) => bytesKey(candidate.messageId) === messageKey)
		const searchHit = searchResults.value.find(
			(candidate) => bytesKey(candidate.messageId) === messageKey,
		)
		const messageId = message?.messageId ?? searchHit?.messageId
		if (!messageId) return undefined
		selectedMessageKey.value = messageKey
		return messageId.slice()
	}

	function clearSelectedMessage(): void {
		selectedMessageKey.value = ''
	}

	function updateSearchText(value: string): void {
		searchText.value = value
	}

	function currentSearchDraft(): { query: string; accountId?: Uint8Array } {
		const account = accounts.value.find(
			(candidate) => bytesKey(candidate.accountId) === selectedAccountKey.value,
		)
		return {
			query: searchText.value.trim(),
			accountId: account?.accountId.slice(),
		}
	}

	async function appendPage(loadPage: () => Promise<void>): Promise<void> {
		if (loadingMore.value) return
		loadingMore.value = true
		try {
			await loadPage()
		} finally {
			loadingMore.value = false
		}
	}

	function clearAllState(): void {
		accounts.value = []
		accountCursor.value = new Uint8Array()
		selectedAccountKey.value = ''
		clearConversationState()
	}

	function clearConversationState(): void {
		conversations.value = []
		conversationCursor.value = new Uint8Array()
		selectedConversationKey.value = ''
		conversationGeneration += 1
		clearMessageState()
	}

	function clearMessageState(): void {
		messages.value = []
		messageCursor.value = new Uint8Array()
		selectedMessageKey.value = ''
		messageGeneration += 1
	}

	return {
		clearSelectedMessage,
		currentSearchDraft,
		load,
		loadMoreAccounts,
		loadMoreConversations,
		loadMoreMessages,
		loadMoreSearchResults,
		model,
		search,
		selectAccount,
		selectConversation,
		selectMessage,
		updateSearchText,
	}
}

function appendUnique<T>(
	current: readonly T[],
	next: readonly T[],
	key: (item: T) => string,
): readonly T[] {
	const keys = new Set(current.map(key))
	return [...current, ...next.filter((item) => !keys.has(key(item)))]
}
