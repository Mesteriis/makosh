import type {
	ZulipCommandOperationStatusV1,
	ZulipCommandReceiptV1,
} from '../../../gen/makosh/zulip/v1/client_pb'
import { getZulipCommandConnectClient } from './zulipCommandClient'
import { getZulipQueryConnectClient } from './zulipQueryClient'

export async function sendZulipStreamMessage(input: {
	accountId: string
	stream: string
	topic: string
	content: string
	operationId: string
}): Promise<ZulipCommandReceiptV1> {
	return getZulipCommandConnectClient().executeCommand({
		command: {
			case: 'sendStream',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				stream: requireIdentifier('stream', input.stream),
				topic: requireIdentifier('topic', input.topic),
				content: requireContent(input.content),
				operationId: requireIdentifier('operation ID', input.operationId),
			},
		},
	})
}

export async function sendZulipDirectMessage(input: {
	accountId: string
	recipients: readonly string[]
	content: string
	operationId: string
}): Promise<ZulipCommandReceiptV1> {
	const recipients = input.recipients.map((recipient) => recipient.trim()).filter(Boolean)
	if (recipients.length === 0) {
		throw new RangeError('Zulip recipient is required')
	}
	return getZulipCommandConnectClient().executeCommand({
		command: {
			case: 'sendDirect',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				recipient: recipients,
				content: requireContent(input.content),
				operationId: requireIdentifier('operation ID', input.operationId),
			},
		},
	})
}

export async function getZulipOperationStatus(
	operationId: string,
): Promise<ZulipCommandOperationStatusV1 | null> {
	const response = await getZulipQueryConnectClient().getOperationStatus({
		operationId: requireIdentifier('operation ID', operationId),
	})
	return response.status ?? null
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`Zulip ${label} is required`)
	}
	return normalized
}

function requireContent(value: string): string {
	const content = value.trim()
	if (!content) {
		throw new RangeError('Zulip message content is required')
	}
	return content
}
