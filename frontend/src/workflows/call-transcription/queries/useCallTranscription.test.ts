import { create } from '@bufbuild/protobuf'
import { nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	CallTranscriptionArtifactV1Schema,
	CallTranscriptionCompletenessV1,
	CallTranscriptionLanguageV1,
	CallTranscriptionStateV1,
	CallTranscriptionStatusChangedV1Schema,
	GetCallTranscriptionResponseV1Schema,
} from '../../../gen/makosh/call_transcription/v1/transcription_pb'
import type {
	CallTranscriptionRealtimeObserverV1,
	CallTranscriptionSourceV1,
} from '../api/callTranscription'
import {
	getCallTranscriptionStatus,
	openCallTranscriptionRealtime,
	readCallTranscript,
	startCallTranscription,
} from '../api/callTranscription'
import { useCallTranscription } from './useCallTranscription'

vi.mock('../api/callTranscription', () => ({
	getCallTranscriptionStatus: vi.fn(),
	openCallTranscriptionRealtime: vi.fn(),
	readCallTranscript: vi.fn(),
	startCallTranscription: vi.fn(),
}))

const id = (value: number): Uint8Array => new Uint8Array(16).fill(value)
const runId = id(9)

describe('useCallTranscription', () => {
	beforeEach(() => vi.clearAllMocks())

	it('shows an honest skeleton without a gate or exact selected source', async () => {
		const available = ref(false)
		const selected = ref<CallTranscriptionSourceV1>()
		const workflow = useCallTranscription(() => available.value, () => selected.value)
		await nextTick()
		expect(workflow.model.value).toMatchObject({ available: false, status: 'unavailable' })
		expect(openCallTranscriptionRealtime).not.toHaveBeenCalled()
		expect(startCallTranscription).not.toHaveBeenCalled()

		available.value = true
		await nextTick()
		expect(workflow.model.value.status).toBe('waiting-source')
		expect(startCallTranscription).not.toHaveBeenCalled()
	})

	it('opens realtime before Start, reconciles once and reads ready bytes through ClientBlob', async () => {
		const order: string[] = []
		let observer: CallTranscriptionRealtimeObserverV1 | undefined
		const close = vi.fn()
		const attachRun = vi.fn()
		vi.mocked(openCallTranscriptionRealtime).mockImplementation(next => {
			order.push('realtime')
			observer = next
			return { ready: Promise.resolve(), attachRun, close }
		})
		vi.mocked(startCallTranscription).mockImplementation(async () => {
			order.push('start')
			return runId
		})
		vi.mocked(getCallTranscriptionStatus).mockImplementation(async () => {
			order.push('get')
			return create(GetCallTranscriptionResponseV1Schema, {
				runId,
				callEvidenceId: id(2),
				callEvidenceRevision: 3n,
				recordingEvidenceId: id(4),
				recordingRevision: 5n,
				state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_AWAITING_STT,
				stateRevision: 2n,
			})
		})
		vi.mocked(readCallTranscript).mockResolvedValue({
			text: 'verified transcript',
			detectedLanguage: CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_ENGLISH,
			durationMillis: 61_000n,
			segmentCount: 2,
			completeness: CallTranscriptionCompletenessV1.CALL_TRANSCRIPTION_COMPLETENESS_COMPLETE,
			confidenceBasisPoints: 9_000,
		})
		const selected = ref<CallTranscriptionSourceV1>(source())
		const workflow = useCallTranscription(() => true, () => selected.value)

		await vi.waitFor(() => expect(workflow.model.value.status).toBe('awaiting-stt'))
		expect(order).toEqual(['realtime', 'start', 'get'])
		expect(attachRun).toHaveBeenCalledWith(runId)

		observer?.onStatus(create(CallTranscriptionStatusChangedV1Schema, {
			runId,
			state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_READY,
			stateRevision: 3n,
			artifact: artifact(),
			occurredAtUnixMillis: 1n,
		}))
		await vi.waitFor(() => expect(workflow.model.value.status).toBe('ready'))
		expect(workflow.model.value.transcriptText).toBe('verified transcript')
		expect(readCallTranscript).toHaveBeenCalledWith(runId, artifact(), expect.any(AbortSignal))
		expect(close).toHaveBeenCalledOnce()
	})
})

function source(): CallTranscriptionSourceV1 {
	return {
		operationId: id(1),
		callEvidenceId: id(2),
		callEvidenceRevision: 3n,
		recordingEvidenceId: id(4),
		recordingRevision: 5n,
		consentReceiptId: id(6),
		consentPolicyRevision: 7,
		requestedLanguage: CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_AUTO,
	}
}

function artifact() {
	return create(CallTranscriptionArtifactV1Schema, {
		transcriptSha256: new Uint8Array(32).fill(8),
		transcriptSizeBytes: 19n,
		detectedLanguage: CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_ENGLISH,
		durationMillis: 61_000n,
		segmentCount: 2,
		completeness: CallTranscriptionCompletenessV1.CALL_TRANSCRIPTION_COMPLETENESS_COMPLETE,
		confidenceBasisPoints: 9_000,
	})
}
