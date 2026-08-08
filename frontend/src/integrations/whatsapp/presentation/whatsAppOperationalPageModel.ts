import type { WhatsAppCommandOperationStatusV1 } from '../../../gen/makosh/whatsapp/v1/client_pb'

export type WhatsAppOperationalPageModel = {
	accountId: string
	providerChatId: string
	draft: string
	operationId: string
	busy: boolean
	canSend: boolean
	notice: string
	status: WhatsAppOperationStatusCard | null
}

export type WhatsAppOperationStatusCard = {
	operationId: string
	accountId: string
	state: string
	requestedAt: string
	completedAt: string
}

export function buildWhatsAppOperationStatusCard(
	status: WhatsAppCommandOperationStatusV1 | null,
): WhatsAppOperationStatusCard | null {
	if (!status) {
		return null
	}
	return {
		operationId: status.operationId,
		accountId: status.accountId,
		state: status.state || 'unknown',
		requestedAt: formatUnixSeconds(status.requestedAtUnixSeconds),
		completedAt: status.completedAtUnixSeconds === undefined
			? 'Pending'
			: formatUnixSeconds(status.completedAtUnixSeconds),
	}
}

function formatUnixSeconds(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds) || milliseconds <= 0) {
		return 'Unknown'
	}
	return new Intl.DateTimeFormat('en', {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(new Date(milliseconds))
}
