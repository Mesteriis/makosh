import { Code, ConnectError } from '@connectrpc/connect'

import {
	type TelegramRuntimeRequestPriority,
	withTelegramRuntimeRequestQueue,
} from './telegramRuntimeRequestQueue'

const ATTEMPTS = 40
const DELAY_MILLIS = 250

export async function withTelegramOperationalRuntimeV1<T>(
	operation: () => Promise<T>,
	priority: TelegramRuntimeRequestPriority = 'interactive',
	accountId = 'configuration',
): Promise<T> {
	for (let attempt = 1; attempt <= ATTEMPTS; attempt += 1) {
		try {
			return await withTelegramRuntimeRequestQueue(operation, priority, accountId)
		} catch (error) {
			if (attempt === ATTEMPTS || !isTransientRuntimeContention(error)) throw error
			await new Promise<void>((resolve) => globalThis.setTimeout(resolve, DELAY_MILLIS))
		}
	}
	throw new Error('telegram_operational_retry_exhausted')
}

function isTransientRuntimeContention(error: unknown): boolean {
	if (error instanceof ConnectError) {
		return error.code === Code.Internal || error.code === Code.Unavailable || error.code === Code.Unknown
	}
	if (!error || typeof error !== 'object') return false
	const candidate = error as { name?: unknown; code?: unknown; message?: unknown }
	return candidate.name === 'ConnectError'
		&& (candidate.code === Code.Internal || candidate.code === Code.Unavailable || candidate.code === Code.Unknown)
		|| typeof candidate.message === 'string'
			&& /telegram runtime control channel is unavailable/i.test(candidate.message)
}
