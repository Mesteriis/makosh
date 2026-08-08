import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { ZulipOperationalQueryService } from '../../../gen/makosh/zulip/operational/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof ZulipOperationalQueryService> | null = null

export function getZulipOperationalReadConnectClient(): Client<typeof ZulipOperationalQueryService> {
	if (!client) {
		client = createClient(
			ZulipOperationalQueryService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return client
}

export function resetZulipOperationalReadConnectClientForTests(): void {
	client = null
}
