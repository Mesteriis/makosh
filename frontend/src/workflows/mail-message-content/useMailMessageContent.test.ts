import { describe, expect, it, vi } from 'vitest'

import { useMailMessageContent } from './useMailMessageContent'

describe('useMailMessageContent', () => {
	it('resolves provider evidence through Communications and renders exact UTF-8 body text', async () => {
		const evidenceId = new Uint8Array(16).fill(1)
		const messageId = new Uint8Array(16).fill(2)
		const resolveMessageId = vi.fn(async () => messageId)
		const readContent = vi.fn(async () => ({
			bytes: new TextEncoder().encode('Decoded message body'),
			mediaType: 'text/plain' as const,
		}))
		const content = useMailMessageContent(
			{ canRead: () => true },
			{ resolveMessageId, readContent },
		)

		await content.open(evidenceId)

		expect(resolveMessageId).toHaveBeenCalledWith(evidenceId)
		expect(readContent).toHaveBeenCalledWith(messageId, expect.any(AbortSignal))
		expect(content.model.value).toEqual({
			status: 'ready',
			statusMessage: '',
			bodyText: 'Decoded message body',
			bodyFormat: 'text',
		})
	})

	it('preserves the admitted HTML media type for isolated presentation', async () => {
		const content = useMailMessageContent(
			{ canRead: () => true },
			{
				resolveMessageId: async () => new Uint8Array(16).fill(2),
				readContent: async () => ({
					bytes: new TextEncoder().encode('<p><strong>HTML body</strong></p>'),
					mediaType: 'text/html',
				}),
			},
		)

		await content.open(new Uint8Array(16).fill(1))

		expect(content.model.value.bodyFormat).toBe('html')
		expect(content.model.value.bodyText).toContain('<strong>HTML body</strong>')
	})

	it('does not cross owner boundaries without admitted content capability', async () => {
		const resolveMessageId = vi.fn()
		const readContent = vi.fn()
		const content = useMailMessageContent(
			{ canRead: () => false },
			{ resolveMessageId, readContent },
		)

		await content.open(new Uint8Array(16).fill(1))

		expect(resolveMessageId).not.toHaveBeenCalled()
		expect(readContent).not.toHaveBeenCalled()
		expect(content.model.value.status).toBe('unavailable')
	})
})
