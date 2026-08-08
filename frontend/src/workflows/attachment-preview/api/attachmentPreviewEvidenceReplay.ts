import {
	AttachmentPreviewEvidenceReplayErrorV1,
	AttachmentPreviewEvidenceReplayStateV1,
} from '../../../gen/makosh/attachment_preview_evidence_replay/v1/replay_pb'
import { getAttachmentPreviewEvidenceReplayCommandClient } from '../../../platform/connect/attachmentPreviewEvidenceReplayCommandClient'

const ID_BYTES = 16

type AttachmentPreviewEvidenceReplayPort = {
	start(
		operationId: Uint8Array,
		attachmentAnchorId: Uint8Array,
		signal?: AbortSignal,
	): Promise<{
		operationId: Uint8Array
		state: AttachmentPreviewEvidenceReplayStateV1
		error: AttachmentPreviewEvidenceReplayErrorV1
	}>
}

export async function startAttachmentPreviewEvidenceReplay(
	attachmentAnchorId: Uint8Array,
	operationId: Uint8Array,
	signal?: AbortSignal,
	port: AttachmentPreviewEvidenceReplayPort = defaultPort(),
): Promise<void> {
	validateId(attachmentAnchorId, 'Attachment anchor')
	validateId(operationId, 'Replay operation')
	const response = await port.start(copy(operationId), copy(attachmentAnchorId), signal)
	const accepted = response.state === AttachmentPreviewEvidenceReplayStateV1.ACCEPTED
		|| response.state === AttachmentPreviewEvidenceReplayStateV1.AWAITING_PRODUCERS
		|| response.state === AttachmentPreviewEvidenceReplayStateV1.COMPLETED
	if (
		!equal(response.operationId, operationId)
		|| !accepted
		|| response.error !== AttachmentPreviewEvidenceReplayErrorV1.UNSPECIFIED
	) {
		throw new Error('Retained attachment evidence replay was not accepted')
	}
}

function defaultPort(): AttachmentPreviewEvidenceReplayPort {
	return {
		start: (operationId, attachmentAnchorId, signal) =>
			getAttachmentPreviewEvidenceReplayCommandClient().start(
				{ protocolMajor: 1, operationId, attachmentAnchorId },
				{ signal },
			),
	}
}

function validateId(value: Uint8Array, label: string): void {
	if (value.byteLength !== ID_BYTES || !value.some(byte => byte !== 0)) {
		throw new RangeError(`${label} ID must be ${ID_BYTES} non-zero bytes`)
	}
}

function equal(left: Uint8Array, right: Uint8Array): boolean {
	return left.byteLength === right.byteLength && left.every((byte, index) => byte === right[index])
}

function copy(value: Uint8Array): Uint8Array {
	return new Uint8Array(value)
}
