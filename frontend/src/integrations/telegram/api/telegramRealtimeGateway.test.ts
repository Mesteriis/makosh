import { beforeEach, describe, expect, it, vi } from 'vitest'

import { replayTelegramRealtime } from './telegramRealtimeGateway'
import { getTelegramRealtimeConnectClient } from './telegramRealtimeClient'

vi.mock('./telegramRealtimeClient', () => ({
	getTelegramRealtimeConnectClient: vi.fn(),
}))

const replay = vi.fn()

describe('Telegram realtime Gateway adapter', () => {
	beforeEach(() => {
		replay.mockReset()
		vi.mocked(getTelegramRealtimeConnectClient).mockReturnValue({ replay } as never)
	})

	it('replays only the selected account through the admitted realtime contract', async () => {
		replay.mockResolvedValue({
			frame: [{ accountId: 'account-1', sequence: 7n }],
			nextAfterSequence: 7n,
			resetRequired: false,
			contractMajor: 1,
		})

		await expect(replayTelegramRealtime(' account-1 ', 6n)).resolves.toMatchObject({
			nextAfterSequence: 7n,
			resetRequired: false,
		})
		expect(replay).toHaveBeenCalledWith({
			accountId: 'account-1',
			afterSequence: 6n,
			limit: 100,
		})
	})

	it('rejects a cross-account frame before it reaches the projection', async () => {
		replay.mockResolvedValue({
			frame: [{ accountId: 'account-2', sequence: 7n }],
			nextAfterSequence: 7n,
			resetRequired: false,
			contractMajor: 1,
		})

		await expect(replayTelegramRealtime('account-1', 6n)).rejects.toThrow(
			'crossed an account boundary',
		)
	})
})
