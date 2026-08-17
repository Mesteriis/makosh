import { afterEach, describe, expect, it, vi } from 'vitest'

import { createTelegramMediaVisibilityGate } from './telegramMediaVisibilityGate'

afterEach(() => vi.useRealTimers())

describe('Telegram media visibility gate', () => {
	it('opens only after two uninterrupted visible seconds', () => {
		vi.useFakeTimers()
		const open = vi.fn()
		const gate = createTelegramMediaVisibilityGate(open, 2_000)

		gate.setVisible(true)
		vi.advanceTimersByTime(1_999)
		expect(open).not.toHaveBeenCalled()

		gate.setVisible(false)
		vi.advanceTimersByTime(2_000)
		expect(open).not.toHaveBeenCalled()

		gate.setVisible(true)
		vi.advanceTimersByTime(2_000)
		expect(open).toHaveBeenCalledOnce()
	})

	it('opens immediately on an explicit request and cancels a pending timer', () => {
		vi.useFakeTimers()
		const open = vi.fn()
		const gate = createTelegramMediaVisibilityGate(open, 2_000)

		gate.setVisible(true)
		gate.openNow()
		vi.advanceTimersByTime(2_000)

		expect(open).toHaveBeenCalledOnce()
	})
})
