import { createClient, type Client } from '@connectrpc/connect'

import { MailCompositionCommandService } from '../../../gen/makosh/mail/composition/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailCompositionCommandClient: Client<typeof MailCompositionCommandService> | null = null

function createMailCompositionCommandConnectClient(): Client<typeof MailCompositionCommandService> {
	return createClient(
		MailCompositionCommandService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getMailCompositionCommandConnectClient(): Client<typeof MailCompositionCommandService> {
	if (!mailCompositionCommandClient) {
		mailCompositionCommandClient = createMailCompositionCommandConnectClient()
	}
	return mailCompositionCommandClient
}

export function resetMailCompositionCommandConnectClientForTests(): void {
	mailCompositionCommandClient = null
}
