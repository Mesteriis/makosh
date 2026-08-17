import { createClient, type Client } from '@connectrpc/connect'

import { TelegramRealtimeService } from '../../../gen/makosh/telegram/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway'

let telegramRealtimeClient: Client<typeof TelegramRealtimeService> | null = null

export function getTelegramRealtimeConnectClient(): Client<typeof TelegramRealtimeService> {
	telegramRealtimeClient ??= createClient(
		TelegramRealtimeService,
		createBrowserGatewayConnectTransport(),
	)
	return telegramRealtimeClient
}

export function resetTelegramRealtimeConnectClientForTests(): void {
	telegramRealtimeClient = null
}
