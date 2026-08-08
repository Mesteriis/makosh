import { create } from '@bufbuild/protobuf'
import type { Client } from '@connectrpc/connect'
import { describe, expect, it, vi } from 'vitest'

import {
	AuthorizeOwnerVaultProvisioningResponseV1Schema,
	CommitOwnerVaultProvisioningResponseV1Schema,
	OwnerVaultActionV1,
	OwnerVaultProvisioningService,
	OwnerVaultSecretClassV1,
	PrepareOwnerVaultProvisioningResponseV1Schema,
} from '../../gen/makosh/gateway/v1/owner_vault_provisioning_pb'
import type { OwnerDeviceProofV1 } from '../gateway/ownerDeviceProof'
import {
	OwnerVaultProvisioningClientV1,
} from './ownerVaultProvisioningClient'
import type {
	OwnerVaultProvisioningHostV1,
	SealProvisioningHostInputV1,
} from './ownerVaultProvisioningHost'

describe('OwnerVaultProvisioningClientV1', () => {
	it('accepts the Telegram session-store key class and clears secret bytes', async () => {
		const secret = Uint8Array.from([112, 97, 115, 115])
		let sealedSecret: number[] = []
		const host: OwnerVaultProvisioningHostV1 = {
			start: vi.fn().mockResolvedValue({
				hostSessionId: 'host-session',
				responseRecipientHpkePublicKeyX25519: new Uint8Array(32).fill(9),
			}),
			seal: vi.fn().mockImplementation(async (input: SealProvisioningHostInputV1) => {
				sealedSecret = Array.from(input.secretPayload)
				return {
					operationDigestSha256: new Uint8Array(32).fill(10),
					hpkeEncappedKey: new Uint8Array(32).fill(11),
					ciphertext: new Uint8Array([12]),
					hpkeAuthenticationTag: new Uint8Array(16).fill(13),
				}
			}),
			openReceipt: vi.fn().mockResolvedValue({
				operationId: new Uint8Array(16).fill(1),
				action: OwnerVaultActionV1.CREATE,
				secretRevision: 1n,
				state: 1,
			}),
			cancel: vi.fn().mockResolvedValue(undefined),
		}
		const deviceProof: OwnerDeviceProofV1 = {
			sign: vi.fn().mockResolvedValue(new Uint8Array(64).fill(8)),
		}
		const gateway = gatewayClient()
		const client = new OwnerVaultProvisioningClientV1(gateway.client, host, deviceProof)

		const receipt = await client.provision({
			operationId: new Uint8Array(16).fill(1),
			targetRegistrationId: 'telegram-registration',
			capabilityId: 'telegram.tdlib.credential-provisioning.v1',
			configurationInstanceId: 'telegram-account',
			purposeId: 'telegram_tdlib_session_store_key',
			secretClass: OwnerVaultSecretClassV1.SESSION_STORE_KEY,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
			secretPayload: secret,
		})

		expect(receipt.secretRevision).toBe(1n)
		expect(sealedSecret).toEqual([112, 97, 115, 115])
		expect(secret).toEqual(new Uint8Array(4))
		expect(gateway.prepare.mock.calls[0]?.[0]).toMatchObject({
			targetRegistrationId: 'telegram-registration',
			capabilityId: 'telegram.tdlib.credential-provisioning.v1',
			purposeId: 'telegram_tdlib_session_store_key',
			secretClass: OwnerVaultSecretClassV1.SESSION_STORE_KEY,
		})
		expect(gateway.authorize).toHaveBeenCalledOnce()
		expect(gateway.commit).toHaveBeenCalledOnce()
		expect(host.cancel).not.toHaveBeenCalled()
	})

	it('cancels local host state and clears secret bytes after a rejected prepare', async () => {
		const secret = Uint8Array.from([7, 8])
		const host: OwnerVaultProvisioningHostV1 = {
			start: vi.fn().mockResolvedValue({
				hostSessionId: 'host-session',
				responseRecipientHpkePublicKeyX25519: new Uint8Array(32).fill(9),
			}),
			seal: vi.fn(),
			openReceipt: vi.fn(),
			cancel: vi.fn().mockResolvedValue(undefined),
		}
		const client = {
			prepare: vi.fn().mockRejectedValue(new Error('denied')),
		} as unknown as Client<typeof OwnerVaultProvisioningService>

		await expect(new OwnerVaultProvisioningClientV1(client, host, {
			sign: vi.fn(),
		}).provision({
			targetRegistrationId: 'mail-registration',
			capabilityId: 'mail.imap.credential-provisioning.v1',
			configurationInstanceId: 'mail-account',
			purposeId: 'mail_imap_password',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
			secretPayload: secret,
		})).rejects.toThrow('denied')

		expect(secret).toEqual(new Uint8Array(2))
		expect(host.cancel).toHaveBeenCalledWith('host-session')
	})

	it('uses a native-custodied sealer without a browser secret payload', async () => {
		const host = provisioningHost()
		const gateway = gatewayClient()
		const seal = vi.fn().mockResolvedValue({
			operationDigestSha256: new Uint8Array(32).fill(10),
			hpkeEncappedKey: new Uint8Array(32).fill(11),
			ciphertext: new Uint8Array([12]),
			hpkeAuthenticationTag: new Uint8Array(16).fill(13),
		})
		const client = new OwnerVaultProvisioningClientV1(gateway.client, host, {
			sign: vi.fn().mockResolvedValue(new Uint8Array(64).fill(8)),
		})

		await client.provisionCustodied({
			operationId: new Uint8Array(16).fill(2),
			targetRegistrationId: 'mail-registration',
			capabilityId: 'mail.imap.credential-provisioning.v1',
			configurationInstanceId: 'mail-account',
			purposeId: 'mail_imap_password',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
		}, seal)

		expect(seal).toHaveBeenCalledOnce()
		expect(seal.mock.calls[0]?.[0]).not.toHaveProperty('secretPayload')
		expect(host.seal).not.toHaveBeenCalled()
		expect(host.cancel).not.toHaveBeenCalled()
	})

	it('cancels the Vault host if native custody sealing is rejected', async () => {
		const host = provisioningHost()
		const gateway = gatewayClient()
		const client = new OwnerVaultProvisioningClientV1(gateway.client, host, {
			sign: vi.fn().mockResolvedValue(new Uint8Array(64).fill(8)),
		})

		await expect(client.provisionCustodied({
			targetRegistrationId: 'telegram-registration',
			capabilityId: 'telegram.api-hash.credential-provisioning.v1',
			configurationInstanceId: 'telegram-account',
			purposeId: 'telegram_api_hash',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
		}, vi.fn().mockRejectedValue(new Error('source unavailable')))).rejects.toThrow(
			'source unavailable',
		)

		expect(host.cancel).toHaveBeenCalledWith('host-session')
	})
})

function provisioningHost(): OwnerVaultProvisioningHostV1 {
	return {
		start: vi.fn().mockResolvedValue({
			hostSessionId: 'host-session',
			responseRecipientHpkePublicKeyX25519: new Uint8Array(32).fill(9),
		}),
		seal: vi.fn(),
		openReceipt: vi.fn().mockResolvedValue({
			operationId: new Uint8Array(16).fill(1),
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
			state: 1,
		}),
		cancel: vi.fn().mockResolvedValue(undefined),
	}
}

function gatewayClient() {
	const prepare = vi.fn().mockResolvedValue(create(
		PrepareOwnerVaultProvisioningResponseV1Schema,
		{
			major: 1,
			challengeId: 'challenge',
			challengeBytes: new Uint8Array(32).fill(7),
			expiresAtUnixMillis: 1n,
		},
	))
	const authorize = vi.fn().mockResolvedValue(create(
		AuthorizeOwnerVaultProvisioningResponseV1Schema,
		{
			major: 1,
			provisioningSessionId: 'provisioning-session',
			expiresAtUnixMillis: 2n,
			vaultRuntimeGeneration: 3n,
			vaultHpkePublicKeyX25519: new Uint8Array(32).fill(4),
			audienceRegistrationId: 'mail-registration',
			audienceRuntimeInstanceId: 'owner-vault-runtime',
			audienceRuntimeGeneration: 5n,
			audienceGrantEpoch: 6n,
			leaseRequestId: new Uint8Array(16).fill(7),
			leaseOperationDigestSha256: new Uint8Array(32).fill(8),
			commandRequestId: new Uint8Array(16).fill(9),
			leaseResponseHpkeEncappedKey: new Uint8Array(32).fill(10),
			leaseResponseCiphertext: new Uint8Array([11]),
			leaseResponseHpkeAuthenticationTag: new Uint8Array(16).fill(12),
		},
	))
	const commit = vi.fn().mockResolvedValue(create(
		CommitOwnerVaultProvisioningResponseV1Schema,
		{
			major: 1,
			vaultRuntimeGeneration: 3n,
			commandRequestId: new Uint8Array(16).fill(9),
			operationDigestSha256: new Uint8Array(32).fill(10),
			receiptHpkeEncappedKey: new Uint8Array(32).fill(11),
			receiptCiphertext: new Uint8Array([12]),
			receiptHpkeAuthenticationTag: new Uint8Array(16).fill(13),
		},
	))
	return {
		prepare,
		authorize,
		commit,
		client: { prepare, authorize, commit } as unknown as Client<
			typeof OwnerVaultProvisioningService
		>,
	}
}
