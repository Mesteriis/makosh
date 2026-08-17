import { describe, expect, it } from 'vitest'

import {
	initialTelegramHistoryScrollTop,
	shouldPrefetchTelegramHistory,
} from './telegramHistoryViewport'

describe('Telegram history viewport', () => {
	it('keeps fetching short provider pages until the thread becomes scrollable', () => {
		expect(shouldPrefetchTelegramHistory(true, 480, 480)).toBe(true)
		expect(shouldPrefetchTelegramHistory(true, 481, 480)).toBe(true)
		expect(shouldPrefetchTelegramHistory(true, 482, 480)).toBe(false)
	})

	it('stops at the provider history boundary', () => {
		expect(shouldPrefetchTelegramHistory(false, 320, 480)).toBe(false)
	})

	it('opens a newly rendered cached thread at its newest message', () => {
		expect(initialTelegramHistoryScrollTop(102, 20_154)).toBe(20_154)
		expect(initialTelegramHistoryScrollTop(102, 20_154, 9_928)).toBe(9_928)
		expect(initialTelegramHistoryScrollTop(0, 20_154)).toBeUndefined()
	})
})
