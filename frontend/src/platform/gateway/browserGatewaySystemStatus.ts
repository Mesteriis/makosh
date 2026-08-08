import { fromBinary } from '@bufbuild/protobuf'

import type { ClientRealtimeEventV1 } from '../../gen/makosh/gateway/v1/client_realtime_pb'
import { ClientSystemStatusChangedV1Schema } from '../../gen/makosh/gateway/v1/client_system_status_realtime_pb'
import {
	type ClientBootstrapSnapshot,
	withClientSystemStatus,
} from './clientBootstrap'

export const CLIENT_SYSTEM_STATUS_CONTRACT = 'makosh.gateway.system-status'
export const CLIENT_SYSTEM_STATUS_EVENT_KIND = 'platform.system_status.changed'

export function applyClientSystemStatusEvent(
	snapshot: ClientBootstrapSnapshot,
	event: ClientRealtimeEventV1,
): ClientBootstrapSnapshot | undefined {
	if (event.contractName !== CLIENT_SYSTEM_STATUS_CONTRACT) return undefined
	if (event.contractVersion !== 1 || event.eventKind !== CLIENT_SYSTEM_STATUS_EVENT_KIND) {
		throw new Error('Unsupported client system status event')
	}
	const update = fromBinary(ClientSystemStatusChangedV1Schema, event.payload)
	if (update.revision === 0n) throw new Error('Invalid client system status revision')
	return withClientSystemStatus(snapshot, update.statuses)
}
