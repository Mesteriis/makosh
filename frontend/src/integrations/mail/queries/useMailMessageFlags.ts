import { computed, ref } from 'vue'

import {
	MailMessageFlagOperationOutcomeV1,
	type MailMessageFlagOperationStatusV1,
} from '../../../gen/makosh/mail/message_flags/v1/client_pb'
import {
	getMailMessageFlagStatus,
	mutateMailMessageFlag,
	type MailMessageFlagMutationInput,
} from '../api/mailMessageFlagsGateway'
import type {
	MailMessageFlagModel,
	MailMessageFlagStatus,
} from '../presentation/mailMessageFlagModel'

export type MailMessageFlagSelection = {
	connectionId: string
	messageId: string
	isRead: boolean
	isStarred: boolean
}

export function useMailMessageFlags(input: {
	canMutate: () => boolean
	canQueryStatus: () => boolean
	selection: () => MailMessageFlagSelection | null
	refreshProjection: () => Promise<void>
}) {
	const busy = ref(false)
	const status = ref<MailMessageFlagStatus>('idle')
	const statusMessage = ref('')
	const operationId = ref('')
	const operationConnectionId = ref('')

	const model = computed<MailMessageFlagModel>(() => {
		const selection = input.selection()
		return {
			canMutate: input.canMutate(),
			canQueryStatus: input.canQueryStatus(),
			hasSelection: selection !== null,
			isRead: selection?.isRead ?? false,
			isStarred: selection?.isStarred ?? false,
			busy: busy.value,
			status: capabilityStatus(selection),
			statusMessage: capabilityMessage(selection),
			operationId: operationId.value,
		}
	})

	async function setRead(targetValue: boolean): Promise<void> {
		await submit('read', targetValue)
	}

	async function setStarred(targetValue: boolean): Promise<void> {
		await submit('starred', targetValue)
	}

	async function submit(
		kind: MailMessageFlagMutationInput['kind'],
		targetValue: boolean,
	): Promise<void> {
		const selection = input.selection()
		if (!input.canMutate() || !input.canQueryStatus() || !selection || busy.value) {
			status.value = 'blocked'
			statusMessage.value = 'Mail message flag command and query capabilities are required.'
			return
		}
		busy.value = true
		status.value = 'pending'
		statusMessage.value = 'Submitting provider flag mutation…'
		const nextOperationId = `mail-flag-${globalThis.crypto.randomUUID()}`
		try {
			const acceptedOperationId = await mutateMailMessageFlag({
				operationId: nextOperationId,
				connectionId: selection.connectionId,
				messageId: selection.messageId,
				kind,
				targetValue,
			})
			operationId.value = acceptedOperationId
			operationConnectionId.value = selection.connectionId
			statusMessage.value = 'Provider mutation accepted; waiting for terminal status…'
			await refreshStatus()
		} catch (error) {
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail provider flag mutation failed.'
		} finally {
			busy.value = false
		}
	}

	async function refreshStatus(): Promise<void> {
		if (
			!input.canQueryStatus()
			|| !operationId.value
			|| !operationConnectionId.value
		) return
		try {
			const next = await getMailMessageFlagStatus({
				operationId: operationId.value,
				connectionId: operationConnectionId.value,
			})
			applyStatus(next)
			if (next?.outcome === MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_SUCCEEDED) {
				await input.refreshProjection()
			}
		} catch (error) {
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail provider flag status is unavailable.'
		}
	}

	function applyStatus(next: MailMessageFlagOperationStatusV1 | undefined): void {
		if (!next) {
			status.value = 'error'
			statusMessage.value = 'Mail provider flag operation was not found.'
			return
		}
		if (next.outcome === MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_PENDING) {
			status.value = 'pending'
			statusMessage.value = 'Provider mutation is pending.'
			return
		}
		if (next.outcome === MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_SUCCEEDED) {
			status.value = 'succeeded'
			statusMessage.value = `Provider mutation confirmed at projection revision ${next.projectionRevision ?? 0n}.`
			return
		}
		if (next.outcome === MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_REJECTED) {
			status.value = 'rejected'
			statusMessage.value = 'Provider rejected the flag mutation.'
			return
		}
		if (next.outcome === MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_UNKNOWN) {
			status.value = 'outcome-unknown'
			statusMessage.value = 'Provider outcome is unknown; refresh Mail before retrying.'
			return
		}
		status.value = 'error'
		statusMessage.value = 'Mail provider returned an invalid flag operation status.'
	}

	function capabilityStatus(
		selection: MailMessageFlagSelection | null,
	): MailMessageFlagStatus {
		if (!input.canMutate() || !input.canQueryStatus() || !selection) return 'blocked'
		return status.value
	}

	function capabilityMessage(selection: MailMessageFlagSelection | null): string {
		if (!input.canMutate()) return 'Mail message flag command capability is not admitted.'
		if (!input.canQueryStatus()) return 'Mail message flag status capability is not admitted.'
		if (!selection) return 'Select a Mail message to change provider flags.'
		return statusMessage.value
	}

	return {
		model,
		refreshStatus,
		setRead,
		setStarred,
	}
}
