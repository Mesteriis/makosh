import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { hasOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
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
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canSubmit = computed(() => Boolean(
		module()?.settings
		&& accountId.value.trim()
		&& displayName.value.trim()
		&& apiId.value.trim()
		&& apiHash.value,
	))

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
			await workflow.setup({
				registrationId: current.registrationId,
				expectedDesiredRevision: current.settings.desiredRevision,
				accountId: accountId.value,
				displayName: displayName.value,
				apiId: BigInt(apiId.value),
				apiHash: new TextEncoder().encode(apiHash.value),
			})
			apiHash.value = ''
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
		submit,
	}
}
