import { describe, expect, it, vi } from 'vitest'

import { DevelopmentOwnerDeviceProofV1 } from './developmentOwnerDeviceProof'

describe('DevelopmentOwnerDeviceProofV1', () => {
		it('signs only an exact challenge through the protected same-origin host route', async () => {
			const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(new Response(
				JSON.stringify({
					signatureRaw: Array.from({ length: 64 }, () => 7),
				}),
			{
				status: 200,
				headers: { 'content-type': 'application/json' },
			},
		))
		const proof = new DevelopmentOwnerDeviceProofV1(fetchImpl)

		await expect(proof.sign(new Uint8Array(32).fill(3))).resolves.toEqual(
			new Uint8Array(64).fill(7),
		)
		expect(fetchImpl).toHaveBeenCalledWith(
			'/__makosh/owner-device-proof/v1/sign',
			expect.objectContaining({
				method: 'POST',
				credentials: 'same-origin',
				cache: 'no-store',
				redirect: 'error',
			}),
		)
	})

	it('rejects wrong challenge and response widths', async () => {
		const proof = new DevelopmentOwnerDeviceProofV1(vi.fn<typeof fetch>())
		await expect(proof.sign(new Uint8Array(31))).rejects.toThrow(
			'owner device challenge is invalid',
		)

		const malformed = new DevelopmentOwnerDeviceProofV1(
			vi.fn<typeof fetch>().mockResolvedValue(new Response(
				JSON.stringify({ signatureRaw: [1, 2] }),
				{ status: 200 },
			)),
		)
		await expect(malformed.sign(new Uint8Array(32))).rejects.toThrow(
			'owner device proof host response is invalid',
		)
	})
})
