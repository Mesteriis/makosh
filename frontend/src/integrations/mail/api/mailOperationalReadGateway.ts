import { create } from '@bufbuild/protobuf'

import {
	GetMailMessageQueryV1Schema,
	ListMailFoldersQueryV1Schema,
	ListMailMessagesQueryV1Schema,
	ListMailThreadsQueryV1Schema,
	MailOperationalQueryV1Schema,
	type MailFolderPageV1,
	type MailMessageDetailV1,
	type MailMessagePageV1,
	type MailOperationalQueryV1,
	type MailThreadPageV1,
} from '../../../gen/makosh/mail/operational/v1/client_pb'
import { getMailOperationalQueryConnectClient } from './mailOperationalQueryClient'

const DEFAULT_PAGE_LIMIT = 50
const MAX_PAGE_LIMIT = 200
const MAX_IDENTIFIER_BYTES = 512
const textEncoder = new TextEncoder()

export type MailOperationalPageInput = {
	connectionId: string
	cursor?: string
	limit?: number
}

export async function listMailOperationalFolders(
	input: MailOperationalPageInput,
): Promise<MailFolderPageV1> {
	const response = await query({
		case: 'listFolders',
		value: create(ListMailFoldersQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			cursor: optionalIdentifier('cursor', input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'folders') {
		throw new Error('Mail folders response is unavailable')
	}
	return response.response.value
}

export async function listMailOperationalThreads(
	input: MailOperationalPageInput & { folderId?: string },
): Promise<MailThreadPageV1> {
	const response = await query({
		case: 'listThreads',
		value: create(ListMailThreadsQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			folderId: optionalIdentifier('folder ID', input.folderId),
			cursor: optionalIdentifier('cursor', input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'threads') {
		throw new Error('Mail threads response is unavailable')
	}
	return response.response.value
}

export async function listMailOperationalMessages(
	input: MailOperationalPageInput & {
		folderId?: string
		providerThreadId?: string
	},
): Promise<MailMessagePageV1> {
	const response = await query({
		case: 'listMessages',
		value: create(ListMailMessagesQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			folderId: optionalIdentifier('folder ID', input.folderId),
			providerThreadId: optionalIdentifier('provider thread ID', input.providerThreadId),
			cursor: optionalIdentifier('cursor', input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'messages') {
		throw new Error('Mail messages response is unavailable')
	}
	return response.response.value
}

export async function getMailOperationalMessage(input: {
	connectionId: string
	messageId: string
}): Promise<MailMessageDetailV1> {
	const response = await query({
		case: 'getMessage',
		value: create(GetMailMessageQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			messageId: identifier('provider message ID', input.messageId),
		}),
	})
	if (response.response.case !== 'message' || !response.response.value.summary) {
		throw new Error('Mail message response is unavailable')
	}
	return response.response.value
}

function query(
	queryInput: MailOperationalQueryV1['query'],
) {
	return getMailOperationalQueryConnectClient().query(
		create(MailOperationalQueryV1Schema, { query: queryInput }),
	)
}

function identifier(label: string, value: string): string {
	const normalized = value.trim()
	if (
		!normalized
		|| textEncoder.encode(normalized).length > MAX_IDENTIFIER_BYTES
		|| hasControlCharacter(normalized)
	) {
		throw new RangeError(`Mail ${label} is invalid`)
	}
	return normalized
}

function optionalIdentifier(label: string, value?: string): string | undefined {
	return value === undefined ? undefined : identifier(label, value)
}

function pageLimit(value = DEFAULT_PAGE_LIMIT): number {
	if (!Number.isInteger(value) || value < 1 || value > MAX_PAGE_LIMIT) {
		throw new RangeError('Mail page limit must be between 1 and 200')
	}
	return value
}

function hasControlCharacter(value: string): boolean {
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return true
	}
	return false
}
