import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	AttachmentPreviewContentTypeV1,
	AttachmentPreviewErrorCodeV1,
	AttachmentPreviewStateV1,
	AttachmentPreviewStatusChangedV1Schema,
} from '../../../gen/makosh/attachment_preview/v1/preview_pb'
import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateKindV1,
	ClientRealtimeStreamStateV1Schema,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import {
	getAttachmentPreviewStatus,
	readAttachmentPreview,
	startAttachmentPreview,
	subscribeAttachmentPreviewStatus,
} from './attachmentPreview'

const anchorId = filledId(1)
const operationId = filledId(2)
const runId = filledId(3)

describe('attachment preview browser adapter', () => {
	it('starts and queries only through the typed workflow ports', async () => {
		const ports = fixturePorts()

		await expect(startAttachmentPreview(anchorId, operationId, undefined, ports)).resolves.toEqual(runId)
		await expect(getAttachmentPreviewStatus(runId, undefined, ports)).resolves.toMatchObject({
			runId,
			state: AttachmentPreviewStateV1.READY,
		})
		expect(ports.start).toHaveBeenCalledWith(operationId, anchorId, undefined)
		expect(ports.get).toHaveBeenCalledWith(runId, undefined)
	})

	it('reads an exact one-use client_blob response with no-store', async () => {
		const ports = fixturePorts()
		const artifact = await readAttachmentPreview(runId, undefined, ports)

		expect(artifact).toEqual({
			bytes: new Uint8Array([104, 105]),
			contentType: AttachmentPreviewContentTypeV1.TEXT_UTF8,
		})
		expect(ports.readBlob).toHaveBeenCalledWith(
			'/api/blobs/attachment-preview/v1/artifact',
			expect.objectContaining({ method: 'POST', body: expect.any(Uint8Array) }),
		)
	})

	it('fails closed on stale ticket and mismatched artifact length', async () => {
		const stale = fixturePorts()
		stale.nowUnixSeconds.mockReturnValue(101n)
		await expect(readAttachmentPreview(runId, undefined, stale)).rejects.toThrow(
			'Attachment preview read ticket is invalid',
		)

		const mismatch = fixturePorts()
		mismatch.readBlob.mockResolvedValue(new Response(new Uint8Array([1]), {
			status: 200,
			headers: { 'cache-control': 'no-store' },
		}))
		await expect(readAttachmentPreview(runId, undefined, mismatch)).rejects.toThrow(
			'Attachment preview artifact length is invalid',
		)
	})

	it('waits for the shared stream and filters events by exact contract and run', async () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const close = vi.fn()
		const hub = {
			subscribe: vi.fn((observer: BrowserGatewayRealtimeObserver) => {
				sourceObserver = observer
				return { close }
			}),
		}
		const observer = { onStatus: vi.fn(), onUnavailable: vi.fn() }
		const subscription = subscribeAttachmentPreviewStatus(runId, observer, hub)
		sourceObserver?.onStreamState(create(ClientRealtimeStreamStateV1Schema, {
			state: ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN,
		}))
		await expect(subscription.ready).resolves.toBeUndefined()
		const status = create(AttachmentPreviewStatusChangedV1Schema, {
			runId,
			state: AttachmentPreviewStateV1.READY,
			stateRevision: 1n,
			contentType: AttachmentPreviewContentTypeV1.TEXT_UTF8,
			previewSizeBytes: 2n,
		})

		sourceObserver?.onEvent(create(ClientRealtimeEventV1Schema, {
			contractName: 'attachment_preview.realtime',
			contractVersion: 1,
			eventKind: 'attachment_preview.status_changed.v1',
			payload: toBinary(AttachmentPreviewStatusChangedV1Schema, status),
		}))
		sourceObserver?.onEvent(create(ClientRealtimeEventV1Schema, {
			contractName: 'another.contract',
			contractVersion: 1,
			eventKind: 'attachment_preview.status_changed.v1',
			payload: toBinary(AttachmentPreviewStatusChangedV1Schema, status),
		}))

		expect(observer.onStatus).toHaveBeenCalledTimes(1)
		expect(observer.onStatus).toHaveBeenCalledWith(status)
		expect(observer.onUnavailable).not.toHaveBeenCalled()
		subscription.close()
		expect(close).toHaveBeenCalledTimes(1)
	})
})

function fixturePorts() {
	return {
		start: vi.fn().mockResolvedValue({
			runId,
			state: AttachmentPreviewStateV1.ACCEPTED,
			error: AttachmentPreviewErrorCodeV1.UNSPECIFIED,
		}),
		get: vi.fn().mockResolvedValue({
			runId,
			attachmentAnchorId: anchorId,
			state: AttachmentPreviewStateV1.READY,
			stateRevision: 2n,
			contentType: AttachmentPreviewContentTypeV1.TEXT_UTF8,
			previewSizeBytes: 2n,
			error: AttachmentPreviewErrorCodeV1.UNSPECIFIED,
		}),
		issueRead: vi.fn().mockResolvedValue({
			runId,
			opaqueReadTicket: new Uint8Array(32).fill(7),
			expiresAtUnixSeconds: 100n,
			contentType: AttachmentPreviewContentTypeV1.TEXT_UTF8,
			previewSizeBytes: 2n,
			error: AttachmentPreviewErrorCodeV1.UNSPECIFIED,
		}),
		readBlob: vi.fn().mockResolvedValue(new Response(new Uint8Array([104, 105]), {
			status: 200,
			headers: { 'cache-control': 'no-store' },
		})),
		nowUnixSeconds: vi.fn().mockReturnValue(50n),
	}
}

function filledId(value: number): Uint8Array {
	return new Uint8Array(16).fill(value)
}
