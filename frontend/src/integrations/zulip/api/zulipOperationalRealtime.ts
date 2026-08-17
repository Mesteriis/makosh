import { fromBinary } from '@bufbuild/protobuf'

import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { ZulipOperationalProjectionChangedV1Schema } from '../../../gen/makosh/zulip/v1/client_pb'
import {
	getBrowserGatewayRealtimeHubByAccount,
	type BrowserGatewayRealtimeHub,
} from '../../../platform/gateway/browserGatewayRealtimeHub'

const CONTRACT = 'zulip.operational.projection_changed.v1'

export type ZulipOperationalRealtimeBinding = { close(): void }

export function openZulipOperationalRealtime(
	accountId: string,
	input: {
		onProjectionChanged(revision: bigint): void
		onUnavailable(): void
	},
	hub?: Pick<BrowserGatewayRealtimeHub, 'subscribe'>,
): ZulipOperationalRealtimeBinding {
	const expectedAccountId = accountId.trim()
	if (!expectedAccountId) throw new Error('zulip_operational_realtime_account_invalid')
	const resolvedHub = hub ?? getBrowserGatewayRealtimeHubByAccount({
		provider: 'zulip',
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
		const change = fromBinary(ZulipOperationalProjectionChangedV1Schema, event.payload)
		return change.accountId.trim() && change.revision > 0n
			? { accountId: change.accountId, revision: change.revision }
			: undefined
	} catch {
		return undefined
	}
}
