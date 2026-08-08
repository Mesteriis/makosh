import { fromBinary } from '@bufbuild/protobuf'

import {
	type GetMailContactsSyncResponseV1,
	MailContactsSyncDirectionV1,
	MailContactsSyncErrorCodeV1,
	MailContactsSyncStateV1,
	MailContactsSyncStatusChangedV1Schema,
	type MailContactsSyncStatusChangedV1,
} from '../../../gen/makosh/mail_contacts_sync/v1/sync_pb'
import {
	ClientRealtimeStreamStateKindV1,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import { getMailContactsSyncCommandClient } from '../../../platform/connect/mailContactsSyncCommandClient'
import { getMailContactsSyncQueryClient } from '../../../platform/connect/mailContactsSyncQueryClient'
import { getBrowserGatewayRealtimeHub } from '../../../platform/gateway/browserGatewayRealtimeHub'
import type {
	BrowserGatewayRealtimeObserver,
	BrowserGatewayRealtimeSubscription,
} from '../../../platform/gateway/browserGatewayRealtime'

const REALTIME_CONTRACT = 'mail_contacts_sync_realtime'
const REALTIME_EVENT_KIND = 'mail.contacts-sync.status-changed.v1'
const ID_BYTES = 16
const MAX_BUFFERED_STATUSES = 64

export type MailContactsSyncRealtimeObserverV1 = {
	onStatus(status: MailContactsSyncStatusChangedV1): void
	onUnavailable(): void
}

export type MailContactsSyncRealtimeBindingV1 = BrowserGatewayRealtimeSubscription & {
	ready: Promise<void>
	attachRun(runId: Uint8Array): void
}

type MailContactsSyncRealtimePort = {
	subscribe(observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription
}

export function openMailContactsSyncRealtime(
	observer: MailContactsSyncRealtimeObserverV1,
	hub: MailContactsSyncRealtimePort = getBrowserGatewayRealtimeHub(),
): MailContactsSyncRealtimeBindingV1 {
	let selectedRunId: Uint8Array | undefined
	const buffered: MailContactsSyncStatusChangedV1[] = []
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
			rejectReady?.(new Error('Mail Contacts Sync realtime is unavailable'))
		}
		observer.onUnavailable()
	}
	const subscription = hub.subscribe({
		onEvent: event => {
			const status = decodeStatus(event)
			if (!status) return
			if (!selectedRunId) {
				if (buffered.length === MAX_BUFFERED_STATUSES) buffered.shift()
				buffered.push(status)
				return
			}
			if (equal(status.runId, selectedRunId)) observer.onStatus(status)
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
		attachRun: runId => {
			validateId(runId, 'Mail Contacts Sync run')
			selectedRunId = copy(runId)
			for (const status of buffered) {
				if (equal(status.runId, selectedRunId)) observer.onStatus(status)
			}
			buffered.length = 0
		},
		close: () => subscription.close(),
	}
}

export async function startMailContactsSync(
	accountId: string,
	direction: MailContactsSyncDirectionV1,
	operationId: Uint8Array,
	signal?: AbortSignal,
): Promise<Uint8Array> {
	if (accountId.length === 0
		|| accountId.length > 256
		|| accountId.trim() !== accountId
		|| !/^[\x20-\x7e]+$/.test(accountId)) throw new Error('Mail account is invalid')
	validateId(operationId, 'Mail Contacts Sync operation')
	if (direction !== MailContactsSyncDirectionV1.MAIL_CONTACTS_SYNC_DIRECTION_PROVIDER_TO_CONTACTS
		&& direction !== MailContactsSyncDirectionV1.MAIL_CONTACTS_SYNC_DIRECTION_BIDIRECTIONAL) {
		throw new Error('Mail Contacts Sync direction is invalid')
	}
	const response = await getMailContactsSyncCommandClient().start({
		protocolMajor: 1,
		operationId: copy(operationId),
		accountId,
		direction,
	}, { signal })
	if (!validId(response.runId)
		|| response.state !== MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_ACCEPTED
		|| response.error !== MailContactsSyncErrorCodeV1.MAIL_CONTACTS_SYNC_ERROR_CODE_UNSPECIFIED) {
		throw new Error('Mail Contacts Sync was not accepted')
	}
	return copy(response.runId)
}

export async function getMailContactsSync(
	runId: Uint8Array,
	signal?: AbortSignal,
): Promise<GetMailContactsSyncResponseV1> {
	validateId(runId, 'Mail Contacts Sync run')
	const response = await getMailContactsSyncQueryClient().get({
		protocolMajor: 1,
		runId: copy(runId),
	}, { signal })
	if (!equal(response.runId, runId)
		|| !response.accountId.trim()
		|| response.stateRevision < 1n
		|| !validStateError(response.state, response.error)) {
		throw new Error('Mail Contacts Sync status is invalid')
	}
	return response
}

function decodeStatus(event: ClientRealtimeEventV1): MailContactsSyncStatusChangedV1 | null {
	if (event.contractName !== REALTIME_CONTRACT
		|| event.contractVersion !== 1
		|| event.eventKind !== REALTIME_EVENT_KIND) return null
	try {
		const status = fromBinary(MailContactsSyncStatusChangedV1Schema, event.payload)
		return validId(status.runId)
			&& status.stateRevision >= 1n
			&& validStateError(status.state, status.error)
			? status
			: null
	} catch {
		return null
	}
}

function validStateError(
	state: MailContactsSyncStateV1,
	error: MailContactsSyncErrorCodeV1,
): boolean {
	if (state < MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_ACCEPTED
		|| state > MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_REJECTED) return false
	return state === MailContactsSyncStateV1.MAIL_CONTACTS_SYNC_STATE_REJECTED
		? error !== MailContactsSyncErrorCodeV1.MAIL_CONTACTS_SYNC_ERROR_CODE_UNSPECIFIED
		: error === MailContactsSyncErrorCodeV1.MAIL_CONTACTS_SYNC_ERROR_CODE_UNSPECIFIED
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
