import { describe, expect, it, vi } from 'vitest'

import { TelegramMediaMemoryCache } from './telegramMediaMemoryCache'

describe('TelegramMediaMemoryCache', () => {
	it('keeps avatar and message-media budgets independent', () => {
		const revoke = vi.fn()
		const cache = new TelegramMediaMemoryCache({
			avatar: { maxEntries: 2, maxBytes: 10 },
			media: { maxEntries: 1, maxBytes: 20 },
		}, revoke)

		cache.set('avatar', 'a', { url: 'blob:a', sizeBytes: 4 })
		cache.set('avatar', 'b', { url: 'blob:b', sizeBytes: 4 })
		cache.set('media', 'm', { url: 'blob:m', sizeBytes: 20 })
		cache.set('avatar', 'c', { url: 'blob:c', sizeBytes: 4 })

		expect(cache.get('avatar', 'a')).toBeUndefined()
		expect(cache.get('avatar', 'b')?.url).toBe('blob:b')
		expect(cache.get('avatar', 'c')?.url).toBe('blob:c')
		expect(cache.get('media', 'm')?.url).toBe('blob:m')
		expect(revoke).toHaveBeenCalledWith('blob:a')
	})

	it('refreshes LRU order and refuses an oversized artifact without revoking its live URL', () => {
		const revoke = vi.fn()
		const cache = new TelegramMediaMemoryCache({
			avatar: { maxEntries: 2, maxBytes: 10 },
			media: { maxEntries: 1, maxBytes: 5 },
		}, revoke)

		cache.set('avatar', 'a', { url: 'blob:a', sizeBytes: 4 })
		cache.set('avatar', 'b', { url: 'blob:b', sizeBytes: 4 })
		expect(cache.get('avatar', 'a')?.url).toBe('blob:a')
		cache.set('avatar', 'c', { url: 'blob:c', sizeBytes: 4 })
		cache.set('media', 'large', { url: 'blob:large', sizeBytes: 6 })

		expect(cache.get('avatar', 'a')?.url).toBe('blob:a')
		expect(cache.get('avatar', 'b')).toBeUndefined()
		expect(cache.get('media', 'large')).toBeUndefined()
		expect(revoke).toHaveBeenCalledWith('blob:b')
		expect(revoke).not.toHaveBeenCalledWith('blob:large')
	})
})
