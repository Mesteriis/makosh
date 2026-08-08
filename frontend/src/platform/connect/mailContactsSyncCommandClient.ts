import { createClient, type Client } from '@connectrpc/connect'

import { MailContactsSyncCommandService } from '../../gen/makosh/mail_contacts_sync/v1/sync_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof MailContactsSyncCommandService> | null = null

export function getMailContactsSyncCommandClient(): Client<typeof MailContactsSyncCommandService> {
	client ??= createClient(
		MailContactsSyncCommandService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetMailContactsSyncCommandClientForTests(): void {
	client = null
}
