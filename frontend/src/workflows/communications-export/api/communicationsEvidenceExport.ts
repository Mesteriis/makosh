import { create, fromBinary, toBinary } from '@bufbuild/protobuf'

import {
	CommunicationsExportErrorCodeV1,
	EvidenceExportArtifactReadRequestV1Schema,
	EvidenceExportStatusV1,
	EvidenceExportStatusChangedV1Schema,
	type EvidenceExportStatusChangedV1,
	type GetEvidenceExportStatusResponseV1,
} from '../../../gen/makosh/communications_export/v1/export_pb'
import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { getCommunicationsExportCommandClient } from '../../../platform/connect/communicationsExportCommandClient'
import { getCommunicationsExportQueryClient } from '../../../platform/connect/communicationsExportQueryClient'
import { getCommunicationsExportTicketClient } from '../../../platform/connect/communicationsExportTicketClient'
import { BrowserGatewayFetch } from '../../../platform/gateway/browserGatewayFetch'
import { getBrowserGatewayRealtimeHub } from '../../../platform/gateway/browserGatewayRealtimeHub'
import type {
	BrowserGatewayRealtimeObserver,
	BrowserGatewayRealtimeSubscription,
} from '../../../platform/gateway/browserGatewayRealtime'

const ARTIFACT_READ_PATH = '/api/blobs/communications-export/v1/artifact'
const CANONICAL_ID_BYTES = 16
const READ_TICKET_BYTES = 32
const MAX_MESSAGES = 64
const MAX_ARTIFACT_BYTES = 24 * 1024 * 1024
const REALTIME_CONTRACT = 'communications.export.status_changed'
const REALTIME_EVENT_KIND = 'communications.export.status_changed'
const MAX_BUFFERED_STATUSES = 64

export type CommunicationsEvidenceExportRealtimeObserverV1 = {
	onStatus(status: EvidenceExportStatusChangedV1): void
	onUnavailable(): void
}

export type CommunicationsEvidenceExportRealtimeBindingV1 = BrowserGatewayRealtimeSubscription & {
	ready: Promise<void>
	attachExport(exportId: Uint8Array): void
}

type CommunicationsEvidenceExportRealtimePort = {
	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription
}

type CommunicationsEvidenceExportPorts = {
	start(
		messageIds: Uint8Array[],
		operationId: Uint8Array,
		signal?: AbortSignal,
	): Promise<{ exportId: Uint8Array; error: CommunicationsExportErrorCodeV1 }>
	status(
		exportId: Uint8Array,
		signal?: AbortSignal,
	): Promise<GetEvidenceExportStatusResponseV1>
	issueRead(
		exportId: Uint8Array,
		signal?: AbortSignal,
	): Promise<{
		opaqueReadCapability: Uint8Array
		declaredBytes: bigint
		expiresAtUnixSeconds: bigint
		error: CommunicationsExportErrorCodeV1
	}>
	readBlob(input: RequestInfo | URL, init: RequestInit): Promise<Response>
}

export async function startCommunicationsEvidenceExport(
	messageIds: Uint8Array[],
	operationId: Uint8Array,
	signal?: AbortSignal,
	ports: CommunicationsEvidenceExportPorts = defaultPorts(),
): Promise<Uint8Array> {
	validateMessageIds(messageIds)
	validateId(operationId, 'Evidence export operation')
	const response = await ports.start(messageIds.map(copyBytes), copyBytes(operationId), signal)
	if (
		response.error !== CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_UNSPECIFIED
		|| !equalBytes(response.exportId, operationId)
	) {
		throw new Error('Communications evidence export was not accepted')
	}
	return copyBytes(response.exportId)
}

export async function getCommunicationsEvidenceExportStatus(
	exportId: Uint8Array,
	signal?: AbortSignal,
	ports: CommunicationsEvidenceExportPorts = defaultPorts(),
): Promise<GetEvidenceExportStatusResponseV1> {
	validateId(exportId, 'Evidence export')
	const response = await ports.status(copyBytes(exportId), signal)
	const expectedError = response.status === EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_REJECTED
		? CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_POLICY_REJECTED
		: CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_UNSPECIFIED
	if (
		response.error !== expectedError
		|| !equalBytes(response.exportId, exportId)
		|| response.requestedItems < 1
		|| response.requestedItems > MAX_MESSAGES
		|| response.completedItems > response.requestedItems
		|| response.artifactBytes > BigInt(MAX_ARTIFACT_BYTES)
	) {
		throw new Error('Communications evidence export status is unavailable')
	}
	return response
}

export function openCommunicationsEvidenceExportRealtime(
	observer: CommunicationsEvidenceExportRealtimeObserverV1,
	hub: CommunicationsEvidenceExportRealtimePort = getBrowserGatewayRealtimeHub(),
): CommunicationsEvidenceExportRealtimeBindingV1 {
	let selectedExportId: Uint8Array | undefined
	const buffered: EvidenceExportStatusChangedV1[] = []
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
			rejectReady?.(new Error('Communications evidence export realtime is unavailable'))
		}
		observer.onUnavailable()
	}
	const subscription = hub.subscribe({
		onEvent: event => {
			try {
				const status = decodeRealtimeStatus(event)
				if (!status) return
				if (!selectedExportId) {
					if (buffered.length === MAX_BUFFERED_STATUSES) buffered.shift()
					buffered.push(status)
					return
				}
				if (equalBytes(status.exportId, selectedExportId)) observer.onStatus(status)
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
			} else if (state.state
				=== ClientRealtimeStreamStateKindV1.CLIENT_REALTIME_STREAM_STATE_KIND_CLOSED) {
				unavailable()
			}
		},
		onReplayGap: unavailable,
		onProtocolError: unavailable,
	})
	return {
		ready,
		attachExport: exportId => {
			validateId(exportId, 'Evidence export')
			selectedExportId = copyBytes(exportId)
			for (const status of buffered) {
				if (equalBytes(status.exportId, selectedExportId)) observer.onStatus(status)
			}
			buffered.length = 0
		},
		close: () => subscription.close(),
	}
}

export async function readCommunicationsEvidenceExport(
	exportId: Uint8Array,
	signal?: AbortSignal,
	ports: CommunicationsEvidenceExportPorts = defaultPorts(),
): Promise<Uint8Array> {
	validateId(exportId, 'Evidence export')
	const ticket = await ports.issueRead(copyBytes(exportId), signal)
	const declaredBytes = Number(ticket.declaredBytes)
	if (
		ticket.error !== CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_UNSPECIFIED
		|| ticket.opaqueReadCapability.byteLength !== READ_TICKET_BYTES
		|| !Number.isSafeInteger(declaredBytes)
		|| declaredBytes < 1
		|| declaredBytes > MAX_ARTIFACT_BYTES
		|| ticket.expiresAtUnixSeconds <= BigInt(Math.floor(Date.now() / 1_000))
	) {
		throw new Error('Communications evidence export read ticket is unavailable')
	}
	const request = create(EvidenceExportArtifactReadRequestV1Schema, {
		opaqueReadCapability: ticket.opaqueReadCapability,
	})
	const response = await ports.readBlob(ARTIFACT_READ_PATH, {
		method: 'POST',
		headers: {
			accept: 'application/octet-stream',
			'content-type': 'application/protobuf',
		},
		body: toBinary(EvidenceExportArtifactReadRequestV1Schema, request),
		signal,
	})
	if (
		!response.ok
		|| response.headers.get('content-type')?.split(';', 1)[0] !== 'application/octet-stream'
	) {
		throw new Error('Communications evidence export artifact is unavailable')
	}
	const bytes = new Uint8Array(await response.arrayBuffer())
	if (bytes.byteLength !== declaredBytes || bytes.byteLength > MAX_ARTIFACT_BYTES) {
		throw new Error('Communications evidence export artifact length is invalid')
	}
	return bytes
}

function defaultPorts(): CommunicationsEvidenceExportPorts {
	const browserGateway = new BrowserGatewayFetch()
	return {
		start: (messageIds, operationId, signal) => (
			getCommunicationsExportCommandClient().start(
				{ protocolMajor: 1, messageIds, operationId },
				{ signal },
			)
		),
		status: (exportId, signal) => (
			getCommunicationsExportQueryClient().getStatus(
				{ protocolMajor: 1, exportId },
				{ signal },
			)
		),
		issueRead: (exportId, signal) => (
			getCommunicationsExportTicketClient().issueRead(
				{ protocolMajor: 1, exportId },
				{ signal },
			)
		),
		readBlob: browserGateway.fetch.bind(browserGateway),
	}
}

function decodeRealtimeStatus(event: ClientRealtimeEventV1): EvidenceExportStatusChangedV1 | undefined {
	if (
		event.contractName !== REALTIME_CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== REALTIME_EVENT_KIND
	) return undefined
	const status = fromBinary(EvidenceExportStatusChangedV1Schema, event.payload)
	if (!validRealtimeStatus(status)) throw new Error('invalid status')
	return status
}

function validRealtimeStatus(status: EvidenceExportStatusChangedV1): boolean {
	const expectedError = status.status === EvidenceExportStatusV1.EVIDENCE_EXPORT_STATUS_REJECTED
		? CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_POLICY_REJECTED
		: CommunicationsExportErrorCodeV1.COMMUNICATIONS_EXPORT_ERROR_CODE_UNSPECIFIED
	return status.error === expectedError
		&& status.exportId.byteLength === CANONICAL_ID_BYTES
		&& status.exportId.some(byte => byte !== 0)
		&& status.requestedItems >= 1
		&& status.requestedItems <= MAX_MESSAGES
		&& status.completedItems <= status.requestedItems
		&& status.artifactBytes <= BigInt(MAX_ARTIFACT_BYTES)
		&& status.occurredAtUnixMillis > 0n
}

function validateMessageIds(messageIds: Uint8Array[]): void {
	if (messageIds.length < 1 || messageIds.length > MAX_MESSAGES) {
		throw new RangeError(`Evidence export requires 1-${MAX_MESSAGES} canonical messages`)
	}
	const seen = new Set<string>()
	for (const messageId of messageIds) {
		validateId(messageId, 'Canonical message')
		const key = bytesKey(messageId)
		if (seen.has(key)) throw new RangeError('Evidence export message IDs must be unique')
		seen.add(key)
	}
}

function validateId(value: Uint8Array, label: string): void {
	if (value.byteLength !== CANONICAL_ID_BYTES || value.every((byte) => byte === 0)) {
		throw new RangeError(`${label} ID must be ${CANONICAL_ID_BYTES} non-zero bytes`)
	}
}

function bytesKey(value: Uint8Array): string {
	return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function equalBytes(left: Uint8Array, right: Uint8Array): boolean {
	return left.byteLength === right.byteLength && left.every((byte, index) => byte === right[index])
}

function copyBytes(value: Uint8Array): Uint8Array {
	return new Uint8Array(value)
}
