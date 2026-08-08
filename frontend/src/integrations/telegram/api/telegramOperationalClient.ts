import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { TelegramOperationalService } from '../../../gen/makosh/telegram/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let telegramOperationalClient: Client<typeof TelegramOperationalService> | null = null

function createTelegramOperationalConnectClient(): Client<typeof TelegramOperationalService> {
	return createClient(TelegramOperationalService, createBrowserGatewayConnectTransport())
}

export function getTelegramOperationalConnectClient(): Client<typeof TelegramOperationalService> {
	if (!telegramOperationalClient) {
		telegramOperationalClient = createTelegramOperationalConnectClient()
	}

	return telegramOperationalClient
}

export function resetTelegramOperationalConnectClientForTests(): void {
	telegramOperationalClient = null
}
