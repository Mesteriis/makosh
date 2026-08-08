import { createClient, type Client } from '@connectrpc/connect'

import { MailMessageFlagCommandService } from '../../../gen/makosh/mail/message_flags/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailMessageFlagCommandClient: Client<typeof MailMessageFlagCommandService> | null = null

function createMailMessageFlagCommandConnectClient(): Client<typeof MailMessageFlagCommandService> {
	return createClient(
		MailMessageFlagCommandService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getMailMessageFlagCommandConnectClient(): Client<typeof MailMessageFlagCommandService> {
	if (!mailMessageFlagCommandClient) {
		mailMessageFlagCommandClient = createMailMessageFlagCommandConnectClient()
	}
	return mailMessageFlagCommandClient
}

export function resetMailMessageFlagCommandConnectClientForTests(): void {
	mailMessageFlagCommandClient = null
}
