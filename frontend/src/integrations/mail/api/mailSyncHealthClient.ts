import { createClient, type Client } from '@connectrpc/connect'

import { MailSyncHealthQueryService } from '../../../gen/makosh/mail/sync_health/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailSyncHealthClient: Client<typeof MailSyncHealthQueryService> | null = null

function createMailSyncHealthConnectClient(): Client<typeof MailSyncHealthQueryService> {
	return createClient(
		MailSyncHealthQueryService,
		createBrowserGatewayConnectTransport(),
	)
}

export function getMailSyncHealthConnectClient(): Client<typeof MailSyncHealthQueryService> {
	if (!mailSyncHealthClient) {
		mailSyncHealthClient = createMailSyncHealthConnectClient()
	}
	return mailSyncHealthClient
}

export function resetMailSyncHealthConnectClientForTests(): void {
	mailSyncHealthClient = null
}
