import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MailMessageFlagOperationOutcomeV1 } from '../../../gen/makosh/mail/message_flags/v1/client_pb'
import {
	getMailMessageFlagStatus,
	mutateMailMessageFlag,
} from '../api/mailMessageFlagsGateway'
import { useMailMessageFlags } from './useMailMessageFlags'

vi.mock('../api/mailMessageFlagsGateway', () => ({
	getMailMessageFlagStatus: vi.fn(),
	mutateMailMessageFlag: vi.fn(),
}))

describe('Mail message flag controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
	})

	it('waits for provider confirmation before refreshing the operational projection', async () => {
		const refreshProjection = vi.fn().mockResolvedValue(undefined)
		vi.mocked(mutateMailMessageFlag).mockResolvedValue('flag-operation')
		vi.mocked(getMailMessageFlagStatus)
			.mockResolvedValueOnce({
				operationId: 'flag-operation',
				connectionId: 'mail-account',
				messageId: 'provider-message',
				outcome: MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_PENDING,
				requestedAtUnixSeconds: 100n,
			} as never)
			.mockResolvedValueOnce({
				operationId: 'flag-operation',
				connectionId: 'mail-account',
				messageId: 'provider-message',
				outcome: MailMessageFlagOperationOutcomeV1.MAIL_MESSAGE_FLAG_OPERATION_OUTCOME_SUCCEEDED,
				requestedAtUnixSeconds: 100n,
				completedAtUnixSeconds: 101n,
				projectionRevision: 7n,
			} as never)
		const controller = useMailMessageFlags({
			canMutate: () => true,
			canQueryStatus: () => true,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'provider-message',
				isRead: false,
				isStarred: false,
			}),
			refreshProjection,
		})

		await controller.setRead(true)

		expect(mutateMailMessageFlag).toHaveBeenCalledWith(expect.objectContaining({
			connectionId: 'mail-account',
			messageId: 'provider-message',
			kind: 'read',
			targetValue: true,
		}))
		expect(controller.model.value.status).toBe('pending')
		expect(refreshProjection).not.toHaveBeenCalled()

		await controller.refreshStatus()

		expect(controller.model.value.status).toBe('succeeded')
		expect(controller.model.value.statusMessage).toContain('revision 7')
		expect(refreshProjection).toHaveBeenCalledOnce()
	})

	it('fails closed before transport without both exact capabilities', async () => {
		const controller = useMailMessageFlags({
			canMutate: () => true,
			canQueryStatus: () => false,
			selection: () => ({
				connectionId: 'mail-account',
				messageId: 'provider-message',
				isRead: false,
				isStarred: false,
			}),
			refreshProjection: vi.fn(),
		})

		await controller.setStarred(true)

		expect(controller.model.value.status).toBe('blocked')
		expect(mutateMailMessageFlag).not.toHaveBeenCalled()
		expect(getMailMessageFlagStatus).not.toHaveBeenCalled()
	})
})
