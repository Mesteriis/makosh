import { createClient, type Client } from '@connectrpc/connect'

import { MailContactsSyncQueryService } from '../../gen/makosh/mail_contacts_sync/v1/sync_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof MailContactsSyncQueryService> | null = null

export function getMailContactsSyncQueryClient(): Client<typeof MailContactsSyncQueryService> {
	client ??= createClient(
		MailContactsSyncQueryService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetMailContactsSyncQueryClientForTests(): void {
	client = null
}
