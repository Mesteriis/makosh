import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ApplyOwnerManagedIntegrationSettingsReceiptV1Schema,
	CreateOwnerModuleSettingsTargetReceiptV1Schema,
	UpdateOwnerModuleSettingsReceiptV1Schema,
} from '../../../gen/makosh/gateway/v1/owner_module_settings_pb'
import {
	MailAccountReadinessV1,
	MailAccountStatusV1Schema,
	MailCredentialBindingReceiptV1Schema,
	MailCredentialBindingStateV1,
	MailCredentialBindingStatusV1Schema,
	MailCredentialPurposeV1,
	MailConnectorProfileV1,
	MailProviderPathReadinessV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import {
	GmailOAuthOperationKindV1,
	GmailOAuthOperationStatusV1Schema,
	GmailOAuthOutcomeV1,
	GmailOAuthStartedV1Schema,
	MailAcceptedV1Schema,
} from '../../../gen/makosh/mail/v1/client_pb'
import {
	MailAccountConfigurationV1Schema,
	MailAccountExportV1Schema,
	MailAddressBookProviderV1,
	MailExportAccountReadinessV1,
	MailExportConnectorProfileV1,
	MailExportProviderPathReadinessV1,
	MailGmailConfigurationV1Schema,
	MailHttpEndpointV1Schema,
	MailImapConfigurationV1Schema,
	MailSmtpConfigurationV1Schema,
	MailTlsEndpointV1Schema,
} from '../../../gen/makosh/mail/portability/v1/portability_pb'
import { OwnerVaultActionV1 } from '../../../platform/vault'
import { serializeMailAccountExportV1 } from './mailAccountPortabilityCodec'
import { MAIL_SETTINGS_SCHEMA_REVISION_V2 } from './mailAccountPortabilityCodec'
import {
	MailAccountPortabilityWorkflowV1,
	type MailAccountPortabilityPortsV1,
} from './mailAccountPortabilityWorkflow'

describe('MailAccountPortabilityWorkflowV1', () => {
	it('keeps every IMAP/SMTP receipt visible across configuration and activation successors', async () => {
		const ports = fakePorts()
		const workflow = new MailAccountPortabilityWorkflowV1(ports)
		let state = workflow.initializeImport({
			json: imapExportJson(true),
			targetRegistrationId: 'mail-registration',
			expectedDesiredRevision: 1n,
		})

		state = await workflow.updateSettings(state)
		state = await workflow.applyConfiguration(state)
		const imapSecret = new Uint8Array([1, 2])
		state = await workflow.provisionCredential(state, 'imap', imapSecret)
		const smtpSecret = new Uint8Array([3, 4])
		state = await workflow.provisionCredential(state, 'smtp', smtpSecret)
		state = await workflow.activateCredentials(state)

		expect(state.phase).toBe('ready')
		expect(state.settingsUpdateReceipt?.desiredRevision).toBe(2n)
		expect(state.configurationApplyReceipt?.runtimeGeneration).toBe(10n)
		expect(state.imap?.vaultReceipt?.secretRevision).toBe(1n)
		expect(state.imap?.bindingReceipt?.bindingRevision).toBe(1n)
		expect(state.smtp?.vaultReceipt?.secretRevision).toBe(1n)
		expect(state.smtp?.bindingReceipt?.bindingRevision).toBe(1n)
		expect(state.activationApplyReceipt?.runtimeGeneration).toBe(11n)
		expect(imapSecret).toEqual(new Uint8Array(2))
		expect(smtpSecret).toEqual(new Uint8Array(2))
		expect(ports.settings.applyManagedIntegration).toHaveBeenCalledTimes(2)
		expect(ports.vault.provision).toHaveBeenCalledTimes(2)
		expect(ports.mail.bind).toHaveBeenCalledTimes(2)
	})

	it('resumes binding from a durable Vault receipt without provisioning twice', async () => {
		const ports = fakePorts(false)
		ports.mail.bind = vi.fn()
			.mockRejectedValueOnce(new Error('route unavailable'))
			.mockResolvedValueOnce(create(MailCredentialBindingReceiptV1Schema, {
				connectionId: 'mail-account',
				purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
				bindingRevision: 1n,
				state: MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_PENDING_RESTART,
			}))
		const workflow = new MailAccountPortabilityWorkflowV1(ports)
		let state = workflow.initializeImport({
			json: imapExportJson(false),
			targetRegistrationId: 'mail-registration',
			expectedDesiredRevision: 1n,
		})
		state = await workflow.updateSettings(state)
		state = await workflow.applyConfiguration(state)

		state = await workflow.provisionCredential(state, 'imap', new Uint8Array([9]))
		expect(state.lastErrorCode).toBe('mail_import_imap_credential_failed')
		expect(state.imap?.vaultReceipt?.secretRevision).toBe(1n)
		expect(state.imap?.bindingReceipt).toBeUndefined()

		state = await workflow.provisionCredential(state, 'imap')
		expect(state.phase).toBe('credentials_bound')
		expect(state.imap?.bindingReceipt?.bindingRevision).toBe(1n)
		expect(ports.vault.provision).toHaveBeenCalledOnce()
		expect(ports.mail.bind).toHaveBeenCalledTimes(2)
	})

	it('keeps Gmail authorization, acceptance and terminal reconciliation as separate receipts', async () => {
		const configurationOnly = create(MailAccountStatusV1Schema, {
			connectionId: 'gmail-account',
			settingsRevision: 2n,
			runtimeGeneration: 20n,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY,
			connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL,
			syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED,
			deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED,
		})
		const ready = create(MailAccountStatusV1Schema, {
			...configurationOnly,
			runtimeGeneration: 21n,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
			deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
		})
		const ports: MailAccountPortabilityPortsV1 = {
			settings: {
				createConfigurationTarget: vi.fn().mockResolvedValue(create(
					CreateOwnerModuleSettingsTargetReceiptV1Schema,
					{
						registrationId: 'mail-registration',
						configurationInstanceId: 'mail-target',
						desiredRevision: 1n,
						applyState: 'draft',
					},
				)),
				exportEffective: vi.fn(),
				updateDesired: vi.fn().mockResolvedValue(create(
					UpdateOwnerModuleSettingsReceiptV1Schema,
					{ registrationId: 'mail-registration', desiredRevision: 2n },
				)),
				applyManagedIntegration: vi.fn().mockResolvedValue(create(
					ApplyOwnerManagedIntegrationSettingsReceiptV1Schema,
					{
						registrationId: 'mail-registration',
						effectiveRevision: 2n,
						runtimeGeneration: 20n,
						applyState: 'current',
					},
				)),
			},
			vault: { provision: vi.fn() },
			mail: {
				status: vi.fn()
					.mockResolvedValueOnce(configurationOnly)
					.mockResolvedValueOnce(ready),
				bind: vi.fn(),
			},
			gmailOAuth: {
				start: vi.fn().mockImplementation(async (operationId) => create(
					GmailOAuthStartedV1Schema,
					{
						operationId,
						setupId: 'setup',
						authorizationUrl: 'https://accounts.google.test/authorize',
						expiresAtUnixSeconds: 100n,
					},
				)),
				complete: vi.fn().mockImplementation(async (input) => create(
					MailAcceptedV1Schema,
					{ operationId: input.operationId },
				)),
				status: vi.fn().mockImplementation(async (operationId) => create(
					GmailOAuthOperationStatusV1Schema,
					{
						operationId,
						kind: GmailOAuthOperationKindV1.GMAIL_OAUTH_OPERATION_KIND_COMPLETE,
						outcome: GmailOAuthOutcomeV1.GMAIL_OAUTH_OUTCOME_COMPLETED,
						requestedAtUnixSeconds: 1n,
						completedAtUnixSeconds: 2n,
					},
				)),
			},
		}
		const workflow = new MailAccountPortabilityWorkflowV1(ports)
		let state = workflow.initializeImport({
			json: gmailExportJson(),
			targetRegistrationId: 'mail-registration',
			expectedDesiredRevision: 1n,
		})
		state = await workflow.updateSettings(state)
		state = await workflow.applyConfiguration(state)
		state = await workflow.startGmailOAuth(state)
		state = await workflow.completeGmailOAuth(state, {
			state: 'returned-state',
			authorizationCode: 'one-use-code',
		})
		state = await workflow.reconcile(state)

		expect(state.phase).toBe('ready')
		expect(state.gmailOAuthStarted?.setupId).toBe('setup')
		expect(state.gmailOAuthAccepted?.operationId).toBe(state.gmailOAuthOperationId)
		expect(state.gmailOAuthStatus?.outcome)
			.toBe(GmailOAuthOutcomeV1.GMAIL_OAUTH_OUTCOME_COMPLETED)
		expect(ports.vault.provision).not.toHaveBeenCalled()
		expect(ports.mail.bind).not.toHaveBeenCalled()
	})
})

function fakePorts(smtp = true): MailAccountPortabilityPortsV1 & {
	settings: MailAccountPortabilityPortsV1['settings'] & {
		updateDesired: ReturnType<typeof vi.fn>
		applyManagedIntegration: ReturnType<typeof vi.fn>
	}
	vault: { provision: ReturnType<typeof vi.fn> }
	mail: MailAccountPortabilityPortsV1['mail'] & {
		status: ReturnType<typeof vi.fn>
		bind: ReturnType<typeof vi.fn>
	}
} {
	const configurationOnly = status(
		MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY,
		smtp,
	)
	const ready = status(MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY, smtp)
	return {
		settings: {
			createConfigurationTarget: vi.fn().mockResolvedValue(create(
				CreateOwnerModuleSettingsTargetReceiptV1Schema,
				{
					registrationId: 'mail-registration',
					configurationInstanceId: 'mail-target',
					desiredRevision: 1n,
					applyState: 'draft',
				},
			)),
			exportEffective: vi.fn(),
			updateDesired: vi.fn().mockResolvedValue(create(
				UpdateOwnerModuleSettingsReceiptV1Schema,
				{
					registrationId: 'mail-registration',
					desiredRevision: 2n,
					applyState: 'pending_apply',
				},
			)),
			applyManagedIntegration: vi.fn()
				.mockResolvedValueOnce(create(
					ApplyOwnerManagedIntegrationSettingsReceiptV1Schema,
					{
						registrationId: 'mail-registration',
						effectiveRevision: 2n,
						runtimeGeneration: 10n,
						applyState: 'current',
					},
				))
				.mockResolvedValueOnce(create(
					ApplyOwnerManagedIntegrationSettingsReceiptV1Schema,
					{
						registrationId: 'mail-registration',
						effectiveRevision: 2n,
						runtimeGeneration: 11n,
						applyState: 'current',
					},
				)),
		},
		vault: {
			provision: vi.fn().mockImplementation(async (input) => {
				input.secretPayload.fill(0)
				return {
					operationId: input.operationId,
					action: OwnerVaultActionV1.CREATE,
					secretRevision: input.secretRevision,
					state: 1,
				}
			}),
		},
		mail: {
			status: vi.fn()
				.mockResolvedValueOnce(configurationOnly)
				.mockResolvedValueOnce(ready),
			bind: vi.fn().mockImplementation(async (input) => create(
				MailCredentialBindingReceiptV1Schema,
				{
					connectionId: input.connectionId,
					purpose: input.purpose,
					bindingRevision: input.expectedBindingRevision + 1n,
					state: MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_PENDING_RESTART,
				},
			)),
		},
		gmailOAuth: {
			start: vi.fn(),
			complete: vi.fn(),
			status: vi.fn(),
		},
	}
}

function status(readiness: MailAccountReadinessV1, smtp: boolean) {
	const bindings = [
		create(MailCredentialBindingStatusV1Schema, {
			purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
			state: MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_UNCONFIGURED,
		}),
	]
	if (smtp) {
		bindings.push(create(MailCredentialBindingStatusV1Schema, {
			purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD,
			state: MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_UNCONFIGURED,
		}))
	}
	return create(MailAccountStatusV1Schema, {
		connectionId: 'mail-account',
		settingsRevision: 2n,
		runtimeGeneration: readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
			? 11n
			: 10n,
		readiness,
		connectorProfile: smtp
			? MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP_SMTP
			: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP,
		syncReadiness: readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
			? MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY
			: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED,
		deliveryReadiness: smtp
			? MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED
			: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_NOT_CONFIGURED,
		binding: bindings,
	})
}

function imapExportJson(smtp: boolean): string {
	return serializeMailAccountExportV1(create(MailAccountExportV1Schema, {
		major: 1,
		exportedAtUnixMillis: 1n,
		sourceRegistrationId: 'source-mail-registration',
		settingsSchemaMajor: 2,
		settingsSchemaRevision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
		effectiveSettingsRevision: 1n,
		connectorProfile: smtp
			? MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_IMAP_SMTP
			: MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_IMAP,
		readiness: MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_READY,
		syncReadiness: MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_READY,
		deliveryReadiness: smtp
			? MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_READY
			: MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_NOT_CONFIGURED,
		configuration: create(MailAccountConfigurationV1Schema, {
			addressBookProvider: MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_NONE,
			connectionId: 'mail-account',
			syncWindow: 100,
			syncWindows: 2,
			inbound: {
				case: 'imap',
				value: create(MailImapConfigurationV1Schema, {
					host: 'imap.example.test',
					port: 993,
					username: 'owner@example.test',
				}),
			},
			smtp: smtp
				? create(MailSmtpConfigurationV1Schema, {
					host: 'smtp.example.test',
					port: 465,
					username: 'owner@example.test',
					fromAddress: 'owner@example.test',
				})
				: undefined,
		}),
	}))
}

function gmailExportJson(): string {
	return serializeMailAccountExportV1(create(MailAccountExportV1Schema, {
		major: 1,
		exportedAtUnixMillis: 1n,
		sourceRegistrationId: 'source-mail-registration',
		settingsSchemaMajor: 2,
		settingsSchemaRevision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
		effectiveSettingsRevision: 1n,
		connectorProfile: MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_GMAIL,
		readiness: MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_READY,
		syncReadiness: MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_READY,
		deliveryReadiness: MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_READY,
		configuration: create(MailAccountConfigurationV1Schema, {
			addressBookProvider: MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_NONE,
			connectionId: 'gmail-account',
			syncWindow: 100,
			syncWindows: 2,
			inbound: {
				case: 'gmail',
				value: create(MailGmailConfigurationV1Schema, {
					userId: 'owner@example.test',
					fromAddress: 'owner@example.test',
					apiEndpoint: create(MailTlsEndpointV1Schema, {
						host: 'gmail.googleapis.com',
						port: 443,
					}),
					oauthClientId: 'client-id',
					oauthRedirectUri: 'http://127.0.0.1/callback',
					oauthAuthorizationEndpoint: create(MailHttpEndpointV1Schema, {
						host: 'accounts.google.com',
						port: 443,
						path: '/o/oauth2/v2/auth',
					}),
					oauthTokenEndpoint: create(MailHttpEndpointV1Schema, {
						host: 'oauth2.googleapis.com',
						port: 443,
						path: '/token',
					}),
				}),
			},
		}),
	}))
}
