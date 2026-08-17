import { create } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientRealtimeEventV1Schema,
	ClientRealtimeStreamStateV1Schema,
	ClientReplayGapV1Schema,
} from '../../gen/makosh/gateway/v1/client_realtime_pb'
import type {
	BrowserGatewayRealtimeObserver,
	BrowserGatewayRealtimeSubscription,
} from './browserGatewayRealtime'
import {
	BrowserGatewayRealtimeHub,
	getBrowserGatewayRealtimeHubByAccount,
	resetBrowserGatewayRealtimeHubForTests,
} from './browserGatewayRealtimeHub'

describe('BrowserGatewayRealtimeHub', () => {
	it('routes every valid account identity through one physical realtime hub', () => {
		resetBrowserGatewayRealtimeHubForTests()
		const telegramA = getBrowserGatewayRealtimeHubByAccount({
			provider: 'telegram',
			accountId: 'acc-a',
		})
		const telegramAAgain = getBrowserGatewayRealtimeHubByAccount({
			provider: 'telegram',
			accountId: 'acc-a',
		})
		const telegramB = getBrowserGatewayRealtimeHubByAccount({
			provider: 'telegram',
			accountId: 'acc-b',
		})
		const zulipA = getBrowserGatewayRealtimeHubByAccount({
			provider: 'zulip',
			accountId: 'acc-a',
		})

		expect(telegramA).toBe(telegramAAgain)
		expect(telegramA).toBe(telegramB)
		expect(telegramA).toBe(zulipA)
	})

	it('throws when account identity is malformed', () => {
		resetBrowserGatewayRealtimeHubForTests()
		expect(() =>
			getBrowserGatewayRealtimeHubByAccount({
				provider: 'telegram',
				accountId: ' ',
			}),
		).toThrow('browser_realtime_hub_identity_invalid')
	})

	it('shares one browser EventSource subscription across independent consumers', () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const closeSource = vi.fn()
		const subscribe = vi.fn((observer: BrowserGatewayRealtimeObserver): BrowserGatewayRealtimeSubscription => {
			sourceObserver = observer
			return { close: closeSource }
		})
		const hub = new BrowserGatewayRealtimeHub({ subscribe })
		const first = observerFixture()
		const second = observerFixture()
		const firstSubscription = hub.subscribe(first)
		const secondSubscription = hub.subscribe(second)

		expect(subscribe).toHaveBeenCalledTimes(1)
		const event = create(ClientRealtimeEventV1Schema, { cursor: 'cursor-1' })
		sourceObserver?.onEvent(event)
		expect(first.onEvent).toHaveBeenCalledWith(event)
		expect(second.onEvent).toHaveBeenCalledWith(event)

		firstSubscription.close()
		expect(closeSource).not.toHaveBeenCalled()
		secondSubscription.close()
		expect(closeSource).toHaveBeenCalledTimes(1)
	})

	it('isolates a failing consumer without starving later account lanes', () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const hub = new BrowserGatewayRealtimeHub({
			subscribe(observer) {
				sourceObserver = observer
				return { close: vi.fn() }
			},
		})
		const failedOnEvent = vi.fn(() => { throw new Error('account_observer_failed') })
		const failed = { ...observerFixture(), onEvent: failedOnEvent }
		const healthy = observerFixture()
		hub.subscribe(failed)
		hub.subscribe(healthy)
		const event = create(ClientRealtimeEventV1Schema, { cursor: 'cursor-isolated' })
		const errorLog = vi.spyOn(console, 'error').mockImplementation(() => undefined)

		expect(() => sourceObserver?.onEvent(event)).not.toThrow()
		expect(failedOnEvent).toHaveBeenCalledWith(event)
		expect(healthy.onEvent).toHaveBeenCalledWith(event)
		expect(errorLog).toHaveBeenCalledWith(
			'browser_gateway_realtime_observer_delivery_failed',
			{ signalKind: 'event' },
		)
		errorLog.mockRestore()
	})

	it('broadcasts stream state and fail-closed replay signals', () => {
		vi.useFakeTimers()
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const closeSource = vi.fn()
		const subscribe = vi.fn((observer: BrowserGatewayRealtimeObserver) => {
			sourceObserver = observer
			return { close: closeSource }
		})
		const hub = new BrowserGatewayRealtimeHub({ subscribe })
		const observer = observerFixture()
		hub.subscribe(observer)

		const state = create(ClientRealtimeStreamStateV1Schema, { cursor: 'cursor-2' })
		const gap = create(ClientReplayGapV1Schema, { reasonCode: 'history_gap' })
		sourceObserver?.onStreamState(state)
		sourceObserver?.onReplayGap(gap)
		sourceObserver?.onProtocolError()

		expect(observer.onStreamState).toHaveBeenCalledWith(state)
		expect(observer.onReplayGap).toHaveBeenCalledWith(gap)
		expect(observer.onProtocolError).toHaveBeenCalledTimes(1)
		expect(closeSource).toHaveBeenCalledTimes(1)
		vi.advanceTimersByTime(1_000)
		expect(subscribe).toHaveBeenCalledTimes(2)
		vi.useRealTimers()
	})

	it('replays the current stream state to a late consumer of the shared source', () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const subscribe = vi.fn((observer: BrowserGatewayRealtimeObserver) => {
			sourceObserver = observer
			return { close: vi.fn() }
		})
		const hub = new BrowserGatewayRealtimeHub({ subscribe })
		const first = observerFixture()
		const second = observerFixture()
		hub.subscribe(first)

		const state = create(ClientRealtimeStreamStateV1Schema, { cursor: 'cursor-3' })
		sourceObserver?.onStreamState(state)
		hub.subscribe(second)

		expect(subscribe).toHaveBeenCalledTimes(1)
		expect(second.onStreamState).toHaveBeenCalledTimes(1)
		expect(second.onStreamState).toHaveBeenCalledWith(state)
	})
})

function observerFixture(): BrowserGatewayRealtimeObserver {
	return {
		onEvent: vi.fn(),
		onStreamState: vi.fn(),
		onReplayGap: vi.fn(),
		onProtocolError: vi.fn(),
	}
}
