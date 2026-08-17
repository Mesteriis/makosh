import { describe, expect, it } from 'vitest'

import { withTelegramRuntimeRequestQueue } from './telegramRuntimeRequestQueue'

describe('Telegram runtime request queue', () => {
	it('keeps one account work class bounded to one active request', async () => {
		let active = 0
		let maximumActive = 0
		let releaseFirst!: () => void
		const firstGate = new Promise<void>((resolve) => {
			releaseFirst = resolve
		})

		const first = withTelegramRuntimeRequestQueue(async () => {
			active += 1
			maximumActive = Math.max(maximumActive, active)
			await firstGate
			active -= 1
			return 'first'
		}, 'interactive', 'account-a')
		const second = withTelegramRuntimeRequestQueue(async () => {
			active += 1
			maximumActive = Math.max(maximumActive, active)
			active -= 1
			return 'second'
		}, 'interactive', 'account-a')

		await Promise.resolve()
		expect(active).toBe(1)
		releaseFirst()
		await expect(Promise.all([first, second])).resolves.toEqual(['first', 'second'])
		expect(maximumActive).toBe(1)
	})

	it('does not let stalled media block an interactive chat request', async () => {
		const order: string[] = []
		let releaseActive!: () => void
		const activeGate = new Promise<void>((resolve) => {
			releaseActive = resolve
		})
		const active = withTelegramRuntimeRequestQueue(async () => {
			order.push('active-media')
			await activeGate
		}, 'media', 'account-a')
		const queuedMedia = withTelegramRuntimeRequestQueue(async () => {
			order.push('queued-media')
		}, 'media', 'account-a')
		const chat = withTelegramRuntimeRequestQueue(async () => {
			order.push('chat')
		}, 'interactive', 'account-a')

		await expect(chat).resolves.toBeUndefined()
		expect(order).toEqual(['active-media', 'chat'])
		releaseActive()
		await Promise.all([active, queuedMedia])

		expect(order).toEqual(['active-media', 'chat', 'queued-media'])
	})

	it('does not serialize requests from different accounts', async () => {
		let releaseFirst!: () => void
		const firstGate = new Promise<void>((resolve) => { releaseFirst = resolve })
		const first = withTelegramRuntimeRequestQueue(
			() => firstGate,
			'interactive',
			'account-a',
		)

		await expect(withTelegramRuntimeRequestQueue(
			async () => 'account-b',
			'interactive',
			'account-b',
		)).resolves.toBe('account-b')
		releaseFirst()
		await first
	})
})
