import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { MailDeliveryQueryService } from '../../../gen/makosh/mail/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailDeliveryQueryClient: Client<typeof MailDeliveryQueryService> | null = null

function createMailDeliveryQueryConnectClient(): Client<typeof MailDeliveryQueryService> {
	return createClient(MailDeliveryQueryService, createBrowserGatewayConnectTransport())
}

export function getMailDeliveryQueryConnectClient(): Client<typeof MailDeliveryQueryService> {
	if (!mailDeliveryQueryClient) {
		mailDeliveryQueryClient = createMailDeliveryQueryConnectClient()
	}

	return mailDeliveryQueryClient
}

export function resetMailDeliveryQueryConnectClientForTests(): void {
	mailDeliveryQueryClient = null
}
