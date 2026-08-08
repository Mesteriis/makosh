import { computed, ref } from 'vue'

import {
	MailMessageLocationOperationOutcomeV1,
	type MailMessageLocationOperationStatusV1,
} from '../../../gen/makosh/mail/message_location/v1/client_pb'
import {
	getMailMessageLocationStatus,
	mutateMailMessageLocation,
	type MailMessageLocationMutationInput,
} from '../api/mailMessageLocationGateway'
import type {
	MailMessageLocationFolderOption,
	MailMessageLocationModel,
	MailMessageLocationStatus,
} from '../presentation/mailMessageLocationModel'

export type MailMessageLocationSelection = {
	connectionId: string
	messageId: string
	isTrashed: boolean
	folders: readonly MailMessageLocationFolderOption[]
}

export function useMailMessageLocation(input: {
	canMutate: () => boolean
	canQueryStatus: () => boolean
	selection: () => MailMessageLocationSelection | null
	refreshProjection: () => Promise<void>
}) {
	const busy = ref(false)
	const status = ref<MailMessageLocationStatus>('idle')
	const statusMessage = ref('')
	const operationId = ref('')
	const operationConnectionId = ref('')
	const targetFolderId = ref('')

	const model = computed<MailMessageLocationModel>(() => {
		const selection = input.selection()
		return {
			canMutate: input.canMutate(),
			canQueryStatus: input.canQueryStatus(),
			hasSelection: selection !== null,
			isTrashed: selection?.isTrashed ?? false,
			busy: busy.value,
			status: capabilityStatus(selection),
			statusMessage: capabilityMessage(selection),
			operationId: operationId.value,
			targetFolderId: targetFolderId.value,
			targetFolders: selection?.folders ?? [],
		}
	})

	async function archive(): Promise<void> {
		await submit('archive')
	}

	async function trash(): Promise<void> {
		await submit('trash')
	}

	async function restore(): Promise<void> {
		await submit('restore')
	}

	async function move(): Promise<void> {
		await submit('move', targetFolderId.value)
	}

	function selectTargetFolder(value: string): void {
		targetFolderId.value = value
	}

	async function submit(
		kind: MailMessageLocationMutationInput['kind'],
		targetFolderId?: string,
	): Promise<void> {
		const selection = input.selection()
		if (
			!input.canMutate()
			|| !input.canQueryStatus()
			|| !selection
			|| busy.value
			|| (kind === 'move' && !targetFolderId)
		) {
			status.value = 'blocked'
			statusMessage.value = 'Mail message location command and query capabilities are required.'
			return
		}
		busy.value = true
		status.value = 'pending'
		statusMessage.value = 'Submitting provider location mutation…'
		const nextOperationId = `mail-location-${globalThis.crypto.randomUUID()}`
		try {
			const acceptedOperationId = await mutateMailMessageLocation({
				operationId: nextOperationId,
				connectionId: selection.connectionId,
				messageId: selection.messageId,
				kind,
				targetFolderId,
			})
			operationId.value = acceptedOperationId
			operationConnectionId.value = selection.connectionId
			statusMessage.value = 'Provider location mutation accepted; waiting for terminal status…'
			await refreshStatus()
		} catch (error) {
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail provider location mutation failed.'
		} finally {
			busy.value = false
		}
	}

	async function refreshStatus(): Promise<void> {
		if (!input.canQueryStatus() || !operationId.value || !operationConnectionId.value) return
		try {
			const next = await getMailMessageLocationStatus({
				operationId: operationId.value,
				connectionId: operationConnectionId.value,
			})
			applyStatus(next)
			if (
				next?.outcome
				=== MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_SUCCEEDED
			) {
				await input.refreshProjection()
			}
		} catch (error) {
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail provider location status is unavailable.'
		}
	}

	function applyStatus(next: MailMessageLocationOperationStatusV1 | undefined): void {
		if (!next) {
			status.value = 'error'
			statusMessage.value = 'Mail provider location operation was not found.'
			return
		}
		if (next.outcome === MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_PENDING) {
			status.value = 'pending'
			statusMessage.value = 'Provider location mutation is pending.'
			return
		}
		if (next.outcome === MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_SUCCEEDED) {
			status.value = 'succeeded'
			statusMessage.value = `Provider location confirmed at projection revision ${next.projectionRevision ?? 0n}.`
			return
		}
		if (next.outcome === MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_REJECTED) {
			status.value = 'rejected'
			statusMessage.value = 'Provider rejected the location mutation.'
			return
		}
		if (next.outcome === MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_UNSUPPORTED) {
			status.value = 'unsupported'
			statusMessage.value = 'This provider does not support the requested safe location mutation.'
			return
		}
		if (next.outcome === MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_UNKNOWN) {
			status.value = 'outcome-unknown'
			statusMessage.value = 'Provider outcome is unknown; refresh Mail before retrying.'
			return
		}
		status.value = 'error'
		statusMessage.value = 'Mail provider returned an invalid location operation status.'
	}

	function capabilityStatus(
		selection: MailMessageLocationSelection | null,
	): MailMessageLocationStatus {
		if (!input.canMutate() || !input.canQueryStatus() || !selection) return 'blocked'
		return status.value
	}

	function capabilityMessage(selection: MailMessageLocationSelection | null): string {
		if (!input.canMutate()) return 'Mail message location command capability is not admitted.'
		if (!input.canQueryStatus()) return 'Mail message location status capability is not admitted.'
		if (!selection) return 'Select a Mail message to change its provider location.'
		return statusMessage.value
	}

	return {
		archive,
		model,
		move,
		refreshStatus,
		restore,
		selectTargetFolder,
		trash,
	}
}
