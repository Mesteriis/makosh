import {
	MailCredentialPurposeV1,
	type MailAccountCatalogV1,
	type MailAccountStatusV1,
	type MailCredentialBindingReceiptV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import type { MailAccountLifecycleReceiptV1 } from '../../../gen/makosh/mail/account_lifecycle/v1/client_pb'
import type { ApplyOwnerManagedIntegrationSettingsReceiptV1 } from '../../../gen/makosh/gateway/v1/owner_module_settings_pb'
import { ManagedIntegrationSetupV1 } from '../../../platform/settings'
import { OwnerModuleSettingsClientV1 } from '../../../platform/settings/ownerModuleSettingsClient'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import { bindMailCredential } from '../api/mailCredentialBindingClient'
import {
	deleteMailAccount,
	getMailAccountLifecycleStatus,
	retireMailAccount,
	retryMailAccountLifecycle,
} from '../api/mailAccountLifecycleClient'
import { getMailAccountStatus, listMailAccounts } from '../api/mailAccountQueryClient'
import { mailGmailPreauthorizationSettings } from '../setup/mailAccountSetupWorkflow'

const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

export type MailPasswordPurposeV1 = 'imap' | 'smtp'

type MailAccountManagementPortsV1 = {
	catalog(): Promise<MailAccountCatalogV1>
	status(connectionId: string): Promise<MailAccountStatusV1>
	retire(input: {
		connectionId: string
		expectedLifecycleRevision: bigint
	}): Promise<MailAccountLifecycleReceiptV1>
	delete(input: {
		connectionId: string
		expectedLifecycleRevision: bigint
	}): Promise<MailAccountLifecycleReceiptV1>
	retry(input: {
		operationId: string
		connectionId: string
		expectedLifecycleRevision: bigint
	}): Promise<MailAccountLifecycleReceiptV1>
	lifecycleStatus(input: {
		operationId: string
		connectionId: string
	}): Promise<MailAccountLifecycleReceiptV1>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provision'>
	bind(input: {
		connectionId: string
		purpose: MailCredentialPurposeV1
		expectedBindingRevision: bigint
		credentialRevision: bigint
	}): Promise<MailCredentialBindingReceiptV1>
	activation: Pick<OwnerModuleSettingsClientV1, 'applyManagedIntegration'>
	configuration: Pick<ManagedIntegrationSetupV1, 'apply'>
}

export type MailPasswordRotationReceiptV1 = {
	vault: SanitizedProvisioningHostReceiptV1
	binding: MailCredentialBindingReceiptV1
	application: ApplyOwnerManagedIntegrationSettingsReceiptV1
	status: MailAccountStatusV1
}

export class MailAccountManagementWorkflowV1 {
	constructor(private readonly ports: MailAccountManagementPortsV1 = defaultPorts()) {}

	catalog(): Promise<MailAccountCatalogV1> {
		return this.ports.catalog()
	}

	status(connectionId: string): Promise<MailAccountStatusV1> {
		return this.ports.status(required(connectionId))
	}

	retire(status: MailAccountStatusV1): Promise<MailAccountLifecycleReceiptV1> {
		return this.ports.retire({
			connectionId: required(status.connectionId),
			expectedLifecycleRevision: status.lifecycleRevision,
		})
	}

	delete(status: MailAccountStatusV1): Promise<MailAccountLifecycleReceiptV1> {
		return this.ports.delete({
			connectionId: required(status.connectionId),
			expectedLifecycleRevision: status.lifecycleRevision,
		})
	}

	async retry(status: MailAccountStatusV1): Promise<MailAccountLifecycleReceiptV1> {
		const operationId = status.lifecycleOperationId
		if (!operationId) throw new Error('mail_lifecycle_operation_missing')
		return this.ports.retry({
			operationId,
			connectionId: status.connectionId,
			expectedLifecycleRevision: status.lifecycleRevision,
		})
	}

	async refreshLifecycle(status: MailAccountStatusV1): Promise<MailAccountLifecycleReceiptV1> {
		const operationId = status.lifecycleOperationId
		if (!operationId) throw new Error('mail_lifecycle_operation_missing')
		return this.ports.lifecycleStatus({
			operationId,
			connectionId: status.connectionId,
		})
	}

	async rotatePassword(input: {
		registrationId: string
		storageCapabilityId: string
		configurationInstanceId: string
		expectedDesiredRevision: bigint
		status: MailAccountStatusV1
		purpose: MailPasswordPurposeV1
		secretPayload: Uint8Array
	}): Promise<MailPasswordRotationReceiptV1> {
		const purpose = input.purpose === 'imap'
			? MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD
			: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD
		const current = input.status.binding.find((entry) => entry.purpose === purpose)
		if (!current?.bindingRevision || !current.credentialRevision) {
			throw new Error(`mail_${input.purpose}_binding_missing`)
		}
		const vault = await this.ports.vault.provision({
			targetRegistrationId: required(input.registrationId),
			capabilityId: `mail.${input.purpose}.credential-provisioning.v1`,
			configurationInstanceId: required(input.configurationInstanceId),
			purposeId: `mail_${input.purpose}_password`,
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.REPLACE_CAS,
			secretRevision: current.credentialRevision + 1n,
			secretPayload: input.secretPayload,
		})
		const binding = await this.ports.bind({
			connectionId: input.status.connectionId,
			purpose,
			expectedBindingRevision: current.bindingRevision,
			credentialRevision: vault.secretRevision,
		})
		const application = await this.ports.activation.applyManagedIntegration({
			registrationId: input.registrationId,
			storageCapabilityId: input.storageCapabilityId,
			configurationInstanceId: input.configurationInstanceId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			requestHostBridge: false,
		})
		return {
			vault,
			binding,
			application,
			status: await this.ports.status(input.status.connectionId),
		}
	}

	async normalizeGmailOAuthRedirect(input: {
		registrationId: string
		configurationInstanceId: string
		expectedDesiredRevision: bigint
		connectionId: string
		clientId: string
		redirectUri: string
	}): Promise<void> {
		await this.ports.configuration.apply({
			registrationId: required(input.registrationId),
			storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
			configurationInstanceId: required(input.configurationInstanceId),
			expectedDesiredRevision: input.expectedDesiredRevision,
			requestHostBridge: false,
			values: mailGmailPreauthorizationSettings({
				connectionId: required(input.connectionId),
				clientId: required(input.clientId),
				redirectUri: required(input.redirectUri),
			}),
		})
	}
}

function defaultPorts(): MailAccountManagementPortsV1 {
	return {
		catalog: listMailAccounts,
		status: getMailAccountStatus,
		retire: retireMailAccount,
		delete: deleteMailAccount,
		retry: retryMailAccountLifecycle,
		lifecycleStatus: getMailAccountLifecycleStatus,
		vault: new OwnerVaultProvisioningClientV1(),
		bind: bindMailCredential,
		activation: new OwnerModuleSettingsClientV1(),
		configuration: new ManagedIntegrationSetupV1(),
	}
}

function required(value: string): string {
	const normalized = value.trim()
	if (!normalized) throw new Error('mail_account_identifier_invalid')
	return normalized
}
