import { describe, expect, it } from 'vitest'

import { buildTelegramMessageInspectionView } from './telegramMessageInspectorModel'

describe('Telegram message inspector presentation model', () => {
	it('maps versions, lineage, mutations and command audit into bounded rows', () => {
		const model = buildTelegramMessageInspectionView({
			message: { messageId: 'message-1' } as never,
			versions: [{
				versionId: 'version-1',
				versionNumber: 2,
				source: 'provider_edit',
				bodyText: 'Updated',
			} as never],
			tombstones: [],
			mutations: [{
				mutation: { case: 'pin', value: { isPinned: true } },
			} as never],
			references: {
				replyTo: { providerMessageId: 'previous-1' },
			} as never,
			replyChain: [],
			forwardChain: [],
			attachment: { state: 'safe' } as never,
			file: undefined,
			reactions: [],
			reactionSummary: [{ emoji: '👍', count: 2, isActive: true } as never],
			commands: [{
				operation: {
					operationId: 'operation-1',
					commandKind: 'pin',
					state: 'completed',
				},
			} as never],
			pinned: true,
		})

		expect(model.overview).toEqual([
			'pinned',
			'has reply reference',
			'attachment safe',
		])
		expect(model.versions[0]).toMatchObject({ title: 'Version 2 · provider_edit' })
		expect(model.mutations[0]).toMatchObject({ title: 'Pinned' })
		expect(model.reactions[0]).toMatchObject({ title: '👍 · 2' })
		expect(model.commands[0]).toMatchObject({ id: 'operation-1', detail: 'completed' })
	})

	it('describes media without square-bracket transport markers', () => {
		const model = buildTelegramMessageInspectionView({
			message: { messageId: 'message-1' } as never,
			versions: [],
			tombstones: [],
			mutations: [],
			references: {} as never,
			replyChain: [{
				messageId: 'reply-1',
				text: 'Video',
				media: { kind: 'video', filename: '', caption: '' },
			} as never],
			forwardChain: [],
			reactions: [],
			reactionSummary: [],
			commands: [],
			pinned: false,
		} as never)

		expect(model.replyChain[0]?.detail).toBe('Video')

		const modelFromForward = buildTelegramMessageInspectionView({
			message: { messageId: 'message-1' } as never,
			versions: [],
			tombstones: [],
			mutations: [],
			references: {},
			replyChain: [],
			forwardChain: [{
				messageId: 'forward-1',
				text: undefined,
				media: { kind: 'photo', filename: '[photo]', caption: '[video]' },
			} as never],
			reactions: [],
			reactionSummary: [],
			commands: [],
			pinned: false,
		} as never)

		expect(modelFromForward.forwardChain[0]?.detail).toBe('Photo')
	})
})
