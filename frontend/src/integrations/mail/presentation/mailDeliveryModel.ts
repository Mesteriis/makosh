import {
	MailDeliveryOutcomeV1,
	type MailDeliveryOperationStatusV1,
} from '../../../gen/makosh/mail/v1/client_pb'

export type MailDeliveryModel = {
	operationId: string
	busy: boolean
	canDeliver: boolean
	notice: string
	status: MailDeliveryStatusCard | null
}

export type MailDeliveryStatusCard = {
	operationId: string
	connectionId: string
	outcome: string
	requestedAt: string
	completedAt: string
	responseCode: string
}

export function buildMailDeliveryStatusCard(
	status: MailDeliveryOperationStatusV1 | null,
): MailDeliveryStatusCard | null {
	if (!status) return null
	return {
		operationId: status.operationId,
		connectionId: status.connectionId,
		outcome: mailDeliveryOutcomeLabel(status.outcome),
		requestedAt: formatUnixSeconds(status.requestedAtUnixSeconds),
		completedAt: status.completedAtUnixSeconds === undefined
			? 'Pending'
			: formatUnixSeconds(status.completedAtUnixSeconds),
		responseCode: status.responseCode === undefined ? '—' : String(status.responseCode),
	}
}

function mailDeliveryOutcomeLabel(outcome: MailDeliveryOutcomeV1): string {
	if (outcome === MailDeliveryOutcomeV1.MAIL_DELIVERY_OUTCOME_PENDING) return 'pending'
	if (outcome === MailDeliveryOutcomeV1.MAIL_DELIVERY_OUTCOME_ACCEPTED) return 'accepted'
	if (outcome === MailDeliveryOutcomeV1.MAIL_DELIVERY_OUTCOME_REJECTED) return 'rejected'
	return 'unknown'
}

function formatUnixSeconds(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds) || milliseconds <= 0) return 'Unknown'
	return new Intl.DateTimeFormat('en', {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(new Date(milliseconds))
}
