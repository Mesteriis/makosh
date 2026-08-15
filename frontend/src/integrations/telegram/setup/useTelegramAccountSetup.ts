import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	DevelopmentOwnerVaultProvisioningHostV1,
	hasDevelopmentOwnerVaultProvisioningHostV1,
	hasOwnerVaultProvisioningHostV1,
} from '../../../platform/vault'
import { TelegramAccountSetupWorkflowV1 } from './telegramAccountSetupWorkflow'

export function useTelegramAccountSetup(
	module: () => ClientModuleBootstrapV1 | null,
	workflow = new TelegramAccountSetupWorkflowV1(),
) {
	const accountId = ref('')
	const displayName = ref('')
	const apiId = ref('')
	const apiHash = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasOwnerVaultProvisioningHostV1()
	const developmentHost = hasDevelopmentOwnerVaultProvisioningHostV1()
		? new DevelopmentOwnerVaultProvisioningHostV1()
		: null
	const developmentCredentialsAvailable = ref(false)
	const provisionedLocally = ref(false)
	let developmentCredentialsRequest: Promise<boolean> | null = null
	const configured = computed(() => isTelegramSetupConfigured(
		module()?.settings?.effectiveRevision ?? 0n,
		provisionedLocally.value,
	))
	const canSubmit = computed(() => Boolean(
		module()?.settings
		&& accountId.value.trim()
		&& displayName.value.trim()
		&& apiId.value.trim()
		&& (apiHash.value || developmentCredentialsAvailable.value),
	))

	async function prepareDevelopmentCredentials(): Promise<boolean> {
		if (!developmentHost) return false
		if (developmentCredentialsAvailable.value) return true
		if (developmentCredentialsRequest) return developmentCredentialsRequest
		developmentCredentialsRequest = developmentHost.telegramCredentials()
			.then((credentials) => {
				apiId.value = credentials.apiId.toString()
				if (!accountId.value) accountId.value = 'personal-telegram'
				if (!displayName.value) displayName.value = 'Personal Telegram'
				developmentCredentialsAvailable.value = true
				return true
			})
			.catch(() => false)
			.finally(() => {
				developmentCredentialsRequest = null
			})
		return developmentCredentialsRequest
	}

	async function submit(): Promise<boolean> {
		const current = module()
		if (!current?.settings || !canSubmit.value) return false
		if (!secureHostAvailable) {
			message.value = 'Use the desktop shell or root make dev to seal the Telegram API hash and session key.'
			messageTone.value = 'neutral'
			return false
		}
		busy.value = true
		message.value = ''
		try {
			const replaceExistingCredentials = shouldReplaceTelegramCredentials(
				current.settings.effectiveRevision,
			)
			const common = {
				registrationId: current.registrationId,
				expectedDesiredRevision: current.settings.desiredRevision,
				accountId: accountId.value,
				displayName: displayName.value,
				apiId: BigInt(apiId.value),
				replaceExistingCredentials,
			}
			if (developmentCredentialsAvailable.value && developmentHost) {
				await workflow.setup({
					...common,
					apiHashSealer: (input) => developmentHost.sealTelegramApiHash(input),
				})
			} else {
				await workflow.setup({
					...common,
					apiHash: new TextEncoder().encode(apiHash.value),
				})
			}
			apiHash.value = ''
			provisionedLocally.value = true
			message.value = 'Telegram user account saved. Preparing the provider-issued QR code.'
			messageTone.value = 'success'
			return true
		} catch {
			apiHash.value = ''
			message.value = 'Telegram setup failed before provider authorization. No secret was written to Settings.'
			messageTone.value = 'error'
			return false
		} finally {
			busy.value = false
		}
	}

	return {
		accountId,
		displayName,
		apiId,
		apiHash,
		busy,
		message,
		messageTone,
		configured,
		canSubmit,
		secureHostAvailable,
		developmentCredentialsAvailable,
		prepareDevelopmentCredentials,
		submit,
	}
}

export function shouldReplaceTelegramCredentials(effectiveSettingsRevision: bigint): boolean {
	return effectiveSettingsRevision > 0n
}

export function isTelegramSetupConfigured(
	effectiveSettingsRevision: bigint,
	provisionedLocally: boolean,
): boolean {
	return effectiveSettingsRevision > 0n || provisionedLocally
}
