import {
	getClientAccountLaneRegistry,
	type ClientAccountWorkClass,
} from '../../../platform/gateway/clientAccountLane'

export type TelegramRuntimeRequestPriority = ClientAccountWorkClass | 'background'

/**
 * Routes Telegram requests into provider/account-local work classes. The Core
 * Gateway remains the single physical transport; slow enrichment/media work
 * no longer owns the browser critical path for interactive or realtime work.
 */
export function withTelegramRuntimeRequestQueue<T>(
	operation: () => Promise<T>,
	priority: TelegramRuntimeRequestPriority = 'interactive',
	accountId = 'configuration',
): Promise<T> {
	const lane = getClientAccountLaneRegistry().get({
		provider: 'telegram',
		accountId: accountId.trim() || 'configuration',
	})
	return lane.run(workClass(priority), async () => operation())
}

function workClass(priority: TelegramRuntimeRequestPriority): ClientAccountWorkClass {
	return priority === 'background' ? 'enrichment' : priority
}
