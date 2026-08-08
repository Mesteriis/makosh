import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { TelegramReconfigurationService } from '../../../gen/makosh/telegram/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let telegramReconfigurationClient: Client<typeof TelegramReconfigurationService> | null = null

export function getTelegramReconfigurationConnectClient(): Client<
	typeof TelegramReconfigurationService
> {
	if (!telegramReconfigurationClient) {
		telegramReconfigurationClient = createClient(
			TelegramReconfigurationService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return telegramReconfigurationClient
}

export function resetTelegramReconfigurationConnectClientForTests(): void {
	telegramReconfigurationClient = null
}
