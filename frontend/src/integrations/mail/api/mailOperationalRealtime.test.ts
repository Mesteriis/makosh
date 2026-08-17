import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import { ClientRealtimeEventV1Schema } from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { MailOperationalProjectionChangedV1Schema } from '../../../gen/makosh/mail/v1/client_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import { openMailOperationalRealtime } from './mailOperationalRealtime'

describe('Mail operational realtime adapter', () => {
	it('delivers only the selected connection revision', () => {
		let observer: BrowserGatewayRealtimeObserver | undefined
		const changed = vi.fn()
		openMailOperationalRealtime('connection-a', {
			onProjectionChanged: changed,
			onUnavailable: vi.fn(),
		}, {
			subscribe: vi.fn((value) => {
				observer = value
				return { close: vi.fn() }
			}),
		} as never)

		observer?.onEvent(event('connection-b', 1n))
		observer?.onEvent(event('connection-a', 2n))
		observer?.onEvent(event('connection-a', 2n))
		observer?.onEvent(event('connection-a', 1n))
		expect(changed).toHaveBeenCalledOnce()
		expect(changed).toHaveBeenCalledWith(2n)
	})
})

function event(connectionId: string, revision: bigint) {
	return create(ClientRealtimeEventV1Schema, {
		contractName: 'mail.operational.projection_changed.v1',
		contractVersion: 1,
		eventKind: 'mail.operational.projection_changed.v1',
		payload: toBinary(
			MailOperationalProjectionChangedV1Schema,
			create(MailOperationalProjectionChangedV1Schema, { connectionId, revision }),
		),
	})
}
