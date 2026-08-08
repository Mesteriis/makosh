import type { AttachmentPreviewContentTypeV1 } from '../../../gen/makosh/attachment_preview/v1/preview_pb'

export type AttachmentPreviewPanelStatus =
	| 'idle'
	| 'unavailable'
	| 'starting'
	| 'awaiting-evidence'
	| 'rendering'
	| 'ready'
	| 'unsupported'
	| 'rejected'
	| 'error'

export type AttachmentPreviewPanelModel = {
	visible: boolean
	available: boolean
	busy: boolean
	status: AttachmentPreviewPanelStatus
	statusMessage: string
	artifactText: string
	artifactUrl: string
	contentType: AttachmentPreviewContentTypeV1
	truncated: boolean
	canRetry: boolean
}
