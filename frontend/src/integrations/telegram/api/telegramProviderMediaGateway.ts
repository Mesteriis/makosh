import { create, toBinary } from '@bufbuild/protobuf'

import {
	FileIdQuerySchema,
	type TelegramFileSnapshotProjection,
} from '../../../gen/makosh/telegram/v1/client_pb'
import { BrowserGatewayFetch } from '../../../platform/gateway/browserGatewayFetch'
import { downloadTelegramFile } from './telegramMediaCommandGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'
import { withTelegramOperationalRuntimeV1 } from './telegramOperationalRuntimeRetry'
import { withTelegramRuntimeRequestQueue } from './telegramRuntimeRequestQueue'
import {
	TelegramMediaMemoryCache,
	type TelegramMediaCacheClass,
} from './telegramMediaMemoryCache'
import { TelegramMediaLaneScheduler } from './telegramMediaLaneScheduler'

const MEDIA_READ_PATH = '/api/blobs/telegram/v1/media'
const MAX_MEDIA_BYTES = 4 * 1024 * 1024 * 1024
const INTERACTIVE_POLL_ATTEMPTS = 1_200
const BACKGROUND_POLL_ATTEMPTS = 8
const POLL_DELAY_MILLIS = 500
const PROVIDER_START_STALL_ATTEMPTS = 60
const PROVIDER_DOWNLOAD_STALL_ATTEMPTS = 120
const MAX_CONCURRENT_MATERIALIZATIONS = 4
const MAX_CONCURRENT_MATERIALIZATIONS_PER_ACCOUNT = 1
const MEDIA_READ_ATTEMPTS = 40
const MEDIA_READ_RETRY_DELAY_MILLIS = 250

export type TelegramProviderMediaArtifact = {
	url: string
	sizeBytes: number
}

export type TelegramProviderMediaPriority = 'interactive' | 'background'
export type TelegramProviderMediaDelivery = 'inline' | 'range'

export function telegramMediaDeliveryForKind(kind: string): TelegramProviderMediaDelivery {
	return ['animation', 'audio', 'video'].includes(kind) ? 'range' : 'inline'
}

export function telegramMediaPollAttemptLimit(priority: TelegramProviderMediaPriority): number {
	return priority === 'interactive' ? INTERACTIVE_POLL_ATTEMPTS : BACKGROUND_POLL_ATTEMPTS
}

export function shouldStopTelegramMediaPolling(
	file: Pick<TelegramFileSnapshotProjection, 'isDownloaded' | 'isDownloading'> | undefined,
	stalledAttempts: number,
): boolean {
	if (file?.isDownloaded) return false
	const stallLimit = file?.isDownloading
		? PROVIDER_DOWNLOAD_STALL_ATTEMPTS
		: PROVIDER_START_STALL_ATTEMPTS
	return stalledAttempts >= stallLimit
}

export async function telegramMediaDownloadOperationId(
	accountId: string,
	providerFileId: string,
): Promise<string> {
	const input = new TextEncoder().encode(
		`${requireIdentifier('account ID', accountId)}\u0000${requireIdentifier('provider file ID', providerFileId)}`,
	)
	const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', input))
	return `telegram-media-download-${Array.from(
		digest,
		byte => byte.toString(16).padStart(2, '0'),
	).join('')}`
}

const memoryCache = new TelegramMediaMemoryCache({
	avatar: { maxEntries: 512, maxBytes: 64 * 1024 * 1024 },
	media: { maxEntries: 64, maxBytes: 256 * 1024 * 1024 },
}, url => URL.revokeObjectURL(url))
const pending = new Map<string, Promise<TelegramProviderMediaArtifact>>()
const materializationScheduler = new TelegramMediaLaneScheduler(
	MAX_CONCURRENT_MATERIALIZATIONS,
	MAX_CONCURRENT_MATERIALIZATIONS_PER_ACCOUNT,
)
let activeScopeKey = ''
let activeChatListScopeKey = ''

export function activateTelegramMediaScope(scopeKey: string, chatListScopeKey: string): void {
	activeScopeKey = requireIdentifier('media scope', scopeKey)
	activeChatListScopeKey = requireIdentifier('chat list media scope', chatListScopeKey)
	materializationScheduler.notifyScopeChanged()
}

export function deactivateTelegramMediaScopes(): void {
	activeScopeKey = ''
	activeChatListScopeKey = ''
	materializationScheduler.notifyScopeChanged()
}

export function loadTelegramProviderMedia(
	accountId: string,
	providerFileId: string,
	contentType: string,
	scopeKey: string,
	priority: TelegramProviderMediaPriority = 'interactive',
	cacheClass: TelegramMediaCacheClass = 'media',
	delivery: TelegramProviderMediaDelivery = 'inline',
): Promise<TelegramProviderMediaArtifact> {
	const normalizedAccountId = requireIdentifier('account ID', accountId)
	const normalizedProviderFileId = requireIdentifier('provider file ID', providerFileId)
	const normalizedScopeKey = requireIdentifier('media scope', scopeKey)
	const key = `${normalizedAccountId}:${normalizedProviderFileId}`
	const existing = memoryCache.get(cacheClass, key)
	if (existing) return Promise.resolve(existing)
	const pendingKey = `${cacheClass}:${delivery}:${normalizedScopeKey}:${key}`
	const inflight = pending.get(pendingKey)
	if (inflight) return inflight
	const load = materializationScheduler.schedule({
		laneKey: normalizedAccountId,
		scopeKey: normalizedScopeKey,
		priority,
		isScopeActive: isActiveScope,
		run: () =>
		materializeMedia(
			normalizedAccountId,
			normalizedProviderFileId,
			contentType,
			normalizedScopeKey,
			priority,
			delivery,
		),
	})
		.then((artifact) => {
			if (artifact.url.startsWith('blob:')) memoryCache.set(cacheClass, key, artifact)
			return artifact
		})
		.finally(() => pending.delete(pendingKey))
	pending.set(pendingKey, load)
	return load
}

export function readCachedTelegramProviderMedia(
	accountId: string,
	providerFileId: string,
	cacheClass: TelegramMediaCacheClass = 'media',
): TelegramProviderMediaArtifact | undefined {
	const normalizedAccountId = accountId.trim()
	const normalizedProviderFileId = providerFileId.trim()
	if (!normalizedAccountId || !normalizedProviderFileId) return undefined
	return memoryCache.get(cacheClass, `${normalizedAccountId}:${normalizedProviderFileId}`)
}

async function materializeMedia(
	accountId: string,
	providerFileId: string,
	contentType: string,
	scopeKey: string,
	priority: TelegramProviderMediaPriority,
	delivery: TelegramProviderMediaDelivery,
): Promise<TelegramProviderMediaArtifact> {
	assertActiveScope(scopeKey)
	let file = await loadFile(accountId, providerFileId)
	assertActiveScope(scopeKey)
	if (!hasBlobReceipt(file)) {
		const commandStartedAt = Date.now()
		const operation = await downloadTelegramFile({
			accountId,
			providerFileId,
			operationId: await telegramMediaDownloadOperationId(accountId, providerFileId),
			priority: 32,
		}, 'media')
		recordTelegramMediaSpan('download_command', commandStartedAt, 'completed', {
			state: operation.state,
			retryCount: operation.retryCount,
			maxRetries: operation.maxRetries,
			hasProviderError: Boolean(operation.lastError),
		})
		assertActiveScope(scopeKey)
		const pollAttempts = telegramMediaPollAttemptLimit(priority)
		let lastDownloadedSize = Number(file?.downloadedSizeBytes ?? 0n)
		let stalledAttempts = 0
		for (let attempt = 0; attempt < pollAttempts; attempt += 1) {
			await wait(POLL_DELAY_MILLIS)
			assertActiveScope(scopeKey)
			file = await loadFile(accountId, providerFileId)
			assertActiveScope(scopeKey)
			if (hasBlobReceipt(file)) break
			const downloadedSize = Number(file?.downloadedSizeBytes ?? 0n)
			if (Number.isSafeInteger(downloadedSize) && downloadedSize > lastDownloadedSize) {
				lastDownloadedSize = downloadedSize
				stalledAttempts = 0
			} else {
				stalledAttempts += 1
			}
			if (shouldStopTelegramMediaPolling(file, stalledAttempts)) break
		}
		recordTelegramMediaSpan('blob_receipt_poll', commandStartedAt, hasBlobReceipt(file)
			? 'completed'
			: 'failed', {
			priority,
			isDownloading: file?.isDownloading ?? false,
			isDownloaded: file?.isDownloaded ?? false,
			downloadedSizeBytes: Number(file?.downloadedSizeBytes ?? 0n),
			expectedSizeBytes: Number(file?.expectedSizeBytes ?? file?.sizeBytes ?? 0n),
		})
	}
	if (!hasBlobReceipt(file)) throw new Error('Telegram media download is unavailable')
	const declaredSize = Number(file.sizeBytes ?? file.downloadedSizeBytes ?? 0n)
	if (!Number.isSafeInteger(declaredSize) || declaredSize < 1 || declaredSize > MAX_MEDIA_BYTES) {
		throw new Error('Telegram media size is invalid')
	}
	const request = create(FileIdQuerySchema, { accountId, providerFileId })
	const type = normalizedContentType(contentType)
	const streamable = delivery === 'range'
	const response = await readMediaBytes(
		accountId,
		toBinary(FileIdQuerySchema, request),
		scopeKey,
		streamable ? type : undefined,
	)
	assertActiveScope(scopeKey)
	if (!response.ok || response.headers.get('cache-control') !== 'no-store') {
		throw new Error('Telegram media bytes are unavailable')
	}
	if (streamable) {
		const streamUrl = (await response.text()).trim()
		assertActiveScope(scopeKey)
		if (!new RegExp(`^${MEDIA_READ_PATH}/stream/[a-f0-9]{64}$`).test(streamUrl)) {
			throw new Error('Telegram media stream is invalid')
		}
		return { url: streamUrl, sizeBytes: declaredSize }
	}
	const bytes = await response.arrayBuffer()
	if (bytes.byteLength !== declaredSize || bytes.byteLength > MAX_MEDIA_BYTES) {
		throw new Error('Telegram media length is invalid')
	}
	return {
		url: URL.createObjectURL(new Blob([bytes], { type })),
		sizeBytes: bytes.byteLength,
	}
}

async function readMediaBytes(
	accountId: string,
	requestBody: Uint8Array,
	scopeKey: string,
	streamContentType?: string,
): Promise<Response> {
	const gateway = new BrowserGatewayFetch()
	const body = Uint8Array.from(requestBody).buffer
	for (let attempt = 1; attempt <= MEDIA_READ_ATTEMPTS; attempt += 1) {
		assertActiveScope(scopeKey)
		const response = await withTelegramRuntimeRequestQueue(() => gateway.fetch(MEDIA_READ_PATH, {
			method: 'POST',
				headers: {
					accept: 'application/octet-stream',
					'content-type': 'application/protobuf',
					...(streamContentType ? {
						'x-makosh-blob-mode': 'range-v1',
						'x-makosh-blob-content-type': streamContentType,
					} : {}),
			},
			body,
		}), 'media', accountId)
		assertActiveScope(scopeKey)
		if (response.status !== 503 || attempt === MEDIA_READ_ATTEMPTS) return response
		await wait(MEDIA_READ_RETRY_DELAY_MILLIS)
	}
	throw new Error('Telegram media read retry exhausted')
}

async function loadFile(
	accountId: string,
	providerFileId: string,
): Promise<TelegramFileSnapshotProjection | undefined> {
	const response = await withTelegramOperationalRuntimeV1(
		() => getTelegramOperationalConnectClient().executeQuery({
			query: { case: 'file', value: { accountId, providerFileId } },
		}),
		'media',
		accountId,
	)
	if (response.response.case !== 'file') throw new Error('Telegram file projection is unavailable')
	return response.response.value.file
}

function hasBlobReceipt(file?: TelegramFileSnapshotProjection): file is TelegramFileSnapshotProjection & {
	blobReferenceId: Uint8Array
	blobPlaintextSha256: Uint8Array
	blobBackupClass: number
} {
	return file?.isDownloaded === true
		&& file.blobReferenceId?.byteLength === 16
		&& file.blobPlaintextSha256?.byteLength === 32
		&& (file.blobBackupClass ?? 0) > 0
}

function normalizedContentType(value: string): string {
	const normalized = value.trim().toLowerCase()
	if (/^(image|video|audio)\/[a-z0-9.+-]+$/.test(normalized)) return normalized
	return 'application/octet-stream'
}

function wait(delayMillis: number): Promise<void> {
	return new Promise((resolve) => globalThis.setTimeout(resolve, delayMillis))
}

function assertActiveScope(scopeKey: string): void {
	if (!isActiveScope(scopeKey)) throw scopeChangedError()
}

function isActiveScope(scopeKey: string): boolean {
	return scopeKey === activeScopeKey || scopeKey === activeChatListScopeKey
}

function scopeChangedError(): Error {
	const error = new Error('Telegram media request was superseded')
	error.name = 'AbortError'
	return error
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) throw new RangeError(`Telegram ${label} is required`)
	return normalized
}

function recordTelegramMediaSpan(
	stage: 'download_command' | 'blob_receipt_poll',
	startedAt: number,
	outcome: 'completed' | 'failed',
	fields: Record<string, unknown>,
): void {
	if (!import.meta.env.DEV) return
	console.debug('telegram_media.span', JSON.stringify({
		provider: 'telegram',
		stage,
		outcome,
		durationMillis: Math.max(0, Date.now() - startedAt),
		...fields,
	}))
}
