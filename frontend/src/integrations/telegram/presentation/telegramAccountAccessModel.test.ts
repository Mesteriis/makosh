import { describe, expect, it } from 'vitest'

import {
	authorizationView,
	buildTelegramAccountRows,
	canStartTelegramOperationalLane,
	isTelegramAccountOperational,
} from './telegramAccountAccessModel'

describe('Telegram account access presentation model', () => {
	it('maps lifecycle projections without transport objects', () => {
		expect(buildTelegramAccountRows([{
			accountId: 'account-1',
			displayName: 'Personal',
			state: 'active',
			runtimeState: 'ready',
		} as never], 'account-1', 'waiting_qr_scan')).toEqual([{
			id: 'account-1',
			title: 'Personal',
			detail: 'active · ready · authorization: waiting_qr_scan',
			selected: true,
		}])
	})

	it('does not present lifecycle readiness as completed provider authorization', () => {
		const rows = buildTelegramAccountRows([{
			accountId: 'account-1',
			displayName: 'Personal',
			state: 'ready',
			runtimeState: 'running',
		} as never], 'account-1', 'waiting_password')

		expect(rows[0]?.detail).toContain('authorization: waiting_password')
	})

	it('recognizes a persisted ready account without conflating it with authorization status', () => {
		const accounts = [{
			accountId: 'account-1',
			state: 'ready',
			runtimeState: 'running',
		}] as never

		expect(isTelegramAccountOperational(accounts, 'account-1')).toBe(true)
		expect(isTelegramAccountOperational(accounts, 'account-2')).toBe(false)
	})

	it('does not admit the operational lane from stale lifecycle readiness', () => {
		expect(canStartTelegramOperationalLane({
			selectedAccountId: 'account-1',
			selectedAccountOperational: true,
			authorizationState: 'waiting_qr_scan',
		})).toBe(false)
		expect(canStartTelegramOperationalLane({
			selectedAccountId: 'account-1',
			selectedAccountOperational: true,
			authorizationState: 'ready',
		})).toBe(true)
	})

	it('keeps authorization secrets out of the model mapper', () => {
		expect(authorizationView({
			state: 'waiting_password',
			passwordHint: 'two words',
		})).toEqual({
			state: 'waiting_password',
			passwordHint: 'two words',
			qrLink: '',
		})
		expect(authorizationView(null).state).toBe('unknown')
	})
})
