import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	CommunicationsExportErrorCodeV1,
	EvidenceExportStatusV1,
	EvidenceExportStatusChangedV1Schema,
	GetEvidenceExportStatusResponseV1Schema,
	IssueEvidenceExportReadResponseV1Schema,
	StartEvidenceExportResponseV1Schema,
} from '../../../gen/makosh/communications_export/v1/export_pb'
import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateKindV1,
	ClientRealtimeStreamStateV1Schema,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import {
	getCommunicationsEvidenceExportStatus,
	openCommunicationsEvidenceExportRealtime,
	readCommunicationsEvidenceExport,
	startCommunicationsEvidenceExport,
} from './communicationsEvidenceExport'

const id = (byte: number): Uint8Array => new Uint8Array(16).fill(byte)

function ports() {
	return {
		start: vi.fn(async (_messageIds: Uint8Array[], operationId: Uint8Array) => create(
			StartEvidenceExportResponseV1Schema,
			{ exportId: operationId, error: CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_UNSPECIFIED },
		)),
		status: vi.fn(async (exportId: Uint8Array) => create(
			GetEvidenceExportStatusResponseV1Schema,
			{
				exportId,
				status: EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_READY,
				requestedItems: 1,
				completedItems: 1,
				artifactBytes: 3n,
			},
		)),
		issueRead: vi.fn(async () => create(
			IssueEvidenceExportReadResponseV1Schema,
			{
				opaqueReadCapability: new Uint8Array(32).fill(9),
				declaredBytes: 3n,
				expiresAtUnixSeconds: BigInt(Math.floor(Date.now() / 1_000) + 30),
			},
		)),
		readBlob: vi.fn(async (_input: RequestInfo | URL, _init: RequestInit) => new Response(new Uint8Array([1, 2, 3]), {
			headers: { 'content-type': 'application/octet-stream' },
		})),
	}
}

describe('Communications evidence export client', () => {
	it('opens shared realtime before Start and replays a buffered exact-export status', async () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const close = vi.fn()
		const hub = {
			subscribe: vi.fn((observer: BrowserGatewayRealtimeObserver) => {
				sourceObserver = observer
				return { close }
			}),
		}
		const observer = { onStatus: vi.fn(), onUnavailable: vi.fn() }
		const binding = openCommunicationsEvidenceExportRealtime(observer, hub)
		sourceObserver?.onStreamState(create(ClientRealtimeStreamStateV1Schema, {
			state: ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN,
		}))
		await expect(binding.ready).resolves.toBeUndefined()

		const exportId = id(7)
		const status = create(EvidenceExportStatusChangedV1Schema, {
			exportId,
			status: EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_READY,
			requestedItems: 2,
			completedItems: 2,
			artifactBytes: 3n,
			occurredAtUnixMillis: 1n,
		})
		sourceObserver?.onEvent(create(ClientRealtimeEventV1Schema, {
			contractName: 'communications.export.status_changed',
			contractVersion: 1,
			eventKind: 'communications.export.status_changed',
			payload: toBinary(EvidenceExportStatusChangedV1Schema, status),
		}))
		expect(observer.onStatus).not.toHaveBeenCalled()
		binding.attachExport(exportId)
		expect(observer.onStatus).toHaveBeenCalledWith(status)
		binding.close()
		expect(close).toHaveBeenCalledTimes(1)
	})

	it('uses an exact stable operation ID and bounded canonical message set', async () => {
		const adapter = ports()
		const exportId = await startCommunicationsEvidenceExport([id(1), id(2)], id(7), undefined, adapter)
		expect(exportId).toEqual(id(7))
		expect(adapter.start).toHaveBeenCalledOnce()
		await expect(
			startCommunicationsEvidenceExport([id(1), id(1)], id(7), undefined, adapter),
		).rejects.toThrow('unique')
	})

	it('validates status and sends only a one-use capability to client_blob', async () => {
		const adapter = ports()
		const status = await getCommunicationsEvidenceExportStatus(id(7), undefined, adapter)
		expect(status.status).toBe(EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_READY)
		const bytes = await readCommunicationsEvidenceExport(id(7), undefined, adapter)
		expect(bytes).toEqual(new Uint8Array([1, 2, 3]))
		expect(adapter.readBlob).toHaveBeenCalledWith(
			'/api/blobs/communications-export/v1/artifact',
			expect.objectContaining({ method: 'POST' }),
		)
		const request = adapter.readBlob.mock.calls[0]?.[1]
		expect(new Uint8Array(request?.body as Uint8Array)).not.toContain(7)
	})

	it('fails closed on expired tickets and response length drift', async () => {
		const expired = ports()
		expired.issueRead.mockResolvedValueOnce(create(
			IssueEvidenceExportReadResponseV1Schema,
			{
				opaqueReadCapability: new Uint8Array(32).fill(9),
				declaredBytes: 3n,
				expiresAtUnixSeconds: 1n,
			},
		))
		await expect(readCommunicationsEvidenceExport(id(7), undefined, expired)).rejects.toThrow('ticket')

		const truncated = ports()
		truncated.readBlob.mockResolvedValueOnce(new Response(new Uint8Array([1, 2]), {
			headers: { 'content-type': 'application/octet-stream' },
		}))
		await expect(readCommunicationsEvidenceExport(id(7), undefined, truncated)).rejects.toThrow('length')
	})
})
