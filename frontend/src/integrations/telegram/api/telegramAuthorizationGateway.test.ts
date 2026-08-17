import { beforeEach, describe, expect, it, vi } from 'vitest'
import { Code, ConnectError } from '@connectrpc/connect'

import {
	getTelegramAuthorizationStatus,
	submitTelegramAuthorizationPassword,
} from './telegramAuthorizationGateway'
import { getTelegramAuthorizationConnectClient } from './telegramAuthorizationClient'

vi.mock('./telegramAuthorizationClient', () => ({
	getTelegramAuthorizationConnectClient: vi.fn(),
}))

const authorize = vi.fn()

describe('Telegram authorization Gateway adapter', () => {
	beforeEach(() => {
		authorize.mockReset()
		vi.mocked(getTelegramAuthorizationConnectClient).mockReturnValue({ authorize } as never)
	})

	it('reads authorization state through the exact generated service', async () => {
		authorize
			.mockRejectedValueOnce(new ConnectError('runtime busy', Code.Unavailable))
			.mockResolvedValue({
				response: {
					case: 'authorizationStatus',
					value: { state: 'waiting_password', passwordHint: 'hint' },
				},
			})

		await expect(getTelegramAuthorizationStatus()).resolves.toEqual({
			state: 'waiting_password',
			passwordHint: 'hint',
			qrLink: undefined,
		})
		expect(authorize).toHaveBeenNthCalledWith(2, {
			request: { case: 'authorizationStatus', value: {} },
		})
	})

	it('submits a bounded password without exposing it to another owner adapter', async () => {
		authorize.mockResolvedValue({
			response: { case: 'passwordAccepted', value: {} },
		})

		await expect(submitTelegramAuthorizationPassword(' secret ')).resolves.toBeUndefined()
		expect(authorize).toHaveBeenCalledWith({
			request: {
				case: 'submitPassword',
				value: { password: 'secret' },
			},
		})
	})

	it('rejects an empty password before transport', async () => {
		await expect(submitTelegramAuthorizationPassword(' ')).rejects.toThrow(
			'Telegram authorization password is required',
		)
		expect(authorize).not.toHaveBeenCalled()
	})
})
