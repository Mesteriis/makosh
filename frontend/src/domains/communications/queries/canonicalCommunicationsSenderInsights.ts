import {
	SenderInsightsErrorCodeV1,
	type SenderInsightV1,
} from '../../../gen/makosh/communications/sender_insights/v1/sender_insights_pb'
import { getCommunicationsSenderInsightsConnectClient } from '../../../platform/connect/communicationsSenderInsightsClient'
import type { CanonicalCommunicationsPage } from './canonicalCommunicationsRead'

const MAX_PAGE_LIMIT = 100
const MAX_CURSOR_BYTES = 68

export async function listCanonicalSenderInsights(
	accountId?: Uint8Array,
	limit = 20,
	cursor: Uint8Array = new Uint8Array(),
): Promise<CanonicalCommunicationsPage<SenderInsightV1>> {
	const response = await getCommunicationsSenderInsightsConnectClient().list({
		protocolMajor: 1,
		accountId: optionalId16(accountId),
		limit: pageLimit(limit),
		cursor: boundedCursor(cursor),
	})
	if (response.error !== SenderInsightsErrorCodeV1.SENDER_INSIGHTS_ERROR_CODE_UNSPECIFIED) {
		throw new CanonicalSenderInsightsError(response.error)
	}
	return {
		items: response.items,
		nextCursor: boundedResponseCursor(response.nextCursor),
	}
}

export class CanonicalSenderInsightsError extends Error {
	readonly code: SenderInsightsErrorCodeV1

	constructor(code: SenderInsightsErrorCodeV1) {
		super(senderInsightsErrorMessage(code))
		this.name = 'CanonicalSenderInsightsError'
		this.code = code
	}
}

function senderInsightsErrorMessage(code: SenderInsightsErrorCodeV1): string {
	switch (code) {
		case SenderInsightsErrorCodeV1.SENDER_INSIGHTS_ERROR_CODE_INVALID_REQUEST:
			return 'Sender-insights request is invalid'
		case SenderInsightsErrorCodeV1.SENDER_INSIGHTS_ERROR_CODE_NOT_FOUND:
			return 'Canonical account was not found'
		default:
			return 'Sender insights are unavailable'
	}
}

function pageLimit(value: number): number {
	if (!Number.isInteger(value) || value < 1 || value > MAX_PAGE_LIMIT) {
		throw new RangeError(`Sender-insights limit must be between 1 and ${MAX_PAGE_LIMIT}`)
	}
	return value
}

function boundedCursor(value: Uint8Array): Uint8Array {
	if (value.byteLength > MAX_CURSOR_BYTES) throw new RangeError('Sender-insights cursor is too large')
	return value
}

function boundedResponseCursor(value: Uint8Array): Uint8Array {
	if (value.byteLength > MAX_CURSOR_BYTES) throw new Error('Sender-insights response cursor is invalid')
	return value
}

function optionalId16(value: Uint8Array | undefined): Uint8Array | undefined {
	if (!value) return undefined
	if (value.byteLength !== 16 || value.every((byte) => byte === 0)) {
		throw new RangeError('Canonical account identifier must be 16 non-zero bytes')
	}
	return value
}
