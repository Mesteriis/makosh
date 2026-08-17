import { fromBinary } from '@bufbuild/protobuf'

import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { TelegramOperationalProjectionChangedV1Schema } from '../../../gen/makosh/telegram/v1/client_pb'
import {
	getBrowserGatewayRealtimeHubByAccount,
	type BrowserGatewayRealtimeHub,
} from '../../../platform/gateway/browserGatewayRealtimeHub'

const CONTRACT = 'telegram.operational.projection_changed.v1'

export type TelegramOperationalRealtimeBinding = { close(): void }

export function openTelegramOperationalRealtime(
	accountId: string,
	input: {
		onProjectionChanged(latestSequence: bigint): void
		onLive(): void
		onUnavailable(): void
	},
	hub?: Pick<BrowserGatewayRealtimeHub, 'subscribe'>,
): TelegramOperationalRealtimeBinding {
	const expectedAccountId = accountId.trim()
	if (!expectedAccountId) throw new Error('telegram_operational_realtime_account_invalid')
	const resolvedHub = hub ?? getBrowserGatewayRealtimeHubByAccount({
		provider: 'telegram',
		accountId: expectedAccountId,
	})
	let observedRevision = 0n
	return resolvedHub.subscribe({
		onEvent: event => {
			const change = decodeProjectionChanged(event)
			if (change?.accountId === expectedAccountId
				&& change.latestSequence > observedRevision) {
				observedRevision = change.latestSequence
				input.onProjectionChanged(change.latestSequence)
			}
		},
		onStreamState: state => {
			if (state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN) {
				input.onLive()
			} else if (
				state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_CLOSED
			) {
				input.onUnavailable()
			}
		},
		onReplayGap: input.onUnavailable,
		onProtocolError: input.onUnavailable,
	})
}

function decodeProjectionChanged(event: ClientRealtimeEventV1): {
	accountId: string
	latestSequence: bigint
} | undefined {
	if (event.contractName !== CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== CONTRACT) return undefined
	try {
		const change = fromBinary(TelegramOperationalProjectionChangedV1Schema, event.payload)
		return change.accountId.trim() && change.latestSequence > 0n
			? { accountId: change.accountId, latestSequence: change.latestSequence }
			: undefined
	} catch {
		return undefined
	}
}
