import { create } from '@bufbuild/protobuf'
import { nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	AttachmentPreviewContentTypeV1,
	AttachmentPreviewErrorCodeV1,
	AttachmentPreviewStateV1,
	AttachmentPreviewStatusChangedV1Schema,
	GetAttachmentPreviewResponseV1Schema,
	type AttachmentPreviewStatusChangedV1,
	type GetAttachmentPreviewResponseV1,
} from '../../../gen/makosh/attachment_preview/v1/preview_pb'
import type { AttachmentPreviewRealtimeObserverV1 } from '../api/attachmentPreview'
import {
	getAttachmentPreviewStatus,
	readAttachmentPreview,
	startAttachmentPreview,
	subscribeAttachmentPreviewStatus,
} from '../api/attachmentPreview'
import { useAttachmentPreview } from './useAttachmentPreview'
import { startAttachmentPreviewEvidenceReplay } from '../api/attachmentPreviewEvidenceReplay'

vi.mock('../api/attachmentPreview', () => ({
	getAttachmentPreviewStatus: vi.fn(),
	readAttachmentPreview: vi.fn(),
	startAttachmentPreview: vi.fn(),
	subscribeAttachmentPreviewStatus: vi.fn(),
}))
vi.mock('../api/attachmentPreviewEvidenceReplay', () => ({
	startAttachmentPreviewEvidenceReplay: vi.fn(),
}))

const anchorId = new Uint8Array(16).fill(1)
const runId = new Uint8Array(16).fill(2)

describe('useAttachmentPreview', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(startAttachmentPreviewEvidenceReplay).mockResolvedValue()
	})

	it('shows an honest unavailable skeleton without starting a request', async () => {
		const available = ref(false)
		const candidate = ref<Uint8Array | undefined>(anchorId)
		const workflow = useAttachmentPreview(
			() => available.value,
			() => available.value,
			() => candidate.value,
		)
		await nextTick()

		expect(workflow.model.value).toMatchObject({
			visible: true,
			available: false,
			status: 'unavailable',
		})
		expect(startAttachmentPreview).not.toHaveBeenCalled()
	})

	it('uses realtime terminal state and ignores an older reconciliation snapshot', async () => {
		let realtimeObserver: AttachmentPreviewRealtimeObserverV1 | undefined
		let resolveSnapshot: ((value: GetAttachmentPreviewResponseV1) => void) | undefined
		vi.mocked(startAttachmentPreview).mockResolvedValue(runId)
		vi.mocked(subscribeAttachmentPreviewStatus).mockImplementation((_runId, observer) => {
			realtimeObserver = observer
			return { close: vi.fn(), ready: Promise.resolve() }
		})
		vi.mocked(getAttachmentPreviewStatus).mockReturnValue(new Promise(resolve => {
			resolveSnapshot = resolve
		}))
		vi.mocked(readAttachmentPreview).mockResolvedValue({
			bytes: new TextEncoder().encode('safe preview'),
			contentType: AttachmentPreviewContentTypeV1.TEXT_UTF8,
		})
		const available = ref(true)
		const candidate = ref<Uint8Array | undefined>(anchorId)
		const workflow = useAttachmentPreview(
			() => available.value,
			() => available.value,
			() => candidate.value,
		)

		await vi.waitFor(() => expect(realtimeObserver).toBeDefined())
		expect(startAttachmentPreviewEvidenceReplay).toHaveBeenCalledWith(
			anchorId,
			expect.any(Uint8Array),
			expect.any(AbortSignal),
		)
		realtimeObserver?.onStatus(readyStatus(4n))
		await vi.waitFor(() => expect(workflow.model.value.status).toBe('ready'))
		expect(workflow.model.value.artifactText).toBe('safe preview')

		resolveSnapshot?.(create(GetAttachmentPreviewResponseV1Schema, {
			runId,
			attachmentAnchorId: anchorId,
			state: AttachmentPreviewStateV1.ACCEPTED,
			stateRevision: 1n,
			previewKind: 0,
			contentType: AttachmentPreviewContentTypeV1.UNSPECIFIED,
			previewSizeBytes: 0n,
			truncated: false,
			error: AttachmentPreviewErrorCodeV1.UNSPECIFIED,
		}))
		await nextTick()

		expect(workflow.model.value.status).toBe('ready')
		expect(readAttachmentPreview).toHaveBeenCalledTimes(1)
	})
})

function readyStatus(stateRevision: bigint): AttachmentPreviewStatusChangedV1 {
	return create(AttachmentPreviewStatusChangedV1Schema, {
		runId,
		state: AttachmentPreviewStateV1.READY,
		stateRevision,
		previewKind: 1,
		contentType: AttachmentPreviewContentTypeV1.TEXT_UTF8,
		previewSizeBytes: 12n,
		truncated: false,
		occurredAtUnixMillis: 1n,
		error: AttachmentPreviewErrorCodeV1.UNSPECIFIED,
	})
}
