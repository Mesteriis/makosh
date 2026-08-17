import { fromBinary } from '@bufbuild/protobuf'

import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { MailOperationalProjectionChangedV1Schema } from '../../../gen/makosh/mail/v1/client_pb'
import {
	getBrowserGatewayRealtimeHubByAccount,
	type BrowserGatewayRealtimeHub,
} from '../../../platform/gateway/browserGatewayRealtimeHub'

const CONTRACT = 'mail.operational.projection_changed.v1'

export type MailOperationalRealtimeBinding = { close(): void }

export function openMailOperationalRealtime(
	connectionId: string,
	input: {
		onProjectionChanged(revision: bigint): void
		onUnavailable(): void
	},
	hub?: Pick<BrowserGatewayRealtimeHub, 'subscribe'>,
): MailOperationalRealtimeBinding {
	const expectedConnectionId = connectionId.trim()
	if (!expectedConnectionId) throw new Error('mail_operational_realtime_connection_invalid')
	const resolvedHub = hub ?? getBrowserGatewayRealtimeHubByAccount({
		provider: 'mail',
		accountId: expectedConnectionId,
	})
	let observedRevision = 0n
	return resolvedHub.subscribe({
		onEvent: event => {
			const change = decodeProjectionChanged(event)
			if (change?.connectionId === expectedConnectionId
				&& change.revision > observedRevision) {
				observedRevision = change.revision
				input.onProjectionChanged(change.revision)
			}
		},
		onStreamState: state => {
			if (state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_CLOSED) {
				input.onUnavailable()
			}
		},
		onReplayGap: input.onUnavailable,
		onProtocolError: input.onUnavailable,
	})
}

function decodeProjectionChanged(event: ClientRealtimeEventV1): {
	connectionId: string
	revision: bigint
} | undefined {
	if (event.contractName !== CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== CONTRACT) return undefined
	try {
		const change = fromBinary(MailOperationalProjectionChangedV1Schema, event.payload)
		return change.connectionId.trim() && change.revision > 0n
			? { connectionId: change.connectionId, revision: change.revision }
			: undefined
	} catch {
		return undefined
	}
}
