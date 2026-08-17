import { fromBinary } from '@bufbuild/protobuf'

import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { WhatsAppOperationalProjectionChangedV1Schema } from '../../../gen/makosh/whatsapp/v1/client_pb'
import {
	getBrowserGatewayRealtimeHubByAccount,
	type BrowserGatewayRealtimeHub,
} from '../../../platform/gateway/browserGatewayRealtimeHub'

const CONTRACT = 'whatsapp.operational.projection_changed.v1'

export type WhatsAppOperationalRealtimeBinding = { close(): void }

export function openWhatsAppOperationalRealtime(
	accountId: string,
	input: {
		onProjectionChanged(revision: bigint): void
		onUnavailable(): void
	},
	hub?: Pick<BrowserGatewayRealtimeHub, 'subscribe'>,
): WhatsAppOperationalRealtimeBinding {
	const expectedAccountId = accountId.trim()
	if (!expectedAccountId) throw new Error('whatsapp_operational_realtime_account_invalid')
	const resolvedHub = hub ?? getBrowserGatewayRealtimeHubByAccount({
		provider: 'whatsapp',
		accountId: expectedAccountId,
	})
	let observedRevision = 0n
	return resolvedHub.subscribe({
		onEvent: event => {
			const change = decodeProjectionChanged(event)
			if (change?.accountId === expectedAccountId
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
	accountId: string
	revision: bigint
} | undefined {
	if (event.contractName !== CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== CONTRACT) return undefined
	try {
		const change = fromBinary(WhatsAppOperationalProjectionChangedV1Schema, event.payload)
		return change.accountId.trim() && change.revision > 0n
			? { accountId: change.accountId, revision: change.revision }
			: undefined
	} catch {
		return undefined
	}
}
