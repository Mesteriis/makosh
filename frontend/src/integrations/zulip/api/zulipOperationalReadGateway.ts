import { create } from '@bufbuild/protobuf'

import {
	GetAccountStatusQuerySchema,
	ListConversationsQuerySchema,
	ListEventsQuerySchema,
	ListMessagesQuerySchema,
	SearchMessagesQuerySchema,
	type ZulipAccountStatusV1,
	type ZulipConversationPageV1,
	type ZulipEventPageV1,
	type ZulipMessagePageV1,
	ZulipOperationalQueryV1Schema,
	type ZulipOperationalEventKindV1,
	type ZulipOperationalQueryV1,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import { getZulipOperationalReadConnectClient } from './zulipOperationalReadClient'

const DEFAULT_PAGE_LIMIT = 50
const MAX_PAGE_LIMIT = 200
const MAX_IDENTIFIER_BYTES = 512
const MAX_SEARCH_QUERY_BYTES = 1_024
const textEncoder = new TextEncoder()

export type ZulipOperationalPageInput = {
	accountId: string
	cursor?: string
	limit?: number
}

export async function listZulipOperationalMessages(
	input: ZulipOperationalPageInput & { providerConversationId?: string },
): Promise<ZulipMessagePageV1> {
	const response = await query({
		case: 'listMessages',
		value: create(ListMessagesQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			providerConversationId: optionalIdentifier(
				'provider conversation ID',
				input.providerConversationId,
			),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'messages') {
		throw new Error('Zulip messages response is unavailable')
	}
	return response.response.value
}

export async function searchZulipOperationalMessages(
	input: ZulipOperationalPageInput & {
		providerConversationId?: string
		searchQuery: string
	},
): Promise<ZulipMessagePageV1> {
	const response = await query({
		case: 'searchMessages',
		value: create(SearchMessagesQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			providerConversationId: optionalIdentifier(
				'provider conversation ID',
				input.providerConversationId,
			),
			query: searchQuery(input.searchQuery),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'messages') {
		throw new Error('Zulip search response is unavailable')
	}
	return response.response.value
}

export async function listZulipOperationalConversations(
	input: ZulipOperationalPageInput,
): Promise<ZulipConversationPageV1> {
	const response = await query({
		case: 'listConversations',
		value: create(ListConversationsQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'conversations') {
		throw new Error('Zulip conversations response is unavailable')
	}
	return response.response.value
}

export async function listZulipOperationalEvents(
	input: ZulipOperationalPageInput & {
		kind?: ZulipOperationalEventKindV1
		providerConversationId?: string
	},
): Promise<ZulipEventPageV1> {
	const response = await query({
		case: 'listEvents',
		value: create(ListEventsQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			kind: input.kind,
			providerConversationId: optionalIdentifier(
				'provider conversation ID',
				input.providerConversationId,
			),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'events') {
		throw new Error('Zulip events response is unavailable')
	}
	return response.response.value
}

export async function getZulipOperationalAccountStatus(
	accountId: string,
): Promise<ZulipAccountStatusV1> {
	const response = await query({
		case: 'getAccountStatus',
		value: create(GetAccountStatusQuerySchema, {
			accountId: identifier('account ID', accountId),
		}),
	})
	if (response.response.case !== 'accountStatus') {
		throw new Error('Zulip account status response is unavailable')
	}
	return response.response.value
}

function query(queryInput: ZulipOperationalQueryV1['query']) {
	return getZulipOperationalReadConnectClient().query(
		create(ZulipOperationalQueryV1Schema, { query: queryInput }),
	)
}

function identifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!validText(normalized, MAX_IDENTIFIER_BYTES)) {
		throw new RangeError(`Zulip ${label} is invalid`)
	}
	return normalized
}

function optionalIdentifier(label: string, value?: string): string | undefined {
	return value === undefined ? undefined : identifier(label, value)
}

function searchQuery(value: string): string {
	const normalized = value.trim()
	if (!validText(normalized, MAX_SEARCH_QUERY_BYTES)) {
		throw new RangeError('Zulip search query is invalid')
	}
	return normalized
}

function optionalCursor(value?: string): string | undefined {
	if (value === undefined) return undefined
	const normalized = value.trim()
	if (
		!normalized
		|| textEncoder.encode(normalized).length > MAX_IDENTIFIER_BYTES
		|| !/^[\x21-\x7e]+$/.test(normalized)
	) {
		throw new RangeError('Zulip cursor is invalid')
	}
	return normalized
}

function pageLimit(value = DEFAULT_PAGE_LIMIT): number {
	if (!Number.isInteger(value) || value < 1 || value > MAX_PAGE_LIMIT) {
		throw new RangeError('Zulip page limit must be between 1 and 200')
	}
	return value
}

function validText(value: string, maxBytes: number): boolean {
	if (!value || textEncoder.encode(value).length > maxBytes) return false
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code === 0 || code === 0x0a || code === 0x0d) return false
	}
	return true
}
