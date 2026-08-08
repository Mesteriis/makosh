import { computed, ref } from 'vue'
import type { ZulipCommandOperationStatusV1 } from '../../../gen/makosh/zulip/v1/client_pb'
import {
	getZulipOperationStatus,
	sendZulipDirectMessage,
	sendZulipStreamMessage,
} from '../api/zulipOperationalGateway'
import {
	buildZulipOperationStatusCard,
	type ZulipDestination,
	type ZulipOperationalPageModel,
} from '../presentation/zulipOperationalPageModel'

export function useZulipOperationalPage(canCommand: () => boolean) {
	const destination = ref<ZulipDestination>('stream')
	const accountId = ref('')
	const stream = ref('')
	const topic = ref('')
	const recipients = ref('')
	const content = ref('')
	const operationId = ref('')
	const busy = ref(false)
	const notice = ref('')
	const status = ref<ZulipCommandOperationStatusV1 | null>(null)

	const model = computed<ZulipOperationalPageModel>(() => ({
		destination: destination.value,
		accountId: accountId.value,
		stream: stream.value,
		topic: topic.value,
		recipients: recipients.value,
		content: content.value,
		operationId: operationId.value,
		busy: busy.value,
		canCommand: canCommand(),
		notice: notice.value,
		status: buildZulipOperationStatusCard(status.value),
	}))

	async function send(): Promise<void> {
		if (!canCommand()) {
			notice.value = 'Zulip command capability is not admitted.'
			return
		}
		busy.value = true
		notice.value = ''
		const nextOperationId = crypto.randomUUID()
		try {
			const receipt = destination.value === 'stream'
				? await sendZulipStreamMessage({
					accountId: accountId.value,
					stream: stream.value,
					topic: topic.value,
					content: content.value,
					operationId: nextOperationId,
				})
				: await sendZulipDirectMessage({
					accountId: accountId.value,
					recipients: recipients.value.split(','),
					content: content.value,
					operationId: nextOperationId,
				})
			operationId.value = receipt.operationId
			content.value = ''
			notice.value = `Operation ${receipt.operationId} accepted for ${receipt.accountId}.`
			await refreshStatus()
		} catch (error) {
			notice.value = messageFrom(error, 'Zulip command failed.')
		} finally {
			busy.value = false
		}
	}

	async function refreshStatus(): Promise<void> {
		busy.value = true
		notice.value = ''
		try {
			status.value = await getZulipOperationStatus(operationId.value)
			notice.value = status.value ? '' : 'No Zulip operation was found for this ID.'
		} catch (error) {
			notice.value = messageFrom(error, 'Zulip status is unavailable.')
		} finally {
			busy.value = false
		}
	}

	function updateDestination(value: ZulipDestination): void { destination.value = value }
	function updateAccountId(value: string): void { accountId.value = value }
	function updateStream(value: string): void { stream.value = value }
	function updateTopic(value: string): void { topic.value = value }
	function updateRecipients(value: string): void { recipients.value = value }
	function updateContent(value: string): void { content.value = value }
	function updateOperationId(value: string): void { operationId.value = value }

	return {
		model,
		send,
		refreshStatus,
		updateDestination,
		updateAccountId,
		updateStream,
		updateTopic,
		updateRecipients,
		updateContent,
		updateOperationId,
	}
}

function messageFrom(error: unknown, fallback: string): string {
	return error instanceof Error ? error.message : fallback
}
