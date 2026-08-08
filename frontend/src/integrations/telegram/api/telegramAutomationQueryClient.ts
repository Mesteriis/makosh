import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { TelegramAutomationQueryService } from '../../../gen/makosh/telegram/automation/v1/automation_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let queryClient: Client<typeof TelegramAutomationQueryService> | null = null

export function getTelegramAutomationQueryClient(): Client<typeof TelegramAutomationQueryService> {
	if (!queryClient) {
		queryClient = createClient(
			TelegramAutomationQueryService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return queryClient
}

export function resetTelegramAutomationQueryClientForTests(): void {
	queryClient = null
}
