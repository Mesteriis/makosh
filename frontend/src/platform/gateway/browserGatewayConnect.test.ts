import type { Transport } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import { describe, expect, it, vi } from 'vitest'

import {
	BROWSER_GATEWAY_REQUEST_TIMEOUT_MS,
	createBrowserGatewayConnectTransport,
} from './browserGatewayConnect'

vi.mock('@connectrpc/connect-web', () => ({
	createConnectTransport: vi.fn(() => ({} as Transport)),
}))

describe('createBrowserGatewayConnectTransport', () => {
	it('pins Connect to the browser origin and preserves the cookie-only boundary', async () => {
		const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(new Response(null, { status: 204 }))

		createBrowserGatewayConnectTransport({
			origin: 'https://hub.local',
			fetchImpl,
			defaultTimeoutMs: 5_000,
		})

		expect(createConnectTransport).toHaveBeenCalledWith(expect.objectContaining({
			baseUrl: '/',
			defaultTimeoutMs: 5_000,
			useBinaryFormat: true,
			useHttpGet: false,
		}))
		const config = vi.mocked(createConnectTransport).mock.calls[0]?.[0]
		if (!config?.fetch) throw new Error('Connect fetch boundary was not configured')

		await config.fetch('https://hub.local/makosh.gateway.v1.SessionService/Status', {
			method: 'POST',
		})

		expect(fetchImpl).toHaveBeenCalledWith(
			'/makosh.gateway.v1.SessionService/Status',
			expect.objectContaining({
				cache: 'no-store',
				credentials: 'same-origin',
				mode: 'same-origin',
				redirect: 'error',
			}),
		)
	})

	it('does not permit a Connect request to leave the exact browser origin', () => {
		const fetchImpl = vi.fn<typeof fetch>()
		createBrowserGatewayConnectTransport({ origin: 'https://hub.local', fetchImpl })
		const config = vi.mocked(createConnectTransport).mock.calls[0]?.[0]
		if (!config?.fetch) throw new Error('Connect fetch boundary was not configured')

		expect(() => config.fetch?.('https://other.local/makosh.gateway.v1.SessionService/Status')).toThrow('same-origin')
		expect(fetchImpl).not.toHaveBeenCalled()
	})

	it('owns the admitted Gateway deadline and rejects incompatible overrides', () => {
		createBrowserGatewayConnectTransport({ origin: 'https://hub.local' })
		expect(createConnectTransport).toHaveBeenLastCalledWith(expect.objectContaining({
			defaultTimeoutMs: BROWSER_GATEWAY_REQUEST_TIMEOUT_MS,
		}))

		expect(() => createBrowserGatewayConnectTransport({
			origin: 'https://hub.local',
			defaultTimeoutMs: BROWSER_GATEWAY_REQUEST_TIMEOUT_MS + 1,
		})).toThrow('exceeds the admitted transport deadline')
	})
})
