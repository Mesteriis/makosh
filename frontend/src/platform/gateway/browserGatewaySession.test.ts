import { create } from '@bufbuild/protobuf'
import type { Client } from '@connectrpc/connect'
import { describe, expect, it, vi } from 'vitest'

import {
	BrowserGatewayAccessModeV1,
	BrowserSessionService,
	BrowserSessionStatusResponseV1Schema,
} from '../../gen/makosh/gateway/v1/browser_session_pb'
import { fetchBrowserGatewaySessionStatus } from './browserGatewaySession'

describe('fetchBrowserGatewaySessionStatus', () => {
	it('accepts the explicit LAN development access mode', async () => {
		const client = {
			getStatus: vi.fn().mockResolvedValue(create(BrowserSessionStatusResponseV1Schema, {
				major: 1,
				sessionExpiresAtUnixMillis: 100n,
				accessMode: BrowserGatewayAccessModeV1.LAN_DEVELOPMENT,
			})),
		} as unknown as Client<typeof BrowserSessionService>

		await expect(fetchBrowserGatewaySessionStatus(client)).resolves.toEqual({
			accessMode: BrowserGatewayAccessModeV1.LAN_DEVELOPMENT,
			expiresAtUnixMillis: 100n,
		})
	})

	it('accepts the process-local full-stack development access mode', async () => {
		const client = {
			getStatus: vi.fn().mockResolvedValue(create(BrowserSessionStatusResponseV1Schema, {
				major: 1,
				sessionExpiresAtUnixMillis: 100n,
				accessMode: BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT,
			})),
		} as unknown as Client<typeof BrowserSessionService>

		await expect(fetchBrowserGatewaySessionStatus(client)).resolves.toEqual({
			accessMode: BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT,
			expiresAtUnixMillis: 100n,
		})
	})

	it('fails closed for an unknown access mode', async () => {
		const client = {
			getStatus: vi.fn().mockResolvedValue(create(BrowserSessionStatusResponseV1Schema, {
				major: 1,
				accessMode: 99 as never,
			})),
		} as unknown as Client<typeof BrowserSessionService>

		await expect(fetchBrowserGatewaySessionStatus(client)).rejects.toThrow('Unsupported browser Gateway session contract')
	})
})
