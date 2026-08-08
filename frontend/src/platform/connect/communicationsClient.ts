import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'
import { CommunicationsService } from '../../gen/makosh/communications/v1/communications_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let communicationsClient: Client<typeof CommunicationsService> | null = null

function createCommunicationsConnectClient(): Client<typeof CommunicationsService> {
	return createClient(CommunicationsService, createBrowserGatewayConnectTransport())
}

export function getCommunicationsConnectClient(): Client<typeof CommunicationsService> {
	if (!communicationsClient) {
		communicationsClient = createCommunicationsConnectClient()
	}

	return communicationsClient
}

export function resetCommunicationsConnectClientForTests(): void {
	communicationsClient = null
}
