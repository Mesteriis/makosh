import { createClient, type Client } from '@connectrpc/connect'

import { MailCompositionQueryService } from '../../../gen/makosh/mail/composition/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailCompositionQueryClient: Client<typeof MailCompositionQueryService> | null = null

function createMailCompositionQueryConnectClient(): Client<typeof MailCompositionQueryService> {
	return createClient(
		MailCompositionQueryService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getMailCompositionQueryConnectClient(): Client<typeof MailCompositionQueryService> {
	if (!mailCompositionQueryClient) {
		mailCompositionQueryClient = createMailCompositionQueryConnectClient()
	}
	return mailCompositionQueryClient
}

export function resetMailCompositionQueryConnectClientForTests(): void {
	mailCompositionQueryClient = null
}
