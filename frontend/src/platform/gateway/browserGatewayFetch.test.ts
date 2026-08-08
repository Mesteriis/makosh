import { describe, expect, it, vi } from 'vitest'

import { BrowserGatewayFetch } from './browserGatewayFetch'

describe('BrowserGatewayFetch', () => {
	it('uses only relative same-origin paths and the HttpOnly cookie boundary', async () => {
		const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(new Response(null, { status: 204 }))
		const client = new BrowserGatewayFetch({ origin: 'https://hub.local', fetchImpl })

		await client.fetch('/gateway/v1/status?verbose=0', { method: 'GET' })

		expect(fetchImpl).toHaveBeenCalledWith('/gateway/v1/status?verbose=0', expect.objectContaining({
			cache: 'no-store',
			credentials: 'same-origin',
			mode: 'same-origin',
			redirect: 'error',
		}))
		expect(new Headers(fetchImpl.mock.calls[0]?.[1]?.headers).has('X-Макошь-Secret')).toBe(false)
	})

	it('rejects cross-origin paths and legacy client credentials before fetch', () => {
		const fetchImpl = vi.fn<typeof fetch>()
		const client = new BrowserGatewayFetch({ origin: 'https://hub.local', fetchImpl })

		expect(() => client.fetch('https://other.local/gateway/v1/status')).toThrow('same-origin')
		expect(() => client.fetch('/gateway/v1/status', {
			headers: { 'X-Макошь-Secret': 'legacy-secret' },
		})).toThrow('legacy credential')
		expect(() => client.fetch('/gateway/v1/status', {
			headers: { Authorization: 'Bearer legacy-token' },
		})).toThrow('legacy credential')
		expect(() => client.fetch(new Request('https://hub.local/gateway/v1/status', {
			headers: { 'X-Макошь-Secret': 'legacy-secret' },
		}))).toThrow('legacy credential')
		expect(fetchImpl).not.toHaveBeenCalled()
	})

	it('allows HTTPS plus explicit loopback and private-LAN development origins', () => {
		expect(() => new BrowserGatewayFetch({ origin: 'http://hub.local' })).toThrow('origin is invalid')
		expect(() => new BrowserGatewayFetch({ origin: 'http://203.0.113.10:9444' })).toThrow('origin is invalid')
		expect(() => new BrowserGatewayFetch({ origin: 'https://hub.local/gateway' })).toThrow('origin is invalid')
		expect(() => new BrowserGatewayFetch({ origin: 'http://localhost:5173' })).not.toThrow()
		expect(() => new BrowserGatewayFetch({ origin: 'http://192.168.0.39:9444' })).not.toThrow()
		expect(() => new BrowserGatewayFetch({ origin: 'http://10.20.30.40:9444' })).not.toThrow()
		expect(() => new BrowserGatewayFetch({ origin: 'http://172.31.0.10:9444' })).not.toThrow()
		expect(() => new BrowserGatewayFetch({ origin: 'http://[fd00::39]:9444' })).not.toThrow()
	})
})
