import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { ZulipOperationalRealtimeService } from '../../../gen/makosh/zulip/operational/realtime/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof ZulipOperationalRealtimeService> | null = null

export function getZulipOperationalRealtimeConnectClient(): Client<typeof ZulipOperationalRealtimeService> {
	if (!client) {
		client = createClient(
			ZulipOperationalRealtimeService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return client
}

export function resetZulipOperationalRealtimeConnectClientForTests(): void {
	client = null
}
