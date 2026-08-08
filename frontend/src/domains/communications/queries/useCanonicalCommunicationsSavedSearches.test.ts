import { describe, expect, it, vi } from 'vitest'

import type { SavedSearchSummaryV1 } from '../../../gen/makosh/communications/saved_search/v1/saved_search_pb'
import { useCanonicalCommunicationsSavedSearches } from './useCanonicalCommunicationsSavedSearches'

describe('canonical saved-search lifecycle', () => {
	it('is inert without the exact admitted capability', async () => {
		const operations = operationsFixture()
		const controller = useCanonicalCommunicationsSavedSearches(
			() => false,
			() => ({ query: 'obligation' }),
			operations,
		)

		await controller.load()

		expect(controller.model.value.status).toBe('unavailable')
		expect(operations.list).not.toHaveBeenCalled()
	})

	it('fences a stale list response and creates from the transient current query', async () => {
		const pending: Array<(value: {
			items: readonly SavedSearchSummaryV1[]
			nextCursor: Uint8Array<ArrayBuffer>
		}) => void> = []
		const operations = operationsFixture()
		operations.list.mockImplementation(() => new Promise((resolve) => pending.push(resolve)))
		const controller = useCanonicalCommunicationsSavedSearches(
			() => true,
			() => ({ query: 'obligation', accountId: new Uint8Array(16).fill(7) }),
			operations,
		)

		const first = controller.load()
		const second = controller.load()
		pending[0]?.({ items: [summary(1, 'stale')], nextCursor: new Uint8Array() })
		pending[1]?.({ items: [summary(2, 'current')], nextCursor: new Uint8Array() })
		await Promise.all([first, second])
		expect(controller.model.value.items[0]?.name).toBe('current')

		controller.updateName('Follow up')
		controller.updateScopeCurrentAccount(true)
		await controller.create()
		expect(operations.create).toHaveBeenCalledWith(expect.objectContaining({
			name: 'Follow up',
			query: 'obligation',
			accountId: new Uint8Array(16).fill(7),
		}))
	})
})

function operationsFixture() {
	return {
		list: vi.fn(async () => ({
			items: [] as readonly SavedSearchSummaryV1[],
			nextCursor: new Uint8Array(),
		})),
		create: vi.fn(async (draft: { savedSearchId: Uint8Array; name: string }) => (
			summary(draft.savedSearchId[0] ?? 1, draft.name)
		)),
		replace: vi.fn(async (current: SavedSearchSummaryV1) => current),
		remove: vi.fn(async () => 2n),
		execute: vi.fn(async () => ({
			items: [],
			nextCursor: new Uint8Array(),
			definitionRevision: 1n,
		})),
		newId: vi.fn(() => new Uint8Array(16).fill(9)),
	}
}

function summary(seed: number, name: string): SavedSearchSummaryV1 {
	return {
		$typeName: 'makosh.communications.saved_search.v1.SavedSearchSummaryV1',
		savedSearchId: new Uint8Array(16).fill(seed),
		name,
		tokenCount: 1,
		revision: 1n,
		createdAtUnixSeconds: 1n,
		updatedAtUnixSeconds: 1n,
	}
}
