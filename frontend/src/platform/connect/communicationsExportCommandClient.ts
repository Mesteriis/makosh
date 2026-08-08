import { createClient, type Client } from '@connectrpc/connect'

import { CommunicationsExportCommandService } from '../../gen/makosh/communications_export/v1/export_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof CommunicationsExportCommandService> | null = null

export function getCommunicationsExportCommandClient():
	Client<typeof CommunicationsExportCommandService> {
	client ??= createClient(
		CommunicationsExportCommandService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetCommunicationsExportCommandClientForTests(): void {
	client = null
}
