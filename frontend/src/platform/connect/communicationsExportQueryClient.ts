import { createClient, type Client } from '@connectrpc/connect'

import { CommunicationsExportQueryService } from '../../gen/makosh/communications_export/v1/export_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof CommunicationsExportQueryService> | null = null

export function getCommunicationsExportQueryClient():
	Client<typeof CommunicationsExportQueryService> {
	client ??= createClient(
		CommunicationsExportQueryService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetCommunicationsExportQueryClientForTests(): void {
	client = null
}
