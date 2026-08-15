import { describe, expect, it } from 'vitest'

import {
	isTelegramSetupConfigured,
	shouldReplaceTelegramCredentials,
} from './useTelegramAccountSetup'

describe('shouldReplaceTelegramCredentials', () => {
	it('creates the first credentials while required settings are not effective', () => {
		expect(shouldReplaceTelegramCredentials(0n)).toBe(false)
	})

	it('replaces credentials only after Telegram settings became effective', () => {
		expect(shouldReplaceTelegramCredentials(1n)).toBe(true)
	})
})

describe('isTelegramSetupConfigured', () => {
	it('closes the prerequisite form immediately after a successful local setup', () => {
		expect(isTelegramSetupConfigured(0n, true)).toBe(true)
	})

	it('stays configured after the effective Settings projection arrives', () => {
		expect(isTelegramSetupConfigured(2n, false)).toBe(true)
	})
})
