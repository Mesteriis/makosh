import { computed, ref, shallowRef, watch } from 'vue'
import type { SenderInsightV1 } from '../../../gen/makosh/communications/sender_insights/v1/sender_insights_pb'
import {
	buildCanonicalSenderInsightRows,
	type CanonicalSenderInsightsPanelModel,
	type CanonicalSenderInsightsPanelStatus,
} from '../presentation/canonicalSenderInsightsPanelModel'
import { bytesKey } from '../presentation/canonicalCommunicationsPageModel'
import { listCanonicalSenderInsights } from './canonicalCommunicationsSenderInsights'
import type { CanonicalCommunicationsPage } from './canonicalCommunicationsRead'

type SenderInsightsOperations = {
	list(
		accountId?: Uint8Array,
		limit?: number,
		cursor?: Uint8Array,
	): Promise<CanonicalCommunicationsPage<SenderInsightV1>>
}

const DEFAULT_OPERATIONS: SenderInsightsOperations = {
	list: listCanonicalSenderInsights,
}

export function useCanonicalCommunicationsSenderInsights(
	canRead: () => boolean,
	currentAccount: () => Uint8Array | undefined,
	operations: SenderInsightsOperations = DEFAULT_OPERATIONS,
) {
	const items = ref<readonly SenderInsightV1[]>([])
	const cursor = shallowRef<Uint8Array>(new Uint8Array())
	const scopeCurrentAccount = ref(false)
	const status = ref<CanonicalSenderInsightsPanelStatus>('loading')
	const statusMessage = ref('Loading sender insights…')
	let requestGeneration = 0

	const model = computed<CanonicalSenderInsightsPanelModel>(() => ({
		status: status.value,
		statusMessage: statusMessage.value,
		scopeCurrentAccount: scopeCurrentAccount.value,
		canScopeToCurrentAccount: currentAccount() !== undefined,
		items: buildCanonicalSenderInsightRows(items.value),
		hasMore: cursor.value.byteLength > 0,
		busy: status.value === 'loading',
	}))

	async function load(): Promise<void> {
		const generation = ++requestGeneration
		if (!guardCapability()) return
		status.value = 'loading'
		statusMessage.value = 'Loading sender insights…'
		try {
			const page = await operations.list(scopedAccount())
			if (generation !== requestGeneration) return
			items.value = page.items
			cursor.value = page.nextCursor
			status.value = 'ready'
			statusMessage.value = page.items.length === 0
				? 'No incoming sender evidence is available yet.'
				: ''
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'Sender insights are temporarily unavailable.'
		}
	}

	async function loadMore(): Promise<void> {
		if (!guardCapability() || cursor.value.byteLength === 0) return
		const generation = ++requestGeneration
		status.value = 'loading'
		try {
			const page = await operations.list(scopedAccount(), 20, cursor.value)
			if (generation !== requestGeneration) return
			items.value = appendUnique(items.value, page.items)
			cursor.value = page.nextCursor
			status.value = 'ready'
			statusMessage.value = ''
		} catch {
			if (generation !== requestGeneration) return
			status.value = 'error'
			statusMessage.value = 'More sender insights could not be loaded.'
		}
	}

	function updateScopeCurrentAccount(value: boolean): void {
		scopeCurrentAccount.value = value && currentAccount() !== undefined
		void load()
	}

	function scopedAccount(): Uint8Array | undefined {
		return scopeCurrentAccount.value ? currentAccount()?.slice() : undefined
	}

	function guardCapability(): boolean {
		if (canRead()) return true
		requestGeneration += 1
		items.value = []
		cursor.value = new Uint8Array()
		status.value = 'unavailable'
		statusMessage.value = 'Sender insights are not admitted for this runtime.'
		return false
	}

	watch(
		() => bytesKey(currentAccount() ?? new Uint8Array()),
		() => {
			if (!scopeCurrentAccount.value) return
			if (!currentAccount()) scopeCurrentAccount.value = false
			void load()
		},
	)

	return {
		load,
		loadMore,
		model,
		requestGeneration: () => requestGeneration,
		updateScopeCurrentAccount,
	}
}

function appendUnique(
	current: readonly SenderInsightV1[],
	next: readonly SenderInsightV1[],
): readonly SenderInsightV1[] {
	const keys = new Set(current.map((item) => bytesKey(item.senderId)))
	return [...current, ...next.filter((item) => !keys.has(bytesKey(item.senderId)))]
}
