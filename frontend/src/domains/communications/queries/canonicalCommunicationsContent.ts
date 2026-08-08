import { create, toBinary } from '@bufbuild/protobuf'

import {
	ReadMessageBodyRequestV1Schema,
} from '../../../gen/makosh/communications/content/read/v1/read_pb'
import type {
	IssueMessageBodyReadResponseV1,
} from '../../../gen/makosh/communications/content/ticket/v1/ticket_pb'
import { getCommunicationsContentTicketConnectClient } from '../../../platform/connect/communicationsContentTicketClient'
import { BrowserGatewayFetch } from '../../../platform/gateway/browserGatewayFetch'

const CONTENT_READ_PATH = '/api/blobs/communications/v1/message-body'
const CANONICAL_ID_BYTES = 16
const CONTENT_TICKET_BYTES = 32
const MAX_MESSAGE_BODY_BYTES = 256 * 1024

type CanonicalCommunicationsContentPorts = {
	issueTicket(
		messageId: Uint8Array,
		signal?: AbortSignal,
	): Promise<IssueMessageBodyReadResponseV1>
	readBlob(input: RequestInfo | URL, init: RequestInit): Promise<Response>
}

export type CanonicalCommunicationContent = {
	bytes: Uint8Array
	mediaType: 'text/plain' | 'text/html'
}

export async function readCanonicalCommunicationContent(
	messageId: Uint8Array,
	signal?: AbortSignal,
	ports: CanonicalCommunicationsContentPorts = defaultPorts(),
): Promise<CanonicalCommunicationContent> {
	if (messageId.byteLength !== CANONICAL_ID_BYTES) {
		throw new RangeError(`Canonical Communications message ID must be ${CANONICAL_ID_BYTES} bytes`)
	}
	const ticket = await ports.issueTicket(messageId, signal)
	const declaredBytes = Number(ticket.declaredBytes)
	if (
		ticket.errorCode
		|| ticket.opaqueReadCapability.byteLength !== CONTENT_TICKET_BYTES
		|| !Number.isSafeInteger(declaredBytes)
		|| declaredBytes < 1
		|| declaredBytes > MAX_MESSAGE_BODY_BYTES
		|| ticket.expiresAtUnixSeconds <= 0n
		|| (ticket.mediaType !== 'text/plain' && ticket.mediaType !== 'text/html')
	) {
		throw new Error('Canonical communication content ticket is unavailable')
	}
	const request = create(ReadMessageBodyRequestV1Schema, {
		protocolMajor: 1,
		opaqueReadCapability: ticket.opaqueReadCapability,
	})
	const response = await ports.readBlob(CONTENT_READ_PATH, {
		method: 'POST',
		headers: {
			accept: 'application/octet-stream',
			'content-type': 'application/protobuf',
		},
		body: toBinary(ReadMessageBodyRequestV1Schema, request),
		signal,
	})
	if (
		!response.ok
		|| response.headers.get('content-type')?.split(';', 1)[0] !== 'application/octet-stream'
	) {
		throw new Error('Canonical communication content is unavailable')
	}
	const content = new Uint8Array(await response.arrayBuffer())
	if (content.byteLength !== declaredBytes || content.byteLength > MAX_MESSAGE_BODY_BYTES) {
		throw new Error('Canonical communication content length is invalid')
	}
	return { bytes: content, mediaType: ticket.mediaType }
}

function defaultPorts(): CanonicalCommunicationsContentPorts {
	const browserGateway = new BrowserGatewayFetch()
	return {
		issueTicket: (messageId, signal) => (
			getCommunicationsContentTicketConnectClient().issueMessageBodyRead(
				{ protocolMajor: 1, messageId },
				{ signal },
			)
		),
		readBlob: browserGateway.fetch.bind(browserGateway),
	}
}
