import { createClient, type Client } from '@connectrpc/connect'

import { CommunicationsContentTicketService } from '../../gen/makosh/communications/content/ticket/v1/ticket_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let communicationsContentTicketClient:
	Client<typeof CommunicationsContentTicketService> | null = null

export function getCommunicationsContentTicketConnectClient():
	Client<typeof CommunicationsContentTicketService> {
	if (!communicationsContentTicketClient) {
		communicationsContentTicketClient = createClient(
			CommunicationsContentTicketService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return communicationsContentTicketClient
}

export function resetCommunicationsContentTicketConnectClientForTests(): void {
	communicationsContentTicketClient = null
}
