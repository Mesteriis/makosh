import { createClient, type Client } from '@connectrpc/connect'

import { MailMessageLocationCommandService } from '../../../gen/makosh/mail/message_location/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof MailMessageLocationCommandService> | null = null

export function getMailMessageLocationCommandConnectClient(): Client<typeof MailMessageLocationCommandService> {
	if (!client) {
		client = createClient(
			MailMessageLocationCommandService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return client
}

export function resetMailMessageLocationCommandConnectClientForTests(): void {
	client = null
}
