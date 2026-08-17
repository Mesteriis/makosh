import {
	BrowserGatewayRealtime,
	type BrowserGatewayRealtimeObserver,
	type BrowserGatewayRealtimeSubscription,
} from './browserGatewayRealtime'
import type { ClientRealtimeStreamStateV1 } from '../../gen/makosh/gateway/v1/client_realtime_pb'

type BrowserGatewayRealtimePort = {
	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription
}

type BrowserGatewayRealtimeSignalKind = 'event' | 'stream_state' | 'replay_gap' | 'protocol_error'

export class BrowserGatewayRealtimeHub {
	private readonly observers = new Set<BrowserGatewayRealtimeObserver>()
	private readonly realtime: BrowserGatewayRealtimePort
	private sourceSubscription?: BrowserGatewayRealtimeSubscription
	private streamState?: ClientRealtimeStreamStateV1
	private reconnectTimer?: ReturnType<typeof setTimeout>

	constructor(realtime: BrowserGatewayRealtimePort = new BrowserGatewayRealtime()) {
		this.realtime = realtime
	}

	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription {
		this.observers.add(observer)
		this.openSource()
		if (this.streamState) observer.onStreamState(this.streamState)
		let closed = false
		return {
			close: () => {
				if (closed) return
				closed = true
				this.observers.delete(observer)
				if (this.observers.size === 0) {
					this.cancelReconnect()
					this.closeSource()
				}
			},
		}
	}

	private openSource(): void {
		if (this.sourceSubscription || this.observers.size === 0) return
		this.cancelReconnect()
		this.sourceSubscription = this.realtime.subscribe({
			onEvent: event => this.deliver('event', observer => observer.onEvent(event)),
			onStreamState: state => {
				this.streamState = state
				this.deliver('stream_state', observer => observer.onStreamState(state))
			},
			onReplayGap: gap => {
				this.closeSource()
				this.deliver('replay_gap', observer => observer.onReplayGap(gap))
				this.scheduleReconnect()
			},
			onProtocolError: () => {
				this.closeSource()
				this.deliver('protocol_error', observer => observer.onProtocolError())
				this.scheduleReconnect()
			},
		})
	}

	private closeSource(): void {
		this.sourceSubscription?.close()
		this.sourceSubscription = undefined
		this.streamState = undefined
	}

	private scheduleReconnect(): void {
		if (this.reconnectTimer || this.observers.size === 0) return
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = undefined
			this.openSource()
		}, 1_000)
	}

	private cancelReconnect(): void {
		if (!this.reconnectTimer) return
		clearTimeout(this.reconnectTimer)
		this.reconnectTimer = undefined
	}

	private deliver(
		signalKind: BrowserGatewayRealtimeSignalKind,
		delivery: (observer: BrowserGatewayRealtimeObserver) => void,
	): void {
		for (const observer of this.observers) {
			try {
				delivery(observer)
			} catch {
				console.error('browser_gateway_realtime_observer_delivery_failed', { signalKind })
			}
		}
	}
}

let sharedHub: BrowserGatewayRealtimeHub | undefined

export type BrowserGatewayRealtimeHubIdentity = {
	provider: string
	accountId: string
}

export function getBrowserGatewayRealtimeHub(): BrowserGatewayRealtimeHub {
	sharedHub ??= new BrowserGatewayRealtimeHub()
	return sharedHub
}

export function getBrowserGatewayRealtimeHubByAccount(
	identity: BrowserGatewayRealtimeHubIdentity,
): BrowserGatewayRealtimeHub {
	const provider = identity.provider.trim()
	const accountId = identity.accountId.trim()
	if (!provider || !accountId) {
		throw new Error('browser_realtime_hub_identity_invalid')
	}
	return getBrowserGatewayRealtimeHub()
}

export function resetBrowserGatewayRealtimeHubForTests(): void {
	sharedHub = undefined
}
