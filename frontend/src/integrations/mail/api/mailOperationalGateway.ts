import type {
	MailDeliveryOperationStatusV1,
	SyncInboxAcceptedV1,
} from '../../../gen/makosh/mail/v1/client_pb'
import { getMailDeliveryCommandConnectClient } from './mailDeliveryCommandClient'
import { getMailDeliveryQueryConnectClient } from './mailDeliveryQueryClient'
import { getMailSyncConnectClient } from './mailSyncClient'

export async function syncMailInbox(
	connectionId: string,
	operationId: string,
): Promise<SyncInboxAcceptedV1> {
	return getMailSyncConnectClient().sync({
		connectionId: requireIdentifier('connection ID', connectionId),
		operationId: requireIdentifier('operation ID', operationId),
	})
}

export async function sendMailMessage(input: {
	connectionId: string
	operationId: string
	providerConversationId: string
	toRecipients: readonly string[]
	ccRecipients: readonly string[]
	bccRecipients: readonly string[]
	subject: string
	textBody: string
}): Promise<string> {
	const toRecipients = normalizedRecipients(input.toRecipients)
	const ccRecipients = normalizedRecipients(input.ccRecipients)
	const bccRecipients = normalizedRecipients(input.bccRecipients)
	if (toRecipients.length === 0) {
		throw new RangeError('Mail recipient is required')
	}
	const textBody = input.textBody.trim()
	if (!textBody) {
		throw new RangeError('Mail body is required')
	}
	const response = await getMailDeliveryCommandConnectClient().send({
		connectionId: requireIdentifier('connection ID', input.connectionId),
		operationId: requireIdentifier('operation ID', input.operationId),
		providerConversationId: input.providerConversationId.trim(),
		recipient: toRecipients,
		ccRecipient: ccRecipients,
		bccRecipient: bccRecipients,
		subject: input.subject.trim(),
		textBody,
		attachmentAnchorId: [],
	})
	return response.operationId
}

function normalizedRecipients(values: readonly string[]): string[] {
	return values.map((recipient) => recipient.trim()).filter(Boolean)
}

export async function getMailDeliveryStatus(
	connectionId: string,
	operationId: string,
): Promise<MailDeliveryOperationStatusV1 | null> {
	const response = await getMailDeliveryQueryConnectClient().getOperationStatus({
		connectionId: requireIdentifier('connection ID', connectionId),
		operationId: requireIdentifier('operation ID', operationId),
	})
	return response.status ?? null
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`Mail ${label} is required`)
	}
	return normalized
}
