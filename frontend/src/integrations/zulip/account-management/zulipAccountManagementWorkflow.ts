import type {
	ZulipAccountLifecycleReceiptV1,
} from '../../../gen/makosh/zulip/account/v1/client_pb'
import type {
	ZulipAccountStatusV1,
} from '../../../gen/makosh/zulip/operational/v1/client_pb'
import type {
	ApplyOwnerManagedIntegrationSettingsReceiptV1,
} from '../../../gen/makosh/gateway/v1/owner_module_settings_pb'
import { OwnerModuleSettingsClientV1 } from '../../../platform/settings/ownerModuleSettingsClient'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import {
	bindZulipCredential,
	retireZulipAccount,
} from '../api/zulipAccountLifecycleClient'
import { getZulipAccountStatus } from '../api/zulipAccountStatusClient'

const ZULIP_STORAGE_CAPABILITY_ID = 'zulip.storage.v1'
const ZULIP_PROVISIONING_CAPABILITY_ID = 'zulip.api-key.credential-provisioning.v1'

type ZulipAccountManagementPortsV1 = {
	status(accountId: string): Promise<ZulipAccountStatusV1>
	retire(input: {
		accountId: string
		expectedBindingRevision: bigint
	}): Promise<ZulipAccountLifecycleReceiptV1>
	bind(input: {
		accountId: string
		expectedBindingRevision: bigint
		credentialRevision: bigint
	}): Promise<ZulipAccountLifecycleReceiptV1>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provision'>
	activation: Pick<OwnerModuleSettingsClientV1, 'applyManagedIntegration'>
}

export type ZulipApiKeyRotationReceiptV1 = {
	vault: SanitizedProvisioningHostReceiptV1
	binding: ZulipAccountLifecycleReceiptV1
	application: ApplyOwnerManagedIntegrationSettingsReceiptV1
	status: ZulipAccountStatusV1
}

export class ZulipAccountManagementWorkflowV1 {
	constructor(private readonly ports: ZulipAccountManagementPortsV1 = defaultPorts()) {}

	status(accountId: string): Promise<ZulipAccountStatusV1> {
		return this.ports.status(required(accountId))
	}

	retire(status: ZulipAccountStatusV1): Promise<ZulipAccountLifecycleReceiptV1> {
		return this.ports.retire({
			accountId: required(status.accountId),
			expectedBindingRevision: status.bindingRevision,
		})
	}

	async rotateApiKey(input: {
		registrationId: string
		accountId: string
		expectedDesiredRevision: bigint
		status: ZulipAccountStatusV1
		secretPayload: Uint8Array
	}): Promise<ZulipApiKeyRotationReceiptV1> {
		const credentialRevision = input.status.credentialRevision
		if (!credentialRevision || input.status.bindingRevision <= 0n) {
			throw new Error('zulip_active_credential_binding_missing')
		}
		const accountId = required(input.accountId)
		const vault = await this.ports.vault.provision({
			targetRegistrationId: required(input.registrationId),
			capabilityId: ZULIP_PROVISIONING_CAPABILITY_ID,
			configurationInstanceId: accountId,
			purposeId: 'zulip_api_key',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.REPLACE_CAS,
			secretRevision: credentialRevision + 1n,
			secretPayload: input.secretPayload,
		})
		const binding = await this.ports.bind({
			accountId,
			expectedBindingRevision: input.status.bindingRevision,
			credentialRevision: vault.secretRevision,
		})
		const application = await this.ports.activation.applyManagedIntegration({
			registrationId: input.registrationId,
			storageCapabilityId: ZULIP_STORAGE_CAPABILITY_ID,
			configurationInstanceId: accountId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			requestHostBridge: false,
		})
		return {
			vault,
			binding,
			application,
			status: await this.ports.status(accountId),
		}
	}
}

function defaultPorts(): ZulipAccountManagementPortsV1 {
	return {
		status: getZulipAccountStatus,
		retire: retireZulipAccount,
		bind: bindZulipCredential,
		vault: new OwnerVaultProvisioningClientV1(),
		activation: new OwnerModuleSettingsClientV1(),
	}
}

function required(value: string): string {
	const normalized = value.trim()
	if (!normalized) throw new Error('zulip_account_identifier_invalid')
	return normalized
}
