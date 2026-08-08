import { computed, ref } from 'vue'

import {
	MailMessagePermanentDeleteOperationOutcomeV1,
	type MailMessagePermanentDeleteOperationStatusV1,
} from '../../../gen/makosh/mail/message_permanent_delete/v1/client_pb'
import {
	getMailMessagePermanentDeleteStatus,
	permanentlyDeleteMailMessage,
} from '../api/mailMessagePermanentDeleteGateway'
import type {
	MailMessagePermanentDeleteModel,
	MailMessagePermanentDeleteStatus,
} from '../presentation/mailMessagePermanentDeleteModel'

export type MailMessagePermanentDeleteSelection = {
	connectionId: string
	messageId: string
	projectionRevision: bigint
	isTrashed: boolean
}

export function useMailMessagePermanentDelete(input: {
	canDelete: () => boolean
	canQueryStatus: () => boolean
	selection: () => MailMessagePermanentDeleteSelection | null
	refreshProjection: () => Promise<void>
}) {
	const confirmed = ref(false)
	const busy = ref(false)
	const status = ref<MailMessagePermanentDeleteStatus>('idle')
	const statusMessage = ref('')
	const operationId = ref('')
	const operationConnectionId = ref('')

	const model = computed<MailMessagePermanentDeleteModel>(() => {
		const selection = input.selection()
		const hasTrashSelection = Boolean(selection?.isTrashed)
		return {
			canDelete: input.canDelete(),
			canQueryStatus: input.canQueryStatus(),
			hasTrashSelection,
			confirmed: confirmed.value,
			busy: busy.value,
			status: capabilityStatus(hasTrashSelection),
			statusMessage: capabilityMessage(selection),
			operationId: operationId.value,
		}
	})

	function setConfirmed(value: boolean): void {
		confirmed.value = value
	}

	async function permanentlyDelete(): Promise<void> {
		const selection = input.selection()
		if (
			!input.canDelete()
			|| !input.canQueryStatus()
			|| !selection?.isTrashed
			|| !confirmed.value
			|| busy.value
		) {
			status.value = 'blocked'
			statusMessage.value = 'Select a Trash message and confirm permanent provider deletion.'
			return
		}
		busy.value = true
		status.value = 'pending'
		statusMessage.value = 'Submitting permanent provider deletion…'
		try {
			const acceptedOperationId = await permanentlyDeleteMailMessage({
				operationId: `mail-permanent-delete-${globalThis.crypto.randomUUID()}`,
				connectionId: selection.connectionId,
				messageId: selection.messageId,
				expectedProjectionRevision: selection.projectionRevision,
				confirmed: true,
			})
			operationId.value = acceptedOperationId
			operationConnectionId.value = selection.connectionId
			confirmed.value = false
			statusMessage.value = 'Permanent deletion accepted; waiting for terminal status…'
			await refreshStatus()
		} catch (error) {
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail permanent delete command failed.'
		} finally {
			busy.value = false
		}
	}

	async function refreshStatus(): Promise<void> {
		if (!input.canQueryStatus() || !operationId.value || !operationConnectionId.value) return
		try {
			const next = await getMailMessagePermanentDeleteStatus({
				operationId: operationId.value,
				connectionId: operationConnectionId.value,
			})
			applyStatus(next)
			if (
				next?.outcome
				=== MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_SUCCEEDED
			) {
				await input.refreshProjection()
			}
		} catch (error) {
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail permanent delete status is unavailable.'
		}
	}

	function applyStatus(next: MailMessagePermanentDeleteOperationStatusV1 | undefined): void {
		if (!next) {
			status.value = 'error'
			statusMessage.value = 'Mail permanent delete operation was not found.'
			return
		}
		switch (next.outcome) {
			case MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_PENDING:
				status.value = 'pending'
				statusMessage.value = 'Permanent provider deletion is pending.'
				return
			case MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_SUCCEEDED:
				status.value = 'succeeded'
				statusMessage.value = 'Provider message was permanently deleted; canonical evidence remains.'
				return
			case MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_REJECTED:
				status.value = 'rejected'
				statusMessage.value = 'Provider rejected permanent deletion.'
				return
			case MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_UNSUPPORTED:
				status.value = 'unsupported'
				statusMessage.value = 'This provider cannot safely delete only the selected message.'
				return
			case MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_REAUTHORIZATION_REQUIRED:
				status.value = 'reauthorization-required'
				statusMessage.value = 'Gmail requires separate owner-approved permanent-delete authorization.'
				return
			case MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_UNKNOWN:
				status.value = 'outcome-unknown'
				statusMessage.value = 'Provider outcome is unknown; refresh Mail before retrying.'
				return
			default:
				status.value = 'error'
				statusMessage.value = 'Mail provider returned an invalid permanent delete status.'
		}
	}

	function capabilityStatus(hasTrashSelection: boolean): MailMessagePermanentDeleteStatus {
		if (!input.canDelete() || !input.canQueryStatus() || !hasTrashSelection) return 'blocked'
		return status.value
	}

	function capabilityMessage(selection: MailMessagePermanentDeleteSelection | null): string {
		if (!input.canDelete()) return 'Mail permanent delete command capability is not admitted.'
		if (!input.canQueryStatus()) return 'Mail permanent delete status capability is not admitted.'
		if (!selection?.isTrashed) return 'Permanent deletion is available only from provider Trash.'
		return statusMessage.value
	}

	return {
		model,
		permanentlyDelete,
		refreshStatus,
		setConfirmed,
	}
}
