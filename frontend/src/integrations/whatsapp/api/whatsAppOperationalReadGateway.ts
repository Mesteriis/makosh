import { create } from '@bufbuild/protobuf'

import {
	GetRuntimeStatusQuerySchema,
	ListDialogsQuerySchema,
	ListEventsQuerySchema,
	ListMessagesQuerySchema,
	ListParticipantsQuerySchema,
	SearchMessagesQuerySchema,
	type WhatsAppDialogPageV1,
	type WhatsAppEventPageV1,
	type WhatsAppMessagePageV1,
	WhatsAppOperationalQueryV1Schema,
	type WhatsAppOperationalQueryV1,
	type WhatsAppOperationalRuntimeStatusV1,
	type WhatsAppParticipantPageV1,
} from '../../../gen/makosh/whatsapp/operational/v1/client_pb'
import type { ProviderEventKind } from '../../../gen/makosh/whatsapp/v1/client_pb'
import { getWhatsAppOperationalReadConnectClient } from './whatsAppOperationalReadClient'

const DEFAULT_PAGE_LIMIT = 50
const MAX_PAGE_LIMIT = 200
const MAX_IDENTIFIER_BYTES = 512
const MAX_SEARCH_QUERY_BYTES = 1_024
const textEncoder = new TextEncoder()

export type WhatsAppOperationalPageInput = {
	accountId: string
	cursor?: string
	limit?: number
}

export async function listWhatsAppOperationalMessages(
	input: WhatsAppOperationalPageInput & { providerChatId?: string },
): Promise<WhatsAppMessagePageV1> {
	const response = await query({
		case: 'listMessages',
		value: create(ListMessagesQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			providerChatId: optionalIdentifier('provider chat ID', input.providerChatId),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'messages') {
		throw new Error('WhatsApp messages response is unavailable')
	}
	return response.response.value
}

export async function searchWhatsAppOperationalMessages(
	input: WhatsAppOperationalPageInput & {
		providerChatId?: string
		searchQuery: string
	},
): Promise<WhatsAppMessagePageV1> {
	const response = await query({
		case: 'searchMessages',
		value: create(SearchMessagesQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			providerChatId: optionalIdentifier('provider chat ID', input.providerChatId),
			query: searchQuery(input.searchQuery),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'messages') {
		throw new Error('WhatsApp search response is unavailable')
	}
	return response.response.value
}

export async function listWhatsAppOperationalDialogs(
	input: WhatsAppOperationalPageInput,
): Promise<WhatsAppDialogPageV1> {
	const response = await query({
		case: 'listDialogs',
		value: create(ListDialogsQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'dialogs') {
		throw new Error('WhatsApp dialogs response is unavailable')
	}
	return response.response.value
}

export async function listWhatsAppOperationalParticipants(
	input: WhatsAppOperationalPageInput & { providerChatId: string },
): Promise<WhatsAppParticipantPageV1> {
	const response = await query({
		case: 'listParticipants',
		value: create(ListParticipantsQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			providerChatId: identifier('provider chat ID', input.providerChatId),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'participants') {
		throw new Error('WhatsApp participants response is unavailable')
	}
	return response.response.value
}

export async function listWhatsAppOperationalEvents(
	input: WhatsAppOperationalPageInput & {
		kind?: ProviderEventKind
		providerChatId?: string
	},
): Promise<WhatsAppEventPageV1> {
	const response = await query({
		case: 'listEvents',
		value: create(ListEventsQuerySchema, {
			accountId: identifier('account ID', input.accountId),
			kind: input.kind,
			providerChatId: optionalIdentifier('provider chat ID', input.providerChatId),
			cursor: optionalCursor(input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'events') {
		throw new Error('WhatsApp events response is unavailable')
	}
	return response.response.value
}

export async function getWhatsAppOperationalRuntimeStatus(
	accountId: string,
): Promise<WhatsAppOperationalRuntimeStatusV1> {
	const response = await query({
		case: 'getRuntimeStatus',
		value: create(GetRuntimeStatusQuerySchema, {
			accountId: identifier('account ID', accountId),
		}),
	})
	if (response.response.case !== 'runtimeStatus') {
		throw new Error('WhatsApp runtime status response is unavailable')
	}
	return response.response.value
}

function query(queryInput: WhatsAppOperationalQueryV1['query']) {
	return getWhatsAppOperationalReadConnectClient().query(
		create(WhatsAppOperationalQueryV1Schema, { query: queryInput }),
	)
}

function identifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!validText(normalized, MAX_IDENTIFIER_BYTES)) {
		throw new RangeError(`WhatsApp ${label} is invalid`)
	}
	return normalized
}

function optionalIdentifier(label: string, value?: string): string | undefined {
	return value === undefined ? undefined : identifier(label, value)
}

function searchQuery(value: string): string {
	const normalized = value.trim()
	if (!validText(normalized, MAX_SEARCH_QUERY_BYTES)) {
		throw new RangeError('WhatsApp search query is invalid')
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
		throw new RangeError('WhatsApp cursor is invalid')
	}
	return normalized
}

function pageLimit(value = DEFAULT_PAGE_LIMIT): number {
	if (!Number.isInteger(value) || value < 1 || value > MAX_PAGE_LIMIT) {
		throw new RangeError('WhatsApp page limit must be between 1 and 200')
	}
	return value
}

function validText(value: string, maxBytes: number): boolean {
	if (!value || textEncoder.encode(value).length > maxBytes) return false
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return false
	}
	return true
}
