import type {
	MailAccountStatusV1,
	MailCredentialBindingReceiptV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import { MailCredentialPurposeV1 } from '../../../gen/makosh/mail/account/v1/client_pb'
import type {
	GmailOAuthStartedV1,
	MailAcceptedV1,
} from '../../../gen/makosh/mail/v1/client_pb'
import {
	ManagedIntegrationSetupV1,
	type ManagedIntegrationSetupReceiptV1,
	type OwnerSettingInputV1,
} from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
} from '../../../platform/vault'
import { bindMailCredential } from '../api/mailCredentialBindingClient'
import { getMailAccountStatus } from '../api/mailAccountQueryClient'
import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'

const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailAccountSetupPortsV1 = {
	configuration: Pick<ManagedIntegrationSetupV1, 'createTarget' | 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provision'>
	mail: {
		status(connectionId: string): Promise<MailAccountStatusV1>
		bind(input: {
			connectionId: string
			purpose: MailCredentialPurposeV1
			expectedBindingRevision: bigint
			credentialRevision: bigint
		}): Promise<MailCredentialBindingReceiptV1>
	}
	oauth: Pick<MailGmailOAuthClientV1, 'start' | 'complete'>
}

export type MailImapAccountSetupInputV1 = {
	imapPassword: Uint8Array
} & MailImapSettingsInputV1

export type MailImapSettingsInputV1 = {
	registrationId: string
	expectedDesiredRevision: bigint
	connectionId: string
	imapHost: string
	imapPort: bigint
	username: string
	smtp?: {
		host: string
		port: bigint
		username: string
		fromAddress: string
		password: Uint8Array
	}
}

export type MailGmailSettingsInputV1 = {
	connectionId: string
	email: string
	clientId: string
	redirectUri: string
}

export type MailGmailSetupStateV1 = {
	operationId: string
	connectionId: string
	configurationInstanceId: string
	started: GmailOAuthStartedV1
	configuration: ManagedIntegrationSetupReceiptV1
}

export class MailAccountSetupWorkflowV1 {
	constructor(private readonly ports: MailAccountSetupPortsV1 = defaultPorts()) {}

	async setupImap(input: MailImapAccountSetupInputV1): Promise<void> {
		const connectionId = required(input.connectionId, 'mail_connection_id_invalid')
		const target = await this.ports.configuration.createTarget(input.registrationId)
		await this.ports.configuration.apply({
			registrationId: input.registrationId,
			expectedDesiredRevision: target.desiredRevision,
			storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
			configurationInstanceId: target.configurationInstanceId,
			requestHostBridge: false,
			values: mailImapSettings(input),
		})
		let status = await this.ports.mail.status(connectionId)
		await this.provisionAndBind({
			registrationId: input.registrationId,
			configurationInstanceId: target.configurationInstanceId,
			connectionId,
			kind: 'imap',
			secretPayload: input.imapPassword,
			status,
		})
		if (input.smtp) {
			status = await this.ports.mail.status(connectionId)
			await this.provisionAndBind({
				registrationId: input.registrationId,
				configurationInstanceId: target.configurationInstanceId,
				connectionId,
				kind: 'smtp',
				secretPayload: input.smtp.password,
				status,
			})
		}
	}

	async startGmail(input: {
		registrationId: string
		expectedDesiredRevision: bigint
		connectionId: string
		clientId: string
		redirectUri: string
	}): Promise<MailGmailSetupStateV1> {
		const connectionId = required(input.connectionId, 'mail_connection_id_invalid')
		const target = await this.ports.configuration.createTarget(input.registrationId)
		const configuration = await this.ports.configuration.apply({
			registrationId: input.registrationId,
			expectedDesiredRevision: target.desiredRevision,
			storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
			configurationInstanceId: target.configurationInstanceId,
			requestHostBridge: false,
			values: mailGmailPreauthorizationSettings(input),
		})
		const operationId = crypto.randomUUID()
		const started = await this.ports.oauth.start(operationId, connectionId)
		return {
			operationId,
			connectionId,
			configurationInstanceId: target.configurationInstanceId,
			started,
			configuration,
		}
	}

	async completeGmail(
		state: MailGmailSetupStateV1,
		input: { returnedState: string; authorizationCode: string },
	): Promise<MailAcceptedV1> {
		return this.ports.oauth.complete({
			operationId: state.operationId,
			connectionId: state.connectionId,
			setupId: state.started.setupId,
			state: required(input.returnedState, 'mail_gmail_state_required'),
			authorizationCode: required(
				input.authorizationCode,
				'mail_gmail_authorization_code_required',
			),
		})
	}

	private async provisionAndBind(input: {
		registrationId: string
		configurationInstanceId: string
		connectionId: string
		kind: 'imap' | 'smtp'
		secretPayload: Uint8Array
		status: MailAccountStatusV1
	}): Promise<void> {
		const purpose = input.kind === 'imap'
			? MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD
			: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD
		const current = input.status.binding.find((entry) => entry.purpose === purpose)
		const credentialRevision = (current?.credentialRevision ?? 0n) + 1n
		const vault = await this.ports.vault.provision({
			targetRegistrationId: input.registrationId,
			capabilityId: `mail.${input.kind}.credential-provisioning.v1`,
			configurationInstanceId: input.configurationInstanceId,
			purposeId: `mail_${input.kind}_password`,
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: current?.credentialRevision
				? OwnerVaultActionV1.REPLACE_CAS
				: OwnerVaultActionV1.CREATE,
			secretRevision: credentialRevision,
			secretPayload: input.secretPayload,
		})
		await this.ports.mail.bind({
			connectionId: input.connectionId,
			purpose,
			expectedBindingRevision: current?.bindingRevision ?? 0n,
			credentialRevision: vault.secretRevision,
		})
	}
}

function defaultPorts(): MailAccountSetupPortsV1 {
	return {
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		mail: { status: getMailAccountStatus, bind: bindMailCredential },
		oauth: new MailGmailOAuthClientV1(),
	}
}

export function mailImapSettings(input: MailImapSettingsInputV1): OwnerSettingInputV1[] {
	const values = [
		stringInput('mail.connection_id', input.connectionId),
		stringInput('mail.imap.host', required(input.imapHost, 'mail_imap_host_invalid')),
		unsignedInput('mail.imap.port', port(input.imapPort)),
		stringInput('mail.imap.username', required(input.username, 'mail_username_invalid')),
		stringInput('mail.inbound.kind', 'imap'),
		booleanInput('mail.smtp.enabled', Boolean(input.smtp)),
		unsignedInput('mail.sync.window', 100n),
		unsignedInput('mail.sync.windows', 10n),
	]
	if (input.smtp) {
		values.push(
			stringInput('mail.smtp.from_address', required(input.smtp.fromAddress, 'mail_smtp_from_invalid')),
			stringInput('mail.smtp.host', required(input.smtp.host, 'mail_smtp_host_invalid')),
			unsignedInput('mail.smtp.port', port(input.smtp.port)),
			stringInput('mail.smtp.username', required(input.smtp.username, 'mail_smtp_username_invalid')),
		)
	}
	return values.sort(bySettingId)
}

export function mailGmailSettings(input: MailGmailSettingsInputV1): OwnerSettingInputV1[] {
	const email = required(input.email, 'mail_gmail_email_invalid')
	return gmailSettings(input, email, email)
}

export function mailGmailPreauthorizationSettings(input: Omit<MailGmailSettingsInputV1, 'email'>):
	OwnerSettingInputV1[] {
	return gmailSettings(input, 'me')
}

function gmailSettings(
	input: Omit<MailGmailSettingsInputV1, 'email'>,
	userId: string,
	fromAddress?: string,
): OwnerSettingInputV1[] {
	const values = [
		stringInput('mail.connection_id', input.connectionId),
		stringInput('mail.gmail.api_host', 'gmail.googleapis.com'),
		unsignedInput('mail.gmail.api_port', 443n),
		stringInput('mail.gmail.oauth.authorization_host', 'accounts.google.com'),
		stringInput('mail.gmail.oauth.authorization_path', '/o/oauth2/v2/auth'),
		unsignedInput('mail.gmail.oauth.authorization_port', 443n),
		stringInput('mail.gmail.oauth.client_id', required(input.clientId, 'mail_gmail_client_id_invalid')),
		stringInput('mail.gmail.oauth.redirect_uri', required(input.redirectUri, 'mail_gmail_redirect_invalid')),
		stringInput('mail.gmail.oauth.token_host', 'oauth2.googleapis.com'),
		stringInput('mail.gmail.oauth.token_path', '/token'),
		unsignedInput('mail.gmail.oauth.token_port', 443n),
		stringInput('mail.gmail.user_id', userId),
		stringInput('mail.inbound.kind', 'gmail'),
		booleanInput('mail.smtp.enabled', false),
		unsignedInput('mail.sync.window', 100n),
		unsignedInput('mail.sync.windows', 10n),
	]
	if (fromAddress) values.push(stringInput('mail.gmail.from_address', fromAddress))
	return values.sort(bySettingId)
}

function stringInput(settingId: string, value: string): OwnerSettingInputV1 {
	return { settingId, value: { case: 'stringValue', value } }
}

function unsignedInput(settingId: string, value: bigint): OwnerSettingInputV1 {
	return { settingId, value: { case: 'unsignedIntegerValue', value } }
}

function booleanInput(settingId: string, value: boolean): OwnerSettingInputV1 {
	return { settingId, value: { case: 'booleanValue', value } }
}

function bySettingId(left: OwnerSettingInputV1, right: OwnerSettingInputV1): number {
	return left.settingId.localeCompare(right.settingId)
}

function port(value: bigint): bigint {
	if (value <= 0n || value > 65_535n) throw new Error('mail_port_invalid')
	return value
}

function required(value: string, code: string): string {
	const normalized = value.trim()
	if (!normalized || normalized.length > 4096) throw new Error(code)
	return normalized
}
