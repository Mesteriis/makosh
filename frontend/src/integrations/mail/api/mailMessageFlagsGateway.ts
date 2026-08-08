import { create } from '@bufbuild/protobuf'

import {
	MailMessageFlagCommandV1Schema,
	MailMessageFlagKindV1,
	type MailMessageFlagOperationStatusV1,
	MailMessageFlagStatusRequestV1Schema,
} from '../../../gen/makosh/mail/message_flags/v1/client_pb'
import { getMailMessageFlagCommandConnectClient } from './mailMessageFlagCommandClient'
import { getMailMessageFlagQueryConnectClient } from './mailMessageFlagQueryClient'

const MAX_IDENTIFIER_BYTES = 512
const textEncoder = new TextEncoder()

export type MailMessageFlagMutationInput = {
	operationId: string
	connectionId: string
	messageId: string
	kind: 'read' | 'starred'
	targetValue: boolean
}

export async function mutateMailMessageFlag(
	input: MailMessageFlagMutationInput,
): Promise<string> {
	const response = await getMailMessageFlagCommandConnectClient().mutate(
		create(MailMessageFlagCommandV1Schema, {
			operationId: identifier('operation ID', input.operationId),
			connectionId: identifier('connection ID', input.connectionId),
			messageId: identifier('provider message ID', input.messageId),
			kind: input.kind === 'read'
				? MailMessageFlagKindV1.MAIL_MESSAGE_FLAG_KIND_READ
				: MailMessageFlagKindV1.MAIL_MESSAGE_FLAG_KIND_STARRED,
			targetValue: input.targetValue,
		}),
	)
	return identifier('accepted operation ID', response.operationId)
}

export async function getMailMessageFlagStatus(input: {
	operationId: string
	connectionId: string
}): Promise<MailMessageFlagOperationStatusV1 | undefined> {
	const response = await getMailMessageFlagQueryConnectClient().getOperationStatus(
		create(MailMessageFlagStatusRequestV1Schema, {
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
