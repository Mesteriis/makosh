import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { TelegramAutomationCommandService } from '../../../gen/makosh/telegram/automation/v1/automation_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let commandClient: Client<typeof TelegramAutomationCommandService> | null = null

export function getTelegramAutomationCommandClient(): Client<typeof TelegramAutomationCommandService> {
	if (!commandClient) {
		commandClient = createClient(
			TelegramAutomationCommandService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return commandClient
}

export function resetTelegramAutomationCommandClientForTests(): void {
	commandClient = null
}
