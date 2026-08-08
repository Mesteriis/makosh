import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MailMessagePermanentDeleteOperationOutcomeV1 } from '../../../gen/makosh/mail/message_permanent_delete/v1/client_pb'
import {
	getMailMessagePermanentDeleteStatus,
	permanentlyDeleteMailMessage,
} from '../api/mailMessagePermanentDeleteGateway'
import { useMailMessagePermanentDelete } from './useMailMessagePermanentDelete'

vi.mock('../api/mailMessagePermanentDeleteGateway', () => ({
	getMailMessagePermanentDeleteStatus: vi.fn(),
	permanentlyDeleteMailMessage: vi.fn(),
}))

describe('Mail message permanent delete controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.stubGlobal('crypto', { randomUUID: () => 'delete-operation' })
	})

	it('submits an exact confirmed Trash deletion and refreshes only after provider success', async () => {
		const refreshProjection = vi.fn().mockResolvedValue(undefined)
		vi.mocked(permanentlyDeleteMailMessage).mockResolvedValue('accepted-delete-operation')
		vi.mocked(getMailMessagePermanentDeleteStatus).mockResolvedValue({
			operationId: 'accepted-delete-operation',
			connectionId: 'mail-account',
			messageId: 'message-1',
			expectedProjectionRevision: 8n,
			outcome:
				MailMessagePermanentDeleteOperationOutcomeV1.MAIL_MESSAGE_PERMANENT_DELETE_OPERATION_OUTCOME_SUCCEEDED,
			requestedAtUnixSeconds: 100n,
			completedAtUnixSeconds: 101n,
			deletionProjectionRevision: 9n,
		} as never)
		const controller = useMailMessagePermanentDelete({
			canDelete: () => true,
			canQueryStatus: () => true,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'message-1',
				projectionRevision: 8n,
				isTrashed: true,
			}),
			refreshProjection,
		})

		controller.setConfirmed(true)
		await controller.permanentlyDelete()

		expect(permanentlyDeleteMailMessage).toHaveBeenCalledWith({
			operationId: 'mail-permanent-delete-delete-operation',
			connectionId: 'mail-account',
			messageId: 'message-1',
			expectedProjectionRevision: 8n,
			confirmed: true,
		})
		expect(getMailMessagePermanentDeleteStatus).toHaveBeenCalledWith({
			operationId: 'accepted-delete-operation',
			connectionId: 'mail-account',
		})
		expect(refreshProjection).toHaveBeenCalledOnce()
		expect(controller.model.value.status).toBe('succeeded')
		expect(controller.model.value.confirmed).toBe(false)
	})

	it('fails closed before transport without explicit confirmation', async () => {
		const controller = useMailMessagePermanentDelete({
			canDelete: () => true,
			canQueryStatus: () => true,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'message-1',
				projectionRevision: 8n,
				isTrashed: true,
			}),
			refreshProjection: vi.fn(),
		})

		await controller.permanentlyDelete()

		expect(permanentlyDeleteMailMessage).not.toHaveBeenCalled()
		expect(controller.model.value.status).toBe('blocked')
	})

	it('fails closed before transport when the selected message is not in Trash', async () => {
		const controller = useMailMessagePermanentDelete({
			canDelete: () => true,
			canQueryStatus: () => true,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'message-1',
				projectionRevision: 8n,
				isTrashed: false,
			}),
			refreshProjection: vi.fn(),
		})

		controller.setConfirmed(true)
		await controller.permanentlyDelete()

		expect(permanentlyDeleteMailMessage).not.toHaveBeenCalled()
		expect(controller.model.value.status).toBe('blocked')
	})
})
