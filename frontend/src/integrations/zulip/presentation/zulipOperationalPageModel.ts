import type { ZulipCommandOperationStatusV1 } from '../../../gen/makosh/zulip/v1/client_pb'

export type ZulipDestination = 'direct' | 'stream'

export type ZulipOperationalPageModel = {
	destination: ZulipDestination
	accountId: string
	stream: string
	topic: string
	recipients: string
	content: string
	operationId: string
	busy: boolean
	canCommand: boolean
	notice: string
	status: ZulipOperationStatusCard | null
}

export type ZulipOperationStatusCard = {
	operationId: string
	accountId: string
	outcome: string
	providerMessageId: string
	requestedAt: string
	completedAt: string
}

export function buildZulipOperationStatusCard(
	status: ZulipCommandOperationStatusV1 | null,
): ZulipOperationStatusCard | null {
	if (!status) {
		return null
	}
	return {
		operationId: status.operationId,
		accountId: status.accountId,
		outcome: status.outcome || 'unknown',
		providerMessageId: status.providerMessageId?.toString() ?? 'Pending',
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
