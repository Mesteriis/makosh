import type { CommunicationSearchHitV1 } from '../../../gen/makosh/communications/query/v1/query_pb'
import { getCommunicationsQueryConnectClient } from '../../../platform/connect/communicationsQueryClient'
import type { CanonicalCommunicationsPage } from './canonicalCommunicationsRead'

const MAX_SEARCH_LIMIT = 100
const MAX_CURSOR_BYTES = 64

export async function searchCanonicalCommunications(
	query: string,
	limit = 20,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<CommunicationSearchHitV1>> {
	const normalizedQuery = query.trim()
	if (!normalizedQuery) {
		throw new RangeError('Communications search query must not be empty')
	}
	if (!Number.isInteger(limit) || limit < 1 || limit > MAX_SEARCH_LIMIT) {
		throw new RangeError(`Communications search limit must be between 1 and ${MAX_SEARCH_LIMIT}`)
	}
	if (cursor.byteLength > MAX_CURSOR_BYTES) {
		throw new RangeError(`Communications search cursor must not exceed ${MAX_CURSOR_BYTES} bytes`)
	}

	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'searchCommunications',
			value: { query: normalizedQuery, limit, cursor },
		},
	})
	if (response.errorCode || response.result.case !== 'searchCommunications') {
		throw new Error('Communications canonical search is unavailable')
	}
	if (response.result.value.nextCursor.byteLength > MAX_CURSOR_BYTES) {
		throw new Error('Communications canonical search returned an invalid cursor')
	}

	return {
		items: response.result.value.hits,
		nextCursor: response.result.value.nextCursor,
	}
}
