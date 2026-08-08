import { createClient, type Client } from '@connectrpc/connect'

import { WhatsAppOperationalQueryService } from '../../../gen/makosh/whatsapp/operational/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof WhatsAppOperationalQueryService> | null = null

export function getWhatsAppOperationalReadConnectClient(): Client<
	typeof WhatsAppOperationalQueryService
> {
	client ??= createClient(
		WhatsAppOperationalQueryService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetWhatsAppOperationalReadConnectClientForTests(): void {
	client = null
}
