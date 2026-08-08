import { create, fromBinary, toBinary } from '@bufbuild/protobuf'

import {
	AttachmentPreviewContentTypeV1,
	AttachmentPreviewErrorCodeV1,
	AttachmentPreviewStateV1,
	AttachmentPreviewStatusChangedV1Schema,
	type AttachmentPreviewStatusChangedV1,
	type GetAttachmentPreviewResponseV1,
} from '../../../gen/makosh/attachment_preview/v1/preview_pb'
import { ReadAttachmentPreviewRequestV1Schema } from '../../../gen/makosh/attachment_preview/read/v1/read_pb'
import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { getAttachmentPreviewCommandClient } from '../../../platform/connect/attachmentPreviewCommandClient'
import { getAttachmentPreviewQueryClient } from '../../../platform/connect/attachmentPreviewQueryClient'
import { getAttachmentPreviewTicketClient } from '../../../platform/connect/attachmentPreviewTicketClient'
import { BrowserGatewayFetch } from '../../../platform/gateway/browserGatewayFetch'
import {
	getBrowserGatewayRealtimeHub,
} from '../../../platform/gateway/browserGatewayRealtimeHub'
import type {
	BrowserGatewayRealtimeObserver,
	BrowserGatewayRealtimeSubscription,
} from '../../../platform/gateway/browserGatewayRealtime'

const PREVIEW_READ_PATH = '/api/blobs/attachment-preview/v1/artifact'
const REALTIME_CONTRACT = 'attachment_preview.realtime'
const REALTIME_EVENT_KIND = 'attachment_preview.status_changed.v1'
const ID_BYTES = 16
const READ_TICKET_BYTES = 32
const MAX_PREVIEW_BYTES = 32 * 1024 * 1024

export type AttachmentPreviewArtifactV1 = {
	bytes: Uint8Array
	contentType: AttachmentPreviewContentTypeV1
}

export type AttachmentPreviewRealtimeObserverV1 = {
	onStatus(status: AttachmentPreviewStatusChangedV1): void
	onUnavailable(): void
}

export type AttachmentPreviewRealtimeSubscriptionV1 = BrowserGatewayRealtimeSubscription & {
	ready: Promise<void>
}

type AttachmentPreviewPorts = {
	start(
		operationId: Uint8Array,
		attachmentAnchorId: Uint8Array,
		signal?: AbortSignal,
	): Promise<{ runId: Uint8Array; state: AttachmentPreviewStateV1; error: AttachmentPreviewErrorCodeV1 }>
	get(
		runId: Uint8Array,
		signal?: AbortSignal,
	): Promise<GetAttachmentPreviewResponseV1>
	issueRead(
		runId: Uint8Array,
		signal?: AbortSignal,
	): Promise<{
		runId: Uint8Array
		opaqueReadTicket: Uint8Array
		expiresAtUnixSeconds: bigint
		contentType: AttachmentPreviewContentTypeV1
		previewSizeBytes: bigint
		error: AttachmentPreviewErrorCodeV1
	}>
	readBlob(input: RequestInfo | URL, init: RequestInit): Promise<Response>
	nowUnixSeconds(): bigint
}

type AttachmentPreviewRealtimePort = {
	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription
}

export async function startAttachmentPreview(
	attachmentAnchorId: Uint8Array,
	operationId: Uint8Array,
	signal?: AbortSignal,
	ports: AttachmentPreviewPorts = defaultPorts(),
): Promise<Uint8Array> {
	validateId(attachmentAnchorId, 'Attachment anchor')
	validateId(operationId, 'Preview operation')
	const response = await ports.start(copy(operationId), copy(attachmentAnchorId), signal)
	if (
		!validStateAndError(response.state, response.error)
		|| !validId(response.runId)
	) {
		throw new Error('Attachment preview was not accepted')
	}
	return copy(response.runId)
}

export async function getAttachmentPreviewStatus(
	runId: Uint8Array,
	signal?: AbortSignal,
	ports: AttachmentPreviewPorts = defaultPorts(),
): Promise<GetAttachmentPreviewResponseV1> {
	validateId(runId, 'Attachment preview')
	const response = await ports.get(copy(runId), signal)
	if (
		!equal(response.runId, runId)
		|| !validId(response.attachmentAnchorId)
		|| response.stateRevision < 1n
		|| !validStatusFields(
			response.state,
			response.contentType,
			response.previewSizeBytes,
			response.error,
		)
	) {
		throw new Error('Attachment preview status is invalid')
	}
	return response
}

export async function readAttachmentPreview(
	runId: Uint8Array,
	signal?: AbortSignal,
	ports: AttachmentPreviewPorts = defaultPorts(),
): Promise<AttachmentPreviewArtifactV1> {
	validateId(runId, 'Attachment preview')
	const ticket = await ports.issueRead(copy(runId), signal)
	const declaredSize = Number(ticket.previewSizeBytes)
	if (
		ticket.error !== AttachmentPreviewErrorCodeV1.UNSPECIFIED
		|| !equal(ticket.runId, runId)
		|| ticket.opaqueReadTicket.byteLength !== READ_TICKET_BYTES
		|| ticket.expiresAtUnixSeconds <= ports.nowUnixSeconds()
		|| !validContentType(ticket.contentType)
		|| !Number.isSafeInteger(declaredSize)
		|| declaredSize < 1
		|| declaredSize > MAX_PREVIEW_BYTES
	) {
		throw new Error('Attachment preview read ticket is invalid')
	}
	const request = create(ReadAttachmentPreviewRequestV1Schema, {
		protocolMajor: 1,
		opaqueReadTicket: ticket.opaqueReadTicket,
	})
	const response = await ports.readBlob(PREVIEW_READ_PATH, {
		method: 'POST',
		headers: {
			accept: 'application/octet-stream',
			'content-type': 'application/protobuf',
		},
		body: toBinary(ReadAttachmentPreviewRequestV1Schema, request),
		signal,
	})
	if (!response.ok || response.headers.get('cache-control') !== 'no-store') {
		throw new Error('Attachment preview artifact is unavailable')
	}
	const bytes = new Uint8Array(await response.arrayBuffer())
	if (bytes.byteLength !== declaredSize || bytes.byteLength > MAX_PREVIEW_BYTES) {
		throw new Error('Attachment preview artifact length is invalid')
	}
	return { bytes, contentType: ticket.contentType }
}

export function subscribeAttachmentPreviewStatus(
	runId: Uint8Array,
	observer: AttachmentPreviewRealtimeObserverV1,
	hub: AttachmentPreviewRealtimePort = getBrowserGatewayRealtimeHub(),
): AttachmentPreviewRealtimeSubscriptionV1 {
	validateId(runId, 'Attachment preview')
	let resolveReady: (() => void) | undefined
	let rejectReady: ((reason: Error) => void) | undefined
	const ready = new Promise<void>((resolve, reject) => {
		resolveReady = resolve
		rejectReady = reject
	})
	let settled = false
	const resolveStream = (): void => {
		if (settled) return
		settled = true
		resolveReady?.()
	}
	const rejectStream = (): void => {
		if (settled) return
		settled = true
		rejectReady?.(new Error('Attachment preview realtime is unavailable'))
	}
	const subscription = hub.subscribe({
		onEvent: event => deliverPreviewEvent(event, runId, observer),
		onStreamState: state => {
			if (state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_OPEN) {
				resolveStream()
			} else if (state.state === ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_CLOSED) {
				rejectStream()
				observer.onUnavailable()
			}
		},
		onReplayGap: () => {
			rejectStream()
			observer.onUnavailable()
		},
		onProtocolError: () => {
			rejectStream()
			observer.onUnavailable()
		},
	})
	return { close: () => subscription.close(), ready }
}

function deliverPreviewEvent(
	event: ClientRealtimeEventV1,
	runId: Uint8Array,
	observer: AttachmentPreviewRealtimeObserverV1,
): void {
	if (
		event.contractName !== REALTIME_CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== REALTIME_EVENT_KIND
	) return
	try {
		const status = fromBinary(AttachmentPreviewStatusChangedV1Schema, event.payload)
		if (!equal(status.runId, runId) || !validRealtimeStatus(status)) return
		observer.onStatus(status)
	} catch {
		observer.onUnavailable()
	}
}

function defaultPorts(): AttachmentPreviewPorts {
	const gateway = new BrowserGatewayFetch()
	return {
		start: (operationId, attachmentAnchorId, signal) => getAttachmentPreviewCommandClient().start(
			{ protocolMajor: 1, operationId, attachmentAnchorId },
			{ signal },
		),
		get: (runId, signal) => getAttachmentPreviewQueryClient().get(
			{ protocolMajor: 1, runId },
			{ signal },
		),
		issueRead: (runId, signal) => getAttachmentPreviewTicketClient().issueRead(
			{ protocolMajor: 1, runId },
			{ signal },
		),
		readBlob: gateway.fetch.bind(gateway),
		nowUnixSeconds: () => BigInt(Math.floor(Date.now() / 1_000)),
	}
}

function validRealtimeStatus(status: AttachmentPreviewStatusChangedV1): boolean {
	return validId(status.runId)
		&& status.stateRevision >= 1n
		&& validStatusFields(status.state, status.contentType, status.previewSizeBytes, status.error)
}

function validStatusFields(
	state: AttachmentPreviewStateV1,
	contentType: AttachmentPreviewContentTypeV1,
	previewSizeBytes: bigint,
	error: AttachmentPreviewErrorCodeV1,
): boolean {
	if (!validStateAndError(state, error) || previewSizeBytes > BigInt(MAX_PREVIEW_BYTES)) return false
	if (state === AttachmentPreviewStateV1.READY) {
		return validContentType(contentType) && previewSizeBytes > 0n
	}
	return contentType === AttachmentPreviewContentTypeV1.UNSPECIFIED && previewSizeBytes === 0n
}

function validStateAndError(
	state: AttachmentPreviewStateV1,
	error: AttachmentPreviewErrorCodeV1,
): boolean {
	if (state < AttachmentPreviewStateV1.ACCEPTED || state > AttachmentPreviewStateV1.REJECTED) return false
	const terminalFailure = state === AttachmentPreviewStateV1.UNSUPPORTED
		|| state === AttachmentPreviewStateV1.REJECTED
	return terminalFailure
		? error !== AttachmentPreviewErrorCodeV1.UNSPECIFIED
		: error === AttachmentPreviewErrorCodeV1.UNSPECIFIED
}

function validContentType(value: AttachmentPreviewContentTypeV1): boolean {
	return value >= AttachmentPreviewContentTypeV1.TEXT_UTF8
		&& value <= AttachmentPreviewContentTypeV1.MP4_VIDEO
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
