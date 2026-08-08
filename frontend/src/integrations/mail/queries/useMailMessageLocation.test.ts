import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MailMessageLocationOperationOutcomeV1 } from '../../../gen/makosh/mail/message_location/v1/client_pb'
import {
	getMailMessageLocationStatus,
	mutateMailMessageLocation,
} from '../api/mailMessageLocationGateway'
import { useMailMessageLocation } from './useMailMessageLocation'

vi.mock('../api/mailMessageLocationGateway', () => ({
	getMailMessageLocationStatus: vi.fn(),
	mutateMailMessageLocation: vi.fn(),
}))

describe('Mail message location controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.stubGlobal('crypto', { randomUUID: () => 'location-operation' })
	})

	it('submits an exact archive and refreshes after terminal success', async () => {
		const refreshProjection = vi.fn().mockResolvedValue(undefined)
		vi.mocked(mutateMailMessageLocation).mockResolvedValue('accepted-location-operation')
		vi.mocked(getMailMessageLocationStatus)
			.mockResolvedValueOnce({
				operationId: 'accepted-location-operation',
				connectionId: 'mail-account',
				messageId: 'message-1',
				outcome: MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_PENDING,
				requestedAtUnixSeconds: 100n,
			} as never)
			.mockResolvedValueOnce({
				operationId: 'accepted-location-operation',
				connectionId: 'mail-account',
				messageId: 'message-1',
				outcome: MailMessageLocationOperationOutcomeV1.MAIL_MESSAGE_LOCATION_OPERATION_OUTCOME_SUCCEEDED,
				requestedAtUnixSeconds: 100n,
				completedAtUnixSeconds: 101n,
				projectionRevision: 8n,
			} as never)
		const controller = useMailMessageLocation({
			canMutate: () => true,
			canQueryStatus: () => true,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'message-1',
				isTrashed: false,
				folders: [{ id: 'Archive', label: 'Archive' }],
			}),
			refreshProjection,
		})

		await controller.archive()
		await controller.refreshStatus()

		expect(mutateMailMessageLocation).toHaveBeenCalledWith(expect.objectContaining({
			operationId: 'mail-location-location-operation',
			connectionId: 'mail-account',
			messageId: 'message-1',
			kind: 'archive',
			targetFolderId: undefined,
		}))
		expect(refreshProjection).toHaveBeenCalledOnce()
		expect(controller.model.value.status).toBe('succeeded')
	})

	it('fails closed before transport when the exact query capability is absent', async () => {
		const controller = useMailMessageLocation({
			canMutate: () => true,
			canQueryStatus: () => false,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'message-1',
				isTrashed: false,
				folders: [],
			}),
			refreshProjection: vi.fn(),
		})

		await controller.trash()

		expect(mutateMailMessageLocation).not.toHaveBeenCalled()
		expect(controller.model.value.status).toBe('blocked')
	})
})
