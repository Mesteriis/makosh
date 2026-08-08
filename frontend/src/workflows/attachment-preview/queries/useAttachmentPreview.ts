import { computed, onUnmounted, ref, watch, type ComputedRef } from 'vue'

import {
	AttachmentPreviewContentTypeV1,
	AttachmentPreviewErrorCodeV1,
	AttachmentPreviewStateV1,
	type AttachmentPreviewStatusChangedV1,
	type GetAttachmentPreviewResponseV1,
} from '../../../gen/makosh/attachment_preview/v1/preview_pb'
import {
	getAttachmentPreviewStatus,
	readAttachmentPreview,
	startAttachmentPreview,
	subscribeAttachmentPreviewStatus,
	type AttachmentPreviewRealtimeSubscriptionV1,
} from '../api/attachmentPreview'
import { startAttachmentPreviewEvidenceReplay } from '../api/attachmentPreviewEvidenceReplay'
import type {
	AttachmentPreviewPanelModel,
	AttachmentPreviewPanelStatus,
} from '../presentation/attachmentPreviewPanelModel'

const ID_BYTES = 16

export function useAttachmentPreview(
	canPreview: () => boolean,
	canReplayEvidence: () => boolean,
	candidateAnchorId: () => Uint8Array | undefined,
): {
	model: ComputedRef<AttachmentPreviewPanelModel>
	retry: () => Promise<void>
} {
	const status = ref<AttachmentPreviewPanelStatus>('idle')
	const statusMessage = ref('')
	const contentType = ref(AttachmentPreviewContentTypeV1.UNSPECIFIED)
	const truncated = ref(false)
	const artifactText = ref('')
	const artifactUrl = ref('')
	let generation = 0
	let appliedStateRevision = 0n
	let requestController: AbortController | undefined
	let realtimeSubscription: AttachmentPreviewRealtimeSubscriptionV1 | undefined

	const model = computed<AttachmentPreviewPanelModel>(() => {
		const candidate = candidateAnchorId()
		const visible = validId(candidate)
		const previewAvailable = canPreview()
		const replayAvailable = canReplayEvidence()
		const available = previewAvailable && replayAvailable
		return {
			visible,
			available,
			busy: ['starting', 'awaiting-evidence', 'rendering'].includes(status.value),
			status: visible && !available ? 'unavailable' : status.value,
			statusMessage: visible && !available
				? previewAvailable
					? 'Retained evidence replay is not admitted for this runtime.'
					: 'Attachment Preview is not admitted for this runtime.'
				: statusMessage.value,
			artifactText: artifactText.value,
			artifactUrl: artifactUrl.value,
			contentType: contentType.value,
			truncated: truncated.value,
			canRetry: visible && available && ['unsupported', 'rejected', 'error'].includes(status.value),
		}
	})

	watch(
		[candidateAnchorId, canPreview, canReplayEvidence],
		([candidate, previewAvailable, replayAvailable]) => {
			cancelCurrent()
			resetArtifact()
			if (!validId(candidate)) {
				status.value = 'idle'
				statusMessage.value = ''
				return
			}
			if (!previewAvailable || !replayAvailable) {
				status.value = 'unavailable'
				return
			}
			void start(candidate)
		},
		{ immediate: true },
	)

	onUnmounted(() => {
		cancelCurrent()
		resetArtifact()
	})

	async function retry(): Promise<void> {
		const candidate = candidateAnchorId()
		if (!canPreview() || !canReplayEvidence() || !validId(candidate)) return
		cancelCurrent()
		resetArtifact()
		await start(candidate)
	}

	async function start(anchorId: Uint8Array): Promise<void> {
		const currentGeneration = ++generation
		const controller = new AbortController()
		requestController = controller
		status.value = 'starting'
		statusMessage.value = 'Submitting a bounded preview request…'
		try {
			const previewOperationId = crypto.getRandomValues(new Uint8Array(ID_BYTES))
			const runId = await startAttachmentPreview(anchorId, previewOperationId, controller.signal)
			if (currentGeneration !== generation) return
			realtimeSubscription = subscribeAttachmentPreviewStatus(runId, {
				onStatus: next => void handleRealtimeStatus(next, currentGeneration, controller.signal),
				onUnavailable: () => failRealtime(currentGeneration),
			})
			await realtimeSubscription.ready
			if (currentGeneration !== generation) return
			const replayOperationId = crypto.getRandomValues(new Uint8Array(ID_BYTES))
			await startAttachmentPreviewEvidenceReplay(anchorId, replayOperationId, controller.signal)
			if (currentGeneration !== generation) return
			const snapshot = await getAttachmentPreviewStatus(runId, controller.signal)
			if (currentGeneration === generation) await applySnapshot(snapshot, currentGeneration, controller.signal)
		} catch {
			if (currentGeneration !== generation || controller.signal.aborted) return
			fail(realtimeSubscription
				? 'Retained attachment evidence could not be recovered.'
				: 'Attachment preview could not be started.')
		}
	}

	async function applyRealtimeStatus(
		next: AttachmentPreviewStatusChangedV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		if (currentGeneration !== generation) return
		await applyState(
			next.state,
			next.stateRevision,
			next.contentType,
			next.truncated,
			next.error,
			next.runId,
			currentGeneration,
			signal,
		)
	}

	async function handleRealtimeStatus(
		next: AttachmentPreviewStatusChangedV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		try {
			await applyRealtimeStatus(next, currentGeneration, signal)
		} catch {
			if (currentGeneration === generation && !signal.aborted) {
				fail('Attachment preview artifact is unavailable.')
			}
		}
	}

	async function applySnapshot(
		next: GetAttachmentPreviewResponseV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		await applyState(
			next.state,
			next.stateRevision,
			next.contentType,
			next.truncated,
			next.error,
			next.runId,
			currentGeneration,
			signal,
		)
	}

	async function applyState(
		nextState: AttachmentPreviewStateV1,
		stateRevision: bigint,
		nextContentType: AttachmentPreviewContentTypeV1,
		isTruncated: boolean,
		error: AttachmentPreviewErrorCodeV1,
		runId: Uint8Array,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		if (currentGeneration !== generation || stateRevision <= appliedStateRevision) return
		appliedStateRevision = stateRevision
		if (nextState === AttachmentPreviewStateV1.READY) {
			await loadArtifact(runId, nextContentType, isTruncated, currentGeneration, signal)
			return
		}
		if (nextState === AttachmentPreviewStateV1.ACCEPTED) {
			setProgress('starting', 'Preview request accepted.')
			return
		}
		if (nextState === AttachmentPreviewStateV1.AWAITING_EVIDENCE) {
			setProgress('awaiting-evidence', 'Waiting for current safe attachment evidence…')
			return
		}
		if (nextState === AttachmentPreviewStateV1.RENDERING) {
			setProgress('rendering', 'Rendering a bounded derived preview…')
			return
		}
		if (nextState === AttachmentPreviewStateV1.UNSUPPORTED) {
			setTerminal('unsupported', previewErrorMessage(error, 'This attachment format is not supported.'))
			return
		}
		if (nextState === AttachmentPreviewStateV1.REJECTED) {
			setTerminal('rejected', previewErrorMessage(error, 'Attachment preview was rejected.'))
			return
		}
		fail('Attachment preview returned an invalid state.')
	}

	async function loadArtifact(
		runId: Uint8Array,
		nextContentType: AttachmentPreviewContentTypeV1,
		isTruncated: boolean,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		const artifact = await readAttachmentPreview(runId, signal)
		if (currentGeneration !== generation) return
		if (artifact.contentType !== nextContentType) {
			fail('Attachment preview content type changed before read.')
			return
		}
		resetArtifact()
		contentType.value = artifact.contentType
		truncated.value = isTruncated
		if (artifact.contentType === AttachmentPreviewContentTypeV1.TEXT_UTF8) {
			try {
				artifactText.value = new TextDecoder('utf-8', { fatal: true }).decode(artifact.bytes)
			} catch {
				fail('Attachment preview text is not valid UTF-8.')
				return
			}
		} else {
			artifactUrl.value = URL.createObjectURL(new Blob([ownedArrayBuffer(artifact.bytes)], {
				type: browserContentType(artifact.contentType),
			}))
		}
		setTerminal('ready', isTruncated ? 'Preview is ready and was safely truncated.' : 'Preview is ready.')
	}

	function failRealtime(currentGeneration: number): void {
		if (currentGeneration !== generation) return
		fail('Realtime preview status is unavailable. Retry to reconcile safely.')
	}

	function setProgress(nextStatus: AttachmentPreviewPanelStatus, message: string): void {
		status.value = nextStatus
		statusMessage.value = message
	}

	function setTerminal(nextStatus: AttachmentPreviewPanelStatus, message: string): void {
		realtimeSubscription?.close()
		realtimeSubscription = undefined
		status.value = nextStatus
		statusMessage.value = message
	}

	function fail(message: string): void {
		setTerminal('error', message)
	}

	function cancelCurrent(): void {
		generation += 1
		appliedStateRevision = 0n
		requestController?.abort()
		requestController = undefined
		realtimeSubscription?.close()
		realtimeSubscription = undefined
	}

	function resetArtifact(): void {
		if (artifactUrl.value) URL.revokeObjectURL(artifactUrl.value)
		artifactUrl.value = ''
		artifactText.value = ''
		contentType.value = AttachmentPreviewContentTypeV1.UNSPECIFIED
		truncated.value = false
	}

	return { model, retry }
}

function validId(value: Uint8Array | undefined): value is Uint8Array {
	return Boolean(value && value.byteLength === ID_BYTES && value.some(byte => byte !== 0))
}

function previewErrorMessage(error: AttachmentPreviewErrorCodeV1, fallback: string): string {
	if (error === AttachmentPreviewErrorCodeV1.NOT_SAFE) return 'Current attachment evidence is not safe for preview.'
	if (error === AttachmentPreviewErrorCodeV1.SOURCE_TOO_LARGE) return 'Attachment exceeds the bounded preview limit.'
	if (error === AttachmentPreviewErrorCodeV1.INVALID_CONTENT) return 'Attachment content is malformed.'
	if (error === AttachmentPreviewErrorCodeV1.RENDERER_UNAVAILABLE) return 'The admitted preview renderer is unavailable.'
	return fallback
}

function browserContentType(contentType: AttachmentPreviewContentTypeV1): string {
	if (contentType === AttachmentPreviewContentTypeV1.PNG) return 'image/png'
	if (contentType === AttachmentPreviewContentTypeV1.MPEG_AUDIO) return 'audio/mpeg'
	if (contentType === AttachmentPreviewContentTypeV1.MP4_VIDEO) return 'video/mp4'
	throw new Error('Unsupported attachment preview content type')
}

function ownedArrayBuffer(bytes: Uint8Array): ArrayBuffer {
	const buffer = new ArrayBuffer(bytes.byteLength)
	new Uint8Array(buffer).set(bytes)
	return buffer
}
