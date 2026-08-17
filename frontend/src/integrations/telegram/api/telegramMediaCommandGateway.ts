import type { TelegramOperationResponse } from '../../../gen/makosh/telegram/v1/client_pb'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'
import { withTelegramOperationalRuntimeV1 } from './telegramOperationalRuntimeRetry'

const MAX_REFERENCE_ID_BYTES = 64

export type TelegramMediaTarget = {
	accountId: string
	providerChatId: string
	operationId: string
}

export async function sendTelegramMedia(input: TelegramMediaTarget & {
	mediaKind: string
	blobRef: string
	referenceIdHex: string
	declaredSize: bigint
	backupClass: number
	caption?: string
	filename?: string
}): Promise<TelegramOperationResponse> {
	if (input.declaredSize <= 0n) {
		throw new RangeError('Telegram media size must be positive')
	}
	return withTelegramOperationalRuntimeV1(() => getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'sendMedia',
			value: {
				...normalizeTarget(input),
				mediaKind: requireIdentifier('media kind', input.mediaKind),
				blob: {
					blobRef: requireIdentifier('Blob reference', input.blobRef),
					referenceId: parseReferenceId(input.referenceIdHex),
					declaredSize: input.declaredSize,
					backupClass: input.backupClass,
				},
				caption: optionalIdentifier(input.caption),
				filename: optionalIdentifier(input.filename),
			},
		},
		}), 'interactive', input.accountId)
}

export async function downloadTelegramFile(input: {
	accountId: string
	providerFileId: string
	operationId: string
	priority: number
	}, requestPriority: 'interactive' | 'media' = 'interactive'): Promise<TelegramOperationResponse> {
	return withTelegramOperationalRuntimeV1(() => getTelegramOperationalConnectClient().executeCommand({
		command: {
			case: 'downloadFile',
			value: {
				accountId: requireIdentifier('account ID', input.accountId),
				providerFileId: requireIdentifier('provider file ID', input.providerFileId),
				operationId: requireIdentifier('operation ID', input.operationId),
				priority: input.priority,
			},
		},
		}), requestPriority, input.accountId)
}

function normalizeTarget(target: TelegramMediaTarget): TelegramMediaTarget {
	return {
		accountId: requireIdentifier('account ID', target.accountId),
		providerChatId: requireIdentifier('chat ID', target.providerChatId),
		operationId: requireIdentifier('operation ID', target.operationId),
	}
}

function parseReferenceId(value: string): Uint8Array {
	const normalized = value.trim().toLowerCase()
	if (!/^(?:[0-9a-f]{2})+$/.test(normalized)) {
		throw new RangeError('Telegram Blob reference ID must be even-length hexadecimal')
	}
	const bytes = Uint8Array.from(
		normalized.match(/[0-9a-f]{2}/g)?.map((pair) => Number.parseInt(pair, 16)) || [],
	)
	if (bytes.length === 0 || bytes.length > MAX_REFERENCE_ID_BYTES) {
		throw new RangeError(`Telegram Blob reference ID must be 1-${MAX_REFERENCE_ID_BYTES} bytes`)
	}
	return bytes
}

function optionalIdentifier(value?: string): string | undefined {
	const normalized = value?.trim()
	return normalized || undefined
}

function requireIdentifier(label: string, value: string): string {
	const normalized = value.trim()
	if (!normalized) {
		throw new RangeError(`${label} is required`)
	}
	return normalized
}
