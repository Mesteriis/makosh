import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateKindV1,
	ClientRealtimeStreamStateV1Schema,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { ZulipOperationalProjectionChangedV1Schema } from '../../../gen/makosh/zulip/v1/client_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import { openZulipOperationalRealtime } from './zulipOperationalRealtime'

describe('Zulip operational realtime adapter', () => {
	it('demultiplexes client-safe projection changes by account', () => {
		let observer: BrowserGatewayRealtimeObserver | undefined
		const onProjectionChanged = vi.fn()
		const onUnavailable = vi.fn()
		const binding = openZulipOperationalRealtime('account-a', {
			onProjectionChanged,
			onUnavailable,
		}, {
			subscribe: vi.fn((value) => {
				observer = value
				return { close: vi.fn() }
			}),
		} as never)

		observer?.onEvent(event('account-b', 1n))
		observer?.onEvent(event('account-a', 2n))
		observer?.onEvent(event('account-a', 2n))
		observer?.onEvent(event('account-a', 1n))
		observer?.onStreamState(create(ClientRealtimeStreamStateV1Schema, {
			state: ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_CLOSED,
		}))

		expect(onProjectionChanged).toHaveBeenCalledOnce()
		expect(onProjectionChanged).toHaveBeenCalledWith(2n)
		expect(onUnavailable).toHaveBeenCalledOnce()
		binding.close()
	})
})

function event(accountId: string, revision: bigint): ClientRealtimeEventV1 {
	return create(ClientRealtimeEventV1Schema, {
		contractName: 'zulip.operational.projection_changed.v1',
		contractVersion: 1,
		eventKind: 'zulip.operational.projection_changed.v1',
		payload: toBinary(
			ZulipOperationalProjectionChangedV1Schema,
			create(ZulipOperationalProjectionChangedV1Schema, { accountId, revision }),
		),
	})
}
