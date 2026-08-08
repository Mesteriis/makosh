import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { ZulipQueryService } from '../../../gen/makosh/zulip/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let zulipQueryClient: Client<typeof ZulipQueryService> | null = null

function createZulipQueryConnectClient(): Client<typeof ZulipQueryService> {
	return createClient(ZulipQueryService, createBrowserGatewayConnectTransport())
}

export function getZulipQueryConnectClient(): Client<typeof ZulipQueryService> {
	if (!zulipQueryClient) {
		zulipQueryClient = createZulipQueryConnectClient()
	}

	return zulipQueryClient
}

export function resetZulipQueryConnectClientForTests(): void {
	zulipQueryClient = null
}
