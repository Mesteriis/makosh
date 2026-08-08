import { createClient, type Client } from '@connectrpc/connect'

import { CommunicationsExportTicketService } from '../../gen/makosh/communications_export/v1/export_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof CommunicationsExportTicketService> | null = null

export function getCommunicationsExportTicketClient():
	Client<typeof CommunicationsExportTicketService> {
	client ??= createClient(
		CommunicationsExportTicketService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetCommunicationsExportTicketClientForTests(): void {
	client = null
}
