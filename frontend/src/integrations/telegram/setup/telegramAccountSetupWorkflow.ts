import type { TelegramAccountResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import {
	ManagedIntegrationSetupV1,
	type ManagedIntegrationSetupReceiptV1,
} from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	type OwnerVaultCustodiedSealerV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import { provisionTelegramAccount } from '../api/telegramLifecycleGateway'
import { withTelegramConfigurationRuntimeV1 } from './telegramConfigurationRuntimeRetry'

const TELEGRAM_STORAGE_CAPABILITY_ID = 'telegram.storage.v1'
const API_HASH_PROVISIONING_CAPABILITY_ID =
	'telegram.api-hash.credential-provisioning.v1'
const SESSION_KEY_PROVISIONING_CAPABILITY_ID =
	'telegram.session-store-key.credential-provisioning.v1'

type TelegramAccountSetupPortsV1 = {
	configuration: Pick<ManagedIntegrationSetupV1, 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provision' | 'provisionCustodied'>
	lifecycle: {
		provision(input: {
			accountId: string
			displayName: string
			externalAccountId: string
			credentials: readonly { purpose: string; revision: bigint }[]
		}): Promise<TelegramAccountResponse>
	}
}

export type TelegramAccountSetupReceiptV1 = {
	apiHash: SanitizedProvisioningHostReceiptV1
	sessionKey: SanitizedProvisioningHostReceiptV1
	configuration: ManagedIntegrationSetupReceiptV1
	account: TelegramAccountResponse
}

export class TelegramAccountSetupWorkflowV1 {
	constructor(private readonly ports: TelegramAccountSetupPortsV1 = defaultPorts()) {}

	async setup(input: {
		registrationId: string
		expectedDesiredRevision: bigint
		accountId: string
		displayName: string
		apiId: bigint
		replaceExistingCredentials?: boolean
	} & (
		| { apiHash: Uint8Array; apiHashSealer?: never }
		| { apiHash?: never; apiHashSealer: OwnerVaultCustodiedSealerV1 }
	)): Promise<TelegramAccountSetupReceiptV1> {
		const accountId = required(input.accountId, 'telegram_account_id_invalid')
		const displayName = required(input.displayName, 'telegram_display_name_invalid')
		if (input.apiId <= 0n) throw new Error('telegram_api_id_invalid')
		const configurationInstanceId = input.registrationId
		const credentialAction = input.replaceExistingCredentials
			? OwnerVaultActionV1.REPLACE_CAS
			: OwnerVaultActionV1.CREATE
		const credentialRevision = input.replaceExistingCredentials ? 2n : 1n
		const apiHashCeremony = {
			targetRegistrationId: input.registrationId,
			capabilityId: API_HASH_PROVISIONING_CAPABILITY_ID,
			configurationInstanceId,
			purposeId: 'telegram_api_hash',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: credentialAction,
			secretRevision: credentialRevision,
		}
		const apiHash = input.apiHashSealer === undefined
			? await this.ports.vault.provision({
				...apiHashCeremony,
				secretPayload: input.apiHash,
			})
			: await this.ports.vault.provisionCustodied(
				apiHashCeremony,
				input.apiHashSealer,
			)
		const sessionPayload = crypto.getRandomValues(new Uint8Array(32))
		const sessionKey = await this.ports.vault.provision({
			targetRegistrationId: input.registrationId,
			capabilityId: SESSION_KEY_PROVISIONING_CAPABILITY_ID,
			configurationInstanceId,
			purposeId: 'telegram_session_store_key',
			secretClass: OwnerVaultSecretClassV1.SESSION_STORE_KEY,
			action: credentialAction,
			secretRevision: credentialRevision,
			secretPayload: sessionPayload,
		})
		const configuration = await this.ports.configuration.apply({
			registrationId: input.registrationId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			storageCapabilityId: TELEGRAM_STORAGE_CAPABILITY_ID,
			configurationInstanceId,
			requestHostBridge: false,
			values: [
				{
					settingId: 'telegram.account_id',
					value: { case: 'stringValue', value: accountId },
				},
				{
					settingId: 'telegram.api_id',
					value: { case: 'signedIntegerValue', value: input.apiId },
				},
			],
		})
		const account = await withTelegramConfigurationRuntimeV1(() =>
			this.ports.lifecycle.provision({
			accountId,
			displayName,
			externalAccountId: '',
			credentials: [
				{ purpose: 'telegram_api_hash', revision: apiHash.secretRevision },
				{
					purpose: 'telegram_session_encryption_key',
					revision: sessionKey.secretRevision,
				},
			],
			}),
		)
		return { apiHash, sessionKey, configuration, account }
	}
}

function defaultPorts(): TelegramAccountSetupPortsV1 {
	return {
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		lifecycle: { provision: provisionTelegramAccount },
	}
}

function required(value: string, code: string): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > 128) throw new Error(code)
	return normalized
}
