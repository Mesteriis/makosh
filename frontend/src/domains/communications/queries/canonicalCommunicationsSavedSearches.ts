import {
	SavedSearchErrorCodeV1,
	type SavedSearchHitV1,
	type SavedSearchSummaryV1,
} from '../../../gen/makosh/communications/saved_search/v1/saved_search_pb'
import { getCommunicationsSavedSearchConnectClient } from '../../../platform/connect/communicationsSavedSearchClient'
import type { CanonicalCommunicationsPage } from './canonicalCommunicationsRead'

const MAX_PAGE_LIMIT = 100
const MAX_CURSOR_BYTES = 64

export type CanonicalSavedSearchDraft = {
	savedSearchId: Uint8Array
	name: string
	description?: string
	accountId?: Uint8Array
	query: string
}

export async function listCanonicalSavedSearches(
	limit = 50,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<SavedSearchSummaryV1>> {
	const response = await manage({
		case: 'list',
		value: { limit: pageLimit(limit), cursor: boundedCursor(cursor) },
	})
	if (response.result.case !== 'list') throw new Error('Saved-search list response is unavailable')
	return {
		items: response.result.value.items,
		nextCursor: boundedResponseCursor(response.result.value.nextCursor),
	}
}

export async function createCanonicalSavedSearch(
	draft: CanonicalSavedSearchDraft,
): Promise<SavedSearchSummaryV1> {
	const response = await manage({
		case: 'create',
		value: {
			savedSearchId: id16(draft.savedSearchId),
			name: boundedName(draft.name),
			description: boundedDescription(draft.description),
			accountId: optionalId16(draft.accountId),
			query: boundedQuery(draft.query),
		},
	})
	if (response.result.case !== 'mutation' || !response.result.value.item) {
		throw new Error('Saved-search create response is unavailable')
	}
	return response.result.value.item
}

export async function replaceCanonicalSavedSearch(
	current: SavedSearchSummaryV1,
	query: string,
	accountId?: Uint8Array,
): Promise<SavedSearchSummaryV1> {
	const response = await manage({
		case: 'replace',
		value: {
			savedSearchId: id16(current.savedSearchId),
			expectedRevision: current.revision,
			name: boundedName(current.name),
			description: boundedDescription(current.description),
			accountId: optionalId16(accountId),
			query: boundedQuery(query),
		},
	})
	if (response.result.case !== 'mutation' || !response.result.value.item) {
		throw new Error('Saved-search replace response is unavailable')
	}
	return response.result.value.item
}

export async function deleteCanonicalSavedSearch(
	savedSearchId: Uint8Array,
	expectedRevision: bigint,
): Promise<bigint> {
	const response = await manage({
		case: 'delete',
		value: { savedSearchId: id16(savedSearchId), expectedRevision },
	})
	if (response.result.case !== 'delete') throw new Error('Saved-search delete response is unavailable')
	return response.result.value.revision
}

export async function executeCanonicalSavedSearch(
	savedSearchId: Uint8Array,
	limit = 20,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<SavedSearchHitV1> & { definitionRevision: bigint }> {
	const response = await manage({
		case: 'execute',
		value: {
			savedSearchId: id16(savedSearchId),
			limit: pageLimit(limit),
			cursor: boundedCursor(cursor),
		},
	})
	if (response.result.case !== 'execute') {
		throw new Error('Saved-search execution response is unavailable')
	}
	return {
		items: response.result.value.hits,
		nextCursor: boundedResponseCursor(response.result.value.nextCursor),
		definitionRevision: response.result.value.definitionRevision,
	}
}

export function newCanonicalSavedSearchId(): Uint8Array {
	const id = crypto.getRandomValues(new Uint8Array(16))
	if (id.every((byte) => byte === 0)) id[0] = 1
	return id
}

async function manage(
	operation: Parameters<ReturnType<typeof getCommunicationsSavedSearchConnectClient>['manage']>[0]['operation'],
) {
	const response = await getCommunicationsSavedSearchConnectClient().manage({
		protocolMajor: 1,
		operation,
	})
	if (response.error !== SavedSearchErrorCodeV1.SAVED_SEARCH_ERROR_CODE_UNSPECIFIED) {
		throw new CanonicalSavedSearchError(response.error)
	}
	return response
}

export class CanonicalSavedSearchError extends Error {
	readonly code: SavedSearchErrorCodeV1

	constructor(code: SavedSearchErrorCodeV1) {
		super(savedSearchErrorMessage(code))
		this.name = 'CanonicalSavedSearchError'
		this.code = code
	}
}

function savedSearchErrorMessage(code: SavedSearchErrorCodeV1): string {
	switch (code) {
		case SavedSearchErrorCodeV1.SAVED_SEARCH_ERROR_CODE_INVALID_REQUEST:
			return 'Saved search request is invalid'
		case SavedSearchErrorCodeV1.SAVED_SEARCH_ERROR_CODE_NOT_FOUND:
			return 'Saved search was not found'
		case SavedSearchErrorCodeV1.SAVED_SEARCH_ERROR_CODE_REVISION_CONFLICT:
			return 'Saved search changed in another client'
		case SavedSearchErrorCodeV1.SAVED_SEARCH_ERROR_CODE_KEY_REVISION_STALE:
			return 'Saved search must be replaced after search-key rotation'
		default:
			return 'Saved search is unavailable'
	}
}

function pageLimit(value: number): number {
	if (!Number.isInteger(value) || value < 1 || value > MAX_PAGE_LIMIT) {
		throw new RangeError(`Saved-search limit must be between 1 and ${MAX_PAGE_LIMIT}`)
	}
	return value
}

function boundedCursor(value: Uint8Array): Uint8Array {
	if (value.byteLength > MAX_CURSOR_BYTES) throw new RangeError('Saved-search cursor is too large')
	return value
}

function boundedResponseCursor(value: Uint8Array): Uint8Array {
	if (value.byteLength > MAX_CURSOR_BYTES) throw new Error('Saved-search response cursor is invalid')
	return value
}

function id16(value: Uint8Array): Uint8Array {
	if (value.byteLength !== 16 || value.every((byte) => byte === 0)) {
		throw new RangeError('Saved-search identifier must be 16 non-zero bytes')
	}
	return value
}

function optionalId16(value: Uint8Array | undefined): Uint8Array | undefined {
	return value ? id16(value) : undefined
}

function boundedName(value: string): string {
	const normalized = value.trim()
	if (!normalized || new TextEncoder().encode(normalized).byteLength > 128 || hasControls(normalized)) {
		throw new RangeError('Saved-search name is invalid')
	}
	return normalized
}

function boundedDescription(value: string | undefined): string | undefined {
	const normalized = value?.trim()
	if (!normalized) return undefined
	if (new TextEncoder().encode(normalized).byteLength > 512 || hasControls(normalized)) {
		throw new RangeError('Saved-search description is invalid')
	}
	return normalized
}

function boundedQuery(value: string): string {
	const normalized = value.trim()
	if (!normalized || new TextEncoder().encode(normalized).byteLength > 512) {
		throw new RangeError('Saved-search query is invalid')
	}
	return normalized
}

function hasControls(value: string): boolean {
	return Array.from(value).some((character) => {
		const code = character.codePointAt(0) ?? 0
		return code < 0x20 || (code >= 0x7f && code <= 0x9f)
	})
}
