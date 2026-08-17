import { describe, expect, it, vi } from 'vitest'

import { TelegramMediaLaneScheduler } from './telegramMediaLaneScheduler'

function deferred<T>() {
	let resolve!: (value: T) => void
	const promise = new Promise<T>((next) => { resolve = next })
	return { promise, resolve }
}

describe('TelegramMediaLaneScheduler', () => {
	it('does not let one blocked account stop another account lane', async () => {
		const scheduler = new TelegramMediaLaneScheduler(4, 1)
		const first = deferred<string>()
		const runA1 = vi.fn(() => first.promise)
		const runA2 = vi.fn(async () => 'a2')
		const runB1 = vi.fn(async () => 'b1')
		const active = () => true

		const a1 = scheduler.schedule({ laneKey: 'a', scopeKey: 'a:chat', priority: 'interactive', isScopeActive: active, run: runA1 })
		const a2 = scheduler.schedule({ laneKey: 'a', scopeKey: 'a:chat', priority: 'interactive', isScopeActive: active, run: runA2 })
		const b1 = scheduler.schedule({ laneKey: 'b', scopeKey: 'b:chat', priority: 'interactive', isScopeActive: active, run: runB1 })

		await expect(b1).resolves.toBe('b1')
		expect(runA1).toHaveBeenCalledOnce()
		expect(runA2).not.toHaveBeenCalled()
		expect(runB1).toHaveBeenCalledOnce()
		first.resolve('a1')
		await expect(a1).resolves.toBe('a1')
		await expect(a2).resolves.toBe('a2')
	})

	it('rejects queued work whose visible scope was replaced', async () => {
		const scheduler = new TelegramMediaLaneScheduler(1, 1)
		const first = deferred<string>()
		let activeScope = 'old'
		const active = (scopeKey: string) => scopeKey === activeScope

		const running = scheduler.schedule({ laneKey: 'a', scopeKey: 'old', priority: 'interactive', isScopeActive: active, run: () => first.promise })
		const stale = scheduler.schedule({ laneKey: 'a', scopeKey: 'old', priority: 'background', isScopeActive: active, run: async () => 'stale' })
		activeScope = 'new'
		first.resolve('done')

		await expect(running).resolves.toBe('done')
		await expect(stale).rejects.toMatchObject({ name: 'AbortError' })
	})
})
