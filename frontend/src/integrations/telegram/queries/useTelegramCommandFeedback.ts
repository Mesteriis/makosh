import { ref } from 'vue'

import type { TelegramOperationResponse } from '../../../gen/makosh/telegram/v1/client_pb'

export function useTelegramCommandFeedback(canCommand: () => boolean) {
	const pending = ref(false)
	const statusMessage = ref('')

	async function run(action: () => Promise<TelegramOperationResponse>): Promise<void> {
		if (!canCommand()) {
			statusMessage.value = 'Telegram command capability is not admitted.'
			return
		}
		pending.value = true
		statusMessage.value = ''
		try {
			const operation = await action()
			statusMessage.value = `Operation ${operation.operationId} is ${operation.state || 'accepted'}.`
		} catch (error) {
			statusMessage.value = error instanceof Error ? error.message : 'Telegram command failed.'
		} finally {
			pending.value = false
		}
	}

	return { pending, statusMessage, run }
}
