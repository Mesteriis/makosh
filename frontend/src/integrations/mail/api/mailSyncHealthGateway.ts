import { create } from '@bufbuild/protobuf'

import {
	GetMailSyncRunQueryV1Schema,
	GetMailSyncStatusQueryV1Schema,
	ListMailSyncRunsQueryV1Schema,
	MailSyncHealthQueryV1Schema,
	type MailSyncHealthQueryV1,
	type MailSyncRunPageV1,
	type MailSyncRunV1,
	type MailSyncStatusV1,
} from '../../../gen/makosh/mail/sync_health/v1/client_pb'
import { getMailSyncHealthConnectClient } from './mailSyncHealthClient'

const DEFAULT_PAGE_LIMIT = 50
const MAX_PAGE_LIMIT = 200
const MAX_IDENTIFIER_BYTES = 512
const textEncoder = new TextEncoder()

export async function getMailSyncStatus(
	connectionId: string,
): Promise<MailSyncStatusV1> {
	const response = await query({
		case: 'getStatus',
		value: create(GetMailSyncStatusQueryV1Schema, {
			connectionId: identifier('connection ID', connectionId),
		}),
	})
	if (response.response.case !== 'status') {
		throw new Error('Mail sync status response is unavailable')
	}
	return response.response.value
}

export async function listMailSyncRuns(input: {
	connectionId: string
	cursor?: string
	limit?: number
}): Promise<MailSyncRunPageV1> {
	const response = await query({
		case: 'listRuns',
		value: create(ListMailSyncRunsQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			cursor: optionalIdentifier('cursor', input.cursor),
			limit: pageLimit(input.limit),
		}),
	})
	if (response.response.case !== 'runs') {
		throw new Error('Mail sync runs response is unavailable')
	}
	return response.response.value
}

export async function getMailSyncRun(input: {
	connectionId: string
	operationId: string
}): Promise<MailSyncRunV1 | null> {
	const response = await query({
		case: 'getRun',
		value: create(GetMailSyncRunQueryV1Schema, {
			connectionId: identifier('connection ID', input.connectionId),
			operationId: identifier('operation ID', input.operationId),
		}),
	})
	if (response.response.case !== 'run') {
		throw new Error('Mail sync run response is unavailable')
	}
	return response.response.value.run ?? null
}

function query(queryInput: MailSyncHealthQueryV1['query']) {
	return getMailSyncHealthConnectClient().query(
		create(MailSyncHealthQueryV1Schema, { query: queryInput }),
	)
}

function identifier(label: string, value: string): string {
	const normalized = value.trim()
	if (
		!normalized
		|| textEncoder.encode(normalized).length > MAX_IDENTIFIER_BYTES
		|| hasControlCharacter(normalized)
	) {
		throw new RangeError(`Mail sync ${label} is invalid`)
	}
	return normalized
}

function optionalIdentifier(label: string, value?: string): string | undefined {
	return value === undefined ? undefined : identifier(label, value)
}

function pageLimit(value = DEFAULT_PAGE_LIMIT): number {
	if (!Number.isInteger(value) || value < 1 || value > MAX_PAGE_LIMIT) {
		throw new RangeError('Mail sync page limit must be between 1 and 200')
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
