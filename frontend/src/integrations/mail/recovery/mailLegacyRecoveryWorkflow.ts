import {
	ClientSettingsApplyStateV1,
	type ClientModuleSettingsTargetBootstrapV1,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailAccountReadinessV1,
	MailCredentialBindingStateV1,
	MailCredentialPurposeV1,
	type MailAccountStatusV1,
	type MailCredentialBindingReceiptV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import type {
	GmailOAuthStartedV1,
	MailAcceptedV1,
} from '../../../gen/makosh/mail/v1/client_pb'
import {
	createLegacyProviderRecoveryHostV1,
	legacyProviderRecoveryOperationKeyFromStepV1,
	type LegacyProviderRecoveryCandidateV1,
	type LegacyProviderRecoveryHostV1,
	type LegacyProviderRecoveryPlanV1,
	LegacyProviderRecoveryStepJournalV1,
} from '../../../platform/legacy-recovery'
import { ManagedIntegrationSetupV1 } from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
} from '../../../platform/vault'
import { bindMailCredential } from '../api/mailCredentialBindingClient'
import { getMailAccountStatus } from '../api/mailAccountQueryClient'
import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'
import {
	mailGmailPreauthorizationSettings,
	mailImapSettings,
} from '../setup/mailAccountSetupWorkflow'

const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailLegacyRecoveryPortsV1 = {
	source: LegacyProviderRecoveryHostV1
	configuration: Pick<ManagedIntegrationSetupV1, 'createTarget' | 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provisionCustodied'>
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

export type MailLegacyRecoveryResultV1 =
	| {
		kind: 'gmail'
		state: 'reauthorization_required'
		oauth: {
			operationId: string
			connectionId: string
			started: GmailOAuthStartedV1
		}
	}
	| {
		kind: 'icloud'
		state: 'ready' | 'applied_pending_readiness'
	}

export class MailLegacyRecoveryWorkflowV1 {
	constructor(private readonly ports: MailLegacyRecoveryPortsV1 = defaultPorts()) {}

	async recover(input: {
		registrationId: string
		plan: LegacyProviderRecoveryPlanV1
		candidate: LegacyProviderRecoveryCandidateV1
		settingsTargets?: readonly ClientModuleSettingsTargetBootstrapV1[]
		explicitRetryOutcomeUnknown?: boolean
	}): Promise<MailLegacyRecoveryResultV1> {
		if (input.candidate.kind !== 'gmail' && input.candidate.kind !== 'icloud') {
			throw new Error('mail legacy recovery candidate is invalid')
		}
		const source = await this.ports.source.source(
			input.plan.recoverySessionId,
			input.candidate.sourceHandle,
		)
		if (source.kind !== input.candidate.kind
			|| source.sourceHandle !== input.candidate.sourceHandle) {
			throw new Error('mail legacy recovery source is invalid')
		}
		const journal = new LegacyProviderRecoveryStepJournalV1(
			this.ports.source,
			input.plan.recoverySessionId,
			input.candidate.sourceHandle,
			input.explicitRetryOutcomeUnknown === true,
		)
		const createStepId = source.kind === 'gmail'
			? 'mail_gmail_create_target'
			: 'mail_icloud_create_target'
		const createStep = await journal.begin(createStepId)
		const existingTarget = matchingMailTarget(input.settingsTargets ?? [], source.accountId)
		const target = existingTarget ?? await this.ports.configuration.createTarget(
			input.registrationId,
			createStep.operationId,
		)
		await journal.complete(createStepId, createStep, {
			targetConfigurationInstanceId: target.configurationInstanceId,
			publicRevision: target.desiredRevision,
		})
		let settingsRevision = target.desiredRevision
		const updateStepId = source.kind === 'gmail'
			? 'mail_gmail_update_settings'
			: 'mail_icloud_update_settings'
		const applyStepId = source.kind === 'gmail'
			? 'mail_gmail_apply_settings'
			: 'mail_icloud_apply_settings'
		const settingsStep = target.applyState === 'current'
			? journal.inspect.bind(journal)
			: journal.begin.bind(journal)
		const updateStep = await settingsStep(
			updateStepId,
			target.configurationInstanceId,
		)
		const applyStep = await settingsStep(
			applyStepId,
			target.configurationInstanceId,
		)
		if (target.applyState === 'blocked_config') {
			if (updateStep.disposition === 'completed'
				|| applyStep.disposition === 'completed') {
				throw new Error('mail legacy recovery receipt contradicts Settings state')
			}
			const applied = await this.ports.configuration.apply({
				registrationId: input.registrationId,
				expectedDesiredRevision: target.desiredRevision,
				storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
				configurationInstanceId: target.configurationInstanceId,
				requestHostBridge: false,
				values: source.kind === 'gmail'
					? mailGmailPreauthorizationSettings({
						connectionId: source.accountId,
						clientId: source.oauthClientId,
						redirectUri: source.oauthRedirectUri,
					})
					: mailImapSettings({
						registrationId: input.registrationId,
						expectedDesiredRevision: target.desiredRevision,
						connectionId: source.accountId,
						imapHost: source.imapHost,
						imapPort: BigInt(source.imapPort),
						username: source.username,
					}),
				updateOperationId: updateStep.operationId,
				applyOperationId: applyStep.operationId,
			})
			settingsRevision = applied.settings.desiredRevision
		} else if (target.applyState !== 'current') {
			throw new Error('mail legacy recovery settings outcome is ambiguous')
		}
		await journal.complete(updateStepId, updateStep, {
			targetConfigurationInstanceId: target.configurationInstanceId,
			publicRevision: settingsRevision,
		})
		await journal.complete(applyStepId, applyStep, {
			targetConfigurationInstanceId: target.configurationInstanceId,
			publicRevision: settingsRevision,
		})
		if (source.kind === 'gmail') {
			const oauthStepId =
				`mail_gmail_oauth_start_revision_${settingsRevision}` as const
			const oauthStep = await journal.begin(
				oauthStepId,
				target.configurationInstanceId,
			)
			const operationId = legacyProviderRecoveryOperationKeyFromStepV1(oauthStep)
			const started = await this.ports.oauth.start(operationId, source.accountId)
			await journal.complete(oauthStepId, oauthStep, {
				targetConfigurationInstanceId: target.configurationInstanceId,
				publicRevision: settingsRevision,
			})
			await journal.finish(
				target.configurationInstanceId,
				'reauthorization_required',
			)
			return {
				kind: 'gmail',
				state: 'reauthorization_required',
				oauth: { operationId, connectionId: source.accountId, started },
			}
		}

		const current = await this.ports.mail.status(source.accountId)
		const binding = current.binding.find(
			(entry) => entry.purpose
				=== MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
		)
		const provisionStepId = 'mail_icloud_provision_imap_password'
		const provisionStep = await journal.begin(
			provisionStepId,
			target.configurationInstanceId,
		)
		const needsCredentialRepair = binding
			? binding.state
				!== MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_ACTIVE
			: !provisionStep.publicRevision
		const needsBindingRepair = !binding
			|| binding.state
				!== MailCredentialBindingStateV1.MAIL_CREDENTIAL_BINDING_STATE_ACTIVE
		let credentialRevision = binding?.credentialRevision
			?? provisionStep.publicRevision
		if (needsCredentialRepair) {
			const vault = await this.ports.vault.provisionCustodied({
				operationId: provisionStep.operationId,
				targetRegistrationId: input.registrationId,
				capabilityId: 'mail.imap.credential-provisioning.v1',
				configurationInstanceId: target.configurationInstanceId,
				purposeId: 'mail_imap_password',
				secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
				action: OwnerVaultActionV1.CREATE,
				secretRevision: credentialRevision ?? 1n,
			}, (authorized) => this.ports.source.sealSource({
				...authorized,
				recoverySessionId: input.plan.recoverySessionId,
				sourceHandle: input.candidate.sourceHandle,
				secretPurpose: 'icloud_imap_password',
			}))
			credentialRevision = vault.secretRevision
		}
		if (!credentialRevision) {
			throw new Error('mail legacy recovery credential revision is unavailable')
		}
		await journal.complete(provisionStepId, provisionStep, {
			targetConfigurationInstanceId: target.configurationInstanceId,
			publicRevision: credentialRevision,
		})
		const bindStepId = 'mail_icloud_bind_imap_password'
		const bindStep = needsBindingRepair
			? await journal.begin(bindStepId, target.configurationInstanceId)
			: await journal.inspect(bindStepId, target.configurationInstanceId)
		if (needsBindingRepair) {
			await this.ports.mail.bind({
				connectionId: source.accountId,
				purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
				expectedBindingRevision: binding?.bindingRevision ?? 0n,
				credentialRevision,
			})
		}
		await journal.complete(bindStepId, bindStep, {
			targetConfigurationInstanceId: target.configurationInstanceId,
			publicRevision: credentialRevision,
		})
		const status = await this.ports.mail.status(source.accountId)
		const result: MailLegacyRecoveryResultV1 = {
			kind: 'icloud',
			state: status.readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
				? 'ready'
				: 'applied_pending_readiness',
		}
		await journal.finish(
			target.configurationInstanceId,
			result.state === 'ready' ? 'completed' : 'blocked_config',
		)
		return result
	}

	async completeGmail(
		result: Extract<MailLegacyRecoveryResultV1, { kind: 'gmail' }>,
		input: { returnedState: string; authorizationCode: string },
	): Promise<MailAcceptedV1> {
		return this.ports.oauth.complete({
			operationId: result.oauth.operationId,
			connectionId: result.oauth.connectionId,
			setupId: result.oauth.started.setupId,
			state: required(input.returnedState),
			authorizationCode: required(input.authorizationCode),
		})
	}
}

function matchingMailTarget(
	targets: readonly ClientModuleSettingsTargetBootstrapV1[],
	connectionId: string,
): { configurationInstanceId: string; desiredRevision: bigint; applyState: string } | undefined {
	const matches = targets.filter((target) => target.values.some((entry) =>
		entry.settingId === 'mail.connection_id'
		&& entry.value?.value.case === 'stringValue'
		&& entry.value.value.value === connectionId,
	))
	if (matches.length > 1) {
		throw new Error('mail legacy recovery target is ambiguous')
	}
	const target = matches[0]
	if (!target) return undefined
	if (target.applyState !== ClientSettingsApplyStateV1.CURRENT
		&& target.applyState !== ClientSettingsApplyStateV1.BLOCKED_CONFIG) {
		throw new Error('mail legacy recovery settings outcome is ambiguous')
	}
	return {
		configurationInstanceId: target.configurationInstanceId,
		desiredRevision: target.desiredRevision,
		applyState: target.applyState === ClientSettingsApplyStateV1.CURRENT
			? 'current'
			: 'blocked_config',
	}
}

function defaultPorts(): MailLegacyRecoveryPortsV1 {
	return {
		source: createLegacyProviderRecoveryHostV1(),
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		mail: { status: getMailAccountStatus, bind: bindMailCredential },
		oauth: new MailGmailOAuthClientV1(),
	}
}

function required(value: string): string {
	const normalized = value.trim()
	if (!normalized) throw new Error('Gmail OAuth completion input is required')
	return normalized
}
