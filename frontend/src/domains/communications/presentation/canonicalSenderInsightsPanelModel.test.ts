import { describe, expect, it } from 'vitest'

import type { SenderInsightV1 } from '../../../gen/makosh/communications/sender_insights/v1/sender_insights_pb'
import { buildCanonicalSenderInsightRows } from './canonicalSenderInsightsPanelModel'

describe('canonical sender-insights presentation model', () => {
	it('renders bounded provider-neutral activity and an opaque fallback', () => {
		const rows = buildCanonicalSenderInsightRows([
			sender(1, 'Ada <ada@example.test>'),
			sender(2),
		])

		expect(rows[0]).toMatchObject({
			displayLabel: 'Ada <ada@example.test>',
			messageCountLabel: '3 messages',
			conversationCountLabel: '2 conversations',
		})
		expect(rows[1]?.displayLabel).toBe('Sender #020202020202')
		expect(JSON.stringify(rows)).not.toMatch(/provider|importance|body/i)
	})
})

function sender(seed: number, displayLabel?: string): SenderInsightV1 {
	return {
		$typeName: 'makosh.communications.sender_insights.v1.SenderInsightV1',
		senderId: new Uint8Array(16).fill(seed),
		displayLabel,
		messageCount: 3n,
		conversationCount: 2n,
		firstObservedAtUnixSeconds: 1_783_024_000n,
		lastObservedAtUnixSeconds: 1_783_024_060n,
	}
}
