import { describe, expect, it, vi } from 'vitest'

import { DevelopmentLegacyProviderRecoveryHostV1 } from './developmentLegacyProviderRecoveryHost'

describe('DevelopmentLegacyProviderRecoveryHostV1', () => {
	it('accepts only the exact sanitized three-candidate plan', async () => {
		const fetchImpl = vi.fn().mockResolvedValue(response({
			schemaRevision: 1,
			recoverySessionId: 'a'.repeat(32),
			bundleFingerprintSha256: 'b'.repeat(64),
			counts: {
				gmailActive: 1,
				icloudActive: 1,
				telegramUserActive: 1,
				gmailDeleted: 2,
			},
			candidates: [
				{ sourceHandle: '1'.repeat(64), kind: 'gmail', state: 'reauthorization_required' },
				{ sourceHandle: '2'.repeat(64), kind: 'icloud', state: 'ready_to_apply' },
				{ sourceHandle: '3'.repeat(64), kind: 'telegram_user', state: 'qr_authorization_required' },
			],
		}))

		const plan = await new DevelopmentLegacyProviderRecoveryHostV1(
			fetchImpl as typeof fetch,
		).start()

		expect(plan.counts).toEqual({
			gmailActive: 1,
			icloudActive: 1,
			telegramUserActive: 1,
			gmailDeleted: 2,
		})
		expect(fetchImpl).toHaveBeenCalledWith(
			'/__makosh/legacy-provider-recovery/v1/start',
			expect.objectContaining({
				credentials: 'same-origin',
				cache: 'no-store',
				redirect: 'error',
			}),
		)
	})

	it('seals an opaque source handle without a secret payload field', async () => {
		const fetchImpl = vi.fn().mockResolvedValue(response({
			operationDigestSha256: Array.from({ length: 32 }, () => 1),
			hpkeEncappedKey: Array.from({ length: 32 }, () => 2),
			ciphertext: [3],
			hpkeAuthenticationTag: Array.from({ length: 16 }, () => 4),
		}))

		await new DevelopmentLegacyProviderRecoveryHostV1(
			fetchImpl as typeof fetch,
		).sealSource({
			recoverySessionId: 'a'.repeat(32),
			sourceHandle: 'b'.repeat(64),
			secretPurpose: 'icloud_imap_password',
			hostSessionId: 'host-session',
			operationId: new Uint8Array(16).fill(5),
			action: 1,
			secretClass: 1,
			authorized: {
				vaultRuntimeGeneration: 1n,
				vaultHpkePublicKeyX25519: new Uint8Array(32).fill(6),
				audienceRegistrationId: 'mail-registration',
				audienceRuntimeInstanceId: 'vault-runtime',
				audienceRuntimeGeneration: 2n,
				audienceGrantEpoch: 3n,
				leaseRequestId: new Uint8Array(16).fill(7),
				leaseOperationDigestSha256: new Uint8Array(32).fill(8),
				commandRequestId: new Uint8Array(16).fill(9),
				leaseResponseHpkeEncappedKey: new Uint8Array(32).fill(10),
				leaseResponseCiphertext: new Uint8Array([11]),
				leaseResponseHpkeAuthenticationTag: new Uint8Array(16).fill(12),
			},
		})

		const request = JSON.parse(fetchImpl.mock.calls[0]?.[1].body as string)
		expect(request.secretPurpose).toBe('icloud_imap_password')
		expect(request.sourceHandle).toBe('b'.repeat(64))
		expect(request).not.toHaveProperty('secretPayload')
	})

	it('accepts only exact persisted step dispositions and unsigned public revisions', async () => {
		const fetchImpl = vi.fn().mockResolvedValue(response({
			disposition: 'completed',
			operationId: Array.from({ length: 16 }, () => 5),
			targetConfigurationInstanceId: 'mail-target',
			publicRevision: '7',
		}))

		const step = await new DevelopmentLegacyProviderRecoveryHostV1(
			fetchImpl as typeof fetch,
		).beginStep({
			recoverySessionId: 'a'.repeat(32),
			sourceHandle: 'b'.repeat(64),
			stepIdentifier: 'mail_icloud_create_target',
			explicitRetry: false,
		})

		expect(step).toEqual({
			disposition: 'completed',
			operationId: new Uint8Array(16).fill(5),
			targetConfigurationInstanceId: 'mail-target',
			publicRevision: 7n,
		})
	})
})

function response(value: unknown): Response {
	return {
		ok: true,
		json: vi.fn().mockResolvedValue(value),
	} as unknown as Response
}
