import { create, toBinary } from '@bufbuild/protobuf'
import { describe, expect, it, vi } from 'vitest'

import {
	ClientRealtimeEventV1Schema,
	type ClientRealtimeEventV1,
} from '../../../gen/makosh/gateway/v1/client_realtime_pb'
import {
	AuthorizationStatusResponseSchema,
} from '../../../gen/makosh/telegram/v1/client_pb'
import type { BrowserGatewayRealtimeObserver } from '../../../platform/gateway/browserGatewayRealtime'
import { openTelegramAuthorizationRealtime } from './telegramAuthorizationRealtime'

describe('Telegram authorization realtime adapter', () => {
	it('uses the shared SSE signal without exposing provider QR material', () => {
		let sourceObserver: BrowserGatewayRealtimeObserver | undefined
		const close = vi.fn()
		const hub = {
			subscribe: vi.fn((observer: BrowserGatewayRealtimeObserver) => {
				sourceObserver = observer
				return { close }
			}),
		}
		const onStatusChanged = vi.fn()
		const onUnavailable = vi.fn()
		const binding = openTelegramAuthorizationRealtime(
			onStatusChanged,
			onUnavailable,
			hub as never,
		)

		sourceObserver?.onEvent(event({ state: 'waiting_qr_scan' }))
		expect(onStatusChanged).toHaveBeenCalledWith('waiting_qr_scan')

		sourceObserver?.onEvent(event({
			state: 'waiting_qr_scan',
			qrLink: 'tg://private-token',
		}))
		expect(onStatusChanged).toHaveBeenCalledTimes(1)
		binding.close()
		expect(close).toHaveBeenCalledOnce()
	})
})

function event(status: { state: string; qrLink?: string }): ClientRealtimeEventV1 {
	return create(ClientRealtimeEventV1Schema, {
		contractName: 'telegram.authorization.status_changed.v1',
		contractVersion: 1,
		eventKind: 'telegram.authorization.status_changed.v1',
		payload: toBinary(
			AuthorizationStatusResponseSchema,
			create(AuthorizationStatusResponseSchema, status),
		),
	})
}
