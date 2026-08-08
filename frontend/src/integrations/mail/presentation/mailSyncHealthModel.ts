import {
	MailSyncFailureCodeV1,
	MailSyncOutcomeV1,
	MailSyncProviderPathReadinessV1,
	MailSyncTriggerV1,
	type MailSyncRunV1,
	type MailSyncStatusV1,
} from '../../../gen/makosh/mail/sync_health/v1/client_pb'
import type {
	MailAccountConnection as MailSyncHealthConnection,
} from '../queries/mailAccountConnections'

export type MailSyncHealthState = 'blocked' | 'empty' | 'error' | 'loading' | 'ready'

export type MailSyncHealthModel = {
	canQuery: boolean
	state: MailSyncHealthState
	statusMessage: string
	connections: readonly MailSyncHealthConnectionOption[]
	selectedConnectionId: string
	readiness: string
	readinessTone: MailSyncHealthTone
	latestOutcome: string
	latestOutcomeTone: MailSyncHealthTone
	lastSuccessAt: string
	consecutiveFailures: string
	projectionRevision: string
	runs: readonly MailSyncRunRow[]
	hasMoreRuns: boolean
}

export type MailSyncHealthConnectionOption = {
	id: string
	label: string
}

export type MailSyncRunRow = {
	operationId: string
	trigger: string
	outcome: string
	outcomeTone: MailSyncHealthTone
	observedMessages: string
	startedAt: string
	completedAt: string
	failure: string
	runtimeGeneration: string
	projectionRevision: string
}

export type MailSyncHealthTone = 'danger' | 'neutral' | 'progress' | 'success'

export function buildMailSyncHealthModel(input: {
	canQuery: boolean
	state: MailSyncHealthState
	statusMessage: string
	connections: readonly MailSyncHealthConnection[]
	selectedConnectionId: string
	status: MailSyncStatusV1 | undefined
	runs: readonly MailSyncRunV1[]
	hasMoreRuns: boolean
}): MailSyncHealthModel {
	const latestRun = input.status?.latestRun
	return {
		canQuery: input.canQuery,
		state: input.state,
		statusMessage: input.statusMessage,
		connections: input.connections.map((connection) => ({
			id: connection.connectionId,
			label: connection.connectionId,
		})),
		selectedConnectionId: input.selectedConnectionId,
		readiness: providerPathReadinessLabel(input.status?.providerPathReadiness),
		readinessTone: providerPathReadinessTone(input.status?.providerPathReadiness),
		latestOutcome: latestRun ? syncOutcomeLabel(latestRun.outcome) : 'No runs',
		latestOutcomeTone: syncOutcomeTone(latestRun?.outcome),
		lastSuccessAt: formatUnixSeconds(input.status?.lastSuccessAtUnixSeconds),
		consecutiveFailures: `${input.status?.consecutiveFailures ?? 0}`,
		projectionRevision: `${input.status?.projectionRevision ?? 0n}`,
		runs: input.runs.map(buildMailSyncRunRow),
		hasMoreRuns: input.hasMoreRuns,
	}
}

export function buildMailSyncRunRow(run: MailSyncRunV1): MailSyncRunRow {
	return {
		operationId: run.operationId,
		trigger: syncTriggerLabel(run.trigger),
		outcome: syncOutcomeLabel(run.outcome),
		outcomeTone: syncOutcomeTone(run.outcome),
		observedMessages: `${run.observedMessages}`,
		startedAt: formatUnixSeconds(run.startedAtUnixSeconds),
		completedAt: formatUnixSeconds(run.completedAtUnixSeconds),
		failure: syncFailureLabel(run.failureCode),
		runtimeGeneration: `${run.runtimeGeneration}`,
		projectionRevision: `${run.projectionRevision}`,
	}
}

function providerPathReadinessLabel(
	readiness: MailSyncProviderPathReadinessV1 | undefined,
): string {
	if (readiness === MailSyncProviderPathReadinessV1.MAIL_SYNC_PROVIDER_PATH_READINESS_READY) {
		return 'Ready'
	}
	if (readiness === MailSyncProviderPathReadinessV1.MAIL_SYNC_PROVIDER_PATH_READINESS_UNAVAILABLE) {
		return 'Unavailable'
	}
	return 'Unknown'
}

function providerPathReadinessTone(
	readiness: MailSyncProviderPathReadinessV1 | undefined,
): MailSyncHealthTone {
	if (readiness === MailSyncProviderPathReadinessV1.MAIL_SYNC_PROVIDER_PATH_READINESS_READY) {
		return 'success'
	}
	if (readiness === MailSyncProviderPathReadinessV1.MAIL_SYNC_PROVIDER_PATH_READINESS_UNAVAILABLE) {
		return 'danger'
	}
	return 'neutral'
}

function syncTriggerLabel(trigger: MailSyncTriggerV1): string {
	if (trigger === MailSyncTriggerV1.MAIL_SYNC_TRIGGER_MANUAL) return 'Manual'
	if (trigger === MailSyncTriggerV1.MAIL_SYNC_TRIGGER_SCHEDULED) return 'Scheduled'
	return 'Unknown'
}

function syncOutcomeLabel(outcome: MailSyncOutcomeV1 | undefined): string {
	if (outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_RUNNING) return 'Running'
	if (outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_SUCCEEDED) return 'Succeeded'
	if (outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_FAILED) return 'Failed'
	if (outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_INTERRUPTED) return 'Interrupted'
	return 'Unknown'
}

function syncOutcomeTone(outcome: MailSyncOutcomeV1 | undefined): MailSyncHealthTone {
	if (outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_RUNNING) return 'progress'
	if (outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_SUCCEEDED) return 'success'
	if (
		outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_FAILED
		|| outcome === MailSyncOutcomeV1.MAIL_SYNC_OUTCOME_INTERRUPTED
	) return 'danger'
	return 'neutral'
}

function syncFailureLabel(failure: MailSyncFailureCodeV1 | undefined): string {
	if (failure === undefined || failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_UNSPECIFIED) {
		return 'None'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_ADMISSION_REJECTED) {
		return 'Admission rejected'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_CONTROL_UNAVAILABLE) {
		return 'Control unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_STORAGE_UNAVAILABLE) {
		return 'Storage unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_CREDENTIAL_UNAVAILABLE) {
		return 'Credential unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_PERSISTENCE_UNAVAILABLE) {
		return 'Persistence unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_PROVIDER_UNAVAILABLE) {
		return 'Provider unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_EVENT_HUB_UNAVAILABLE) {
		return 'Event Hub unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_ATTACHMENT_ANCHOR_UNAVAILABLE) {
		return 'Attachment anchor unavailable'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_RUNTIME_RESTARTED) {
		return 'Runtime restarted'
	}
	if (failure === MailSyncFailureCodeV1.MAIL_SYNC_FAILURE_CODE_DEADLINE_EXCEEDED) {
		return 'Deadline exceeded'
	}
	return 'Unknown'
}

function formatUnixSeconds(value: bigint | undefined): string {
	if (value === undefined || value <= 0n || value > BigInt(Math.floor(Number.MAX_SAFE_INTEGER / 1_000))) {
		return 'Not recorded'
	}
	const date = new Date(Number(value) * 1_000)
	if (Number.isNaN(date.getTime())) return 'Not recorded'
	return date.toISOString().replace('T', ' ').replace('.000Z', ' UTC')
}
