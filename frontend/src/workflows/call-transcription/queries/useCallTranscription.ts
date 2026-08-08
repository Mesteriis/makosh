import { computed, onUnmounted, ref, watch, type ComputedRef } from 'vue'

import {
	CallTranscriptionErrorCodeV1,
	CallTranscriptionLanguageV1,
	CallTranscriptionStateV1,
	type CallTranscriptionArtifactV1,
	type CallTranscriptionStatusChangedV1,
	type GetCallTranscriptionResponseV1,
} from '../../../gen/makosh/call_transcription/v1/transcription_pb'
import {
	getCallTranscriptionStatus,
	openCallTranscriptionRealtime,
	readCallTranscript,
	startCallTranscription,
	type CallTranscriptionRealtimeBindingV1,
	type CallTranscriptionSourceV1,
} from '../api/callTranscription'
import type {
	CallTranscriptionPanelModel,
	CallTranscriptionPanelStatus,
} from '../presentation/callTranscriptionPanelModel'

export function useCallTranscription(
	canTranscribe: () => boolean,
	selectedSource: () => CallTranscriptionSourceV1 | undefined,
): {
	model: ComputedRef<CallTranscriptionPanelModel>
	retry: () => Promise<void>
} {
	const status = ref<CallTranscriptionPanelStatus>('waiting-source')
	const statusMessage = ref('Select a recorded call with current consent evidence to transcribe it.')
	const transcriptText = ref('')
	const detectedLanguage = ref('')
	const durationLabel = ref('')
	let generation = 0
	let appliedStateRevision = 0n
	let requestController: AbortController | undefined
	let realtimeBinding: CallTranscriptionRealtimeBindingV1 | undefined

	const model = computed<CallTranscriptionPanelModel>(() => ({
		available: canTranscribe(),
		busy: ['starting', 'awaiting-recording', 'awaiting-stt', 'materializing'].includes(status.value),
		canRetry: canTranscribe() && Boolean(selectedSource()) && ['rejected', 'error'].includes(status.value),
		status: canTranscribe() ? status.value : 'unavailable',
		statusMessage: canTranscribe()
			? statusMessage.value
			: 'Call Transcription is not admitted for this runtime.',
		transcriptText: transcriptText.value,
		detectedLanguage: detectedLanguage.value,
		durationLabel: durationLabel.value,
	}))

	watch(
		[canTranscribe, selectedSource],
		([available, source]) => {
			cancelCurrent()
			resetArtifact()
			if (!available) {
				status.value = 'unavailable'
				return
			}
			if (!source) {
				status.value = 'waiting-source'
				statusMessage.value = 'Select a recorded call with current consent evidence to transcribe it.'
				return
			}
			void start(source)
		},
		{ immediate: true },
	)

	onUnmounted(cancelCurrent)

	async function retry(): Promise<void> {
		const source = selectedSource()
		if (!canTranscribe() || !source) return
		cancelCurrent()
		resetArtifact()
		await start(source)
	}

	async function start(source: CallTranscriptionSourceV1): Promise<void> {
		const currentGeneration = ++generation
		const controller = new AbortController()
		requestController = controller
		status.value = 'starting'
		statusMessage.value = 'Opening the authenticated realtime stream…'
		realtimeBinding = openCallTranscriptionRealtime({
			onStatus: next => void applyRealtime(next, currentGeneration, controller.signal),
			onUnavailable: () => failRealtime(currentGeneration),
		})
		try {
			await realtimeBinding.ready
			if (currentGeneration !== generation) return
			const runId = await startCallTranscription(source, controller.signal)
			if (currentGeneration !== generation) return
			realtimeBinding.attachRun(runId)
			status.value = 'awaiting-recording'
			statusMessage.value = 'Waiting for the exact consent-bound recording evidence…'
			const snapshot = await getCallTranscriptionStatus(runId, controller.signal)
			if (currentGeneration === generation) {
				await applySnapshot(snapshot, currentGeneration, controller.signal)
			}
		} catch {
			if (currentGeneration !== generation || controller.signal.aborted) return
			fail('Call transcription could not be started.')
		}
	}

	async function applyRealtime(
		next: CallTranscriptionStatusChangedV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		try {
			await applyState(
				next.runId,
				next.state,
				next.stateRevision,
				next.artifact,
				next.error,
				currentGeneration,
				signal,
			)
		} catch {
			if (currentGeneration === generation && !signal.aborted) {
				fail('Call transcript bytes are unavailable.')
			}
		}
	}

	async function applySnapshot(
		next: GetCallTranscriptionResponseV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		await applyState(
			next.runId,
			next.state,
			next.stateRevision,
			next.artifact,
			next.error,
			currentGeneration,
			signal,
		)
	}

	async function applyState(
		runId: Uint8Array,
		nextState: CallTranscriptionStateV1,
		stateRevision: bigint,
		artifact: CallTranscriptionArtifactV1 | undefined,
		error: CallTranscriptionErrorCodeV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		if (currentGeneration !== generation || stateRevision <= appliedStateRevision) return
		appliedStateRevision = stateRevision
		switch (nextState) {
			case CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_ACCEPTED:
			case CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_AWAITING_RECORDING:
				setProgress('awaiting-recording', 'Waiting for the exact consent-bound recording evidence…')
				return
			case CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_AWAITING_STT:
				setProgress('awaiting-stt', 'The provider-neutral speech-to-text engine is processing the recording.')
				return
			case CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_MATERIALIZING_TRANSCRIPT:
				setProgress('materializing', 'Materializing the bounded transcript under workflow custody…')
				return
			case CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_READY:
				if (!artifact) throw new Error('missing artifact metadata')
				await loadTranscript(runId, artifact, currentGeneration, signal)
				return
			case CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_REJECTED:
				setTerminal('rejected', rejectionMessage(error))
				return
			default:
				throw new Error('invalid transcription state')
		}
	}

	async function loadTranscript(
		runId: Uint8Array,
		artifact: CallTranscriptionArtifactV1,
		currentGeneration: number,
		signal: AbortSignal,
	): Promise<void> {
		const document = await readCallTranscript(runId, artifact, signal)
		if (currentGeneration !== generation) return
		transcriptText.value = document.text
		detectedLanguage.value = languageLabel(artifact.detectedLanguage)
		durationLabel.value = duration(artifact.durationMillis)
		setTerminal('ready', 'Transcript is ready from a fresh one-use authenticated read.')
	}

	function failRealtime(currentGeneration: number): void {
		if (currentGeneration !== generation || isTerminal(status.value)) return
		fail('Realtime transcription status is unavailable. Retry to reconcile safely.')
	}

	function setProgress(next: CallTranscriptionPanelStatus, message: string): void {
		status.value = next
		statusMessage.value = message
	}

	function setTerminal(next: CallTranscriptionPanelStatus, message: string): void {
		realtimeBinding?.close()
		realtimeBinding = undefined
		status.value = next
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
		realtimeBinding?.close()
		realtimeBinding = undefined
	}

	function resetArtifact(): void {
		transcriptText.value = ''
		detectedLanguage.value = ''
		durationLabel.value = ''
	}

	return { model, retry }
}

function rejectionMessage(error: CallTranscriptionErrorCodeV1): string {
	if (error === CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_RECORDING_REJECTED) {
		return 'The recording or its consent evidence was rejected.'
	}
	if (error === CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_STT_REJECTED) {
		return 'The speech-to-text engine rejected the bounded recording.'
	}
	if (error === CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_STALE_AUTHORITY) {
		return 'The recording authority changed before the transcript was committed.'
	}
	return 'Call transcription was rejected by the current workflow policy.'
}

function languageLabel(language: CallTranscriptionLanguageV1): string {
	if (language === CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_ENGLISH) return 'English'
	if (language === CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_RUSSIAN) return 'Russian'
	if (language === CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_SPANISH) return 'Spanish'
	return 'Detected language'
}

function duration(milliseconds: bigint): string {
	const seconds = Number(milliseconds / 1_000n)
	const minutes = Math.floor(seconds / 60)
	const remainder = seconds % 60
	return `${minutes}:${remainder.toString().padStart(2, '0')}`
}

function isTerminal(status: CallTranscriptionPanelStatus): boolean {
	return ['ready', 'rejected', 'error', 'unavailable', 'waiting-source'].includes(status)
}
