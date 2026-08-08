import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	MailSyncFailureCodeV1,
	MailSyncOutcomeV1,
	MailSyncProviderPathReadinessV1,
	MailSyncRunV1Schema,
	MailSyncStatusV1Schema,
	MailSyncTriggerV1,
} from '../../../gen/makosh/mail/sync_health/v1/client_pb'
import {
	buildMailSyncHealthModel,
	buildMailSyncRunRow,
} from './mailSyncHealthModel'

describe('Mail sync health presentation model', () => {
	it('projects bounded provider readiness and terminal run evidence', () => {
		const run = create(MailSyncRunV1Schema, {
			operationId: 'operation-1',
			connectionId: 'primary',
			trigger: MailSyncTriggerV1.MAIL_SYNC_TRIGGER_SCHEDULED,
			outcome: MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_INTERRUPTED,
			observedMessages: 12n,
			startedAtUnixSeconds: 1_700_000_000n,
			completedAtUnixSeconds: 1_700_000_030n,
			failureCode: MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_RUNTIME_RESTARTED,
			runtimeGeneration: 4n,
			projectionRevision: 9n,
		})
		const status = create(MailSyncStatusV1Schema, {
			connectionId: 'primary',
			providerPathReadiness:
				MailSyncProviderPathReadinessV1.MAIL_SYNC_PROVIDER_PATH_READINESS_UNAVAILABLE,
			latestRun: run,
			consecutiveFailures: 2,
			lastSuccessAtUnixSeconds: 1_699_999_000n,
			projectionRevision: 9n,
		})

		const model = buildMailSyncHealthModel({
			canQuery: true,
			state: 'ready',
			statusMessage: '',
			connections: [{
				connectionId: 'primary',
				deliveryReady: true,
				registrationId: 'mail-primary',
				syncReady: true,
			}],
			selectedConnectionId: 'primary',
			status,
			runs: [run],
			hasMoreRuns: true,
		})

		expect(model).toMatchObject({
			readiness: 'Unavailable',
			readinessTone: 'danger',
			latestOutcome: 'Interrupted',
			latestOutcomeTone: 'danger',
			consecutiveFailures: '2',
			projectionRevision: '9',
			hasMoreRuns: true,
		})
		expect(model.runs[0]).toMatchObject({
			trigger: 'Scheduled',
			outcome: 'Interrupted',
			failure: 'Runtime restarted',
			observedMessages: '12',
			runtimeGeneration: '4',
		})
		expect(model.runs[0]!.startedAt).toBe('2023-11-14 22:13:20 UTC')
	})

	it('does not surface invalid timestamps or unspecified codes as diagnostics', () => {
		const row = buildMailSyncRunRow(create(MailSyncRunV1Schema, {
			operationId: 'operation-unknown',
			connectionId: 'primary',
			startedAtUnixSeconds: 0n,
		}))

		expect(row).toMatchObject({
			trigger: 'Unknown',
			outcome: 'Unknown',
			failure: 'None',
			startedAt: 'Not recorded',
			completedAt: 'Not recorded',
		})
	})

	it('renders the bounded operation deadline as a distinct terminal failure', () => {
		const row = buildMailSyncRunRow(create(MailSyncRunV1Schema, {
			operationId: 'operation-deadline',
			connectionId: 'primary',
			outcome: MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_FAILED,
			failureCode: MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_DEADLINE_EXCEEDED,
			startedAtUnixSeconds: 1_700_000_000n,
			completedAtUnixSeconds: 1_700_000_300n,
		}))

		expect(row).toMatchObject({
			outcome: 'Failed',
			failure: 'Deadline exceeded',
		})
	})
})
