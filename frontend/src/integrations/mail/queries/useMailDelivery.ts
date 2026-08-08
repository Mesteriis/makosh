import { computed, ref } from 'vue'
import type { MailDeliveryOperationStatusV1 } from '../../../gen/makosh/mail/v1/client_pb'
import {
	getMailDeliveryStatus,
	sendMailMessage,
} from '../api/mailOperationalGateway'
import {
	buildMailDeliveryStatusCard,
	type MailDeliveryModel,
} from '../presentation/mailDeliveryModel'
import type { MailDeliveryInput } from './useMailComposition'

export function useMailDelivery(capabilities: {
	canDeliver: () => boolean
	connectionId: () => string
}) {
	const operationId = ref('')
	const busy = ref(false)
	const notice = ref('')
	const status = ref<MailDeliveryOperationStatusV1 | null>(null)

	const model = computed<MailDeliveryModel>(() => ({
		operationId: operationId.value,
		busy: busy.value,
		canDeliver: capabilities.canDeliver() && Boolean(capabilities.connectionId()),
		notice: notice.value,
		status: buildMailDeliveryStatusCard(status.value),
	}))

	async function deliver(input: MailDeliveryInput): Promise<void> {
		if (!capabilities.canDeliver()) {
			notice.value = 'Mail delivery capability is not admitted.'
			return
		}
		await run(async () => {
			operationId.value = await sendMailMessage({
				connectionId: input.connectionId,
				operationId: crypto.randomUUID(),
				providerConversationId: input.providerConversationId,
				toRecipients: input.toRecipients,
				ccRecipients: input.ccRecipients,
				bccRecipients: input.bccRecipients,
				subject: input.subject,
				textBody: input.textBody,
			})
			notice.value = `Mail operation ${operationId.value} accepted.`
		})
	}

	async function refreshStatus(): Promise<void> {
		await run(async () => {
			status.value = await getMailDeliveryStatus(
				capabilities.connectionId(),
				operationId.value,
			)
			if (!status.value) notice.value = 'No Mail delivery was found for this operation ID.'
		})
	}

	async function run(work: () => Promise<void>): Promise<void> {
		busy.value = true
		notice.value = ''
		try {
			await work()
		} catch (error) {
			notice.value = error instanceof Error ? error.message : 'Mail delivery failed.'
		} finally {
			busy.value = false
		}
	}

	return {
		model,
		deliver,
		refreshStatus,
		updateOperationId: (value: string) => { operationId.value = value },
	}
}
