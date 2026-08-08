import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { MailDeliveryCommandService } from '../../../gen/makosh/mail/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailDeliveryCommandClient: Client<typeof MailDeliveryCommandService> | null = null

function createMailDeliveryCommandConnectClient(): Client<typeof MailDeliveryCommandService> {
	return createClient(MailDeliveryCommandService, createBrowserGatewayConnectTransport())
}

export function getMailDeliveryCommandConnectClient(): Client<typeof MailDeliveryCommandService> {
	if (!mailDeliveryCommandClient) {
		mailDeliveryCommandClient = createMailDeliveryCommandConnectClient()
	}

	return mailDeliveryCommandClient
}

export function resetMailDeliveryCommandConnectClientForTests(): void {
	mailDeliveryCommandClient = null
}
