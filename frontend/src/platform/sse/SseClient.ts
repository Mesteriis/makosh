export type SseEventHandler = (event: SseMessageEvent) => void
export type SseErrorHandler = (error: unknown) => void
export type SseTransport = 'sse'
export type SseConnectionState = 'connecting' | 'connected' | 'reconnecting' | 'disconnected'
export type SseStatusEvent = {
	transport: SseTransport
	state: SseConnectionState
	attempt?: number
	maxAttempts?: number
	error?: string
}
export type SseStatusHandler = (status: SseStatusEvent) => void

export type SseMessageEvent = { id: string; event: string; data: string }

export interface SseClientOptions {
	url: string
	secret: string
	lastEventId?: string
	onMessage?: SseEventHandler
	onError?: SseErrorHandler
	onStatus?: SseStatusHandler
	reconnectDelay?: number
	maxReconnectAttempts?: number
	fetchImpl?: typeof fetch
}

/** Fetch-based authenticated SSE client with replay-cursor reconnect. */
export class SseClient {
	private readonly url: string
	private readonly secret: string
	private lastEventId: string
	private readonly onMessage?: SseEventHandler
	private readonly onError?: SseErrorHandler
	private readonly onStatus?: SseStatusHandler
	private readonly reconnectDelay: number
	private readonly maxReconnectAttempts: number
	private readonly fetchImpl: typeof fetch
	private reconnectAttempts = 0
	private shouldReconnect = true
	private abortController: AbortController | null = null
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null
	private buffer = ''

	constructor(options: SseClientOptions) {
		this.url = options.url
		this.secret = options.secret.trim()
		if (!this.secret) throw new Error('x-makosh-secret cannot be empty')
		this.lastEventId = options.lastEventId?.trim() ?? ''
		this.onMessage = options.onMessage
		this.onError = options.onError
		this.onStatus = options.onStatus
		this.reconnectDelay = options.reconnectDelay ?? 3000
		this.maxReconnectAttempts = options.maxReconnectAttempts ?? Number.POSITIVE_INFINITY
		this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis)
	}

	connect(): void {
		this.stopCurrentConnection()
		this.shouldReconnect = true
		this.reconnectAttempts = 0
		this.reportStatus({ transport: 'sse', state: 'connecting' })
		void this.connectOnce()
	}

	disconnect(): void {
		this.shouldReconnect = false
		this.stopCurrentConnection()
		this.reportStatus({ transport: 'sse', state: 'disconnected' })
	}

	private async connectOnce(): Promise<void> {
		this.abortController = new AbortController()
		try {
			const response = await this.fetchImpl(this.replayUrl(), {
				method: 'GET', headers: this.headers(), cache: 'no-store', signal: this.abortController.signal
			})
			if (!response.ok) throw new Error(`SSE connection failed with HTTP ${response.status}`)
			if (!response.body) throw new Error('SSE response body is unavailable')
			this.reportStatus({ transport: 'sse', state: 'connected' })
			await this.readBody(response.body)
			if (this.shouldReconnect) this.scheduleReconnect()
		} catch (error) {
			if (!this.shouldReconnect) return
			this.onError?.(error)
			this.scheduleReconnect(error)
		}
	}

	private async readBody(body: ReadableStream<Uint8Array>): Promise<void> {
		const reader = body.getReader()
		const decoder = new TextDecoder()
		try {
			while (this.shouldReconnect) {
				const { done, value } = await reader.read()
				if (done) break
				this.buffer += decoder.decode(value, { stream: true })
				this.dispatchBufferedEvents()
			}
			this.buffer += decoder.decode()
			this.dispatchBufferedEvents(true)
		} finally { reader.releaseLock() }
	}

	private dispatchBufferedEvents(final = false): void {
		this.buffer = this.buffer.replaceAll('\r\n', '\n').replaceAll('\r', '\n')
		let boundary = this.buffer.indexOf('\n\n')
		while (boundary !== -1) {
			this.dispatchEventBlock(this.buffer.slice(0, boundary))
			this.buffer = this.buffer.slice(boundary + 2)
			boundary = this.buffer.indexOf('\n\n')
		}
		if (final && this.buffer.trim()) { this.dispatchEventBlock(this.buffer); this.buffer = '' }
	}

	private dispatchEventBlock(block: string): void {
		let id = '', event = 'message'
		const data: string[] = []
		for (const line of block.split('\n')) {
			if (!line || line.startsWith(':')) continue
			const separator = line.indexOf(':')
			const field = separator === -1 ? line : line.slice(0, separator)
			let value = separator === -1 ? '' : line.slice(separator + 1)
			if (value.startsWith(' ')) value = value.slice(1)
			if (field === 'id') id = value
			else if (field === 'event') event = value || 'message'
			else if (field === 'data') data.push(value)
		}
		if (id) this.lastEventId = id
		if (!id && event === 'message' && data.length === 0) return
		this.reconnectAttempts = 0
		this.onMessage?.({ id, event, data: data.join('\n') })
	}

	private replayUrl(): string {
		if (!this.lastEventId) return this.url
		const parsed = new URL(this.url, globalThis.location?.origin ?? 'http://localhost')
		parsed.searchParams.set('after_position', this.lastEventId)
		return parsed.toString()
	}

	private headers(): Record<string, string> {
		const headers: Record<string, string> = { Accept: 'text/event-stream', 'x-makosh-secret': this.secret }
		if (this.lastEventId) headers['Last-Event-ID'] = this.lastEventId
		return headers
	}

	private scheduleReconnect(error?: unknown): void {
		if (!this.shouldReconnect) return
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			this.reportStatus({ transport: 'sse', state: 'disconnected', error: errorMessage(error) })
			return
		}
		this.reconnectAttempts += 1
		this.reportStatus({ transport: 'sse', state: 'reconnecting', attempt: this.reconnectAttempts, maxAttempts: this.maxReconnectAttempts, error: errorMessage(error) })
		this.reconnectTimer = setTimeout(() => void this.connectOnce(), this.reconnectDelay * Math.min(this.reconnectAttempts, 5))
	}

	private stopCurrentConnection(): void {
		if (this.reconnectTimer) { clearTimeout(this.reconnectTimer); this.reconnectTimer = null }
		this.abortController?.abort(); this.abortController = null
	}

	private reportStatus(status: SseStatusEvent): void { this.onStatus?.(status) }
}

function errorMessage(error: unknown): string | undefined {
	if (!error) return undefined
	return error instanceof Error ? error.message : typeof error === 'string' ? error : 'Unknown SSE error'
}
