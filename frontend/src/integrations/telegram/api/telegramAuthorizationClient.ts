import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { TelegramAuthorizationService } from '../../../gen/makosh/telegram/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let telegramAuthorizationClient: Client<typeof TelegramAuthorizationService> | null = null

function createTelegramAuthorizationConnectClient(): Client<typeof TelegramAuthorizationService> {
	return createClient(TelegramAuthorizationService, createBrowserGatewayConnectTransport())
}

export function getTelegramAuthorizationConnectClient(): Client<typeof TelegramAuthorizationService> {
	if (!telegramAuthorizationClient) {
		telegramAuthorizationClient = createTelegramAuthorizationConnectClient()
	}
	return telegramAuthorizationClient
}

export function resetTelegramAuthorizationConnectClientForTests(): void {
	telegramAuthorizationClient = null
}
