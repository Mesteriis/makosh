import { createClient } from '@connectrpc/connect'
import type { Client } from '@connectrpc/connect'

import { MailSyncService } from '../../../gen/makosh/mail/v1/client_pb'
import { createBrowserGatewayConnectTransport } from '../../../platform/gateway/browserGatewayConnect'

let mailSyncClient: Client<typeof MailSyncService> | null = null

function createMailSyncConnectClient(): Client<typeof MailSyncService> {
	return createClient(MailSyncService, createBrowserGatewayConnectTransport())
}

export function getMailSyncConnectClient(): Client<typeof MailSyncService> {
	if (!mailSyncClient) {
		mailSyncClient = createMailSyncConnectClient()
	}

	return mailSyncClient
}

export function resetMailSyncConnectClientForTests(): void {
	mailSyncClient = null
}
