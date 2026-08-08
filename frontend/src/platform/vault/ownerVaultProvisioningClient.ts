import { create } from '@bufbuild/protobuf'
import { createClient, type Client } from '@connectrpc/connect'

import {
	AuthorizeOwnerVaultProvisioningRequestV1Schema,
	CommitOwnerVaultProvisioningRequestV1Schema,
	OwnerVaultActionV1,
	OwnerVaultProvisioningService,
	OwnerVaultSecretClassV1,
	PrepareOwnerVaultProvisioningRequestV1Schema,
} from '../../gen/makosh/gateway/v1/owner_vault_provisioning_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'
import type { OwnerDeviceProofV1 } from '../gateway/ownerDeviceProof'
import { createOwnerDeviceProofV1 } from '../gateway/ownerDeviceProofFactory'
import {
	isOwnerOperationIdV1,
	resolveOwnerOperationIdV1,
} from '../gateway/ownerOperationId'
import {
	type OwnerVaultProvisioningHostV1,
	type SanitizedProvisioningHostReceiptV1,
	type SealedProvisioningHostCommandV1,
	type SealProvisioningHostInputV1,
} from './ownerVaultProvisioningHost'
import { createOwnerVaultProvisioningHostV1 } from './ownerVaultProvisioningHostFactory'

export type OwnerVaultProvisioningCeremonyInputV1 = {
	operationId?: Uint8Array
	targetRegistrationId: string
	capabilityId: string
	configurationInstanceId: string
	purposeId: string
	secretClass: OwnerVaultSecretClassV1
	action: OwnerVaultActionV1
	secretRevision: bigint
}

export type OwnerVaultProvisioningInputV1 = OwnerVaultProvisioningCeremonyInputV1 & {
	secretPayload: Uint8Array
}

export type OwnerVaultCustodiedSealerV1 = (
	input: Omit<SealProvisioningHostInputV1, 'secretPayload'>,
) => Promise<SealedProvisioningHostCommandV1>

export class OwnerVaultProvisioningClientV1 {
	constructor(
		private readonly client: Client<typeof OwnerVaultProvisioningService> = createClient(
			OwnerVaultProvisioningService,
			createBrowserGatewayConnectTransport(),
		),
		private readonly host: OwnerVaultProvisioningHostV1 =
			createOwnerVaultProvisioningHostV1(),
		private readonly deviceProof: OwnerDeviceProofV1 =
			createOwnerDeviceProofV1(),
	) {}

	async provision(input: OwnerVaultProvisioningInputV1): Promise<SanitizedProvisioningHostReceiptV1> {
		validateCeremonyInput(input)
		if (input.secretPayload.byteLength === 0 || input.secretPayload.byteLength > 65_536) {
			throw new Error('owner Vault provisioning input is invalid')
		}
		try {
			return await this.runCeremony(input, (authorized) => this.host.seal({
				...authorized,
				secretPayload: input.secretPayload,
			}))
		} finally {
			input.secretPayload.fill(0)
		}
	}

	async provisionCustodied(
		input: OwnerVaultProvisioningCeremonyInputV1,
		seal: OwnerVaultCustodiedSealerV1,
	): Promise<SanitizedProvisioningHostReceiptV1> {
		validateCeremonyInput(input)
		return this.runCeremony(input, seal)
	}

	private async runCeremony(
		input: OwnerVaultProvisioningCeremonyInputV1,
		seal: OwnerVaultCustodiedSealerV1,
	): Promise<SanitizedProvisioningHostReceiptV1> {
		const operationId = resolveOwnerOperationIdV1(input.operationId)
		const started = await this.host.start()
		let completed = false
		try {
			const prepared = await this.client.prepare(create(
				PrepareOwnerVaultProvisioningRequestV1Schema,
				{
					operationId,
					targetRegistrationId: input.targetRegistrationId,
					capabilityId: input.capabilityId,
					configurationInstanceId: input.configurationInstanceId,
					purposeId: input.purposeId,
					secretClass: input.secretClass,
					action: input.action,
					secretRevision: input.secretRevision,
					responseRecipientHpkePublicKeyX25519:
						started.responseRecipientHpkePublicKeyX25519,
				},
			))
			requireBytes(prepared.challengeBytes, 32)
			const signature = await this.deviceProof.sign(prepared.challengeBytes)
			const authorized = await this.client.authorize(create(
				AuthorizeOwnerVaultProvisioningRequestV1Schema,
				{
					challengeId: prepared.challengeId,
					deviceSignatureRaw: signature,
				},
			))
			const sealed = await seal({
				hostSessionId: started.hostSessionId,
				operationId,
				action: input.action,
				secretClass: input.secretClass,
				authorized: {
					vaultRuntimeGeneration: authorized.vaultRuntimeGeneration,
					vaultHpkePublicKeyX25519: authorized.vaultHpkePublicKeyX25519,
					audienceRegistrationId: authorized.audienceRegistrationId,
					audienceRuntimeInstanceId: authorized.audienceRuntimeInstanceId,
					audienceRuntimeGeneration: authorized.audienceRuntimeGeneration,
					audienceGrantEpoch: authorized.audienceGrantEpoch,
					leaseRequestId: authorized.leaseRequestId,
					leaseOperationDigestSha256: authorized.leaseOperationDigestSha256,
					commandRequestId: authorized.commandRequestId,
					leaseResponseHpkeEncappedKey: authorized.leaseResponseHpkeEncappedKey,
					leaseResponseCiphertext: authorized.leaseResponseCiphertext,
					leaseResponseHpkeAuthenticationTag:
						authorized.leaseResponseHpkeAuthenticationTag,
				},
			})
			const committed = await this.client.commit(create(
				CommitOwnerVaultProvisioningRequestV1Schema,
				{
					provisioningSessionId: authorized.provisioningSessionId,
					operationDigestSha256: sealed.operationDigestSha256,
					hpkeEncappedKey: sealed.hpkeEncappedKey,
					ciphertext: sealed.ciphertext,
					hpkeAuthenticationTag: sealed.hpkeAuthenticationTag,
				},
			))
			const receipt = await this.host.openReceipt(started.hostSessionId, {
				vaultRuntimeGeneration: committed.vaultRuntimeGeneration,
				commandRequestId: committed.commandRequestId,
				operationDigestSha256: committed.operationDigestSha256,
				receiptHpkeEncappedKey: committed.receiptHpkeEncappedKey,
				receiptCiphertext: committed.receiptCiphertext,
				receiptHpkeAuthenticationTag: committed.receiptHpkeAuthenticationTag,
			})
			completed = true
			return receipt
		} finally {
			if (!completed) await this.host.cancel(started.hostSessionId).catch(() => undefined)
		}
	}
}

function validateCeremonyInput(input: OwnerVaultProvisioningCeremonyInputV1): void {
	const identifiers = [
		input.targetRegistrationId,
		input.capabilityId,
		input.configurationInstanceId,
		input.purposeId,
	]
	if (identifiers.some((value) => value.trim().length === 0 || value.length > 128)
		|| input.secretRevision <= 0n
		|| !isSecretClass(input.secretClass)
		|| !isAction(input.action)
		|| (input.operationId !== undefined && !isOwnerOperationIdV1(input.operationId))) {
		throw new Error('owner Vault provisioning input is invalid')
	}
}

function requireBytes(value: Uint8Array, length: number): void {
	if (value.byteLength !== length) throw new Error('owner Vault provisioning response is invalid')
}

function isSecretClass(value: OwnerVaultSecretClassV1): boolean {
	return value === OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL
		|| value === OwnerVaultSecretClassV1.OAUTH_REFRESH_CREDENTIAL
		|| value === OwnerVaultSecretClassV1.SESSION_CREDENTIAL_BLOB
		|| value === OwnerVaultSecretClassV1.SESSION_STORE_KEY
}

function isAction(value: OwnerVaultActionV1): boolean {
	return value === OwnerVaultActionV1.CREATE
		|| value === OwnerVaultActionV1.REPLACE_CAS
		|| value === OwnerVaultActionV1.RETIRE
		|| value === OwnerVaultActionV1.DELETE
}
