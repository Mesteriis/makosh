import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeFrameV1Schema,
	ClientReplayGapV1Schema,
	type ClientRealtimeFrameV1,
} from '../../gen/makosh/gateway/v1/client_realtime_pb'
import { BrowserGatewayRealtime, type BrowserGatewayEventSource } from './browserGatewayRealtime'

class EventSourceFixture implements BrowserGatewayEventSource {
	readonly listeners = new Map<string, (event: Event) => void>()
	readonly close = vi.fn()

	addEventListener(type: string, listener: (event: Event) => void): void {
		this.listeners.set(type, listener)
	}

	emit(type: string, data: string): void {
		this.listeners.get(type)?.(new MessageEvent(type, { data }))
	}
}

describe('BrowserGatewayRealtime', () => {
	it('uses same-origin EventSource replay rather than a cursor URL and decodes only the typed frame', () => {
		const source = new EventSourceFixture()
		const factory = vi.fn(() => source)
		const onEvent = vi.fn()
		const onStreamState = vi.fn()
		const onReplayGap = vi.fn()
		const onProtocolError = vi.fn()
		new BrowserGatewayRealtime({ eventSourceFactory: factory }).subscribe({
			onEvent,
			onStreamState,
			onReplayGap,
			onProtocolError,
		})

		expect(factory).toHaveBeenCalledWith('/api/realtime/v1/events', { withCredentials: true })
		source.emit('makosh.realtime.v1', frameData({
			case: 'event',
			value: create(ClientRealtimeEventV1Schema, {
				eventId: new Uint8Array([7]),
				cursor: 'cursor-1',
				contractName: 'makosh.client.status',
				contractVersion: 1,
				eventKind: 'status_changed',
				occurredAtUnixMillis: 1n,
				payload: new Uint8Array([3]),
			}),
		}))

		expect(onEvent).toHaveBeenCalledWith(expect.objectContaining({ cursor: 'cursor-1' }))
		expect(source.close).not.toHaveBeenCalled()
		expect(onStreamState).not.toHaveBeenCalled()
		expect(onReplayGap).not.toHaveBeenCalled()
		expect(onProtocolError).not.toHaveBeenCalled()
	})

	it('makes an explicit replay gap terminal and fails closed on malformed frames', () => {
		const source = new EventSourceFixture()
		const onReplayGap = vi.fn()
		const onProtocolError = vi.fn()
		new BrowserGatewayRealtime({ eventSourceFactory: () => source }).subscribe({
			onEvent: vi.fn(),
			onStreamState: vi.fn(),
			onReplayGap,
			onProtocolError,
		})

		source.emit('makosh.realtime.v1', frameData({
			case: 'replayGap',
			value: create(ClientReplayGapV1Schema, { reasonCode: 'live_buffer_overrun' }),
		}))
		expect(source.close).toHaveBeenCalledTimes(1)
		expect(onReplayGap).toHaveBeenCalledWith(expect.objectContaining({ reasonCode: 'live_buffer_overrun' }))

		source.emit('makosh.realtime.v1', 'invalid payload')
		expect(source.close).toHaveBeenCalledTimes(2)
		expect(onProtocolError).toHaveBeenCalledTimes(1)
	})
})

function frameData(frame: ClientRealtimeFrameV1['frame']): string {
	return bytesToBase64Url(toBinary(ClientRealtimeFrameV1Schema, create(ClientRealtimeFrameV1Schema, { frame })))
}

function bytesToBase64Url(value: Uint8Array): string {
	let binary = ''
	for (const byte of value) binary += String.fromCharCode(byte)
	return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}
