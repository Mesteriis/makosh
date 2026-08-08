import {
	BrowserGatewayRealtime,
	type BrowserGatewayRealtimeObserver,
	type BrowserGatewayRealtimeSubscription,
} from './browserGatewayRealtime'
import type { ClientRealtimeStreamStateV1 } from '../../gen/makosh/gateway/v1/client_realtime_pb'

type BrowserGatewayRealtimePort = {
	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription
}

export class BrowserGatewayRealtimeHub {
	private readonly observers = new Set<BrowserGatewayRealtimeObserver>()
	private readonly realtime: BrowserGatewayRealtimePort
	private sourceSubscription?: BrowserGatewayRealtimeSubscription
	private streamState?: ClientRealtimeStreamStateV1

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
				if (this.observers.size === 0) this.closeSource()
			},
		}
	}

	private openSource(): void {
		if (this.sourceSubscription) return
		this.sourceSubscription = this.realtime.subscribe({
			onEvent: event => this.deliver(observer => observer.onEvent(event)),
			onStreamState: state => {
				this.streamState = state
				this.deliver(observer => observer.onStreamState(state))
			},
			onReplayGap: gap => {
				this.closeSource()
				this.deliver(observer => observer.onReplayGap(gap))
			},
			onProtocolError: () => {
				this.closeSource()
				this.deliver(observer => observer.onProtocolError())
			},
		})
	}

	private closeSource(): void {
		this.sourceSubscription?.close()
		this.sourceSubscription = undefined
		this.streamState = undefined
	}

	private deliver(delivery: (observer: BrowserGatewayRealtimeObserver) => void): void {
		for (const observer of this.observers) delivery(observer)
	}
}

let sharedHub: BrowserGatewayRealtimeHub | undefined

export function getBrowserGatewayRealtimeHub(): BrowserGatewayRealtimeHub {
	sharedHub ??= new BrowserGatewayRealtimeHub()
	return sharedHub
}

export function resetBrowserGatewayRealtimeHubForTests(): void {
	sharedHub = undefined
}
