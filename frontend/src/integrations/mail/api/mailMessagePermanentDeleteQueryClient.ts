import { createClient, type Client } from '@connectrpc/connect'

import { MailMessagePermanentDeleteQueryService } from '../../../gen/makosh/mail/message_permanent_delete/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof MailMessagePermanentDeleteQueryService> | null = null

export function getMailMessagePermanentDeleteQueryConnectClient(): Client<typeof MailMessagePermanentDeleteQueryService> {
	if (!client) {
		client = createClient(
			MailMessagePermanentDeleteQueryService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return client
}

export function resetMailMessagePermanentDeleteQueryConnectClientForTests(): void {
	client = null
}
