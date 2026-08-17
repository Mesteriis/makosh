import { Code, ConnectError } from '@connectrpc/connect'

import {
	type TelegramRuntimeRequestPriority,
	withTelegramRuntimeRequestQueue,
} from '../api/telegramRuntimeRequestQueue'

const DEFAULT_ATTEMPTS = 40
const DEFAULT_DELAY_MILLIS = 250

export type TelegramConfigurationRuntimeRetryOptionsV1 = {
	attempts?: number
	delayMillis?: number
	wait?: (delayMillis: number) => Promise<void>
	priority?: TelegramRuntimeRequestPriority
}

/**
 * Settings application restarts the managed integration asynchronously. The
 * provider-owned configuration client is therefore allowed a short, bounded
 * retry only while the Core route reports that exact control-channel handoff.
 */
export async function withTelegramConfigurationRuntimeV1<T>(
	operation: () => Promise<T>,
	options: TelegramConfigurationRuntimeRetryOptionsV1 = {},
): Promise<T> {
	const attempts = options.attempts ?? DEFAULT_ATTEMPTS
	const delayMillis = options.delayMillis ?? DEFAULT_DELAY_MILLIS
	const wait = options.wait ?? waitForDelay
	const priority = options.priority ?? 'interactive'
	if (!Number.isSafeInteger(attempts) || attempts <= 0) {
		throw new Error('telegram_configuration_retry_attempts_invalid')
	}
	if (!Number.isSafeInteger(delayMillis) || delayMillis < 0) {
		throw new Error('telegram_configuration_retry_delay_invalid')
	}
	for (let attempt = 1; attempt <= attempts; attempt += 1) {
		try {
			return await withTelegramRuntimeRequestQueue(operation, priority)
		} catch (error) {
			if (attempt === attempts || !isConfigurationRuntimeHandoff(error)) throw error
			await wait(delayMillis)
		}
	}
	throw new Error('telegram_configuration_retry_exhausted')
}

function isConfigurationRuntimeHandoff(error: unknown): boolean {
	if (error instanceof ConnectError) {
		return error.code === Code.Internal
			|| error.code === Code.Unavailable
			|| error.code === Code.Unknown
	}
	if (isStructurallyTransientConnectError(error)) return true
	return error instanceof Error
		&& /telegram runtime control channel is unavailable/i.test(error.message)
}

function isStructurallyTransientConnectError(
	error: unknown,
): error is { name: 'ConnectError'; code: Code } {
	if (!error || typeof error !== 'object') return false
	const candidate = error as { name?: unknown; code?: unknown }
	return candidate.name === 'ConnectError'
		&& (candidate.code === Code.Internal
			|| candidate.code === Code.Unavailable
			|| candidate.code === Code.Unknown)
}

function waitForDelay(delayMillis: number): Promise<void> {
	return new Promise((resolve) => globalThis.setTimeout(resolve, delayMillis))
}
