import type {
	SavedSearchHitV1,
	SavedSearchSummaryV1,
} from '../../../gen/makosh/communications/saved_search/v1/saved_search_pb'
import {
	buildCanonicalSearchRows,
	bytesKey,
	type CanonicalSearchResultRow,
} from './canonicalCommunicationsPageModel'

export type CanonicalSavedSearchPanelStatus =
	| 'unavailable'
	| 'loading'
	| 'ready'
	| 'error'
	| 'mutating'
	| 'executing'

export type CanonicalSavedSearchRow = {
	key: string
	name: string
	description: string
	scopeLabel: string
	tokenLabel: string
	revisionLabel: string
	updatedLabel: string
	active: boolean
}

export type CanonicalSavedSearchPanelModel = {
	status: CanonicalSavedSearchPanelStatus
	statusMessage: string
	name: string
	description: string
	scopeCurrentAccount: boolean
	canScopeToCurrentAccount: boolean
	items: readonly CanonicalSavedSearchRow[]
	results: readonly CanonicalSearchResultRow[]
	hasMoreItems: boolean
	hasMoreResults: boolean
	busy: boolean
}

export function buildCanonicalSavedSearchRows(
	items: readonly SavedSearchSummaryV1[],
	activeKey: string,
): readonly CanonicalSavedSearchRow[] {
	return items.map((item) => ({
		key: bytesKey(item.savedSearchId),
		name: item.name,
		description: item.description ?? '',
		scopeLabel: item.accountId
			? `Canonical account #${bytesKey(item.accountId).slice(0, 12)}`
			: 'All canonical accounts',
		tokenLabel: `${item.tokenCount} exact token${item.tokenCount === 1 ? '' : 's'}`,
		revisionLabel: `Revision ${item.revision}`,
		updatedLabel: formatUnixSeconds(item.updatedAtUnixSeconds),
		active: bytesKey(item.savedSearchId) === activeKey,
	}))
}

export function buildCanonicalSavedSearchResults(
	hits: readonly SavedSearchHitV1[],
	selectedMessageKey: string,
): readonly CanonicalSearchResultRow[] {
	return buildCanonicalSearchRows(hits, selectedMessageKey)
}

function formatUnixSeconds(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds)) return 'Time unavailable'
	const date = new Date(milliseconds)
	if (Number.isNaN(date.getTime())) return 'Time unavailable'
	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(date)
}
