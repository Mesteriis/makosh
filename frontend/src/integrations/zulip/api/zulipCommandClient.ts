import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { ZulipCommandService } from '../../../gen/makosh/zulip/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let zulipCommandClient: Client<typeof ZulipCommandService> | null = null

function createZulipCommandConnectClient(): Client<typeof ZulipCommandService> {
	return createClient(ZulipCommandService, createBrowserGatewayConnectTransport())
}

export function getZulipCommandConnectClient(): Client<typeof ZulipCommandService> {
	if (!zulipCommandClient) {
		zulipCommandClient = createZulipCommandConnectClient()
	}

	return zulipCommandClient
}

export function resetZulipCommandConnectClientForTests(): void {
	zulipCommandClient = null
}
