import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	CallTranscriptionArtifactV1Schema,
	CallTranscriptionCompletenessV1,
	CallTranscriptionErrorCodeV1,
	CallTranscriptionLanguageV1,
	CallTranscriptionStateV1,
	CallTranscriptionStatusChangedV1Schema,
	GetCallTranscriptionResponseV1Schema,
	IssueCallTranscriptReadResponseV1Schema,
	StartCallTranscriptionResponseV1Schema,
} from '../../../gen/makosh/call_transcription/v1/transcription_pb'
import {
	SpeechTranscriptCompletenessV1,
	SpeechTranscriptDocumentV1Schema,
	SpeechTranscriptLanguageV1,
} from '../../../gen/makosh/speech_transcript/v1/transcript_pb'
import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateKindV1,
	ClientRealtimeStreamStateV1Schema,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import {
	getCallTranscriptionStatus,
	openCallTranscriptionRealtime,
	readCallTranscript,
	startCallTranscription,
	type CallTranscriptionSourceV1,
} from './callTranscription'

const id = (value: number): Uint8Array => new Uint8Array(16).fill(value)
const runId = id(9)

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
		transcriptSizeBytes: 3n,
		detectedLanguage: CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_ENGLISH,
		durationMillis: 1_000n,
		segmentCount: 1,
		completeness: CallTranscriptionCompletenessV1.CALL_TRANSCRIPTION_COMPLETENESS_COMPLETE,
		confidenceBasisPoints: 9_000,
	})
}

function ports() {
	const documentBytes = toBinary(SpeechTranscriptDocumentV1Schema, create(
		SpeechTranscriptDocumentV1Schema,
		{
			protocolMajor: 1,
			requestId: id(10),
			detectedLanguage: SpeechTranscriptLanguageV1.ENGLISH,
			durationMillis: 1_000n,
			segments: [{ index: 0, startMillis: 0n, endMillis: 1_000n, contentUtf8: new Uint8Array([65, 66, 67]) }],
			completeness: SpeechTranscriptCompletenessV1.COMPLETE,
			confidenceBasisPoints: 9_000,
		},
	))
	return {
		start: vi.fn(async () => create(StartCallTranscriptionResponseV1Schema, {
			runId,
			state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_AWAITING_RECORDING,
			stateRevision: 1n,
		})),
		get: vi.fn(async () => create(GetCallTranscriptionResponseV1Schema, {
			runId,
			callEvidenceId: id(2),
			callEvidenceRevision: 3n,
			recordingEvidenceId: id(4),
			recordingRevision: 5n,
			state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_READY,
			stateRevision: 6n,
			artifact: artifact(),
		})),
		issueRead: vi.fn(async () => create(IssueCallTranscriptReadResponseV1Schema, {
			runId,
			opaqueReadTicket: new Uint8Array(32).fill(7),
			expiresAtUnixSeconds: 100n,
			transcriptSizeBytes: BigInt(documentBytes.byteLength),
		})),
		readBlob: vi.fn(async () => new Response(documentBytes, {
			status: 200,
			headers: {
				'cache-control': 'no-store',
				'content-type': 'application/octet-stream',
			},
		})),
		nowUnixSeconds: vi.fn(() => 50n),
	}
}

describe('call transcription browser adapter', () => {
	it('uses only exact source evidence and generated workflow ports', async () => {
		const adapter = ports()
		const input = source()
		await expect(startCallTranscription(input, undefined, adapter)).resolves.toEqual(runId)
		expect(adapter.start).toHaveBeenCalledWith(input, undefined)
		await expect(getCallTranscriptionStatus(runId, undefined, adapter)).resolves.toMatchObject({
			runId,
			state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_READY,
		})
	})

	it('opens the shared stream before Start and delivers its buffered exact-run status', async () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const close = vi.fn()
		const hub = {
			subscribe: vi.fn((observer: BrowserGatewayRealtimeObserver) => {
				sourceObserver = observer
				return { close }
			}),
		}
		const observer = { onStatus: vi.fn(), onUnavailable: vi.fn() }
		const binding = openCallTranscriptionRealtime(observer, hub)
		sourceObserver?.onStreamState(create(ClientRealtimeStreamStateV1Schema, {
			state: ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN,
		}))
		await expect(binding.ready).resolves.toBeUndefined()

		const status = create(CallTranscriptionStatusChangedV1Schema, {
			runId,
			state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_READY,
			stateRevision: 6n,
			artifact: artifact(),
			occurredAtUnixMillis: 1n,
		})
		sourceObserver?.onEvent(create(ClientRealtimeEventV1Schema, {
			contractName: 'call_transcription.status_changed',
			contractVersion: 1,
			eventKind: 'call_transcription.status_changed',
			payload: toBinary(CallTranscriptionStatusChangedV1Schema, status),
		}))
		expect(observer.onStatus).not.toHaveBeenCalled()
		binding.attachRun(runId)
		expect(observer.onStatus).toHaveBeenCalledWith(status)
		binding.close()
		expect(close).toHaveBeenCalledOnce()
	})

	it('reads transcript bytes only through a fresh one-use client_blob ticket', async () => {
		const adapter = ports()
		await expect(readCallTranscript(runId, artifact(), undefined, adapter)).resolves.toMatchObject({
			text: 'ABC',
			detectedLanguage: CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_ENGLISH,
			segmentCount: 1,
		})
		expect(adapter.readBlob).toHaveBeenCalledWith(
			'/api/blobs/call-transcription/v1/transcript',
			expect.objectContaining({ method: 'POST', body: expect.any(Uint8Array) }),
		)
	})

	it('fails closed on stale tickets, response length drift and invalid source authority', async () => {
		const stale = ports()
		stale.nowUnixSeconds.mockReturnValue(101n)
		await expect(readCallTranscript(runId, artifact(), undefined, stale)).rejects.toThrow('ticket')

		const truncated = ports()
		truncated.readBlob.mockResolvedValue(new Response(new Uint8Array([65]), {
			status: 200,
			headers: {
				'cache-control': 'no-store',
				'content-type': 'application/octet-stream',
			},
		}))
		await expect(readCallTranscript(runId, artifact(), undefined, truncated)).rejects.toThrow('length')

		const invalid = source()
		invalid.consentPolicyRevision = 0
		await expect(startCallTranscription(invalid, undefined, ports())).rejects.toThrow('source')
	})

	it('rejects a terminal status whose error and artifact disagree', async () => {
		const invalid = ports()
		invalid.get.mockResolvedValueOnce(create(GetCallTranscriptionResponseV1Schema, {
			runId,
			callEvidenceId: id(2),
			callEvidenceRevision: 3n,
			recordingEvidenceId: id(4),
			recordingRevision: 5n,
			state: CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_REJECTED,
			stateRevision: 6n,
			error: CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_STT_REJECTED,
			artifact: artifact(),
		}))
		await expect(getCallTranscriptionStatus(runId, undefined, invalid)).rejects.toThrow('invalid')
	})
})
