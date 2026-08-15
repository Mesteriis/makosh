import { describe, expect, it, vi } from 'vitest'

import { DevelopmentOwnerVaultProvisioningHostV1 } from './developmentOwnerVaultProvisioningHost'

describe('DevelopmentOwnerVaultProvisioningHostV1', () => {
	it('maps the complete loopback host lifecycle without losing u64 precision', async () => {
		const requests: Array<{ path: string; body: Record<string, unknown> }> = []
		const fetchImpl = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
			const path = String(input)
			const body = JSON.parse(String(init?.body)) as Record<string, unknown>
			requests.push({ path, body })
			if (path.endsWith('/start')) {
				return Response.json({
					hostSessionId: 'host-session',
					responseRecipientHpkePublicKeyX25519: bytes(32, 1),
				})
			}
			if (path.endsWith('/seal')) {
				return Response.json({
					operationDigestSha256: bytes(32, 2),
					hpkeEncappedKey: bytes(32, 3),
					ciphertext: [4],
					hpkeAuthenticationTag: bytes(16, 5),
				})
			}
			if (path.endsWith('/telegram-credentials')) {
				return Response.json({ apiId: '9007199254740993' })
			}
			if (path.endsWith('/seal-telegram-api-hash')) {
				return Response.json({
					operationDigestSha256: bytes(32, 21),
					hpkeEncappedKey: bytes(32, 22),
					ciphertext: [23],
					hpkeAuthenticationTag: bytes(16, 24),
				})
			}
			if (path.endsWith('/open-receipt')) {
				return Response.json({
					operationId: bytes(16, 6),
					action: 1,
					secretRevision: '9007199254740993',
					state: 1,
				})
			}
			if (path.endsWith('/cancel')) return Response.json({})
			return Response.json({ code: 'not_found' }, { status: 404 })
		})
		const host = new DevelopmentOwnerVaultProvisioningHostV1(
			fetchImpl as typeof fetch,
		)

		const started = await host.start()
		const authorized = {
			vaultRuntimeGeneration: 9_007_199_254_740_993n,
			vaultHpkePublicKeyX25519: new Uint8Array(32).fill(10),
			audienceRegistrationId: 'telegram-registration',
			audienceRuntimeInstanceId: 'telegram-runtime',
			audienceRuntimeGeneration: 2n,
			audienceGrantEpoch: 3n,
			leaseRequestId: new Uint8Array(16).fill(11),
			leaseOperationDigestSha256: new Uint8Array(32).fill(12),
			commandRequestId: new Uint8Array(16).fill(13),
			leaseResponseHpkeEncappedKey: new Uint8Array(32).fill(14),
			leaseResponseCiphertext: new Uint8Array([15]),
			leaseResponseHpkeAuthenticationTag: new Uint8Array(16).fill(16),
		}
		const sealed = await host.seal({
			hostSessionId: started.hostSessionId,
			operationId: new Uint8Array(16).fill(7),
			action: 1,
			secretClass: 4,
			secretPayload: Uint8Array.from([8, 9]),
			authorized,
		})
		const telegramCredentials = await host.telegramCredentials()
		await host.sealTelegramApiHash({
			hostSessionId: started.hostSessionId,
			operationId: new Uint8Array(16).fill(25),
			action: 1,
			secretClass: 4,
			authorized,
		})
		const receipt = await host.openReceipt(started.hostSessionId, {
			vaultRuntimeGeneration: 9_007_199_254_740_993n,
			commandRequestId: new Uint8Array(16).fill(17),
			operationDigestSha256: sealed.operationDigestSha256,
			receiptHpkeEncappedKey: new Uint8Array(32).fill(18),
			receiptCiphertext: new Uint8Array([19]),
			receiptHpkeAuthenticationTag: new Uint8Array(16).fill(20),
		})
		await host.cancel(started.hostSessionId)

		expect(receipt.secretRevision).toBe(9_007_199_254_740_993n)
		expect(telegramCredentials.apiId).toBe(9_007_199_254_740_993n)
		expect(requests.map(({ path }) => path)).toEqual([
			'/__makosh/owner-vault-host/v1/start',
			'/__makosh/owner-vault-host/v1/seal',
			'/__makosh/owner-vault-host/v1/telegram-credentials',
			'/__makosh/owner-vault-host/v1/seal-telegram-api-hash',
			'/__makosh/owner-vault-host/v1/open-receipt',
			'/__makosh/owner-vault-host/v1/cancel',
		])
		expect(requests[1]?.body).toMatchObject({
			secretClass: 4,
			secretPayload: [8, 9],
			authorized: {
				vaultRuntimeGeneration: '9007199254740993',
				audienceRuntimeGeneration: '2',
				audienceGrantEpoch: '3',
			},
		})
		expect(requests[3]?.body).toMatchObject({
			secretPurpose: 'telegram_api_hash',
			authorized: { vaultRuntimeGeneration: '9007199254740993' },
		})
		expect(requests[3]?.body).not.toHaveProperty('secretPayload')
		expect(requests[4]?.body).toMatchObject({
			committed: { vaultRuntimeGeneration: '9007199254740993' },
		})
		for (const call of fetchImpl.mock.calls) {
			expect(call[1]).toMatchObject({
				method: 'POST',
				credentials: 'same-origin',
				cache: 'no-store',
				redirect: 'error',
			})
		}
	})

	it('fails closed for rejected or malformed host responses', async () => {
		const rejected = new DevelopmentOwnerVaultProvisioningHostV1(
			vi.fn().mockResolvedValue(Response.json(
				{ code: 'denied' },
				{ status: 403 },
			)) as typeof fetch,
		)
		await expect(rejected.start()).rejects.toThrow(
			'owner Vault development host rejected request (403)',
		)

		const malformed = new DevelopmentOwnerVaultProvisioningHostV1(
			vi.fn().mockResolvedValue(Response.json({
				hostSessionId: 'host-session',
				responseRecipientHpkePublicKeyX25519: [1],
			})) as typeof fetch,
		)
		await expect(malformed.start()).rejects.toThrow(
			'owner Vault development host response is invalid',
		)
	})
})

function bytes(length: number, value: number): number[] {
	return Array.from({ length }, () => value)
}
