import { createClient, type Client } from '@connectrpc/connect'

import { MailMessagePermanentDeleteCommandService } from '../../../gen/makosh/mail/message_permanent_delete/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let client: Client<typeof MailMessagePermanentDeleteCommandService> | null = null

export function getMailMessagePermanentDeleteCommandConnectClient(): Client<typeof MailMessagePermanentDeleteCommandService> {
	if (!client) {
		client = createClient(
			MailMessagePermanentDeleteCommandService,
			createBrowserGatewayConnectTransport(),
		)
	}
	return client
}

export function resetMailMessagePermanentDeleteCommandConnectClientForTests(): void {
	client = null
}
