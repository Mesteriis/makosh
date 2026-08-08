import { create } from '@bufbuild/protobuf'

import {
	ZulipOperationalReplayRequestV1Schema,
	type ZulipOperationalReplayResponseV1,
} from '../../../gen/makosh/zulip/operational/realtime/v1/client_pb'
import { getZulipOperationalRealtimeConnectClient } from './zulipOperationalRealtimeClient'

const DEFAULT_REPLAY_LIMIT = 100
const MAX_REPLAY_LIMIT = 200
const MAX_ACCOUNT_ID_BYTES = 512
const MAX_REPLAY_SEQUENCE = 18_446_744_073_709_551_615n
const textEncoder = new TextEncoder()

export async function replayZulipOperationalEvents(input: {
	accountId: string
	afterSequence?: bigint
	limit?: number
}): Promise<ZulipOperationalReplayResponseV1> {
	const accountId = input.accountId.trim()
	if (
		!accountId
		|| textEncoder.encode(accountId).length > MAX_ACCOUNT_ID_BYTES
		|| hasForbiddenIdentifierCharacter(accountId)
	) {
		throw new RangeError('Zulip replay account ID is invalid')
	}
	const afterSequence = input.afterSequence ?? 0n
	if (afterSequence < 0n || afterSequence > MAX_REPLAY_SEQUENCE) {
		throw new RangeError('Zulip replay sequence is invalid')
	}
	const limit = input.limit ?? DEFAULT_REPLAY_LIMIT
	if (!Number.isInteger(limit) || limit < 1 || limit > MAX_REPLAY_LIMIT) {
		throw new RangeError('Zulip replay limit must be between 1 and 200')
	}
	const response = await getZulipOperationalRealtimeConnectClient().replay(
		create(ZulipOperationalReplayRequestV1Schema, {
			accountId,
			afterSequence,
			limit,
		}),
	)
	if (response.accountId !== accountId) {
		throw new Error('Zulip replay account response is invalid')
	}
	return response
}

function hasForbiddenIdentifierCharacter(value: string): boolean {
	return value.includes('\u0000') || value.includes('\r') || value.includes('\n')
}
