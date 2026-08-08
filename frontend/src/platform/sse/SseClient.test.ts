import { afterEach, describe, expect, it, vi } from 'vitest'
import { SseClient } from './SseClient'

describe('SseClient', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it('reports protected SSE transport status transitions', async () => {
    vi.useFakeTimers()
    const statuses: string[] = []
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 42\nevent: event\ndata: {"ok":true}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(stream, {
        status: 200,
        headers: { 'Content-Type': 'text/event-stream' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<void>((resolve) => {
      const client = new SseClient({
		url: 'http://127.0.0.1:8080/api/realtime/v2/events',
        secret: 'test-secret',
        reconnectDelay: 60_000,
        onStatus: (status) => statuses.push(`${status.transport}:${status.state}`),
        onMessage: () => {
          client.disconnect()
          resolve()
        }
      })
      client.connect()
    })

    await received

    expect(statuses).toEqual(['sse:connecting', 'sse:connected', 'sse:disconnected'])
  })

  it('connects with the local API secret and replays from the last event id', async () => {
    vi.useFakeTimers()
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 42\nevent: event\ndata: {"ok":true}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(stream, {
        status: 200,
        headers: { 'Content-Type': 'text/event-stream' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<{ id: string; event: string; data: string }>((resolve) => {
      const client = new SseClient({
		url: 'http://127.0.0.1:8080/api/realtime/v2/events',
        secret: 'test-secret',
        lastEventId: '41',
        reconnectDelay: 60_000,
        onMessage: (event) => {
          resolve(event)
          client.disconnect()
        }
      })
      client.connect()
    })

    await expect(received).resolves.toEqual({
      id: '42',
      event: 'event',
      data: '{"ok":true}'
    })
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
	expect(url).toBe('http://127.0.0.1:8080/api/realtime/v2/events?after_position=41')
    expect(init.headers).toMatchObject({
      Accept: 'text/event-stream',
      'X-Макошь-Secret': 'test-secret',
      'Last-Event-ID': '41'
    })
  })

  it('builds replay requests for relative stream URLs', async () => {
    vi.useFakeTimers()
    vi.stubGlobal('location', { origin: 'http://127.0.0.1:5174' })
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 10\nevent: event\ndata: {}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(new Response(stream, { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<void>((resolve) => {
      const client = new SseClient({
		url: '/api/realtime/v2/events?source=frontend',
        secret: 'test-secret',
        lastEventId: '9',
        reconnectDelay: 60_000,
        onMessage: () => {
          client.disconnect()
          resolve()
        }
      })
      client.connect()
    })

    await received
    const [url] = fetchMock.mock.calls[0]
	expect(url).toBe('http://127.0.0.1:5174/api/realtime/v2/events?source=frontend&after_position=9')
  })

	it('reports SSE disconnection when its retry budget is exhausted', async () => {
    vi.useFakeTimers()
    const statuses: string[] = []
    const fetchMock = vi
      .fn()
		.mockResolvedValueOnce(new Response('stream unavailable', { status: 503 }))
    vi.stubGlobal('fetch', fetchMock)

	const client = new SseClient({
		url: 'http://127.0.0.1:8080/api/realtime/v2/events',
        secret: 'test-secret',
        lastEventId: '41',
        maxReconnectAttempts: 0,
		reconnectDelay: 60_000,
        onStatus: (status) => statuses.push(`${status.transport}:${status.state}`),
	})
	client.connect()
	await vi.advanceTimersByTimeAsync(0)

	expect(fetchMock).toHaveBeenCalledOnce()
	expect(statuses).toContain('sse:disconnected')
  })
})
