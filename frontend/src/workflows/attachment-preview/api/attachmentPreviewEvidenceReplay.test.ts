import { describe, expect, it, vi } from 'vitest'

import {
	AttachmentPreviewEvidenceReplayErrorV1,
	AttachmentPreviewEvidenceReplayStateV1,
} from '../../../gen/makosh/attachment_preview_evidence_replay/v1/replay_pb'
import { startAttachmentPreviewEvidenceReplay } from './attachmentPreviewEvidenceReplay'

const anchorId = new Uint8Array(16).fill(1)
const operationId = new Uint8Array(16).fill(2)

describe('attachment preview evidence replay browser adapter', () => {
	it('sends only the provider-neutral operation and attachment anchor', async () => {
		const start = vi.fn().mockResolvedValue({
			operationId,
			state: AttachmentPreviewEvidenceReplayStateV1.AWAITING_PRODUCERS,
			error: AttachmentPreviewEvidenceReplayErrorV1.UNSPECIFIED,
		})

		await expect(startAttachmentPreviewEvidenceReplay(
			anchorId,
			operationId,
			undefined,
			{ start },
		)).resolves.toBeUndefined()
		expect(start).toHaveBeenCalledWith(operationId, anchorId, undefined)
	})

	it('fails closed on a terminal producer failure', async () => {
		const start = vi.fn().mockResolvedValue({
			operationId,
			state: AttachmentPreviewEvidenceReplayStateV1.UNAVAILABLE,
			error: AttachmentPreviewEvidenceReplayErrorV1.PRODUCER_UNAVAILABLE,
		})

		await expect(startAttachmentPreviewEvidenceReplay(
			anchorId,
			operationId,
			undefined,
			{ start },
		)).rejects.toThrow('Retained attachment evidence replay was not accepted')
	})
})
