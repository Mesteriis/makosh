import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	ClientModuleBootstrapV1Schema,
	ClientSystemComponentStatusV1Schema,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { ClientRealtimeEventV1Schema } from '../../gen/makosh/gateway/v1/client_realtime_pb'
import { ClientSystemStatusChangedV1Schema } from '../../gen/makosh/gateway/v1/client_system_status_realtime_pb'
import type { ClientBootstrapSnapshot } from './clientBootstrap'
import {
	applyClientSystemStatusEvent,
	CLIENT_SYSTEM_STATUS_CONTRACT,
	CLIENT_SYSTEM_STATUS_EVENT_KIND,
} from './browserGatewaySystemStatus'

describe('applyClientSystemStatusEvent', () => {
	it('atomically replaces only the typed system status projection', () => {
		const snapshot = Object.assign(new Map(), {
			modules: [create(ClientModuleBootstrapV1Schema, {
				registrationId: 'mail-1',
				moduleId: 'mail',
			})],
			systemStatus: statuses(ClientSystemComponentStateV1.UNAVAILABLE),
		}) as ClientBootstrapSnapshot
		const updated = applyClientSystemStatusEvent(
			snapshot,
			systemStatusEvent(statuses(ClientSystemComponentStateV1.HEALTHY)),
		)

		expect(updated).toBeDefined()
		expect(updated!.modules).toEqual(snapshot.modules)
		expect(updated!.systemStatus.every(
			(status) => status.state === ClientSystemComponentStateV1.HEALTHY,
		)).toBe(true)
	})

	it('ignores other multiplexed contracts and rejects malformed exact events', () => {
		const snapshot = Object.assign(new Map(), {
			modules: [],
			systemStatus: statuses(ClientSystemComponentStateV1.HEALTHY),
		}) as ClientBootstrapSnapshot
		const unrelated = systemStatusEvent(statuses(ClientSystemComponentStateV1.HEALTHY))
		unrelated.contractName = 'communications.delivery-status'
		expect(applyClientSystemStatusEvent(snapshot, unrelated)).toBeUndefined()

		const malformed = systemStatusEvent(statuses(ClientSystemComponentStateV1.HEALTHY))
		malformed.payload = new Uint8Array([255])
		expect(() => applyClientSystemStatusEvent(snapshot, malformed)).toThrow()
	})
})

function statuses(state: ClientSystemComponentStateV1) {
	return Object.values(ClientSystemComponentIdV1)
		.filter((value): value is ClientSystemComponentIdV1 =>
			typeof value === 'number' && value !== ClientSystemComponentIdV1.UNSPECIFIED)
		.map((componentId) => create(ClientSystemComponentStatusV1Schema, { componentId, state }))
}

function systemStatusEvent(systemStatus: ReturnType<typeof statuses>) {
	return create(ClientRealtimeEventV1Schema, {
		eventId: new Uint8Array([1]),
		cursor: 'gateway-system-status-1',
		contractName: CLIENT_SYSTEM_STATUS_CONTRACT,
		contractVersion: 1,
		eventKind: CLIENT_SYSTEM_STATUS_EVENT_KIND,
		occurredAtUnixMillis: 1n,
		payload: toBinary(ClientSystemStatusChangedV1Schema, create(ClientSystemStatusChangedV1Schema, {
			revision: 1n,
			statuses: systemStatus,
		})),
	})
}
