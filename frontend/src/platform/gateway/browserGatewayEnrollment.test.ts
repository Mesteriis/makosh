import { afterEach, describe, expect, it, vi } from 'vitest'

vi.mock('./browserLocalDeviceKey', () => ({
	createBrowserLocalDeviceKey: vi.fn().mockResolvedValue({ publicKey: new Uint8Array([4, ...new Uint8Array(64)]), privateKey: {} }),
	saveBrowserLocalDeviceKey: vi.fn().mockResolvedValue(undefined),
	deleteBrowserLocalDeviceKey: vi.fn().mockResolvedValue(undefined),
}))

import { BrowserGatewayEnrollment } from './browserGatewayEnrollment'

describe('BrowserGatewayEnrollment', () => {
	afterEach(() => vi.unstubAllGlobals())

	it('accepts the generated protobuf JSON registration envelope', async () => {
		const credential = {
			rawId: Uint8Array.of(1).buffer,
			response: { attestationObject: Uint8Array.of(2).buffer, clientDataJSON: Uint8Array.of(3).buffer },
			type: 'public-key',
		}
		const credentialCreate = vi.fn().mockResolvedValue(credential)
		const fetchImpl = vi.fn<typeof fetch>()
			.mockResolvedValueOnce(new Response(JSON.stringify({
				public_key: { publicKey: {
					rp: { name: 'localhost', id: 'localhost' },
					user: { id: 'b3duZXI', name: 'owner', displayName: 'Макошь owner' },
					challenge: 'AQ', pubKeyCredParams: [{ type: 'public-key', alg: -7 }],
				} },
			}), { status: 200 }))
			.mockResolvedValueOnce(new Response('{"enrolled":true}', { status: 200 }))
		vi.stubGlobal('fetch', fetchImpl)
		vi.stubGlobal('location', { origin: 'https://localhost:9443' })
		vi.stubGlobal('navigator', { credentials: { create: credentialCreate } })
		vi.stubGlobal('PublicKeyCredential', class {})
		Object.setPrototypeOf(credential, globalThis.PublicKeyCredential.prototype)

		await expect(new BrowserGatewayEnrollment().enroll('a'.repeat(64))).resolves.toBe('AQ')
		expect(credentialCreate).toHaveBeenCalledOnce()
		expect(fetchImpl).toHaveBeenNthCalledWith(2,
			`/browser/v1/pairing/${'a'.repeat(64)}/registration/finish`,
			expect.objectContaining({ method: 'POST' }),
		)
		expect(fetchImpl.mock.calls[1]?.[1]?.body).toContain('browser_key_public_key')
	})
})
