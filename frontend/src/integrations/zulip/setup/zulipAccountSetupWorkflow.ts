import type { ZulipAccountLifecycleReceiptV1 } from '../../../gen/makosh/zulip/account/v1/client_pb'
import {
	ManagedIntegrationSetupV1,
	OwnerModuleSettingsClientV1,
	type ManagedIntegrationSetupReceiptV1,
} from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import { bindZulipCredential } from '../api/zulipAccountLifecycleClient'

const ZULIP_STORAGE_CAPABILITY_ID = 'zulip.storage.v1'
const ZULIP_PROVISIONING_CAPABILITY_ID = 'zulip.api-key.credential-provisioning.v1'

type ZulipAccountSetupPortsV1 = {
	configuration: Pick<ManagedIntegrationSetupV1, 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provision'>
	lifecycle: {
		bind(input: {
			accountId: string
			expectedBindingRevision: bigint
			credentialRevision: bigint
		}): Promise<ZulipAccountLifecycleReceiptV1>
	}
	activation: Pick<OwnerModuleSettingsClientV1, 'applyManagedIntegration'>
}

export type ZulipAccountSetupReceiptV1 = {
	vault: SanitizedProvisioningHostReceiptV1
	configuration: ManagedIntegrationSetupReceiptV1
	binding: ZulipAccountLifecycleReceiptV1
}

export class ZulipAccountSetupWorkflowV1 {
	constructor(private readonly ports: ZulipAccountSetupPortsV1 = defaultPorts()) {}

	async setup(input: {
		registrationId: string
		expectedDesiredRevision: bigint
		accountId: string
		accountEmail: string
		realmUrl: string
		apiKey: Uint8Array
	}): Promise<ZulipAccountSetupReceiptV1> {
		const accountId = required(input.accountId, 'zulip_account_id_invalid')
		const accountEmail = required(input.accountEmail, 'zulip_account_email_invalid')
		const realmUrl = validRealm(input.realmUrl)
		const vault = await this.ports.vault.provision({
			targetRegistrationId: input.registrationId,
			capabilityId: ZULIP_PROVISIONING_CAPABILITY_ID,
			configurationInstanceId: accountId,
			purposeId: 'zulip_api_key',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
			secretPayload: input.apiKey,
		})
		const configuration = await this.ports.configuration.apply({
			registrationId: input.registrationId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			storageCapabilityId: ZULIP_STORAGE_CAPABILITY_ID,
			configurationInstanceId: accountId,
			requestHostBridge: false,
			values: [
				stringInput('zulip.account_id', accountId),
				stringInput('zulip.account_email', accountEmail),
				stringInput('zulip.realm_url', realmUrl),
			],
		})
		const binding = await this.ports.lifecycle.bind({
			accountId,
			expectedBindingRevision: 0n,
			credentialRevision: vault.secretRevision,
		})
		await this.ports.activation.applyManagedIntegration({
			registrationId: input.registrationId,
			storageCapabilityId: ZULIP_STORAGE_CAPABILITY_ID,
			configurationInstanceId: accountId,
			expectedDesiredRevision: configuration.settings.desiredRevision,
			requestHostBridge: false,
		})
		return { vault, configuration, binding }
	}
}

function defaultPorts(): ZulipAccountSetupPortsV1 {
	return {
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		lifecycle: { bind: bindZulipCredential },
		activation: new OwnerModuleSettingsClientV1(),
	}
}

function stringInput(settingId: string, value: string) {
	return { settingId, value: { case: 'stringValue' as const, value } }
}

function required(value: string, code: string): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > 256) throw new Error(code)
	return normalized
}

function validRealm(value: string): string {
	const normalized = required(value, 'zulip_realm_invalid')
	const parsed = new URL(normalized)
	if (parsed.protocol !== 'https:' || parsed.pathname !== '/' || parsed.search || parsed.hash) {
		throw new Error('zulip_realm_invalid')
	}
	return parsed.origin
}
