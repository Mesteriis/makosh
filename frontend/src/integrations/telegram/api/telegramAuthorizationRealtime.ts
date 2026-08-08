import { fromBinary } from '@bufbuild/protobuf'

import type { ClientRealtimeEventV1 } from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { AuthorizationStatusResponseSchema } from '../../../gen/makosh/telegram/v1/client_pb'
import {
	getBrowserGatewayRealtimeHub,
	type BrowserGatewayRealtimeHub,
} from '../../../platform/gateway/browserGatewayRealtimeHub'

const CONTRACT = 'telegram.authorization.status_changed.v1'
const STATES = new Set([
	'waiting_parameters',
	'waiting_encryption_key',
	'waiting_qr_scan',
	'waiting_password',
	'ready',
	'closing',
	'closed',
	'error',
	'other',
])

export type TelegramAuthorizationRealtimeBinding = { close(): void }

export function openTelegramAuthorizationRealtime(
	onStatusChanged: (state: string) => void,
	onUnavailable: () => void,
	hub: Pick<BrowserGatewayRealtimeHub, 'subscribe'> = getBrowserGatewayRealtimeHub(),
): TelegramAuthorizationRealtimeBinding {
	return hub.subscribe({
		onEvent: event => {
			const state = decodeStatusChanged(event)
			if (state) onStatusChanged(state)
		},
		onStreamState: () => {},
		onReplayGap: onUnavailable,
		onProtocolError: onUnavailable,
	})
}

function decodeStatusChanged(event: ClientRealtimeEventV1): string | undefined {
	if (event.contractName !== CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== CONTRACT) return undefined
	try {
		const status = fromBinary(AuthorizationStatusResponseSchema, event.payload)
		return STATES.has(status.state)
			&& status.qrLink === undefined
			&& status.passwordHint === undefined
			? status.state
			: undefined
	} catch {
		return undefined
	}
}
