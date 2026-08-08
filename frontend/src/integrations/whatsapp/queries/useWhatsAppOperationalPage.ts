import { computed, ref } from 'vue'
import type { WhatsAppCommandOperationStatusV1 } from '../../../gen/makosh/whatsapp/v1/client_pb'
import {
	getWhatsAppOperationStatus,
	sendWhatsAppText,
} from '../api/whatsappOperationalGateway'
import {
	buildWhatsAppOperationStatusCard,
	type WhatsAppOperationalPageModel,
} from '../presentation/whatsAppOperationalPageModel'

export function useWhatsAppOperationalPage(canSend: () => boolean) {
	const accountId = ref('')
	const providerChatId = ref('')
	const draft = ref('')
	const operationId = ref('')
	const busy = ref(false)
	const notice = ref('')
	const status = ref<WhatsAppCommandOperationStatusV1 | null>(null)

	const model = computed<WhatsAppOperationalPageModel>(() => ({
		accountId: accountId.value,
		providerChatId: providerChatId.value,
		draft: draft.value,
		operationId: operationId.value,
		busy: busy.value,
		canSend: canSend(),
		notice: notice.value,
		status: buildWhatsAppOperationStatusCard(status.value),
	}))

	async function send(): Promise<void> {
		if (!canSend()) {
			notice.value = 'WhatsApp command capability is not admitted.'
			return
		}
		busy.value = true
		notice.value = ''
		try {
			const accepted = await sendWhatsAppText({
				accountId: accountId.value,
				providerChatId: providerChatId.value,
				text: draft.value,
				operationId: crypto.randomUUID(),
			})
			operationId.value = accepted.operationId
			draft.value = ''
			notice.value = `Operation ${accepted.operationId} accepted by ${accepted.contractName}.`
			await refreshStatus()
		} catch (error) {
			notice.value = messageFrom(error, 'WhatsApp command failed.')
		} finally {
			busy.value = false
		}
	}

	async function refreshStatus(): Promise<void> {
		busy.value = true
		notice.value = ''
		try {
			status.value = await getWhatsAppOperationStatus(operationId.value)
			notice.value = status.value
				? ''
				: 'No WhatsApp operation was found for this ID.'
		} catch (error) {
			notice.value = messageFrom(error, 'WhatsApp status is unavailable.')
		} finally {
			busy.value = false
		}
	}

	function updateAccountId(value: string): void { accountId.value = value }
	function updateProviderChatId(value: string): void { providerChatId.value = value }
	function updateDraft(value: string): void { draft.value = value }
	function updateOperationId(value: string): void { operationId.value = value }

	return {
		model,
		send,
		refreshStatus,
		updateAccountId,
		updateProviderChatId,
		updateDraft,
		updateOperationId,
	}
}

function messageFrom(error: unknown, fallback: string): string {
	return error instanceof Error ? error.message : fallback
}
