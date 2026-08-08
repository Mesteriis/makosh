import type {
	CommittedProvisioningHostInputV1,
	OwnerVaultProvisioningHostV1,
	SanitizedProvisioningHostReceiptV1,
	SealedProvisioningHostCommandV1,
	SealProvisioningHostInputV1,
	StartedProvisioningHostSessionV1,
} from './ownerVaultProvisioningHost'

const DEVELOPMENT_HOST_BASE_PATH = '/__makosh/owner-vault-host/v1'

type HostFetch = typeof fetch

export class DevelopmentOwnerVaultProvisioningHostV1 implements OwnerVaultProvisioningHostV1 {
	constructor(
		private readonly fetchImpl: HostFetch =
			(input, init) => fetch(input, init),
	) {}

	async start(): Promise<StartedProvisioningHostSessionV1> {
		const response = await this.post('/start', {})
		return {
			hostSessionId: requiredString(response.hostSessionId),
			responseRecipientHpkePublicKeyX25519: bytes(
				response.responseRecipientHpkePublicKeyX25519,
				32,
			),
		}
	}

	async seal(input: SealProvisioningHostInputV1): Promise<SealedProvisioningHostCommandV1> {
		const secretPayload = Array.from(input.secretPayload)
		try {
			const response = await this.post('/seal', {
				hostSessionId: input.hostSessionId,
				operationId: Array.from(input.operationId),
				action: input.action,
				secretClass: input.secretClass,
				secretPayload,
				authorized: {
					vaultRuntimeGeneration: input.authorized.vaultRuntimeGeneration.toString(),
					vaultHpkePublicKeyX25519: Array.from(input.authorized.vaultHpkePublicKeyX25519),
					audienceRegistrationId: input.authorized.audienceRegistrationId,
					audienceRuntimeInstanceId: input.authorized.audienceRuntimeInstanceId,
					audienceRuntimeGeneration: input.authorized.audienceRuntimeGeneration.toString(),
					audienceGrantEpoch: input.authorized.audienceGrantEpoch.toString(),
					leaseRequestId: Array.from(input.authorized.leaseRequestId),
					leaseOperationDigestSha256: Array.from(input.authorized.leaseOperationDigestSha256),
					commandRequestId: Array.from(input.authorized.commandRequestId),
					leaseResponseHpkeEncappedKey: Array.from(input.authorized.leaseResponseHpkeEncappedKey),
					leaseResponseCiphertext: Array.from(input.authorized.leaseResponseCiphertext),
					leaseResponseHpkeAuthenticationTag: Array.from(
						input.authorized.leaseResponseHpkeAuthenticationTag,
					),
				},
			})
			return {
				operationDigestSha256: bytes(response.operationDigestSha256, 32),
				hpkeEncappedKey: bytes(response.hpkeEncappedKey, 32),
				ciphertext: bytes(response.ciphertext),
				hpkeAuthenticationTag: bytes(response.hpkeAuthenticationTag, 16),
			}
		} finally {
			secretPayload.fill(0)
		}
	}

	async openReceipt(
		hostSessionId: string,
		committed: CommittedProvisioningHostInputV1,
	): Promise<SanitizedProvisioningHostReceiptV1> {
		const response = await this.post('/open-receipt', {
			hostSessionId,
			committed: {
				vaultRuntimeGeneration: committed.vaultRuntimeGeneration.toString(),
				commandRequestId: Array.from(committed.commandRequestId),
				operationDigestSha256: Array.from(committed.operationDigestSha256),
				receiptHpkeEncappedKey: Array.from(committed.receiptHpkeEncappedKey),
				receiptCiphertext: Array.from(committed.receiptCiphertext),
				receiptHpkeAuthenticationTag: Array.from(committed.receiptHpkeAuthenticationTag),
			},
		})
		return {
			operationId: bytes(response.operationId, 16),
			action: requiredInteger(response.action),
			secretRevision: requiredUnsigned(response.secretRevision),
			state: requiredInteger(response.state),
		}
	}

	async cancel(hostSessionId: string): Promise<void> {
		await this.post('/cancel', { hostSessionId })
	}

	private async post(path: string, body: Record<string, unknown>): Promise<Record<string, unknown>> {
		const response = await this.fetchImpl(`${DEVELOPMENT_HOST_BASE_PATH}${path}`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body),
			credentials: 'same-origin',
			cache: 'no-store',
			redirect: 'error',
		})
		if (!response.ok) {
			throw new Error(`owner Vault development host rejected request (${response.status})`)
		}
		const value: unknown = await response.json()
		if (!isRecord(value)) throw new Error('owner Vault development host response is invalid')
		return value
	}
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function requiredString(value: unknown): string {
	if (typeof value !== 'string' || !value || value.length > 128) {
		throw new Error('owner Vault development host response is invalid')
	}
	return value
}

function requiredInteger(value: unknown): number {
	if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0) {
		throw new Error('owner Vault development host response is invalid')
	}
	return value
}

function requiredUnsigned(value: unknown): bigint {
	if (typeof value !== 'string' || !/^(0|[1-9][0-9]*)$/.test(value)) {
		throw new Error('owner Vault development host response is invalid')
	}
	return BigInt(value)
}

function bytes(value: unknown, exactLength?: number): Uint8Array {
	if (!Array.isArray(value)
		|| value.some((item) => !Number.isInteger(item) || item < 0 || item > 255)
		|| (exactLength !== undefined && value.length !== exactLength)) {
		throw new Error('owner Vault development host response is invalid')
	}
	return Uint8Array.from(value)
}
