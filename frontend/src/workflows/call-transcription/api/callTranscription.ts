import { create, fromBinary, toBinary } from '@bufbuild/protobuf'

import {
	CallTranscriptionCompletenessV1,
	CallTranscriptionErrorCodeV1,
	CallTranscriptionLanguageV1,
	CallTranscriptionStateV1,
	CallTranscriptionStatusChangedV1Schema,
	ReadCallTranscriptRequestV1Schema,
	type CallTranscriptionArtifactV1,
	type CallTranscriptionStatusChangedV1,
	type GetCallTranscriptionResponseV1,
} from '../../../gen/makosh/call_transcription/v1/transcription_pb'
import {
	SpeechTranscriptCompletenessV1,
	SpeechTranscriptDocumentV1Schema,
	SpeechTranscriptLanguageV1,
} from '../../../gen/makosh/speech_transcript/v1/transcript_pb'
import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { getCallTranscriptionCommandClient } from '../../../platform/connect/callTranscriptionCommandClient'
import { getCallTranscriptionQueryClient } from '../../../platform/connect/callTranscriptionQueryClient'
import { getCallTranscriptTicketClient } from '../../../platform/connect/callTranscriptTicketClient'
import { BrowserGatewayFetch } from '../../../platform/gateway/browserGatewayFetch'
import type {
	BrowserGatewayRealtimeObserver,
	BrowserGatewayRealtimeSubscription,
} from '../../../platform/gateway/browserGatewayRealtime'
import { getBrowserGatewayRealtimeHub } from '../../../platform/gateway/browserGatewayRealtimeHub'

const TRANSCRIPT_READ_PATH = '/api/blobs/call-transcription/v1/transcript'
const REALTIME_CONTRACT = 'call_transcription.status_changed'
const REALTIME_EVENT_KIND = 'call_transcription.status_changed'
const ID_BYTES = 16
const READ_TICKET_BYTES = 32
const MAX_TRANSCRIPT_BYTES = 4 * 1024 * 1024
const MAX_DURATION_MILLIS = 4n * 60n * 60n * 1_000n
const MAX_SEGMENTS = 100_000
const MAX_CONFIDENCE_BASIS_POINTS = 10_000
const MAX_BUFFERED_STATUSES = 64
const MAX_SEGMENT_BYTES = 64 * 1024

export type CallTranscriptDocumentV1 = {
	text: string
	detectedLanguage: CallTranscriptionLanguageV1
	durationMillis: bigint
	segmentCount: number
	completeness: CallTranscriptionCompletenessV1
	confidenceBasisPoints: number
}

export type CallTranscriptionSourceV1 = {
	operationId: Uint8Array
	callEvidenceId: Uint8Array
	callEvidenceRevision: bigint
	recordingEvidenceId: Uint8Array
	recordingRevision: bigint
	consentReceiptId: Uint8Array
	consentPolicyRevision: number
	requestedLanguage: CallTranscriptionLanguageV1
}

export type CallTranscriptionRealtimeObserverV1 = {
	onStatus(status: CallTranscriptionStatusChangedV1): void
	onUnavailable(): void
}

export type CallTranscriptionRealtimeBindingV1 = BrowserGatewayRealtimeSubscription & {
	ready: Promise<void>
	attachRun(runId: Uint8Array): void
}

type CallTranscriptionRealtimePort = {
	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription
}

type CallTranscriptionPorts = {
	start(source: CallTranscriptionSourceV1, signal?: AbortSignal): Promise<{
		runId: Uint8Array
		state: CallTranscriptionStateV1
		stateRevision: bigint
		error: CallTranscriptionErrorCodeV1
	}>
	get(runId: Uint8Array, signal?: AbortSignal): Promise<GetCallTranscriptionResponseV1>
	issueRead(runId: Uint8Array, signal?: AbortSignal): Promise<{
		runId: Uint8Array
		opaqueReadTicket: Uint8Array
		expiresAtUnixSeconds: bigint
		transcriptSizeBytes: bigint
		error: CallTranscriptionErrorCodeV1
	}>
	readBlob(input: RequestInfo | URL, init: RequestInit): Promise<Response>
	nowUnixSeconds(): bigint
}

export async function startCallTranscription(
	source: CallTranscriptionSourceV1,
	signal?: AbortSignal,
	ports: CallTranscriptionPorts = defaultPorts(),
): Promise<Uint8Array> {
	validateSource(source)
	const response = await ports.start(copySource(source), signal)
	if (
		response.error !== CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_UNSPECIFIED
		|| response.state !== CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_AWAITING_RECORDING
		|| response.stateRevision < 1n
		|| !validId(response.runId)
	) {
		throw new Error('Call transcription was not accepted')
	}
	return copy(response.runId)
}

export async function getCallTranscriptionStatus(
	runId: Uint8Array,
	signal?: AbortSignal,
	ports: CallTranscriptionPorts = defaultPorts(),
): Promise<GetCallTranscriptionResponseV1> {
	validateId(runId, 'Call transcription')
	const response = await ports.get(copy(runId), signal)
	if (
		!equal(response.runId, runId)
		|| !validId(response.callEvidenceId)
		|| !validId(response.recordingEvidenceId)
		|| response.callEvidenceRevision < 1n
		|| response.recordingRevision < 1n
		|| response.stateRevision < 1n
		|| !validStatus(response.state, response.error, response.artifact)
	) {
		throw new Error('Call transcription status is invalid')
	}
	return response
}

export async function readCallTranscript(
	runId: Uint8Array,
	expected: CallTranscriptionArtifactV1,
	signal?: AbortSignal,
	ports: CallTranscriptionPorts = defaultPorts(),
): Promise<CallTranscriptDocumentV1> {
	validateId(runId, 'Call transcription')
	const ticket = await ports.issueRead(copy(runId), signal)
	const declaredSize = Number(ticket.transcriptSizeBytes)
	if (
		ticket.error !== CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_UNSPECIFIED
		|| !equal(ticket.runId, runId)
		|| ticket.opaqueReadTicket.byteLength !== READ_TICKET_BYTES
		|| ticket.expiresAtUnixSeconds <= ports.nowUnixSeconds()
		|| !Number.isSafeInteger(declaredSize)
		|| declaredSize < 1
		|| declaredSize > MAX_TRANSCRIPT_BYTES
	) {
		throw new Error('Call transcript read ticket is invalid')
	}
	const request = create(ReadCallTranscriptRequestV1Schema, {
		protocolMajor: 1,
		opaqueReadTicket: ticket.opaqueReadTicket,
	})
	const response = await ports.readBlob(TRANSCRIPT_READ_PATH, {
		method: 'POST',
		headers: {
			accept: 'application/octet-stream',
			'content-type': 'application/protobuf',
		},
		body: toBinary(ReadCallTranscriptRequestV1Schema, request),
		signal,
	})
	if (
		!response.ok
		|| response.headers.get('cache-control') !== 'no-store'
		|| response.headers.get('content-type')?.split(';', 1)[0] !== 'application/octet-stream'
	) {
		throw new Error('Call transcript is unavailable')
	}
	const bytes = new Uint8Array(await response.arrayBuffer())
	if (bytes.byteLength !== declaredSize || bytes.byteLength > MAX_TRANSCRIPT_BYTES) {
		throw new Error('Call transcript length is invalid')
	}
	return decodeTranscriptDocument(bytes, expected)
}

function decodeTranscriptDocument(
	bytes: Uint8Array,
	expected: CallTranscriptionArtifactV1,
): CallTranscriptDocumentV1 {
	const document = fromBinary(SpeechTranscriptDocumentV1Schema, bytes)
	const detectedLanguage = transcriptLanguage(document.detectedLanguage)
	const completeness = transcriptCompleteness(document.completeness)
	if (
		document.protocolMajor !== 1
		|| !validId(document.requestId)
		|| document.durationMillis !== expected.durationMillis
		|| document.segments.length !== expected.segmentCount
		|| detectedLanguage !== expected.detectedLanguage
		|| completeness !== expected.completeness
		|| document.confidenceBasisPoints !== expected.confidenceBasisPoints
		|| document.segments.length < 1
		|| document.segments.length > MAX_SEGMENTS
	) throw new Error('Call transcript document metadata is invalid')
	const decoder = new TextDecoder('utf-8', { fatal: true })
	let previousEnd = 0n
	const lines = document.segments.map((segment, index) => {
		if (
			segment.index !== index
			|| segment.startMillis < previousEnd
			|| segment.endMillis <= segment.startMillis
			|| segment.endMillis > document.durationMillis
			|| segment.contentUtf8.byteLength < 1
			|| segment.contentUtf8.byteLength > MAX_SEGMENT_BYTES
		) throw new Error('Call transcript segment is invalid')
		previousEnd = segment.endMillis
		const text = decoder.decode(segment.contentUtf8)
		if (!text.trim()) throw new Error('Call transcript segment is invalid')
		return text
	})
	return {
		text: lines.join('\n'),
		detectedLanguage,
		durationMillis: document.durationMillis,
		segmentCount: document.segments.length,
		completeness,
		confidenceBasisPoints: document.confidenceBasisPoints,
	}
}

function transcriptLanguage(value: SpeechTranscriptLanguageV1): CallTranscriptionLanguageV1 {
	if (value === SpeechTranscriptLanguageV1.AUTO) {
		return CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_AUTO
	}
	if (value === SpeechTranscriptLanguageV1.ENGLISH) {
		return CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_ENGLISH
	}
	if (value === SpeechTranscriptLanguageV1.RUSSIAN) {
		return CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_RUSSIAN
	}
	if (value === SpeechTranscriptLanguageV1.SPANISH) {
		return CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_SPANISH
	}
	throw new Error('Call transcript language is invalid')
}

function transcriptCompleteness(
	value: SpeechTranscriptCompletenessV1,
): CallTranscriptionCompletenessV1 {
	if (value === SpeechTranscriptCompletenessV1.COMPLETE) {
		return CallTranscriptionCompletenessV1.CALL_TRANSCRIPTION_COMPLETENESS_COMPLETE
	}
	if (value === SpeechTranscriptCompletenessV1.PARTIAL) {
		return CallTranscriptionCompletenessV1.CALL_TRANSCRIPTION_COMPLETENESS_PARTIAL
	}
	throw new Error('Call transcript completeness is invalid')
}

export function openCallTranscriptionRealtime(
	observer: CallTranscriptionRealtimeObserverV1,
	hub: CallTranscriptionRealtimePort = getBrowserGatewayRealtimeHub(),
): CallTranscriptionRealtimeBindingV1 {
	let selectedRunId: Uint8Array | undefined
	const buffered: CallTranscriptionStatusChangedV1[] = []
	let resolveReady: (() => void) | undefined
	let rejectReady: ((reason: Error) => void) | undefined
	let settled = false
	const ready = new Promise<void>((resolve, reject) => {
		resolveReady = resolve
		rejectReady = reject
	})
	const unavailable = (): void => {
		if (!settled) {
			settled = true
			rejectReady?.(new Error('Call transcription realtime is unavailable'))
		}
		observer.onUnavailable()
	}
	const subscription = hub.subscribe({
		onEvent: event => {
			try {
				const status = decodeRealtimeStatus(event)
				if (!status) return
				if (!selectedRunId) {
					if (buffered.length === MAX_BUFFERED_STATUSES) buffered.shift()
					buffered.push(status)
					return
				}
				if (equal(status.runId, selectedRunId)) observer.onStatus(status)
			} catch {
				unavailable()
			}
		},
		onStreamState: state => {
			if (state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN) {
				if (!settled) {
					settled = true
					resolveReady?.()
				}
			} else if (state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_CLOSED) {
				unavailable()
			}
		},
		onReplayGap: unavailable,
		onProtocolError: unavailable,
	})
	return {
		ready,
		attachRun: runId => {
			validateId(runId, 'Call transcription')
			selectedRunId = copy(runId)
			for (const status of buffered) {
				if (equal(status.runId, selectedRunId)) observer.onStatus(status)
			}
			buffered.length = 0
		},
		close: () => subscription.close(),
	}
}

function decodeRealtimeStatus(event: ClientRealtimeEventV1): CallTranscriptionStatusChangedV1 | undefined {
	if (
		event.contractName !== REALTIME_CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== REALTIME_EVENT_KIND
	) return undefined
	const status = fromBinary(CallTranscriptionStatusChangedV1Schema, event.payload)
	if (
		!validId(status.runId)
		|| status.stateRevision < 1n
		|| status.occurredAtUnixMillis < 1n
		|| !validStatus(status.state, status.error, status.artifact)
	) {
		throw new Error('Call transcription realtime status is invalid')
	}
	return status
}

function defaultPorts(): CallTranscriptionPorts {
	const gateway = new BrowserGatewayFetch()
	return {
		start: (source, signal) => getCallTranscriptionCommandClient().start({
			protocolMajor: 1,
			operationId: source.operationId,
			callEvidenceId: source.callEvidenceId,
			expectedCallEvidenceRevision: source.callEvidenceRevision,
			recordingEvidenceId: source.recordingEvidenceId,
			expectedRecordingRevision: source.recordingRevision,
			consentReceiptId: source.consentReceiptId,
			consentPolicyRevision: source.consentPolicyRevision,
			requestedLanguage: source.requestedLanguage,
		}, { signal }),
		get: (runId, signal) => getCallTranscriptionQueryClient().get(
			{ protocolMajor: 1, runId },
			{ signal },
		),
		issueRead: (runId, signal) => getCallTranscriptTicketClient().issueRead(
			{ protocolMajor: 1, runId },
			{ signal },
		),
		readBlob: gateway.fetch.bind(gateway),
		nowUnixSeconds: () => BigInt(Math.floor(Date.now() / 1_000)),
	}
}

function validateSource(source: CallTranscriptionSourceV1): void {
	validateId(source.operationId, 'Call transcription operation')
	validateId(source.callEvidenceId, 'Call evidence')
	validateId(source.recordingEvidenceId, 'Recording evidence')
	validateId(source.consentReceiptId, 'Consent receipt')
	if (
		source.callEvidenceRevision < 1n
		|| source.recordingRevision < 1n
		|| !Number.isInteger(source.consentPolicyRevision)
		|| source.consentPolicyRevision < 1
		|| source.requestedLanguage === CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_UNSPECIFIED
	) throw new RangeError('Call transcription source is invalid')
}

function validStatus(
	state: CallTranscriptionStateV1,
	error: CallTranscriptionErrorCodeV1,
	artifact: GetCallTranscriptionResponseV1['artifact'],
): boolean {
	if (state === CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_UNSPECIFIED) return false
	if (state === CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_READY) {
		return error === CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_UNSPECIFIED
			&& artifact !== undefined
			&& artifact.transcriptSha256.byteLength === 32
			&& artifact.transcriptSizeBytes > 0n
			&& artifact.transcriptSizeBytes <= BigInt(MAX_TRANSCRIPT_BYTES)
			&& artifact.detectedLanguage !== CallTranscriptionLanguageV1.CALL_TRANSCRIPTION_LANGUAGE_UNSPECIFIED
			&& artifact.durationMillis > 0n
			&& artifact.durationMillis <= MAX_DURATION_MILLIS
			&& artifact.segmentCount <= MAX_SEGMENTS
			&& artifact.completeness !== CallTranscriptionCompletenessV1.CALL_TRANSCRIPTION_COMPLETENESS_UNSPECIFIED
			&& artifact.confidenceBasisPoints <= MAX_CONFIDENCE_BASIS_POINTS
	}
	if (state === CallTranscriptionStateV1.CALL_TRANSCRIPTION_STATE_REJECTED) {
		return error !== CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_UNSPECIFIED
			&& artifact === undefined
	}
	return error === CallTranscriptionErrorCodeV1.CALL_TRANSCRIPTION_ERROR_CODE_UNSPECIFIED
		&& artifact === undefined
}

function copySource(source: CallTranscriptionSourceV1): CallTranscriptionSourceV1 {
	return {
		...source,
		operationId: copy(source.operationId),
		callEvidenceId: copy(source.callEvidenceId),
		recordingEvidenceId: copy(source.recordingEvidenceId),
		consentReceiptId: copy(source.consentReceiptId),
	}
}

function validateId(value: Uint8Array, label: string): void {
	if (!validId(value)) throw new RangeError(`${label} ID must be ${ID_BYTES} non-zero bytes`)
}

function validId(value: Uint8Array): boolean {
	return value.byteLength === ID_BYTES && value.some(byte => byte !== 0)
}

function equal(left: Uint8Array, right: Uint8Array): boolean {
	return left.byteLength === right.byteLength && left.every((byte, index) => byte === right[index])
}

function copy(value: Uint8Array): Uint8Array {
	return new Uint8Array(value)
}
