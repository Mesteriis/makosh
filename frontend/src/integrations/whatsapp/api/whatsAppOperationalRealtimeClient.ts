import { createClient, type Client } from '@connectrpc/connect'

import { WhatsAppOperationalRealtimeService } from '../../../gen/makosh/whatsapp/operational/realtime/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof WhatsAppOperationalRealtimeService> | null = null

export function getWhatsAppOperationalRealtimeConnectClient(): Client<
	typeof WhatsAppOperationalRealtimeService
> {
	client ??= createClient(
		WhatsAppOperationalRealtimeService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetWhatsAppOperationalRealtimeConnectClientForTests(): void {
	client = null
}
