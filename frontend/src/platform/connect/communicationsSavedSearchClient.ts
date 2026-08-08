import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { CommunicationsSavedSearchService } from '../../gen/makosh/communications/saved_search/v1/saved_search_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let savedSearchClient: Client<typeof CommunicationsSavedSearchService> | null = null

function createCommunicationsSavedSearchConnectClient():
	Client<typeof CommunicationsSavedSearchService> {
	return createClient(
		CommunicationsSavedSearchService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getCommunicationsSavedSearchConnectClient():
	Client<typeof CommunicationsSavedSearchService> {
	if (!savedSearchClient) {
		savedSearchClient = createCommunicationsSavedSearchConnectClient()
	}
	return savedSearchClient
}

export function resetCommunicationsSavedSearchConnectClientForTests(): void {
	savedSearchClient = null
}
