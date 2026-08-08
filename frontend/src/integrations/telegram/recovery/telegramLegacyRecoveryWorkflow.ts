import type { TelegramAccountResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import {
	createLegacyProviderRecoveryHostV1,
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
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import {
	listTelegramAccounts,
	provisionTelegramAccount,
} from '../api/telegramLifecycleGateway'
import { withTelegramConfigurationRuntimeV1 } from '../setup/telegramConfigurationRuntimeRetry'

const TELEGRAM_STORAGE_CAPABILITY_ID = 'telegram.storage.v1'
const API_HASH_PROVISIONING_CAPABILITY_ID =
	'telegram.api-hash.credential-provisioning.v1'
const SESSION_KEY_PROVISIONING_CAPABILITY_ID =
	'telegram.session-store-key.credential-provisioning.v1'
const RECOVERY_CREDENTIAL_REVISION = 1n

type TelegramLegacyRecoveryPortsV1 = {
	source: LegacyProviderRecoveryHostV1
	configuration: Pick<ManagedIntegrationSetupV1, 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provisionCustodied'>
	lifecycle: {
		list(): Promise<readonly TelegramAccountResponse[]>
		provision(input: {
			accountId: string
			displayName: string
			externalAccountId: string
			credentials: readonly { purpose: string; revision: bigint }[]
		}): Promise<TelegramAccountResponse>
	}
}

export type TelegramLegacyRecoveryResultV1 = {
	state: 'qr_authorization_required'
	apiHash?: SanitizedProvisioningHostReceiptV1
	sessionKey?: SanitizedProvisioningHostReceiptV1
}

export class TelegramLegacyRecoveryWorkflowV1 {
	constructor(private readonly ports: TelegramLegacyRecoveryPortsV1 = defaultPorts()) {}

	async recover(input: {
		registrationId: string
		expectedDesiredRevision: bigint
		plan: LegacyProviderRecoveryPlanV1
		candidate: LegacyProviderRecoveryCandidateV1
		explicitRetryOutcomeUnknown?: boolean
	}): Promise<TelegramLegacyRecoveryResultV1> {
		if (input.candidate.kind !== 'telegram_user') {
			throw new Error('Telegram legacy recovery candidate is invalid')
		}
		const source = await this.ports.source.source(
			input.plan.recoverySessionId,
			input.candidate.sourceHandle,
		)
		if (source.kind !== 'telegram_user'
			|| source.sourceHandle !== input.candidate.sourceHandle) {
			throw new Error('Telegram legacy recovery source is invalid')
		}
		const journal = new LegacyProviderRecoveryStepJournalV1(
			this.ports.source,
			input.plan.recoverySessionId,
			input.candidate.sourceHandle,
			input.explicitRetryOutcomeUnknown === true,
		)
		const configurationInstanceId = input.registrationId
		const nextDesiredRevision = input.expectedDesiredRevision + 1n
		const existing = await withTelegramConfigurationRuntimeV1(
			() => this.ports.lifecycle.list(),
		)
		const existingAccount = existing.find(
			(account) => account.accountId === source.accountId,
		)
		const updateStepId =
			`telegram_update_settings_revision_${input.expectedDesiredRevision}` as const
		const applyStepId =
			`telegram_apply_settings_revision_${nextDesiredRevision}` as const
		const updateStep = existingAccount
			? await journal.inspect(updateStepId, configurationInstanceId)
			: await journal.begin(updateStepId, configurationInstanceId)
		const applyStep = existingAccount
			? await journal.inspect(applyStepId, configurationInstanceId)
			: await journal.begin(applyStepId, configurationInstanceId)
		if (existingAccount) {
			await journal.complete(updateStepId, updateStep, {
				targetConfigurationInstanceId: configurationInstanceId,
				publicRevision: input.expectedDesiredRevision,
			})
			await journal.complete(applyStepId, applyStep, {
				targetConfigurationInstanceId: configurationInstanceId,
				publicRevision: input.expectedDesiredRevision,
			})
			for (const stepIdentifier of [
				'telegram_provision_api_hash',
				'telegram_provision_session_store_key',
				'telegram_provision_account',
			] as const) {
				const step = await journal.inspect(stepIdentifier, configurationInstanceId)
				await journal.complete(stepIdentifier, step, {
					targetConfigurationInstanceId: configurationInstanceId,
					publicRevision: stepIdentifier === 'telegram_provision_account'
						? existingAccount.runtimeEpoch
						: RECOVERY_CREDENTIAL_REVISION,
				})
			}
			await journal.finish(configurationInstanceId, 'qr_authorization_required')
			return { state: 'qr_authorization_required' }
		}
		if ((updateStep.disposition === 'completed')
			!== (applyStep.disposition === 'completed')) {
			throw new Error('Telegram recovery Settings receipt is inconsistent')
		}
		if (updateStep.disposition !== 'completed') {
			await this.ports.configuration.apply({
				registrationId: input.registrationId,
				expectedDesiredRevision: input.expectedDesiredRevision,
				storageCapabilityId: TELEGRAM_STORAGE_CAPABILITY_ID,
				configurationInstanceId,
				requestHostBridge: false,
				values: [
					{
						settingId: 'telegram.account_id',
						value: { case: 'stringValue', value: source.accountId },
					},
					{
						settingId: 'telegram.api_id',
						value: { case: 'signedIntegerValue', value: source.apiId },
					},
				],
				updateOperationId: updateStep.operationId,
				applyOperationId: applyStep.operationId,
			})
		}
		await journal.complete(updateStepId, updateStep, {
			targetConfigurationInstanceId: configurationInstanceId,
			publicRevision: nextDesiredRevision,
		})
		await journal.complete(applyStepId, applyStep, {
			targetConfigurationInstanceId: configurationInstanceId,
			publicRevision: nextDesiredRevision,
		})
		const apiHashStepId = 'telegram_provision_api_hash'
		const apiHashStep = await journal.begin(apiHashStepId, configurationInstanceId)
		if (apiHashStep.disposition === 'completed' && !apiHashStep.publicRevision) {
			throw new Error('Telegram API hash receipt revision is unavailable')
		}
		let apiHash: SanitizedProvisioningHostReceiptV1 | undefined
		const apiHashRevision = apiHashStep.disposition === 'completed'
			? apiHashStep.publicRevision!
			: (apiHash = await this.ports.vault.provisionCustodied({
				operationId: apiHashStep.operationId,
				targetRegistrationId: input.registrationId,
				capabilityId: API_HASH_PROVISIONING_CAPABILITY_ID,
				configurationInstanceId,
				purposeId: 'telegram_api_hash',
				secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
				action: OwnerVaultActionV1.CREATE,
				secretRevision: 1n,
			}, (authorized) => this.ports.source.sealSource({
				...authorized,
				recoverySessionId: input.plan.recoverySessionId,
				sourceHandle: input.candidate.sourceHandle,
				secretPurpose: 'telegram_api_hash',
			}))).secretRevision
		await journal.complete(apiHashStepId, apiHashStep, {
			targetConfigurationInstanceId: configurationInstanceId,
			publicRevision: apiHashRevision,
		})
		const sessionKeyStepId = 'telegram_provision_session_store_key'
		const sessionKeyStep = await journal.begin(
			sessionKeyStepId,
			configurationInstanceId,
		)
		if (sessionKeyStep.disposition === 'completed' && !sessionKeyStep.publicRevision) {
			throw new Error('Telegram session key receipt revision is unavailable')
		}
		let sessionKey: SanitizedProvisioningHostReceiptV1 | undefined
		const sessionKeyRevision = sessionKeyStep.disposition === 'completed'
			? sessionKeyStep.publicRevision!
			: (sessionKey = await this.ports.vault.provisionCustodied({
				operationId: sessionKeyStep.operationId,
				targetRegistrationId: input.registrationId,
				capabilityId: SESSION_KEY_PROVISIONING_CAPABILITY_ID,
				configurationInstanceId,
				purposeId: 'telegram_session_store_key',
				secretClass: OwnerVaultSecretClassV1.SESSION_STORE_KEY,
				action: OwnerVaultActionV1.CREATE,
				secretRevision: 1n,
			}, (authorized) => this.ports.source.sealSource({
				...authorized,
				recoverySessionId: input.plan.recoverySessionId,
				sourceHandle: input.candidate.sourceHandle,
				secretPurpose: 'generated_telegram_session_store_key',
			}))).secretRevision
		await journal.complete(sessionKeyStepId, sessionKeyStep, {
			targetConfigurationInstanceId: configurationInstanceId,
			publicRevision: sessionKeyRevision,
		})
		const provisionStepId = 'telegram_provision_account'
		const provisionStep = await journal.begin(provisionStepId, configurationInstanceId)
		const account = await withTelegramConfigurationRuntimeV1(() =>
			this.ports.lifecycle.provision({
			accountId: source.accountId,
			displayName: source.displayName,
			externalAccountId: source.externalAccountId,
			credentials: credentials(apiHashRevision, sessionKeyRevision),
			}),
		)
		await journal.complete(provisionStepId, provisionStep, {
			targetConfigurationInstanceId: configurationInstanceId,
			publicRevision: account.runtimeEpoch,
		})
		await journal.finish(configurationInstanceId, 'qr_authorization_required')
		return {
			state: 'qr_authorization_required',
			...(apiHash ? { apiHash } : {}),
			...(sessionKey ? { sessionKey } : {}),
		}
	}
}

function credentials(
	apiHashRevision: bigint,
	sessionKeyRevision: bigint,
): readonly { purpose: string; revision: bigint }[] {
	return [
		{ purpose: 'telegram_api_hash', revision: apiHashRevision },
		{ purpose: 'telegram_session_encryption_key', revision: sessionKeyRevision },
	]
}

function defaultPorts(): TelegramLegacyRecoveryPortsV1 {
	return {
		source: createLegacyProviderRecoveryHostV1(),
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		lifecycle: {
			list: listTelegramAccounts,
			provision: provisionTelegramAccount,
		},
	}
}
