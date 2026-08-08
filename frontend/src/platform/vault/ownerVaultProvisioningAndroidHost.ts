import {
	type CommittedProvisioningHostInputV1,
	type OwnerVaultProvisioningHostV1,
	type SealProvisioningHostInputV1,
	type SealedProvisioningHostCommandV1,
	type StartedProvisioningHostSessionV1,
	type SanitizedProvisioningHostReceiptV1,
} from './ownerVaultProvisioningHost'

type AndroidOwnerVaultProvisioningHostBridgeV1 = {
	start: () => Promise<{
		host_session_id: string
		response_recipient_hpke_public_key_x25519: number[]
	}>
	seal: (request: {
		request: AndroidSealProvisioningHostRequestV1
	}) => Promise<{
		operation_digest_sha256: number[]
		hpke_encapped_key: number[]
		ciphertext: number[]
		hpke_authentication_tag: number[]
	}>
	open_receipt: (request: {
		request: {
			host_session_id: string
			committed: AndroidCommittedProvisioningHostInputV1
		}
	}) => Promise<{
		operation_id: number[]
		action: number
		secret_revision: string
		state: number
	}>
	cancel: (request: { host_session_id: string }) => Promise<unknown>
}

type AndroidSealProvisioningHostRequestV1 = {
	host_session_id: string
	operation_id: number[]
	action: number
	secret_class: number
	secret_payload: number[]
	authorized: {
		vault_runtime_generation: string
		vault_hpke_public_key_x25519: number[]
		audience_registration_id: string
		audience_runtime_instance_id: string
		audience_runtime_generation: string
		audience_grant_epoch: string
		lease_request_id: number[]
		lease_operation_digest_sha256: number[]
		command_request_id: number[]
		lease_response_hpke_encapped_key: number[]
		lease_response_ciphertext: number[]
		lease_response_hpke_authentication_tag: number[]
	}
}

type AndroidCommittedProvisioningHostInputV1 = {
	vault_runtime_generation: string
	command_request_id: number[]
	operation_digest_sha256: number[]
	receipt_hpke_encapped_key: number[]
	receipt_ciphertext: number[]
	receipt_hpke_authentication_tag: number[]
}

declare global {
	interface Window {
		__MAKOSH_ANDROID_OWNER_VAULT_HOST__?: {
			vaultProvisioningHost?: AndroidOwnerVaultProvisioningHostBridgeV1
		}
	}
}

const BRIDGE = 'vaultProvisioningHost'

export class AndroidOwnerVaultProvisioningHostV1 implements OwnerVaultProvisioningHostV1 {
	private readonly bridgeImpl: AndroidOwnerVaultProvisioningHostBridgeV1

	constructor(
		private readonly bridge: Window['__MAKOSH_ANDROID_OWNER_VAULT_HOST__'] =
			typeof window === 'undefined' ? undefined : window.__MAKOSH_ANDROID_OWNER_VAULT_HOST__,
	) {
		this.bridgeImpl = resolveBridge(bridge)
	}

	start(): Promise<StartedProvisioningHostSessionV1> {
		return this.bridgeImpl.start().then((response) => ({
			hostSessionId: response.host_session_id,
			responseRecipientHpkePublicKeyX25519: bytes(response.response_recipient_hpke_public_key_x25519),
		}))
	}

	seal(input: SealProvisioningHostInputV1): Promise<SealedProvisioningHostCommandV1> {
		const secretPayload = Array.from(input.secretPayload)
		try {
			return this.bridgeImpl.seal({
				request: {
					host_session_id: input.hostSessionId,
					operation_id: Array.from(input.operationId),
					action: input.action,
					secret_class: input.secretClass,
					secret_payload: secretPayload,
					authorized: {
						vault_runtime_generation: input.authorized.vaultRuntimeGeneration.toString(),
						vault_hpke_public_key_x25519: Array.from(input.authorized.vaultHpkePublicKeyX25519),
						audience_registration_id: input.authorized.audienceRegistrationId,
						audience_runtime_instance_id: input.authorized.audienceRuntimeInstanceId,
						audience_runtime_generation: input.authorized.audienceRuntimeGeneration.toString(),
						audience_grant_epoch: input.authorized.audienceGrantEpoch.toString(),
						lease_request_id: Array.from(input.authorized.leaseRequestId),
						lease_operation_digest_sha256: Array.from(input.authorized.leaseOperationDigestSha256),
						command_request_id: Array.from(input.authorized.commandRequestId),
						lease_response_hpke_encapped_key: Array.from(input.authorized.leaseResponseHpkeEncappedKey),
						lease_response_ciphertext: Array.from(input.authorized.leaseResponseCiphertext),
						lease_response_hpke_authentication_tag: Array.from(input.authorized.leaseResponseHpkeAuthenticationTag),
					},
				},
			}).then((response) => ({
				operationDigestSha256: bytes(response.operation_digest_sha256),
				hpkeEncappedKey: bytes(response.hpke_encapped_key),
				ciphertext: bytes(response.ciphertext),
				hpkeAuthenticationTag: bytes(response.hpke_authentication_tag),
			}))
		} finally {
			secretPayload.fill(0)
		}
	}

	openReceipt(
		hostSessionId: string,
		committed: CommittedProvisioningHostInputV1,
	): Promise<SanitizedProvisioningHostReceiptV1> {
		return this.bridgeImpl.open_receipt({
			request: {
				host_session_id: hostSessionId,
				committed: {
					vault_runtime_generation: committed.vaultRuntimeGeneration.toString(),
					command_request_id: Array.from(committed.commandRequestId),
					operation_digest_sha256: Array.from(committed.operationDigestSha256),
					receipt_hpke_encapped_key: Array.from(committed.receiptHpkeEncappedKey),
					receipt_ciphertext: Array.from(committed.receiptCiphertext),
					receipt_hpke_authentication_tag: Array.from(committed.receiptHpkeAuthenticationTag),
				},
			},
		}).then((response) => ({
			operationId: bytes(response.operation_id),
			action: response.action,
			secretRevision: BigInt(response.secret_revision),
			state: response.state,
		}))
	}

	cancel(hostSessionId: string): Promise<void> {
		return this.bridgeImpl.cancel({ host_session_id: hostSessionId }).then(() => undefined)
	}
}

function resolveBridge(
	bridge: Window['__MAKOSH_ANDROID_OWNER_VAULT_HOST__'],
): AndroidOwnerVaultProvisioningHostBridgeV1 {
	if (!bridge?.[BRIDGE]) throw new Error('android host provisioning bridge is unavailable')
	return bridge[BRIDGE]
}

function bytes(value: number[]): Uint8Array {
	if (!Array.isArray(value) || value.some((item) => !Number.isInteger(item) || item < 0 || item > 255)) {
		throw new Error('android vault provisioning host response is invalid')
	}
	return Uint8Array.from(value)
}
