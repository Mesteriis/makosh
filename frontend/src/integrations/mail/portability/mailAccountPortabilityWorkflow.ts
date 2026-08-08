import type {
	ApplyOwnerManagedIntegrationSettingsReceiptV1,
	CreateOwnerModuleSettingsTargetReceiptV1,
	ExportEffectiveOwnerModuleSettingsReceiptV1,
	UpdateOwnerModuleSettingsReceiptV1,
} from '../../../gen/makosh/gateway/v1/owner_module_settings_pb'
import {
	MailAccountReadinessV1,
	type MailAccountStatusV1,
	type MailCredentialBindingReceiptV1,
	MailCredentialPurposeV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import type {
	GmailOAuthOperationStatusV1,
	GmailOAuthStartedV1,
	MailAcceptedV1,
} from '../../../gen/makosh/mail/v1/client_pb'
import type { MailAccountExportV1 } from '../../../gen/makosh/mail/portability/v1/portability_pb'
import { resolveOwnerOperationIdV1 } from '../../../platform/gateway/ownerOperationId'
import {
	OwnerModuleSettingsClientV1,
	type ApplyOwnerManagedIntegrationSettingsInputV1,
	type CreateOwnerModuleSettingsTargetInputV1,
	type ExportEffectiveOwnerModuleSettingsInputV1,
	type UpdateOwnerModuleSettingsInputV1,
} from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
	type OwnerVaultProvisioningInputV1,
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import {
	bindMailCredential,
	type BindMailCredentialInputV1,
} from '../api/mailCredentialBindingClient'
import { getMailAccountStatus } from '../api/mailAccountQueryClient'
import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'
import {
	buildMailAccountExportV1,
	mailAccountExportSettingsInputs,
	parseMailAccountExportV1,
	serializeMailAccountExportV1,
} from './mailAccountPortabilityCodec'

const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'
const MAIL_IMAP_PROVISIONING_CAPABILITY_ID = 'mail.imap.credential-provisioning.v1'
const MAIL_SMTP_PROVISIONING_CAPABILITY_ID = 'mail.smtp.credential-provisioning.v1'

export type MailImportCredentialKindV1 = 'imap' | 'smtp'
export type MailImportPhaseV1 =
	| 'validated'
	| 'settings_updated'
	| 'configuration_applied'
	| 'credentials_bound'
	| 'credentials_activated'
	| 'awaiting_gmail_authorization'
	| 'gmail_completion_accepted'
	| 'ready'

export type MailCredentialImportProgressV1 = {
	operationId: Uint8Array
	vaultReceipt?: SanitizedProvisioningHostReceiptV1
	bindingReceipt?: MailCredentialBindingReceiptV1
}

export type MailAccountImportStateV1 = {
	exported: MailAccountExportV1
	targetRegistrationId: string
	expectedDesiredRevision: bigint
	phase: MailImportPhaseV1
	configurationTargetOperationId: Uint8Array
	settingsUpdateOperationId: Uint8Array
	configurationApplyOperationId: Uint8Array
	activationApplyOperationId: Uint8Array
	configurationTargetReceipt?: CreateOwnerModuleSettingsTargetReceiptV1
	settingsUpdateReceipt?: UpdateOwnerModuleSettingsReceiptV1
	configurationApplyReceipt?: ApplyOwnerManagedIntegrationSettingsReceiptV1
	imap?: MailCredentialImportProgressV1
	smtp?: MailCredentialImportProgressV1
	activationApplyReceipt?: ApplyOwnerManagedIntegrationSettingsReceiptV1
	gmailOAuthOperationId?: string
	gmailOAuthStarted?: GmailOAuthStartedV1
	gmailOAuthAccepted?: MailAcceptedV1
	gmailOAuthStatus?: GmailOAuthOperationStatusV1
	accountStatus?: MailAccountStatusV1
	lastErrorCode?: string
}

type SettingsPortV1 = {
	createConfigurationTarget(
		input: CreateOwnerModuleSettingsTargetInputV1,
	): Promise<CreateOwnerModuleSettingsTargetReceiptV1>
	exportEffective(
		input: ExportEffectiveOwnerModuleSettingsInputV1,
	): Promise<ExportEffectiveOwnerModuleSettingsReceiptV1>
	updateDesired(
		input: UpdateOwnerModuleSettingsInputV1,
	): Promise<UpdateOwnerModuleSettingsReceiptV1>
	applyManagedIntegration(
		input: ApplyOwnerManagedIntegrationSettingsInputV1,
	): Promise<ApplyOwnerManagedIntegrationSettingsReceiptV1>
}

type VaultPortV1 = {
	provision(
		input: OwnerVaultProvisioningInputV1,
	): Promise<SanitizedProvisioningHostReceiptV1>
}

type MailPortV1 = {
	status(connectionId: string): Promise<MailAccountStatusV1>
	bind(input: BindMailCredentialInputV1): Promise<MailCredentialBindingReceiptV1>
}

type GmailOAuthPortV1 = {
	start(operationId: string, connectionId: string): Promise<GmailOAuthStartedV1>
	complete(input: {
		operationId: string
		connectionId: string
		setupId: string
		state: string
		authorizationCode: string
	}): Promise<MailAcceptedV1>
	status(
		operationId: string,
		connectionId: string,
	): Promise<GmailOAuthOperationStatusV1 | undefined>
}

export type MailAccountPortabilityPortsV1 = {
	settings: SettingsPortV1
	vault: VaultPortV1
	mail: MailPortV1
	gmailOAuth: GmailOAuthPortV1
}

export class MailAccountPortabilityWorkflowV1 {
	constructor(
		private readonly ports: MailAccountPortabilityPortsV1 = defaultPorts(),
	) {}

	async exportAccount(input: {
		registrationId: string
		expectedEffectiveRevision: bigint
		connectionId: string
	}): Promise<{ exported: MailAccountExportV1; json: string }> {
		const status = await this.ports.mail.status(input.connectionId)
		const settings = await this.ports.settings.exportEffective({
			registrationId: input.registrationId,
			configurationInstanceId: status.configurationInstanceId,
			expectedEffectiveRevision: input.expectedEffectiveRevision,
		})
		const exported = buildMailAccountExportV1(settings, status)
		return { exported, json: serializeMailAccountExportV1(exported) }
	}

	initializeImport(input: {
		json: string
		targetRegistrationId: string
		expectedDesiredRevision: bigint
	}): MailAccountImportStateV1 {
		if (input.targetRegistrationId.trim().length === 0
			|| input.expectedDesiredRevision <= 0n) {
			throw new Error('Mail import target is invalid')
		}
		const exported = parseMailAccountExportV1(input.json)
		return {
			exported,
			targetRegistrationId: input.targetRegistrationId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			phase: 'validated',
			configurationTargetOperationId: resolveOwnerOperationIdV1(),
			settingsUpdateOperationId: resolveOwnerOperationIdV1(),
			configurationApplyOperationId: resolveOwnerOperationIdV1(),
			activationApplyOperationId: resolveOwnerOperationIdV1(),
			imap: exported.configuration?.inbound.case === 'imap'
				? { operationId: resolveOwnerOperationIdV1() }
				: undefined,
			smtp: exported.configuration?.smtp !== undefined
				? { operationId: resolveOwnerOperationIdV1() }
				: undefined,
		}
	}

	async updateSettings(
		state: MailAccountImportStateV1,
	): Promise<MailAccountImportStateV1> {
		if (state.settingsUpdateReceipt) return state
		let next = state
		try {
			const target = state.configurationTargetReceipt
				?? await this.ports.settings.createConfigurationTarget({
					operationId: state.configurationTargetOperationId,
					registrationId: state.targetRegistrationId,
				})
			next = {
				...state,
				configurationTargetReceipt: target,
				lastErrorCode: undefined,
			}
			const receipt = await this.ports.settings.updateDesired({
				operationId: state.settingsUpdateOperationId,
				registrationId: state.targetRegistrationId,
				configurationInstanceId: target.configurationInstanceId,
				expectedDesiredRevision: target.desiredRevision,
				values: mailAccountExportSettingsInputs(state.exported),
			})
			return {
				...next,
				phase: 'settings_updated',
				settingsUpdateReceipt: receipt,
				lastErrorCode: undefined,
			}
		} catch {
			return blocked(next, 'mail_import_settings_update_failed')
		}
	}

	async applyConfiguration(
		state: MailAccountImportStateV1,
	): Promise<MailAccountImportStateV1> {
		if (state.configurationApplyReceipt) return state
		if (!state.settingsUpdateReceipt) {
			return blocked(state, 'mail_import_settings_receipt_required')
		}
		try {
			const receipt = await this.ports.settings.applyManagedIntegration({
				operationId: state.configurationApplyOperationId,
				registrationId: state.targetRegistrationId,
				storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
				configurationInstanceId: configurationInstanceId(state),
				expectedDesiredRevision: state.settingsUpdateReceipt.desiredRevision,
				requestHostBridge: false,
			})
			const accountStatus = await this.ports.mail.status(connectionId(state))
			return {
				...state,
				phase: 'configuration_applied',
				configurationApplyReceipt: receipt,
				accountStatus,
				lastErrorCode: undefined,
			}
		} catch {
			return blocked(state, 'mail_import_configuration_apply_failed')
		}
	}

	async provisionCredential(
		state: MailAccountImportStateV1,
		kind: MailImportCredentialKindV1,
		secretPayload?: Uint8Array,
	): Promise<MailAccountImportStateV1> {
		const progress = state[kind]
		if (!progress) return blocked(state, 'mail_import_credential_not_required')
		if (!state.configurationApplyReceipt || !state.accountStatus) {
			return blocked(state, 'mail_import_configuration_receipt_required')
		}
		if (progress.bindingReceipt) return state
		let next = state
		let vaultReceipt = progress.vaultReceipt
		try {
			if (!vaultReceipt) {
				if (!secretPayload?.byteLength) {
					return blocked(state, `mail_import_${kind}_secret_required`)
				}
				const current = credentialStatus(state.accountStatus, kind)
				const nextRevision = (current?.credentialRevision ?? 0n) + 1n
				vaultReceipt = await this.ports.vault.provision({
					operationId: progress.operationId,
					targetRegistrationId: state.targetRegistrationId,
					capabilityId: kind === 'imap'
						? MAIL_IMAP_PROVISIONING_CAPABILITY_ID
						: MAIL_SMTP_PROVISIONING_CAPABILITY_ID,
					configurationInstanceId: configurationInstanceId(state),
					purposeId: kind === 'imap' ? 'mail_imap_password' : 'mail_smtp_password',
					secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
					action: current?.credentialRevision
						? OwnerVaultActionV1.REPLACE_CAS
						: OwnerVaultActionV1.CREATE,
					secretRevision: nextRevision,
					secretPayload,
				})
				next = {
					...state,
					[kind]: { ...progress, vaultReceipt },
					lastErrorCode: undefined,
				}
			}
			const current = credentialStatus(state.accountStatus, kind)
			const bindingReceipt = await this.ports.mail.bind({
				connectionId: connectionId(state),
				purpose: credentialPurpose(kind),
				expectedBindingRevision: current?.bindingRevision ?? 0n,
				credentialRevision: vaultReceipt.secretRevision,
			})
			const completed = {
				...next,
				[kind]: { ...next[kind]!, vaultReceipt, bindingReceipt },
				lastErrorCode: undefined,
			}
			return {
				...completed,
				phase: credentialsBound(completed) ? 'credentials_bound' : completed.phase,
			}
		} catch {
			return blocked(next, `mail_import_${kind}_credential_failed`)
		}
	}

	async activateCredentials(
		state: MailAccountImportStateV1,
	): Promise<MailAccountImportStateV1> {
		if (state.activationApplyReceipt) return state
		if (!state.settingsUpdateReceipt || !credentialsBound(state)) {
			return blocked(state, 'mail_import_binding_receipts_required')
		}
		try {
			const receipt = await this.ports.settings.applyManagedIntegration({
				operationId: state.activationApplyOperationId,
				registrationId: state.targetRegistrationId,
				storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
				configurationInstanceId: configurationInstanceId(state),
				expectedDesiredRevision: state.settingsUpdateReceipt.desiredRevision,
				requestHostBridge: false,
			})
			const accountStatus = await this.ports.mail.status(connectionId(state))
			return {
				...state,
				phase: accountStatus.readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
					? 'ready'
					: 'credentials_activated',
				activationApplyReceipt: receipt,
				accountStatus,
				lastErrorCode: undefined,
			}
		} catch {
			return blocked(state, 'mail_import_credential_activation_failed')
		}
	}

	async startGmailOAuth(
		state: MailAccountImportStateV1,
	): Promise<MailAccountImportStateV1> {
		if (!isGmail(state) || !state.configurationApplyReceipt) {
			return blocked(state, 'mail_import_gmail_configuration_required')
		}
		if (state.gmailOAuthStarted) return state
		const operationId = state.gmailOAuthOperationId ?? crypto.randomUUID()
		try {
			const started = await this.ports.gmailOAuth.start(
				operationId,
				connectionId(state),
			)
			return {
				...state,
				phase: 'awaiting_gmail_authorization',
				gmailOAuthOperationId: operationId,
				gmailOAuthStarted: started,
				lastErrorCode: undefined,
			}
		} catch {
			return blocked(
				{ ...state, gmailOAuthOperationId: operationId },
				'mail_import_gmail_oauth_start_failed',
			)
		}
	}

	async completeGmailOAuth(
		state: MailAccountImportStateV1,
		input: { state: string; authorizationCode: string },
	): Promise<MailAccountImportStateV1> {
		if (!state.gmailOAuthStarted || !state.gmailOAuthOperationId) {
			return blocked(state, 'mail_import_gmail_oauth_start_receipt_required')
		}
		try {
			const accepted = await this.ports.gmailOAuth.complete({
				operationId: state.gmailOAuthOperationId,
				connectionId: connectionId(state),
				setupId: state.gmailOAuthStarted.setupId,
				state: input.state,
				authorizationCode: input.authorizationCode,
			})
			return {
				...state,
				phase: 'gmail_completion_accepted',
				gmailOAuthAccepted: accepted,
				lastErrorCode: undefined,
			}
		} catch {
			return blocked(state, 'mail_import_gmail_oauth_complete_failed')
		}
	}

	async reconcile(
		state: MailAccountImportStateV1,
	): Promise<MailAccountImportStateV1> {
		try {
			const [accountStatus, gmailOAuthStatus] = await Promise.all([
				this.ports.mail.status(connectionId(state)),
				state.gmailOAuthOperationId
					? this.ports.gmailOAuth.status(
						state.gmailOAuthOperationId,
						connectionId(state),
					)
					: Promise.resolve(undefined),
			])
			return {
				...state,
				phase: accountStatus.readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
					? 'ready'
					: state.phase,
				accountStatus,
				gmailOAuthStatus,
				lastErrorCode: undefined,
			}
		} catch {
			return blocked(state, 'mail_import_reconciliation_failed')
		}
	}

}

function defaultPorts(): MailAccountPortabilityPortsV1 {
	return {
		settings: new OwnerModuleSettingsClientV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		mail: {
			status: getMailAccountStatus,
			bind: bindMailCredential,
		},
		gmailOAuth: new MailGmailOAuthClientV1(),
	}
}

function connectionId(state: MailAccountImportStateV1): string {
	const value = state.exported.configuration?.connectionId
	if (!value) throw new Error('Mail import connection is unavailable')
	return value
}

function configurationInstanceId(state: MailAccountImportStateV1): string {
	const value = state.configurationTargetReceipt?.configurationInstanceId
	if (!value) throw new Error('Mail import configuration target is unavailable')
	return value
}

function credentialStatus(
	status: MailAccountStatusV1,
	kind: MailImportCredentialKindV1,
) {
	const purpose = credentialPurpose(kind)
	return status.binding.find((binding) => binding.purpose === purpose)
}

function credentialPurpose(
	kind: MailImportCredentialKindV1,
):
	| MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD
	| MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD {
	return kind === 'imap'
		? MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD
		: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_SMTP_PASSWORD
}

function credentialsBound(state: MailAccountImportStateV1): boolean {
	return (!state.imap || Boolean(state.imap.bindingReceipt))
		&& (!state.smtp || Boolean(state.smtp.bindingReceipt))
}

function isGmail(state: MailAccountImportStateV1): boolean {
	return state.exported.configuration?.inbound.case === 'gmail'
}

function blocked(
	state: MailAccountImportStateV1,
	lastErrorCode: string,
): MailAccountImportStateV1 {
	return { ...state, lastErrorCode }
}
