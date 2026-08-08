import { createClient, type Client } from '@connectrpc/connect'

import { MailOperationalQueryService } from '../../../gen/makosh/mail/operational/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailOperationalQueryClient: Client<typeof MailOperationalQueryService> | null = null

function createMailOperationalQueryConnectClient(): Client<typeof MailOperationalQueryService> {
	return createClient(
		MailOperationalQueryService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getMailOperationalQueryConnectClient(): Client<typeof MailOperationalQueryService> {
	if (!mailOperationalQueryClient) {
		mailOperationalQueryClient = createMailOperationalQueryConnectClient()
	}
	return mailOperationalQueryClient
}

export function resetMailOperationalQueryConnectClientForTests(): void {
	mailOperationalQueryClient = null
}
