import type {
	WhatsAppCommandOperationStatusV1,
	WhatsAppOperationAcceptedV1,
} from '../../../gen/makosh/whatsapp/v1/client_pb'
import { getWhatsAppCommandConnectClient } from './whatsappCommandClient'
import { getWhatsAppQueryConnectClient } from './whatsappQueryClient'

export async function sendWhatsAppText(input: {
	accountId: string
	providerChatId: string
	text: string
	operationId: string
}): Promise<WhatsAppOperationAcceptedV1> {
	const text = input.text.trim()
	if (!text) {
		throw new RangeError('WhatsApp message text is required')
	}
	return getWhatsAppCommandConnectClient().executeCommand({
		command: {
			case: 'sendText',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				providerChatId: requireIdentifier('chat ID', input.providerChatId),
				text,
				operationId: requireIdentifier('operation ID', input.operationId),
			},
		},
	})
}

export async function getWhatsAppOperationStatus(
	operationId: string,
): Promise<WhatsAppCommandOperationStatusV1 | null> {
	const response = await getWhatsAppQueryConnectClient().getOperationStatus({
		operationId: requireIdentifier('operation ID', operationId),
	})
	return response.status ?? null
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`WhatsApp ${label} is required`)
	}
	return normalized
}
