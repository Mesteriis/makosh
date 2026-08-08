import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { WhatsAppQueryService } from '../../../gen/makosh/whatsapp/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof WhatsAppQueryService> | null = null

export function getWhatsAppQueryConnectClient(): Client<typeof WhatsAppQueryService> {
	if (!client) {
		client = createClient(WhatsAppQueryService, createBrowserGatewayConnectTransport())
	}
	return client
}

export function resetWhatsAppQueryConnectClientForTests(): void {
	client = null
}
