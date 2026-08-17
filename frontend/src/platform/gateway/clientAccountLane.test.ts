import { describe, expect, it, vi } from 'vitest'

import {
	ClientAccountLane,
	ClientAccountLaneOverflowError,
	ClientAccountLaneRegistry,
} from './clientAccountLane'

describe('client account lanes', () => {
	it('does not let stalled media block same-account interactive or realtime work', async () => {
		const lane = new ClientAccountLane({ provider: 'telegram', accountId: 'account-a' })
		const media = deferred<void>()
		const mediaWork = lane.run('media', async () => media.promise)
		const interactive = lane.run('interactive', async () => 'history')
		const realtime = lane.run('realtime', async () => 'event')

		await expect(interactive).resolves.toBe('history')
		await expect(realtime).resolves.toBe('event')
		media.resolve()
		await mediaWork
	})

	it('isolates equal work classes between provider accounts', async () => {
		const registry = new ClientAccountLaneRegistry()
		const stalled = deferred<void>()
		const accountA = registry.get({ provider: 'mail', accountId: 'account-a' })
		const accountB = registry.get({ provider: 'mail', accountId: 'account-b' })
		const first = accountA.run('interactive', async () => stalled.promise)

		await expect(accountB.run('interactive', async () => 'account-b')).resolves.toBe('account-b')
		stalled.resolve()
		await first
	})

	it('coalesces repeated projection invalidations to the latest revision', async () => {
		const lane = new ClientAccountLane({ provider: 'whatsapp', accountId: 'account-a' })
		const first = deferred<void>()
		const applied: bigint[] = []
		const refresh = vi.fn(async (revision: bigint) => {
			applied.push(revision)
			if (revision === 1n) await first.promise
		})

		lane.invalidate(1n, refresh)
		await Promise.resolve()
		lane.invalidate(2n, refresh)
		lane.invalidate(3n, refresh)
		first.resolve()
		await vi.waitFor(() => expect(applied).toEqual([1n, 3n]))
	})

	it('bounds interactive work without silently dropping an accepted operation', async () => {
		const lane = new ClientAccountLane(
			{ provider: 'mail', accountId: 'account-a' },
			{ maxPendingPerWorkClass: { interactive: 1 } },
		)
		const active = deferred<void>()
		const first = lane.run('interactive', async () => active.promise)
		const accepted = lane.run('interactive', async () => 'accepted')

		const overflow = await lane.run('interactive', async () => 'overflow').catch(error => error)
		expect(overflow).toBeInstanceOf(ClientAccountLaneOverflowError)
		expect(overflow).toMatchObject({ workClass: 'interactive' })
		active.resolve()
		await first
		await expect(accepted).resolves.toBe('accepted')
	})

	it('replaces stale queued media work while keeping the active materialization isolated', async () => {
		const lane = new ClientAccountLane(
			{ provider: 'zulip', accountId: 'account-a' },
			{ maxPendingPerWorkClass: { media: 1 } },
		)
		const active = deferred<void>()
		const first = lane.run('media', async () => active.promise)
		const stale = lane.run('media', async () => 'stale')
		const latest = lane.run('media', async () => 'latest')

		await expect(stale).rejects.toBeInstanceOf(ClientAccountLaneOverflowError)
		active.resolve()
		await first
		await expect(latest).resolves.toBe('latest')
	})

	it('keeps equal account ids isolated between different providers', async () => {
		const registry = new ClientAccountLaneRegistry()
		expect(registry.get({ provider: 'telegram', accountId: 'personal' }))
			.not.toBe(registry.get({ provider: 'whatsapp', accountId: 'personal' }))
	})

	it('emits payload-safe queue spans without account identity', async () => {
		const measurements: unknown[] = []
		const ticks = [10, 15, 23]
		const lane = new ClientAccountLane(
			{ provider: 'mail', accountId: 'private-account-id' },
			{
				now: () => ticks.shift() ?? 23,
				onMeasurement: measurement => measurements.push(measurement),
			},
		)

		await lane.run('interactive', async () => 'done')
		await vi.waitFor(() => expect(measurements).toHaveLength(1))

		expect(measurements).toEqual([{
			laneKind: 'provider_account',
			provider: 'mail',
			workClass: 'interactive',
			queueDepthAtEnqueue: 0,
			queueWaitMillis: 5,
			executionMillis: 8,
			outcome: 'completed',
		}])
		expect(JSON.stringify(measurements)).not.toContain('private-account-id')
	})

	it('coalesces replay-gap recovery and exposes stale to live lifecycle', async () => {
		const states: string[] = []
		const lane = new ClientAccountLane(
			{ provider: 'telegram', accountId: 'account-a' },
			{ onLifecycleChange: state => states.push(state) },
		)
		const active = deferred<void>()
		const activeWork = lane.run('realtime', async () => active.promise)
		const recovered: string[] = []

		lane.recover(async () => { recovered.push('stale') })
		lane.recover(async () => { recovered.push('latest') })
		expect(lane.state()).toBe('stale')
		active.resolve()
		await activeWork
		await vi.waitFor(() => expect(recovered).toEqual(['latest']))
		await vi.waitFor(() => expect(lane.state()).toBe('live'))

		expect(states).toEqual(['stale', 'recovering', 'live'])
	})
})

function deferred<T>(): { promise: Promise<T>; resolve(value: T): void } {
	let resolvePromise: (value: T) => void = () => undefined
	const promise = new Promise<T>((resolve) => { resolvePromise = resolve })
	return { promise, resolve: resolvePromise }
}
