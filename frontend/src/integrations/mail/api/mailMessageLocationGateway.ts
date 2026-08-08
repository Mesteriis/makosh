import { create } from '@bufbuild/protobuf'

import {
	MailMessageLocationCommandV1Schema,
	MailMessageLocationKindV1,
	type MailMessageLocationOperationStatusV1,
	MailMessageLocationStatusRequestV1Schema,
} from '../../../gen/makosh/mail/message_location/v1/client_pb'
import { getMailMessageLocationCommandConnectClient } from './mailMessageLocationCommandClient'
import { getMailMessageLocationQueryConnectClient } from './mailMessageLocationQueryClient'

const MAX_IDENTIFIER_BYTES = 512
const textEncoder = new TextEncoder()

export type MailMessageLocationMutationInput = {
	operationId: string
	connectionId: string
	messageId: string
	kind: 'archive' | 'trash' | 'restore' | 'move'
	targetFolderId?: string
}

export async function mutateMailMessageLocation(
	input: MailMessageLocationMutationInput,
): Promise<string> {
	const response = await getMailMessageLocationCommandConnectClient().mutate(
		create(MailMessageLocationCommandV1Schema, {
			operationId: identifier('operation ID', input.operationId),
			connectionId: identifier('connection ID', input.connectionId),
			messageId: identifier('message ID', input.messageId),
			kind: locationKind(input.kind),
			targetFolderId: input.targetFolderId
				? identifier('target folder ID', input.targetFolderId)
				: undefined,
		}),
	)
	return identifier('accepted operation ID', response.operationId)
}

export async function getMailMessageLocationStatus(input: {
	operationId: string
	connectionId: string
}): Promise<MailMessageLocationOperationStatusV1 | undefined> {
	const response = await getMailMessageLocationQueryConnectClient().getOperationStatus(
		create(MailMessageLocationStatusRequestV1Schema, {
			operationId: identifier('operation ID', input.operationId),
			connectionId: identifier('connection ID', input.connectionId),
		}),
	)
	return response.status
}

function locationKind(kind: MailMessageLocationMutationInput['kind']): MailMessageLocationKindV1 {
	if (kind === 'archive') return MailMessageLocationKindV1.MAIL_MESSAGE_LOCATION_KIND_ARCHIVE
	if (kind === 'trash') return MailMessageLocationKindV1.MAIL_MESSAGE_LOCATION_KIND_TRASH
	if (kind === 'restore') return MailMessageLocationKindV1.MAIL_MESSAGE_LOCATION_KIND_RESTORE
	return MailMessageLocationKindV1.MAIL_MESSAGE_LOCATION_KIND_MOVE
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
