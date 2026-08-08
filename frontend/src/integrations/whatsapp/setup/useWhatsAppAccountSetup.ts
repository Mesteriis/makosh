import { computed, ref } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { WhatsAppAccountSetupWorkflowV1 } from './whatsAppAccountSetupWorkflow'

export function useWhatsAppAccountSetup(
	module: () => ClientModuleBootstrapV1 | null,
	workflow = new WhatsAppAccountSetupWorkflowV1(),
) {
	const accountId = ref('')
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const configured = computed(() => (module()?.settings?.effectiveRevision ?? 0n) > 0n)
	const canSubmit = computed(() => Boolean(module()?.settings && accountId.value.trim()))

	async function submit(): Promise<void> {
		const current = module()
		if (!current?.settings || !canSubmit.value) return
		busy.value = true
		message.value = ''
		try {
			await workflow.setup({
				registrationId: current.registrationId,
				expectedDesiredRevision: current.settings.desiredRevision,
				accountId: accountId.value,
			})
			message.value = 'WhatsApp account profile configured. Continue in the separate QR pairing panel.'
			messageTone.value = 'success'
		} catch {
			message.value = 'WhatsApp account setup failed. The admitted runtime or host bridge is unavailable.'
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	return {
		accountId,
		busy,
		message,
		messageTone,
		configured,
		canSubmit,
		submit,
	}
}
