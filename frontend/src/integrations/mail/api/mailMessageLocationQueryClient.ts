import { createClient, type Client } from '@connectrpc/connect'

import { MailMessageLocationQueryService } from '../../../gen/makosh/mail/message_location/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof MailMessageLocationQueryService> | null = null

export function getMailMessageLocationQueryConnectClient(): Client<typeof MailMessageLocationQueryService> {
	if (!client) {
		client = createClient(
			MailMessageLocationQueryService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return client
}

export function resetMailMessageLocationQueryConnectClientForTests(): void {
	client = null
}
