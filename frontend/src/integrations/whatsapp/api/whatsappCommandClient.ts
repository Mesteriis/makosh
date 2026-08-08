import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { WhatsAppCommandService } from '../../../gen/makosh/whatsapp/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof WhatsAppCommandService> | null = null

export function getWhatsAppCommandConnectClient(): Client<typeof WhatsAppCommandService> {
	if (!client) {
		client = createClient(WhatsAppCommandService, createBrowserGatewayConnectTransport())
	}
	return client
}

export function resetWhatsAppCommandConnectClientForTests(): void {
	client = null
}
