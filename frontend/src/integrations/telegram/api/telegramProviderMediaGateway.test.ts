import { describe, expect, it } from 'vitest'

import {
	shouldStopTelegramMediaPolling,
	telegramMediaDeliveryForKind,
	telegramMediaDownloadOperationId,
	telegramMediaPollAttemptLimit,
} from './telegramProviderMediaGateway'

describe('telegram provider media delivery', () => {
	it('uses range delivery from the typed message kind even without a usable MIME', () => {
		expect(telegramMediaDeliveryForKind('video')).toBe('range')
		expect(telegramMediaDeliveryForKind('animation')).toBe('range')
		expect(telegramMediaDeliveryForKind('audio')).toBe('range')
		expect(telegramMediaDeliveryForKind('image')).toBe('inline')
		expect(telegramMediaDeliveryForKind('file')).toBe('inline')
	})

	it('keeps an admitted large file alive while bounding stalled provider work', () => {
		expect(telegramMediaPollAttemptLimit('interactive')).toBe(1_200)
		expect(telegramMediaPollAttemptLimit('background')).toBe(8)
		expect(shouldStopTelegramMediaPolling({ isDownloaded: true, isDownloading: false }, 1_200))
			.toBe(false)
		expect(shouldStopTelegramMediaPolling({ isDownloaded: false, isDownloading: true }, 119))
			.toBe(false)
		expect(shouldStopTelegramMediaPolling({ isDownloaded: false, isDownloading: true }, 120))
			.toBe(true)
		expect(shouldStopTelegramMediaPolling(undefined, 60)).toBe(true)
	})

	it('deduplicates provider downloads without persisting provider identifiers', async () => {
		const first = await telegramMediaDownloadOperationId('account-a', 'file-a')
		const repeated = await telegramMediaDownloadOperationId('account-a', 'file-a')
		const another = await telegramMediaDownloadOperationId('account-a', 'file-b')

		expect(first).toBe(repeated)
		expect(first).not.toBe(another)
		expect(first).toMatch(/^telegram-media-download-[a-f0-9]{64}$/)
		expect(first).not.toContain('account-a')
		expect(first).not.toContain('file-a')
	})
})
