import { create } from '@bufbuild/protobuf'

import {
	MailMessagePermanentDeleteCommandV1Schema,
	MailMessagePermanentDeleteConfirmationV1,
	type MailMessagePermanentDeleteOperationStatusV1,
	MailMessagePermanentDeleteStatusRequestV1Schema,
} from '../../../gen/makosh/mail/message_permanent_delete/v1/client_pb'
import { getMailMessagePermanentDeleteCommandConnectClient } from './mailMessagePermanentDeleteCommandClient'
import { getMailMessagePermanentDeleteQueryConnectClient } from './mailMessagePermanentDeleteQueryClient'

const MAX_IDENTIFIER_BYTES = 512
const textEncoder = new TextEncoder()

export async function permanentlyDeleteMailMessage(input: {
	operationId: string
	connectionId: string
	messageId: string
	expectedProjectionRevision: bigint
	confirmed: true
}): Promise<string> {
	if (input.expectedProjectionRevision <= 0n || input.confirmed !== true) {
		throw new RangeError('Mail permanent delete requires current projection and confirmation')
	}
	const response = await getMailMessagePermanentDeleteCommandConnectClient().mutate(create(
		MailMessagePermanentDeleteCommandV1Schema,
		{
			operationId: identifier('operation ID', input.operationId),
			connectionId: identifier('connection ID', input.connectionId),
			messageId: identifier('message ID', input.messageId),
			expectedProjectionRevision: input.expectedProjectionRevision,
			confirmation:
				MailMessagePermanentDeleteConfirmationV1.MAIL_MESSAGE_PERMANENT_DELETE_CONFIRMATION_CONFIRMED,
		},
	))
	return identifier('accepted operation ID', response.operationId)
}

export async function getMailMessagePermanentDeleteStatus(input: {
	operationId: string
	connectionId: string
}): Promise<MailMessagePermanentDeleteOperationStatusV1 | undefined> {
	const response = await getMailMessagePermanentDeleteQueryConnectClient().getOperationStatus(
		create(MailMessagePermanentDeleteStatusRequestV1Schema, {
			operationId: identifier('operation ID', input.operationId),
			connectionId: identifier('connection ID', input.connectionId),
		}),
	)
	return response.status
}

function identifier(label: string, value: string): string {
	const normalized = value.trim()
	if (
		!normalized
		|| textEncoder.encode(normalized).length > MAX_IDENTIFIER_BYTES
		|| hasControlCharacter(normalized)
	) {
		throw new RangeError(`Mail ${label} is invalid`)
	}
	return normalized
}

function hasControlCharacter(value: string): boolean {
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return true
	}
	return false
}
