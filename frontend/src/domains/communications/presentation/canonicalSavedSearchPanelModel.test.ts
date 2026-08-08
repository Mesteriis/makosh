import { describe, expect, it } from 'vitest'

import {
	buildCanonicalSavedSearchResults,
	buildCanonicalSavedSearchRows,
} from './canonicalSavedSearchPanelModel'

describe('canonical saved-search presentation model', () => {
	it('maps only canonical identifiers and definition metadata', () => {
		const savedSearchId = new Uint8Array(16).fill(1)
		const accountId = new Uint8Array(16).fill(2)
		const rows = buildCanonicalSavedSearchRows([{
			$typeName: 'makosh.communications.saved_search.v1.SavedSearchSummaryV1',
			savedSearchId,
			name: 'Unread obligations',
			description: 'Private exact-token definition',
			accountId,
			tokenCount: 2,
			revision: 3n,
			createdAtUnixSeconds: 1n,
			updatedAtUnixSeconds: 2n,
		}], Array.from(savedSearchId, (byte) => byte.toString(16).padStart(2, '0')).join(''))

		expect(rows[0]).toMatchObject({
			name: 'Unread obligations',
			scopeLabel: 'Canonical account #020202020202',
			tokenLabel: '2 exact tokens',
			revisionLabel: 'Revision 3',
			active: true,
		})
		expect(buildCanonicalSavedSearchResults([], '')).toEqual([])
	})
})
