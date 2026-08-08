import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { CommunicationsSenderInsightsService } from '../../gen/makosh/communications/sender_insights/v1/sender_insights_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let senderInsightsClient: Client<typeof CommunicationsSenderInsightsService> | null = null

function createCommunicationsSenderInsightsConnectClient():
	Client<typeof CommunicationsSenderInsightsService> {
	return createClient(
		CommunicationsSenderInsightsService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getCommunicationsSenderInsightsConnectClient():
	Client<typeof CommunicationsSenderInsightsService> {
	if (!senderInsightsClient) {
		senderInsightsClient = createCommunicationsSenderInsightsConnectClient()
	}
	return senderInsightsClient
}

export function resetCommunicationsSenderInsightsConnectClientForTests(): void {
	senderInsightsClient = null
}
