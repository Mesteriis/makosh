import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getTelegramAuthorizationStatus } from '../api/telegramAuthorizationGateway'
import { openTelegramAuthorizationRealtime } from '../api/telegramAuthorizationRealtime'
import { listTelegramAccounts } from '../api/telegramLifecycleGateway'
import { useTelegramAccountAccess } from './useTelegramAccountAccess'

vi.mock('../api/telegramAuthorizationGateway', () => ({
	getTelegramAuthorizationStatus: vi.fn(),
	submitTelegramAuthorizationPassword: vi.fn(),
}))
vi.mock('../api/telegramAuthorizationRealtime', () => ({
	openTelegramAuthorizationRealtime: vi.fn(),
}))
vi.mock('../api/telegramLifecycleGateway', () => ({
	listTelegramAccounts: vi.fn(),
	provisionTelegramAccount: vi.fn(),
	replayTelegramAccount: vi.fn(),
	restartTelegramAccount: vi.fn(),
	retireTelegramAccount: vi.fn(),
}))
vi.mock('../linking/telegramQrArtifact', () => ({ telegramQrDataUrl: vi.fn() }))

describe('Telegram account access', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(openTelegramAuthorizationRealtime).mockReturnValue({ close: vi.fn() })
	})

	it('keeps a durable operational account when authorization status is transiently unavailable', async () => {
		vi.mocked(listTelegramAccounts).mockResolvedValue([{
			accountId: 'account-1',
			displayName: 'Telegram',
			state: 'ready',
			runtimeState: 'running',
		}] as never)
		vi.mocked(getTelegramAuthorizationStatus).mockRejectedValue(new Error('temporarily unavailable'))
		const access = useTelegramAccountAccess({
			canAuthorize: () => true,
			canManageLifecycle: () => true,
			canReconfigure: () => true,
		})

		await access.refresh()

		expect(access.model.value.accounts).toHaveLength(1)
		expect(access.model.value.selectedAccountId).toBe('account-1')
		expect(access.model.value.selectedAccountOperational).toBe(true)
		expect(access.model.value.authorizationState).toBe('unknown')
		expect(access.model.value.statusMessage).toBe(
			'Telegram account loaded; authorization status is temporarily unavailable.',
		)
	})

	it('tracks authorization transitions from the shared realtime stream', async () => {
		vi.mocked(listTelegramAccounts).mockResolvedValue([{
			accountId: 'account-1',
			state: 'ready',
			runtimeState: 'running',
		}] as never)
		vi.mocked(getTelegramAuthorizationStatus).mockResolvedValue({
			state: 'waiting_qr_scan',
			qrLink: 'tg://private-token',
		})
		let onStatusChanged: ((state: string) => void) | undefined
		vi.mocked(openTelegramAuthorizationRealtime).mockImplementation((callback) => {
			onStatusChanged = callback
			return { close: vi.fn() }
		})
		const access = useTelegramAccountAccess({
			canAuthorize: () => true,
			canManageLifecycle: () => true,
			canReconfigure: () => true,
		})

		await access.refresh()
		expect(access.model.value.authorizationState).toBe('waiting_qr_scan')
		onStatusChanged?.('ready')

		expect(access.model.value.authorizationState).toBe('ready')
		expect(access.model.value.authorizationQrDataUrl).toBe('')
	})
})
