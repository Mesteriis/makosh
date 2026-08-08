import { describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'

import type { SenderInsightV1 } from '../../../gen/makosh/communications/sender_insights/v1/sender_insights_pb'
import { useCanonicalCommunicationsSenderInsights } from './useCanonicalCommunicationsSenderInsights'

describe('canonical sender-insights lifecycle', () => {
	it('is inert without the exact admitted capability', async () => {
		const operations = operationsFixture()
		const controller = useCanonicalCommunicationsSenderInsights(
			() => false,
			() => undefined,
			operations,
		)

		await controller.load()

		expect(controller.model.value.status).toBe('unavailable')
		expect(operations.list).not.toHaveBeenCalled()
	})

	it('fences stale responses and reloads with the selected canonical account', async () => {
		const pending: Array<(value: {
			items: readonly SenderInsightV1[]
			nextCursor: Uint8Array<ArrayBuffer>
		}) => void> = []
		const operations = operationsFixture()
		operations.list.mockImplementation(() => new Promise((resolve) => pending.push(resolve)))
		const accountId = new Uint8Array(16).fill(7)
		const controller = useCanonicalCommunicationsSenderInsights(
			() => true,
			() => accountId,
			operations,
		)

		const first = controller.load()
		const second = controller.load()
		pending[0]?.({ items: [sender(1, 'stale')], nextCursor: new Uint8Array() })
		pending[1]?.({ items: [sender(2, 'current')], nextCursor: new Uint8Array() })
		await Promise.all([first, second])
		expect(controller.model.value.items[0]?.displayLabel).toBe('current')

		controller.updateScopeCurrentAccount(true)
		pending[2]?.({ items: [sender(3, 'scoped')], nextCursor: new Uint8Array() })
		await vi.waitFor(() => expect(controller.model.value.status).toBe('ready'))
		expect(operations.list).toHaveBeenLastCalledWith(accountId)
		expect(controller.model.value.items[0]?.displayLabel).toBe('scoped')
	})

	it('falls back to the owner-wide projection when the selected account disappears', async () => {
		const operations = operationsFixture()
		const accountId = ref<Uint8Array | undefined>(new Uint8Array(16).fill(7))
		const controller = useCanonicalCommunicationsSenderInsights(
			() => true,
			() => accountId.value,
			operations,
		)

		await controller.load()
		controller.updateScopeCurrentAccount(true)
		await vi.waitFor(() => expect(controller.model.value.status).toBe('ready'))
		accountId.value = undefined
		await nextTick()
		await vi.waitFor(() => expect(controller.model.value.scopeCurrentAccount).toBe(false))

		expect(operations.list).toHaveBeenLastCalledWith(undefined)
	})
})

function operationsFixture() {
	return {
		list: vi.fn(async () => ({
			items: [] as readonly SenderInsightV1[],
			nextCursor: new Uint8Array(),
		})),
	}
}

function sender(seed: number, displayLabel: string): SenderInsightV1 {
	return {
		$typeName: 'makosh.communications.sender_insights.v1.SenderInsightV1',
		senderId: new Uint8Array(16).fill(seed),
		displayLabel,
		messageCount: 1n,
		conversationCount: 1n,
		firstObservedAtUnixSeconds: 1n,
		lastObservedAtUnixSeconds: 1n,
	}
}
