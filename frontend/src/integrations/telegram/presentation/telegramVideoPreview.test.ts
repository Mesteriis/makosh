import { describe, expect, it } from 'vitest'

import {
	fallbackVideoPreviewTime,
	inlineVideoPreviewDataUrl,
	shouldRewindFallbackPreview,
} from './telegramVideoPreview'

describe('Telegram video preview fallback', () => {
	it('renders an inline TDLib JPEG without requesting the full video', () => {
		expect(inlineVideoPreviewDataUrl(new Uint8Array())).toBe('')
		expect(inlineVideoPreviewDataUrl(new Uint8Array([0xff, 0xd8, 0xff, 0xd9])))
			.toBe('data:image/jpeg;base64,/9j/2Q==')
	})

	it('chooses a bounded early frame only for playable video', () => {
		expect(fallbackVideoPreviewTime(Number.NaN)).toBe(0)
		expect(fallbackVideoPreviewTime(0)).toBe(0)
		expect(fallbackVideoPreviewTime(0.1)).toBe(0.01)
		expect(fallbackVideoPreviewTime(2)).toBe(0.1)
		expect(fallbackVideoPreviewTime(120)).toBe(0.25)
	})

	it('rewinds only an untouched generated preview before playback', () => {
		expect(shouldRewindFallbackPreview(0.25, 0.25)).toBe(true)
		expect(shouldRewindFallbackPreview(0.2, 0.25)).toBe(true)
		expect(shouldRewindFallbackPreview(4, 0.25)).toBe(false)
		expect(shouldRewindFallbackPreview(0, 0)).toBe(false)
	})
})
