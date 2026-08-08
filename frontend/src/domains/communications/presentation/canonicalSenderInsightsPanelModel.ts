import type { SenderInsightV1 } from '../../../gen/makosh/communications/sender_insights/v1/sender_insights_pb'
import { bytesKey } from './canonicalCommunicationsPageModel'

export type CanonicalSenderInsightsPanelStatus =
	| 'unavailable'
	| 'loading'
	| 'ready'
	| 'error'

export type CanonicalSenderInsightRow = {
	key: string
	displayLabel: string
	referenceLabel: string
	messageCountLabel: string
	conversationCountLabel: string
	observedRangeLabel: string
}

export type CanonicalSenderInsightsPanelModel = {
	status: CanonicalSenderInsightsPanelStatus
	statusMessage: string
	scopeCurrentAccount: boolean
	canScopeToCurrentAccount: boolean
	items: readonly CanonicalSenderInsightRow[]
	hasMore: boolean
	busy: boolean
}

export function buildCanonicalSenderInsightRows(
	items: readonly SenderInsightV1[],
): readonly CanonicalSenderInsightRow[] {
	return items.map((item) => {
		const key = bytesKey(item.senderId)
		return {
			key,
			displayLabel: item.displayLabel?.trim() || `Sender #${key.slice(0, 12)}`,
			referenceLabel: `Canonical sender #${key.slice(0, 12)}`,
			messageCountLabel: countLabel(item.messageCount, 'message'),
			conversationCountLabel: countLabel(item.conversationCount, 'conversation'),
			observedRangeLabel: observedRange(
				item.firstObservedAtUnixSeconds,
				item.lastObservedAtUnixSeconds,
			),
		}
	})
}

function countLabel(value: bigint, singular: string): string {
	const suffix = value === 1n ? singular : `${singular}s`
	return `${new Intl.NumberFormat().format(value)} ${suffix}`
}

function observedRange(first: bigint, last: bigint): string {
	const firstLabel = formatUnixSeconds(first)
	const lastLabel = formatUnixSeconds(last)
	return first === last ? `Observed ${lastLabel}` : `${firstLabel} — ${lastLabel}`
}

function formatUnixSeconds(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds)) return 'Time unavailable'
	const date = new Date(milliseconds)
	if (Number.isNaN(date.getTime())) return 'Time unavailable'
	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(date)
}
