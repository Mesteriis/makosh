import { computed, ref } from 'vue'

import {
	inspectTelegramMessage,
} from '../api/telegramMessageInspectorGateway'
import type { TelegramMessageInspection } from '../api/telegramMessageInspectorGateway'
import {
	buildTelegramMessageInspectionView,
} from '../presentation/telegramMessageInspectorModel'
import type { TelegramMessageInspectorModel } from '../presentation/telegramMessageInspectorModel'

export function useTelegramMessageInspector(input: {
	accountId: () => string
	canQuery: () => boolean
	messageId: () => string
	providerChatId: () => string
	providerMessageId: () => string
	senderPersonaNames?: () => ReadonlyMap<string, string>
}) {
	const inspection = ref<TelegramMessageInspection | null>(null)
	const pending = ref(false)
	const statusMessage = ref('')

	const model = computed<TelegramMessageInspectorModel>(() => ({
		selectedMessageId: input.providerMessageId(),
		pending: pending.value,
		statusMessage: statusMessage.value,
		canQuery: input.canQuery(),
		...buildTelegramMessageInspectionView(
			inspection.value,
			input.senderPersonaNames?.(),
		),
	}))

	async function inspect(): Promise<void> {
		if (!input.canQuery()) {
			statusMessage.value = 'Telegram query capability is not admitted.'
			return
		}
		pending.value = true
		statusMessage.value = ''
		try {
			inspection.value = await inspectTelegramMessage({
				accountId: input.accountId(),
				providerChatId: input.providerChatId(),
				messageId: input.messageId(),
				providerMessageId: input.providerMessageId(),
			})
			statusMessage.value = 'Telegram message inspection loaded.'
		} catch (error) {
			statusMessage.value = error instanceof Error
				? error.message
				: 'Telegram message inspection failed.'
		} finally {
			pending.value = false
		}
	}

	return { model, inspect }
}
