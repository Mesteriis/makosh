import { create } from '@bufbuild/protobuf'

import {
	WhatsAppOperationalReplayRequestV1Schema,
	type WhatsAppOperationalReplayResponseV1,
} from '../../../gen/makosh/whatsapp/operational/realtime/v1/client_pb'
import { getWhatsAppOperationalRealtimeConnectClient } from './whatsAppOperationalRealtimeClient'

const DEFAULT_REPLAY_LIMIT = 100
const MAX_REPLAY_LIMIT = 500
const MAX_ACCOUNT_ID_BYTES = 512
const MAX_REPLAY_SEQUENCE = 9_223_372_036_854_775_807n
const textEncoder = new TextEncoder()

export async function replayWhatsAppOperationalEvents(input: {
	accountId: string
	afterSequence?: bigint
	limit?: number
}): Promise<WhatsAppOperationalReplayResponseV1> {
	const accountId = input.accountId.trim()
	if (
		!accountId
		|| textEncoder.encode(accountId).length > MAX_ACCOUNT_ID_BYTES
		|| hasControlCharacter(accountId)
	) {
		throw new RangeError('WhatsApp replay account ID is invalid')
	}
	const afterSequence = input.afterSequence ?? 0n
	if (afterSequence < 0n || afterSequence > MAX_REPLAY_SEQUENCE) {
		throw new RangeError('WhatsApp replay sequence is invalid')
	}
	const limit = input.limit ?? DEFAULT_REPLAY_LIMIT
	if (!Number.isInteger(limit) || limit < 1 || limit > MAX_REPLAY_LIMIT) {
		throw new RangeError('WhatsApp replay limit must be between 1 and 500')
	}
	const response = await getWhatsAppOperationalRealtimeConnectClient().replay(
		create(WhatsAppOperationalReplayRequestV1Schema, {
			accountId,
			afterSequence,
			limit,
		}),
	)
	if (response.accountId !== accountId) {
		throw new Error('WhatsApp replay account response is invalid')
	}
	return response
}

function hasControlCharacter(value: string): boolean {
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return true
	}
	return false
}
