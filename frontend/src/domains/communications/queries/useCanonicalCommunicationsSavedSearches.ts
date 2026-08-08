import { computed, ref, shallowRef } from 'vue'
import type {
	SavedSearchHitV1,
	SavedSearchSummaryV1,
} from '../../../gen/makosh/communications/saved_search/v1/saved_search_pb'
import {
	buildCanonicalSavedSearchResults,
	buildCanonicalSavedSearchRows,
	type CanonicalSavedSearchPanelModel,
	type CanonicalSavedSearchPanelStatus,
} from '../presentation/canonicalSavedSearchPanelModel'
import { bytesKey } from '../presentation/canonicalCommunicationsPageModel'
import {
	createCanonicalSavedSearch,
	deleteCanonicalSavedSearch,
	executeCanonicalSavedSearch,
	listCanonicalSavedSearches,
	newCanonicalSavedSearchId,
	replaceCanonicalSavedSearch,
	type CanonicalSavedSearchDraft,
} from './canonicalCommunicationsSavedSearches'

type CurrentCanonicalSearchDraft = {
	query: string
	accountId?: Uint8Array
}

type CanonicalSavedSearchOperations = {
	list: typeof listCanonicalSavedSearches
	create: typeof createCanonicalSavedSearch
	replace: typeof replaceCanonicalSavedSearch
	remove: typeof deleteCanonicalSavedSearch
	execute: typeof executeCanonicalSavedSearch
	newId: typeof newCanonicalSavedSearchId
}

const DEFAULT_OPERATIONS: CanonicalSavedSearchOperations = {
	list: listCanonicalSavedSearches,
	create: createCanonicalSavedSearch,
	replace: replaceCanonicalSavedSearch,
	remove: deleteCanonicalSavedSearch,
	execute: executeCanonicalSavedSearch,
	newId: newCanonicalSavedSearchId,
}

export function useCanonicalCommunicationsSavedSearches(
	canManage: () => boolean,
	currentSearchDraft: () => CurrentCanonicalSearchDraft,
	operations: CanonicalSavedSearchOperations = DEFAULT_OPERATIONS,
) {
	const items = ref<readonly SavedSearchSummaryV1[]>([])
	const results = ref<readonly SavedSearchHitV1[]>([])
	const itemCursor = shallowRef<Uint8Array>(new Uint8Array())
	const resultCursor = shallowRef<Uint8Array>(new Uint8Array())
	const activeSavedSearchKey = ref('')
	const selectedMessageKey = ref('')
	const name = ref('')
	const description = ref('')
	const scopeCurrentAccount = ref(false)
	const status = ref<CanonicalSavedSearchPanelStatus>('loading')
	const statusMessage = ref('Loading saved searches…')
	let requestGeneration = 0

	const model = computed<CanonicalSavedSearchPanelModel>(() => ({
		status: status.value,
		statusMessage: statusMessage.value,
		name: name.value,
		description: description.value,
		scopeCurrentAccount: scopeCurrentAccount.value,
		canScopeToCurrentAccount: currentSearchDraft().accountId !== undefined,
		items: buildCanonicalSavedSearchRows(items.value, activeSavedSearchKey.value),
		results: buildCanonicalSavedSearchResults(results.value, selectedMessageKey.value),
		hasMoreItems: itemCursor.value.byteLength > 0,
		hasMoreResults: resultCursor.value.byteLength > 0,
		busy: status.value === 'loading' || status.value === 'mutating' || status.value === 'executing',
	}))

	async function load(): Promise<void> {
		const generation = ++requestGeneration
		if (!guardCapability()) return
		status.value = 'loading'
		statusMessage.value = 'Loading saved searches…'
		try {
			const page = await operations.list()
			if (generation !== requestGeneration) return
			items.value = page.items
			itemCursor.value = page.nextCursor
			status.value = 'ready'
			statusMessage.value = page.items.length === 0
				? 'No private saved searches yet.'
				: ''
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'Saved searches are temporarily unavailable.'
		}
	}

	async function create(): Promise<void> {
		const draft = mutationDraft()
		if (!guardDraft(draft) || !name.value.trim()) return
		const generation = ++requestGeneration
		status.value = 'mutating'
		statusMessage.value = 'Saving current search…'
		try {
			const item = await operations.create({
				savedSearchId: operations.newId(),
				name: name.value,
				description: description.value,
				accountId: draft.accountId,
				query: draft.query,
			})
			if (generation !== requestGeneration) return
			items.value = [item, ...items.value]
			name.value = ''
			description.value = ''
			status.value = 'ready'
			statusMessage.value = ''
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'The current search could not be saved.'
		}
	}

	async function replace(itemKey: string): Promise<void> {
		const item = findItem(itemKey)
		const draft = mutationDraft()
		if (!item || !guardDraft(draft)) return
		const generation = ++requestGeneration
		status.value = 'mutating'
		statusMessage.value = 'Replacing saved search…'
		try {
			const updated = await operations.replace(item, draft.query, draft.accountId)
			if (generation !== requestGeneration) return
			items.value = items.value.map((candidate) => (
				bytesKey(candidate.savedSearchId) === itemKey ? updated : candidate
			))
			clearExecutionIfRevisionChanged(itemKey)
			status.value = 'ready'
			statusMessage.value = ''
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'The saved search could not be replaced.'
		}
	}

	async function remove(itemKey: string): Promise<void> {
		const item = findItem(itemKey)
		if (!item || !guardCapability()) return
		const generation = ++requestGeneration
		status.value = 'mutating'
		statusMessage.value = 'Deleting saved search…'
		try {
			await operations.remove(item.savedSearchId, item.revision)
			if (generation !== requestGeneration) return
			items.value = items.value.filter((candidate) => bytesKey(candidate.savedSearchId) !== itemKey)
			clearExecutionIfRevisionChanged(itemKey)
			status.value = 'ready'
			statusMessage.value = items.value.length === 0 ? 'No private saved searches yet.' : ''
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'The saved search could not be deleted.'
		}
	}

	async function execute(itemKey: string): Promise<void> {
		const item = findItem(itemKey)
		if (!item || !guardCapability()) return
		const generation = ++requestGeneration
		status.value = 'executing'
		statusMessage.value = 'Running saved search…'
		try {
			const page = await operations.execute(item.savedSearchId)
			if (generation !== requestGeneration) return
			activeSavedSearchKey.value = itemKey
			results.value = page.items
			resultCursor.value = page.nextCursor
			selectedMessageKey.value = ''
			status.value = 'ready'
			statusMessage.value = page.items.length === 0
				? 'No canonical evidence matched this saved search.'
				: ''
		} catch {
			if (generation !== requestGeneration) return
			clearExecution()
			status.value = 'error'
			statusMessage.value = 'The saved search could not be executed.'
		}
	}

	async function loadMoreItems(): Promise<void> {
		if (!guardCapability() || itemCursor.value.byteLength === 0) return
		const generation = ++requestGeneration
		status.value = 'loading'
		try {
			const page = await operations.list(50, itemCursor.value)
			if (generation !== requestGeneration) return
			items.value = appendUnique(items.value, page.items)
			itemCursor.value = page.nextCursor
			status.value = 'ready'
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'More saved searches could not be loaded.'
		}
	}

	async function loadMoreResults(): Promise<void> {
		const item = findItem(activeSavedSearchKey.value)
		if (!item || !guardCapability() || resultCursor.value.byteLength === 0) return
		const generation = ++requestGeneration
		status.value = 'executing'
		try {
			const page = await operations.execute(item.savedSearchId, 20, resultCursor.value)
			if (generation !== requestGeneration) return
			results.value = appendUniqueHits(results.value, page.items)
			resultCursor.value = page.nextCursor
			status.value = 'ready'
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'More saved-search results could not be loaded.'
		}
	}

	function selectMessage(messageKey: string): Uint8Array | undefined {
		const result = results.value.find((candidate) => bytesKey(candidate.messageId) === messageKey)
		if (!result) return undefined
		selectedMessageKey.value = messageKey
		return result.messageId.slice()
	}

	function clearSelectedMessage(): void {
		selectedMessageKey.value = ''
	}

	function updateName(value: string): void {
		name.value = value
	}

	function updateDescription(value: string): void {
		description.value = value
	}

	function updateScopeCurrentAccount(value: boolean): void {
		scopeCurrentAccount.value = value
	}

	function mutationDraft(): CurrentCanonicalSearchDraft {
		const draft = currentSearchDraft()
		return {
			query: draft.query,
			accountId: scopeCurrentAccount.value ? draft.accountId : undefined,
		}
	}

	function guardCapability(): boolean {
		if (canManage()) return true
		clearExecution()
		items.value = []
		itemCursor.value = new Uint8Array()
		status.value = 'unavailable'
		statusMessage.value = 'Saved searches are not admitted for this runtime.'
		return false
	}

	function guardDraft(draft: CurrentCanonicalSearchDraft): draft is Required<Pick<CanonicalSavedSearchDraft, 'query'>> & CurrentCanonicalSearchDraft {
		if (!guardCapability()) return false
		if (draft.query.trim()) return true
		status.value = 'error'
		statusMessage.value = 'Enter a canonical search before saving or replacing it.'
		return false
	}

	function findItem(itemKey: string): SavedSearchSummaryV1 | undefined {
		return items.value.find((candidate) => bytesKey(candidate.savedSearchId) === itemKey)
	}

	function clearExecutionIfRevisionChanged(itemKey: string): void {
		if (activeSavedSearchKey.value === itemKey) clearExecution()
	}

	function clearExecution(): void {
		activeSavedSearchKey.value = ''
		results.value = []
		resultCursor.value = new Uint8Array()
		selectedMessageKey.value = ''
	}

	return {
		clearSelectedMessage,
		create,
		execute,
		load,
		loadMoreItems,
		loadMoreResults,
		model,
		remove,
		replace,
		requestGeneration: () => requestGeneration,
		selectMessage,
		updateDescription,
		updateName,
		updateScopeCurrentAccount,
	}
}

function appendUnique(
	current: readonly SavedSearchSummaryV1[],
	next: readonly SavedSearchSummaryV1[],
): readonly SavedSearchSummaryV1[] {
	const keys = new Set(current.map((item) => bytesKey(item.savedSearchId)))
	return [...current, ...next.filter((item) => !keys.has(bytesKey(item.savedSearchId)))]
}

function appendUniqueHits(
	current: readonly SavedSearchHitV1[],
	next: readonly SavedSearchHitV1[],
): readonly SavedSearchHitV1[] {
	const keys = new Set(current.map((item) => bytesKey(item.evidenceId)))
	return [...current, ...next.filter((item) => !keys.has(bytesKey(item.evidenceId)))]
}
