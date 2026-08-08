import { create } from '@bufbuild/protobuf'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	MailSyncOutcomeV1,
	MailSyncProviderPathReadinessV1,
	MailSyncRunPageV1Schema,
	MailSyncRunV1Schema,
	MailSyncStatusV1Schema,
	MailSyncTriggerV1,
} from '../../../gen/makosh/mail/sync_health/v1/client_pb'
import {
	getMailSyncStatus,
	listMailSyncRuns,
} from '../api/mailSyncHealthGateway'
import { useMailSyncHealth } from './useMailSyncHealth'

vi.mock('../api/mailSyncHealthGateway', () => ({
	getMailSyncStatus: vi.fn(),
	listMailSyncRuns: vi.fn(),
}))

describe('Mail sync health controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(getMailSyncStatus).mockResolvedValue(create(MailSyncStatusV1Schema, {
			connectionId: 'primary',
			providerPathReadiness:
				MailSyncProviderPathReadinessV1.MAIL_SYNC_PROVIDER_PATH_READINESS_READY,
			consecutiveFailures: 0,
			projectionRevision: 2n,
		}))
		vi.mocked(listMailSyncRuns).mockResolvedValue(create(MailSyncRunPageV1Schema, {
			item: [run('operation-1')],
			nextCursor: 'cursor-2',
		}))
	})

	it('loads status and paged restart-safe history for an admitted Mail connection', async () => {
		const controller = useMailSyncHealth({
			canQuery: () => true,
			connections: () => [mailConnection()],
		})

		await controller.reconcile()

		expect(getMailSyncStatus).toHaveBeenCalledWith('primary')
		expect(listMailSyncRuns).toHaveBeenCalledWith({ connectionId: 'primary' })
		expect(controller.model.value).toMatchObject({
			state: 'ready',
			selectedConnectionId: 'primary',
			readiness: 'Ready',
			hasMoreRuns: true,
		})
		expect(controller.model.value.runs).toHaveLength(1)

		vi.mocked(listMailSyncRuns).mockResolvedValueOnce(create(MailSyncRunPageV1Schema, {
			item: [run('operation-1'), run('operation-2')],
		}))
		await controller.loadMore()

		expect(listMailSyncRuns).toHaveBeenLastCalledWith({
			connectionId: 'primary',
			cursor: 'cursor-2',
		})
		expect(controller.model.value.runs.map(({ operationId }) => operationId)).toEqual([
			'operation-1',
			'operation-2',
		])
		expect(controller.model.value.hasMoreRuns).toBe(false)
	})

	it('fails closed before transport when capability or effective connection is absent', async () => {
		const blocked = useMailSyncHealth({
			canQuery: () => false,
			connections: () => [mailConnection()],
		})
		await blocked.reconcile()
		expect(blocked.model.value.state).toBe('blocked')

		const noConnection = useMailSyncHealth({
			canQuery: () => true,
			connections: () => [],
		})
		await noConnection.reconcile()
		expect(noConnection.model.value.state).toBe('empty')
		expect(getMailSyncStatus).not.toHaveBeenCalled()
		expect(listMailSyncRuns).not.toHaveBeenCalled()
	})
})

function run(operationId: string) {
	return create(MailSyncRunV1Schema, {
		operationId,
		connectionId: 'primary',
		trigger: MailSyncTriggerV1.MAIL_SYNC_TRIGGER_MANUAL,
		outcome: MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_SUCCEEDED,
		observedMessages: 1n,
		startedAtUnixSeconds: 1_700_000_000n,
		completedAtUnixSeconds: 1_700_000_001n,
		runtimeGeneration: 1n,
		projectionRevision: 2n,
	})
}

function mailConnection() {
	return {
		registrationId: 'mail-primary',
		connectionId: 'primary',
		deliveryReady: true,
		syncReady: true,
	}
}
