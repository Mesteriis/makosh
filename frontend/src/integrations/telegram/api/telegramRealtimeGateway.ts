import type { TelegramRealtimeFrameProjection } from '../../../gen/makosh/telegram/v1/client_pb'
import { withTelegramConfigurationRuntimeV1 } from '../setup/telegramConfigurationRuntimeRetry'
import { getTelegramRealtimeConnectClient } from './telegramRealtimeClient'

const REPLAY_LIMIT = 100
const TELEGRAM_REALTIME_CONTRACT_MAJOR = 1

export type TelegramRealtimeReplay = {
	frames: readonly TelegramRealtimeFrameProjection[]
	nextAfterSequence: bigint
	resetRequired: boolean
}

export async function replayTelegramRealtime(
	accountId: string,
	afterSequence: bigint,
): Promise<TelegramRealtimeReplay> {
	const normalizedAccountId = accountId.trim()
	if (!normalizedAccountId) throw new RangeError('Telegram account ID is required')
	if (afterSequence < 0n) throw new RangeError('Telegram realtime cursor is invalid')
	const response = await withTelegramConfigurationRuntimeV1(() =>
		getTelegramRealtimeConnectClient().replay({
			accountId: normalizedAccountId,
			afterSequence,
			limit: REPLAY_LIMIT,
		}),
		{ priority: 'background' },
	)
	if (response.contractMajor !== TELEGRAM_REALTIME_CONTRACT_MAJOR) {
		throw new Error('Telegram realtime contract is unavailable')
	}
	if (response.frame.some(frame => frame.accountId !== normalizedAccountId)) {
		throw new Error('Telegram realtime replay crossed an account boundary')
	}
	return {
		frames: response.frame,
		nextAfterSequence: response.nextAfterSequence,
		resetRequired: response.resetRequired,
	}
}
