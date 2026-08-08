import { fromBinary } from '@bufbuild/protobuf'

import {
	ClientRealtimeFrameV1Schema,
	type ClientRealtimeEventV1,
	type ClientRealtimeFrameV1,
	type ClientRealtimeStreamStateV1,
	type ClientReplayGapV1,
} from '../../gen/makosh/gateway/v1/client_realtime_pb'

const REALTIME_PATH = '/api/realtime/v1/events'
const REALTIME_EVENT_TYPE = 'makosh.realtime.v1'

export type BrowserGatewayEventSource = {
	addEventListener(type: string, listener: (event: Event) => void): void
	close(): void
}

export type BrowserGatewayRealtimeOptions = {
	eventSourceFactory?: (url: string, init: EventSourceInit) => BrowserGatewayEventSource
}

export type BrowserGatewayRealtimeObserver = {
	onEvent(event: ClientRealtimeEventV1): void
	onStreamState(state: ClientRealtimeStreamStateV1): void
	onReplayGap(gap: ClientReplayGapV1): void
	onProtocolError(): void
}

export type BrowserGatewayRealtimeSubscription = {
	close(): void
}

/**
 * Browser-only client realtime boundary. EventSource reconnects with the last
 * SSE id itself, so this adapter never puts a replay cursor in a URL.
 */
export class BrowserGatewayRealtime {
	private readonly eventSourceFactory: (url: string, init: EventSourceInit) => BrowserGatewayEventSource

	constructor(options: BrowserGatewayRealtimeOptions = {}) {
		this.eventSourceFactory = options.eventSourceFactory ?? defaultEventSource
	}

	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription {
		const source = this.eventSourceFactory(REALTIME_PATH, { withCredentials: true })
		source.addEventListener(REALTIME_EVENT_TYPE, event => {
			const frame = decodeFrame(event)
			if (!frame) {
				source.close()
				observer.onProtocolError()
				return
			}
			deliverFrame(frame, source, observer)
		})
		return { close: () => source.close() }
	}
}

function defaultEventSource(url: string, init: EventSourceInit): BrowserGatewayEventSource {
	return new EventSource(url, init)
}

function decodeFrame(event: Event): ClientRealtimeFrameV1 | undefined {
	const data = (event as Event & { data?: unknown }).data
	if (typeof data !== 'string') return undefined
	try {
		return fromBinary(ClientRealtimeFrameV1Schema, base64UrlBytes(data))
	} catch {
		return undefined
	}
}

function deliverFrame(
	frame: ClientRealtimeFrameV1,
	source: BrowserGatewayEventSource,
	observer: BrowserGatewayRealtimeObserver,
): void {
	switch (frame.frame.case) {
		case 'event':
			observer.onEvent(frame.frame.value)
			return
		case 'streamState':
			observer.onStreamState(frame.frame.value)
			return
		case 'replayGap':
			source.close()
			observer.onReplayGap(frame.frame.value)
			return
		default:
			source.close()
			observer.onProtocolError()
	}
}

function base64UrlBytes(value: string): Uint8Array {
	if (!value || !/^[A-Za-z0-9_-]+$/.test(value)) throw new Error('realtime payload is invalid')
	const padded = value.replace(/-/g, '+').replace(/_/g, '/').padEnd(Math.ceil(value.length / 4) * 4, '=')
	const binary = atob(padded)
	return Uint8Array.from(binary, byte => byte.charCodeAt(0))
}
