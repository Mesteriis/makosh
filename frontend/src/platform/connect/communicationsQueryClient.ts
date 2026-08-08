import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { CommunicationsQueryService } from '../../gen/makosh/communications/query/v1/query_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let communicationsQueryClient: Client<typeof CommunicationsQueryService> | null = null

function createCommunicationsQueryConnectClient(): Client<typeof CommunicationsQueryService> {
	return createClient(CommunicationsQueryService, createBrowserGatewayConnectTransport())
}

/**
 * Owner-local, metadata-only Communications query client.
 *
 * This is deliberately separate from the legacy provider operational client:
 * it exposes only the clean-room owner contract admitted by the Core Gateway.
 */
export function getCommunicationsQueryConnectClient(): Client<typeof CommunicationsQueryService> {
	if (!communicationsQueryClient) {
		communicationsQueryClient = createCommunicationsQueryConnectClient()
	}

	return communicationsQueryClient
}

export function resetCommunicationsQueryConnectClientForTests(): void {
	communicationsQueryClient = null
}
