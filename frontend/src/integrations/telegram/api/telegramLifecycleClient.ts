import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { TelegramLifecycleService } from '../../../gen/makosh/telegram/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let telegramLifecycleClient: Client<typeof TelegramLifecycleService> | null = null

function createTelegramLifecycleConnectClient(): Client<typeof TelegramLifecycleService> {
	return createClient(TelegramLifecycleService, createBrowserGatewayConnectTransport())
}

export function getTelegramLifecycleConnectClient(): Client<typeof TelegramLifecycleService> {
	if (!telegramLifecycleClient) {
		telegramLifecycleClient = createTelegramLifecycleConnectClient()
	}
	return telegramLifecycleClient
}

export function resetTelegramLifecycleConnectClientForTests(): void {
	telegramLifecycleClient = null
}
