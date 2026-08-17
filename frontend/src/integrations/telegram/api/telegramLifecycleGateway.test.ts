import { beforeEach, describe, expect, it, vi } from 'vitest'
import { Code, ConnectError } from '@connectrpc/connect'

import {
	listTelegramAccounts,
	provisionTelegramAccount,
	replayTelegramAccount,
	restartTelegramAccount,
	retireTelegramAccount,
	retryTelegramOperation,
} from './telegramLifecycleGateway'
import { getTelegramLifecycleConnectClient } from './telegramLifecycleClient'
import { getTelegramReconfigurationConnectClient } from './telegramReconfigurationClient'

vi.mock('./telegramLifecycleClient', () => ({
	getTelegramLifecycleConnectClient: vi.fn(),
}))
vi.mock('./telegramReconfigurationClient', () => ({
	getTelegramReconfigurationConnectClient: vi.fn(),
}))

const execute = vi.fn()
const executeReconfiguration = vi.fn()

describe('Telegram lifecycle Gateway adapter', () => {
	beforeEach(() => {
		execute.mockReset()
		executeReconfiguration.mockReset()
		vi.mocked(getTelegramLifecycleConnectClient).mockReturnValue({ execute } as never)
		vi.mocked(getTelegramReconfigurationConnectClient).mockReturnValue({
			execute: executeReconfiguration,
		} as never)
	})

	it('lists and provisions owner-local accounts', async () => {
		execute
			.mockRejectedValueOnce(new ConnectError('runtime busy', Code.Unavailable))
			.mockResolvedValueOnce({
				response: { case: 'accounts', value: { account: [{ accountId: 'account-1' }] } },
			})
			.mockResolvedValueOnce({
				response: { case: 'account', value: { accountId: 'account-2' } },
			})

		await expect(listTelegramAccounts()).resolves.toHaveLength(1)
		expect(execute).toHaveBeenCalledTimes(2)
		await expect(provisionTelegramAccount({
			accountId: ' account-2 ',
			displayName: ' Personal ',
			externalAccountId: ' @owner ',
			credentials: [],
		})).resolves.toMatchObject({ accountId: 'account-2' })

		expect(execute).toHaveBeenNthCalledWith(3, {
			request: {
				case: 'provision',
				value: {
					accountId: 'account-2',
					displayName: 'Personal',
					externalAccountId: '@owner',
					credential: [],
					qrAuthorized: false,
				},
			},
		})
	})

	it('keeps reconfiguration separate from replay, retry and retire lifecycle actions', async () => {
		execute
			.mockResolvedValueOnce({
				response: {
					case: 'operation',
					value: { operationId: 'replay-1', state: 'accepted' },
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'operation',
					value: { operationId: 'retry-1', state: 'accepted' },
				},
			})
			.mockResolvedValueOnce({ response: { case: 'accepted', value: { operationId: 'retire-1' } } })
		executeReconfiguration.mockResolvedValueOnce({
			reconfigurationId: 'reconfigure-1',
			accountId: 'account-1',
			expectedRuntimeEpoch: 7n,
			targetRuntimeEpoch: 8n,
			state: 'accepted',
			contractMajor: 1,
		})

		await expect(restartTelegramAccount(
			'account-1',
			7n,
			'reconfigure-1',
		)).resolves.toMatchObject({ targetRuntimeEpoch: 8n })
		await expect(replayTelegramAccount('account-1', 8n)).resolves.toMatchObject({
			operationId: 'replay-1',
		})
		await expect(retryTelegramOperation('retry-1', 101n)).resolves.toMatchObject({
			operationId: 'retry-1',
		})
		await expect(retireTelegramAccount('account-1')).resolves.toBe('retire-1')

		expect(executeReconfiguration).toHaveBeenCalledWith({
			request: {
				case: 'begin',
				value: {
					reconfigurationId: 'reconfigure-1',
					accountId: 'account-1',
					expectedRuntimeEpoch: 7n,
				},
			},
		})
		expect(execute).toHaveBeenNthCalledWith(1, {
			request: {
				case: 'replay',
				value: { accountId: 'account-1', afterSequence: 8n, limit: 100 },
			},
		})
		expect(execute).toHaveBeenNthCalledWith(2, {
			request: {
				case: 'retry',
				value: {
					operationId: 'retry-1',
					nowUnixSeconds: 101n,
					nextAttemptAtUnixSeconds: 101n,
				},
			},
		})
		expect(execute).toHaveBeenNthCalledWith(3, {
			request: { case: 'retireAccount', value: { accountId: 'account-1' } },
		})
	})

	it('rejects missing lifecycle identifiers before transport', async () => {
		await expect(restartTelegramAccount(' ', 1n, 'reconfigure-1')).rejects.toThrow(
			'account ID is required',
		)
		await expect(restartTelegramAccount('account-1', 0n, 'reconfigure-1')).rejects.toThrow(
			'runtime epoch is required',
		)
		expect(execute).not.toHaveBeenCalled()
		expect(executeReconfiguration).not.toHaveBeenCalled()
	})
})
