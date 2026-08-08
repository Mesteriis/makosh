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
	})
})
