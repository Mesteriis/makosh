import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateKindV1,
	ClientRealtimeStreamStateV1Schema,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { WhatsAppOperationalProjectionChangedV1Schema } from '../../../gen/makosh/whatsapp/v1/client_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import { openWhatsAppOperationalRealtime } from './whatsAppOperationalRealtime'

describe('WhatsApp operational realtime adapter', () => {
	it('demultiplexes client-safe projection changes by account', () => {
		let observer: BrowserGatewayRealtimeObserver | undefined
		const onProjectionChanged = vi.fn()
		const onUnavailable = vi.fn()
		const binding = openWhatsAppOperationalRealtime('account-a', {
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
		contractName: 'whatsapp.operational.projection_changed.v1',
		contractVersion: 1,
		eventKind: 'whatsapp.operational.projection_changed.v1',
		payload: toBinary(
			WhatsAppOperationalProjectionChangedV1Schema,
			create(WhatsAppOperationalProjectionChangedV1Schema, { accountId, revision }),
		),
	})
}
