import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { hasOwnerVaultProvisioningHostV1 } from '../../../platform/vault'
import { ZulipAccountSetupWorkflowV1 } from './zulipAccountSetupWorkflow'

export function useZulipAccountSetup(
	module: () => ClientModuleBootstrapV1 | null,
	workflow = new ZulipAccountSetupWorkflowV1(),
) {
	const accountId = ref('')
	const realmUrl = ref('')
	const accountEmail = ref('')
	const apiKey = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const secureHostAvailable = hasOwnerVaultProvisioningHostV1()
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canSubmit = computed(() => Boolean(
		module()?.settings
		&& accountId.value.trim()
		&& realmUrl.value.trim()
		&& accountEmail.value.trim()
		&& apiKey.value,
	))

	async function submit(): Promise<void> {
		const current = module()
		if (!current?.settings || !canSubmit.value) return
		if (!secureHostAvailable) {
			message.value = 'Use the desktop shell or root make dev to seal the Zulip API key.'
			messageTone.value = 'neutral'
			return
		}
		busy.value = true
		message.value = ''
		try {
			await workflow.setup({
				registrationId: current.registrationId,
				expectedDesiredRevision: current.settings.desiredRevision,
				accountId: accountId.value,
				realmUrl: realmUrl.value,
				accountEmail: accountEmail.value,
				apiKey: new TextEncoder().encode(apiKey.value),
			})
			apiKey.value = ''
			message.value = 'Zulip account configured and credential binding activated.'
			messageTone.value = 'success'
		} catch {
			apiKey.value = ''
			message.value = 'Zulip setup failed before readiness. No secret was written to Settings.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	return {
		accountId,
		realmUrl,
		accountEmail,
		apiKey,
		busy,
		message,
		messageTone,
		configured,
		canSubmit,
		secureHostAvailable,
		submit,
	}
}
