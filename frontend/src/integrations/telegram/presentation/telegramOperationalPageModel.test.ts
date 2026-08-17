import { describe, expect, it } from 'vitest'

import {
	buildTelegramChatRows,
	buildTelegramMessageRows,
} from './telegramOperationalPageModel'

describe('Telegram operational presentation model', () => {
	it('maps provider projections without leaking transport state to the component', () => {
		expect(buildTelegramChatRows([{
			providerChatId: 'chat-1',
			title: 'Architecture',
			username: 'makosh_arch',
			kind: 'supergroup',
		} as never], 'chat-1')).toEqual([{
		id: 'chat-1',
		title: 'Architecture',
		detail: '@makosh_arch · supergroup',
		selected: true,
		avatarProviderFileId: '',
		}])

		expect(buildTelegramMessageRows([{
			messageId: 'message-1',
			senderDisplayName: 'Alex',
			text: 'Boundary approved',
			observedAtUnixSeconds: 1_753_520_400n,
			deliveryState: 'received',
		} as never])[0]).toMatchObject({
			id: 'message-1',
			providerMessageId: 'message-1',
			sender: 'Alex',
			body: 'Boundary approved',
			outgoing: false,
			selected: false,
		})

		const mediaMessage = buildTelegramMessageRows([{
			messageId: 'message-video',
			providerMessageId: 'provider-video',
			text: 'Video',
			observedAtUnixSeconds: 1_753_520_401n,
			deliveryState: 'received',
			media: {
				kind: 'video',
				providerFileId: 'full-file',
				previewProviderFileId: 'preview-file',
				previewInlineData: new Uint8Array([0xff, 0xd8]),
				contentType: 'video/mp4',
				previewContentType: 'image/jpeg',
			},
		} as never])[0]
		expect(mediaMessage?.body).toBe('')
		expect(mediaMessage?.media?.filename).toBe('video')
		expect(mediaMessage?.media).toMatchObject({
			providerFileId: 'full-file',
			previewProviderFileId: 'preview-file',
			previewInlineData: new Uint8Array([0xff, 0xd8]),
			previewContentType: 'image/jpeg',
			renderKind: 'video',
		})

		expect(buildTelegramMessageRows([{
			messageId: 'message-captioned-video',
			text: 'Release demo',
			observedAtUnixSeconds: 1_753_520_402n,
			deliveryState: 'received',
			media: { kind: 'video', caption: 'Release demo' },
		} as never])[0]?.body).toBe('Release demo')
		expect(buildTelegramMessageRows([{
			messageId: 'message-placeholder',
			observedAtUnixSeconds: 1_753_520_403n,
			deliveryState: 'received',
			media: {
				kind: 'video',
				caption: '[video]',
				filename: '[photo]',
			},
		} as never])[0]).toMatchObject({
			body: '',
			media: {
				filename: 'video',
			},
		})
	})

	it('never exposes provider sender ids and renders audio and documents as interactive media', () => {
		const rows = buildTelegramMessageRows([{
			messageId: 'voice',
			senderId: '4815162342',
			observedAtUnixSeconds: 1n,
			deliveryState: 'received',
			media: {
				kind: 'voiceNote',
				providerFileId: 'voice-file',
				contentType: 'audio/ogg',
			},
		}, {
			messageId: 'document',
			senderId: '108',
			observedAtUnixSeconds: 2n,
			deliveryState: 'received',
			media: {
				kind: 'document',
				providerFileId: 'document-file',
				contentType: 'application/pdf',
			},
		}] as never)

		expect(rows[0]?.sender).toBe('Telegram user')
		expect(rows[0]?.media?.renderKind).toBe('audio')
		expect(rows[1]?.sender).toBe('Telegram user')
		expect(rows[1]?.media?.renderKind).toBe('file')
	})

	it('does not expose a provider chat id when Telegram has no title', () => {
		expect(buildTelegramChatRows([{
			providerChatId: '-1001234567890',
			title: '',
			username: '',
			kind: 'private',
		} as never], '')[0]).toMatchObject({
			title: 'Untitled Telegram chat',
			detail: 'private',
		})
	})

	it('renders Telegram animations and TGS stickers without a generic video control', () => {
		const rows = buildTelegramMessageRows([{
			messageId: 'animation',
			observedAtUnixSeconds: 1n,
			deliveryState: 'received',
			media: { kind: 'animation', contentType: 'video/webm' },
		}, {
			messageId: 'tgs-sticker',
			observedAtUnixSeconds: 2n,
			deliveryState: 'received',
			media: { kind: 'animation', contentType: 'application/x-tgsticker' },
		}] as never)

		expect(rows[0]?.media?.renderKind).toBe('animation')
		expect(rows[1]?.media?.renderKind).toBe('tgs')
	})

	it('prefers an exactly linked Persona name and never fuzzy-matches another source', () => {
		const id = (value: number) => new Uint8Array(16).fill(value)
		const sourceIdentity = {
			integrationPublicId: id(1),
			accountPublicId: id(2),
			providerSourceContactPublicId: id(3),
		}
		const exactKey = [id(1), id(2), id(3)]
			.map(value => Array.from(value, byte => byte.toString(16).padStart(2, '0')).join(''))
			.join(':')
		const message = {
			messageId: 'message-persona',
			senderDisplayName: 'Telegram Name',
			senderSourceIdentity: sourceIdentity,
			observedAtUnixSeconds: 1n,
			deliveryState: 'received',
		} as never

		expect(buildTelegramMessageRows([message], '', new Map([[exactKey, 'Persona Name']]))[0]?.sender)
			.toBe('Persona Name')
		expect(buildTelegramMessageRows([message], '', new Map([['wrong-source', 'Wrong Person']]))[0]?.sender)
			.toBe('Telegram Name')
	})

	it('uses the provider participant directory and never labels an outgoing message as the private peer', () => {
		const incoming = {
			messageId: 'message-incoming',
			senderId: '7',
			senderDisplayName: 'Telegram user',
			observedAtUnixSeconds: 1n,
			deliveryState: 'received',
		} as never
		const outgoing = {
			messageId: 'message-outgoing',
			senderDisplayName: 'Telegram user',
			observedAtUnixSeconds: 2n,
			deliveryState: 'sent',
		} as never

		expect(buildTelegramMessageRows(
			[incoming],
			'',
			new Map(),
			new Map([['7', 'Provider nickname']]),
			'Private peer',
		)[0]?.sender).toBe('Provider nickname')
		expect(buildTelegramMessageRows([incoming], '', new Map(), new Map(), 'Private peer')[0]?.sender)
			.toBe('Private peer')
		expect(buildTelegramMessageRows([outgoing], '', new Map(), new Map(), 'Private peer')[0]?.sender)
			.toBe('You')
	})
})
