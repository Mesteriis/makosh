import { fromBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import { ReadMessageBodyRequestV1Schema } from '../../../gen/hermes/communications/content/read/v1/read_pb'
import { readCanonicalCommunicationContent } from './canonicalCommunicationsContent'

describe('canonical Communications content adapter', () => {
	it('exchanges an opaque one-use ticket for exact same-origin body bytes', async () => {
		const messageId = new Uint8Array(16).fill(7)
		const capability = new Uint8Array(32).fill(8)
		const body = new TextEncoder().encode('Exact canonical body')
		const issueTicket = vi.fn().mockResolvedValue({
			$typeName:
				'hermes.communications.content.ticket.v1.IssueMessageBodyReadResponseV1',
			opaqueReadCapability: capability,
			declaredBytes: BigInt(body.byteLength),
			expiresAtUnixSeconds: 100n,
			mediaType: 'text/plain',
			errorCode: '',
		})
		const readBlob = vi.fn().mockResolvedValue(new Response(body, {
			status: 200,
			headers: { 'content-type': 'application/octet-stream' },
		}))

		await expect(readCanonicalCommunicationContent(
			messageId,
			undefined,
			{ issueTicket, readBlob },
		)).resolves.toEqual({ bytes: body, mediaType: 'text/plain' })

		expect(issueTicket).toHaveBeenCalledWith(messageId, undefined)
		expect(readBlob).toHaveBeenCalledWith(
			'/api/blobs/communications/v1/message-body',
			expect.objectContaining({
				method: 'POST',
				headers: {
					accept: 'application/octet-stream',
					'content-type': 'application/protobuf',
				},
			}),
		)
		const request = fromBinary(
			ReadMessageBodyRequestV1Schema,
			new Uint8Array(readBlob.mock.calls[0]?.[1].body),
		)
		expect(request.protocolMajor).toBe(1)
		expect(request.opaqueReadCapability).toEqual(capability)
	})

	it('fails closed for invalid IDs, tickets, media types and lengths', async () => {
		const ticket = {
			$typeName:
				'hermes.communications.content.ticket.v1.IssueMessageBodyReadResponseV1',
			opaqueReadCapability: new Uint8Array(32).fill(1),
			declaredBytes: 3n,
			expiresAtUnixSeconds: 100n,
			mediaType: 'text/plain',
			errorCode: '',
		}
		const issueTicket = vi.fn().mockResolvedValue(ticket)
		const readBlob = vi.fn()

		await expect(readCanonicalCommunicationContent(
			new Uint8Array(15),
			undefined,
			{ issueTicket, readBlob },
		)).rejects.toThrow(RangeError)

		readBlob.mockResolvedValueOnce(new Response(new Uint8Array([1, 2, 3]), {
			status: 200,
			headers: { 'content-type': 'text/plain' },
		}))
		await expect(readCanonicalCommunicationContent(
			new Uint8Array(16),
			undefined,
			{ issueTicket, readBlob },
		)).rejects.toThrow('unavailable')

		readBlob.mockResolvedValueOnce(new Response(new Uint8Array([1, 2]), {
			status: 200,
			headers: { 'content-type': 'application/octet-stream' },
		}))
		await expect(readCanonicalCommunicationContent(
			new Uint8Array(16),
			undefined,
			{ issueTicket, readBlob },
		)).rejects.toThrow('length')
	})
})
